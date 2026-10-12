//! Deterministic payload reach and mutation regressions for the public parsers.
//!
//! Fixtures are assembled from wire fields, not serializers, so a
//! serializer/parser pair cannot share the same mistake.

#![allow(clippy::expect_used)]

use std::panic::{AssertUnwindSafe, catch_unwind};

use natnet::command::CommandResponse;
use natnet::{Error, FrameOfData, Message, ModelDef, ParsePolicy, ServerInfo, Version};

fn push_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_f64(out: &mut Vec<u8>, value: f64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn section(out: &mut Vec<u8>, count: i32, body: &[u8]) {
    push_i32(out, count);
    push_i32(
        out,
        i32::try_from(body.len()).expect("literal section fits"),
    );
    out.extend_from_slice(body);
}

fn envelope(message_id: u16, declared_size: u16, payload: &[u8]) -> Vec<u8> {
    let mut datagram = Vec::with_capacity(4 + payload.len());
    datagram.extend_from_slice(&message_id.to_le_bytes());
    datagram.extend_from_slice(&declared_size.to_le_bytes());
    datagram.extend_from_slice(payload);
    datagram
}

/// A small V4.1 frame with one marker set and one rigid body. Every section
/// header and the timing tail is present, including the two V4.1 precision
/// fields and end marker.
fn frame_payload() -> Vec<u8> {
    let mut out = Vec::new();
    push_i32(&mut out, 42);

    let mut marker_sets = Vec::new();
    marker_sets.extend_from_slice(b"wand\0");
    push_i32(&mut marker_sets, 1);
    push_f32(&mut marker_sets, 1.0);
    push_f32(&mut marker_sets, 2.0);
    push_f32(&mut marker_sets, 3.0);
    section(&mut out, 1, &marker_sets);

    section(&mut out, 0, &[]);

    let mut rigid_bodies = Vec::new();
    push_i32(&mut rigid_bodies, 7);
    push_f32(&mut rigid_bodies, 1.0);
    push_f32(&mut rigid_bodies, 2.0);
    push_f32(&mut rigid_bodies, 3.0);
    push_f32(&mut rigid_bodies, 0.0);
    push_f32(&mut rigid_bodies, 0.0);
    push_f32(&mut rigid_bodies, 0.0);
    push_f32(&mut rigid_bodies, 1.0);
    push_f32(&mut rigid_bodies, 0.01);
    push_u16(&mut rigid_bodies, 1);
    section(&mut out, 1, &rigid_bodies);

    section(&mut out, 0, &[]); // skeletons
    section(&mut out, 0, &[]); // assets
    section(&mut out, 0, &[]); // labelled markers
    section(&mut out, 0, &[]); // force plates
    section(&mut out, 0, &[]); // devices

    push_u32(&mut out, 0x0102_0304);
    push_u32(&mut out, 5);
    push_f64(&mut out, 0.35);
    push_u64(&mut out, 11);
    push_u64(&mut out, 12);
    push_u64(&mut out, 13);
    push_u32(&mut out, 14);
    push_u32(&mut out, 15);
    push_u16(&mut out, 0);
    push_i32(&mut out, 0);
    out
}

/// A literal MODELDEF with a marker set and an unknown length-delimited
/// dataset. The latter pins preservation of opaque bytes while the marker set
/// exercises a nested count and two terminated strings.
fn model_payload() -> Vec<u8> {
    let mut out = Vec::new();
    push_i32(&mut out, 2);

    let marker_set = b"wand\0\x01\0\0\0m1\0";
    push_i32(&mut out, 0);
    push_i32(
        &mut out,
        i32::try_from(marker_set.len()).expect("literal dataset fits"),
    );
    out.extend_from_slice(marker_set);

    push_i32(&mut out, 99);
    push_i32(&mut out, 3);
    out.extend_from_slice(&[0xa1, 0xb2, 0xc3]);
    out
}

/// A literal 279-byte SERVERINFO payload. The nonzero bytes after the name's
/// terminator model stale fixed-buffer contents and prove the NUL-stop path.
fn server_payload() -> Vec<u8> {
    let mut out = vec![0u8; 256];
    out[..6].copy_from_slice(b"Motive");
    out[6] = 0;
    out[7..15].copy_from_slice(b".195:301");
    out.extend_from_slice(&[3, 1, 2, 1]);
    out.extend_from_slice(&[4, 1, 0, 0]);
    push_u64(&mut out, 10_000_000);
    push_u16(&mut out, 1511);
    out.push(1);
    out.extend_from_slice(&[239, 255, 42, 99]);
    out
}

fn command_payload() -> Vec<u8> {
    b"ok\0".to_vec()
}

/// A literal AUGI/1 image record: 12-byte prefix, the 40-byte image header,
/// and a three-byte payload. The record is intentionally not produced by an
/// imagery serializer.
#[cfg(feature = "imagery")]
fn image_record() -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&natnet::imagery::MAGIC.to_le_bytes());
    out.extend_from_slice(&natnet::imagery::KIND_IMAGE_FRAME.to_le_bytes());
    out.extend_from_slice(&(natnet::imagery::IMAGE_FRAME_HEADER_LEN as u16).to_le_bytes());
    out.extend_from_slice(&3u32.to_le_bytes());

    push_u32(&mut out, 42);
    push_u32(&mut out, 2);
    push_u64(&mut out, 100);
    push_u64(&mut out, 101);
    push_f64(&mut out, 0.5);
    push_u16(&mut out, 2);
    push_u16(&mut out, 1);
    out.push(natnet::imagery::ImageEncoding::Mono8.to_wire());
    out.push(natnet::imagery::FLAG_MID_EXPOSURE_VALID);
    push_u16(&mut out, 0);
    out.extend_from_slice(&[0xaa, 0xbb, 0xcc]);
    out
}

fn assert_prefixes_error<F>(label: &str, input: &[u8], mut parse: F)
where
    F: FnMut(&[u8]) -> bool + std::panic::UnwindSafe,
{
    for length in 0..input.len() {
        let result = catch_unwind(AssertUnwindSafe(|| parse(&input[..length])));
        assert!(result.is_ok(), "{label} panicked at prefix length {length}");
        assert!(
            result.expect("panic checked"),
            "{label} accepted truncated prefix length {length}"
        );
    }
}

fn assert_mutations_do_not_panic<F>(label: &str, input: &[u8], mut parse: F, offsets: &[usize])
where
    F: FnMut(&[u8]) + std::panic::UnwindSafe,
{
    for &offset in offsets {
        let mut mutated = input.to_vec();
        mutated[offset] ^= 0x5a;
        let result = catch_unwind(AssertUnwindSafe(|| parse(&mutated)));
        assert!(
            result.is_ok(),
            "{label} panicked after mutating byte {offset}"
        );
    }
}

#[test]
fn valid_envelopes_dispatch_and_ignore_telemetry_size() {
    let server = server_payload();
    let model = model_payload();
    let frame = frame_payload();

    let server_datagram = envelope(natnet::NAT_SERVERINFO, 0, &server);
    let model_datagram = envelope(natnet::NAT_MODELDEF, u16::MAX, &model);
    let frame_datagram = envelope(natnet::NAT_FRAMEOFDATA, 1, &frame);

    let parsed_server = natnet::Decoder::new(Version::V4_1)
        .expect("supported")
        .with_policy(natnet::ParsePolicy::Compatible)
        .parse(&server_datagram)
        .map(natnet::DecodedMessage::into_message)
        .expect("SERVERINFO");
    assert!(
        matches!(parsed_server, Message::ServerInfo(info) if info.application_name == "Motive")
    );

    let parsed_model = natnet::Decoder::new(Version::V4_1)
        .expect("supported")
        .with_policy(natnet::ParsePolicy::Compatible)
        .parse(&model_datagram)
        .map(natnet::DecodedMessage::into_message)
        .expect("MODELDEF");
    assert!(matches!(parsed_model, Message::ModelDef(def) if def.datasets.len() == 2));

    let parsed_frame = natnet::Decoder::new(Version::V4_1)
        .expect("supported")
        .with_policy(natnet::ParsePolicy::Compatible)
        .parse(&frame_datagram)
        .map(natnet::DecodedMessage::into_message)
        .expect("frame");
    assert!(matches!(parsed_frame, Message::FrameOfData(frame) if frame.frame_number == 42));

    let command_datagram = envelope(natnet::NAT_RESPONSE, 0, &command_payload());
    assert_eq!(
        CommandResponse::parse(&command_datagram).expect("command response"),
        CommandResponse::Value("ok".to_owned())
    );
    let status_payload = 0i32.to_le_bytes();
    assert_eq!(
        CommandResponse::parse_payload(&status_payload).expect("status payload"),
        CommandResponse::Status(0)
    );
    assert_eq!(
        CommandResponse::parse_value_payload(&status_payload).expect("status value reading"),
        ""
    );
}

#[test]
fn unknown_id_preserves_bytes_and_declared_size_is_telemetry() {
    let payload = [0x00, 0x7f, 0xa5, 0xff];
    let datagram = envelope(0xbeef, 0, &payload);
    let parsed = natnet::Decoder::new(Version::V4_1)
        .expect("supported")
        .with_policy(natnet::ParsePolicy::Compatible)
        .parse(&datagram)
        .map(natnet::DecodedMessage::into_message)
        .expect("unknown message");
    assert!(matches!(
        parsed,
        Message::Other { id: 0xbeef, payload: got } if got == payload
    ));
}

#[test]
fn every_typed_seed_is_safely_truncated_at_each_byte() {
    let frame = frame_payload();
    assert!(FrameOfData::parse_payload(&frame, Version::V4_1).is_ok());
    assert_prefixes_error("frame payload", &frame, |bytes| {
        FrameOfData::parse_payload(bytes, Version::V4_1).is_err()
    });

    let model = model_payload();
    assert!(ModelDef::parse_payload(&model, Version::V4_1).is_ok());
    assert_prefixes_error("MODELDEF payload", &model, |bytes| {
        ModelDef::parse_payload(bytes, Version::V4_1).is_err()
    });

    let server = server_payload();
    assert!(ServerInfo::parse_payload(&server).is_ok());
    assert_prefixes_error("SERVERINFO payload", &server, |bytes| {
        ServerInfo::parse_payload(bytes).is_err()
    });

    // A command datagram shorter than its four-byte header is refused. Past
    // the header, a cut value reads as the text before the cut, because
    // Motive sends values without a NUL terminator.
    let command = envelope(natnet::NAT_RESPONSE, 0, &command_payload());
    assert!(CommandResponse::parse(&command).is_ok());
    assert_prefixes_error("command datagram header", &command[..4], |bytes| {
        CommandResponse::parse(bytes).is_err()
    });

    let command_value = command_payload();
    assert_eq!(
        CommandResponse::parse_value_payload(&command_value).expect("value payload"),
        "ok"
    );
    for (length, text) in [(0, ""), (1, "o"), (2, "ok")] {
        assert_eq!(
            CommandResponse::parse(&command[..4 + length]).expect("cut datagram"),
            CommandResponse::Value(text.to_owned()),
            "command datagram cut after {length} payload bytes"
        );
        assert_eq!(
            CommandResponse::parse_value_payload(&command_value[..length]).expect("cut payload"),
            text,
            "command value payload cut after {length} bytes"
        );
    }

    #[cfg(feature = "imagery")]
    {
        let record = image_record();
        assert!(natnet::imagery::parse_record(&record).is_ok());
        assert_prefixes_error("imagery record", &record, |bytes| {
            natnet::imagery::parse_record(bytes).is_err()
        });

        for length in 0..=record.len() {
            let result = catch_unwind(|| natnet::imagery::record_len(&record[..length]));
            assert!(
                result.is_ok(),
                "record_len panicked at prefix length {length}"
            );
            let length_result = result.expect("panic checked").expect("record length");
            if length < natnet::imagery::PREFIX_LEN {
                assert_eq!(
                    length_result, None,
                    "short imagery prefix must request more bytes"
                );
            } else {
                assert_eq!(
                    length_result,
                    Some(record.len()),
                    "a complete imagery prefix determines the full record length"
                );
            }
        }
        let total = natnet::imagery::record_len(&record)
            .expect("record length")
            .expect("complete prefix");
        assert_eq!(total, record.len());
    }
}

#[test]
fn structured_mutations_reach_meaningful_error_classes() {
    let frame = frame_payload();

    let mut section_residue = frame.clone();
    section_residue[8..12].copy_from_slice(&22i32.to_le_bytes());
    assert!(matches!(
        FrameOfData::parse_payload(&section_residue, Version::V4_1),
        Err(Error::SectionSizeMismatch {
            declared: 22,
            consumed: 21
        })
    ));

    let mut negative_marker_count = frame.clone();
    negative_marker_count[4..8].copy_from_slice(&(-1i32).to_le_bytes());
    assert!(matches!(
        FrameOfData::parse_payload(&negative_marker_count, Version::V4_1),
        Err(Error::ImplausibleCount { .. } | Error::InvalidCount { .. })
    ));

    let mut unterminated_frame_name = frame;
    for byte in &mut unterminated_frame_name[16..] {
        if *byte == 0 {
            *byte = 1;
        }
    }
    assert!(matches!(
        FrameOfData::parse_payload(&unterminated_frame_name, Version::V4_1),
        Err(Error::UnterminatedString { .. })
    ));

    let model = model_payload();
    let mut model_count = model.clone();
    model_count[0..4].copy_from_slice(&(-1i32).to_le_bytes());
    assert!(matches!(
        ModelDef::parse_payload(&model_count, Version::V4_1),
        Err(Error::ImplausibleCount { .. } | Error::InvalidCount { .. })
    ));

    let mut model_name = model;
    for byte in &mut model_name[12..24] {
        if *byte == 0 {
            *byte = b'X';
        }
    }
    assert!(matches!(
        ModelDef::parse_payload(&model_name, Version::V4_1),
        Err(Error::UnterminatedString { .. })
    ));

    assert!(matches!(
        ServerInfo::parse_payload(&server_payload()[..278]),
        Err(Error::UnexpectedEof { .. })
    ));

    // An unterminated value is text, not an error: Motive sends values
    // without a NUL terminator.
    assert_eq!(
        CommandResponse::parse_payload(b"bad").expect("unterminated value"),
        CommandResponse::Value("bad".to_owned())
    );
    assert!(matches!(
        CommandResponse::parse(&envelope(2, 3, b"ok\0")),
        Err(Error::UnexpectedMessageId {
            found: 2,
            expected: natnet::NAT_RESPONSE
        })
    ));

    #[cfg(feature = "imagery")]
    {
        let record = image_record();
        let mut bad_magic = record.clone();
        bad_magic[0] ^= 1;
        assert!(matches!(
            natnet::imagery::record_len(&bad_magic),
            Err(Error::InvalidMagic { .. })
        ));

        let mut oversized_payload = record.clone();
        oversized_payload[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        let oversized_result = natnet::imagery::parse_record(&oversized_payload);
        if cfg!(target_pointer_width = "32") {
            // Prefix + header + u32::MAX cannot be represented on i686.
            assert!(matches!(
                oversized_result,
                Err(Error::ImplausibleCount { count, available })
                    if count == usize::MAX && available == oversized_payload.len()
            ));
        } else {
            assert!(matches!(oversized_result, Err(Error::UnexpectedEof { .. })));
        }

        let mut short_header = record;
        short_header[6..8].copy_from_slice(&1u16.to_le_bytes());
        assert!(matches!(
            natnet::imagery::parse_record(&short_header),
            Err(Error::UnexpectedEof { .. })
        ));
    }
}

#[test]
fn explicit_parse_policy_pins_compatibility_and_strict_errors() {
    let frame = frame_payload();
    assert!(
        FrameOfData::parse_payload_with_policy(&frame, Version::V4_1, ParsePolicy::Strict).is_ok()
    );

    let mut bad_terminator = frame.clone();
    let end = bad_terminator.len() - 4;
    bad_terminator[end..].copy_from_slice(&7i32.to_le_bytes());
    assert!(FrameOfData::parse_payload(&bad_terminator, Version::V4_1).is_ok());
    assert!(matches!(
        FrameOfData::parse_payload_with_policy(&bad_terminator, Version::V4_1, ParsePolicy::Strict),
        Err(Error::InvalidTerminator { value: 7, .. })
    ));

    let mut frame_suffix = frame.clone();
    frame_suffix.push(0xee);
    assert!(FrameOfData::parse_payload(&frame_suffix, Version::V4_1).is_ok());
    assert!(matches!(
        FrameOfData::parse_payload_with_policy(&frame_suffix, Version::V4_1, ParsePolicy::Strict),
        Err(Error::TrailingBytes { remaining: 1 })
    ));

    let mut bad_typed_count = frame;
    bad_typed_count[4..8].copy_from_slice(&(-1i32).to_le_bytes());
    assert_eq!(
        FrameOfData::parse_payload(&bad_typed_count, Version::V4_1),
        Err(Error::InvalidCount {
            count: -1,
            offset: 4
        })
    );
    assert_eq!(
        FrameOfData::parse_payload_with_policy(
            &bad_typed_count,
            Version::V4_1,
            ParsePolicy::Strict
        ),
        Err(Error::InvalidCount {
            count: -1,
            offset: 4
        })
    );

    let model = model_payload();
    assert!(ModelDef::parse_payload(&model, Version::V4_1).is_ok());
    let mut model_suffix = model;
    model_suffix.push(0xdd);
    assert!(ModelDef::parse_payload(&model_suffix, Version::V4_1).is_ok());
    assert!(matches!(
        ModelDef::parse_payload_with_policy(&model_suffix, Version::V4_1, ParsePolicy::Strict),
        Err(Error::TrailingBytes { remaining: 1 })
    ));

    let server = server_payload();
    let mut server_suffix = server;
    server_suffix.push(0xcc);
    assert!(ServerInfo::parse_payload(&server_suffix).is_ok());
    assert!(matches!(
        ServerInfo::parse_payload_with_policy(&server_suffix, ParsePolicy::Strict),
        Err(Error::TrailingBytes { remaining: 1 })
    ));
}

#[test]
fn deterministic_byte_mutations_never_panic_at_each_entrypoint() {
    let frame = frame_payload();
    assert_mutations_do_not_panic(
        "frame payload",
        &frame,
        |bytes| {
            let _ = FrameOfData::parse_payload(bytes, Version::V4_1);
        },
        &[0, 4, 8, 16, 21, 44, frame.len() - 1],
    );

    let model = model_payload();
    assert_mutations_do_not_panic(
        "MODELDEF payload",
        &model,
        |bytes| {
            let _ = ModelDef::parse_payload(bytes, Version::V4_1);
        },
        &[0, 4, 8, 12, 16, model.len() - 1],
    );

    let server = server_payload();
    assert_mutations_do_not_panic(
        "SERVERINFO payload",
        &server,
        |bytes| {
            let _ = ServerInfo::parse_payload(bytes);
        },
        &[0, 6, 256, 260, 272, server.len() - 1],
    );

    let command = envelope(natnet::NAT_RESPONSE, 3, &command_payload());
    assert_mutations_do_not_panic(
        "command datagram",
        &command,
        |bytes| {
            let _ = CommandResponse::parse(bytes);
        },
        &[0, 2, 4, 6],
    );
    let command_value = command_payload();
    assert_mutations_do_not_panic(
        "command value payload",
        &command_value,
        |bytes| {
            let _ = CommandResponse::parse_value_payload(bytes);
        },
        &[0, 1, 2],
    );

    #[cfg(feature = "imagery")]
    {
        let record = image_record();
        assert_mutations_do_not_panic(
            "imagery record length",
            &record,
            |bytes| {
                let _ = natnet::imagery::record_len(bytes);
            },
            &[0, 4, 6, 8, 11, record.len() - 1],
        );
        assert_mutations_do_not_panic(
            "imagery record parser",
            &record,
            |bytes| {
                let _ = natnet::imagery::parse_record(bytes);
            },
            &[0, 4, 6, 8, 12, 20, record.len() - 1],
        );
    }
}
