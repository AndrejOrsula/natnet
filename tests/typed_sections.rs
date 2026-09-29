//! Typed skeleton, asset, force-plate and device frame sections and typed
//! MODELDEF dataset types 2–6.
//!
//! Every wire fixture here is assembled field by field from the NatNet SDK
//! 4.1.0.0 `PacketClient.cpp` unpackers, independently of the crate's
//! encoders, so a shared encoder/decoder mistake cannot make both sides agree.
//! The layouts are spec-derived: no real Motive capture with these sections
//! is available.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "fixture helpers assert on literal values"
)]

use std::panic::{AssertUnwindSafe, catch_unwind};

use natnet::frame::{decode_id, encode_id};
use natnet::modeldef::{TYPE_ASSET, TYPE_CAMERA, TYPE_DEVICE, TYPE_FORCE_PLATE, TYPE_SKELETON};
use natnet::{
    Asset, AssetDef, CameraDef, DataSet, Decoder, Device, DeviceDef, Error, ForcePlate,
    ForcePlateDef, FrameOfData, LabeledMarker, MarkerDef, ModelDef, ParsePolicy, PlannedBody,
    PlannedSections, RawSection, RigidBody, RigidBodyDef, Skeleton, SkeletonDef, SyntheticServer,
    Version,
};

const VERSIONS: [Version; 4] = [
    Version::new(3, 0, 0, 0),
    Version::new(3, 1, 0, 0),
    Version::new(4, 0, 0, 0),
    Version::V4_1,
];

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

fn put_f32s(out: &mut Vec<u8>, values: &[f32]) {
    for value in values {
        put_f32(out, *value);
    }
}

fn put_cstr(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(value.as_bytes());
    out.push(0);
}

fn sized(version: Version) -> bool {
    version.major == 4 && version.minor == 1
}

// ---------------------------------------------------------------------------
// FrameOfData fixtures
// ---------------------------------------------------------------------------

fn frame_body(id: i32, x: f32) -> RigidBody {
    RigidBody {
        id,
        position: [x, x + 1.0, x + 2.0],
        orientation: [0.0, 0.0, 0.5, 0.75],
        mean_marker_error: 0.001,
        params: 1,
    }
}

fn frame_marker(id: i32, x: f32) -> LabeledMarker {
    LabeledMarker {
        id,
        position: [x, -x, 2.0 * x],
        size: 0.014,
        params: 0x0028,
        residual: 0.0002,
    }
}

/// The literal 38-byte rigid body of `UnpackRigidBodyData`.
fn put_frame_body(out: &mut Vec<u8>, body: &RigidBody) {
    put_i32(out, body.id);
    put_f32s(out, &body.position);
    put_f32s(out, &body.orientation);
    put_f32(out, body.mean_marker_error);
    put_u16(out, body.params);
}

/// The literal 26-byte marker of `UnpackAssetMarkerData`.
fn put_frame_marker(out: &mut Vec<u8>, marker: &LabeledMarker) {
    put_i32(out, marker.id);
    put_f32s(out, &marker.position);
    put_f32(out, marker.size);
    put_u16(out, marker.params);
    put_f32(out, marker.residual);
}

fn expected_skeletons() -> Vec<Skeleton> {
    vec![
        Skeleton {
            id: 1,
            rigid_bodies: vec![
                frame_body(encode_id(1, 1).unwrap(), 1.0),
                frame_body(encode_id(1, 2).unwrap(), 4.0),
            ],
        },
        Skeleton {
            id: 2,
            rigid_bodies: vec![],
        },
    ]
}

fn expected_assets() -> Vec<Asset> {
    vec![Asset {
        id: 5,
        rigid_bodies: vec![frame_body(encode_id(5, 1).unwrap(), 7.0)],
        markers: vec![
            frame_marker(encode_id(5, 1).unwrap(), 0.5),
            frame_marker(encode_id(5, 2).unwrap(), 1.5),
        ],
    }]
}

fn expected_force_plates() -> Vec<ForcePlate> {
    vec![ForcePlate {
        id: 3,
        channels: vec![vec![1.0, 2.0, 3.0], vec![], vec![-0.25]],
    }]
}

fn expected_devices() -> Vec<Device> {
    vec![
        Device {
            id: 8,
            channels: vec![vec![0.5]],
        },
        Device {
            id: 9,
            channels: vec![],
        },
    ]
}

/// `UnpackSkeletonData` entries for `expected_skeletons`.
fn skeleton_body() -> Vec<u8> {
    let mut out = Vec::new();
    put_i32(&mut out, 1);
    put_i32(&mut out, 2);
    put_frame_body(&mut out, &frame_body(0x0001_0001, 1.0));
    put_frame_body(&mut out, &frame_body(0x0001_0002, 4.0));
    put_i32(&mut out, 2);
    put_i32(&mut out, 0);
    out
}

/// `UnpackAssetData` entries for `expected_assets`.
fn asset_body() -> Vec<u8> {
    let mut out = Vec::new();
    put_i32(&mut out, 5);
    put_i32(&mut out, 1);
    put_frame_body(&mut out, &frame_body(0x0005_0001, 7.0));
    put_i32(&mut out, 2);
    put_frame_marker(&mut out, &frame_marker(0x0005_0001, 0.5));
    put_frame_marker(&mut out, &frame_marker(0x0005_0002, 1.5));
    out
}

/// `UnpackForcePlateData` entries for `expected_force_plates`.
fn force_plate_body() -> Vec<u8> {
    let mut out = Vec::new();
    put_i32(&mut out, 3);
    put_i32(&mut out, 3);
    put_i32(&mut out, 3);
    put_f32s(&mut out, &[1.0, 2.0, 3.0]);
    put_i32(&mut out, 0);
    put_i32(&mut out, 1);
    put_f32(&mut out, -0.25);
    out
}

/// `UnpackDeviceData` entries for `expected_devices`.
fn device_body() -> Vec<u8> {
    let mut out = Vec::new();
    put_i32(&mut out, 8);
    put_i32(&mut out, 1);
    put_i32(&mut out, 1);
    put_f32(&mut out, 0.5);
    put_i32(&mut out, 9);
    put_i32(&mut out, 0);
    out
}

struct Sections<'a> {
    skeletons: (i32, &'a [u8]),
    assets: (i32, &'a [u8]),
    force_plates: (i32, &'a [u8]),
    devices: (i32, &'a [u8]),
}

fn put_section(out: &mut Vec<u8>, version: Version, (count, body): (i32, &[u8])) {
    put_i32(out, count);
    if sized(version) {
        put_i32(
            out,
            i32::try_from(body.len()).expect("literal section fits"),
        );
    }
    out.extend_from_slice(body);
}

/// A literal FrameOfData payload in `version`'s layout with empty typed
/// sections and the given raw section bodies.
fn frame_payload(version: Version, sections: &Sections<'_>) -> Vec<u8> {
    let mut out = Vec::new();
    put_i32(&mut out, 55);
    for _ in 0..3 {
        put_section(&mut out, version, (0, &[])); // marker sets, other markers, rigid bodies
    }
    put_section(&mut out, version, sections.skeletons);
    if sized(version) {
        put_section(&mut out, version, sections.assets);
    }
    put_section(&mut out, version, (0, &[])); // labelled markers
    put_section(&mut out, version, sections.force_plates);
    put_section(&mut out, version, sections.devices);
    put_u32(&mut out, 1);
    put_u32(&mut out, 2);
    put_f64(&mut out, 3.5);
    put_u64(&mut out, 4);
    put_u64(&mut out, 5);
    put_u64(&mut out, 6);
    if sized(version) {
        put_u32(&mut out, 7);
        put_u32(&mut out, 8);
    }
    put_u16(&mut out, 0);
    put_i32(&mut out, 0);
    out
}

fn nonempty_frame_payload(version: Version) -> Vec<u8> {
    let skeletons = skeleton_body();
    let assets = asset_body();
    let force_plates = force_plate_body();
    let devices = device_body();
    frame_payload(
        version,
        &Sections {
            skeletons: (2, &skeletons),
            assets: (1, &assets),
            force_plates: (1, &force_plates),
            devices: (2, &devices),
        },
    )
}

#[test]
fn nonempty_frame_sections_decode_and_reencode_byte_exactly_per_version() {
    for version in VERSIONS {
        let payload = nonempty_frame_payload(version);
        for policy in [ParsePolicy::Strict, ParsePolicy::Compatible] {
            let frame = FrameOfData::parse_payload_with_policy(&payload, version, policy)
                .unwrap_or_else(|e| panic!("{version} {policy:?}: {e}"));

            let skeletons = frame.decode_skeletons().expect("skeletons decode");
            let assets = frame.decode_assets().expect("assets decode");
            let force_plates = frame.decode_force_plates().expect("force plates decode");
            let devices = frame.decode_devices().expect("devices decode");
            assert_eq!(skeletons, expected_skeletons(), "{version}");
            assert_eq!(force_plates, expected_force_plates(), "{version}");
            assert_eq!(devices, expected_devices(), "{version}");
            if sized(version) {
                assert_eq!(assets, expected_assets());
            } else {
                assert!(assets.is_empty(), "{version} carries no assets");
            }

            // from(decode(raw)) == raw, byte for byte.
            assert_eq!(
                RawSection::from_skeletons(&skeletons),
                Ok(frame.skeletons.clone())
            );
            assert_eq!(RawSection::from_assets(&assets), Ok(frame.assets.clone()));
            assert_eq!(
                RawSection::from_force_plates(&force_plates),
                Ok(frame.force_plates.clone())
            );
            assert_eq!(
                RawSection::from_devices(&devices),
                Ok(frame.devices.clone())
            );
            assert_eq!(frame.to_payload(version), Ok(payload.clone()), "{version}");
        }

        // A frame built from typed values encodes to the literal payload, and
        // decode(from(x)) == x.
        let mut built = FrameOfData::parse_payload(
            &frame_payload(
                version,
                &Sections {
                    skeletons: (0, &[]),
                    assets: (0, &[]),
                    force_plates: (0, &[]),
                    devices: (0, &[]),
                },
            ),
            version,
        )
        .expect("empty control");
        built.skeletons = RawSection::from_skeletons(&expected_skeletons()).unwrap();
        if sized(version) {
            built.assets = RawSection::from_assets(&expected_assets()).unwrap();
        }
        built.force_plates = RawSection::from_force_plates(&expected_force_plates()).unwrap();
        built.devices = RawSection::from_devices(&expected_devices()).unwrap();
        assert_eq!(built.to_payload(version), Ok(payload), "{version}");
        assert_eq!(built.decode_skeletons(), Ok(expected_skeletons()));
        assert_eq!(built.decode_force_plates(), Ok(expected_force_plates()));
        assert_eq!(built.decode_devices(), Ok(expected_devices()));
    }
}

#[test]
fn synthetic_server_emits_nonempty_typed_sections_at_every_version() {
    for version in VERSIONS {
        let server = SyntheticServer::new(version, 1_000_000);
        let bodies = [PlannedBody::tracked(
            3,
            [1.0, 2.0, 3.0],
            [0.0, 0.0, 0.0, 1.0],
        )];
        let sections = PlannedSections {
            skeletons: expected_skeletons(),
            assets: if sized(version) {
                expected_assets()
            } else {
                Vec::new()
            },
            force_plates: expected_force_plates(),
            devices: expected_devices(),
        };
        let wire = server
            .datagram_with_sections(9, 0.25, &bodies, &sections)
            .expect("typed sections encode");
        let decoded = Decoder::new(version).unwrap().parse(&wire).expect("strict");
        let natnet::Message::FrameOfData(frame) = decoded.message() else {
            panic!("frame expected");
        };
        assert_eq!(frame.rigid_bodies.len(), 1);
        assert_eq!(frame.decode_skeletons(), Ok(sections.skeletons.clone()));
        assert_eq!(frame.decode_assets(), Ok(sections.assets.clone()));
        assert_eq!(
            frame.decode_force_plates(),
            Ok(sections.force_plates.clone())
        );
        assert_eq!(frame.decode_devices(), Ok(sections.devices.clone()));
        assert_eq!(decoded.to_datagram(), Ok(wire.clone()));
        assert_eq!(
            **frame,
            server
                .frame_with_sections(9, 0.25, &bodies, &sections)
                .unwrap()
        );

        if !sized(version) {
            let with_assets = PlannedSections {
                assets: expected_assets(),
                ..sections
            };
            assert!(matches!(
                server.datagram_with_sections(9, 0.25, &bodies, &with_assets),
                Err(Error::InvalidValue {
                    field: "assets",
                    ..
                })
            ));
        }
    }
}

#[test]
fn a_nan_sample_costs_value_equality_but_not_bytes() {
    let mut body = Vec::new();
    put_i32(&mut body, 3);
    put_i32(&mut body, 1);
    put_i32(&mut body, 1);
    let nan_bits = 0x7fc0_1234u32;
    put_u32(&mut body, nan_bits);
    let raw = RawSection {
        count: 1,
        payload: body,
    };
    let mut frame =
        FrameOfData::parse_payload(&nonempty_frame_payload(Version::V4_1), Version::V4_1)
            .expect("fixture");
    frame.force_plates = raw.clone();
    let plates = frame.decode_force_plates().unwrap();
    assert_eq!(plates[0].channels[0][0].to_bits(), nan_bits);
    assert_ne!(plates, plates.clone(), "a NaN is unequal to itself");
    assert_eq!(RawSection::from_force_plates(&plates), Ok(raw));
}

/// Each raw frame section with one byte of residue or one byte missing.
fn frame_section_mutations() -> Vec<(&'static str, Vec<u8>, Vec<u8>)> {
    let cases = [
        ("skeletons", skeleton_body()),
        ("assets", asset_body()),
        ("force_plates", force_plate_body()),
        ("devices", device_body()),
    ];
    cases
        .into_iter()
        .map(|(name, body)| {
            let mut residue = body.clone();
            residue.push(0xaa);
            let truncated = body[..body.len() - 1].to_vec();
            (name, residue, truncated)
        })
        .collect()
}

fn with_section(name: &str, body: &[u8]) -> Vec<u8> {
    let skeletons = skeleton_body();
    let assets = asset_body();
    let force_plates = force_plate_body();
    let devices = device_body();
    let mut sections = Sections {
        skeletons: (2, &skeletons),
        assets: (1, &assets),
        force_plates: (1, &force_plates),
        devices: (2, &devices),
    };
    match name {
        "skeletons" => sections.skeletons.1 = body,
        "assets" => sections.assets.1 = body,
        "force_plates" => sections.force_plates.1 = body,
        _ => sections.devices.1 = body,
    }
    frame_payload(Version::V4_1, &sections)
}

fn decode_named(frame: &FrameOfData, name: &str) -> natnet::Result<()> {
    match name {
        "skeletons" => frame.decode_skeletons().map(drop),
        "assets" => frame.decode_assets().map(drop),
        "force_plates" => frame.decode_force_plates().map(drop),
        _ => frame.decode_devices().map(drop),
    }
}

#[test]
fn residue_and_truncation_in_each_frame_section_are_refused_strictly_and_by_decode() {
    for (name, residue, truncated) in frame_section_mutations() {
        // Residue: strict frame parsing refuses it; compatibility parsing keeps
        // the bytes, and the typed decoder refuses them with both sizes.
        let payload = with_section(name, &residue);
        assert_eq!(
            FrameOfData::parse_payload_with_policy(&payload, Version::V4_1, ParsePolicy::Strict),
            Err(Error::SectionSizeMismatch {
                declared: residue.len(),
                consumed: residue.len() - 1,
            }),
            "{name}"
        );
        let frame = FrameOfData::parse_payload_with_policy(
            &payload,
            Version::V4_1,
            ParsePolicy::Compatible,
        )
        .expect("compatibility keeps the raw body");
        assert_eq!(
            decode_named(&frame, name),
            Err(Error::SectionSizeMismatch {
                declared: residue.len(),
                consumed: residue.len() - 1,
            }),
            "{name}"
        );
        assert_eq!(frame.to_payload_lossy(Version::V4_1), Ok(payload.clone()));
        assert!(frame.to_payload(Version::V4_1).is_err(), "{name}");

        // Truncation: the last entry cannot complete.
        let payload = with_section(name, &truncated);
        assert!(
            FrameOfData::parse_payload_with_policy(&payload, Version::V4_1, ParsePolicy::Strict)
                .is_err(),
            "{name}"
        );
        let frame = FrameOfData::parse_payload_with_policy(
            &payload,
            Version::V4_1,
            ParsePolicy::Compatible,
        )
        .expect("compatibility keeps the raw body");
        assert!(
            matches!(
                decode_named(&frame, name),
                Err(Error::UnexpectedEof { .. } | Error::ImplausibleCount { .. })
            ),
            "{name}"
        );
    }
}

#[test]
fn decoders_refuse_negative_and_implausible_counts() {
    let mut frame =
        FrameOfData::parse_payload(&nonempty_frame_payload(Version::V4_1), Version::V4_1)
            .expect("fixture");
    frame.skeletons.count = -1;
    assert_eq!(
        frame.decode_skeletons(),
        Err(Error::InvalidCount {
            count: -1,
            offset: 0
        })
    );
    frame.devices = RawSection {
        count: 3,
        payload: vec![0; 16],
    };
    assert_eq!(
        frame.decode_devices(),
        Err(Error::ImplausibleCount {
            count: 3,
            available: 16
        })
    );
    // A nested negative channel count inside an otherwise sized body.
    let mut body = Vec::new();
    put_i32(&mut body, 3);
    put_i32(&mut body, -2);
    frame.force_plates = RawSection {
        count: 1,
        payload: body,
    };
    assert_eq!(
        frame.decode_force_plates(),
        Err(Error::InvalidCount {
            count: -2,
            offset: 4
        })
    );
}

#[test]
fn legacy_checked_encoding_refuses_raw_sections_the_typed_grammar_refuses() {
    let version = Version::new(4, 0, 0, 0);
    let mut frame =
        FrameOfData::parse_payload(&nonempty_frame_payload(version), version).expect("fixture");
    let mut residue = skeleton_body();
    residue.push(0);
    frame.skeletons.payload = residue;
    assert!(matches!(
        frame.to_payload(version),
        Err(Error::SectionSizeMismatch { .. })
    ));
}

#[test]
fn composite_ids_split_and_rebuild_like_the_sdk() {
    assert_eq!(decode_id(0x0003_0005), (3, 5));
    assert_eq!(decode_id(0x0001_ffff), (1, 0xffff));
    assert_eq!(decode_id(-1), (-1, 0xffff));
    assert_eq!(decode_id(i32::MIN), (-32_768, 0));
    for id in [
        0,
        1,
        0x0003_0005,
        -1,
        i32::MIN,
        i32::MAX,
        0x7fff_0000,
        -65_536,
    ] {
        let (entity, member) = decode_id(id);
        assert_eq!(encode_id(entity, member), Ok(id), "{id:#x}");
    }
    assert!(matches!(
        encode_id(32_768, 0),
        Err(Error::InvalidValue { .. })
    ));
    assert!(matches!(
        encode_id(-32_769, 0),
        Err(Error::InvalidValue { .. })
    ));
    assert!(matches!(
        encode_id(0, 65_536),
        Err(Error::InvalidValue { .. })
    ));
    assert!(matches!(encode_id(0, -1), Err(Error::InvalidValue { .. })));
}

// ---------------------------------------------------------------------------
// MODELDEF fixtures
// ---------------------------------------------------------------------------

struct RigidMarker {
    position: [f32; 3],
    label: i32,
    name: &'static str,
}

/// A literal `UnpackRigidBodyDescription` body and its typed value.
fn rigid_body_def(
    version: Version,
    name: &str,
    id: i32,
    parent_id: i32,
    offset: [f32; 3],
    markers: &[RigidMarker],
) -> (Vec<u8>, RigidBodyDef) {
    let mut tail = Vec::new();
    put_i32(&mut tail, i32::try_from(markers.len()).unwrap());
    for marker in markers {
        put_f32s(&mut tail, &marker.position);
    }
    for marker in markers {
        put_i32(&mut tail, marker.label);
    }
    if version.major >= 4 {
        for marker in markers {
            put_cstr(&mut tail, marker.name);
        }
    }
    let mut bytes = Vec::new();
    put_cstr(&mut bytes, name);
    put_i32(&mut bytes, id);
    put_i32(&mut bytes, parent_id);
    put_f32s(&mut bytes, &offset);
    bytes.extend_from_slice(&tail);
    let value = RigidBodyDef {
        name: name.into(),
        id,
        parent_id,
        offset,
        tail,
    };
    (bytes, value)
}

fn hip(version: Version) -> (Vec<u8>, RigidBodyDef) {
    rigid_body_def(
        version,
        "Hip",
        1,
        -1,
        [0.0, 1.0, 0.0],
        &[
            RigidMarker {
                position: [0.1, 0.2, 0.3],
                label: 0,
                name: "Hip_M1",
            },
            RigidMarker {
                position: [0.4, 0.5, 0.6],
                label: 7,
                name: "Hip_M2",
            },
        ],
    )
}

fn spine(version: Version) -> (Vec<u8>, RigidBodyDef) {
    rigid_body_def(version, "Spine", 2, 1, [0.0, 0.1, 0.0], &[])
}

fn skeleton_def(version: Version) -> (Vec<u8>, DataSet) {
    let (hip_bytes, hip) = hip(version);
    let (spine_bytes, spine) = spine(version);
    let mut out = Vec::new();
    put_cstr(&mut out, "Skel");
    put_i32(&mut out, 1);
    put_i32(&mut out, 2);
    out.extend_from_slice(&hip_bytes);
    out.extend_from_slice(&spine_bytes);
    let value = DataSet::Skeleton(SkeletonDef {
        name: "Skel".into(),
        id: 1,
        rigid_bodies: vec![hip, spine],
    });
    (out, value)
}

fn force_plate_def() -> (Vec<u8>, DataSet) {
    let mut calibration_matrix = [[0.0f32; 12]; 12];
    for (index, value) in calibration_matrix.iter_mut().flatten().enumerate() {
        *value = index as f32 * 0.5;
    }
    let corners = [
        [0.0, 0.0, 0.0],
        [0.6, 0.0, 0.0],
        [0.6, 0.0, 0.9],
        [0.0, 0.0, 0.9],
    ];
    let channel_names = ["Fx", "Fy", "Fz", "Mx", "My", "Mz"];
    let mut out = Vec::new();
    put_i32(&mut out, 1);
    put_cstr(&mut out, "FP-001");
    put_f32(&mut out, 0.6);
    put_f32(&mut out, 0.9);
    put_f32s(&mut out, &[0.01, 0.02, -0.03]);
    for index in 0..144 {
        put_f32(&mut out, index as f32 * 0.5);
    }
    for corner in corners {
        put_f32s(&mut out, &corner);
    }
    put_i32(&mut out, 2);
    put_i32(&mut out, 0);
    put_i32(&mut out, 6);
    for name in channel_names {
        put_cstr(&mut out, name);
    }
    let value = DataSet::ForcePlate(Box::new(ForcePlateDef {
        id: 1,
        serial_number: "FP-001".into(),
        width: 0.6,
        length: 0.9,
        origin: [0.01, 0.02, -0.03],
        calibration_matrix,
        corners,
        plate_type: 2,
        channel_data_type: 0,
        channel_names: channel_names.map(String::from).to_vec(),
    }));
    (out, value)
}

fn device_def() -> (Vec<u8>, DataSet) {
    let mut out = Vec::new();
    put_i32(&mut out, 2);
    put_cstr(&mut out, "NI-DAQ");
    put_cstr(&mut out, "SN-1");
    put_i32(&mut out, 1);
    put_i32(&mut out, 3);
    put_i32(&mut out, 2);
    put_cstr(&mut out, "ai0");
    put_cstr(&mut out, "ai1");
    let value = DataSet::Device(DeviceDef {
        id: 2,
        name: "NI-DAQ".into(),
        serial_number: "SN-1".into(),
        device_type: 1,
        channel_data_type: 3,
        channel_names: vec!["ai0".into(), "ai1".into()],
    });
    (out, value)
}

fn camera_def() -> (Vec<u8>, DataSet) {
    let mut out = Vec::new();
    put_cstr(&mut out, "Prime 13 #1");
    put_f32s(&mut out, &[1.0, 2.5, -3.0]);
    put_f32s(&mut out, &[0.1, 0.2, 0.3, 0.9]);
    let value = DataSet::Camera(CameraDef {
        name: "Prime 13 #1".into(),
        position: [1.0, 2.5, -3.0],
        orientation: [0.1, 0.2, 0.3, 0.9],
    });
    (out, value)
}

fn asset_def(version: Version) -> (Vec<u8>, DataSet) {
    let (body_bytes, body) = rigid_body_def(
        version,
        "A1",
        0x0003_0001,
        -1,
        [0.0; 3],
        &[RigidMarker {
            position: [1.0, 1.0, 1.0],
            label: 3,
            name: "A1_M1",
        }],
    );
    let markers = [
        MarkerDef {
            name: "A1_M1".into(),
            id: 0x0003_0001,
            position: [1.0, 1.0, 1.0],
            size: 0.014,
            params: 1,
        },
        MarkerDef {
            name: "A1_M2".into(),
            id: 0x0003_0002,
            position: [2.0, 1.0, 1.0],
            size: 0.012,
            params: 0,
        },
    ];
    let mut out = Vec::new();
    put_cstr(&mut out, "Trained");
    put_i32(&mut out, 1);
    put_i32(&mut out, 3);
    put_i32(&mut out, 1);
    out.extend_from_slice(&body_bytes);
    put_i32(&mut out, 2);
    for marker in &markers {
        put_cstr(&mut out, &marker.name);
        put_i32(&mut out, marker.id);
        put_f32s(&mut out, &marker.position);
        put_f32(&mut out, marker.size);
        put_u16(&mut out, marker.params);
    }
    let value = DataSet::Asset(AssetDef {
        name: "Trained".into(),
        asset_type: 1,
        id: 3,
        rigid_bodies: vec![body],
        markers: markers.to_vec(),
    });
    (out, value)
}

/// Types 2–6, each with a literal body and its typed value.
fn typed_datasets(version: Version) -> Vec<(i32, Vec<u8>, DataSet)> {
    [
        (TYPE_SKELETON, skeleton_def(version)),
        (TYPE_FORCE_PLATE, force_plate_def()),
        (TYPE_DEVICE, device_def()),
        (TYPE_CAMERA, camera_def()),
        (TYPE_ASSET, asset_def(version)),
    ]
    .into_iter()
    .map(|(type_id, (body, value))| (type_id, body, value))
    .collect()
}

fn model_payload(version: Version, datasets: &[(i32, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    put_i32(&mut out, i32::try_from(datasets.len()).unwrap());
    for (type_id, body) in datasets {
        put_i32(&mut out, *type_id);
        if sized(version) {
            put_i32(&mut out, i32::try_from(body.len()).unwrap());
        }
        out.extend_from_slice(body);
    }
    out
}

#[test]
fn typed_modeldef_datasets_round_trip_byte_exactly_per_version() {
    for version in VERSIONS {
        let datasets = typed_datasets(version);
        let entries: Vec<(i32, &[u8])> = datasets
            .iter()
            .map(|(type_id, body, _)| (*type_id, body.as_slice()))
            .collect();
        let payload = model_payload(version, &entries);
        let expected = ModelDef {
            datasets: datasets.iter().map(|(_, _, value)| value.clone()).collect(),
        };
        for (type_id, _, value) in &datasets {
            assert_eq!(value.type_id(), *type_id);
        }

        for policy in [ParsePolicy::Strict, ParsePolicy::Compatible] {
            assert_eq!(
                ModelDef::parse_payload_with_policy(&payload, version, policy),
                Ok(expected.clone()),
                "{version} {policy:?}"
            );
        }
        assert_eq!(
            expected.to_payload_version(version),
            Ok(payload.clone()),
            "{version}"
        );

        // The same bytes through the generator and the versioned decoder.
        let wire = SyntheticServer::new(version, 1)
            .model_def_datagram(&expected)
            .expect("generator encodes");
        assert_eq!(&wire[4..], payload.as_slice());
        let decoded = Decoder::new(version).unwrap().parse(&wire).expect("strict");
        assert_eq!(
            decoded.message(),
            &natnet::Message::ModelDef(expected.clone())
        );
        assert_eq!(decoded.to_datagram(), Ok(wire));
    }
}

#[test]
fn v4_1_typed_body_residue_and_truncation_strict_rejects_compatible_keeps_other() {
    for (type_id, body, _) in typed_datasets(Version::V4_1) {
        let mut residue = body.clone();
        residue.push(0xaa);
        let truncated = body[..body.len() - 1].to_vec();
        for (label, malformed) in [("residue", residue), ("truncated", truncated)] {
            let payload = model_payload(Version::V4_1, &[(type_id, &malformed)]);
            let strict =
                ModelDef::parse_payload_with_policy(&payload, Version::V4_1, ParsePolicy::Strict);
            if label == "residue" {
                assert_eq!(
                    strict,
                    Err(Error::TrailingBytes { remaining: 1 }),
                    "type {type_id}"
                );
            } else {
                assert!(
                    matches!(
                        strict,
                        Err(Error::UnexpectedEof { .. }
                            | Error::UnterminatedString { .. }
                            | Error::ImplausibleCount { .. })
                    ),
                    "type {type_id} {label}: {strict:?}"
                );
            }
            let compatible = ModelDef::parse_payload_with_policy(
                &payload,
                Version::V4_1,
                ParsePolicy::Compatible,
            )
            .expect("compatible fallback");
            assert_eq!(
                compatible.datasets,
                vec![DataSet::Other {
                    type_id,
                    body: malformed.clone(),
                }],
                "type {type_id} {label}"
            );
            assert_eq!(
                compatible.to_payload_version(Version::V4_1),
                Ok(payload),
                "type {type_id} {label}: no bytes lost"
            );
        }
    }
}

#[test]
fn v4_1_negative_nested_count_strict_rejects_compatible_keeps_other() {
    let (mut body, _) = skeleton_def(Version::V4_1);
    // "Skel\0", id, then the bone count at byte 9.
    body[9..13].copy_from_slice(&(-1i32).to_le_bytes());
    let payload = model_payload(Version::V4_1, &[(TYPE_SKELETON, &body)]);
    assert_eq!(
        ModelDef::parse_payload_with_policy(&payload, Version::V4_1, ParsePolicy::Strict),
        Err(Error::InvalidCount {
            count: -1,
            offset: 12 + 9
        })
    );
    assert_eq!(
        ModelDef::parse_payload_version(&payload, Version::V4_1).map(|m| m.datasets),
        Ok(vec![DataSet::Other {
            type_id: TYPE_SKELETON,
            body,
        }])
    );
}

#[test]
fn legacy_truncated_typed_dataset_is_refused_under_both_policies() {
    for version in &VERSIONS[..3] {
        for (type_id, body, _) in typed_datasets(*version) {
            let payload = model_payload(*version, &[(type_id, &body[..body.len() - 1])]);
            for policy in [ParsePolicy::Strict, ParsePolicy::Compatible] {
                assert!(
                    ModelDef::parse_payload_with_policy(&payload, *version, policy).is_err(),
                    "{version} type {type_id} {policy:?}"
                );
            }
        }
    }
}

#[test]
fn a_non_utf8_name_is_typed_lossily_when_strict_and_kept_verbatim_when_compatible() {
    for version in VERSIONS {
        let (mut body, _) = camera_def();
        body[0] = 0xff; // "\xffrime 13 #1"
        let payload = model_payload(version, &[(TYPE_CAMERA, &body)]);
        let strict = ModelDef::parse_payload_with_policy(&payload, version, ParsePolicy::Strict)
            .expect("strict accepts lossy strings");
        let DataSet::Camera(camera) = &strict.datasets[0] else {
            panic!("strict keeps the typed camera");
        };
        assert!(camera.name.starts_with('\u{fffd}'));
        assert_ne!(strict.to_payload_version(version), Ok(payload.clone()));

        let compatible = ModelDef::parse_payload_version(&payload, version).expect("compatible");
        assert_eq!(
            compatible.datasets,
            vec![DataSet::Other {
                type_id: TYPE_CAMERA,
                body,
            }],
            "{version}"
        );
        assert_eq!(compatible.to_payload_version(version), Ok(payload));
    }
}

#[test]
fn checked_encoding_refuses_unrepresentable_typed_datasets() {
    let version = Version::V4_1;
    let (_, skeleton) = skeleton_def(version);
    let DataSet::Skeleton(mut skeleton) = skeleton else {
        panic!("fixture variant")
    };
    // A bone tail has no body size of its own: a suffix is refused.
    skeleton.rigid_bodies[1].tail.push(0);
    let model = ModelDef {
        datasets: vec![DataSet::Skeleton(skeleton.clone())],
    };
    assert_eq!(
        model.to_payload_version(version),
        Err(Error::TrailingBytes { remaining: 1 })
    );
    skeleton.rigid_bodies[1].tail.clear();
    assert!(matches!(
        ModelDef {
            datasets: vec![DataSet::Skeleton(skeleton)],
        }
        .to_payload_version(version),
        Err(Error::UnexpectedEof { .. })
    ));

    let (_, camera) = camera_def();
    let DataSet::Camera(mut camera) = camera else {
        panic!("fixture variant")
    };
    camera.name = "a\0b".into();
    assert!(matches!(
        ModelDef {
            datasets: vec![DataSet::Camera(camera)],
        }
        .to_payload_version(version),
        Err(Error::InteriorNul {
            field: "camera name",
            ..
        })
    ));

    let (_, device) = device_def();
    let DataSet::Device(mut device) = device else {
        panic!("fixture variant")
    };
    device.channel_names.push("x\0".into());
    assert!(matches!(
        ModelDef {
            datasets: vec![DataSet::Device(device)],
        }
        .to_payload_version(version),
        Err(Error::InteriorNul {
            field: "device channel name",
            ..
        })
    ));

    // A legacy `Other` body must still match the typed grammar exactly.
    let (mut body, _) = camera_def();
    body.push(0);
    assert!(
        ModelDef {
            datasets: vec![DataSet::Other {
                type_id: TYPE_CAMERA,
                body,
            }],
        }
        .to_payload_version(Version::new(4, 0, 0, 0))
        .is_err()
    );
}

#[test]
fn cross_layout_conversion_refuses_bone_tails_but_converts_layout_free_types() {
    let source = Version::V4_1;
    let (skeleton_body, _) = skeleton_def(source);
    let wire = [
        &[5u8, 0, 0, 0][..],
        &model_payload(source, &[(TYPE_SKELETON, &skeleton_body)]),
    ]
    .concat();
    let decoded = Decoder::new(source).unwrap().parse(&wire).unwrap();
    assert!(matches!(
        decoded.to_datagram_for(Version::new(3, 0, 0, 0)),
        Err(Error::InvalidValue { .. })
    ));

    let layout_free: Vec<(i32, Vec<u8>)> = [force_plate_def(), device_def(), camera_def()]
        .into_iter()
        .zip([TYPE_FORCE_PLATE, TYPE_DEVICE, TYPE_CAMERA])
        .map(|((body, _), type_id)| (type_id, body))
        .collect();
    let entries: Vec<(i32, &[u8])> = layout_free
        .iter()
        .map(|(type_id, body)| (*type_id, body.as_slice()))
        .collect();
    let wire = [&[5u8, 0, 0, 0][..], &model_payload(source, &entries)].concat();
    let decoded = Decoder::new(source).unwrap().parse(&wire).unwrap();
    let legacy = Version::new(3, 0, 0, 0);
    let converted = decoded
        .to_datagram_for(legacy)
        .expect("layout-free types convert");
    assert_eq!(&converted[4..], model_payload(legacy, &entries).as_slice());
}

// ---------------------------------------------------------------------------
// Randomized robustness
// ---------------------------------------------------------------------------

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

/// Checks the frame decode/encode property and returns a violation, if any.
fn frame_property(bytes: &[u8]) -> Option<String> {
    for version in VERSIONS {
        for policy in [ParsePolicy::Strict, ParsePolicy::Compatible] {
            let Ok(frame) = FrameOfData::parse_payload_with_policy(bytes, version, policy) else {
                continue;
            };
            let checks = [
                (
                    frame
                        .decode_skeletons()
                        .map(|x| RawSection::from_skeletons(&x)),
                    &frame.skeletons,
                ),
                (
                    frame.decode_assets().map(|x| RawSection::from_assets(&x)),
                    &frame.assets,
                ),
                (
                    frame
                        .decode_force_plates()
                        .map(|x| RawSection::from_force_plates(&x)),
                    &frame.force_plates,
                ),
                (
                    frame.decode_devices().map(|x| RawSection::from_devices(&x)),
                    &frame.devices,
                ),
            ];
            for (reencoded, raw) in checks {
                if let Ok(reencoded) = reencoded
                    && reencoded.as_ref() != Ok(raw)
                {
                    return Some(format!("{version} {policy:?}: {reencoded:?} != {raw:?}"));
                }
            }
            let _ = frame.to_payload(version);
        }
    }
    None
}

/// Checks that every typed type 2–6 dataset a parser returns re-encodes.
fn model_property(bytes: &[u8]) -> Option<String> {
    for version in VERSIONS {
        for policy in [ParsePolicy::Strict, ParsePolicy::Compatible] {
            let Ok(model) = ModelDef::parse_payload_with_policy(bytes, version, policy) else {
                continue;
            };
            for dataset in &model.datasets {
                if matches!(
                    dataset,
                    DataSet::Skeleton(_)
                        | DataSet::ForcePlate(_)
                        | DataSet::Device(_)
                        | DataSet::Camera(_)
                        | DataSet::Asset(_)
                ) {
                    let single = ModelDef {
                        datasets: vec![dataset.clone()],
                    };
                    if let Err(error) = single.to_payload_version(version) {
                        return Some(format!("{version} {policy:?}: {dataset:?}: {error}"));
                    }
                }
            }
            let _ = model.to_payload_version(version);
            let _ = model.to_payload_lossy_version(version);
        }
    }
    None
}

#[test]
fn random_mutations_of_typed_seeds_never_panic_and_keep_the_byte_property() {
    let mut seeds: Vec<(bool, Vec<u8>)> = Vec::new();
    for version in VERSIONS {
        seeds.push((true, nonempty_frame_payload(version)));
        let datasets = typed_datasets(version);
        let entries: Vec<(i32, &[u8])> = datasets
            .iter()
            .map(|(type_id, body, _)| (*type_id, body.as_slice()))
            .collect();
        seeds.push((false, model_payload(version, &entries)));
    }

    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    let mut decoded_frames = 0usize;
    for trial in 0..4_000 {
        let (is_frame, seed) = &seeds[(xorshift(&mut state) % seeds.len() as u64) as usize];
        let mut bytes = seed.clone();
        let flips = 1 + xorshift(&mut state) % 4;
        for _ in 0..flips {
            let at = (xorshift(&mut state) % bytes.len() as u64) as usize;
            bytes[at] = (xorshift(&mut state) & 0xff) as u8;
        }
        if xorshift(&mut state).is_multiple_of(4) {
            let len = (xorshift(&mut state) % (bytes.len() as u64 + 1)) as usize;
            bytes.truncate(len);
        }
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            if *is_frame {
                frame_property(&bytes)
            } else {
                model_property(&bytes)
            }
        }));
        match outcome {
            Ok(None) => {}
            Ok(Some(violation)) => panic!("trial {trial}: {violation}"),
            Err(payload) => {
                let message = payload
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or_else(|| payload.downcast_ref::<&str>().copied())
                    .unwrap_or("non-string panic payload");
                panic!("trial {trial} panicked ({message}) on {bytes:02x?}");
            }
        }
        if *is_frame
            && FrameOfData::parse_payload(&bytes, Version::V4_1)
                .is_ok_and(|f| f.decode_skeletons().is_ok_and(|s| !s.is_empty()))
        {
            decoded_frames += 1;
        }
    }
    // Reach: mutations must leave many frames whose typed sections decode.
    assert!(decoded_frames > 100, "only {decoded_frames} frames decoded");
}
