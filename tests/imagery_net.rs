//! [`ImageClient`] over loopback: a synthetic AUGI/1 stream is written by a
//! thread and read back record-by-record, exercising both connection postures
//! (the client dials out; the client adopts an accepted stream) with no
//! hardware and no `motive-bridge` build.

#![cfg(all(feature = "imagery", feature = "net"))]
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "the loopback integration harness uses assertion-oriented failure paths"
)]

use std::fmt::Write as _;
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use natnet::imagery::net::{ImageClient, MAX_RECORD_LEN};
use natnet::imagery::{
    self, CameraDesc, Hello, ImageEncoding, ImageFrame, MODE_LISTEN, PREFIX_LEN, PROTO_MAJOR,
    PROTO_MINOR, Record, parse_record,
};

/// Hello, one CameraDesc, three ImageFrames, a Heartbeat, Bye.
fn synthetic_stream() -> Vec<u8> {
    let mut out = Hello {
        proto_major: PROTO_MAJOR,
        proto_minor: PROTO_MINOR,
        clock_hz: 10_000_000,
        qpf: 10_000_000,
        natnet_major: 4,
        natnet_minor: 1,
        motive_build: 31_021,
        host_wall_unix_secs: 1_612_137_600,
        mode: MODE_LISTEN,
        camera_count: 1,
        bridge_id: "test".to_string(),
        profile: String::new(),
        calibration: String::new(),
    }
    .to_record()
    .expect("hello encodes");

    let rays = vec![0u8; 24]; // 1x1 grid
    out.extend_from_slice(
        &CameraDesc {
            camera_index: 0,
            camera_id: 1,
            serial: 12_345,
            video_mode: 1,
            encoding: ImageEncoding::Mono8,
            contributes_to_solve: true,
            frame_rate_divisor: 12,
            width: 4,
            height: 2,
            position: [0.0; 3],
            orientation: [0.0; 9],
            ray_grid_w: 1,
            ray_grid_h: 1,
            rays: &rays,
            name: "Prime 13",
        }
        .to_record()
        .expect("camera desc encodes"),
    );

    for n in 0..3u32 {
        let payload: Vec<u8> = (0..8u8).map(|p| p.wrapping_add(n as u8)).collect();
        out.extend_from_slice(
            &ImageFrame {
                frame_number: 1_000 + n,
                camera_index: 0,
                // Frame 1 has no correlation: absent, not zero.
                camera_mid_exposure: (n != 1).then_some(u64::from(n) * 100_000),
                grab_ticks: u64::from(n) * 7,
                api_timestamp_secs: f64::from(n) * 0.1,
                width: 4,
                height: 2,
                encoding: ImageEncoding::Mono8,
                payload: &payload,
            }
            .to_record()
            .expect("image frame encodes"),
        );
    }

    out.extend_from_slice(&imagery::heartbeat_record());
    out.extend_from_slice(&imagery::bye_record());
    out
}

/// Drains a client until Bye, asserting the stream's shape.
fn drain_and_check(client: &mut ImageClient) {
    let mut buf = Vec::new();
    let mut images = 0u32;
    let mut absent = 0u32;

    client
        .recv_record(&mut buf, Duration::from_secs(1))
        .expect("hello");
    match parse_record(&buf).expect("parse hello") {
        Record::Hello(h) => {
            assert_eq!(h.clock_hz, 10_000_000);
            assert_eq!(h.camera_count, 1);
        }
        other => panic!("expected Hello first, got {other:?}"),
    }

    client
        .recv_record(&mut buf, Duration::from_secs(1))
        .expect("camera desc");
    match parse_record(&buf).expect("parse desc") {
        Record::CameraDesc(d) => {
            assert_eq!(d.serial, 12_345);
            assert_eq!(d.name, "Prime 13");
        }
        other => panic!("expected CameraDesc, got {other:?}"),
    }

    loop {
        client
            .recv_record(&mut buf, Duration::from_secs(1))
            .expect("record");
        match parse_record(&buf).expect("parse") {
            Record::ImageFrame(f) => {
                assert_eq!(
                    f.payload.len(),
                    usize::from(f.width) * usize::from(f.height)
                );
                if f.camera_mid_exposure.is_none() {
                    absent += 1;
                }
                images += 1;
            }
            Record::Heartbeat => {}
            Record::Bye => break,
            other => panic!("unexpected {other:?}"),
        }
    }
    assert_eq!(images, 3);
    assert_eq!(absent, 1, "exactly one frame had no correlation");
}

#[test]
fn client_dials_out_and_reads_a_whole_stream() {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = match listener.local_addr().expect("addr") {
        std::net::SocketAddr::V4(a) => a,
        other => panic!("expected v4, got {other}"),
    };

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        stream.write_all(&synthetic_stream()).expect("write");
    });

    let mut client =
        ImageClient::connect(Ipv4Addr::LOCALHOST, addr, Duration::from_secs(1)).expect("connect");
    assert_eq!(client.peer_addr().unwrap().port(), addr.port());
    drain_and_check(&mut client);
    server.join().expect("server thread");
}

#[test]
fn client_adopts_an_accepted_stream_reverse_mode() {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = listener.local_addr().expect("addr");

    // The "bridge" dials the consumer, as it does with `connect_to` set.
    let bridge = thread::spawn(move || {
        let mut stream = TcpStream::connect(addr).expect("dial consumer");
        stream.write_all(&synthetic_stream()).expect("write");
    });

    let (stream, _) = listener.accept().expect("accept");
    let mut client = ImageClient::from_stream(stream, Duration::from_secs(1)).expect("from stream");
    drain_and_check(&mut client);
    bridge.join().expect("bridge thread");
}

#[test]
fn a_wrong_interface_is_refused_rather_than_streamed() {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = match listener.local_addr().expect("addr") {
        std::net::SocketAddr::V4(a) => a,
        other => panic!("expected v4, got {other}"),
    };
    let accepter = thread::spawn(move || drop(listener.accept()));

    // 127.0.0.2 is loopback but not the source the kernel picks for
    // 127.0.0.1, so the post-connect check must fire.
    let err = ImageClient::connect(Ipv4Addr::new(127, 0, 0, 2), addr, Duration::from_secs(1))
        .unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::AddrNotAvailable);
    accepter.join().expect("accepter");
}

#[test]
fn a_hostile_length_is_refused_before_allocating() {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = listener.local_addr().expect("addr");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        // A well-formed prefix claiming a 4 GiB payload.
        let mut prefix = Vec::new();
        prefix.extend_from_slice(&imagery::MAGIC.to_le_bytes());
        prefix.extend_from_slice(&3u16.to_le_bytes());
        prefix.extend_from_slice(&40u16.to_le_bytes());
        prefix.extend_from_slice(&u32::MAX.to_le_bytes());
        stream.write_all(&prefix).expect("hostile prefix");
    });

    let mut client = ImageClient::from_stream(
        TcpStream::connect(addr).expect("connect"),
        Duration::from_secs(1),
    )
    .expect("from stream");
    let mut buf = Vec::new();
    let err = client
        .recv_record(&mut buf, Duration::from_secs(1))
        .unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    if cfg!(target_pointer_width = "32") {
        // The advertised total cannot fit usize, so framing rejects it
        // before the transport can compare it with its smaller record cap.
        assert_eq!(
            err.get_ref()
                .and_then(|source| source.downcast_ref::<natnet::Error>()),
            Some(&natnet::Error::ImplausibleCount {
                count: usize::MAX,
                available: imagery::PREFIX_LEN,
            })
        );
    } else {
        assert!(err.to_string().contains("MAX_RECORD_LEN"));
    }
    let retry = client
        .recv_record(&mut buf, Duration::from_secs(1))
        .expect_err("oversize is terminal");
    assert_eq!(retry.kind(), err.kind());
    assert_eq!(retry.to_string(), err.to_string());
    assert!(buf.is_empty());
    assert!(u64::try_from(MAX_RECORD_LEN).unwrap() < u64::from(u32::MAX));
    server.join().expect("server");
}

#[test]
fn a_non_augi_stream_is_refused() {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = listener.local_addr().expect("addr");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        stream
            .write_all(b"GET / HTTP/1.1\r\n\r\n")
            .expect("non-AUGI bytes");
    });

    let mut client = ImageClient::from_stream(
        TcpStream::connect(addr).expect("connect"),
        Duration::from_secs(1),
    )
    .expect("from stream");
    let mut buf = Vec::new();
    let err = client
        .recv_record(&mut buf, Duration::from_secs(1))
        .unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    server.join().expect("server");
}

#[test]
fn clean_eof_is_false_and_preserves_the_last_record_buffer() {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = listener.local_addr().expect("addr");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        stream
            .write_all(&imagery::heartbeat_record())
            .expect("heartbeat");
        drop(stream);
    });

    let mut client = ImageClient::from_stream(
        TcpStream::connect(addr).expect("connect"),
        Duration::from_secs(1),
    )
    .expect("from stream");
    let mut buf = Vec::new();
    assert!(
        client
            .recv_record(&mut buf, Duration::from_secs(1))
            .expect("record")
    );
    let last = buf.clone();
    assert!(
        !client
            .recv_record(&mut buf, Duration::from_secs(1))
            .expect("clean eof")
    );
    assert_eq!(buf, last);
    assert!(
        !client
            .recv_record(&mut buf, Duration::from_secs(1))
            .expect("closed remains closed")
    );
    assert_eq!(buf, last);
    server.join().expect("server");
}

#[test]
fn partial_prefix_is_terminal_and_clears_the_incomplete_buffer() {
    for count in 1..PREFIX_LEN {
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            stream
                .write_all(&imagery::heartbeat_record()[..count])
                .expect("partial prefix");
        });

        let mut client = ImageClient::from_stream(
            TcpStream::connect(addr).expect("connect"),
            Duration::from_secs(1),
        )
        .expect("from stream");
        let mut buf = b"stale record".to_vec();
        let first = client
            .recv_record(&mut buf, Duration::from_secs(1))
            .expect_err("partial prefix fails");
        assert_eq!(first.kind(), std::io::ErrorKind::UnexpectedEof);
        assert!(buf.is_empty(), "incomplete record must be cleared");
        let second = client
            .recv_record(&mut buf, Duration::from_secs(1))
            .expect_err("terminal failure");
        assert_eq!(second.kind(), first.kind());
        assert_eq!(second.to_string(), first.to_string());
        assert!(buf.is_empty());
        server.join().expect("server");
    }
}

#[test]
fn fragmented_record_is_reassembled_without_packet_boundary_assumptions() {
    let record = imagery::heartbeat_record();
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = listener.local_addr().expect("addr");
    let (ready_tx, ready_rx) = mpsc::channel();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        ready_tx.send(()).expect("ready");
        for byte in record {
            stream.write_all(&[byte]).expect("fragment");
        }
    });

    let mut client = ImageClient::from_stream(
        TcpStream::connect(addr).expect("connect"),
        Duration::from_secs(1),
    )
    .expect("from stream");
    ready_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("ready");
    let mut buf = Vec::new();
    assert!(
        client
            .recv_record(&mut buf, Duration::from_secs(1))
            .expect("fragmented record")
    );
    assert_eq!(buf, imagery::heartbeat_record());
    server.join().expect("server");
}

#[test]
fn invalid_magic_is_terminal_before_the_next_read() {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = listener.local_addr().expect("addr");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut prefix = imagery::heartbeat_record();
        prefix[..4].copy_from_slice(&0x0102_0304u32.to_le_bytes());
        stream.write_all(&prefix).expect("bad magic");
    });

    let mut client = ImageClient::from_stream(
        TcpStream::connect(addr).expect("connect"),
        Duration::from_secs(1),
    )
    .expect("from stream");
    let mut buf = b"stale record".to_vec();
    let first = client
        .recv_record(&mut buf, Duration::from_secs(1))
        .expect_err("bad magic fails");
    assert_eq!(first.kind(), std::io::ErrorKind::InvalidData);
    match first
        .get_ref()
        .and_then(|source| source.downcast_ref::<natnet::Error>())
    {
        Some(natnet::Error::InvalidMagic { found, expected }) => {
            assert_eq!(*found, 0x0102_0304);
            assert_eq!(*expected, imagery::MAGIC);
        }
        other => panic!("expected source-preserving InvalidMagic, got {other:?}"),
    }
    assert!(buf.is_empty());
    let second = client
        .recv_record(&mut buf, Duration::from_secs(1))
        .expect_err("bad magic is terminal");
    assert_eq!(second.kind(), first.kind());
    assert_eq!(second.to_string(), first.to_string());
    assert_eq!(
        second
            .get_ref()
            .and_then(|source| source.downcast_ref::<natnet::Error>()),
        first
            .get_ref()
            .and_then(|source| source.downcast_ref::<natnet::Error>()),
        "terminal retry retains structured wire error context"
    );
    server.join().expect("server");
}

#[test]
fn partial_payload_is_terminal_and_preserves_the_original_eof_context() {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = listener.local_addr().expect("addr");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut prefix = Vec::new();
        prefix.extend_from_slice(&imagery::MAGIC.to_le_bytes());
        prefix.extend_from_slice(&4u16.to_le_bytes());
        prefix.extend_from_slice(&0u16.to_le_bytes());
        prefix.extend_from_slice(&2u32.to_le_bytes());
        prefix.push(0xaa);
        stream.write_all(&prefix).expect("partial payload");
    });

    let mut client = ImageClient::from_stream(
        TcpStream::connect(addr).expect("connect"),
        Duration::from_secs(1),
    )
    .expect("from stream");
    let mut buf = b"stale record".to_vec();
    let first = client
        .recv_record(&mut buf, Duration::from_secs(1))
        .expect_err("partial payload fails");
    assert_eq!(first.kind(), std::io::ErrorKind::UnexpectedEof);
    assert!(buf.is_empty());
    let second = client
        .recv_record(&mut buf, Duration::from_secs(1))
        .expect_err("partial payload remains terminal");
    assert_eq!(second.to_string(), first.to_string());
    server.join().expect("server");
}

#[test]
fn record_deadline_is_total_and_timeout_is_terminal() {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = listener.local_addr().expect("addr");
    let (ready_tx, ready_rx) = mpsc::channel();
    let (hold_tx, hold_rx) = mpsc::channel::<()>();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut prefix = Vec::new();
        prefix.extend_from_slice(&imagery::MAGIC.to_le_bytes());
        prefix.extend_from_slice(&4u16.to_le_bytes());
        prefix.extend_from_slice(&0u16.to_le_bytes());
        prefix.extend_from_slice(&2u32.to_le_bytes());
        prefix.push(0xaa);
        stream.write_all(&prefix).expect("slow-drip prefix");
        ready_tx.send(()).expect("ready");
        assert!(
            hold_rx.recv().is_err(),
            "test releases stalled peer by drop"
        );
    });

    let mut client = ImageClient::from_stream(
        TcpStream::connect(addr).expect("connect"),
        Duration::from_secs(1),
    )
    .expect("from stream");
    ready_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("ready");
    let mut buf = Vec::new();
    let first = client
        .recv_record(&mut buf, Duration::from_millis(100))
        .expect_err("stall exceeds total deadline");
    assert_eq!(first.kind(), std::io::ErrorKind::TimedOut);
    assert!(buf.is_empty());
    let second = client
        .recv_record(&mut buf, Duration::from_secs(1))
        .expect_err("timeout is terminal");
    assert_eq!(second.kind(), first.kind());
    assert_eq!(second.to_string(), first.to_string());
    drop(hold_tx);
    server.join().expect("server");
}

#[test]
fn zero_timeout_is_rejected_without_touching_the_socket() {
    let err = ImageClient::connect(
        Ipv4Addr::LOCALHOST,
        SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9),
        Duration::ZERO,
    )
    .expect_err("zero connect timeout");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
}

#[test]
fn partial_header_reports_exact_record_extent_and_is_terminal() {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = listener.local_addr().expect("address");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut bytes = imagery::MAGIC.to_le_bytes().to_vec();
        bytes.extend_from_slice(&99u16.to_le_bytes());
        bytes.extend_from_slice(&8u16.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&[1, 2, 3]);
        stream.write_all(&bytes).expect("partial header");
    });
    let mut client = ImageClient::from_stream(
        TcpStream::connect(addr).expect("connect"),
        Duration::from_secs(1),
    )
    .expect("adopt");
    let mut buf = Vec::new();
    for _ in 0..2 {
        let error = client
            .recv_record(&mut buf, Duration::from_secs(1))
            .expect_err("partial header");
        assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
        assert_eq!(
            error
                .get_ref()
                .and_then(|source| source.downcast_ref::<natnet::Error>()),
            Some(&natnet::Error::UnexpectedEof {
                offset: 0,
                needed: PREFIX_LEN + 8,
                available: PREFIX_LEN + 3
            })
        );
        assert!(buf.is_empty());
    }
    server.join().expect("server");
}

#[test]
fn invalid_record_timeout_preserves_buffer_and_allows_valid_retry() {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = listener.local_addr().expect("address");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        stream
            .write_all(&imagery::heartbeat_record())
            .expect("heartbeat");
    });
    let mut client = ImageClient::from_stream(
        TcpStream::connect(addr).expect("connect"),
        Duration::from_secs(1),
    )
    .expect("adopt");
    let mut buf = b"previous record".to_vec();
    let error = client
        .recv_record(&mut buf, Duration::ZERO)
        .expect_err("zero duration");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    assert_eq!(buf, b"previous record");
    assert!(
        client
            .recv_record(&mut buf, Duration::from_secs(1))
            .expect("retry")
    );
    assert_eq!(buf, imagery::heartbeat_record());
    server.join().expect("server");
}

#[test]
fn exact_record_cap_is_accepted_and_one_byte_over_is_refused() {
    for total in [MAX_RECORD_LEN, MAX_RECORD_LEN + 1] {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind");
        let addr = listener.local_addr().expect("address");
        let payload_len = total - PREFIX_LEN;
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            stream
                .set_write_timeout(Some(Duration::from_secs(10)))
                .expect("write timeout");
            let mut prefix = imagery::MAGIC.to_le_bytes().to_vec();
            prefix.extend_from_slice(&99u16.to_le_bytes());
            prefix.extend_from_slice(&0u16.to_le_bytes());
            prefix.extend_from_slice(
                &u32::try_from(payload_len)
                    .expect("cap fits u32")
                    .to_le_bytes(),
            );
            stream.write_all(&prefix).expect("prefix");
            if total == MAX_RECORD_LEN {
                let block = [0x5au8; 8 * 1024];
                let mut remaining = payload_len;
                while remaining > 0 {
                    let count = remaining.min(block.len());
                    stream.write_all(&block[..count]).expect("payload block");
                    remaining -= count;
                }
            }
        });
        let mut client = ImageClient::from_stream(
            TcpStream::connect(addr).expect("connect"),
            Duration::from_secs(10),
        )
        .expect("adopt");
        let mut buf = Vec::new();
        let result = client.recv_record(&mut buf, Duration::from_secs(10));
        if total == MAX_RECORD_LEN {
            assert!(result.expect("exact cap accepted"));
            assert_eq!(buf.len(), total);
            assert!(buf[PREFIX_LEN..].iter().all(|byte| *byte == 0x5a));
        } else {
            assert_eq!(
                result.expect_err("over cap rejected").kind(),
                std::io::ErrorKind::InvalidData
            );
            assert!(buf.is_empty());
            assert_eq!(buf.capacity(), 0, "reject before payload allocation");
        }
        server.join().expect("server");
    }
}

#[test]
fn byte_drip_cannot_restart_the_total_record_deadline() {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = listener.local_addr().expect("address");
    let (start_tx, start_rx) = mpsc::channel();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        start_rx.recv().expect("start delivery");
        for byte in imagery::heartbeat_record() {
            if stream.write_all(&[byte]).is_err() {
                break;
            }
            thread::sleep(Duration::from_millis(40));
        }
    });
    let mut client = ImageClient::from_stream(
        TcpStream::connect(addr).expect("connect"),
        Duration::from_secs(1),
    )
    .expect("adopt");
    let mut buf = Vec::new();
    start_tx.send(()).expect("start peer");
    let error = client
        .recv_record(&mut buf, Duration::from_millis(120))
        .expect_err("drip exceeds total deadline");
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    assert!(buf.is_empty());
    server.join().expect("server");
}

/// Binds a loopback pair, writes `bytes` from the peer, and returns a client
/// adopting the receiving end only once every one of those bytes is confirmed
/// present in its receive buffer with `peek` — which consumes nothing.
///
/// The confirmation is what keeps the two tests below off a sleep race: when
/// `recv_record` is called the bytes are already there, so nothing in the test
/// depends on how fast loopback is. The returned peer stream must be held by
/// the caller; dropping it would turn an incomplete record into a clean EOF.
fn buffered_peer(bytes: &[u8]) -> (ImageClient, TcpStream) {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = listener.local_addr().expect("address");
    let client_stream = TcpStream::connect(addr).expect("connect");
    let (mut server_stream, _) = listener.accept().expect("accept");
    server_stream
        .write_all(bytes)
        .expect("write pre-buffered bytes");
    server_stream.flush().expect("flush");
    client_stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("probe timeout");
    let mut probe = vec![0u8; bytes.len()];
    let start = Instant::now();
    while client_stream.peek(&mut probe).expect("peek") < bytes.len() {
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "loopback never delivered the pre-buffered bytes"
        );
    }
    let client = ImageClient::from_stream(client_stream, Duration::from_secs(1)).expect("adopt");
    (client, server_stream)
}

/// The record deadline bounds *waiting*, not reading. A record whose every
/// byte is already in the receive buffer must be delivered whenever the
/// deadline passes — before the read that collects it, during it, or after
/// it — because none of those is the client waiting for anything. Discarding
/// such a record would throw away a complete one and, because receive
/// failures are terminal, end the session with the record in hand.
///
/// **Not a bounded retry: this is deterministic.** `buffered_peer` has already
/// confirmed the twelve bytes are present, and where the deadline falls no
/// longer selects between outcomes. Before the first read it makes that read
/// the single post-deadline read, of bytes that are provably there; after the
/// first read it lands once that read has already returned them. Both deliver.
/// Every attempt must therefore succeed, and an attempt that times out is a
/// regression, reported as one rather than retried.
///
/// The two deadlines are the two placements, and the first needs no window at
/// all: a 1 ns timeout is already spent before the first read is issued, so it
/// takes the post-deadline path on every host, every time. On `e157774` it
/// destroyed this record 100 times in 100.
///
/// This is the empty-body case (`Heartbeat`, `total == PREFIX_LEN`, no body
/// read). The bodied case is the test below it, and the two are not
/// interchangeable: an earlier attempt passed this one while destroying every
/// record with a payload.
#[test]
fn a_fully_buffered_record_is_delivered_when_its_deadline_expires_mid_read() {
    const ATTEMPTS: usize = 64;
    let record = imagery::heartbeat_record();
    let mut delivered = 0usize;
    let mut loop_top = 0usize;
    for (attempt, timeout) in [Duration::from_nanos(1), Duration::from_micros(1)]
        .into_iter()
        .flat_map(|t| (0..ATTEMPTS).map(move |a| (a, t)))
    {
        let (mut client, _peer) = buffered_peer(&record);
        let mut buf = Vec::new();
        match client.recv_record(&mut buf, timeout) {
            Ok(true) => {
                assert_eq!(buf, record, "the delivered record must be the bytes sent");
                delivered += 1;
            }
            Ok(false) => panic!("a fully buffered record was reported as a clean EOF"),
            Err(error) => {
                if error.to_string() == "record deadline expired" {
                    loop_top += 1;
                }
                panic!(
                    "timeout {timeout:?}, attempt {attempt}: a complete record was received \
                     and then discarded: {error} (kind {:?}, buf.len() {}, {loop_top} of \
                     these were the deadline check)",
                    error.kind(),
                    buf.len()
                );
            }
        }
    }
    assert_eq!(
        delivered,
        2 * ATTEMPTS,
        "every attempt on a fully buffered record must deliver it"
    );
}

/// The companion to the test above: delivering a complete record past its
/// deadline must not weaken the deadline itself. The same construction with an
/// *incomplete* record — five of the twelve prefix bytes buffered and a peer
/// that sends no more — must still fail with [`std::io::ErrorKind::TimedOut`],
/// clear the buffer, and leave the client terminal. Nothing in the
/// assertion depends on where the deadline falls: five bytes are all there
/// ever will be, so the single post-deadline read is short whenever it runs,
/// and a read issued before the deadline runs out of time instead. Both
/// produce that kind, so this needs no retry.
#[test]
fn an_incomplete_record_past_its_deadline_still_fails_and_is_terminal() {
    let record = imagery::heartbeat_record();
    let (mut client, _peer) = buffered_peer(&record[..5]);
    let mut buf = Vec::new();
    let first = client
        .recv_record(&mut buf, Duration::from_micros(1))
        .expect_err("an incomplete record past its deadline must fail");
    assert_eq!(first.kind(), std::io::ErrorKind::TimedOut);
    assert!(
        buf.is_empty(),
        "an incomplete record must never be handed to the caller"
    );
    let second = client
        .recv_record(&mut buf, Duration::from_secs(1))
        .expect_err("a receive timeout is terminal");
    assert_eq!(second.kind(), first.kind());
    assert_eq!(second.to_string(), first.to_string());
}

/// One `ImageFrame` record, prefix plus a `payload_len`-byte payload.
///
/// Deliberately the record the crate exists for, and deliberately *bodied*:
/// `recv_record` reads the twelve-byte prefix and the payload in two separate
/// phases under one deadline, and a record with no payload never enters the
/// second at all.
fn bodied_frame_record(payload_len: usize) -> Vec<u8> {
    let payload: Vec<u8> = (0..payload_len).map(|i| (i % 251) as u8).collect();
    let record = ImageFrame {
        frame_number: 7,
        camera_index: 0,
        camera_mid_exposure: Some(1_234),
        grab_ticks: 99,
        api_timestamp_secs: 0.5,
        width: 4,
        height: 2,
        encoding: ImageEncoding::Mono8,
        payload: &payload,
    }
    .to_record()
    .expect("image frame encodes");
    assert!(
        record.len() > PREFIX_LEN,
        "this test is worthless unless the record has a body"
    );
    record
}

/// The companion this slice exists for: the same guarantee for a record with a
/// **payload**.
///
/// `recv_record` reads in two phases under one deadline — prefix, then
/// payload — so a deadline that passes while the prefix is being collected
/// arrives at the payload read with nothing left to spend. Before
/// this repair that read failed immediately, discarding a record every byte of
/// which was already in the kernel receive queue and shutting the connection
/// down; the empty-body test above could not see it, because `Heartbeat` and
/// `Bye` never reach the second call. Measured on the previous candidate
/// `e157774`, payloads of 1, 64, 4096 and 65536 bytes were delivered 0 times in
/// 200 attempts each at a 1 µs deadline, while the heartbeat was delivered
/// 200/200 in the same process.
///
/// Deterministic for the same reason as the test above: every byte is
/// `peek`-confirmed present before the client is built, and no read of bytes
/// that are present can fail, on either side of the deadline. The 1 ns row
/// needs no timing window at all — that deadline is spent before the first
/// read is issued — and the 1 µs row is the case where it is spent between the
/// prefix read and the payload read instead. On `e157774` both
/// rows are 0 delivered.
#[test]
fn a_fully_buffered_bodied_record_is_delivered_past_its_deadline() {
    const ATTEMPTS: usize = 32;
    for payload_len in [1usize, 4096] {
        let record = bodied_frame_record(payload_len);
        for (attempt, timeout) in [Duration::from_nanos(1), Duration::from_micros(1)]
            .into_iter()
            .flat_map(|t| (0..ATTEMPTS).map(move |a| (a, t)))
        {
            let (mut client, _peer) = buffered_peer(&record);
            let mut buf = Vec::new();
            match client.recv_record(&mut buf, timeout) {
                Ok(true) => {
                    assert_eq!(buf, record, "the delivered record must be the bytes sent");
                    match parse_record(&buf).expect("the delivered record must parse") {
                        Record::ImageFrame(frame) => {
                            assert_eq!(frame.payload.len(), payload_len);
                        }
                        other => panic!("expected an ImageFrame, got {other:?}"),
                    }
                }
                Ok(false) => panic!("a fully buffered record was reported as a clean EOF"),
                Err(error) => panic!(
                    "payload_len {payload_len}, timeout {timeout:?}, attempt {attempt}: a \
                     complete record was received and then discarded: {error} (kind {:?}, \
                     buf.len() {})",
                    error.kind(),
                    buf.len()
                ),
            }
        }
    }
}

/// Delivering a bodied record past its deadline must not weaken the deadline
/// at the payload read — the boundary the repair moved.
///
/// The prefix is complete and arrives, so `recv_record` gets past `record_len`,
/// past the [`MAX_RECORD_LEN`] check and into the payload read; but only half
/// the payload is buffered and the peer sends no more. That must still be a
/// timeout: [`std::io::ErrorKind::TimedOut`], an empty buffer, and a terminal
/// client. This is the case that separates "finish from bytes that arrived"
/// from "ignore the deadline"; break the repair so that the payload read waits
/// or succeeds regardless, and this test fails while the two above still pass.
#[test]
fn a_record_missing_payload_bytes_past_its_deadline_still_fails_and_is_terminal() {
    let record = bodied_frame_record(4096);
    let short = PREFIX_LEN + (record.len() - PREFIX_LEN) / 2;
    assert!(short > PREFIX_LEN && short < record.len());
    let (mut client, _peer) = buffered_peer(&record[..short]);
    let mut buf = Vec::new();
    let first = client
        .recv_record(&mut buf, Duration::from_micros(1))
        .expect_err("a record missing payload bytes past its deadline must fail");
    assert_eq!(first.kind(), std::io::ErrorKind::TimedOut);
    assert!(
        buf.is_empty(),
        "a record missing payload bytes must never be handed to the caller"
    );
    let second = client
        .recv_record(&mut buf, Duration::from_secs(1))
        .expect_err("a receive timeout is terminal");
    assert_eq!(second.kind(), first.kind());
    assert_eq!(second.to_string(), first.to_string());
}

/// A record finished past its deadline must leave the client able to *wait*
/// for the next one.
///
/// Every read is non-blocking, so the only thing that makes a later call wait
/// is the poll loop against that call's own deadline. An implementation that
/// treated "nothing queued" as a timeout, or that carried the first call's
/// expired deadline into the next, would turn a working connection into a
/// spurious timeout. Nothing about a single delivered record can show that, so
/// this reads two records over one client: the first past a 1 µs deadline, the
/// second with a generous deadline and its bytes written only *after* the
/// first call returned, so it can only be collected by a call that waits.
#[test]
fn a_record_finished_without_waiting_leaves_the_socket_blocking() {
    let first_record = bodied_frame_record(4096);
    let second_record = bodied_frame_record(64);
    let (mut client, mut peer) = buffered_peer(&first_record);
    let mut buf = Vec::new();
    assert!(
        client
            .recv_record(&mut buf, Duration::from_micros(1))
            .expect("the buffered record must be delivered"),
        "expected a record, not a clean EOF"
    );
    assert_eq!(buf, first_record);

    // Only now does the second record exist; the read that collects it has to
    // wait for it, which a socket left non-blocking cannot do.
    let (tx, rx) = mpsc::channel();
    let writer = thread::spawn(move || {
        rx.recv().expect("release");
        thread::sleep(Duration::from_millis(50));
        peer.write_all(&second_record).expect("write second record");
        peer.flush().expect("flush");
        peer
    });
    tx.send(()).expect("release writer");
    let delivered = client
        .recv_record(&mut buf, Duration::from_secs(5))
        .expect("a later record must still be collected by a waiting call");
    assert!(delivered, "expected a record, not a clean EOF");
    assert_eq!(buf, bodied_frame_record(64));
    let _peer = writer.join().expect("writer");
}

/// A twelve-byte prefix declaring `payload_len` payload bytes and nothing else.
///
/// Kind 99 with an empty header, as in
/// [`exact_record_cap_is_accepted_and_one_byte_over_is_refused`]: the two tests
/// below never deliver a record, so the kind is never parsed, and a declared
/// length of tens of megabytes must not cost tens of megabytes to *build*.
fn declared_prefix(payload_len: u32) -> Vec<u8> {
    let mut prefix = imagery::MAGIC.to_le_bytes().to_vec();
    prefix.extend_from_slice(&99u16.to_le_bytes());
    prefix.extend_from_slice(&0u16.to_le_bytes());
    prefix.extend_from_slice(&payload_len.to_le_bytes());
    assert_eq!(prefix.len(), PREFIX_LEN);
    prefix
}

/// **What a record typically costs past its deadline must not scale with the
/// length the peer declared.** The companion to the axis above, on the other
/// mechanism. *Typically* is load-bearing: every cell here is a median of
/// nine, and the paragraph on what this does not pin says what follows from
/// that.
///
/// Twelve bytes are all it takes: a well-formed prefix declaring a payload,
/// with nothing behind it and a deadline already spent. The call must fail,
/// and the *costliest* declaration must fail within a fixed budget of the
/// cheapest one — otherwise a peer buys work from the caller by naming a
/// number, and buys it *after* being told the call was over. On `2c3041f`,
/// which zero-filled the declared length, that cost rose linearly with the
/// declaration.
///
/// The receive path now never allocates the declared length: the buffer grows
/// with bytes received, and past the deadline by at most the 1 MiB room given
/// to the single final read. Every row above 1 MiB therefore does the same
/// bounded work. The ladder and both destination-buffer disciplines — a fresh
/// buffer and one reused across calls — are kept so that a regression to any
/// length-proportional allocation, at any point of the range, fails here.
///
/// **What this does not pin, said plainly.** A cell is a **median of nine**
/// calls, so the budget this asserts is *"no cell's median of nine exceeds 50×
/// the 64 KiB cell plus 10 ms"* — a bound on the typical cost, not on the worst
/// call, and not a time ceiling: no worst-case figure is published for this
/// call. The allocation bound itself is pinned deterministically by
/// `a_bare_prefix_does_not_grow_the_buffer_to_its_declared_length`. If this
/// test fails on a heavily loaded host, the budget is not the thing to change
/// — widening it until it passes buys nothing except a test that
/// discriminates less.
///
/// **Not a race and not a window.** Every row times out on every tree, so
/// there is no outcome to flake; the assertion is on the *ratio* between rows
/// of the same shape measured in the same process, with an absolute floor
/// under it so an idle-host microsecond cannot be divided into a false
/// failure. Medians of nine, because one descheduled call must not decide it.
#[test]
fn the_worst_declared_length_past_the_deadline_stays_within_a_fixed_budget() {
    const REPS: usize = 9;
    const SHORTEST: usize = 64 * 1024;
    fn median_cost(total: usize, reuse: bool) -> Duration {
        let mut shared = Vec::new();
        let mut costs = Vec::with_capacity(REPS);
        for _ in 0..REPS {
            let (mut client, _peer) = buffered_peer(&declared_prefix((total - PREFIX_LEN) as u32));
            let mut once = Vec::new();
            let buf: &mut Vec<u8> = if reuse { &mut shared } else { &mut once };
            let start = Instant::now();
            let outcome = client.recv_record(buf, Duration::from_nanos(1));
            costs.push(start.elapsed());
            assert_eq!(
                outcome
                    .expect_err("a record with no body bytes past its deadline must fail")
                    .kind(),
                std::io::ErrorKind::TimedOut
            );
        }
        costs.sort_unstable();
        costs[REPS / 2]
    }

    let ladder = [
        SHORTEST,
        4 * 1024 * 1024,
        31 * 1024 * 1024,
        32 * 1024 * 1024,
        MAX_RECORD_LEN,
    ];
    let mut small = Duration::MAX;
    let mut worst = Duration::ZERO;
    let mut worst_at = (0usize, false);
    let mut rows = String::new();
    for reuse in [false, true] {
        for total in ladder {
            let cost = median_cost(total, reuse);
            write!(
                rows,
                "\n  {total} bytes, {} buffer: {cost:?}",
                if reuse { "reused" } else { "fresh" }
            )
            .expect("a String never fails to be written to");
            if total == SHORTEST {
                small = small.min(cost);
            }
            if cost > worst {
                worst = cost;
                worst_at = (total, reuse);
            }
        }
    }
    let allowed = small * 50 + Duration::from_millis(10);
    let (worst_len, worst_reuse) = worst_at;
    assert!(
        worst <= allowed,
        "declaring {worst_len} bytes past an expired deadline cost {worst:?} with a {} \
         destination buffer, against {small:?} for a {SHORTEST}-byte declaration — a peer \
         must not be able to buy work by naming a length (allowed {allowed:?}){rows}",
        if worst_reuse { "reused" } else { "fresh" }
    );
}

/// **A record the receive queue cannot hold at once is not completed past its
/// deadline.** The narrowing the one-read bound costs, pinned rather than only
/// written down.
///
/// Bounding the post-deadline drain to a single read (of at most the larger of
/// 1 MiB and the space already allocated) means a record whose bytes the peer
/// has handed to the kernel, but which TCP flow control has parked in the
/// *sender's* buffer, is no longer finished late. The module docs say so; this
/// makes it fail if it ever stops being true, and makes the change visible
/// against `2c3041f`, which delivers every one of these.
///
/// **Writer completion is synchronized.** The peer writes the whole record
/// with one blocking `write_all`, and a channel confirms that call *returns*
/// before `recv_record` is entered. There is no application writer left to
/// refill the queue while the assertion runs. What makes the record
/// uncompletable is only that this end's receive queue does not hold it all at
/// once — which the test **measures**, with a `peek` buffer at least as large
/// as the whole record, rather than assuming from a socket-buffer constant,
/// and asserts before it asserts anything else. On a cold loopback connection
/// here that queue settles at 98 304 bytes on every rep, so 256 KiB is already
/// past it. Measured: on
/// `2c3041f`, which drains until a non-blocking read finds nothing, all
/// **60 of 60** of these are delivered (20 reps × 3 sizes, both profiles);
/// with the drain bound to one read, **0 of 60**.
#[test]
fn a_record_too_long_for_the_receive_queue_is_not_completed_past_its_deadline() {
    const REPS: usize = 5;
    for total in [256 * 1024usize, 1024 * 1024, 2 * 1024 * 1024] {
        for rep in 0..REPS {
            let mut record = declared_prefix((total - PREFIX_LEN) as u32);
            record.resize(total, 0x5a);

            let listener =
                TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).expect("bind");
            let addr = listener.local_addr().expect("address");
            let client_stream = TcpStream::connect(addr).expect("connect");
            let (mut server_stream, _) = listener.accept().expect("accept");

            let (written_tx, written_rx) = mpsc::channel();
            let writer = thread::spawn(move || {
                server_stream
                    .write_all(&record)
                    .expect("write the whole record");
                server_stream.flush().expect("flush");
                written_tx.send(()).expect("announce");
                server_stream
            });
            written_rx
                .recv_timeout(Duration::from_secs(20))
                .expect("the peer's blocking write_all must return before the call");

            // Every byte has left the peer's write_all now. How many are at
            // this end? The probe is the whole record, so `queued < total`
            // proves that one read cannot finish it.
            client_stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .expect("probe timeout");
            let mut probe = vec![0u8; total];
            let queued = client_stream.peek(&mut probe).expect("peek");
            assert!(
                queued < total,
                "this connection's receive queue holds all {total} bytes ({queued} peeked), so \
                 there is nothing for this test to distinguish — rep {rep}"
            );

            let mut client =
                ImageClient::from_stream(client_stream, Duration::from_secs(1)).expect("adopt");
            let mut buf = b"stale record".to_vec();
            let first = client
                .recv_record(&mut buf, Duration::from_nanos(1))
                .expect_err(
                    "a record the receive queue cannot hold at once must not be completed \
                     past its deadline",
                );
            assert_eq!(
                first.kind(),
                std::io::ErrorKind::TimedOut,
                "total {total}, {queued} of it queued, rep {rep}"
            );
            assert!(
                buf.is_empty(),
                "the incomplete record must clear the buffer"
            );
            let second = client
                .recv_record(&mut buf, Duration::from_secs(1))
                .expect_err("a receive timeout is terminal");
            assert_eq!(second.kind(), first.kind());
            assert_eq!(second.to_string(), first.to_string());
            drop(client);
            let _peer = writer.join().expect("writer");
        }
    }
}

/// A connected loopback pair: the client end adopted by nothing yet, and the
/// peer end the test writes from.
fn loopback_pair() -> (TcpStream, TcpStream) {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = listener.local_addr().expect("address");
    let client_stream = TcpStream::connect(addr).expect("connect");
    let (peer, _) = listener.accept().expect("accept");
    (client_stream, peer)
}

/// Writes `bytes` from `peer` once released, after a configured delay, so the
/// client has to wait for them.
fn delayed_writer(
    mut peer: TcpStream,
    bytes: Vec<u8>,
) -> (mpsc::Sender<()>, thread::JoinHandle<TcpStream>) {
    let (tx, rx) = mpsc::channel::<()>();
    let writer = thread::spawn(move || {
        rx.recv().expect("release writer");
        thread::sleep(Duration::from_millis(50));
        peer.write_all(&bytes).expect("write delayed bytes");
        peer.flush().expect("flush");
        peer
    });
    (tx, writer)
}

/// An adopted socket that is already non-blocking — as an accepted socket can
/// be on platforms where `accept` inherits the listener's mode — must still
/// wait for a record that arrives later instead of failing at once.
#[test]
fn an_inherited_nonblocking_socket_still_waits_for_its_record() {
    let (client_stream, peer) = loopback_pair();
    client_stream
        .set_nonblocking(true)
        .expect("simulate an inherited non-blocking socket");
    let mut client =
        ImageClient::from_stream(client_stream, Duration::from_secs(1)).expect("adopt");
    let (release, writer) = delayed_writer(peer, imagery::heartbeat_record());
    release.send(()).expect("release");
    let mut buf = Vec::new();
    let delivered = client
        .recv_record(&mut buf, Duration::from_secs(5))
        .expect("a record written later must be waited for");
    assert!(delivered, "expected a record, not a clean EOF");
    assert_eq!(buf, imagery::heartbeat_record());
    let _peer = writer.join().expect("writer");
}

/// A record deadline that passes before any byte of the next record has
/// arrived is an idle poll: `TimedOut`, but not terminal, with the caller's
/// buffer untouched. A heartbeat sent afterwards is still received.
#[test]
fn an_idle_timeout_is_not_terminal_and_a_later_heartbeat_is_received() {
    let (client_stream, peer) = loopback_pair();
    let mut client =
        ImageClient::from_stream(client_stream, Duration::from_secs(1)).expect("adopt");
    let mut buf = b"previous record".to_vec();
    for poll in 0..2 {
        let started = Instant::now();
        let idle = client
            .recv_record(&mut buf, Duration::from_millis(30))
            .expect_err("nothing was sent");
        assert_eq!(
            idle.kind(),
            std::io::ErrorKind::TimedOut,
            "poll {poll}: {idle:?}"
        );
        assert!(
            started.elapsed() >= Duration::from_millis(30),
            "poll {poll} returned before its deadline"
        );
        assert!(
            !client.is_terminal(),
            "poll {poll}: an idle timeout must not be terminal"
        );
        assert_eq!(
            buf, b"previous record",
            "poll {poll}: an idle timeout leaves the buffer"
        );
    }

    let (release, writer) = delayed_writer(peer, imagery::heartbeat_record());
    release.send(()).expect("release");
    assert!(
        client
            .recv_record(&mut buf, Duration::from_secs(5))
            .expect("the heartbeat after idle polls must be received")
    );
    assert_eq!(buf, imagery::heartbeat_record());
    assert!(!client.is_terminal());
    let _peer = writer.join().expect("writer");
}

/// A bare twelve-byte prefix declaring the largest accepted record must not
/// make the client allocate (or zero) that declared length: the receive buffer
/// grows with bytes actually received. The incomplete record is a timeout
/// after a consumed prefix, so it stays terminal.
#[test]
fn a_bare_prefix_does_not_grow_the_buffer_to_its_declared_length() {
    const GROWTH_BOUND: usize = 4 << 20;
    for timeout in [Duration::from_millis(60), Duration::from_nanos(1)] {
        let (mut client, _peer) =
            buffered_peer(&declared_prefix((MAX_RECORD_LEN - PREFIX_LEN) as u32));
        let mut buf = Vec::new();
        let first = client
            .recv_record(&mut buf, timeout)
            .expect_err("a record with no body bytes must time out");
        assert_eq!(
            first.kind(),
            std::io::ErrorKind::TimedOut,
            "{timeout:?}: {first:?}"
        );
        assert!(buf.is_empty());
        assert!(
            buf.capacity() < GROWTH_BOUND,
            "{timeout:?}: twelve bytes grew the buffer to {} bytes",
            buf.capacity()
        );
        assert!(
            client.is_terminal(),
            "a timeout after a consumed prefix is terminal"
        );
        let second = client
            .recv_record(&mut buf, Duration::from_secs(1))
            .expect_err("terminal");
        assert_eq!(second.to_string(), first.to_string());
    }
}
