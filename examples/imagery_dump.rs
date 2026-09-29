//! Prints one line per AUGI/1 record from a `motive-bridge` (or from the
//! `mock_bridge` binary in that repo, which needs no hardware).
//!
//! ```text
//! cargo run --features imagery,net --example imagery_dump -- --connect 127.0.0.1:1516
//! cargo run --features imagery,net --example imagery_dump -- --listen 0.0.0.0:1516
//! ```
//!
//! `--connect` dials the bridge; `--listen` waits for a bridge running in
//! reverse (`connect_to`) mode. `--interface <ip>` names the local interface
//! the outbound connection must leave by. `--max <n>` stops after `n`
//! records. Omitted `--max` is unlimited. Connect and record operations use
//! finite positive defaults of 5 seconds and 30 seconds; override them with
//! `--connect-timeout-ms` and `--record-timeout-ms`.
//! Reverse mode trusts the first accepted peer; the operator owns listener
//! exposure and session validation.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener};
use std::process::ExitCode;
use std::time::Duration;

use natnet::imagery::net::ImageClient;
use natnet::imagery::{Record, parse_record};

const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_RECORD_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
struct Options {
    connect: Option<SocketAddrV4>,
    listen: Option<SocketAddr>,
    interface: Ipv4Addr,
    max: Option<usize>,
    connect_timeout: Duration,
    record_timeout: Duration,
}

enum ParsedArgs {
    Help,
    Options(Options),
}

fn main() -> ExitCode {
    let options = match parse_args(std::env::args().skip(1)) {
        Ok(ParsedArgs::Help) => {
            print_usage();
            return ExitCode::SUCCESS;
        }
        Ok(ParsedArgs::Options(options)) => options,
        Err(error) => {
            eprintln!("argument error: {error}");
            eprintln!("try --help for usage");
            return ExitCode::from(2);
        }
    };

    // A zero limit is a valid no-op. Keep it before every bind/connect call,
    // including reverse-mode listener setup.
    if options.max == Some(0) {
        return ExitCode::SUCCESS;
    }

    let client = match (options.connect, options.listen) {
        (Some(_), Some(_)) => {
            eprintln!("--connect and --listen are mutually exclusive");
            return ExitCode::from(2);
        }
        (None, Some(addr)) => {
            match TcpListener::bind(addr).and_then(|listener| listener.accept()) {
                Ok((stream, from)) => {
                    eprintln!("bridge dialled in from {from}");
                    match ImageClient::from_stream(stream, options.record_timeout) {
                        Ok(client) => client,
                        Err(error) => {
                            eprintln!("configure stream from {from}: {error}");
                            return ExitCode::FAILURE;
                        }
                    }
                }
                Err(error) => {
                    eprintln!("listen on {addr}: {error}");
                    return ExitCode::FAILURE;
                }
            }
        }
        (Some(server), None) => {
            match ImageClient::connect(options.interface, server, options.connect_timeout) {
                Ok(client) => {
                    eprintln!("connected to {server}");
                    client
                }
                Err(error) => {
                    eprintln!("connect to {server}: {error}");
                    return ExitCode::FAILURE;
                }
            }
        }
        (None, None) => {
            let server = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1516);
            match ImageClient::connect(options.interface, server, options.connect_timeout) {
                Ok(client) => {
                    eprintln!("connected to {server}");
                    client
                }
                Err(error) => {
                    eprintln!("connect to {server}: {error}");
                    return ExitCode::FAILURE;
                }
            }
        }
    };

    dump(client, options.max, options.record_timeout)
}

fn parse_args<I>(args: I) -> Result<ParsedArgs, String>
where
    I: IntoIterator<Item = String>,
{
    let mut connect = None;
    let mut listen = None;
    let mut interface = Ipv4Addr::UNSPECIFIED;
    let mut max = None;
    let mut connect_timeout = DEFAULT_CONNECT_TIMEOUT;
    let mut record_timeout = DEFAULT_RECORD_TIMEOUT;
    let mut help = false;

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--connect" => {
                let value = next_value(&mut args, "--connect")?;
                connect = Some(parse_connect_addr(&value)?);
            }
            "--listen" => {
                let value = next_value(&mut args, "--listen")?;
                listen = Some(
                    value
                        .parse()
                        .map_err(|error| format!("bad --listen address {value}: {error}"))?,
                );
            }
            "--interface" => {
                let value = next_value(&mut args, "--interface")?;
                interface = value
                    .parse()
                    .map_err(|error| format!("bad --interface address {value}: {error}"))?;
            }
            "--max" => {
                let value = next_value(&mut args, "--max")?;
                max = Some(
                    value
                        .parse()
                        .map_err(|error| format!("bad --max value {value}: {error}"))?,
                );
            }
            "--connect-timeout-ms" => {
                let value = next_value(&mut args, "--connect-timeout-ms")?;
                connect_timeout = parse_timeout(&value, "--connect-timeout-ms")?;
            }
            "--record-timeout-ms" => {
                let value = next_value(&mut args, "--record-timeout-ms")?;
                record_timeout = parse_timeout(&value, "--record-timeout-ms")?;
            }
            "-h" | "--help" => help = true,
            other => return Err(format!("unknown argument {other}")),
        }
    }

    if help {
        return Ok(ParsedArgs::Help);
    }
    if connect.is_some() && listen.is_some() {
        return Err("--connect and --listen are mutually exclusive".to_string());
    }
    Ok(ParsedArgs::Options(Options {
        connect,
        listen,
        interface,
        max,
        connect_timeout,
        record_timeout,
    }))
}

fn next_value<I>(args: &mut I, option: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    args.next()
        .ok_or_else(|| format!("missing value for {option}"))
}

fn parse_connect_addr(value: &str) -> Result<SocketAddrV4, String> {
    value
        .parse()
        .map_err(|error| format!("bad --connect address {value}: {error}"))
}

fn parse_timeout(value: &str, option: &str) -> Result<Duration, String> {
    let milliseconds = value
        .parse::<u64>()
        .map_err(|error| format!("bad {option} value {value}: {error}"))?;
    if milliseconds == 0 {
        return Err(format!("{option} must be greater than zero"));
    }
    let seconds = milliseconds / 1_000;
    let remainder = milliseconds % 1_000;
    Ok(Duration::new(seconds, remainder as u32 * 1_000_000))
}

fn print_usage() {
    println!("usage: imagery_dump [--connect IPV4:PORT | --listen ADDR]");
    println!("                    [--interface IPV4] [--max N]");
    println!("                    [--connect-timeout-ms N] [--record-timeout-ms N]");
    println!("defaults: connect timeout 5000 ms, record timeout 30000 ms");
    println!("--max 0 exits successfully without opening a socket");
}

fn dump(mut client: ImageClient, max: Option<usize>, record_timeout: Duration) -> ExitCode {
    let limit = max.unwrap_or(usize::MAX);
    let mut buf = Vec::new();
    for _ in 0..limit {
        match client.recv_record(&mut buf, record_timeout) {
            Ok(true) => {}
            Ok(false) => {
                eprintln!("stream closed");
                return ExitCode::SUCCESS;
            }
            Err(error) => {
                eprintln!("recv: {error}");
                return ExitCode::FAILURE;
            }
        }

        match parse_record(&buf) {
            Ok(Record::Hello(h)) => println!(
                "Hello   proto {}.{} clock {} Hz qpf {} natnet {}.{} motive build {} mode {} \
                 cameras {} id {:?} profile {:?} calibration {:?} \
                 (host wall {} — provenance only, never a key)",
                h.proto_major,
                h.proto_minor,
                h.clock_hz,
                h.qpf,
                h.natnet_major,
                h.natnet_minor,
                h.motive_build,
                h.mode,
                h.camera_count,
                h.bridge_id,
                h.profile,
                h.calibration,
                h.host_wall_unix_secs,
            ),
            Ok(Record::CameraDesc(d)) => println!(
                "Camera  index {} id {} serial {} {}x{} mode {} enc {:?} divisor {} solve {} \
                 grid {}x{} pos {:?} name {:?} first ray {:?}",
                d.camera_index,
                d.camera_id,
                d.serial,
                d.width,
                d.height,
                d.video_mode,
                d.encoding,
                d.frame_rate_divisor,
                d.contributes_to_solve,
                d.ray_grid_w,
                d.ray_grid_h,
                d.position,
                d.name,
                d.ray(0, 0),
            ),
            Ok(Record::ImageFrame(f)) => println!(
                "Image   frame {} cam {} {}x{} enc {:?} {} bytes grab {} api {:.6} mid {}",
                f.frame_number,
                f.camera_index,
                f.width,
                f.height,
                f.encoding,
                f.payload.len(),
                f.grab_ticks,
                f.api_timestamp_secs,
                match f.camera_mid_exposure {
                    Some(ticks) => format!("{ticks} ticks"),
                    None => "ABSENT (bridge NatNet subscription down)".to_string(),
                },
            ),
            Ok(Record::Heartbeat) => println!("Heartbeat"),
            Ok(Record::Bye) => {
                println!("Bye");
                return ExitCode::SUCCESS;
            }
            Ok(Record::Other {
                kind,
                header,
                payload,
            }) => println!(
                "Other   kind {kind} header {} B payload {} B",
                header.len(),
                payload.len()
            ),
            // `Record` is `#[non_exhaustive]`: unreachable today (an unknown
            // kind already lands in `Other`), but required so a future
            // variant does not break this example's build.
            Ok(other) => println!("Record  {other:?}"),
            Err(error) => {
                eprintln!("decode: {error}");
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}
