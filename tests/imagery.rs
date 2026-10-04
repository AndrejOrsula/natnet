//! AUGI/1 record round-trips, framing edge cases and forward compatibility.
//!
//! `parse_record(&x.to_record()?)? == x` for every record type, plus the four
//! behaviours the format's versioning rests on: an unknown kind survives as
//! [`Record::Other`], a truncated buffer returns an [`Err`] and never panics,
//! a header longer than this crate knows is skipped, and a header shorter
//! than it knows is refused.
//!
//! Encoding is fallible: values a serializer would have to distort are
//! refused. Those refusals have their own section below.
//!
//! One golden byte vector guards the wire layout independently of the code
//! under test.

#![cfg(feature = "imagery")]
#![allow(
    clippy::unwrap_used,
    reason = "the integration harness unwraps values immediately after asserting their shape"
)]

use natnet::imagery::{
    self, CAMERA_DESC_HEADER_LEN, CameraDesc, ENCODING_NONE, HELLO_HEADER_LEN, Hello,
    IMAGE_FRAME_HEADER_LEN, ImageEncoding, ImageFrame, KIND_CAMERA_DESC, KIND_HELLO,
    KIND_IMAGE_FRAME, MAGIC, MODE_LISTEN, PREFIX_LEN, PROTO_MAJOR, PROTO_MINOR, Record,
    encode_record, parse_record, record_len,
};
use natnet::{Decoder, FrameOfData, Message, SyntheticServer, Version};

// ---------------------------------------------------------------- fixtures

fn sample_hello() -> Hello {
    Hello {
        proto_major: PROTO_MAJOR,
        proto_minor: PROTO_MINOR,
        clock_hz: 10_000_000,
        qpf: 10_000_000,
        natnet_major: 4,
        natnet_minor: 1,
        motive_build: 31_021,
        // Deliberately 2021: a Motive host clock may be wrong on purpose and
        // this field is provenance, never a key.
        host_wall_unix_secs: 1_612_137_600,
        mode: MODE_LISTEN,
        camera_count: 14,
        bridge_id: "example-bridge".to_string(),
        profile: r"C:\ProgramData\OptiTrack\MotiveProfile.motive".to_string(),
        calibration: r"C:\ProgramData\OptiTrack\Motive\System Calibration.cal".to_string(),
    }
}

/// A 2x2 ray grid: 4 samples of 6 f32.
fn sample_rays() -> Vec<u8> {
    let mut out = Vec::new();
    for i in 0..4u32 {
        for c in 0..6u32 {
            out.extend_from_slice(&(i as f32 * 10.0 + c as f32).to_le_bytes());
        }
    }
    out
}

fn sample_camera_desc(rays: &[u8]) -> CameraDesc<'_> {
    CameraDesc {
        camera_index: 3,
        camera_id: 4,
        serial: 12_345,
        video_mode: 1, // Grayscale
        encoding: ImageEncoding::Mono8,
        contributes_to_solve: true,
        frame_rate_divisor: 12,
        width: 640,
        height: 512,
        position: [1.0, 2.0, 3.0],
        orientation: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        ray_grid_w: 2,
        ray_grid_h: 2,
        rays,
        name: "Prime 13W #3",
    }
}

fn sample_image(payload: &[u8], mid: Option<u64>) -> ImageFrame<'_> {
    ImageFrame {
        frame_number: 15_732_032,
        camera_index: 3,
        camera_mid_exposure: mid,
        grab_ticks: 0x0000_0001_0000_0002,
        api_timestamp_secs: 131_100.266_7,
        width: 640,
        height: 512,
        encoding: ImageEncoding::Mono8,
        payload,
    }
}

// ------------------------------------------------------------- round-trips

#[test]
fn hello_round_trips() {
    let hello = sample_hello();
    let bytes = hello.to_record().expect("encode");
    assert_eq!(record_len(&bytes).unwrap(), Some(bytes.len()));
    match parse_record(&bytes).expect("parse") {
        Record::Hello(back) => assert_eq!(hello, back),
        other => panic!("expected Hello, got {other:?}"),
    }
}

#[test]
fn camera_desc_round_trips_and_decodes_rays() {
    let rays = sample_rays();
    let desc = sample_camera_desc(&rays);
    let bytes = desc.to_record().expect("encode");
    match parse_record(&bytes).expect("parse") {
        Record::CameraDesc(back) => {
            assert_eq!(desc, back);
            assert_eq!(back.name, "Prime 13W #3");
            // Grid index (1, 1) is the fourth sample, base value 30.
            assert_eq!(back.ray(1, 1), Some([30.0, 31.0, 32.0, 33.0, 34.0, 35.0]));
            assert_eq!(back.ray(0, 0), Some([0.0, 1.0, 2.0, 3.0, 4.0, 5.0]));
            assert_eq!(back.ray(2, 0), None, "column outside the grid");
            assert_eq!(back.ray(0, 2), None, "row outside the grid");
        }
        other => panic!("expected CameraDesc, got {other:?}"),
    }
}

#[test]
fn image_frame_round_trips() {
    let payload: Vec<u8> = (0..64u8).collect();
    let frame = sample_image(&payload, Some(0x07ca_b13f_00e3_4100));
    let bytes = frame.to_record().expect("encode");
    match parse_record(&bytes).expect("parse") {
        Record::ImageFrame(back) => {
            assert_eq!(frame, back);
            assert_eq!(back.payload, &payload[..]);
        }
        other => panic!("expected ImageFrame, got {other:?}"),
    }
}

#[test]
fn image_frame_head_plus_payload_equals_whole_record() {
    // The two-write sender path must produce byte-identical output to the
    // one-Vec path, or a bridge writing head-then-payload would desync.
    let payload: Vec<u8> = (0..200u8).collect();
    let frame = sample_image(&payload, None);
    let mut two_writes = frame.to_record_head().expect("encode head");
    two_writes.extend_from_slice(frame.payload);
    assert_eq!(two_writes, frame.to_record().expect("encode"));
}

#[test]
fn absent_mid_exposure_is_not_zero_ticks() {
    let payload = [0u8; 4];

    let absent = sample_image(&payload, None).to_record().expect("encode");
    let zero = sample_image(&payload, Some(0)).to_record().expect("encode");
    assert_ne!(absent, zero, "absent and zero must differ on the wire");

    let Record::ImageFrame(a) = parse_record(&absent).unwrap() else {
        panic!("expected ImageFrame")
    };
    let Record::ImageFrame(z) = parse_record(&zero).unwrap() else {
        panic!("expected ImageFrame")
    };
    assert_eq!(a.camera_mid_exposure, None);
    assert_eq!(z.camera_mid_exposure, Some(0));

    // The flag bit at header offset 37, not the value at 8..16, is what
    // carries the distinction.
    assert_eq!(absent[PREFIX_LEN + 37] & 0x01, 0);
    assert_eq!(zero[PREFIX_LEN + 37] & 0x01, 1);
    assert_eq!(&absent[PREFIX_LEN + 8..PREFIX_LEN + 16], &[0u8; 8]);
    assert_eq!(&zero[PREFIX_LEN + 8..PREFIX_LEN + 16], &[0u8; 8]);
}

#[test]
fn heartbeat_and_bye_round_trip() {
    assert_eq!(
        parse_record(&imagery::heartbeat_record()).unwrap(),
        Record::Heartbeat
    );
    assert_eq!(parse_record(&imagery::bye_record()).unwrap(), Record::Bye);
}

// ------------------------------------------------------- unknown and future

#[test]
fn unknown_kind_lands_in_other() {
    let bytes = encode_record(9_999, b"hdr", b"payload").expect("encode");
    match parse_record(&bytes).expect("parse") {
        Record::Other {
            kind,
            header,
            payload,
        } => {
            assert_eq!(kind, 9_999);
            assert_eq!(header, b"hdr");
            assert_eq!(payload, b"payload");
            // And it re-emits verbatim.
            assert_eq!(
                encode_record(kind, header, payload).expect("re-emit"),
                bytes
            );
        }
        other => panic!("expected Other, got {other:?}"),
    }
}

/// Rewrites a record's `header_len` to `new_len`, appending `extra` filler
/// bytes to the header so the record stays self-consistent. This is what a
/// future minor version's writer would emit.
fn grow_header(record: &[u8], extra: usize) -> Vec<u8> {
    let header_len = u16::from_le_bytes([record[6], record[7]]) as usize;
    let payload_len = u32::from_le_bytes([record[8], record[9], record[10], record[11]]) as usize;
    let mut out = Vec::new();
    out.extend_from_slice(&record[..6]);
    out.extend_from_slice(&u16::try_from(header_len + extra).unwrap().to_le_bytes());
    out.extend_from_slice(&u32::try_from(payload_len).unwrap().to_le_bytes());
    out.extend_from_slice(&record[PREFIX_LEN..PREFIX_LEN + header_len]);
    out.extend(std::iter::repeat_n(0xABu8, extra));
    out.extend_from_slice(&record[PREFIX_LEN + header_len..]);
    out
}

#[test]
fn header_longer_than_known_is_forward_compatible() {
    let hello = sample_hello();
    let grown = grow_header(&hello.to_record().expect("encode"), 8);
    match parse_record(&grown).expect("parse a v1.1-shaped Hello") {
        Record::Hello(back) => assert_eq!(hello, back),
        other => panic!("expected Hello, got {other:?}"),
    }

    let rays = sample_rays();
    let desc = sample_camera_desc(&rays);
    let grown = grow_header(&desc.to_record().expect("encode"), 4);
    match parse_record(&grown).expect("parse a grown CameraDesc") {
        Record::CameraDesc(back) => assert_eq!(desc, back),
        other => panic!("expected CameraDesc, got {other:?}"),
    }

    let payload = [7u8; 16];
    let frame = sample_image(&payload, Some(42));
    let grown = grow_header(&frame.to_record().expect("encode"), 2);
    match parse_record(&grown).expect("parse a grown ImageFrame") {
        Record::ImageFrame(back) => assert_eq!(frame, back),
        other => panic!("expected ImageFrame, got {other:?}"),
    }

    // Heartbeat/Bye know no header at all, so any header is skipped.
    assert_eq!(
        parse_record(&encode_record(imagery::KIND_HEARTBEAT, b"future", b"stuff").expect("encode"))
            .unwrap(),
        Record::Heartbeat
    );
}

#[test]
fn header_shorter_than_known_is_refused() {
    // Appending is safe; removing is not. A short header must be an Err.
    let hello = sample_hello();
    let record = hello.to_record().expect("encode");
    let mut short = record[..PREFIX_LEN + HELLO_HEADER_LEN - 4].to_vec();
    short.extend_from_slice(&record[PREFIX_LEN + HELLO_HEADER_LEN..]);
    short[6..8].copy_from_slice(&u16::try_from(HELLO_HEADER_LEN - 4).unwrap().to_le_bytes());
    let err = parse_record(&short).unwrap_err();
    assert!(
        matches!(err, natnet::Error::UnexpectedEof { .. }),
        "expected UnexpectedEof, got {err:?}"
    );
}

// -------------------------------------------------------------- robustness

#[test]
fn every_truncation_is_an_error_not_a_panic() {
    let rays = sample_rays();
    let payload: Vec<u8> = (0..97u8).collect();
    let records = [
        sample_hello().to_record().expect("encode"),
        sample_camera_desc(&rays).to_record().expect("encode"),
        sample_image(&payload, Some(1)).to_record().expect("encode"),
        imagery::heartbeat_record(),
        imagery::bye_record(),
        encode_record(4_242, b"h", b"p").expect("encode"),
    ];
    for record in &records {
        for n in 0..record.len() {
            // Every proper prefix must fail cleanly.
            let err = parse_record(&record[..n]).unwrap_err();
            assert!(
                matches!(err, natnet::Error::UnexpectedEof { .. }),
                "prefix of {n} bytes gave {err:?}"
            );
            // record_len must never panic either; below PREFIX_LEN it simply
            // cannot answer yet.
            match record_len(&record[..n]) {
                Ok(None) => assert!(n < PREFIX_LEN),
                Ok(Some(total)) => assert_eq!(total, record.len()),
                Err(e) => panic!("record_len on {n} bytes: {e:?}"),
            }
        }
        // The whole record parses.
        assert!(parse_record(record).is_ok());
    }
}

#[test]
fn trailing_bytes_after_a_record_are_ignored() {
    // A stream buffer may hold more than one record; parse_record takes the
    // first and record_len says where the next begins.
    let mut stream = imagery::heartbeat_record();
    let hello = sample_hello();
    stream.extend_from_slice(&hello.to_record().expect("encode"));

    let first = record_len(&stream).unwrap().unwrap();
    assert_eq!(parse_record(&stream).unwrap(), Record::Heartbeat);
    match parse_record(&stream[first..]).unwrap() {
        Record::Hello(back) => assert_eq!(hello, back),
        other => panic!("expected Hello, got {other:?}"),
    }
}

#[test]
fn bad_magic_is_reported_not_guessed() {
    let mut bytes = imagery::heartbeat_record();
    bytes[..4].copy_from_slice(b"XYZ!");
    let found = u32::from_le_bytes(bytes[..4].try_into().expect("magic bytes"));
    let err = record_len(&bytes).unwrap_err();
    assert_eq!(
        err,
        natnet::Error::InvalidMagic {
            found,
            expected: MAGIC,
        }
    );
    assert!(parse_record(&bytes).is_err());
}

/// Header offset of `ray_grid_w` in a [`CameraDesc`] record; `ray_grid_h`
/// follows it. (Prefix, then 68 bytes of fixed fields.)
const RAY_GRID_OFFSET: usize = PREFIX_LEN + 68;

#[test]
fn an_implausible_ray_grid_is_refused_before_allocating() {
    // A 65535x65535 grid claims 103 GB of rays with a 109-byte payload.
    //
    // The encoder now refuses to build this record at all, so the hostile
    // grid is patched into a well-formed one — which is the only way it can
    // arrive anyway: off a wire this crate did not write.
    let rays = sample_rays();
    let mut bytes = sample_camera_desc(&rays).to_record().expect("encode");
    assert_eq!(
        u16::from_le_bytes([bytes[RAY_GRID_OFFSET], bytes[RAY_GRID_OFFSET + 1]]),
        2,
        "RAY_GRID_OFFSET must point at ray_grid_w"
    );
    bytes[RAY_GRID_OFFSET..RAY_GRID_OFFSET + 4].copy_from_slice(&[0xFF; 4]);

    let err = parse_record(&bytes).unwrap_err();
    assert!(
        matches!(
            err,
            natnet::Error::UnexpectedEof { .. } | natnet::Error::ImplausibleCount { .. }
        ),
        "got {err:?}"
    );
}

#[test]
fn a_camera_name_with_no_terminator_is_an_error() {
    let rays = sample_rays();
    let desc = sample_camera_desc(&rays);
    let mut bytes = desc.to_record().expect("encode");
    // Drop the NUL terminator (the last payload byte) and fix payload_len.
    let payload_len = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    bytes.pop();
    bytes[8..12].copy_from_slice(&(payload_len - 1).to_le_bytes());
    let err = parse_record(&bytes).unwrap_err();
    assert!(
        matches!(err, natnet::Error::UnterminatedString { .. }),
        "got {err:?}"
    );
}

#[test]
fn a_camera_name_stops_at_the_first_invalid_utf8_byte() {
    let rays = sample_rays();
    let desc = sample_camera_desc(&rays);
    let mut bytes = desc.to_record().expect("encode");
    // Overwrite the tail of the name ("...#3\0") with an invalid sequence.
    let len = bytes.len();
    bytes[len - 3] = 0xFF;
    match parse_record(&bytes).expect("parse") {
        Record::CameraDesc(back) => assert_eq!(back.name, "Prime 13W "),
        other => panic!("expected CameraDesc, got {other:?}"),
    }
}

#[test]
fn encoding_tags_round_trip_through_the_wire_byte() {
    for (tag, wire) in [
        (ImageEncoding::Mono8, 1u8),
        (ImageEncoding::Jpeg, 2),
        (ImageEncoding::Other(ENCODING_NONE), 0),
        (ImageEncoding::Other(9), 9),
    ] {
        assert_eq!(tag.to_wire(), wire);
        assert_eq!(ImageEncoding::from_wire(wire), tag);
    }
    // from_wire never manufactures an aliasing Other.
    assert_ne!(ImageEncoding::from_wire(1), ImageEncoding::Other(1));
}

// ----------------------------------- encoding refuses what it cannot express

#[test]
fn a_ray_grid_that_does_not_match_the_rays_is_refused_at_encode() {
    // A mismatched grid must fail instead of consuming bytes from the name.
    let name = "0123456789abcdef0123456789abcdef";
    assert_eq!(name.len(), 32);

    let no_rays: [u8; 0] = [];
    let mut desc = sample_camera_desc(&no_rays);
    desc.ray_grid_w = 1;
    desc.ray_grid_h = 1;
    desc.name = name;
    let err = desc.to_record().unwrap_err();
    assert!(
        matches!(
            err,
            natnet::Error::SectionSizeMismatch {
                declared: 24,
                consumed: 0
            }
        ),
        "got {err:?}"
    );

    // Too many rays is refused just as squarely as too few: the 2x2 fixture's
    // 96 bytes declared as a 2x1 grid would leave 48 bytes to be read as a
    // name that is not there.
    let rays = sample_rays();
    let mut desc = sample_camera_desc(&rays);
    desc.ray_grid_h = 1;
    let err = desc.to_record().unwrap_err();
    assert!(
        matches!(
            err,
            natnet::Error::SectionSizeMismatch {
                declared: 48,
                consumed: 96
            }
        ),
        "got {err:?}"
    );

    // Positive control: the same 32-byte name with a grid that matches its
    // rays encodes, and comes back whole.
    let rays = [0u8; 24];
    let mut desc = sample_camera_desc(&rays);
    desc.ray_grid_w = 1;
    desc.ray_grid_h = 1;
    desc.name = name;
    match parse_record(&desc.to_record().expect("encode")).expect("parse") {
        Record::CameraDesc(back) => {
            assert_eq!(back, desc);
            assert_eq!(back.name, name);
            assert_eq!(back.rays.len(), 24);
        }
        other => panic!("expected CameraDesc, got {other:?}"),
    }
}

#[test]
fn a_camera_name_with_an_interior_nul_is_refused_at_encode() {
    // Same shape as the ray-grid mismatch: the name is NUL-terminated on the
    // wire, so an interior NUL encoded `Ok` and parsed back as a shorter name.
    let rays = sample_rays();
    let mut desc = sample_camera_desc(&rays);
    desc.name = "Prime\u{0}13W";
    let err = desc.to_record().unwrap_err();
    assert!(
        matches!(
            err,
            natnet::Error::InteriorNul {
                field: "CameraDesc::name",
                offset: 5
            }
        ),
        "got {err:?}"
    );
}

#[test]
fn a_hello_string_longer_than_its_length_prefix_is_refused_at_encode() {
    // The length prefix cannot truncate the string or split a UTF-8 sequence.
    let mut hello = sample_hello();
    hello.bridge_id = "x".repeat(70_000);
    let err = hello.to_record().unwrap_err();
    assert!(
        matches!(
            err,
            natnet::Error::LengthOverflow {
                field: "Hello::bridge_id",
                length: 70_000,
                max: 65_535
            }
        ),
        "got {err:?}"
    );

    // The longest string the prefix can express still encodes and round-trips.
    hello.bridge_id = "x".repeat(usize::from(u16::MAX));
    match parse_record(&hello.to_record().expect("encode")).expect("parse") {
        Record::Hello(back) => assert_eq!(back, hello),
        other => panic!("expected Hello, got {other:?}"),
    }
}

#[test]
fn a_header_too_long_for_its_length_field_is_refused_at_encode() {
    // The header length must match the bytes appended to the record.
    let header = vec![0xAAu8; 70_000];
    let err = encode_record(9_999, &header, b"tail").unwrap_err();
    assert!(
        matches!(
            err,
            natnet::Error::LengthOverflow {
                field: "header",
                length: 70_000,
                max: 65_535
            }
        ),
        "got {err:?}"
    );

    // The largest header the field *can* express still encodes, and what it
    // produces is self-describing — the exact property the narrowing broke:
    // record_len agrees with the bytes the record occupies.
    let header = vec![0xAAu8; usize::from(u16::MAX)];
    let record = encode_record(9_999, &header, b"tail").expect("65 535 fits");
    assert_eq!(record.len(), PREFIX_LEN + 65_535 + 4);
    assert_eq!(record_len(&record).unwrap(), Some(record.len()));
    match parse_record(&record).expect("parse") {
        Record::Other {
            kind,
            header: back,
            payload,
        } => {
            assert_eq!(kind, 9_999);
            assert_eq!(back.len(), 65_535);
            assert_eq!(payload, b"tail");
        }
        other => panic!("expected Other, got {other:?}"),
    }

    // `payload_len` is a u32 and goes through the same check; reaching it
    // needs a >4 GiB slice, so it is deliberately left unexercised here.
}

#[test]
fn short_borrowed_ray_slices_are_safe_at_the_ray_seam() {
    let rays = [0u8; 24];
    let mut desc = sample_camera_desc(&rays);
    desc.ray_grid_w = 2;
    desc.ray_grid_h = 2;

    assert_eq!(desc.ray(0, 0), Some([0.0; 6]));
    assert_eq!(desc.ray(1, 1), None);
}

#[test]
fn invalid_utf8_in_a_length_prefixed_string_is_replaced_lossily() {
    let mut bytes = sample_hello().to_record().expect("encode");
    let payload_start = PREFIX_LEN + HELLO_HEADER_LEN;
    bytes[payload_start + 2] = 0xFF;

    match parse_record(&bytes).expect("parse") {
        Record::Hello(back) => assert!(back.bridge_id.contains('\u{FFFD}')),
        other => panic!("expected Hello, got {other:?}"),
    }
}

#[test]
fn alias_encoding_tags_are_canonicalized_by_the_wire() {
    let rays = sample_rays();
    for (tag, canonical) in [
        (ImageEncoding::Other(1), ImageEncoding::Mono8),
        (ImageEncoding::Other(2), ImageEncoding::Jpeg),
    ] {
        let mut desc = sample_camera_desc(&rays);
        desc.encoding = tag;
        match parse_record(&desc.to_record().expect("encode camera alias"))
            .expect("parse camera alias")
        {
            Record::CameraDesc(back) => assert_eq!(back.encoding, canonical),
            other => panic!("expected CameraDesc, got {other:?}"),
        }

        let payload = [1u8, 2, 3];
        let mut image = sample_image(&payload, None);
        image.encoding = tag;
        match parse_record(&image.to_record().expect("encode image alias"))
            .expect("parse image alias")
        {
            Record::ImageFrame(back) => assert_eq!(back.encoding, canonical),
            other => panic!("expected ImageFrame, got {other:?}"),
        }
    }
}

#[test]
fn float_nan_serializer_contract_preserves_bits_but_not_value_equality() {
    let rays = sample_rays();
    let mut desc = sample_camera_desc(&rays);
    desc.position[0] = f32::from_bits(0x7FC0_1234);
    let desc_record = desc.to_record().expect("encode NaN camera");
    match parse_record(&desc_record).expect("parse NaN camera") {
        Record::CameraDesc(back) => {
            assert_eq!(back.position[0].to_bits(), desc.position[0].to_bits());
            assert_ne!(back, desc);
        }
        other => panic!("expected CameraDesc, got {other:?}"),
    }

    let payload = [9u8, 8, 7];
    let mut image = sample_image(&payload, None);
    image.api_timestamp_secs = f64::from_bits(0x7FF8_0000_0000_1234);
    let image_record = image.to_record().expect("encode NaN image");
    match parse_record(&image_record).expect("parse NaN image") {
        Record::ImageFrame(back) => {
            assert_eq!(
                back.api_timestamp_secs.to_bits(),
                image.api_timestamp_secs.to_bits()
            );
            assert_ne!(back, image);
        }
        other => panic!("expected ImageFrame, got {other:?}"),
    }
}

// --------------------------------------- the NatNet join (u32 against i32)

#[test]
fn an_image_frame_joins_the_natnet_frame_of_the_same_number() {
    // The join the whole side-channel rests on, end to end and off real wire
    // bytes on both sides: a NatNet datagram parsed into a FrameOfData, an
    // AUGI/1 record parsed into an ImageFrame, one pairing out.
    let server = SyntheticServer::new(Version::V4_1, 10_000_000);
    let datagram = server
        .datagram(15_732_032, 1.5, &[])
        .expect("valid frame encodes");
    let Message::FrameOfData(fod) = Decoder::new(Version::V4_1)
        .expect("supported")
        .parse(&datagram)
        .map(natnet::DecodedMessage::into_message)
        .expect("parse natnet")
    else {
        panic!("expected FrameOfData");
    };
    let fod: FrameOfData = *fod;
    assert_eq!(fod.frame_number, 15_732_032i32);

    let payload = [0u8; 8];
    let image = sample_image(&payload, Some(fod.timing.camera_mid_exposure));
    let record = image.to_record().expect("encode");
    let Record::ImageFrame(image) = parse_record(&record).expect("parse augi") else {
        panic!("expected ImageFrame");
    };

    // The fixture image carries the same Motive frame number as the NatNet
    // fixture, one as u32 and one as i32.
    assert!(image.matches_natnet_frame(fod.frame_number));
    assert_eq!(image.natnet_frame_number(), fod.frame_number);
    // ...and joining delivers the correlation key the join exists for.
    assert_eq!(
        image.camera_mid_exposure,
        Some(fod.timing.camera_mid_exposure)
    );

    // A neighbouring frame must not join.
    assert!(!image.matches_natnet_frame(fod.frame_number + 1));
}

#[test]
fn the_join_survives_the_wrap_that_try_from_would_drop() {
    // Motive's counter is 32-bit and wraps. Past 2^31 (about 207 days at
    // 120 Hz) the NatNet side reads negative while AUGI/1 reads large and
    // positive, on identical bits. `u32::try_from(fod.frame_number)` is the
    // safe-looking join that silently drops every one of those frames.
    let server = SyntheticServer::new(Version::V4_1, 10_000_000);
    for natnet_frame_number in [i32::MIN, i32::MIN + 7, -1i32] {
        let datagram = server
            .datagram(natnet_frame_number, 1.0, &[])
            .expect("valid frame encodes");
        let Message::FrameOfData(fod) = Decoder::new(Version::V4_1)
            .expect("supported")
            .parse(&datagram)
            .map(natnet::DecodedMessage::into_message)
            .expect("parse natnet")
        else {
            panic!("expected FrameOfData");
        };
        let fod: FrameOfData = *fod;
        assert_eq!(fod.frame_number, natnet_frame_number);

        let payload = [0u8; 4];
        let mut image = sample_image(&payload, None);
        image.frame_number = natnet_frame_number as u32;
        assert!(image.frame_number >= 1 << 31, "past the wrap");

        assert!(image.matches_natnet_frame(fod.frame_number));
        assert_eq!(image.natnet_frame_number(), fod.frame_number);

        // The trap: the conversion refuses the frame outright, so a consumer
        // written that way pairs nothing at all.
        assert!(u32::try_from(fod.frame_number).is_err());
        assert_ne!(
            u32::try_from(fod.frame_number).unwrap_or(0),
            image.frame_number
        );
    }
}

// ------------------------------------------------------------ golden bytes

/// One ImageFrame record, written out by hand from the format table in the
/// module docs. **Do not regenerate this from the encoder** — it exists to
/// catch the encoder moving.
const GOLDEN_IMAGE_FRAME: [u8; 56] = [
    // -- prefix ------------------------------------------------------------
    0x41, 0x55, 0x47, 0x49, // magic "AUGI"
    0x03, 0x00, // kind = 3 (ImageFrame)
    0x28, 0x00, // header_len = 40
    0x04, 0x00, 0x00, 0x00, // payload_len = 4
    // -- header ------------------------------------------------------------
    0x04, 0x03, 0x02, 0x01, // frame_number = 0x01020304
    0x07, 0x00, 0x00, 0x00, // camera_index = 7
    0x00, 0x41, 0xE3, 0x00, 0x3F, 0xB1, 0xCA, 0x07, // mid exposure ticks
    0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, // grab_ticks
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF8, 0x3F, // api_timestamp_secs = 1.5
    0x80, 0x02, // width  = 640
    0x00, 0x02, // height = 512
    0x01, // encoding = Mono8
    0x01, // flags = mid-exposure valid
    0x00, 0x00, // reserved
    // -- payload -----------------------------------------------------------
    0x01, 0x02, 0x03, 0x04,
];

#[test]
fn golden_image_frame_matches_the_encoder_byte_for_byte() {
    let payload = [1u8, 2, 3, 4];
    let frame = ImageFrame {
        frame_number: 0x0102_0304,
        camera_index: 7,
        camera_mid_exposure: Some(0x07CA_B13F_00E3_4100),
        grab_ticks: 0x0000_0001_0000_0002,
        api_timestamp_secs: 1.5,
        width: 640,
        height: 512,
        encoding: ImageEncoding::Mono8,
        payload: &payload,
    };
    assert_eq!(frame.to_record().expect("encode"), GOLDEN_IMAGE_FRAME);

    match parse_record(&GOLDEN_IMAGE_FRAME).expect("parse golden") {
        Record::ImageFrame(back) => assert_eq!(back, frame),
        other => panic!("expected ImageFrame, got {other:?}"),
    }
}

#[test]
fn declared_header_lengths_match_what_the_encoders_write() {
    let header_len = |record: &[u8]| usize::from(u16::from_le_bytes([record[6], record[7]]));
    let kind = |record: &[u8]| u16::from_le_bytes([record[4], record[5]]);

    let hello = sample_hello().to_record().expect("encode");
    assert_eq!(kind(&hello), KIND_HELLO);
    assert_eq!(header_len(&hello), HELLO_HEADER_LEN);

    let rays = sample_rays();
    let desc = sample_camera_desc(&rays).to_record().expect("encode");
    assert_eq!(kind(&desc), KIND_CAMERA_DESC);
    assert_eq!(header_len(&desc), CAMERA_DESC_HEADER_LEN);

    let payload = [0u8; 3];
    let frame = sample_image(&payload, None).to_record().expect("encode");
    assert_eq!(kind(&frame), KIND_IMAGE_FRAME);
    assert_eq!(header_len(&frame), IMAGE_FRAME_HEADER_LEN);

    assert_eq!(MAGIC.to_le_bytes(), *b"AUGI");
}
