//! Connects to a NatNet server, prints server and model details, then decodes
//! five frames by default. Replace the example addresses with your Motive
//! server and local interface IPv4 addresses:
//!
//! ```text
//! cargo run --example probe
//! cargo run --example probe -- --server 192.168.1.10
//! cargo run --example probe -- --server 192.168.1.10 --interface 192.168.1.20 --frames 20
//! ```
//!
//! Expected output starts with `server:` and `model:` lines, followed by one
//! `frame` line per decoded frame. Without `--interface`, the probe selects
//! all operational non-loopback IPv4 interfaces for multicast and uses the
//! server route for its command socket. The default frame count is 5, and each
//! frame wait is at most 2 seconds. Without `--server`, the probe listens for
//! up to 2 seconds total for an active NatNet multicast frame on 239.255.42.99:1511.
//! The sender's IPv4 address maps to command port 1510. Use `--interface` to select
//! the local interface, or `--server <ip[:port]>` for unicast/custom settings.
//! Use `--frames 0` to print only server and model details.
//!
//! Frames come from the multicast group the server announces, or, for a
//! unicast server, on the command socket, which then sends NAT_KEEPALIVE
//! every 250 ms as the NatNet SDK sample client does. `--interface` selects
//! the local interface for both sockets. `0.0.0.0` keeps the operating
//! system's default-interface behavior.

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use natnet::command;
use natnet::net::{CommandClient, DataClient, MIN_DATA_BUFFER};
use natnet::{DataSet, FrameOfData, MessageHeader, ModelDef, NAT_FRAMEOFDATA, ServerInfo, Version};

const TIMEOUT: Duration = Duration::from_secs(2);
const KEEPALIVE_INTERVAL: Duration = Duration::from_millis(250);
const COMMAND_PORT: u16 = 1510;
const DATA_PORT: u16 = 1511;
const MULTICAST_GROUP: Ipv4Addr = Ipv4Addr::new(239, 255, 42, 99);

struct Options {
    server: Option<SocketAddrV4>,
    interface: Option<Ipv4Addr>,
    frames: usize,
}

fn main() -> ExitCode {
    let options = match parse_args(std::env::args().skip(1)) {
        Ok(Some(options)) => options,
        Ok(None) => {
            println!(
                "usage: probe [--server <ip[:port]>] [--interface <ip>] [--frames <n>]\n\
                 Without --server, discovers an active multicast sender on\n\
                 239.255.42.99:{DATA_PORT} for up to 2 seconds; command port is {COMMAND_PORT}.\n\
                 Use --server for unicast or custom multicast settings.\n\
                 Defaults: automatic interfaces, 5 frames, 2-second waits.\n\
                 Explicit --interface 0.0.0.0 keeps the OS default interface.\n\
                 Prints server/model details, then one summary per decoded frame;\n\
                 --frames 0 prints only server/model details."
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
    let server = resolve_server(options)?;
    let interface = options
        .interface
        .map_or_else(|| route_interface(server), Ok)?;
    eprintln!("command interface: {interface}");
    let mut client =
        CommandClient::bind(interface, server).map_err(|e| format!("bind command socket: {e}"))?;
    let info = client
        .connect_handshake(TIMEOUT)
        .map_err(|e| format!("NAT_CONNECT: {e}"))?;
    print_server_info(&info);

    let request = command::request_modeldef().map_err(|e| format!("{e:?}"))?;
    let reply = client
        .exchange(&request, TIMEOUT)
        .map_err(|e| format!("NAT_REQUEST_MODELDEF: {e}"))?;
    let model =
        ModelDef::parse(&reply, info.protocol_version).map_err(|e| format!("MODELDEF: {e}"))?;
    print_model(&model);

    if options.frames == 0 {
        return Ok(());
    }
    if info.connection.multicast {
        let data = bind_multicast_data(
            options.interface,
            info.multicast_group(),
            info.connection.data_port,
        )
        .map_err(|e| format!("join {}: {e}", info.multicast_group()))?;
        let mut buf = vec![0u8; MIN_DATA_BUFFER];
        let mut seen = 0;
        let mut last_frame = Instant::now();
        while seen < options.frames {
            // Each receive waits only for the time left, so datagrams that
            // are not frames cannot extend the wait for one.
            let left = TIMEOUT.saturating_sub(last_frame.elapsed());
            if left.is_zero() {
                return Err("no multicast frame within the timeout".into());
            }
            data.set_read_timeout(Some(left))
                .map_err(|e| e.to_string())?;
            match data.recv(&mut buf) {
                Ok((len, source)) => {
                    if !is_server_source(source, server) {
                        continue;
                    }
                    let found = print_frame(&buf[..len], info.protocol_version)?;
                    if found > 0 {
                        last_frame = Instant::now();
                    }
                    seen += found;
                }
                // A silent group times out here; the next pass reports it.
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) => {}
                Err(e) => return Err(format!("multicast receive: {e}")),
            }
        }
    } else {
        // A unicast server streams to the command socket that connected, for
        // as long as that socket keeps sending keepalives.
        let raw = client.into_raw();
        let mut buf = vec![0u8; MIN_DATA_BUFFER];
        let mut seen = 0;
        let mut last_keepalive: Option<Instant> = None;
        let mut last_frame = Instant::now();
        while seen < options.frames {
            // As in multicast mode, and a receive also ends in time for the
            // next keepalive.
            let left = TIMEOUT.saturating_sub(last_frame.elapsed());
            if left.is_zero() {
                return Err("no unicast frame within the timeout".into());
            }
            if last_keepalive.is_none_or(|sent| sent.elapsed() >= KEEPALIVE_INTERVAL) {
                raw.send(&command::keepalive_datagram())
                    .map_err(|e| format!("keepalive: {e}"))?;
                last_keepalive = Some(Instant::now());
            }
            raw.set_read_timeout(Some(left.min(KEEPALIVE_INTERVAL)))
                .map_err(|e| e.to_string())?;
            match raw.recv(&mut buf) {
                Ok((len, source)) => {
                    if !is_server_source(source, server) {
                        continue;
                    }
                    let found = print_frame(&buf[..len], info.protocol_version)?;
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
        }
        raw.send(&command::disconnect_datagram())
            .map_err(|e| format!("disconnect: {e}"))?;
    }
    Ok(())
}

fn resolve_server(options: &Options) -> Result<SocketAddrV4, String> {
    let Some(server) = options.server else {
        let deadline = Instant::now() + TIMEOUT;
        let data = bind_multicast_data(options.interface, MULTICAST_GROUP, DATA_PORT)
            .map_err(|e| format!("discovery multicast {MULTICAST_GROUP}:{DATA_PORT}: {e}"))?;
        let discovered = discover_server(&data, deadline).map_err(|e| {
            format!(
                "{e}; ensure an active NatNet multicast stream is available on {MULTICAST_GROUP}:{DATA_PORT}, or specify --server for unicast/custom settings"
            )
        })?;
        drop(data);
        eprintln!("discovered server: {discovered}");
        return Ok(discovered);
    };
    Ok(server)
}

fn bind_multicast_data(
    interface: Option<Ipv4Addr>,
    group: Ipv4Addr,
    port: u16,
) -> Result<DataClient, String> {
    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port))
        .map_err(|e| format!("bind wildcard data socket: {e}"))?;
    let interfaces = candidate_interfaces(interface, eligible_interfaces)
        .map_err(|e| format!("enumerate local IPv4 interfaces: {e}; use --interface"))?;
    join_interfaces(&interfaces, |interface| {
        socket.join_multicast_v4(&group, &interface)
    })?;
    Ok(DataClient::from_socket(socket))
}

fn candidate_interfaces(
    interface: Option<Ipv4Addr>,
    enumerate: impl FnOnce() -> io::Result<Vec<Ipv4Addr>>,
) -> io::Result<Vec<Ipv4Addr>> {
    interface.map_or_else(enumerate, |interface| Ok(vec![interface]))
}

fn join_interfaces(
    interfaces: &[Ipv4Addr],
    mut join: impl FnMut(Ipv4Addr) -> io::Result<()>,
) -> Result<(), String> {
    if interfaces.is_empty() {
        return Err("no operational non-loopback IPv4 interfaces found; use --interface".into());
    }
    let mut joined = false;
    let mut failures = Vec::new();
    for interface in interfaces {
        match join(*interface) {
            Ok(()) => joined = true,
            Err(error) => failures.push(format!("{interface}: {error}")),
        }
    }
    if !joined {
        return Err(format!(
            "could not join multicast on any candidate interface ({})",
            failures.join(", ")
        ));
    }
    Ok(())
}

#[cfg(any(unix, windows))]
fn eligible_interfaces() -> io::Result<Vec<Ipv4Addr>> {
    let interfaces = if_addrs::get_if_addrs()?;
    Ok(filter_interfaces(interfaces.into_iter().map(|interface| {
        (interface.is_oper_up(), interface.addr.ip())
    })))
}

#[cfg(any(unix, windows, test))]
fn filter_interfaces(interfaces: impl IntoIterator<Item = (bool, IpAddr)>) -> Vec<Ipv4Addr> {
    let mut addresses = interfaces
        .into_iter()
        .filter_map(|(eligible, address)| match (eligible, address) {
            (true, IpAddr::V4(address))
                if !address.is_loopback()
                    && !address.is_unspecified()
                    && !address.is_broadcast()
                    && !address.is_multicast() =>
            {
                Some(address)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    addresses.sort_unstable();
    addresses.dedup();
    addresses
}

#[cfg(not(any(unix, windows)))]
fn eligible_interfaces() -> io::Result<Vec<Ipv4Addr>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "automatic interface enumeration is unsupported on this target",
    ))
}

fn route_interface(server: SocketAddrV4) -> Result<Ipv4Addr, String> {
    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
        .map_err(|e| format!("bind route-selection socket: {e}"))?;
    socket
        .connect(server)
        .map_err(|e| format!("select command route to {server}: {e}"))?;
    match socket
        .local_addr()
        .map_err(|e| format!("read command route address: {e}"))?
    {
        SocketAddr::V4(address) => Ok(*address.ip()),
        SocketAddr::V6(_) => {
            Err("command route selected an IPv6 address for an IPv4 server".into())
        }
    }
}

fn discover_server(data: &DataClient, deadline: Instant) -> Result<SocketAddrV4, String> {
    let mut buf = vec![0u8; MIN_DATA_BUFFER];
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err("no eligible NatNet multicast frame within the discovery timeout".into());
        }
        data.set_read_timeout(Some(left))
            .map_err(|e| format!("set discovery timeout: {e}"))?;
        match data.recv(&mut buf) {
            Ok((len, source)) => {
                if let Some(server) = discovery_candidate(&buf[..len], source) {
                    return Ok(server);
                }
            }
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(e) => return Err(format!("multicast discovery receive: {e}")),
        }
    }
}

fn discovery_candidate(datagram: &[u8], source: SocketAddr) -> Option<SocketAddrV4> {
    let header = MessageHeader::parse(datagram).ok()?;
    if header.message_id != NAT_FRAMEOFDATA || datagram.len() < 8 {
        return None;
    }
    let SocketAddr::V4(source) = source else {
        return None;
    };
    Some(SocketAddrV4::new(*source.ip(), COMMAND_PORT))
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

fn is_server_source(source: SocketAddr, server: SocketAddrV4) -> bool {
    source.ip() == IpAddr::V4(*server.ip())
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
///
/// A FrameOfData that does not parse, or whose skeleton, asset, force-plate or
/// device section does not decode, is an error rather than a counted frame, so
/// a probe never succeeds on malformed input.
fn print_frame(datagram: &[u8], version: Version) -> Result<usize, String> {
    if datagram.get(..2) != Some(&NAT_FRAMEOFDATA.to_le_bytes()[..]) {
        return Ok(0);
    }
    let frame =
        FrameOfData::parse(datagram, version).map_err(|e| format!("frame decode error: {e}"))?;
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
    let skeletons = frame
        .decode_skeletons()
        .map_err(|e| format!("frame {}: skeleton decode error: {e}", frame.frame_number))?;
    for skeleton in skeletons {
        println!(
            "  skeleton {}: {} bones",
            skeleton.id,
            skeleton.rigid_bodies.len()
        );
    }
    // Compatible parsing keeps these sections raw; decode them so their
    // contents are validated too.
    frame
        .decode_assets()
        .map_err(|e| format!("frame {}: asset decode error: {e}", frame.frame_number))?;
    frame.decode_force_plates().map_err(|e| {
        format!(
            "frame {}: force plate decode error: {e}",
            frame.frame_number
        )
    })?;
    frame
        .decode_devices()
        .map_err(|e| format!("frame {}: device decode error: {e}", frame.frame_number))?;
    Ok(1)
}

fn parse_args(mut args: impl Iterator<Item = String>) -> Result<Option<Options>, String> {
    let mut server = None;
    let mut interface = None;
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
                interface = Some(
                    text.parse()
                        .map_err(|e| format!("--interface {text}: {e}"))?,
                );
            }
            "--frames" => {
                let text = value()?;
                frames = text.parse().map_err(|e| format!("--frames {text}: {e}"))?;
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }
    Ok(Some(Options {
        server,
        interface,
        frames,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::UdpSocket;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::thread;

    #[test]
    fn unicast_source_filter_uses_server_ip_and_ignores_source_port() {
        let server = SocketAddrV4::new(Ipv4Addr::LOCALHOST, COMMAND_PORT);
        assert!(is_server_source(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 42_000),
            server,
        ));
        assert!(!is_server_source(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)), COMMAND_PORT),
            server,
        ));
    }

    #[test]
    fn omitted_server_selects_discovery_defaults() {
        let options = parse_args(std::iter::empty()).unwrap().unwrap();
        assert_eq!(options.server, None);
        assert_eq!(options.interface, None);
        assert_eq!(options.frames, 5);
    }

    #[test]
    fn explicit_unspecified_interface_is_preserved() {
        let options = parse_args(args(&["--interface", "0.0.0.0"]))
            .unwrap()
            .unwrap();
        assert_eq!(options.interface, Some(Ipv4Addr::UNSPECIFIED));
    }

    #[test]
    fn explicit_server_addresses_and_argument_errors_are_preserved() {
        let ip_only = parse_args(args(&["--server", "192.0.2.4"]))
            .unwrap()
            .unwrap();
        assert_eq!(ip_only.server, Some("192.0.2.4:1510".parse().unwrap()));

        let with_port = parse_args(args(&["--server", "192.0.2.4:2400"]))
            .unwrap()
            .unwrap();
        assert_eq!(with_port.server, Some("192.0.2.4:2400".parse().unwrap()));

        let configured = parse_args(args(&["--interface", "192.0.2.5", "--frames", "0"]))
            .unwrap()
            .unwrap();
        assert_eq!(configured.server, None);
        assert_eq!(configured.interface, Some(Ipv4Addr::new(192, 0, 2, 5)));
        assert_eq!(configured.frames, 0);
        assert!(parse_args(args(&["--server", "bad"])).is_err());
        assert!(parse_args(args(&["--frames", "many"])).is_err());
        assert!(parse_args(args(&["--other"])).is_err());
        assert!(parse_args(args(&["--help"])).unwrap().is_none());
    }

    #[test]
    fn discovery_candidates_require_frame_header_and_prefix() {
        let ipv4 = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 42_000));
        let frame = [7, 0, 0, 0, 1, 0, 0, 0];
        assert_eq!(
            discovery_candidate(&frame, ipv4),
            Some(SocketAddrV4::new(Ipv4Addr::LOCALHOST, COMMAND_PORT))
        );
        assert_eq!(discovery_candidate(&frame[..7], ipv4), None);
        assert_eq!(discovery_candidate(&[7, 0, 0, 0], ipv4), None);
        assert_eq!(discovery_candidate(&[1, 0, 0, 0, 1, 0, 0, 0], ipv4), None);
        assert_eq!(
            discovery_candidate(&frame, SocketAddr::V6("[::1]:42000".parse().unwrap())),
            None
        );
    }

    #[test]
    fn interface_filter_sorts_deduplicates_and_keeps_link_local() {
        let private = IpAddr::V4(Ipv4Addr::new(192, 0, 2, 8));
        let link_local = IpAddr::V4(Ipv4Addr::new(169, 254, 1, 2));
        let addresses = filter_interfaces([
            (true, private),
            (true, link_local),
            (true, private),
            (false, IpAddr::V4(Ipv4Addr::new(192, 0, 2, 9))),
            (true, IpAddr::V4(Ipv4Addr::LOCALHOST)),
            (true, IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            (true, IpAddr::V4(Ipv4Addr::BROADCAST)),
            (true, IpAddr::V4(Ipv4Addr::new(239, 1, 2, 3))),
            (true, IpAddr::V6("fe80::1".parse().unwrap())),
        ]);
        assert_eq!(
            addresses,
            [Ipv4Addr::new(169, 254, 1, 2), Ipv4Addr::new(192, 0, 2, 8)]
        );
    }

    #[test]
    fn explicit_interface_skips_enumeration() {
        let interface = Ipv4Addr::new(192, 0, 2, 10);
        let candidates = candidate_interfaces(Some(interface), || {
            Err(io::Error::other("should not enumerate"))
        })
        .unwrap();
        assert_eq!(candidates, [interface]);
    }

    #[test]
    fn multicast_join_keeps_successes_and_reports_all_failures() {
        let first = Ipv4Addr::new(192, 0, 2, 10);
        let second = Ipv4Addr::new(192, 0, 2, 11);
        let mut attempted = Vec::new();
        join_interfaces(&[first, second], |interface| {
            attempted.push(interface);
            if interface == first {
                Err(io::Error::other("unavailable"))
            } else {
                Ok(())
            }
        })
        .unwrap();
        assert_eq!(attempted, [first, second]);

        let mut attempted = Vec::new();
        let error = join_interfaces(&[first, second], |interface| {
            attempted.push(interface);
            Err(io::Error::other("unavailable"))
        })
        .unwrap_err();
        assert_eq!(attempted, [first, second]);
        assert!(error.contains(&first.to_string()));
        assert!(error.contains(&second.to_string()));
        assert!(
            join_interfaces(&[], |_| Ok(()))
                .unwrap_err()
                .contains("no operational")
        );
    }

    #[test]
    fn route_selection_uses_loopback_for_local_server() {
        assert_eq!(
            route_interface(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1510)).unwrap(),
            Ipv4Addr::LOCALHOST
        );
    }

    #[test]
    fn discovery_uses_udp_sender_and_command_port() {
        let receiver = DataClient::bind_unicast(Ipv4Addr::LOCALHOST, 0).unwrap();
        let destination = receiver.local_addr().unwrap();
        let sender = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        let thread = thread::spawn(move || {
            let noise = [7, 0, 0, 0, 1, 0, 0];
            sender.send_to(&noise, destination).unwrap();
            sender
                .send_to(&[8, 0, 0, 0, 1, 0, 0, 0], destination)
                .unwrap();
            // Declared payload size is telemetry only; the frame number is present.
            sender
                .send_to(&[7, 0, 0, 0, 42, 0, 0, 0], destination)
                .unwrap();
        });
        let server = discover_server(&receiver, Instant::now() + Duration::from_secs(1)).unwrap();
        thread.join().unwrap();
        assert_eq!(server, SocketAddrV4::new(Ipv4Addr::LOCALHOST, COMMAND_PORT));
        drop(receiver);
        DataClient::bind_unicast(Ipv4Addr::LOCALHOST, destination.port()).unwrap();
    }

    #[test]
    fn silent_discovery_obeys_deadline() {
        let receiver = DataClient::bind_unicast(Ipv4Addr::LOCALHOST, 0).unwrap();
        let start = Instant::now();
        let result = discover_server(&receiver, Instant::now() + Duration::from_millis(100));
        let elapsed = start.elapsed();
        assert!(result.is_err());
        assert!(elapsed >= Duration::from_millis(80));
        assert!(elapsed < Duration::from_millis(600));
    }

    #[test]
    fn expired_discovery_deadline_does_not_wait() {
        let receiver = DataClient::bind_unicast(Ipv4Addr::LOCALHOST, 0).unwrap();
        let result = discover_server(&receiver, Instant::now());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timeout"));
    }

    #[test]
    fn continuous_irrelevant_packets_do_not_extend_discovery_deadline() {
        let receiver = DataClient::bind_unicast(Ipv4Addr::LOCALHOST, 0).unwrap();
        let destination = receiver.local_addr().unwrap();
        let sender = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        let noise_lifetime = Duration::from_millis(400);
        let noise_finished = Arc::new(AtomicBool::new(false));
        let sent = Arc::new(AtomicUsize::new(0));
        let sender_finished = Arc::clone(&noise_finished);
        let sender_count = Arc::clone(&sent);
        let thread = thread::spawn(move || {
            let noise_deadline = Instant::now() + noise_lifetime;
            while Instant::now() < noise_deadline {
                if sender
                    .send_to(&[8, 0, 0, 0, 1, 0, 0, 0], destination)
                    .is_ok()
                {
                    sender_count.fetch_add(1, Ordering::Relaxed);
                }
                thread::sleep(Duration::from_millis(5));
            }
            sender_finished.store(true, Ordering::Release);
        });
        let start = Instant::now();
        let result = discover_server(&receiver, Instant::now() + Duration::from_millis(120));
        let elapsed = start.elapsed();
        let noise_was_active_on_return = !noise_finished.load(Ordering::Acquire);
        thread.join().unwrap();
        assert!(result.is_err());
        assert!(sent.load(Ordering::Relaxed) >= 5);
        assert!(elapsed >= Duration::from_millis(100));
        assert!(elapsed < noise_lifetime);
        assert!(noise_was_active_on_return);
    }

    fn args(values: &[&str]) -> impl Iterator<Item = String> {
        values
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>()
            .into_iter()
    }
}
