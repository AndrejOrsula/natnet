//! Behavioral coverage for the opt-in `serde` and `nalgebra` features.
//!
//! Serde tests exercise the public value types carrying the feature-gated
//! derives. `Message`, `MessageHeader`, command types, and imagery records do
//! not derive serde, so this test target deliberately does not broaden their
//! public contracts.

#[cfg(feature = "nalgebra")]
mod nalgebra_tests {
    use nalgebra::{Point3, Quaternion};
    use natnet::frame::RigidBody;
    use natnet::marker_vector;

    fn body() -> RigidBody {
        RigidBody {
            id: 17,
            position: [11.0, -13.0, 19.0],
            // Deliberately non-unit and distinct. NatNet stores this as
            // [x, y, z, w], while Quaternion::new takes (w, x, y, z).
            orientation: [2.0, 3.0, 5.0, 7.0],
            mean_marker_error: 0.25,
            params: 1,
        }
    }

    #[test]
    fn rigid_body_pose_conversion_preserves_components_and_order() {
        let rigid_body = body();
        let position = Point3::from(&rigid_body);
        let orientation = Quaternion::from(&rigid_body);

        assert_eq!(position, Point3::new(11.0, -13.0, 19.0));
        assert_eq!(orientation.scalar(), 7.0);
        assert_eq!(orientation.vector()[0], 2.0);
        assert_eq!(orientation.vector()[1], 3.0);
        assert_eq!(orientation.vector()[2], 5.0);
        assert_eq!(orientation.as_vector().as_slice(), &[2.0, 3.0, 5.0, 7.0]);
        // The conversion preserves a non-unit quaternion instead of silently
        // normalizing it.
        assert_eq!(orientation.norm_squared(), 87.0);
    }

    #[test]
    fn marker_vector_preserves_all_components() {
        let marker = marker_vector([23.0, -29.0, 31.0]);

        assert_eq!(marker[0], 23.0);
        assert_eq!(marker[1], -29.0);
        assert_eq!(marker[2], 31.0);
    }
}

#[cfg(feature = "serde")]
mod serde_tests {
    use natnet::frame::{ForcePlate, RawSection, RigidBody, Skeleton};
    use natnet::modeldef::{CameraDef, DataSet};
    use serde_test::{Token, assert_tokens};

    #[test]
    fn dataset_camera_roundtrips_as_a_newtype_variant() {
        let value = DataSet::Camera(CameraDef {
            name: "cam".into(),
            position: [1.0, 2.0, 3.0],
            orientation: [0.0, 0.0, 0.0, 1.0],
        });

        assert_tokens(
            &value,
            &[
                Token::NewtypeVariant {
                    name: "DataSet",
                    variant: "Camera",
                },
                Token::Struct {
                    name: "CameraDef",
                    len: 3,
                },
                Token::Str("name"),
                Token::Str("cam"),
                Token::Str("position"),
                Token::Tuple { len: 3 },
                Token::F32(1.0),
                Token::F32(2.0),
                Token::F32(3.0),
                Token::TupleEnd,
                Token::Str("orientation"),
                Token::Tuple { len: 4 },
                Token::F32(0.0),
                Token::F32(0.0),
                Token::F32(0.0),
                Token::F32(1.0),
                Token::TupleEnd,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn frame_skeleton_and_force_plate_roundtrip_typed_entries() {
        let skeleton = Skeleton {
            id: 1,
            rigid_bodies: vec![RigidBody {
                id: 0x0001_0002,
                position: [1.0, 2.0, 3.0],
                orientation: [0.0, 0.0, 0.0, 1.0],
                mean_marker_error: 0.5,
                params: 1,
            }],
        };
        assert_tokens(
            &skeleton,
            &[
                Token::Struct {
                    name: "Skeleton",
                    len: 2,
                },
                Token::Str("id"),
                Token::I32(1),
                Token::Str("rigid_bodies"),
                Token::Seq { len: Some(1) },
                Token::Struct {
                    name: "RigidBody",
                    len: 5,
                },
                Token::Str("id"),
                Token::I32(0x0001_0002),
                Token::Str("position"),
                Token::Tuple { len: 3 },
                Token::F32(1.0),
                Token::F32(2.0),
                Token::F32(3.0),
                Token::TupleEnd,
                Token::Str("orientation"),
                Token::Tuple { len: 4 },
                Token::F32(0.0),
                Token::F32(0.0),
                Token::F32(0.0),
                Token::F32(1.0),
                Token::TupleEnd,
                Token::Str("mean_marker_error"),
                Token::F32(0.5),
                Token::Str("params"),
                Token::U16(1),
                Token::StructEnd,
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );

        let plate = ForcePlate {
            id: 3,
            channels: vec![vec![1.5], vec![]],
        };
        assert_tokens(
            &plate,
            &[
                Token::Struct {
                    name: "ForcePlate",
                    len: 2,
                },
                Token::Str("id"),
                Token::I32(3),
                Token::Str("channels"),
                Token::Seq { len: Some(2) },
                Token::Seq { len: Some(1) },
                Token::F32(1.5),
                Token::SeqEnd,
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn dataset_other_roundtrips_with_enum_and_opaque_bytes_tokens() {
        let value = DataSet::Other {
            type_id: 42,
            body: vec![0x00, 0x7f, 0x80, 0xff],
        };

        assert_tokens(
            &value,
            &[
                Token::StructVariant {
                    name: "DataSet",
                    variant: "Other",
                    len: 2,
                },
                Token::Str("type_id"),
                Token::I32(42),
                Token::Str("body"),
                Token::Seq { len: Some(4) },
                Token::U8(0x00),
                Token::U8(0x7f),
                Token::U8(0x80),
                Token::U8(0xff),
                Token::SeqEnd,
                Token::StructVariantEnd,
            ],
        );
    }

    #[test]
    fn raw_section_roundtrips_opaque_payload_bytes() {
        let value = RawSection {
            count: 3,
            payload: vec![0xde, 0xad, 0x00, 0xbe, 0xef],
        };

        assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "RawSection",
                    len: 2,
                },
                Token::Str("count"),
                Token::I32(3),
                Token::Str("payload"),
                Token::Seq { len: Some(5) },
                Token::U8(0xde),
                Token::U8(0xad),
                Token::U8(0x00),
                Token::U8(0xbe),
                Token::U8(0xef),
                Token::SeqEnd,
                Token::StructEnd,
            ],
        );
    }
}
