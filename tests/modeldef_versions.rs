use natnet::modeldef::{
    AssetDef, CameraDef, DataSet, DeviceDef, ForcePlateDef, MarkerDef, MarkerSetDef, ModelDef,
    RigidBodyDef, SkeletonDef, TYPE_ASSET, TYPE_CAMERA, TYPE_DEVICE, TYPE_FORCE_PLATE,
    TYPE_RIGID_BODY, TYPE_SKELETON,
};
use natnet::{Error, ParsePolicy, Version};

fn i32_bytes(value: i32) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

fn f32_bytes(value: f32) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

fn cstr(value: &str) -> Vec<u8> {
    let mut out = value.as_bytes().to_vec();
    out.push(0);
    out
}

fn append_vec3(out: &mut Vec<u8>, value: [f32; 3]) {
    for component in value {
        out.extend_from_slice(&f32_bytes(component));
    }
}

fn append_quat(out: &mut Vec<u8>, value: [f32; 4]) {
    for component in value {
        out.extend_from_slice(&f32_bytes(component));
    }
}

fn append_rigid_body(out: &mut Vec<u8>, name: &str, marker_name: Option<&str>) {
    out.extend_from_slice(&cstr(name));
    out.extend_from_slice(&i32_bytes(17));
    out.extend_from_slice(&i32_bytes(-1));
    append_vec3(out, [1.0, 2.0, 3.0]);
    out.extend_from_slice(&i32_bytes(1));
    append_vec3(out, [4.0, 5.0, 6.0]);
    out.extend_from_slice(&i32_bytes(0x0102_0304));
    if let Some(marker_name) = marker_name {
        out.extend_from_slice(&cstr(marker_name));
    }
}

fn type_three_body() -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&i32_bytes(3));
    out.extend_from_slice(&cstr("FP-3"));
    out.extend_from_slice(&f32_bytes(0.4));
    out.extend_from_slice(&f32_bytes(0.5));
    append_vec3(&mut out, [1.0, 2.0, 3.0]);
    for value in 0..144 {
        out.extend_from_slice(&f32_bytes(value as f32));
    }
    for value in 0..12 {
        out.extend_from_slice(&f32_bytes((200 + value) as f32));
    }
    out.extend_from_slice(&i32_bytes(4));
    out.extend_from_slice(&i32_bytes(5));
    out.extend_from_slice(&i32_bytes(2));
    out.extend_from_slice(&cstr("Fx"));
    out.extend_from_slice(&cstr("Fy"));
    out
}

fn type_four_body() -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&i32_bytes(4));
    out.extend_from_slice(&cstr("device"));
    out.extend_from_slice(&cstr("serial"));
    out.extend_from_slice(&i32_bytes(7));
    out.extend_from_slice(&i32_bytes(8));
    out.extend_from_slice(&i32_bytes(2));
    out.extend_from_slice(&cstr("ch0"));
    out.extend_from_slice(&cstr("ch1"));
    out
}

fn type_five_body() -> Vec<u8> {
    let mut out = cstr("camera");
    append_vec3(&mut out, [7.0, 8.0, 9.0]);
    append_quat(&mut out, [0.0, 0.0, 0.0, 1.0]);
    out
}

fn type_six_body(marker_name: &str, rigid_marker_name: Option<&str>) -> Vec<u8> {
    let mut out = cstr("asset");
    out.extend_from_slice(&i32_bytes(12));
    out.extend_from_slice(&i32_bytes(99));
    out.extend_from_slice(&i32_bytes(1));
    append_rigid_body(&mut out, "bone", rigid_marker_name);
    out.extend_from_slice(&i32_bytes(1));
    out.extend_from_slice(&cstr(marker_name));
    out.extend_from_slice(&i32_bytes(42));
    append_vec3(&mut out, [10.0, 11.0, 12.0]);
    out.extend_from_slice(&f32_bytes(0.02));
    out.extend_from_slice(&7u16.to_le_bytes());
    out
}

fn known_bodies(rigid_marker_name: Option<&str>) -> Vec<(i32, Vec<u8>)> {
    let rigid = {
        let mut body = Vec::new();
        append_rigid_body(&mut body, "body", rigid_marker_name);
        body
    };
    let skeleton = {
        let mut body = cstr("skeleton");
        body.extend_from_slice(&i32_bytes(77));
        body.extend_from_slice(&i32_bytes(1));
        append_rigid_body(&mut body, "bone", rigid_marker_name);
        body
    };
    let force_plate = type_three_body();
    let device = type_four_body();
    let camera = type_five_body();
    let asset = type_six_body("marker", rigid_marker_name);
    let marker_set = {
        let mut body = cstr("set");
        body.extend_from_slice(&i32_bytes(1));
        body.extend_from_slice(&cstr("m0"));
        body
    };
    vec![
        (0, marker_set),
        (TYPE_RIGID_BODY, rigid),
        (TYPE_SKELETON, skeleton),
        (TYPE_FORCE_PLATE, force_plate),
        (TYPE_DEVICE, device),
        (TYPE_CAMERA, camera),
        (TYPE_ASSET, asset),
    ]
}

/// The typed values the literal bodies of `known_bodies` describe, for
/// dataset types 2 to 6, written independently of the wire helpers above.
fn known_typed(rigid_marker_name: Option<&str>) -> Vec<DataSet> {
    let bone = |name: &str| {
        let mut tail = 1i32.to_le_bytes().to_vec();
        for value in [4.0f32, 5.0, 6.0] {
            tail.extend_from_slice(&value.to_le_bytes());
        }
        tail.extend_from_slice(&0x0102_0304i32.to_le_bytes());
        if let Some(marker_name) = rigid_marker_name {
            tail.extend_from_slice(marker_name.as_bytes());
            tail.push(0);
        }
        RigidBodyDef {
            name: name.into(),
            id: 17,
            parent_id: -1,
            offset: [1.0, 2.0, 3.0],
            tail,
        }
    };
    let mut calibration_matrix = [[0.0f32; 12]; 12];
    for (index, value) in calibration_matrix.iter_mut().flatten().enumerate() {
        *value = index as f32;
    }
    let mut corners = [[0.0f32; 3]; 4];
    for (index, value) in corners.iter_mut().flatten().enumerate() {
        *value = (200 + index) as f32;
    }
    vec![
        DataSet::Skeleton(SkeletonDef {
            name: "skeleton".into(),
            id: 77,
            rigid_bodies: vec![bone("bone")],
        }),
        DataSet::ForcePlate(Box::new(ForcePlateDef {
            id: 3,
            serial_number: "FP-3".into(),
            width: 0.4,
            length: 0.5,
            origin: [1.0, 2.0, 3.0],
            calibration_matrix,
            corners,
            plate_type: 4,
            channel_data_type: 5,
            channel_names: vec!["Fx".into(), "Fy".into()],
        })),
        DataSet::Device(DeviceDef {
            id: 4,
            name: "device".into(),
            serial_number: "serial".into(),
            device_type: 7,
            channel_data_type: 8,
            channel_names: vec!["ch0".into(), "ch1".into()],
        }),
        DataSet::Camera(CameraDef {
            name: "camera".into(),
            position: [7.0, 8.0, 9.0],
            orientation: [0.0, 0.0, 0.0, 1.0],
        }),
        DataSet::Asset(AssetDef {
            name: "asset".into(),
            asset_type: 12,
            id: 99,
            rigid_bodies: vec![bone("bone")],
            markers: vec![MarkerDef {
                name: "marker".into(),
                id: 42,
                position: [10.0, 11.0, 12.0],
                size: 0.02,
                params: 7,
            }],
        }),
    ]
}

fn legacy_payload(datasets: &[(i32, Vec<u8>)]) -> Vec<u8> {
    let mut out = i32_bytes(datasets.len() as i32);
    for (type_id, body) in datasets {
        out.extend_from_slice(&i32_bytes(*type_id));
        out.extend_from_slice(body);
    }
    out
}

fn sized_payload(datasets: &[(i32, Vec<u8>)]) -> Vec<u8> {
    let mut out = i32_bytes(datasets.len() as i32);
    for (type_id, body) in datasets {
        out.extend_from_slice(&i32_bytes(*type_id));
        out.extend_from_slice(&i32_bytes(body.len() as i32));
        out.extend_from_slice(body);
    }
    out
}

#[test]
fn literal_legacy_marker_set_fixture_uses_count_type_body_envelope() {
    // 4-byte dataset count, type 0, "x\0", and a zero marker count.
    let payload = [1, 0, 0, 0, 0, 0, 0, 0, b'x', 0, 0, 0, 0, 0];
    let parsed = ModelDef::parse_payload_version(&payload, Version::new(3, 0, 0, 0))
        .expect("legacy marker-set payload");
    assert_eq!(
        parsed.datasets,
        vec![DataSet::MarkerSet(MarkerSetDef {
            name: "x".into(),
            marker_names: vec![],
        })]
    );
}

#[test]
fn hostile_dataset_count_is_implausible_before_a_truncated_entry() {
    for version in [Version::new(3, 0, 0, 0), Version::V4_1] {
        let hostile = i32_bytes(1_000);
        assert_eq!(
            ModelDef::parse_payload_with_policy(&hostile, version, ParsePolicy::Strict),
            Err(Error::ImplausibleCount {
                count: 1_000,
                available: 0,
            })
        );

        let truncated = if version == Version::V4_1 {
            let mut payload = i32_bytes(1);
            payload.extend_from_slice(&i32_bytes(TYPE_RIGID_BODY));
            payload.extend_from_slice(&i32_bytes(2));
            payload.extend_from_slice(b"x\0");
            payload
        } else {
            let mut payload = i32_bytes(1);
            payload.extend_from_slice(&i32_bytes(TYPE_RIGID_BODY));
            payload.extend_from_slice(b"x\0");
            payload
        };
        assert!(matches!(
            ModelDef::parse_payload_with_policy(&truncated, version, ParsePolicy::Strict),
            Err(Error::UnexpectedEof { .. })
        ));
    }
}

#[test]
fn legacy_nonempty_known_types_are_typed_and_bytes_preserved() {
    for version in [
        Version::new(3, 0, 0, 0),
        Version::new(3, 1, 0, 0),
        Version::new(4, 0, 0, 0),
    ] {
        let rigid_marker_name = (version.major == 4).then_some("marker");
        let bodies = known_bodies(rigid_marker_name);
        let payload = legacy_payload(&bodies);
        let parsed = ModelDef::parse_payload_version(&payload, version).expect("legacy datasets");
        assert_eq!(parsed.datasets.len(), 7);
        assert!(matches!(parsed.datasets[0], DataSet::MarkerSet(_)));
        assert!(matches!(parsed.datasets[1], DataSet::RigidBody(_)));
        assert_eq!(parsed.count_of_type(TYPE_SKELETON), 1);
        assert_eq!(parsed.count_of_type(TYPE_FORCE_PLATE), 1);
        assert_eq!(parsed.count_of_type(TYPE_DEVICE), 1);
        assert_eq!(parsed.count_of_type(TYPE_CAMERA), 1);
        assert_eq!(parsed.count_of_type(TYPE_ASSET), 1);

        assert_eq!(parsed.datasets[2..], known_typed(rigid_marker_name)[..]);
        for policy in [ParsePolicy::Strict, ParsePolicy::Compatible] {
            assert_eq!(
                ModelDef::parse_payload_with_policy(&payload, version, policy),
                Ok(parsed.clone()),
                "{policy:?}"
            );
        }

        let reencoded = parsed
            .to_payload_version(version)
            .expect("checked legacy encoding");
        assert_eq!(reencoded, payload);
    }
}

#[test]
fn v4_1_sized_nonempty_known_bodies_use_same_independent_fixture() {
    let bodies = known_bodies(Some("marker"));
    let payload = sized_payload(&bodies);
    let parsed = ModelDef::parse_payload_with_policy(&payload, Version::V4_1, ParsePolicy::Strict)
        .expect("4.1 datasets");
    assert_eq!(parsed.datasets.len(), 7);
    assert!(matches!(parsed.datasets[0], DataSet::MarkerSet(_)));
    assert!(matches!(parsed.datasets[1], DataSet::RigidBody(_)));
    assert_eq!(parsed.datasets[2..], known_typed(Some("marker"))[..]);
    assert_eq!(
        ModelDef::parse_payload_with_policy(&payload, Version::V4_1, ParsePolicy::Compatible),
        Ok(parsed.clone())
    );
    assert_eq!(parsed.to_payload_version(Version::V4_1), Ok(payload));
}

#[test]
fn v4_0_rigid_body_names_are_included_without_dataset_sizes() {
    let mut body = Vec::new();
    append_rigid_body(&mut body, "body", Some("marker"));
    let payload = legacy_payload(&[(TYPE_RIGID_BODY, body.clone())]);
    let parsed = ModelDef::parse_payload_version(&payload, Version::new(4, 0, 0, 0))
        .expect("4.0 rigid body");
    let DataSet::RigidBody(rigid_body) = &parsed.datasets[0] else {
        panic!("type 1 should be typed");
    };
    let tail_start = cstr("body").len() + 4 + 4 + 12;
    assert_eq!(rigid_body.tail, body[tail_start..]);
    assert_eq!(
        parsed
            .to_payload_version(Version::new(4, 0, 0, 0))
            .expect("4.0 checked encoding"),
        payload
    );
}

#[test]
fn v4_1_sizes_bound_unknown_and_known_opaque_bodies() {
    let unknown = vec![0xde, 0xad, 0xbe, 0xef];
    let camera = vec![1, 2, 3, 4, 5];
    let payload = sized_payload(&[(TYPE_CAMERA, camera.clone()), (99, unknown.clone())]);
    let parsed = ModelDef::parse_payload_version(&payload, Version::V4_1).expect("4.1 payload");
    assert_eq!(
        parsed.datasets,
        vec![
            DataSet::Other {
                type_id: TYPE_CAMERA,
                body: camera,
            },
            DataSet::Other {
                type_id: 99,
                body: unknown,
            },
        ]
    );
    assert_eq!(
        parsed
            .to_payload_version(Version::V4_1)
            .expect("4.1 encode"),
        payload
    );
}

#[test]
fn compatible_rigid_body_parse_keeps_opaque_tail_but_checked_encode_rejects_it() {
    let mut body = cstr("body");
    body.extend_from_slice(&i32_bytes(1));
    body.extend_from_slice(&i32_bytes(-1));
    append_vec3(&mut body, [0.0, 0.0, 0.0]);
    body.extend_from_slice(&[7, 8, 9]);
    let payload = sized_payload(&[(TYPE_RIGID_BODY, body.clone())]);
    assert!(ModelDef::parse_payload_version(&payload, Version::V4_1).is_ok());
    let parsed =
        ModelDef::parse_payload_with_policy(&payload, Version::V4_1, ParsePolicy::Compatible)
            .expect("compatible opaque tail");
    let DataSet::RigidBody(rigid_body) = &parsed.datasets[0] else {
        panic!("type 1 should be typed");
    };
    assert_eq!(&rigid_body.tail[0..], &[7, 8, 9]);
    assert!(matches!(
        ModelDef::parse_payload_with_policy(&payload, Version::V4_1, ParsePolicy::Strict),
        Err(Error::UnexpectedEof { .. }
            | Error::InvalidCount { .. }
            | Error::ImplausibleCount { .. }
            | Error::TrailingBytes { .. },)
    ));
    assert!(parsed.to_payload().is_err());
    assert_eq!(parsed.to_payload_lossy(), payload);
}

#[test]
fn strict_rigid_body_prefix_validation_keeps_opaque_suffix() {
    let mut body = cstr("body");
    body.extend_from_slice(&i32_bytes(1));
    body.extend_from_slice(&i32_bytes(-1));
    append_vec3(&mut body, [0.0, 0.0, 0.0]);
    body.extend_from_slice(&i32_bytes(0));
    body.extend_from_slice(&[7, 8, 9]);
    let payload = sized_payload(&[(TYPE_RIGID_BODY, body)]);
    let parsed = ModelDef::parse_payload_with_policy(&payload, Version::V4_1, ParsePolicy::Strict)
        .expect("valid prefix plus opaque suffix");
    assert_eq!(parsed.to_payload().expect("checked encoding"), payload);
}

#[test]
fn legacy_checked_encode_rejects_rigid_tail_suffix_but_v4_1_preserves_it() {
    let mut body = cstr("body");
    body.extend_from_slice(&i32_bytes(1));
    body.extend_from_slice(&i32_bytes(-1));
    append_vec3(&mut body, [0.0, 0.0, 0.0]);
    body.extend_from_slice(&i32_bytes(0));
    body.push(42);
    let payload = sized_payload(&[(TYPE_RIGID_BODY, body)]);
    let parsed = ModelDef::parse_payload_with_policy(&payload, Version::V4_1, ParsePolicy::Strict)
        .expect("4.1 opaque suffix");

    assert_eq!(parsed.to_payload_version(Version::V4_1), Ok(payload));
    assert!(matches!(
        parsed.to_payload_version(Version::new(4, 0, 0, 0)),
        Err(Error::TrailingBytes { remaining: 1 })
    ));
}

#[test]
fn strict_policy_rejects_marker_residue_and_negative_dataset_count() {
    let mut marker_body = cstr("set");
    marker_body.extend_from_slice(&i32_bytes(0));
    marker_body.push(0xaa);
    let payload = sized_payload(&[(0, marker_body)]);
    let parsed = ModelDef::parse_payload_version(&payload, Version::V4_1)
        .expect("compatible marker residue");
    assert_eq!(
        parsed.datasets,
        vec![DataSet::MarkerSet(MarkerSetDef {
            name: "set".into(),
            marker_names: vec![],
        })]
    );
    let canonical = sized_payload(&[(0, {
        let mut body = cstr("set");
        body.extend_from_slice(&i32_bytes(0));
        body
    })]);
    assert_eq!(parsed.to_payload().expect("canonical encoding"), canonical);
    assert!(matches!(
        ModelDef::parse_payload_with_policy(&payload, Version::V4_1, ParsePolicy::Strict),
        Err(Error::TrailingBytes { remaining: 1 })
    ));

    let mut top_level_residue = canonical.clone();
    top_level_residue.push(0xbb);
    assert_eq!(
        ModelDef::parse_payload_version(&top_level_residue, Version::V4_1)
            .expect("compatible top-level residue"),
        parsed
    );
    assert!(matches!(
        ModelDef::parse_payload_with_policy(&top_level_residue, Version::V4_1, ParsePolicy::Strict),
        Err(Error::TrailingBytes { remaining: 1 })
    ));

    let negative = (-1i32).to_le_bytes();
    assert!(matches!(
        ModelDef::parse_payload_with_policy(
            &negative,
            Version::new(3, 0, 0, 0),
            ParsePolicy::Strict
        ),
        Err(Error::InvalidCount {
            count: -1,
            offset: 0
        })
    ));
}

#[test]
fn bounded_nested_errors_report_absolute_payload_offsets() {
    let truncated_body = cstr("x");
    let truncated = sized_payload(&[(TYPE_RIGID_BODY, truncated_body)]);
    assert_eq!(
        ModelDef::parse_payload_with_policy(&truncated, Version::V4_1, ParsePolicy::Strict),
        Err(Error::UnexpectedEof {
            offset: 14,
            needed: 4,
            available: 0,
        })
    );

    let mut negative_body = cstr("set");
    negative_body.extend_from_slice(&i32_bytes(-1));
    let negative = sized_payload(&[(0, negative_body)]);
    assert_eq!(
        ModelDef::parse_payload_with_policy(&negative, Version::V4_1, ParsePolicy::Strict),
        Err(Error::InvalidCount {
            count: -1,
            offset: 16,
        })
    );
}

#[test]
fn checked_aliases_and_unknown_legacy_types_are_rejected() {
    let alias = ModelDef {
        datasets: vec![DataSet::Other {
            type_id: TYPE_RIGID_BODY,
            body: vec![],
        }],
    };
    assert!(matches!(
        alias.to_payload().unwrap_err(),
        Error::InvalidValue { .. }
    ));

    let payload = legacy_payload(&[(123, vec![1, 2, 3])]);
    assert!(matches!(
        ModelDef::parse_payload_version(&payload, Version::new(3, 1, 0, 0)),
        Err(Error::UnsupportedDataSet { type_id: 123, .. })
    ));
}

#[test]
fn unsupported_minor_is_rejected_without_payload_guessing() {
    let payload = i32_bytes(0);
    assert!(matches!(
        ModelDef::parse_payload_version(&payload, Version::new(4, 2, 0, 0)),
        Err(Error::UnsupportedVersion { major: 4, minor: 2 })
    ));
    assert!(matches!(
        ModelDef { datasets: vec![] }.to_payload_version(Version::new(3, 2, 0, 0)),
        Err(Error::UnsupportedVersion { major: 3, minor: 2 })
    ));
}

#[test]
fn checked_modeldef_datagram_refuses_a_payload_its_header_cannot_declare() {
    let model = ModelDef {
        datasets: vec![DataSet::MarkerSet(MarkerSetDef {
            name: "big".into(),
            marker_names: vec!["m".repeat(99); 700],
        })],
    };
    let payload = model.to_payload().unwrap();
    assert!(payload.len() > usize::from(u16::MAX));
    let overflow = Err(Error::LengthOverflow {
        field: "message payload size",
        length: payload.len(),
        max: 65_535,
    });
    assert_eq!(model.to_datagram().map(|d| d.len()), overflow);
    assert_eq!(
        model.to_datagram_version(Version::V4_1).map(|d| d.len()),
        overflow
    );
    let lossy = model.to_datagram_lossy();
    assert_eq!(&lossy[..4], &[5, 0, 0xff, 0xff]);
    assert_eq!(&lossy[4..], payload.as_slice());
}
