use natnet::frame::{FrameTiming, MarkerSet, RawSection};
use natnet::{Error, FrameOfData, ParsePolicy, Version};

fn put_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn put_f64(out: &mut Vec<u8>, value: f64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn sized_section(out: &mut Vec<u8>, count: i32, body: &[u8]) {
    put_i32(out, count);
    let size = i32::try_from(body.len()).unwrap_or(i32::MAX);
    put_i32(out, size);
    out.extend_from_slice(body);
}

fn empty_legacy_payload() -> Vec<u8> {
    let mut out = Vec::with_capacity(78);
    put_i32(&mut out, 17);
    // marker sets, other markers, rigid bodies, skeletons, labelled markers,
    // force plates and devices: count-only sections in 3.x and 4.0.
    for _ in 0..7 {
        put_i32(&mut out, 0);
    }
    put_u32(&mut out, 0x0102_0304);
    put_u32(&mut out, 0x0506_0708);
    put_f64(&mut out, 17.5);
    put_u64(&mut out, 11);
    put_u64(&mut out, 22);
    put_u64(&mut out, 33);
    put_u16(&mut out, 0x0042);
    put_i32(&mut out, 0);
    out
}

fn empty_v41_payload() -> Vec<u8> {
    let mut out = Vec::with_capacity(122);
    put_i32(&mut out, 17);
    // marker sets, other markers, rigid bodies, skeletons, assets, labelled
    // markers, force plates and devices: count + byte size in 4.1.
    for _ in 0..8 {
        put_i32(&mut out, 0);
        put_i32(&mut out, 0);
    }
    put_u32(&mut out, 0x0102_0304);
    put_u32(&mut out, 0x0506_0708);
    put_f64(&mut out, 17.5);
    put_u64(&mut out, 11);
    put_u64(&mut out, 22);
    put_u64(&mut out, 33);
    put_u32(&mut out, 44);
    put_u32(&mut out, 55);
    put_u16(&mut out, 0x0042);
    put_i32(&mut out, 0);
    out
}

fn empty_frame() -> FrameOfData {
    FrameOfData {
        frame_number: 17,
        marker_sets: Vec::new(),
        other_markers: Vec::new(),
        rigid_bodies: Vec::new(),
        skeletons: RawSection {
            count: 0,
            payload: Vec::new(),
        },
        assets: RawSection {
            count: 0,
            payload: Vec::new(),
        },
        labeled_markers: Vec::new(),
        force_plates: RawSection {
            count: 0,
            payload: Vec::new(),
        },
        devices: RawSection {
            count: 0,
            payload: Vec::new(),
        },
        timing: FrameTiming {
            timecode: 0x0102_0304,
            timecode_subframe: 0x0506_0708,
            timestamp: 17.5,
            camera_mid_exposure: 11,
            camera_data_received: 22,
            transmit: 33,
            precision_timestamp_secs: 44,
            precision_timestamp_fractional_secs: 55,
        },
        params: 0x0042,
    }
}

#[test]
fn legacy_empty_fixture_is_count_only_and_78_bytes() {
    let payload = empty_legacy_payload();
    assert_eq!(payload.len(), 78);
    for version in [
        Version::new(3, 0, 9, 8),
        Version::new(3, 1, 9, 8),
        Version::new(4, 0, 9, 8),
    ] {
        let frame = FrameOfData::parse_payload_with_policy(&payload, version, ParsePolicy::Strict)
            .expect("literal legacy frame");
        assert_eq!(frame.frame_number, 17);
        assert_eq!(frame.timing.timestamp, 17.5);
        assert_eq!(frame.timing.precision_timestamp_secs, 0);
        assert_eq!(frame.to_payload(version).unwrap(), payload);
    }
}

#[test]
fn v41_empty_fixture_has_assets_sizes_and_precision() {
    let payload = empty_v41_payload();
    assert_eq!(payload.len(), 122);
    let frame = FrameOfData::parse_payload_with_policy(
        &payload,
        Version::new(4, 1, 9, 8),
        ParsePolicy::Strict,
    )
    .expect("literal 4.1 frame");
    assert_eq!(frame.assets.count, 0);
    assert_eq!(frame.timing.precision_timestamp_secs, 44);
    assert_eq!(frame.timing.precision_timestamp_fractional_secs, 55);
    assert_eq!(frame.to_payload(Version::V4_1).unwrap(), payload);
}

#[test]
fn v41_labeled_marker_guard_rejects_a_short_entry_before_truncation() {
    let mut payload = empty_v41_payload();
    // The labeled-marker section starts after frame number and five sized
    // sections: marker sets, other markers, rigid bodies, skeletons, assets.
    payload[44..48].copy_from_slice(&1i32.to_le_bytes());
    payload[48..52].copy_from_slice(&25i32.to_le_bytes());
    payload.splice(52..52, std::iter::repeat_n(0u8, 25));

    assert_eq!(
        FrameOfData::parse_payload_with_policy(&payload, Version::V4_1, ParsePolicy::Strict),
        Err(Error::ImplausibleCount {
            count: 1,
            available: 25,
        })
    );
}

#[test]
fn checked_encoding_matches_independent_empty_fixtures() {
    let frame = empty_frame();
    let mut legacy = frame.clone();
    legacy.timing.precision_timestamp_secs = 0;
    legacy.timing.precision_timestamp_fractional_secs = 0;
    assert_eq!(
        legacy.to_payload(Version::new(3, 0, 0, 0)).unwrap(),
        empty_legacy_payload()
    );
    assert_eq!(
        legacy.to_payload(Version::new(4, 0, 0, 0)).unwrap(),
        empty_legacy_payload()
    );
    assert_eq!(
        frame.to_payload(Version::V4_1).unwrap(),
        empty_v41_payload()
    );
}

#[test]
fn legacy_nonempty_nested_sections_walk_to_the_suffix() {
    let mut payload = Vec::new();
    put_i32(&mut payload, 9);

    // One marker set with one marker.
    put_i32(&mut payload, 1);
    payload.extend_from_slice(b"set\0");
    put_i32(&mut payload, 1);
    put_f32(&mut payload, 1.0);
    put_f32(&mut payload, 2.0);
    put_f32(&mut payload, 3.0);

    // One other marker.
    put_i32(&mut payload, 1);
    put_f32(&mut payload, 4.0);
    put_f32(&mut payload, 5.0);
    put_f32(&mut payload, 6.0);

    // One rigid body.
    put_i32(&mut payload, 1);
    put_i32(&mut payload, 42);
    for value in [7.0, 8.0, 9.0] {
        put_f32(&mut payload, value);
    }
    for value in [0.0, 0.0, 0.0, 1.0] {
        put_f32(&mut payload, value);
    }
    put_f32(&mut payload, 0.01);
    put_u16(&mut payload, 1);

    // One skeleton with one rigid body.
    put_i32(&mut payload, 1);
    put_i32(&mut payload, 99);
    put_i32(&mut payload, 1);
    put_i32(&mut payload, 100);
    for _ in 0..3 {
        put_f32(&mut payload, 0.0);
    }
    for value in [0.0, 0.0, 0.0, 1.0] {
        put_f32(&mut payload, value);
    }
    put_f32(&mut payload, 0.0);
    put_u16(&mut payload, 1);

    // One labelled marker.
    put_i32(&mut payload, 1);
    put_i32(&mut payload, 7);
    for value in [1.0, 2.0, 3.0] {
        put_f32(&mut payload, value);
    }
    put_f32(&mut payload, 0.25);
    put_u16(&mut payload, 0x20);
    put_f32(&mut payload, 0.5);

    // One force plate and one device, each with one channel and two samples.
    for id in [5, 6] {
        put_i32(&mut payload, 1);
        put_i32(&mut payload, id);
        put_i32(&mut payload, 1);
        put_i32(&mut payload, 2);
        put_f32(&mut payload, 10.0);
        put_f32(&mut payload, 20.0);
    }

    put_u32(&mut payload, 0);
    put_u32(&mut payload, 0);
    put_f64(&mut payload, 1.25);
    put_u64(&mut payload, 1);
    put_u64(&mut payload, 2);
    put_u64(&mut payload, 3);
    put_u16(&mut payload, 0);
    put_i32(&mut payload, 0);

    let frame = FrameOfData::parse_payload_with_policy(
        &payload,
        Version::new(3, 1, 0, 0),
        ParsePolicy::Strict,
    )
    .expect("literal legacy nonempty frame");
    assert_eq!(frame.marker_sets[0].name, "set");
    assert_eq!(frame.skeletons.count, 1);
    assert_eq!(frame.skeletons.payload.len(), 46);
    assert_eq!(frame.force_plates.payload.len(), 20);
    assert_eq!(frame.devices.payload.len(), 20);
    assert_eq!(frame.to_payload(Version::new(3, 1, 0, 0)).unwrap(), payload);
}

#[test]
fn v41_nonempty_sections_use_independent_sizes_and_nested_extents() {
    let mut payload = Vec::new();
    put_i32(&mut payload, 101);

    let mut body = Vec::new();
    body.extend_from_slice(b"v41\0");
    put_i32(&mut body, 1);
    for value in [1.0, 2.0, 3.0] {
        put_f32(&mut body, value);
    }
    sized_section(&mut payload, 1, &body);

    let mut body = Vec::new();
    for value in [4.0, 5.0, 6.0] {
        put_f32(&mut body, value);
    }
    sized_section(&mut payload, 1, &body);

    let mut body = Vec::new();
    put_i32(&mut body, 42);
    for value in [7.0, 8.0, 9.0] {
        put_f32(&mut body, value);
    }
    for value in [0.0, 0.0, 0.0, 1.0] {
        put_f32(&mut body, value);
    }
    put_f32(&mut body, 0.01);
    put_u16(&mut body, 1);
    sized_section(&mut payload, 1, &body);

    let mut body = Vec::new();
    put_i32(&mut body, 9);
    put_i32(&mut body, 1);
    put_i32(&mut body, 10);
    for _ in 0..3 {
        put_f32(&mut body, 0.0);
    }
    for value in [0.0, 0.0, 0.0, 1.0] {
        put_f32(&mut body, value);
    }
    put_f32(&mut body, 0.0);
    put_u16(&mut body, 1);
    sized_section(&mut payload, 1, &body);

    let mut body = Vec::new();
    put_i32(&mut body, 7); // asset id
    put_i32(&mut body, 1); // rigid body count
    put_i32(&mut body, 11);
    for _ in 0..3 {
        put_f32(&mut body, 0.0);
    }
    for value in [0.0, 0.0, 0.0, 1.0] {
        put_f32(&mut body, value);
    }
    put_f32(&mut body, 0.0);
    put_u16(&mut body, 1);
    put_i32(&mut body, 1); // marker count
    put_i32(&mut body, 12);
    for value in [1.0, 2.0, 3.0] {
        put_f32(&mut body, value);
    }
    put_f32(&mut body, 0.2);
    put_u16(&mut body, 0x20);
    put_f32(&mut body, 0.3);
    sized_section(&mut payload, 1, &body);

    let mut body = Vec::new();
    put_i32(&mut body, 12);
    for value in [1.0, 2.0, 3.0] {
        put_f32(&mut body, value);
    }
    put_f32(&mut body, 0.2);
    put_u16(&mut body, 0x20);
    put_f32(&mut body, 0.3);
    sized_section(&mut payload, 1, &body);

    for (id, samples) in [(5, &[10.0, 20.0][..]), (6, &[30.0][..])] {
        let mut body = Vec::new();
        put_i32(&mut body, id);
        put_i32(&mut body, 1);
        put_i32(&mut body, samples.len() as i32);
        for sample in samples {
            put_f32(&mut body, *sample);
        }
        sized_section(&mut payload, 1, &body);
    }

    put_u32(&mut payload, 0);
    put_u32(&mut payload, 0);
    put_f64(&mut payload, 2.5);
    put_u64(&mut payload, 4);
    put_u64(&mut payload, 5);
    put_u64(&mut payload, 6);
    put_u32(&mut payload, 7);
    put_u32(&mut payload, 8);
    put_u16(&mut payload, 0);
    put_i32(&mut payload, 0);

    let frame = FrameOfData::parse_payload_with_policy(
        &payload,
        Version::new(4, 1, 8, 7),
        ParsePolicy::Strict,
    )
    .expect("literal 4.1 nonempty frame");
    assert_eq!(frame.assets.count, 1);
    assert_eq!(frame.assets.payload.len(), 76);
    assert_eq!(frame.force_plates.payload.len(), 20);
    assert_eq!(frame.devices.payload.len(), 16);
    assert_eq!(frame.to_payload(Version::new(4, 1, 8, 7)).unwrap(), payload);
}

#[test]
fn compatibility_preserves_opaque_v41_raw_body_but_strict_checks_it() {
    let mut payload = empty_v41_payload();
    // Skeleton count/size are the fourth 4.1 section, after frame number and
    // three typed sections: count at 28, size at 32.
    payload[28..32].copy_from_slice(&0i32.to_le_bytes());
    payload[32..36].copy_from_slice(&1i32.to_le_bytes());
    payload.insert(36, 0xa5);

    let compatible =
        FrameOfData::parse_payload_with_policy(&payload, Version::V4_1, ParsePolicy::Compatible)
            .expect("compatibility preserves opaque body");
    assert_eq!(compatible.skeletons.payload, vec![0xa5]);
    assert!(matches!(
        FrameOfData::parse_payload_with_policy(&payload, Version::V4_1, ParsePolicy::Strict),
        Err(Error::SectionSizeMismatch { .. })
    ));
}

#[test]
fn checked_encoder_rejects_values_missing_from_legacy_layout() {
    let mut frame = empty_frame();
    frame.assets = RawSection {
        count: 1,
        payload: vec![],
    };
    assert!(matches!(
        frame.to_payload(Version::new(4, 0, 0, 0)),
        Err(Error::InvalidValue {
            field: "assets",
            ..
        })
    ));

    frame.assets = RawSection {
        count: 0,
        payload: Vec::new(),
    };
    frame.timing.precision_timestamp_secs = 1;
    assert!(matches!(
        frame.to_payload(Version::new(3, 0, 0, 0)),
        Err(Error::InvalidValue {
            field: "precision timestamp",
            ..
        })
    ));

    frame.timing.precision_timestamp_secs = 0;
    frame.marker_sets = vec![MarkerSet {
        name: "bad\0name".to_owned(),
        markers: Vec::new(),
    }];
    assert!(matches!(
        frame.to_payload(Version::V4_1),
        Err(Error::InteriorNul { .. })
    ));
}

#[test]
fn lossy_encoder_rejects_unknown_versions_without_guessing_a_layout() {
    let frame = empty_frame();
    let unknown = Version::new(4, 2, 0, 0);
    assert!(matches!(
        frame.to_payload_lossy(unknown),
        Err(Error::UnsupportedVersion { major: 4, minor: 2 })
    ));
    assert!(matches!(
        frame.to_datagram_lossy(unknown),
        Err(Error::UnsupportedVersion { major: 4, minor: 2 })
    ));
}

#[test]
fn strict_rejects_negative_counts_nonzero_terminator_and_trailing_bytes() {
    let mut negative = empty_v41_payload();
    negative[4..8].copy_from_slice(&(-1i32).to_le_bytes());
    assert!(matches!(
        FrameOfData::parse_payload_with_policy(&negative, Version::V4_1, ParsePolicy::Strict),
        Err(Error::InvalidCount { .. })
    ));

    let mut terminator = empty_v41_payload();
    let at = terminator.len() - 4;
    terminator[at..].copy_from_slice(&7i32.to_le_bytes());
    assert!(matches!(
        FrameOfData::parse_payload_with_policy(&terminator, Version::V4_1, ParsePolicy::Strict),
        Err(Error::InvalidTerminator { value: 7, .. })
    ));

    let mut trailing = empty_v41_payload();
    trailing.push(0xff);
    assert!(matches!(
        FrameOfData::parse_payload_with_policy(&trailing, Version::V4_1, ParsePolicy::Strict),
        Err(Error::TrailingBytes { remaining: 1 })
    ));
    assert!(
        FrameOfData::parse_payload_with_policy(&trailing, Version::V4_1, ParsePolicy::Compatible)
            .is_ok()
    );
}

#[test]
fn unsupported_versions_are_rejected_before_frame_decode() {
    let payload = empty_v41_payload();
    for version in [
        Version::new(2, 9, 0, 0),
        Version::new(3, 2, 0, 0),
        Version::new(4, 2, 0, 0),
        Version::new(99, 0, 0, 0),
    ] {
        assert!(matches!(
            FrameOfData::parse_payload(&payload, version),
            Err(Error::UnsupportedVersion { .. })
        ));
    }
}

#[test]
fn frame_datagram_checked_seam_stamps_message_header() {
    let frame = empty_frame();
    let datagram = frame.to_datagram(Version::V4_1).unwrap();
    assert_eq!(&datagram[..4], &[7, 0, 122, 0]);
    assert_eq!(FrameOfData::parse(&datagram, Version::V4_1).unwrap(), frame);
}

#[test]
fn negative_frame_counts_and_sizes_are_invalid_under_both_policies() {
    let legacy = Version::new(4, 0, 0, 0);
    let v41 = Version::V4_1;
    // (payload, version, expected count, expected offset)
    let mut cases = Vec::new();

    // Top-level typed count (marker sets) in both layouts.
    let mut legacy_count = empty_legacy_payload();
    legacy_count[4..8].copy_from_slice(&(-1i32).to_le_bytes());
    cases.push((legacy_count, legacy, -1, 4));
    let mut v41_count = empty_v41_payload();
    v41_count[4..8].copy_from_slice(&(-2i32).to_le_bytes());
    cases.push((v41_count, v41, -2, 4));

    // 4.1 byte size of a typed section and of an opaque raw section
    // (skeletons: count at 28, size at 32).
    let mut typed_size = empty_v41_payload();
    typed_size[8..12].copy_from_slice(&(-3i32).to_le_bytes());
    cases.push((typed_size, v41, -3, 8));
    let mut raw_size = empty_v41_payload();
    raw_size[32..36].copy_from_slice(&i32::MIN.to_le_bytes());
    cases.push((raw_size, v41, i32::MIN, 32));

    // Nested typed count: one marker set "a" whose marker count is -4.
    let mut nested = Vec::new();
    put_i32(&mut nested, 17);
    put_i32(&mut nested, 1);
    nested.extend_from_slice(b"a\0");
    put_i32(&mut nested, -4);
    nested.extend_from_slice(&empty_legacy_payload()[8..]);
    cases.push((nested, legacy, -4, 10));

    for (payload, version, count, offset) in cases {
        for policy in [ParsePolicy::Strict, ParsePolicy::Compatible] {
            assert_eq!(
                FrameOfData::parse_payload_with_policy(&payload, version, policy),
                Err(Error::InvalidCount { count, offset }),
                "{policy:?} count {count} at {offset}"
            );
        }
    }
}

#[test]
fn checked_frame_datagram_refuses_a_payload_its_header_cannot_declare() {
    let mut frame = empty_frame();
    frame.other_markers = vec![[1.0, 2.0, 3.0]; 5_500];
    let payload = frame.to_payload(Version::V4_1).unwrap();
    assert!(payload.len() > usize::from(u16::MAX));
    assert_eq!(
        frame.to_datagram(Version::V4_1).map(|d| d.len()),
        Err(Error::LengthOverflow {
            field: "message payload size",
            length: payload.len(),
            max: 65_535,
        })
    );

    // Only the lossy encoder keeps the saturated header.
    let lossy = frame.to_datagram_lossy(Version::V4_1).unwrap();
    assert_eq!(&lossy[..4], &[7, 0, 0xff, 0xff]);
    assert_eq!(&lossy[4..], payload.as_slice());

    // The largest declarable payload is still accepted.
    frame.other_markers.clear();
    let base = frame.to_payload(Version::V4_1).unwrap().len();
    let fill = (usize::from(u16::MAX) - base) / 12;
    frame.other_markers = vec![[0.0; 3]; fill];
    let fits = frame.to_datagram(Version::V4_1).unwrap();
    assert_eq!(fits.len() - 4, base + fill * 12);
}

#[test]
fn frame_params_expose_the_bitstream_changed_bit() {
    // NatNetTypes.h 4.1.0.0: b3 (0x08) is "bitstream version changed"; b2
    // (0x04) is Live/Edit mode and must not read as a bitstream change.
    let mut frame = empty_frame();
    for (params, changed) in [
        (0x0000, false),
        (0x0007, false),
        (0x0004, false),
        (0x0008, true),
        (0xffff, true),
    ] {
        frame.params = params;
        assert_eq!(frame.bitstream_changed(), changed, "params {params:#06x}");
    }
}
