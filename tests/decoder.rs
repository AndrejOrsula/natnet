use natnet::{Decoder, Error, Message, ParsePolicy, Version};

fn datagram(id: u16, payload: &[u8]) -> Vec<u8> {
    let mut bytes = id.to_le_bytes().to_vec();
    bytes.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
}

#[test]
fn supported_layouts_keep_build_and_revision_and_refuse_unlisted_pairs() {
    for (major, minor) in [(3, 0), (3, 1), (4, 0), (4, 1)] {
        let version = Version::new(major, minor, 254, 255);
        let decoder = Decoder::new(version).unwrap();
        assert_eq!(decoder.version(), version);
        assert_eq!(decoder.policy(), ParsePolicy::Strict);
    }
    for (major, minor) in [(0, 0), (2, 11), (3, 2), (3, 255), (4, 2), (4, 3), (99, 0)] {
        assert_eq!(
            Decoder::new(Version::new(major, minor, 0, 0)),
            Err(Error::UnsupportedVersion { major, minor })
        );
    }
}

#[test]
fn version_at_least_orders_major_and_minor_but_ignores_build_and_revision() {
    assert!(Version::new(4, 1, 0, 0).at_least(4, 0));
    assert!(Version::new(4, 1, 0, 0).at_least(4, 1));
    assert!(!Version::new(4, 1, 0, 0).at_least(4, 2));
    assert!(!Version::new(4, 1, 0, 0).at_least(5, 0));
    assert!(Version::new(5, 0, 0, 0).at_least(4, 255));
    assert!(!Version::new(3, 255, 255, 255).at_least(4, 0));

    assert!(Version::new(4, 1, 255, 255).at_least(4, 1));
    assert!(!Version::new(4, 0, 255, 255).at_least(4, 1));
}

#[test]
fn legacy_modeldef_retains_layout_and_explicit_typed_conversion() {
    let payload = [1, 0, 0, 0, 0, 0, 0, 0, b'x', 0, 0, 0, 0, 0];
    let wire = datagram(5, &payload);
    let source = Version::new(3, 0, 7, 8);
    let parsed = Decoder::new(source).unwrap().parse(&wire).unwrap();
    assert_eq!(parsed.version(), source);
    assert_eq!(parsed.to_datagram().unwrap(), wire);
    let converted = parsed.to_datagram_for(Version::V4_1).unwrap();
    assert_eq!(converted.len(), wire.len() + 4);
    assert_eq!(
        Decoder::new(Version::V4_1)
            .unwrap()
            .parse(&converted)
            .unwrap()
            .message(),
        parsed.message()
    );
}

#[test]
fn strict_rejects_residue_compatible_accepts_it() {
    let wire = datagram(5, &[0, 0, 0, 0, 42]);
    let decoder = Decoder::new(Version::V4_1).unwrap();
    assert_eq!(
        decoder.parse(&wire),
        Err(Error::TrailingBytes { remaining: 1 })
    );
    assert!(
        decoder
            .with_policy(ParsePolicy::Compatible)
            .parse(&wire)
            .is_ok()
    );
    assert!(natnet::parse_message(&wire, Version::V4_1).is_ok());
}

#[test]
fn opaque_modeldef_refuses_cross_layout_but_preserves_same_layout() {
    let wire = datagram(5, &[1, 0, 0, 0, 99, 0, 0, 0, 3, 0, 0, 0, 7, 8, 9]);
    let parsed = Decoder::new(Version::V4_1).unwrap().parse(&wire).unwrap();
    assert_eq!(parsed.to_datagram().unwrap(), wire);
    assert_eq!(
        parsed.to_datagram_for(Version::new(4, 1, 8, 9)).unwrap(),
        wire
    );
    assert!(matches!(
        parsed.to_datagram_for(Version::new(4, 0, 0, 0)),
        Err(Error::InvalidValue { .. })
    ));
}

#[test]
fn unknown_id_payload_and_telemetry_size_policy_are_preserved() {
    let mut wire = datagram(999, &[1, 2, 3, 4]);
    wire[2..4].copy_from_slice(&0u16.to_le_bytes());
    let parsed = Decoder::new(Version::V4_1).unwrap().parse(&wire).unwrap();
    assert!(
        matches!(parsed.message(), Message::Other { id: 999, payload } if payload == &[1, 2, 3, 4])
    );
    assert_eq!(&parsed.to_datagram().unwrap()[4..], &wire[4..]);
}

#[test]
fn generator_extreme_ticks_have_explicit_saturation_and_checked_version() {
    let server = natnet::SyntheticServer::new(Version::V4_1, 1_000);
    for (time, ticks) in [
        (f64::NAN, 0),
        (f64::NEG_INFINITY, 0),
        (-1.0, 0),
        (f64::INFINITY, u64::MAX),
        (f64::MAX, u64::MAX),
    ] {
        let frame = server.frame(1, time, &[]);
        assert_eq!(frame.timing.camera_mid_exposure, ticks);
        let parsed = Decoder::new(Version::V4_1)
            .unwrap()
            .parse(&server.datagram(1, time, &[]).unwrap())
            .unwrap();
        let Message::FrameOfData(decoded) = parsed.message() else {
            panic!("frame expected")
        };
        assert_eq!(decoded.timing.timestamp.to_bits(), time.to_bits());
        assert_eq!(decoded.timing.transmit, ticks);
    }
    assert!(matches!(
        natnet::SyntheticServer::new(Version::new(4, 2, 0, 0), 1).datagram(0, 0.0, &[]),
        Err(Error::UnsupportedVersion { .. })
    ));
}

#[test]
fn opaque_frame_conversion_requires_explicit_reconstruction() {
    let mut frame = natnet::SyntheticServer::new(Version::V4_1, 1_000).frame(1, 0.0, &[]);
    // Independent raw skeleton: id 17, zero nested rigid bodies.
    frame.skeletons = natnet::RawSection {
        count: 1,
        payload: vec![17, 0, 0, 0, 0, 0, 0, 0],
    };
    let wire = frame.to_datagram(Version::V4_1).unwrap();
    let parsed = Decoder::new(Version::V4_1).unwrap().parse(&wire).unwrap();
    assert_eq!(parsed.to_datagram().unwrap(), wire);
    assert!(matches!(
        parsed.to_datagram_for(Version::new(4, 0, 0, 0)),
        Err(Error::InvalidValue {
            field: "opaque frame sections",
            ..
        })
    ));
    assert_eq!(
        parsed.to_datagram_for(Version::new(4, 1, 9, 8)).unwrap(),
        wire
    );
    // Typed-only frames can convert when unavailable fields are default.
    frame.skeletons = natnet::RawSection {
        count: 0,
        payload: vec![],
    };
    let parsed = Decoder::new(Version::V4_1)
        .unwrap()
        .parse(&frame.to_datagram(Version::V4_1).unwrap())
        .unwrap();
    let legacy = parsed.to_datagram_for(Version::new(3, 1, 0, 0)).unwrap();
    assert_eq!(legacy.len(), 4 + 78);
    assert_eq!(
        Decoder::new(Version::new(3, 1, 0, 0))
            .unwrap()
            .parse(&legacy)
            .unwrap()
            .message(),
        parsed.message()
    );
}

#[test]
fn checked_reencode_of_an_oversized_unknown_message_is_refused() {
    let payload = vec![0x5a; 70_000];
    let mut wire = 42u16.to_le_bytes().to_vec();
    wire.extend_from_slice(&u16::MAX.to_le_bytes());
    wire.extend_from_slice(&payload);
    let parsed = Decoder::new(Version::V4_1).unwrap().parse(&wire).unwrap();
    assert_eq!(
        parsed.to_datagram().map(|d| d.len()),
        Err(Error::LengthOverflow {
            field: "message payload size",
            length: 70_000,
            max: 65_535,
        })
    );
}
