//! Message ids, fire-and-forget command datagrams and NAT_MESSAGESTRING text.
#![allow(
    clippy::expect_used,
    reason = "test failures should name the violated expectation"
)]

use natnet::command;
use natnet::{
    Error, MessageHeader, NAT_CONNECT, NAT_DISCONNECT, NAT_DISCONNECTBYTIMEOUT, NAT_DISCOVERY,
    NAT_ECHOREQUEST, NAT_ECHORESPONSE, NAT_FRAMEOFDATA, NAT_KEEPALIVE, NAT_MESSAGESTRING,
    NAT_MODELDEF, NAT_REQUEST, NAT_REQUEST_FRAMEOFDATA, NAT_REQUEST_MODELDEF, NAT_RESPONSE,
    NAT_SERVERINFO, NAT_UNRECOGNIZED_REQUEST, parse_message_string,
};

/// The ids `NatNetTypes.h` (NatNet SDK 4.1.0.0) defines, in order.
#[test]
fn message_ids_match_the_sdk_header() {
    let ids = [
        (NAT_CONNECT, 0),
        (NAT_SERVERINFO, 1),
        (NAT_REQUEST, 2),
        (NAT_RESPONSE, 3),
        (NAT_REQUEST_MODELDEF, 4),
        (NAT_MODELDEF, 5),
        (NAT_REQUEST_FRAMEOFDATA, 6),
        (NAT_FRAMEOFDATA, 7),
        (NAT_MESSAGESTRING, 8),
        (NAT_DISCONNECT, 9),
        (NAT_KEEPALIVE, 10),
        (NAT_DISCONNECTBYTIMEOUT, 11),
        (NAT_ECHOREQUEST, 12),
        (NAT_ECHORESPONSE, 13),
        (NAT_DISCOVERY, 14),
        (NAT_UNRECOGNIZED_REQUEST, 100),
    ];
    for (constant, expected) in ids {
        assert_eq!(constant, expected);
    }
    #[cfg(feature = "net")]
    assert_eq!(
        natnet::net::NAT_UNRECOGNIZED_REQUEST,
        NAT_UNRECOGNIZED_REQUEST
    );
}

#[test]
fn keepalive_and_disconnect_are_bare_headers() {
    for (datagram, id) in [
        (command::keepalive_datagram(), NAT_KEEPALIVE),
        (command::disconnect_datagram(), NAT_DISCONNECT),
    ] {
        assert_eq!(datagram.len(), 4);
        let header = MessageHeader::parse(&datagram).expect("header");
        assert_eq!(header.message_id, id);
        assert_eq!(header.declared_size, 0);
    }
    assert_eq!(command::keepalive_datagram(), [10, 0, 0, 0]);
    assert_eq!(command::disconnect_datagram(), [9, 0, 0, 0]);
}

#[test]
fn request_frame_of_data_is_an_empty_request() {
    let request = command::request_frame_of_data().expect("fixed request");
    assert_eq!(request.as_bytes(), &[6, 0, 0, 0]);
}

fn message_string(payload: &[u8]) -> Vec<u8> {
    let mut datagram = NAT_MESSAGESTRING.to_le_bytes().to_vec();
    datagram.extend_from_slice(
        &u16::try_from(payload.len())
            .expect("short test payload")
            .to_le_bytes(),
    );
    datagram.extend_from_slice(payload);
    datagram
}

#[test]
fn message_string_ends_at_the_first_nul() {
    let text = parse_message_string(&message_string(b"Recording started\0stale")).expect("text");
    assert_eq!(text, "Recording started");
}

#[test]
fn message_string_without_nul_uses_the_whole_payload() {
    assert_eq!(
        parse_message_string(&message_string(b"no terminator")).expect("text"),
        "no terminator"
    );
    assert_eq!(
        parse_message_string(&message_string(b"")).expect("text"),
        ""
    );
}

#[test]
fn message_string_replaces_invalid_utf8() {
    let text = parse_message_string(&message_string(b"a\xffb\0")).expect("text");
    assert_eq!(text, "a\u{fffd}b");
}

#[test]
fn message_string_refuses_other_messages_and_short_datagrams() {
    let keepalive = command::keepalive_datagram();
    assert!(matches!(
        parse_message_string(&keepalive),
        Err(Error::UnexpectedMessageId {
            found: 10,
            expected: 8
        })
    ));
    assert!(matches!(
        parse_message_string(&[8, 0, 0]),
        Err(Error::UnexpectedEof { .. })
    ));
}
