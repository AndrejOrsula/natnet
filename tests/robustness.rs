//! Malformed / truncated inputs must return `Err`, never panic, overflow or
//! hang (done-when: fuzz-style over every prefix of a real-shaped packet, and
//! over random bytes).

#![allow(
    clippy::expect_used,
    reason = "the robustness oracle expects worker joins after explicitly bounding their lifetime"
)]

use std::panic::catch_unwind;

use natnet::frame::{FrameTiming, MarkerSet, RawSection, RigidBody};
use natnet::{FrameOfData, ModelDef, ServerInfo, Version};

fn well_formed_frame() -> Vec<u8> {
    let f = FrameOfData {
        frame_number: 100,
        marker_sets: vec![MarkerSet {
            name: "wand".to_string(),
            markers: vec![[1.0, 2.0, 3.0]],
        }],
        other_markers: vec![],
        rigid_bodies: vec![RigidBody {
            id: 3,
            position: [1.0, 2.0, 3.0],
            orientation: [0.0, 0.0, 0.0, 1.0],
            mean_marker_error: 0.0,
            params: 0,
        }],
        skeletons: RawSection {
            count: 0,
            payload: vec![],
        },
        assets: RawSection {
            count: 0,
            payload: vec![],
        },
        labeled_markers: vec![],
        force_plates: RawSection {
            count: 0,
            payload: vec![],
        },
        devices: RawSection {
            count: 0,
            payload: vec![],
        },
        timing: FrameTiming {
            timecode: 0,
            timecode_subframe: 0,
            timestamp: 1.0,
            camera_mid_exposure: 1,
            camera_data_received: 2,
            transmit: 3,
            precision_timestamp_secs: 0,
            precision_timestamp_fractional_secs: 0,
        },
        params: 0,
    };
    f.to_datagram(Version::V4_1).expect("valid fixture encodes")
}

/// Every prefix of a well-formed datagram parses to `Ok` or `Err`, never a
/// panic. (`catch_unwind` turns a hypothetical panic into a test failure.)
#[test]
fn every_prefix_is_err_or_ok_never_panic() {
    let full = well_formed_frame();
    for len in 0..=full.len() {
        let slice = full[..len].to_vec();
        let r = catch_unwind(|| {
            let _ = FrameOfData::parse(&slice, Version::V4_1);
            let _ = ServerInfo::parse(&slice);
            let _ = ModelDef::parse(&slice);
            let _ = natnet::parse_message(&slice, Version::V4_1);
        });
        assert!(r.is_ok(), "panicked on prefix length {len}");
    }
    // The full datagram is a valid frame.
    assert!(FrameOfData::parse(&full, Version::V4_1).is_ok());
}

/// Header-level random smoke test. Typed payload mutation is covered separately;
/// random message IDs alone do not exercise known message payloads.
#[test]
fn random_bytes_never_panic() {
    let mut state: u64 = 0x1234_5678_9abc_def0;
    let mut next = || {
        // xorshift64
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for trial in 0..2000 {
        let len = (next() % 600) as usize;
        let mut buf = vec![0u8; len];
        for b in &mut buf {
            *b = (next() & 0xff) as u8;
        }
        let r = catch_unwind(|| {
            let _ = FrameOfData::parse(&buf, Version::V4_1);
            let _ = ServerInfo::parse(&buf);
            let _ = ModelDef::parse(&buf);
            let _ = natnet::parse_message(&buf, Version::V4_1);
        });
        assert!(r.is_ok(), "panicked on random trial {trial} (len {len})");
    }
}

/// A hostile length field (huge count, tiny buffer) is rejected as an error,
/// not an allocation or overflow.
#[test]
fn hostile_count_is_rejected() {
    // id 7 header + frame_number + a marker-set section claiming i32::MAX
    // entries in a 0-byte section.
    let mut d = Vec::new();
    d.extend_from_slice(&7u16.to_le_bytes());
    d.extend_from_slice(&0u16.to_le_bytes());
    d.extend_from_slice(&1i32.to_le_bytes()); // frame_number
    d.extend_from_slice(&i32::MAX.to_le_bytes()); // marker-set count
    d.extend_from_slice(&0i32.to_le_bytes()); // section size 0
    let err = FrameOfData::parse(&d, Version::V4_1).unwrap_err();
    assert!(matches!(
        err,
        natnet::Error::ImplausibleCount { .. } | natnet::Error::SectionSizeMismatch { .. }
    ));
}

/// The V4_1 payload of the all-sections-empty frame, as a value
/// `parse_payload` produced — carrying its own reach control, so a failure in
/// a test below that patches it tracks the patched header field and not the
/// fixture.
fn degenerate_payload() -> Vec<u8> {
    let mut f =
        FrameOfData::parse(&well_formed_frame(), Version::V4_1).expect("the fixture parses");
    f.marker_sets.clear();
    f.rigid_bodies.clear();
    let payload = f.to_payload(Version::V4_1).expect("valid fixture encodes");
    // `frame_number` is the only field ahead of the first section, so the
    // marker-set header sits at a fixed offset: count `[4..8]`, size `[8..12]`.
    assert_eq!(payload.len(), 122, "the degenerate V4_1 payload");
    assert_eq!(
        FrameOfData::parse_payload(&payload, Version::V4_1).expect("the control parses"),
        f,
        "reach control: unpatched, it parses back equal"
    );
    payload
}

/// A sized typed section must consume its complete declared region.
#[test]
fn a_typed_section_declaring_more_than_its_entries_consume_is_refused() {
    // The marker-set header declares eight bytes of entries; the zero entries
    // it also declares consume none.
    let mut d = degenerate_payload();
    d[8..12].copy_from_slice(&8i32.to_le_bytes());
    match FrameOfData::parse_payload(&d, Version::V4_1) {
        Err(natnet::Error::SectionSizeMismatch { declared, consumed }) => {
            assert_eq!(
                (declared, consumed),
                (8, 0),
                "both fields, not just the variant"
            );
        }
        other => panic!("8 declared against 0 consumed must be refused, got {other:?}"),
    }
}

/// A zero-byte region cannot borrow entry bytes from the following section.
#[test]
fn a_typed_section_whose_entries_overrun_the_declared_size_is_refused() {
    // One entry declared in a zero-byte section. The bounded reader rejects
    // the count before any bytes from the following section can be consumed.
    let mut d = degenerate_payload();
    d[4..8].copy_from_slice(&1i32.to_le_bytes());
    match FrameOfData::parse_payload(&d, Version::V4_1) {
        Err(natnet::Error::ImplausibleCount { count, available }) => {
            assert_eq!((count, available), (1, 0));
        }
        other => panic!("bounded section must refuse the entry before overread, got {other:?}"),
    }
}

/// A nonempty section rejects residue after its declared entries.
#[test]
fn a_typed_section_with_residue_between_two_non_zero_sizes_is_refused() {
    let f = FrameOfData::parse(&well_formed_frame(), Version::V4_1).expect("the fixture parses");
    let mut d = f.to_payload(Version::V4_1).expect("valid fixture encodes");
    assert_eq!(
        FrameOfData::parse_payload(&d, Version::V4_1).expect("the control parses"),
        f,
        "reach control: unpatched, it parses back equal"
    );
    // The one marker set's entries are `"wand\0"`, a four-byte marker count and
    // one twelve-byte marker: 21 bytes, declared by the header at `[8..12]`.
    assert_eq!(
        i32::from_le_bytes(d[8..12].try_into().expect("four bytes")),
        21,
        "the declared marker-set size the patch below moves"
    );
    d[8..12].copy_from_slice(&25i32.to_le_bytes());
    match FrameOfData::parse_payload(&d, Version::V4_1) {
        Err(natnet::Error::SectionSizeMismatch { declared, consumed }) => {
            assert_eq!(
                (declared, consumed),
                (25, 21),
                "both fields, not just the variant"
            );
        }
        other => panic!("25 declared against 21 consumed must be refused, got {other:?}"),
    }
}

/// The nested marker count is checked against its bounded section bytes.
#[test]
fn a_typed_section_whose_entries_overrun_a_non_zero_declared_size_is_refused() {
    let f = FrameOfData::parse(&well_formed_frame(), Version::V4_1).expect("the fixture parses");
    let mut d = f.to_payload(Version::V4_1).expect("valid fixture encodes");
    assert_eq!(
        FrameOfData::parse_payload(&d, Version::V4_1).expect("the control parses"),
        f,
        "reach control: unpatched, it parses back equal"
    );
    // The one marker set's entries are `"wand\0"`, a four-byte marker count and
    // one twelve-byte marker: 21 bytes, declared by the header at `[8..12]`.
    // Lowering that declaration to 17 leaves both sides non-zero, and the
    // entries then overrun it by four bytes.
    assert_eq!(
        i32::from_le_bytes(d[8..12].try_into().expect("four bytes")),
        21,
        "the declared marker-set size the patch below moves"
    );
    d[8..12].copy_from_slice(&17i32.to_le_bytes());
    match FrameOfData::parse_payload(&d, Version::V4_1) {
        Err(natnet::Error::ImplausibleCount { count, available }) => {
            assert_eq!((count, available), (1, 8));
        }
        other => panic!("bounded section must refuse the entry before overread, got {other:?}"),
    }
}

/// Even one byte of unexplained typed-section residue is rejected.
#[test]
fn a_typed_section_with_residue_of_one_between_two_non_zero_sizes_is_refused() {
    let f = FrameOfData::parse(&well_formed_frame(), Version::V4_1).expect("the fixture parses");
    let mut d = f.to_payload(Version::V4_1).expect("valid fixture encodes");
    assert_eq!(
        FrameOfData::parse_payload(&d, Version::V4_1).expect("the control parses"),
        f,
        "reach control: unpatched, it parses back equal"
    );
    // The one marker set's entries are `"wand\0"`, a four-byte marker count and
    // one twelve-byte marker: 21 bytes, declared by the header at `[8..12]`.
    // Raising that declaration to 22 leaves both sides non-zero and the entries
    // falling exactly one byte short of it.
    assert_eq!(
        i32::from_le_bytes(d[8..12].try_into().expect("four bytes")),
        21,
        "the declared marker-set size the patch below moves"
    );
    d[8..12].copy_from_slice(&22i32.to_le_bytes());
    match FrameOfData::parse_payload(&d, Version::V4_1) {
        Err(natnet::Error::SectionSizeMismatch { declared, consumed }) => {
            assert_eq!(
                (declared, consumed),
                (22, 21),
                "both fields, not just the variant"
            );
        }
        other => panic!("22 declared against 21 consumed must be refused, got {other:?}"),
    }
}

/// A marker missing one byte cannot read into the following section.
#[test]
fn a_typed_section_whose_entries_overrun_a_non_zero_declared_size_by_one_is_refused() {
    let f = FrameOfData::parse(&well_formed_frame(), Version::V4_1).expect("the fixture parses");
    let mut d = f.to_payload(Version::V4_1).expect("valid fixture encodes");
    assert_eq!(
        FrameOfData::parse_payload(&d, Version::V4_1).expect("the control parses"),
        f,
        "reach control: unpatched, it parses back equal"
    );
    // The one marker set's entries are `"wand\0"`, a four-byte marker count and
    // one twelve-byte marker: 21 bytes, declared by the header at `[8..12]`.
    // Lowering that declaration to 20 leaves both sides non-zero, and the
    // entries then overrun it by exactly one byte.
    assert_eq!(
        i32::from_le_bytes(d[8..12].try_into().expect("four bytes")),
        21,
        "the declared marker-set size the patch below moves"
    );
    d[8..12].copy_from_slice(&20i32.to_le_bytes());
    match FrameOfData::parse_payload(&d, Version::V4_1) {
        Err(natnet::Error::ImplausibleCount { count, available }) => {
            assert_eq!((count, available), (1, 11));
        }
        other => panic!("bounded section must refuse the entry before overread, got {other:?}"),
    }
}
