//! Bounded command transport coverage against loopback only.
//!
//! Every fake peer in this file binds `127.0.0.1`; no test contacts Motive or
//! sends a datagram to a lab interface. Raw transmission is exercised only on
//! the explicitly raw client. Safe operations use validated command requests.
#![cfg(feature = "net")]
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "the loopback integration harness uses assertion-oriented failure paths"
)]

use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use natnet::command::{self, CommandResponse, ConnectOptions, Tier1Write};
use natnet::net::{
    CommandClient, HandshakeError, MAX_IPV4_UDP_PAYLOAD, NAT_UNRECOGNIZED_REQUEST,
    Tier1ExecutionError, Tier1Phase,
};
use natnet::serverinfo::ConnectionInfo;
use natnet::{NAT_CONNECT, NAT_REQUEST, NAT_RESPONSE, ServerInfo, Version};

const LOOPBACK: Ipv4Addr = Ipv4Addr::LOCALHOST;
const COMMAND_TIMEOUT: Duration = Duration::from_millis(300);

fn v4(addr: SocketAddr) -> SocketAddrV4 {
    match addr {
        SocketAddr::V4(addr) => addr,
        SocketAddr::V6(addr) => panic!("expected IPv4 address, got {addr}"),
    }
}

fn sample_serverinfo(protocol_version: Version) -> ServerInfo {
    ServerInfo {
        application_name: "Motive".to_string(),
        server_version: Version::new(3, 1, 2, 1),
        protocol_version,
        high_res_clock_frequency: 10_000_000,
        connection: ConnectionInfo {
            data_port: 1511,
            multicast: true,
            multicast_group: [239, 255, 42, 99],
        },
    }
}

fn command_text(datagram: &[u8]) -> String {
    let payload = datagram.get(4..).unwrap_or_default();
    let end = payload
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(payload.len());
    String::from_utf8_lossy(&payload[..end]).into_owned()
}

// The shape Motive 3.1.2.1 gives a GetProperty reply: the value's bytes with
// no NUL, and the payload length as the declared size.
fn unterminated(value: &str) -> Vec<u8> {
    let mut datagram = NAT_RESPONSE.to_le_bytes().to_vec();
    datagram.extend_from_slice(
        &u16::try_from(value.len())
            .expect("short value")
            .to_le_bytes(),
    );
    datagram.extend_from_slice(value.as_bytes());
    datagram
}

fn property_request(property_len: usize) -> command::Request {
    command::get_property("n", &"x".repeat(property_len)).expect("bounded property request")
}

#[test]
fn safe_handshake_accepts_only_the_configured_peer() {
    let intended = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind intended peer");
    let intended_addr = v4(intended.local_addr().expect("intended address"));
    let foreign = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind foreign peer");
    assert_ne!(
        foreign.local_addr().expect("foreign address"),
        intended.local_addr().unwrap()
    );

    let info = sample_serverinfo(Version::V4_1);
    let reply = info.to_datagram().expect("encode SERVERINFO");
    let server = thread::spawn(move || {
        let mut buf = [0u8; 256];
        let (length, source) = intended.recv_from(&mut buf).expect("receive NAT_CONNECT");
        assert_eq!(u16::from_le_bytes([buf[0], buf[1]]), NAT_CONNECT);
        assert_eq!(length, command::connect_request().unwrap().as_bytes().len());
        intended
            .send_to(&reply, source)
            .expect("send intended reply");
    });

    let mut client = CommandClient::bind(LOOPBACK, intended_addr).expect("bind client");
    let client_addr = client.local_addr().expect("client address");
    foreign
        .send_to(
            &sample_serverinfo(Version::V4_1).to_datagram().unwrap(),
            client_addr,
        )
        .expect("send foreign reply");

    let got = client
        .connect_handshake(COMMAND_TIMEOUT)
        .expect("intended handshake");
    server.join().expect("intended peer thread");
    assert_eq!(got.application_name, "Motive");
    assert_eq!(got.protocol_version, Version::V4_1);
}

#[test]
fn handshake_with_a_requested_version_validates_that_version() {
    let server = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind server");
    let server_addr = v4(server.local_addr().expect("server address"));
    let reply = sample_serverinfo(Version::new(4, 2, 0, 0))
        .to_datagram()
        .expect("encode SERVERINFO");
    let peer = thread::spawn(move || {
        let mut payloads = Vec::new();
        let mut buf = [0u8; 512];
        for _ in 0..2 {
            let (length, source) = server.recv_from(&mut buf).expect("receive NAT_CONNECT");
            assert_eq!(u16::from_le_bytes([buf[0], buf[1]]), NAT_CONNECT);
            payloads.push(buf[4..length].to_vec());
            server.send_to(&reply, source).expect("send SERVERINFO");
        }
        payloads
    });

    let mut client = CommandClient::bind(LOOPBACK, server_addr).expect("bind client");
    // Without a request, the server's newer bitstream is refused.
    assert!(client.connect_handshake(COMMAND_TIMEOUT).is_err());
    // Requesting 4.1 from a NatNet 3+ server makes 4.1 the stream layout.
    let options = ConnectOptions::requesting(Version::V4_1);
    let info = client
        .connect_handshake_with(options, COMMAND_TIMEOUT)
        .expect("requested-version handshake");
    assert_eq!(info.protocol_version, Version::new(4, 2, 0, 0));
    assert_eq!(options.stream_version(&info), Version::V4_1);

    let payloads = peer.join().expect("server thread");
    assert!(payloads[0].is_empty(), "the default hello is unchanged");
    assert_eq!(ConnectOptions::parse(&payloads[1]), Some(options));
}

#[test]
fn unsupported_requested_version_is_reported_as_the_stream_version() {
    let server = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind server");
    let server_addr = v4(server.local_addr().expect("server address"));
    let reply = sample_serverinfo(Version::V4_1)
        .to_datagram()
        .expect("encode SERVERINFO");
    let peer = thread::spawn(move || {
        let mut buf = [0u8; 512];
        let (_, source) = server.recv_from(&mut buf).expect("receive NAT_CONNECT");
        server.send_to(&reply, source).expect("send SERVERINFO");
    });

    let mut client = CommandClient::bind(LOOPBACK, server_addr).expect("bind client");
    let error = client
        .connect_handshake_with(
            ConnectOptions::requesting(Version::new(4, 2, 0, 0)),
            COMMAND_TIMEOUT,
        )
        .expect_err("an unsupported requested layout must be refused");
    peer.join().expect("server thread");
    // The supported server version is kept; the failure names the request.
    let message = error.to_string();
    match error {
        HandshakeError::UnsupportedVersion { info, source } => {
            assert_eq!(info.protocol_version, Version::V4_1);
            assert!(
                matches!(
                    source,
                    natnet::Error::UnsupportedVersion { major: 4, minor: 2 }
                ),
                "{source:?}"
            );
        }
        other => panic!("expected an unsupported stream version, got {other:?}"),
    }
    assert!(
        message.starts_with(&format!(
            "unsupported stream protocol version (SERVERINFO reports {})",
            Version::V4_1
        )),
        "{message}"
    );
}

#[test]
fn foreign_only_reply_times_out_with_peer_filtering() {
    let intended = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind intended peer");
    let intended_addr = v4(intended.local_addr().expect("intended address"));
    let foreign = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind foreign peer");
    let mut client = CommandClient::bind(LOOPBACK, intended_addr).expect("bind client");
    foreign
        .send_to(
            &sample_serverinfo(Version::V4_1).to_datagram().unwrap(),
            client.local_addr().expect("client address"),
        )
        .expect("send foreign reply");

    let started = Instant::now();
    let error = client
        .connect_handshake(Duration::from_millis(80))
        .expect_err("foreign-only reply must not complete handshake");
    assert!(started.elapsed() < Duration::from_secs(1));
    match error {
        HandshakeError::Io(error) => {
            assert_eq!(
                error.kind(),
                io::ErrorKind::TimedOut,
                "a read-timeout expiry must be reported as TimedOut: {error:?}"
            );
        }
        other => panic!("foreign reply must result in I/O timeout, got {other:?}"),
    }
}

#[test]
fn unrelated_same_peer_flood_cannot_extend_the_deadline() {
    let server = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind peer");
    let server_addr = v4(server.local_addr().expect("peer address"));
    let peer = thread::spawn(move || {
        let mut request = [0u8; 256];
        let (_, source) = server.recv_from(&mut request).expect("receive request");
        // NAT_RESPONSE is unrelated to a NAT_CONNECT handshake. A large
        // queued batch exercises the absolute deadline without arbitrary
        // sleeps or packet-order assumptions.
        let unrelated = CommandResponse::status_datagram(0);
        for _ in 0..64 {
            server
                .send_to(&unrelated, source)
                .expect("send unrelated packet");
        }
    });
    let mut client = CommandClient::bind(LOOPBACK, server_addr).expect("bind client");
    let started = Instant::now();
    let error = client
        .connect_handshake(Duration::from_millis(90))
        .expect_err("unrelated same-peer packets must not complete handshake");
    let elapsed = started.elapsed();
    peer.join().expect("peer thread");
    assert!(
        elapsed < Duration::from_secs(1),
        "deadline was extended: {elapsed:?}"
    );
    assert!(
        matches!(error, HandshakeError::Io(ref error) if error.kind() == io::ErrorKind::TimedOut),
        "expected a TimedOut I/O error, got {error:?}"
    );
}

#[test]
fn malformed_expected_peer_reply_is_invalid_data() {
    let server = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind peer");
    let server_addr = v4(server.local_addr().expect("peer address"));
    let peer = thread::spawn(move || {
        let mut request = [0u8; 256];
        let (_, source) = server.recv_from(&mut request).expect("receive request");
        // The first two bytes name SERVERINFO, but the fixed header is short.
        server
            .send_to(&[1, 0], source)
            .expect("send malformed reply");
    });
    let mut client = CommandClient::bind(LOOPBACK, server_addr).expect("bind client");
    let error = client
        .connect_handshake(COMMAND_TIMEOUT)
        .expect_err("short expected reply must fail");
    peer.join().expect("peer thread");
    assert!(
        matches!(error, HandshakeError::Io(error) if error.kind() == io::ErrorKind::InvalidData)
    );
}

#[test]
fn unsupported_handshake_keeps_serverinfo_for_diagnostics() {
    let server = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind peer");
    let server_addr = v4(server.local_addr().expect("peer address"));
    let info = sample_serverinfo(Version::new(99, 0, 7, 2));
    let reply = info.to_datagram().expect("encode SERVERINFO");
    let peer = thread::spawn(move || {
        let mut request = [0u8; 256];
        let (_, source) = server.recv_from(&mut request).expect("receive request");
        server
            .send_to(&reply, source)
            .expect("send unsupported reply");
    });
    let mut client = CommandClient::bind(LOOPBACK, server_addr).expect("bind client");
    let error = client
        .connect_handshake(COMMAND_TIMEOUT)
        .expect_err("unsupported protocol must be refused");
    peer.join().expect("peer thread");
    match error {
        HandshakeError::UnsupportedVersion { info: got, .. } => {
            assert_eq!(got.protocol_version, Version::new(99, 0, 7, 2));
            assert_eq!(got.application_name, "Motive");
        }
        other => panic!("expected diagnostic SERVERINFO, got {other:?}"),
    }
}

#[test]
fn typed_property_exchange_uses_an_owned_full_reply() {
    let server = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind peer");
    let server_addr = v4(server.local_addr().expect("peer address"));
    let peer = thread::spawn(move || {
        let mut request = [0u8; 512];
        let (length, source) = server.recv_from(&mut request).expect("receive request");
        let request = &request[..length];
        assert_eq!(u16::from_le_bytes([request[0], request[1]]), NAT_REQUEST);
        assert_eq!(command_text(request), "GetProperty,cam-1,Enabled");
        server
            .send_to(&CommandResponse::value_datagram("1"), source)
            .expect("send value reply");
    });
    let mut client = CommandClient::bind(LOOPBACK, server_addr).expect("bind client");
    let request = command::get_property("cam-1", "enabled").expect("build request");
    let reply = client
        .exchange(&request, COMMAND_TIMEOUT)
        .expect("typed exchange");
    peer.join().expect("peer thread");
    assert_eq!(CommandResponse::parse_value(&reply).unwrap(), "1");
}

#[test]
fn exchange_pins_the_ipv4_udp_payload_boundary_before_socket_io() {
    assert_eq!(MAX_IPV4_UDP_PAYLOAD, 65_507);
    let server = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind peer");
    let server_addr = v4(server.local_addr().expect("peer address"));
    let at_limit = property_request(65_488);
    assert_eq!(at_limit.as_bytes().len(), MAX_IPV4_UDP_PAYLOAD);

    let peer = thread::spawn(move || {
        let mut request = vec![0u8; MAX_IPV4_UDP_PAYLOAD];
        let (length, source) = server.recv_from(&mut request).expect("receive max request");
        assert_eq!(length, MAX_IPV4_UDP_PAYLOAD);
        server
            .send_to(&CommandResponse::value_datagram("1"), source)
            .expect("send max-request reply");
        server
    });

    let mut client = CommandClient::bind(LOOPBACK, server_addr).expect("bind client");
    client
        .exchange(&at_limit, COMMAND_TIMEOUT)
        .expect("exactly max-sized IPv4 datagram is accepted");
    let server = peer.join().expect("peer thread");

    let over_limit = property_request(65_489);
    assert_eq!(over_limit.as_bytes().len(), MAX_IPV4_UDP_PAYLOAD + 1);
    let error = client
        .exchange(&over_limit, COMMAND_TIMEOUT)
        .expect_err("one byte over the IPv4 payload limit must be rejected");
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);

    server
        .set_read_timeout(Some(Duration::from_millis(120)))
        .expect("set no-send timeout");
    let mut received = [0u8; 64];
    assert!(
        server.recv_from(&mut received).is_err(),
        "oversized request reached the UDP socket"
    );
}

#[test]
fn safe_generic_exchange_rejects_tier1_write_before_socket_io() {
    let server = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind peer");
    let server_addr = v4(server.local_addr().expect("peer address"));
    let mut client = CommandClient::bind(LOOPBACK, server_addr).expect("bind client");
    let step = Tier1Write::camera_enabled(vec!["cam-1".to_owned()], true)
        .expect("build Tier-1 write")
        .begin();

    let error = client
        .exchange(step.set_request(), COMMAND_TIMEOUT)
        .expect_err("safe generic exchange must reject Tier-1 writes");
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);

    server
        .set_read_timeout(Some(Duration::from_millis(120)))
        .expect("set no-send timeout");
    let mut buf = [0u8; 256];
    assert!(
        server.recv_from(&mut buf).is_err(),
        "rejected Tier-1 request reached the socket"
    );
}

#[test]
fn transaction_restores_socket_options_and_raw_conversion_is_explicit() {
    let server = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind peer");
    let server_addr = v4(server.local_addr().expect("peer address"));
    let peer = thread::spawn(move || {
        let mut request = [0u8; 256];
        let (_, source) = server.recv_from(&mut request).expect("receive request");
        server
            .send_to(&CommandResponse::value_datagram("1"), source)
            .expect("send value reply");
    });
    let mut client = CommandClient::bind(LOOPBACK, server_addr).expect("bind client");
    let request = command::get_property("cam-1", "Enabled").expect("build property request");
    client
        .exchange(&request, COMMAND_TIMEOUT)
        .expect("transaction");
    peer.join().expect("peer thread");

    let raw = client.into_raw();
    assert_eq!(raw.read_timeout().unwrap(), None);
    assert_eq!(raw.write_timeout().unwrap(), None);

    let sink = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind raw sink");
    let sink_addr = sink.local_addr().expect("sink address");
    raw.send_to(b"raw-command-bytes", sink_addr)
        .expect("raw send");
    let mut buf = [0u8; 64];
    let (length, source) = sink.recv_from(&mut buf).expect("raw receive");
    assert_eq!(&buf[..length], b"raw-command-bytes");
    assert_eq!(source.ip(), std::net::IpAddr::V4(LOOPBACK));
}

#[test]
fn tier1_executor_stops_after_a_reread_mismatch() {
    let server = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind peer");
    let server_addr = v4(server.local_addr().expect("peer address"));
    let peer = thread::spawn(move || {
        let mut request = [0u8; 512];
        let (length, source) = server
            .recv_from(&mut request)
            .expect("receive canary write");
        assert!(command_text(&request[..length]).starts_with("SetProperty,cam-1,"));
        server
            .send_to(&CommandResponse::status_datagram(0), source)
            .expect("send canary status");
        let (length, source) = server
            .recv_from(&mut request)
            .expect("receive canary reread");
        assert_eq!(
            command_text(&request[..length]),
            "GetProperty,cam-1,Enabled"
        );
        server
            .send_to(&CommandResponse::value_datagram("0"), source)
            .expect("send mismatched value");
        server
            .set_read_timeout(Some(Duration::from_millis(120)))
            .expect("set no-extra-write timeout");
        assert!(
            server.recv_from(&mut request).is_err(),
            "executor sent a later target after mismatch"
        );
    });
    let mut client = CommandClient::bind(LOOPBACK, server_addr).expect("bind client");
    let write = Tier1Write::camera_enabled(vec!["cam-1".to_owned(), "cam-2".to_owned()], true)
        .expect("build Tier-1 write");
    let error = client
        .execute_tier1(write, COMMAND_TIMEOUT)
        .expect_err("mismatched canary must stop sequence");
    peer.join().expect("peer thread");
    assert!(matches!(
        error,
        Tier1ExecutionError::UnknownRemoteOutcome {
            target,
            phase: Tier1Phase::GetPropertyReread,
            ..
        } if target == "cam-1"
    ));
}

#[test]
fn tier1_executor_completes_with_unterminated_rereads() {
    let server = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind peer");
    let server_addr = v4(server.local_addr().expect("peer address"));
    // Bounded, so a client that stops early cannot leave the peer waiting.
    server
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set peer timeout");
    let peer = thread::spawn(move || {
        let mut request = [0u8; 512];
        for camera in ["cam-1", "cam-2"] {
            let (length, source) = server.recv_from(&mut request).expect("receive write");
            assert!(
                command_text(&request[..length]).starts_with(&format!("SetProperty,{camera},"))
            );
            server
                .send_to(&CommandResponse::status_datagram(0), source)
                .expect("send status");
            let (length, source) = server.recv_from(&mut request).expect("receive reread");
            assert_eq!(
                command_text(&request[..length]),
                format!("GetProperty,{camera},Enabled")
            );
            server
                .send_to(&unterminated("true"), source)
                .expect("send unterminated value");
        }
    });
    let mut client = CommandClient::bind(LOOPBACK, server_addr).expect("bind client");
    let write = Tier1Write::camera_enabled(vec!["cam-1".to_owned(), "cam-2".to_owned()], true)
        .expect("build Tier-1 write");
    let progress = client
        .execute_tier1(write, COMMAND_TIMEOUT)
        .expect("both targets verify");
    peer.join().expect("peer thread");
    assert_eq!(progress.len(), 2);
    for (record, camera) in progress.iter().zip(["cam-1", "cam-2"]) {
        assert_eq!(record.target(), camera);
        assert_eq!(record.reread(), "true");
    }
}

#[test]
fn tier1_failure_retains_completed_progress() {
    let server = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind peer");
    let server_addr = v4(server.local_addr().expect("peer address"));
    let peer = thread::spawn(move || {
        let mut request = [0u8; 512];
        let (length, source) = server
            .recv_from(&mut request)
            .expect("receive canary write");
        assert!(command_text(&request[..length]).starts_with("SetProperty,cam-1,"));
        server
            .send_to(&CommandResponse::status_datagram(0), source)
            .expect("send canary status");

        let (length, source) = server
            .recv_from(&mut request)
            .expect("receive canary reread");
        assert_eq!(
            command_text(&request[..length]),
            "GetProperty,cam-1,Enabled"
        );
        server
            .send_to(&CommandResponse::value_datagram("1"), source)
            .expect("send canary value");

        let (length, source) = server
            .recv_from(&mut request)
            .expect("receive second write");
        assert!(command_text(&request[..length]).starts_with("SetProperty,cam-2,"));
        server
            .send_to(&CommandResponse::status_datagram(7), source)
            .expect("send rejected status");
        server
            .set_read_timeout(Some(Duration::from_millis(120)))
            .expect("set no-extra-write timeout");
        assert!(
            server.recv_from(&mut request).is_err(),
            "rejected target must not receive a reread"
        );
    });

    let mut client = CommandClient::bind(LOOPBACK, server_addr).expect("bind client");
    let write = Tier1Write::camera_enabled(vec!["cam-1".to_owned(), "cam-2".to_owned()], true)
        .expect("build Tier-1 write");
    let error = client
        .execute_tier1(write, COMMAND_TIMEOUT)
        .expect_err("second target status must stop sequence");
    peer.join().expect("peer thread");

    match &error {
        Tier1ExecutionError::StatusFailure {
            target,
            status,
            progress,
        } => {
            assert_eq!(target, "cam-2");
            assert_eq!(*status, 7);
            assert_eq!(progress.len(), 1);
            assert_eq!(progress[0].target(), "cam-1");
            assert_eq!(progress[0].reread(), "1");
        }
        other @ Tier1ExecutionError::UnknownRemoteOutcome { .. } => {
            panic!("expected second-target status failure, got {other:?}")
        }
    }
    assert_eq!(error.progress().len(), 1);
}

#[test]
fn same_peer_late_reply_has_no_freshness_proof() {
    let server = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind peer");
    let server_addr = v4(server.local_addr().expect("peer address"));
    let stale = sample_serverinfo(Version::V4_1);
    let fresh = sample_serverinfo(Version::V4_1);
    let peer = thread::spawn(move || {
        let mut request = [0u8; 256];
        let (_, source) = server.recv_from(&mut request).expect("receive request");
        let mut stale_bytes = stale.to_datagram().expect("encode stale reply");
        stale_bytes[4..10].copy_from_slice(b"STALE\0");
        let mut fresh_bytes = fresh.to_datagram().expect("encode fresh reply");
        fresh_bytes[4..10].copy_from_slice(b"FRESH\0");
        server
            .send_to(&stale_bytes, source)
            .expect("send stale reply");
        server
            .send_to(&fresh_bytes, source)
            .expect("send fresh reply");
    });
    let mut client = CommandClient::bind(LOOPBACK, server_addr).expect("bind client");
    let got = client
        .connect_handshake(COMMAND_TIMEOUT)
        .expect("same-peer reply is transport-valid");
    peer.join().expect("peer thread");
    assert_eq!(got.application_name, "STALE");
}

/// A reply that arrives after its request's deadline must not be returned as
/// the answer to the next request. The peer answers request A only after the
/// client has reported A's timeout; request B must then receive B's value.
/// Without draining the socket before B is sent, the queued late "A" reply is
/// the first same-peer NAT_RESPONSE the B transaction sees, and B returns it.
#[test]
fn late_reply_to_a_timed_out_request_does_not_answer_the_next_request() {
    let server = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind peer");
    let server_addr = v4(server.local_addr().expect("peer address"));
    let (timed_out_tx, timed_out_rx) = mpsc::channel::<()>();
    let (late_sent_tx, late_sent_rx) = mpsc::channel::<()>();
    let peer = thread::spawn(move || {
        let mut request = [0u8; 512];
        let (length, source) = server.recv_from(&mut request).expect("receive request A");
        assert_eq!(
            command_text(&request[..length]),
            "GetProperty,cam-a,Enabled"
        );
        timed_out_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("client reports A's timeout");
        server
            .send_to(&CommandResponse::value_datagram("A"), source)
            .expect("send late reply to A");
        late_sent_tx.send(()).expect("announce late reply");

        let (length, source) = server.recv_from(&mut request).expect("receive request B");
        assert_eq!(
            command_text(&request[..length]),
            "GetProperty,cam-b,Enabled"
        );
        server
            .send_to(&CommandResponse::value_datagram("B"), source)
            .expect("send reply to B");
    });

    let mut client = CommandClient::bind(LOOPBACK, server_addr).expect("bind client");
    let request_a = command::get_property("cam-a", "Enabled").expect("request A");
    let error = client
        .exchange(&request_a, Duration::from_millis(80))
        .expect_err("A is answered only after its deadline");
    assert_eq!(error.kind(), io::ErrorKind::TimedOut, "{error:?}");
    timed_out_tx.send(()).expect("release peer");
    late_sent_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("peer sent the late reply");
    // Loopback delivery normally completes inside the peer's send_to; this
    // configured settle only keeps a deferred softirq on a loaded host from
    // delivering the late datagram after B's drain. It cannot make the test
    // pass without the drain: then the late reply is queued ahead of B's.
    thread::sleep(Duration::from_millis(20));

    let request_b = command::get_property("cam-b", "Enabled").expect("request B");
    let reply = client
        .exchange(&request_b, COMMAND_TIMEOUT)
        .expect("B is answered in time");
    peer.join().expect("peer thread");
    assert_eq!(
        CommandResponse::parse_value(&reply).expect("value reply"),
        "B",
        "the late reply to A was returned as the answer to B"
    );
}

/// NAT_UNRECOGNIZED_REQUEST from the configured peer ends the transaction at
/// once with `Unsupported` instead of waiting for the deadline.
#[test]
fn unrecognized_request_reply_fails_immediately() {
    assert_eq!(NAT_UNRECOGNIZED_REQUEST, 100);
    let server = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind peer");
    let server_addr = v4(server.local_addr().expect("peer address"));
    let peer = thread::spawn(move || {
        let mut request = [0u8; 512];
        let (_, source) = server.recv_from(&mut request).expect("receive request");
        let mut refusal = NAT_UNRECOGNIZED_REQUEST.to_le_bytes().to_vec();
        refusal.extend_from_slice(&0u16.to_le_bytes());
        server.send_to(&refusal, source).expect("send refusal");
    });
    let mut client = CommandClient::bind(LOOPBACK, server_addr).expect("bind client");
    let request = command::get_property("cam-1", "Enabled").expect("build request");
    let timeout = Duration::from_secs(5);
    let started = Instant::now();
    let error = client
        .exchange(&request, timeout)
        .expect_err("an unrecognized request must fail");
    let elapsed = started.elapsed();
    peer.join().expect("peer thread");
    assert_eq!(error.kind(), io::ErrorKind::Unsupported, "{error:?}");
    assert!(
        elapsed < timeout / 2,
        "the refusal waited out the deadline: {elapsed:?}"
    );
}

/// A refusal from a foreign peer is filtered like any other foreign datagram.
#[test]
fn unrecognized_request_reply_from_a_foreign_peer_is_ignored() {
    let intended = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind intended peer");
    let intended_addr = v4(intended.local_addr().expect("intended address"));
    let foreign = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind foreign peer");
    let peer = thread::spawn(move || {
        let mut request = [0u8; 512];
        let (_, source) = intended.recv_from(&mut request).expect("receive request");
        let mut refusal = NAT_UNRECOGNIZED_REQUEST.to_le_bytes().to_vec();
        refusal.extend_from_slice(&0u16.to_le_bytes());
        foreign
            .send_to(&refusal, source)
            .expect("send foreign refusal");
        intended
            .send_to(&CommandResponse::value_datagram("1"), source)
            .expect("send intended reply");
    });
    let mut client = CommandClient::bind(LOOPBACK, intended_addr).expect("bind client");
    let request = command::get_property("cam-1", "Enabled").expect("build request");
    let reply = client
        .exchange(&request, COMMAND_TIMEOUT)
        .expect("foreign refusal is filtered");
    peer.join().expect("peer thread");
    assert_eq!(CommandResponse::parse_value(&reply).unwrap(), "1");
}

/// A NAT_REQUEST_FRAMEOFDATA exchange accepts only a FrameOfData reply from
/// the configured peer and returns it whole.
#[test]
fn frame_request_returns_the_frame_reply() {
    let server = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind peer");
    let server_addr = v4(server.local_addr().expect("peer address"));
    let frame = natnet::SyntheticServer::new(Version::V4_1, 1_000_000)
        .datagram(
            9,
            0.25,
            &[natnet::PlannedBody::tracked(
                1,
                [0.0, 1.0, 2.0],
                [0.0, 0.0, 0.0, 1.0],
            )],
        )
        .expect("synthetic frame");
    let reply = frame.clone();
    let peer = thread::spawn(move || {
        let mut request = [0u8; 64];
        let (length, source) = server.recv_from(&mut request).expect("receive request");
        assert_eq!(&request[..length], &[6, 0, 0, 0]);
        // A same-peer reply of another kind is skipped, not returned.
        let mut unrelated = NAT_REQUEST.to_le_bytes().to_vec();
        unrelated.extend_from_slice(&0u16.to_le_bytes());
        server.send_to(&unrelated, source).expect("send unrelated");
        server.send_to(&reply, source).expect("send frame");
    });
    let mut client = CommandClient::bind(LOOPBACK, server_addr).expect("bind client");
    let request = command::request_frame_of_data().expect("build request");
    let datagram = client
        .exchange(&request, Duration::from_secs(5))
        .expect("frame reply");
    peer.join().expect("peer thread");
    assert_eq!(datagram, frame);
    let decoded = natnet::FrameOfData::parse(&datagram, Version::V4_1).expect("decode reply");
    assert_eq!(decoded.frame_number, 9);
}
