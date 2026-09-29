//! Subprocess coverage for the real `imagery_dump` example. The loopback
//! peers are synchronized by channel handshakes; no packet-boundary or sleep
//! assumptions are part of these checks. The example is built by the harness
//! itself, so this target also passes when run alone.

#![cfg(all(feature = "imagery", feature = "net"))]
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "the subprocess harness uses assertion-oriented failure paths"
)]

use std::env;
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::{OnceLock, mpsc};
use std::thread;
use std::time::Duration;

use natnet::imagery;

/// Builds `imagery_dump` once per test process and returns its path.
///
/// `cargo test --test imagery_cli` does not build examples, so the harness
/// runs the same Cargo (`$CARGO`) against this manifest, target directory,
/// target triple, profile and feature set that produced this test binary, and
/// takes the executable path from Cargo's JSON artifact report instead of
/// guessing it. An up-to-date example makes this a no-op build.
fn example_binary() -> &'static Path {
    static BINARY: OnceLock<PathBuf> = OnceLock::new();
    BINARY.get_or_init(build_example)
}

fn build_example() -> PathBuf {
    let test_executable = env::current_exe().expect("test executable location");
    let profile_dir = test_executable
        .parent()
        .and_then(Path::parent)
        .expect("Cargo test executable lives in a profile deps directory");
    let profile = match profile_dir
        .file_name()
        .and_then(|name| name.to_str())
        .expect("profile directory name")
    {
        "debug" => "dev",
        other => other,
    };
    let above_profile = profile_dir.parent().expect("target directory");
    // `<target-dir>/<profile>` or, with `--target`, `<target-dir>/<triple>/<profile>`;
    // Cargo marks the target directory itself with CACHEDIR.TAG.
    let (target_dir, triple) = if above_profile.join("CACHEDIR.TAG").is_file() {
        (above_profile, None)
    } else {
        (
            above_profile
                .parent()
                .expect("target directory above triple"),
            above_profile.file_name(),
        )
    };

    let mut features = vec!["imagery", "net"];
    if cfg!(feature = "serde") {
        features.push("serde");
    }
    if cfg!(feature = "nalgebra") {
        features.push("nalgebra");
    }
    let cargo = env::var_os("CARGO").unwrap_or_else(|| env!("CARGO").into());
    let mut command = Command::new(cargo);
    command
        .arg("build")
        .arg("--locked")
        .arg("--message-format=json-render-diagnostics")
        .args(["--example", "imagery_dump", "--profile", profile])
        .arg("--manifest-path")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .arg("--target-dir")
        .arg(target_dir)
        .args(["--no-default-features", "--features", &features.join(",")]);
    if let Some(triple) = triple {
        command.arg("--target").arg(triple);
    }
    let output = command
        .stderr(Stdio::inherit())
        .output()
        .expect("run cargo build for imagery_dump");
    assert!(
        output.status.success(),
        "cargo build --example imagery_dump failed"
    );
    executable_from_json(&String::from_utf8_lossy(&output.stdout))
        .expect("Cargo reported the imagery_dump executable")
}

/// The `executable` of the `imagery_dump` compiler artifact in Cargo's
/// line-delimited JSON messages. Only the escapes a path can contain here
/// (`\\` and `\"`) are decoded.
fn executable_from_json(messages: &str) -> Option<PathBuf> {
    const KEY: &str = "\"executable\":\"";
    messages
        .lines()
        .filter(|line| {
            line.contains("\"reason\":\"compiler-artifact\"")
                && line.contains("\"name\":\"imagery_dump\"")
        })
        .find_map(|line| {
            let mut path = String::new();
            let mut chars = line.split_once(KEY)?.1.chars();
            loop {
                match chars.next()? {
                    '"' => return Some(PathBuf::from(path)),
                    '\\' => path.push(chars.next()?),
                    other => path.push(other),
                }
            }
        })
}

fn run(args: &[&str]) -> Output {
    Command::new(example_binary())
        .args(args)
        .output()
        .expect("run imagery_dump example")
}

fn run_with_peer(bytes: Vec<u8>, args: &[&str]) -> (Output, thread::JoinHandle<()>) {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = match listener.local_addr().expect("listener address") {
        std::net::SocketAddr::V4(addr) => addr,
        other => panic!("expected v4 listener, got {other}"),
    };
    let (accepted_tx, accepted_rx) = mpsc::channel();
    let peer = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        accepted_tx.send(()).expect("accepted handshake");
        stream.write_all(&bytes).expect("write synthetic stream");
    });

    let address = addr.to_string();
    let mut command_args = vec!["--connect", address.as_str()];
    command_args.extend_from_slice(args);
    let output = run(&command_args);
    accepted_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("example connected to loopback peer");
    (output, peer)
}

#[test]
fn invalid_max_values_fail_before_socket_setup() {
    for value in ["garbage", "-1", "184467440737095516160"] {
        let output = run(&["--connect", "127.0.0.1:9", "--max", value]);
        assert_eq!(output.status.code(), Some(2), "value {value}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("bad --max"), "stderr: {stderr}");
        assert!(!stderr.contains("connect to"), "stderr: {stderr}");
    }
}

#[test]
fn missing_values_and_nonpositive_timeouts_are_usage_errors() {
    for args in [
        vec!["--max"],
        vec!["--connect-timeout-ms"],
        vec!["--record-timeout-ms"],
        vec!["--connect-timeout-ms", "0"],
        vec!["--record-timeout-ms", "-1"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2), "args {args:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("argument error"));
    }
}

#[test]
fn max_zero_exits_without_connecting() {
    let output = run(&["--connect", "127.0.0.1:9", "--max", "0"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn help_and_mode_conflict_are_strict() {
    let help = run(&["--help"]);
    assert_eq!(help.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&help.stdout).contains("--record-timeout-ms"));

    let conflict = run(&["--connect", "127.0.0.1:1516", "--listen", "127.0.0.1:1517"]);
    assert_eq!(conflict.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&conflict.stderr).contains("mutually exclusive"));
}

#[test]
fn clean_eof_is_success_and_partial_eof_is_failure() {
    let (clean, clean_peer) = run_with_peer(
        imagery::heartbeat_record(),
        &["--record-timeout-ms", "1000"],
    );
    assert_eq!(clean.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&clean.stdout)
            .matches("Heartbeat")
            .count(),
        1
    );
    assert!(String::from_utf8_lossy(&clean.stderr).contains("stream closed"));
    clean_peer.join().expect("clean peer");

    let (partial, partial_peer) = run_with_peer(b"AUGI".to_vec(), &["--record-timeout-ms", "1000"]);
    assert_eq!(partial.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&partial.stderr).contains("recv:"));
    partial_peer.join().expect("partial peer");

    let mut missing_payload = b"AUGI".to_vec();
    missing_payload.extend_from_slice(&3u16.to_le_bytes());
    missing_payload.extend_from_slice(&40u16.to_le_bytes());
    missing_payload.extend_from_slice(&1u32.to_le_bytes());
    missing_payload.extend_from_slice(&[0u8; 40]);
    let (truncated, truncated_peer) =
        run_with_peer(missing_payload, &["--record-timeout-ms", "1000"]);
    assert_eq!(truncated.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&truncated.stderr).contains("recv:"));
    truncated_peer.join().expect("truncated peer");
}

#[test]
fn max_one_stops_after_one_record() {
    let mut bytes = imagery::heartbeat_record();
    bytes.extend_from_slice(&imagery::bye_record());
    let (output, peer) = run_with_peer(bytes, &["--max", "1"]);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.matches("Heartbeat").count(), 1);
    assert!(!stdout.contains("Bye"));
    peer.join().expect("max peer");
}
