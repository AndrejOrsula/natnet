//! Round-trip tests over synthetic (spec-derived) values, plus the SERVERINFO
//! NUL-stop behaviour and version branching.
//!
//! The rule under test is the **qualified** one the crate documents —
//! `parse(serialize(x)) == x` for an `x` one of the three NatNet message
//! parsers produced and a serializer accepts, modulo IEEE-754 float equality
//! — not an unconditional round trip, and not a rule over every parser this
//! crate exposes.
//! NaN behavior and explicitly lossy compatibility encoders each get a test below:
//! what still holds and what stops holding. A wire NaN reproduces the payload
//! bytes exactly *and* loses value equality; asserting either half alone
//! passes vacuously. See the crate-level docs and `natnet::frame`'s
//! *Round-trip, and the NaN a frame can carry*.
//!
//! These fixtures are fabricated: the repository carries only synthetic,
//! spec-derived fixtures (see CONTRIBUTING.md).

use natnet::frame::{FrameTiming, LabeledMarker, MarkerSet, RawSection, RigidBody};
use natnet::generate::{PlannedBody, SyntheticServer};
use natnet::modeldef::{DataSet, MarkerSetDef, RigidBodyDef};
use natnet::serverinfo::ConnectionInfo;
use natnet::{FrameOfData, ModelDef, ServerInfo, Version};

/// A quiet NaN bit pattern, as a peer's bytes would carry it.
const NAN_F64_BITS: u64 = 0x7ff8_0000_0000_0001;
const NAN_F32_BITS: u32 = 0x7fc0_0001;

fn sample_frame() -> FrameOfData {
    FrameOfData {
        frame_number: 15_732_032,
        marker_sets: vec![
            MarkerSet {
                name: "CW-500 wand".to_string(),
                markers: vec![[0.9, 0.7, -0.9], [0.96, 0.73, -0.9], [0.94, 0.74, -0.6]],
            },
            MarkerSet {
                name: "all".to_string(),
                markers: vec![[0.9, 0.7, -0.9]],
            },
        ],
        other_markers: vec![[1.0, 2.0, 3.0]],
        rigid_bodies: vec![RigidBody {
            id: 3,
            position: [0.945, 0.756, -0.911],
            orientation: [0.261, 0.730, 0.184, -0.605],
            mean_marker_error: 0.0,
            params: 0, // tracking-invalid, like the real wand
        }],
        skeletons: RawSection {
            count: 0,
            payload: Vec::new(),
        },
        assets: RawSection {
            count: 0,
            payload: Vec::new(),
        },
        labeled_markers: vec![LabeledMarker {
            id: 42,
            position: [1.0, 2.0, 3.0],
            size: 0.01,
            params: 1,
            residual: 0.0005,
        }],
        force_plates: RawSection {
            count: 0,
            payload: Vec::new(),
        },
        devices: RawSection {
            count: 0,
            payload: Vec::new(),
        },
        timing: FrameTiming {
            timecode: 0,
            timecode_subframe: 0,
            timestamp: 131_100.266_7,
            camera_mid_exposure: 0x07ca_b13f_00e3_4100,
            camera_data_received: 0x07ca_b13f_e84b_0000,
            transmit: 0x07ca_b140_0e0f_0000,
            precision_timestamp_secs: 0,
            precision_timestamp_fractional_secs: 0,
        },
        params: 0,
    }
}

#[test]
fn frame_round_trips_at_v4_1() {
    let f = sample_frame();
    let bytes = f.to_datagram(Version::V4_1).expect("fixture encodes");
    let back = FrameOfData::parse(&bytes, Version::V4_1).expect("parse");
    assert_eq!(f, back);
    assert!(!back.rigid_bodies[0].tracking_valid());
}

#[test]
fn frame_payload_round_trips() {
    let f = sample_frame();
    let payload = f.to_payload(Version::V4_1).expect("fixture encodes");
    let back = FrameOfData::parse_payload(&payload, Version::V4_1).expect("parse");
    assert_eq!(f, back);
}

#[test]
fn version_branching_changes_the_wire() {
    // v3.0 omits section sizes, the 4.1 asset section and precision timestamp:
    // its serialization is strictly shorter, and round-trips at its own
    // version.
    let mut f = sample_frame();
    // assets absent for a v3.0 frame.
    f.assets = RawSection {
        count: 0,
        payload: Vec::new(),
    };
    let v30 = Version::new(3, 0, 0, 0);
    let p30 = f.to_payload(v30).expect("fixture encodes");
    let p41 = f.to_payload(Version::V4_1).expect("fixture encodes");
    // v4.1 adds seven section size words (28 B), the asset header (8 B),
    // and the precision timestamp (8 B).
    assert_eq!(p41.len(), p30.len() + 44);
    let back = FrameOfData::parse_payload(&p30, v30).expect("parse v3.0");
    assert_eq!(f, back);
}

#[test]
fn unsupported_old_version_is_err_not_panic() {
    let f = sample_frame();
    let bytes = f.to_datagram(Version::V4_1).expect("fixture encodes");
    let err = FrameOfData::parse(&bytes, Version::new(2, 9, 0, 0)).unwrap_err();
    assert!(matches!(
        err,
        natnet::Error::UnsupportedVersion { major: 2, .. }
    ));
}

fn sample_serverinfo() -> ServerInfo {
    ServerInfo {
        application_name: "Motive".to_string(),
        server_version: Version::new(3, 1, 2, 1),
        protocol_version: Version::V4_1,
        high_res_clock_frequency: 10_000_000,
        connection: ConnectionInfo {
            data_port: 1511,
            multicast: true,
            multicast_group: [239, 255, 42, 99],
        },
    }
}

#[test]
fn serverinfo_round_trips() {
    let s = sample_serverinfo();
    let back = ServerInfo::parse(&s.to_datagram().expect("fixture encodes")).expect("parse");
    assert_eq!(s, back);
    assert_eq!(back.multicast_group().to_string(), "239.255.42.99");
    assert_eq!(back.protocol_version, Version::V4_1);
}

#[test]
fn serverinfo_name_field_stops_at_first_nul() {
    // A 256-byte name field carrying `Motive\0` then stale server-buffer
    // bytes (an address-like fragment), exactly the real gotcha. Decoding
    // must stop at the first NUL.
    let mut payload = Vec::new();
    let mut name = vec![0u8; 256];
    name[..6].copy_from_slice(b"Motive");
    name[6] = 0;
    name[7..17].copy_from_slice(b".195:30149"); // stale, must be ignored
    payload.extend_from_slice(&name);
    payload.extend_from_slice(&[3, 1, 2, 1]); // server version
    payload.extend_from_slice(&[4, 1, 0, 0]); // bitstream
    payload.extend_from_slice(&10_000_000u64.to_le_bytes());
    payload.extend_from_slice(&1511u16.to_le_bytes());
    payload.push(1);
    payload.extend_from_slice(&[239, 255, 42, 99]);

    let s = ServerInfo::parse_payload(&payload).expect("parse");
    assert_eq!(s.application_name, "Motive");
    assert_eq!(s.server_version, Version::new(3, 1, 2, 1));
    assert_eq!(s.high_res_clock_frequency, 10_000_000);
}

fn sample_modeldef() -> ModelDef {
    ModelDef {
        datasets: vec![
            DataSet::RigidBody(RigidBodyDef {
                name: "CW-500 wand".to_string(),
                id: 3,
                parent_id: -1,
                offset: [0.0, 0.0, 0.0],
                tail: vec![0, 0, 0, 0], // zero marker descriptions
            }),
            DataSet::MarkerSet(MarkerSetDef {
                name: "CW-500 wand".to_string(),
                marker_names: vec![
                    "Marker 001".to_string(),
                    "Marker 002".to_string(),
                    "Marker 003".to_string(),
                ],
            }),
            // A camera dataset: "Prime\0", then a zero position and orientation.
            DataSet::Camera(natnet::CameraDef {
                name: "Prime".to_string(),
                position: [0.0; 3],
                orientation: [0.0; 4],
            }),
        ],
    }
}

#[test]
fn modeldef_round_trips_and_resolves_names() {
    let m = sample_modeldef();
    let back = ModelDef::parse(&m.to_datagram().expect("fixture encodes")).expect("parse");
    assert_eq!(m, back);
    assert_eq!(back.resolve_rigid_body("CW-500 wand"), Some(3));
    assert_eq!(back.resolve_rigid_body("nonexistent"), None);
    assert_eq!(back.count_of_type(natnet::modeldef::TYPE_CAMERA), 1);
    assert_eq!(back.count_of_type(natnet::modeldef::TYPE_MARKER_SET), 1);
}

// ---------------------------------------------------------------------------
// NaN and lossy compatibility behavior, and the neighbours
// that are *not* exceptions. Every test below asserts a pair: what still
// holds beside what stops holding, because either half alone passes vacuously
// — a byte-equality assertion passes on a frame with no NaN in it, and an
// inequality assertion passes on a frame that failed to parse.
// ---------------------------------------------------------------------------

/// The byte offset of the one occurrence of `pat`, asserting it is the only
/// one — a patch aimed at a pattern that occurs twice would measure the wrong
/// slot.
fn only_occurrence(payload: &[u8], pat: &[u8]) -> usize {
    let hits: Vec<usize> = payload
        .windows(pat.len())
        .enumerate()
        .filter(|(_, w)| *w == pat)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "pattern must occur exactly once, got {hits:?}"
    );
    hits[0]
}

#[test]
fn a_wire_nan_reproduces_the_payload_bytes_and_costs_value_equality() {
    let v = Version::V4_1;
    let clean = sample_frame();
    let wire = clean.to_payload(v).expect("fixture encodes");

    // Control first, so neither half of the pair below can pass vacuously:
    // with no NaN on the wire, byte equality *and* value equality both hold.
    let c = FrameOfData::parse_payload(&wire, v).expect("the clean frame parses");
    assert_eq!(
        c.to_payload(v).expect("fixture encodes"),
        wire,
        "control: the bytes reproduce"
    );
    assert_eq!(
        FrameOfData::parse_payload(&c.to_payload(v).expect("fixture encodes"), v)
            .expect("re-parse"),
        c,
        "control: the value round-trips"
    );

    // Now a NaN off the wire, in `timing.timestamp` — the one float slot
    // every frame has, whatever its section counts.
    let at = only_occurrence(&wire, &clean.timing.timestamp.to_le_bytes());
    let mut nan_wire = wire.clone();
    nan_wire[at..at + 8].copy_from_slice(&NAN_F64_BITS.to_le_bytes());
    let x = FrameOfData::parse_payload(&nan_wire, v).expect("a NaN frame still parses");
    assert!(x.timing.timestamp.is_nan(), "the slot really holds a NaN");

    // Half one — the bytes reproduce exactly, and it is the NaN's *own* bit
    // pattern that survives, not merely some NaN.
    let re = x.to_payload(v).expect("fixture encodes");
    assert_eq!(re, nan_wire, "a NaN never costs byte equality");
    assert_eq!(x.timing.timestamp.to_bits(), NAN_F64_BITS);

    // Half two — and the value does not come back equal, because it is not
    // equal even to a copy of itself.
    let back = FrameOfData::parse_payload(&re, v).expect("re-parse");
    assert_ne!(back, x, "parse(serialize(x)) != x");
    assert_ne!(x, x.clone(), "and x != x");
}

#[test]
fn a_wire_nan_in_an_f32_slot_behaves_exactly_as_the_f64_one_does() {
    let v = Version::V4_1;
    let clean = sample_frame();
    let wire = clean.to_payload(v).expect("fixture encodes");
    let at = only_occurrence(&wire, &clean.rigid_bodies[0].position[0].to_le_bytes());

    // Control: unpatched, that slot round-trips by value.
    let c = FrameOfData::parse_payload(&wire, v).expect("parse");
    assert_eq!(
        c.rigid_bodies[0].position[0],
        clean.rigid_bodies[0].position[0]
    );

    let mut nan_wire = wire.clone();
    nan_wire[at..at + 4].copy_from_slice(&NAN_F32_BITS.to_le_bytes());
    let x = FrameOfData::parse_payload(&nan_wire, v).expect("parse");
    assert!(x.rigid_bodies[0].position[0].is_nan());
    assert_eq!(x.rigid_bodies[0].position[0].to_bits(), NAN_F32_BITS);
    assert_eq!(
        x.to_payload(v).expect("fixture encodes"),
        nan_wire,
        "the bytes reproduce"
    );
    assert_ne!(x, x.clone(), "the value does not");
}

#[test]
fn a_frame_with_every_section_empty_still_has_a_nan_reachable_slot() {
    let v = Version::V4_1;
    let mut empty = sample_frame();
    empty.marker_sets.clear();
    empty.other_markers.clear();
    empty.rigid_bodies.clear();
    empty.labeled_markers.clear();
    let wire = empty.to_payload(v).expect("fixture encodes");

    // Control: the degenerate frame round-trips both ways untouched.
    let c = FrameOfData::parse_payload(&wire, v).expect("parse");
    assert_eq!(c.to_payload(v).expect("fixture encodes"), wire);
    assert_eq!(c, c.clone());

    let at = only_occurrence(&wire, &empty.timing.timestamp.to_le_bytes());
    let mut nan_wire = wire.clone();
    nan_wire[at..at + 8].copy_from_slice(&NAN_F64_BITS.to_le_bytes());
    let x = FrameOfData::parse_payload(&nan_wire, v).expect("parse");
    assert_eq!(
        x.to_payload(v).expect("fixture encodes"),
        nan_wire,
        "the bytes reproduce"
    );
    assert_ne!(x, x.clone(), "the value does not — with no section at all");
}

#[test]
fn a_planned_nan_makes_a_generated_frame_unequal_to_itself() {
    let v = Version::V4_1;
    let s = SyntheticServer::new(v, 1_000_000);
    let good = [PlannedBody::tracked(
        3,
        [1.0, 2.0, 3.0],
        [0.0, 0.0, 0.0, 1.0],
    )];
    let bad = [PlannedBody::tracked(
        3,
        [f32::NAN, 2.0, 3.0],
        [0.0, 0.0, 0.0, 1.0],
    )];

    // Control: `generate` filters no float, and without a NaN in the plan the
    // generated datagram round-trips.
    let dg = s.datagram(42, 0.35, &good).expect("fixture encodes");
    assert_eq!(
        FrameOfData::parse(&dg, v).expect("parse"),
        s.frame(42, 0.35, &good)
    );

    let dg = s.datagram(42, 0.35, &bad).expect("fixture encodes");
    let x = FrameOfData::parse(&dg, v).expect("parse");
    assert_eq!(
        x.to_datagram(v).expect("fixture encodes"),
        dg,
        "the generated bytes reproduce"
    );
    assert_ne!(x, s.frame(42, 0.35, &bad), "the planned value does not");
    assert_ne!(x, x.clone(), "because it is not equal even to itself");
}

#[test]
fn an_interior_nul_in_a_marker_set_name_costs_the_round_trip_either_way() {
    let v = Version::V4_1;

    // Case one, and the common one: the marker count is read from the wrong
    // four bytes, is implausible, and the payload is refused outright.
    let mut f = sample_frame();
    f.marker_sets[0].name = "a\0b".to_string();
    assert!(matches!(
        f.to_payload(v),
        Err(natnet::Error::InteriorNul { .. })
    ));
    let wire = f.to_payload_lossy(v).expect("supported fixture layout");
    assert!(
        !wire.is_empty(),
        "explicit lossy encoding reproduces the old defect"
    );
    assert!(
        FrameOfData::parse_payload(&wire, v).is_err(),
        "an interior NUL desynchronises the payload"
    );

    // Case two — and this is why the module docs do *not* say an interior NUL
    // always fails to parse. Choose the name so the shifted count and the
    // section's declared size agree: the entry writes (len+1) + 4 + 12*m
    // bytes and the desynchronised read consumes 2 + 4 + 12*c, so len = 13,
    // m = 0, c = 1 balances. It parses `Ok` — into a *different* frame.
    let mut name = vec![b'A', 0];
    name.extend_from_slice(&1i32.to_le_bytes());
    name.extend_from_slice(b"abcdefg");
    assert_eq!(name.len(), 13);
    let mut h = sample_frame();
    h.marker_sets[0].name = String::from_utf8(name).expect("valid UTF-8");
    h.marker_sets[0].markers = Vec::new();
    assert!(h.to_payload(v).is_err());
    let wire = h.to_payload_lossy(v).expect("supported fixture layout");
    let back = FrameOfData::parse_payload(&wire, v).expect("this one does parse");
    assert_ne!(back, h, "and it is not the frame that was serialized");
    assert_eq!(back.marker_sets[0].name, "A", "the name stops at the NUL");
    assert_eq!(
        back.marker_sets[0].markers.len(),
        1,
        "with a marker invented"
    );
    assert!(
        !back.timing.timestamp.is_nan(),
        "no NaN anywhere: this is the string exception, not the float one"
    );

    // Reach control: the same shape with the NUL removed parses and is equal,
    // so the failure tracks the NUL and not the fixture.
    let mut g = sample_frame();
    g.marker_sets[0].name = "ab".to_string();
    let w = g.to_payload(v).expect("fixture encodes");
    assert_eq!(FrameOfData::parse_payload(&w, v).expect("parse"), g);
}

#[test]
fn an_interior_nul_in_a_modeldef_name_breaks_the_round_trip_both_ways() {
    let mut m = sample_modeldef();
    let DataSet::RigidBody(rb) = &mut m.datasets[0] else {
        panic!("dataset 0 of the fixture is the rigid body");
    };
    rb.name = "r\0b".to_string();

    assert!(m.to_datagram().is_err());
    let back = ModelDef::parse(&m.to_datagram_lossy()).expect("it serializes and parses Ok");
    assert_ne!(back, m, "the same bytes, an unequal value, and no NaN");
    let DataSet::RigidBody(rb) = &back.datasets[0] else {
        panic!("it comes back a rigid body, just a different one");
    };
    assert_eq!(rb.name, "r", "the name stops at the NUL");
    assert!(
        rb.offset.iter().all(|f| !f.is_nan()),
        "no NaN anywhere: this is the string exception, not the float one"
    );

    // The same NUL in a marker-set name shifts the marker *count* instead,
    // and that is where the module docs need both halves. Usually the count
    // read from the wrong four bytes is implausible and the reply is refused…
    let mut ms = sample_modeldef();
    let DataSet::MarkerSet(msd) = &mut ms.datasets[1] else {
        panic!("dataset 1 of the fixture is the marker set");
    };
    msd.name = "r\0b".to_string();
    assert!(ms.to_datagram().is_err());
    assert!(
        ModelDef::parse(&ms.to_datagram_lossy()).is_err(),
        "the shifted count is implausible, so this one is refused"
    );

    // …but "refused outright" is not what a MarkerSetDef does in general, and
    // this is the case that proves it. Nothing has to balance here — there is
    // no consumed-equals-size check in `parse_dataset` — so it is enough that
    // the four bytes landing in the count slot read as a plausible count with
    // that many NUL-terminated strings behind them. Spell the count into the
    // name itself: "A", NUL, i32(1), "m", NUL.
    let mut name = vec![b'A', 0];
    name.extend_from_slice(&1i32.to_le_bytes());
    name.extend_from_slice(b"m\0");
    assert_eq!(name.len(), 8);
    let mut ok = sample_modeldef();
    ok.datasets[1] = DataSet::MarkerSet(MarkerSetDef {
        name: String::from_utf8(name).expect("valid UTF-8"),
        marker_names: Vec::new(),
    });
    assert!(ok.to_datagram().is_err());
    let back = ModelDef::parse(&ok.to_datagram_lossy()).expect("this one does parse");
    assert_ne!(back, ok, "and it is not the model definition that went out");
    let DataSet::MarkerSet(msd) = &back.datasets[1] else {
        panic!("it comes back a marker set, just a different one");
    };
    assert_eq!(msd.name, "A", "the name stops at the NUL");
    assert_eq!(
        msd.marker_names,
        vec!["m".to_string()],
        "with a member invented out of the name's own bytes"
    );

    // Reach control: the fixture without the NUL round-trips.
    let clean = sample_modeldef();
    assert_eq!(
        ModelDef::parse(&clean.to_datagram().expect("fixture encodes")).expect("parse"),
        clean
    );
}

#[test]
fn a_dataset_other_carrying_a_decoded_type_id_parses_back_as_that_type() {
    // A body that parses under both decoded shapes: `"x\0"`, then 20 zero
    // bytes (a marker-set count of 0, or an id / parent_id / offset triple).
    let mut body = b"x\0".to_vec();
    body.extend_from_slice(&[0u8; 20]);

    for type_id in [
        natnet::modeldef::TYPE_MARKER_SET,
        natnet::modeldef::TYPE_RIGID_BODY,
    ] {
        let m = ModelDef {
            datasets: vec![DataSet::Other {
                type_id,
                body: body.clone(),
            }],
        };
        assert!(m.to_datagram().is_err());
        let back = ModelDef::parse(&m.to_datagram_lossy()).expect("parse");
        assert_ne!(back, m, "type id {type_id} aliases a decoded dataset");
        assert!(!matches!(back.datasets[0], DataSet::Other { .. }));
    }

    // Reach control: every other type id keeps its `Other` shape and is equal.
    for type_id in [2, natnet::modeldef::TYPE_CAMERA] {
        let m = ModelDef {
            datasets: vec![DataSet::Other {
                type_id,
                body: body.clone(),
            }],
        };
        assert_eq!(
            ModelDef::parse(&m.to_datagram().expect("fixture encodes")).expect("parse"),
            m
        );
    }
}

/// A 279-byte SERVERINFO payload whose 256-byte name field starts with
/// `name_bytes` and is zero-filled after it.
fn serverinfo_payload_with_name_bytes(name_bytes: &[u8]) -> Vec<u8> {
    let mut payload = vec![0u8; 256];
    payload[..name_bytes.len()].copy_from_slice(name_bytes);
    payload.extend_from_slice(&[3, 1, 2, 1]);
    payload.extend_from_slice(&[4, 1, 0, 0]);
    payload.extend_from_slice(&10_000_000u64.to_le_bytes());
    payload.extend_from_slice(&1511u16.to_le_bytes());
    payload.push(1);
    payload.extend_from_slice(&[239, 255, 42, 99]);
    payload
}

#[test]
fn a_hand_built_application_name_past_255_bytes_is_truncated_on_the_wire() {
    let mut s = sample_serverinfo();
    s.application_name = "M".repeat(256);
    assert!(s.to_datagram().is_err());
    let back = ServerInfo::parse(&s.to_datagram_lossy()).expect("explicit lossy encoding parses");
    assert_ne!(back, s, "the name field is 256 bytes and one is the NUL");
    assert_eq!(back.application_name.len(), 255);

    // Reach control at the boundary: 255 bytes fit and round-trip.
    let mut t = sample_serverinfo();
    t.application_name = "M".repeat(255);
    assert_eq!(
        ServerInfo::parse(&t.to_datagram().expect("fixture encodes")).expect("parse"),
        t
    );
}

#[test]
fn a_parsed_application_name_can_outgrow_the_field_it_came_from() {
    // 100 wire bytes that are not UTF-8 and no NUL before them: each decodes
    // lossily to U+FFFD, three bytes out for one byte in, so the *parsed*
    // name is 300 bytes — longer than the 255 the serializer writes back.
    let x = ServerInfo::parse_payload(&serverinfo_payload_with_name_bytes(&[0xff; 100]))
        .expect("parse");
    assert_eq!(x.application_name.len(), 300);
    assert!(x.to_payload().is_err());
    let back = ServerInfo::parse_payload(&x.to_payload_lossy()).expect("lossy re-parse");
    assert_ne!(back, x, "the second exception, with no NaN anywhere");
    assert_eq!(back.application_name.len(), 255);

    // Reach control: 85 such bytes decode to exactly 255 and round-trip, so
    // the failure tracks the length and not the lossy decode itself.
    let y =
        ServerInfo::parse_payload(&serverinfo_payload_with_name_bytes(&[0xff; 85])).expect("parse");
    assert_eq!(y.application_name.len(), 255);
    assert_eq!(
        ServerInfo::parse_payload(&y.to_payload().expect("fixture encodes")).expect("re-parse"),
        y
    );
}

#[test]
fn a_lossy_marker_set_name_costs_payload_bytes_and_not_value_equality() {
    let v = Version::V4_1;
    let mut f = sample_frame();
    f.marker_sets[0].name = "zq".to_string();
    let clean = f.to_payload(v).expect("fixture encodes");

    // Control: left UTF-8, the datagram reproduces byte for byte.
    assert_eq!(
        FrameOfData::parse_payload(&clean, v)
            .expect("parse")
            .to_payload(v)
            .expect("fixture encodes"),
        clean
    );

    let at = only_occurrence(&clean, b"zq");
    let mut wire = clean.clone();
    wire[at] = 0xff; // not UTF-8, and not a NUL, so the name still terminates
    let x = FrameOfData::parse_payload(&wire, v).expect("parse");
    assert_eq!(x.marker_sets[0].name, "\u{fffd}q");

    let re = x.to_payload(v).expect("fixture encodes");
    assert_ne!(re, wire, "the datagram does not round-trip...");
    assert_eq!(
        re.len(),
        wire.len() + 2,
        "...it grows by U+FFFD's extra bytes"
    );
    assert_eq!(
        FrameOfData::parse_payload(&re, v).expect("re-parse"),
        x,
        "but the value does — no NaN is involved either way"
    );
}
