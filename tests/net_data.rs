//! `net`-feature data-channel coverage against loopback only.
//!
//! These tests exercise the public receive-only [`DataClient`] seam. They do
//! not send NatNet commands or contact a real Motive host. The multicast case
//! uses an administratively scoped group on the loopback interface and is
//! skipped when the host cannot provide loopback multicast support.
#![cfg(feature = "net")]
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "the loopback integration harness uses assertion-oriented failure paths"
)]

use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::time::Duration;

use natnet::net::{DataClient, MAX_IPV4_UDP_PAYLOAD, MIN_DATA_BUFFER};

const LOOPBACK: Ipv4Addr = Ipv4Addr::LOCALHOST;
const MULTICAST_GROUP: Ipv4Addr = Ipv4Addr::new(239, 255, 42, 99);

fn v4(addr: SocketAddr) -> SocketAddrV4 {
    match addr {
        SocketAddr::V4(addr) => addr,
        SocketAddr::V6(addr) => panic!("expected IPv4 address, got {addr}"),
    }
}

#[test]
fn unicast_preserves_source_and_local_addresses() {
    let client = DataClient::bind_unicast(LOOPBACK, 0).expect("bind data client");
    let client_addr = v4(client.local_addr().expect("client local address"));
    assert_eq!(*client_addr.ip(), LOOPBACK);
    assert_ne!(client_addr.port(), 0);

    let sender = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind sender");
    let sender_addr = sender.local_addr().expect("sender local address");
    sender
        .send_to(b"natnet-data", client_addr)
        .expect("send loopback datagram");

    let mut buf = [0u8; 64];
    let (len, source) = client.recv(&mut buf).expect("receive loopback datagram");
    assert_eq!(&buf[..len], b"natnet-data");
    assert_eq!(source, sender_addr);
}

#[test]
fn nonblocking_empty_receive_returns_would_block() {
    let client = DataClient::bind_unicast(LOOPBACK, 0).expect("bind data client");
    client.set_nonblocking(true).expect("set nonblocking");

    let mut buf = [0u8; 32];
    let err = client
        .recv(&mut buf)
        .expect_err("empty socket must not block");
    assert_eq!(err.kind(), io::ErrorKind::WouldBlock);
}

#[test]
fn read_timeout_returns_without_traffic() {
    let client = DataClient::bind_unicast(LOOPBACK, 0).expect("bind data client");
    client
        .set_read_timeout(Some(Duration::from_millis(80)))
        .expect("set read timeout");

    let mut buf = [0u8; 32];
    let err = client
        .recv(&mut buf)
        .expect_err("empty socket must time out");
    assert!(
        matches!(
            err.kind(),
            io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
        ),
        "platform returned unexpected timeout error: {err:?}"
    );
}

#[test]
fn duplicate_unicast_bind_is_refused() {
    let first = DataClient::bind_unicast(LOOPBACK, 0).expect("bind first data client");
    let port = v4(first.local_addr().expect("first local address")).port();
    let err = DataClient::bind_unicast(LOOPBACK, port).expect_err("port must be occupied");
    assert_eq!(err.kind(), io::ErrorKind::AddrInUse);
}

#[test]
fn failed_multicast_join_releases_its_bound_socket() {
    // 192.0.2.1 is TEST-NET and is not assigned to this host. The bind can
    // succeed, but the selected-interface join must fail. Dropping that
    // failed constructor must release the wildcard socket it opened first.
    let probe = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("reserve test port");
    let port = v4(probe.local_addr().expect("probe address")).port();
    drop(probe);

    let err = DataClient::bind_multicast(Ipv4Addr::new(192, 0, 2, 1), MULTICAST_GROUP, port)
        .expect_err("unassigned interface must reject multicast join");
    assert!(
        matches!(
            err.kind(),
            io::ErrorKind::AddrNotAvailable | io::ErrorKind::Other
        ) || err.raw_os_error().is_some(),
        "unexpected multicast join error: {err:?}"
    );

    let rebound = DataClient::bind_unicast(LOOPBACK, port).expect("failed join must clean up");
    assert_eq!(
        v4(rebound.local_addr().expect("rebound address")).port(),
        port
    );
}

#[test]
fn multicast_uses_selected_interface_and_receives_loopback_delivery() {
    let client = match DataClient::bind_multicast(LOOPBACK, MULTICAST_GROUP, 0) {
        Ok(client) => client,
        Err(err) if multicast_unavailable(&err) => {
            eprintln!("skipping loopback multicast evidence: {err}");
            return;
        }
        Err(err) => panic!("loopback multicast bind failed unexpectedly: {err}"),
    };
    let port = v4(client.local_addr().expect("multicast local address")).port();
    client
        .set_read_timeout(Some(Duration::from_millis(250)))
        .expect("set multicast read timeout");

    let sender = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind multicast sender");
    // Binding the sender to loopback selects the same interface used for the
    // join; std::net exposes no portable multicast-interface setter.
    sender
        .set_multicast_loop_v4(true)
        .expect("enable local multicast loopback");
    sender.set_multicast_ttl_v4(1).expect("set multicast TTL");
    // macOS routes the group through the default interface, whose address a
    // loopback-bound sender cannot use, and refuses the send.
    match sender.send_to(
        b"natnet-multicast",
        SocketAddrV4::new(MULTICAST_GROUP, port),
    ) {
        Ok(_) => {}
        Err(err) if multicast_unavailable(&err) => {
            eprintln!("skipping loopback multicast evidence: {err}");
            return;
        }
        Err(err) => panic!("send selected-interface multicast datagram: {err}"),
    }

    let mut buf = [0u8; 64];
    let (len, source) = client
        .recv(&mut buf)
        .expect("loopback multicast delivery after a successful join");
    assert_eq!(&buf[..len], b"natnet-multicast");
    assert_eq!(source.ip(), std::net::IpAddr::V4(LOOPBACK));
}

fn multicast_unavailable(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::AddrNotAvailable
            | io::ErrorKind::Unsupported
            | io::ErrorKind::PermissionDenied
            | io::ErrorKind::WouldBlock
            | io::ErrorKind::TimedOut
    )
}

#[test]
fn a_datagram_filling_a_short_buffer_is_reported_as_possibly_truncated() {
    assert_eq!(MIN_DATA_BUFFER, MAX_IPV4_UDP_PAYLOAD);
    assert_eq!(MIN_DATA_BUFFER, 65_507);
    let client = DataClient::bind_unicast(LOOPBACK, 0).expect("bind data client");
    client
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set read timeout");
    let client_addr = client.local_addr().expect("client address");
    let sender = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind sender");

    // 100 bytes into a 64-byte buffer: the OS truncates it to exactly 64, or
    // Windows fails the receive; either way it is reported as InvalidData.
    sender.send_to(&[7u8; 100], client_addr).expect("send long");
    let mut short = [0u8; 64];
    let err = client
        .recv(&mut short)
        .expect_err("a datagram filling a short buffer may be truncated");
    assert_eq!(err.kind(), io::ErrorKind::InvalidData, "{err:?}");

    // A datagram shorter than the buffer is unambiguous.
    sender.send_to(&[8u8; 63], client_addr).expect("send short");
    let (len, _) = client.recv(&mut short).expect("short datagram");
    assert_eq!(len, 63);
}

/// At MIN_DATA_BUFFER, even a maximal IPv4 datagram that fills the buffer
/// exactly is complete by construction.
#[test]
#[cfg_attr(
    target_os = "macos",
    ignore = "macOS refuses UDP sends above net.inet.udp.maxdgram (9216 bytes by default)"
)]
fn a_maximal_ipv4_datagram_filling_min_data_buffer_is_complete() {
    assert_eq!(MIN_DATA_BUFFER, MAX_IPV4_UDP_PAYLOAD);
    let client = DataClient::bind_unicast(LOOPBACK, 0).expect("bind data client");
    client
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set read timeout");
    let client_addr = client.local_addr().expect("client address");
    let sender = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind sender");

    sender
        .send_to(&vec![9u8; MAX_IPV4_UDP_PAYLOAD], client_addr)
        .expect("send maximal IPv4 datagram");
    let mut full = vec![0u8; MIN_DATA_BUFFER];
    let (len, _) = client.recv(&mut full).expect("maximal datagram");
    assert_eq!(len, MAX_IPV4_UDP_PAYLOAD);
    assert!(full.iter().all(|&byte| byte == 9));
}

#[test]
fn from_socket_adopts_a_caller_configured_socket() {
    // The caller owns socket construction; std sets what it can here, and a
    // socket2-built socket with SO_REUSEADDR/SO_RCVBUF converts the same way.
    let socket = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind caller socket");
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("caller-set timeout");
    let expected = socket.local_addr().expect("caller socket address");
    let client = DataClient::from_socket(socket);
    assert_eq!(client.local_addr().expect("adopted address"), expected);

    let sender = UdpSocket::bind(SocketAddrV4::new(LOOPBACK, 0)).expect("bind sender");
    sender
        .send_to(b"adopted", expected)
        .expect("send to adopted socket");
    let mut buf = vec![0u8; MIN_DATA_BUFFER];
    let (len, source) = client.recv(&mut buf).expect("receive on adopted socket");
    assert_eq!(&buf[..len], b"adopted");
    assert_eq!(source, sender.local_addr().expect("sender address"));
}
