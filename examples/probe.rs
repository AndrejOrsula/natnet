//! Connects to a NatNet server, prints its SERVERINFO and model definitions,
//! then decodes a few frames.
//!
//! ```text
//! cargo run --features net --example probe -- --server 192.168.1.10
//! cargo run --features net --example probe -- --server 192.168.1.10 --interface 192.168.1.20 --frames 20
//! ```
//!
//! Frames come from the multicast group the server announces, or, for a
//! unicast server, on the command socket, which then sends NAT_KEEPALIVE
//! every 250 ms as the NatNet SDK sample client does. `--interface` selects
//! the local interface for both sockets (default: any); `--frames` (default 5)
//! bounds the run, and every wait times out after 2 seconds.

use std::net::{Ipv4Addr, SocketAddrV4};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use natnet::command;
use natnet::net::{CommandClient, DataClient, MIN_DATA_BUFFER};
use natnet::{DataSet, FrameOfData, ModelDef, NAT_FRAMEOFDATA, ServerInfo, Version};

const TIMEOUT: Duration = Duration::from_secs(2);
const KEEPALIVE_INTERVAL: Duration = Duration::from_millis(250);
const COMMAND_PORT: u16 = 1510;

struct Options {
    server: SocketAddrV4,
    interface: Ipv4Addr,
    frames: usize,
}

fn main() -> ExitCode {
    let options = match parse_args(std::env::args().skip(1)) {
        Ok(Some(options)) => options,
        Ok(None) => {
            println!(
                "usage: probe --server <ip[:port]> [--interface <ip>] [--frames <n>]\n\
                 The command port defaults to {COMMAND_PORT}."
            );
            return ExitCode::SUCCESS;
        }
        Err(error) => {
            eprintln!("argument error: {error}; try --help");
            return ExitCode::from(2);
        }
    };
    match run(&options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("probe failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(options: &Options) -> Result<(), String> {
    let mut client = CommandClient::bind(options.interface, options.server)
        .map_err(|e| format!("bind command socket: {e}"))?;
    let info = client
        .connect_handshake(TIMEOUT)
        .map_err(|e| format!("NAT_CONNECT: {e}"))?;
    print_server_info(&info);

    let request = command::request_modeldef().map_err(|e| format!("{e:?}"))?;
    let reply = client
        .exchange(&request, TIMEOUT)
        .map_err(|e| format!("NAT_REQUEST_MODELDEF: {e}"))?;
    let model = ModelDef::parse_version(&reply, info.protocol_version)
        .map_err(|e| format!("MODELDEF: {e}"))?;
    print_model(&model);

    if options.frames == 0 {
        return Ok(());
    }
    if info.connection.multicast {
        let data = DataClient::bind_multicast(
            options.interface,
            info.multicast_group(),
            info.connection.data_port,
        )
        .map_err(|e| format!("join {}: {e}", info.multicast_group()))?;
        data.set_read_timeout(Some(TIMEOUT))
            .map_err(|e| e.to_string())?;
        let mut buf = vec![0u8; MIN_DATA_BUFFER];
        let mut seen = 0;
        while seen < options.frames {
            let (len, _) = data
                .recv(&mut buf)
                .map_err(|e| format!("multicast receive: {e}"))?;
            seen += print_frame(&buf[..len], info.protocol_version);
        }
    } else {
        // A unicast server streams to the command socket that connected, for
        // as long as that socket keeps sending keepalives.
        let raw = client.into_raw();
        raw.set_read_timeout(Some(KEEPALIVE_INTERVAL))
            .map_err(|e| e.to_string())?;
        let mut buf = vec![0u8; MIN_DATA_BUFFER];
        let mut seen = 0;
        let mut last_keepalive: Option<Instant> = None;
        let mut last_frame = Instant::now();
        while seen < options.frames {
            if last_keepalive.is_none_or(|sent| sent.elapsed() >= KEEPALIVE_INTERVAL) {
                raw.send(&command::keepalive_datagram())
                    .map_err(|e| format!("keepalive: {e}"))?;
                last_keepalive = Some(Instant::now());
            }
            match raw.recv(&mut buf) {
                Ok((len, _)) => {
                    let found = print_frame(&buf[..len], info.protocol_version);
                    if found > 0 {
                        last_frame = Instant::now();
                    }
                    seen += found;
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) => {}
                Err(e) => return Err(format!("unicast receive: {e}")),
            }
            if last_frame.elapsed() >= TIMEOUT {
                return Err("no unicast frame within the timeout".into());
            }
        }
        raw.send(&command::disconnect_datagram())
            .map_err(|e| format!("disconnect: {e}"))?;
    }
    Ok(())
}

fn print_server_info(info: &ServerInfo) {
    println!(
        "server: {} {} streaming NatNet {} ({}, data port {}), clock {} Hz",
        info.application_name,
        info.server_version,
        info.protocol_version,
        if info.connection.multicast {
            format!("multicast {}", info.multicast_group())
        } else {
            "unicast".to_string()
        },
        info.connection.data_port,
        info.high_res_clock_frequency,
    );
}

fn print_model(model: &ModelDef) {
    println!("model: {} datasets", model.datasets.len());
    for dataset in &model.datasets {
        match dataset {
            DataSet::MarkerSet(set) => {
                println!(
                    "  marker set {:?}: {} markers",
                    set.name,
                    set.marker_names.len()
                );
            }
            DataSet::RigidBody(body) => {
                println!("  rigid body {:?}: id {}", body.name, body.id);
            }
            DataSet::Skeleton(skeleton) => println!(
                "  skeleton {:?}: id {}, {} bones",
                skeleton.name,
                skeleton.id,
                skeleton.rigid_bodies.len()
            ),
            DataSet::ForcePlate(plate) => println!(
                "  force plate {:?}: id {}, {} channels",
                plate.serial_number,
                plate.id,
                plate.channel_names.len()
            ),
            DataSet::Device(device) => println!(
                "  device {:?}: id {}, {} channels",
                device.name,
                device.id,
                device.channel_names.len()
            ),
            DataSet::Camera(camera) => println!("  camera {:?}", camera.name),
            DataSet::Asset(asset) => println!(
                "  asset {:?}: id {}, {} rigid bodies, {} markers",
                asset.name,
                asset.id,
                asset.rigid_bodies.len(),
                asset.markers.len()
            ),
            DataSet::Other { type_id, body } => {
                println!(
                    "  dataset type {type_id}: {} bytes kept verbatim",
                    body.len()
                );
            }
            other => println!("  dataset type {}", other.type_id()),
        }
    }
}

/// Prints a FrameOfData and returns 1, or returns 0 for any other message.
fn print_frame(datagram: &[u8], version: Version) -> usize {
    if datagram.get(..2) != Some(&NAT_FRAMEOFDATA.to_le_bytes()[..]) {
        return 0;
    }
    match FrameOfData::parse(datagram, version) {
        Ok(frame) => {
            let tracked = frame
                .rigid_bodies
                .iter()
                .filter(|body| body.tracking_valid())
                .count();
            println!(
                "frame {}: t={:.3} s, {} rigid bodies ({} tracked), {} labeled markers, \
                 {} skeletons, {} assets, {} force plates, {} devices",
                frame.frame_number,
                frame.timing.timestamp,
                frame.rigid_bodies.len(),
                tracked,
                frame.labeled_markers.len(),
                frame.skeletons.count,
                frame.assets.count,
                frame.force_plates.count,
                frame.devices.count,
            );
            match frame.decode_skeletons() {
                Ok(skeletons) => {
                    for skeleton in skeletons {
                        println!(
                            "  skeleton {}: {} bones",
                            skeleton.id,
                            skeleton.rigid_bodies.len()
                        );
                    }
                }
                Err(e) => println!("  skeleton decode error: {e}"),
            }
        }
        Err(e) => println!("frame decode error: {e}"),
    }
    1
}

fn parse_args(mut args: impl Iterator<Item = String>) -> Result<Option<Options>, String> {
    let mut server = None;
    let mut interface = Ipv4Addr::UNSPECIFIED;
    let mut frames = 5;
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "--server" => {
                let text = value()?;
                server = Some(match text.parse::<SocketAddrV4>() {
                    Ok(addr) => addr,
                    Err(_) => SocketAddrV4::new(
                        text.parse().map_err(|e| format!("--server {text}: {e}"))?,
                        COMMAND_PORT,
                    ),
                });
            }
            "--interface" => {
                let text = value()?;
                interface = text
                    .parse()
                    .map_err(|e| format!("--interface {text}: {e}"))?;
            }
            "--frames" => {
                let text = value()?;
                frames = text.parse().map_err(|e| format!("--frames {text}: {e}"))?;
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }
    let server = server.ok_or("--server is required")?;
    Ok(Some(Options {
        server,
        interface,
        frames,
    }))
}
