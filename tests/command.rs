//! Command grammar, request ownership, and Tier-1 state-machine tests.

use std::panic::catch_unwind;

use natnet::command::{
    self, CanaryStep, CommandError, CommandResponse, ComponentError, ConnectOptions, ProgressPhase,
    PropertyValue, Request, SetProperty, Tier1Write, Tier2Grant, Tier2Property, Tier2Write,
    WriteTier, classify_property,
};
use natnet::serverinfo::ConnectionInfo;
use natnet::{Error, NAT_CONNECT, NAT_REQUEST, NAT_REQUEST_MODELDEF, ServerInfo, Version};

// Assertions shared by tests: malformed output must fail the test immediately.
#[allow(clippy::expect_used)]
fn command_string(request: &Request) -> String {
    let payload = &request.as_bytes()[4..];
    let end = payload
        .iter()
        .position(|&byte| byte == 0)
        .expect("validated command terminator");
    String::from_utf8(payload[..end].to_vec()).expect("validated command text")
}

fn status_ok() -> Vec<u8> {
    CommandResponse::status_datagram(0)
}

fn value(value: &str) -> Vec<u8> {
    CommandResponse::value_datagram(value)
}

#[test]
fn connect_options_use_the_sdk_layout() {
    // The default hello is unchanged: a bare header.
    assert_eq!(
        command::connect_request().unwrap().as_bytes(),
        &[0, 0, 0, 0]
    );

    let options = ConnectOptions {
        subscribed_data_only: true,
        bitstream_version: Some(Version::new(3, 1, 0, 0)),
    };
    let request = command::connect_request_with(options).unwrap();
    let bytes = request.as_bytes();
    assert_eq!(bytes.len(), 4 + command::CONNECT_OPTIONS_PAYLOAD_LEN);
    // Message id 0, payload length 270.
    assert_eq!(&bytes[..4], &[0, 0, 0x0e, 0x01]);
    let payload = &bytes[4..];
    assert!(payload[..264].iter().all(|&byte| byte == 0), "sender block");
    assert_eq!(payload[264], 1, "subscribedDataOnly");
    assert_eq!(&payload[265..269], &[3, 1, 0, 0], "BitstreamVersion");
    assert_eq!(payload[269], 0);
    assert_eq!(ConnectOptions::parse(payload), Some(options));

    // No version requested: 0.0.0.0, which Motive reads as its current one.
    let current = command::connect_request_with(ConnectOptions::default()).unwrap();
    assert_eq!(&current.as_bytes()[4 + 264..], &[0; 6]);
    assert_eq!(
        ConnectOptions::parse(&current.as_bytes()[4..]),
        Some(ConnectOptions::default())
    );

    // The SDK Python sample's hello parses; a payload too short to hold the
    // options, the default hello's among them, carries none.
    let mut python = vec![0u8; 270];
    python[..4].copy_from_slice(b"Ping");
    python[265..269].copy_from_slice(&[4, 1, 0, 0]);
    assert_eq!(
        ConnectOptions::parse(&python),
        Some(ConnectOptions::requesting(Version::V4_1))
    );
    assert_eq!(ConnectOptions::parse(&[]), None);
    assert_eq!(ConnectOptions::parse(&python[..268]), None);
}

#[test]
fn stream_version_follows_the_sdk_rule() {
    let info = |protocol_version| ServerInfo {
        application_name: "Motive".to_string(),
        server_version: Version::new(3, 1, 2, 1),
        protocol_version,
        high_res_clock_frequency: 10_000_000,
        connection: ConnectionInfo {
            data_port: 1511,
            multicast: false,
            multicast_group: [0; 4],
        },
    };
    let newer = info(Version::new(4, 2, 0, 0));
    let old = info(Version::new(2, 10, 0, 0));
    let request = ConnectOptions::requesting(Version::V4_1);

    assert_eq!(request.stream_version(&newer), Version::V4_1);
    // A NatNet 2 server cannot change its bitstream.
    assert_eq!(request.stream_version(&old), Version::new(2, 10, 0, 0));
    assert_eq!(
        ConnectOptions::default().stream_version(&newer),
        Version::new(4, 2, 0, 0)
    );
}

#[test]
fn typed_requests_have_expected_ids_and_bytes() {
    let connect = command::connect_request().unwrap();
    assert_eq!(
        u16::from_le_bytes([connect.as_bytes()[0], connect.as_bytes()[1]]),
        NAT_CONNECT
    );

    let modeldef = command::request_modeldef().unwrap();
    assert_eq!(
        u16::from_le_bytes([modeldef.as_bytes()[0], modeldef.as_bytes()[1]]),
        NAT_REQUEST_MODELDEF
    );

    let get = command::get_property("cam-1", " enabled ").unwrap();
    assert_eq!(
        u16::from_le_bytes([get.as_bytes()[0], get.as_bytes()[1]]),
        NAT_REQUEST
    );
    assert_eq!(command_string(&get), "GetProperty,cam-1,Enabled");
}

#[test]
fn command_component_injection_is_rejected_before_formatting() {
    for node in [
        "cam-1,SetProperty,root",
        "cam-1\0root",
        "cam-1\nroot",
        "cém",
    ] {
        assert!(matches!(
            command::get_property(node, "Enabled"),
            Err(CommandError::InvalidComponent { .. })
        ));
    }
    for property in ["Enabled,Record", "Enabled\0Record", "Enabled\tRecord"] {
        assert!(matches!(
            command::get_property("cam-1", property),
            Err(CommandError::InvalidComponent { .. })
        ));
    }
    assert!(matches!(
        SetProperty::plan(
            vec!["cam-1".into()],
            "Enabled",
            PropertyValue::Text("1,GetProperty,root".into())
        ),
        Err(CommandError::InvalidComponent {
            component: command::CommandComponent::Value,
            reason: ComponentError::Comma,
        })
    ));
}

#[test]
fn ordinary_names_and_root_scope_remain_valid() {
    let root = command::get_property("", "Mode").unwrap();
    assert_eq!(command_string(&root), "GetProperty,,Mode");

    let write = Tier1Write::camera_enabled(vec!["cam-1".into()], true)
        .unwrap()
        .begin();
    assert_eq!(
        command_string(write.set_request()),
        "SetProperty,cam-1,Enabled,1"
    );
    assert_eq!(
        command_string(write.reread_request()),
        "GetProperty,cam-1,Enabled"
    );
}

#[test]
fn request_payload_length_includes_nul_and_is_bounded() {
    let property = "x".repeat(65_520);
    let request = command::get_property("n", &property).unwrap();
    assert_eq!(request.as_bytes().len() - 4, usize::from(u16::MAX));

    let too_long = "x".repeat(65_521);
    assert!(matches!(
        command::get_property("n", &too_long),
        Err(CommandError::RequestTooLong {
            length: 65_536,
            max: 65_535
        })
    ));
}

#[test]
fn all_targets_are_validated_and_exact_duplicates_rejected() {
    assert!(matches!(
        Tier1Write::camera_enabled(vec!["cam-1".into(), "cam-1".into()], true),
        Err(CommandError::DuplicateTarget { ref target }) if target == "cam-1"
    ));
    assert!(matches!(
        Tier1Write::camera_enabled(vec![String::new()], true),
        Err(CommandError::InvalidComponent { .. })
    ));
    assert!(matches!(
        Tier1Write::camera_enabled(vec![], true),
        Err(CommandError::NoTargets)
    ));
}

#[test]
fn classification_is_fail_closed_and_canonicalized() {
    assert_eq!(classify_property("video mode"), WriteTier::Tier1);
    assert_eq!(classify_property("Transmission Type"), WriteTier::Tier2);
    assert_eq!(classify_property("Frobnicate"), WriteTier::Unsanctioned);

    let plan = SetProperty::plan(
        vec!["cam-1".into()],
        " eNaBlEd ",
        PropertyValue::Bool(false),
    )
    .unwrap();
    assert_eq!(plan.property(), "Enabled");
    assert_eq!(
        command_string(plan.begin().set_request()),
        "SetProperty,cam-1,Enabled,0"
    );
}

#[test]
fn tier2_grant_binds_authority_and_action() {
    let grant = Tier2Grant::sign_off("owner @ console 2026-09-08", Tier2Property::Port).unwrap();
    assert_eq!(grant.authority(), "owner @ console 2026-09-08");
    assert_eq!(grant.action(), Tier2Property::Port);

    let write = Tier2Write::authorize(grant, "", PropertyValue::Int(1510)).unwrap();
    assert_eq!(write.authority(), "owner @ console 2026-09-08");
    assert_eq!(write.action(), Tier2Property::Port);
    assert_eq!(command_string(write.request()), "SetProperty,,Port,1510");
    assert_eq!(write.request().as_bytes()[2..4], [23, 0]);
}

#[test]
fn tier2_malformed_authority_and_unsupported_asset_definition_fail() {
    assert!(matches!(
        Tier2Grant::sign_off("", Tier2Property::Port),
        Err(CommandError::InvalidComponent { .. })
    ));
    assert!(matches!(
        Tier2Grant::sign_off("owner\n", Tier2Property::Port),
        Err(CommandError::InvalidComponent { .. })
    ));
    assert!(matches!(
        Tier2Grant::sign_off("   ", Tier2Property::Port),
        Err(CommandError::InvalidComponent {
            component: command::CommandComponent::Authority,
            reason: ComponentError::Empty,
        })
    ));
    let padded = Tier2Grant::sign_off(" owner ", Tier2Property::Port).unwrap();
    assert_eq!(padded.authority(), " owner ");

    let grant = Tier2Grant::sign_off("owner", Tier2Property::AssetDefinition).unwrap();
    assert!(matches!(
        Tier2Write::authorize(grant, "asset", PropertyValue::Text("x".into())),
        Err(CommandError::UnsupportedOperation {
            operation: "Asset Definition"
        })
    ));
}

#[test]
fn every_tier2_action_is_scoped_and_asset_definition_stays_unsupported() {
    for action in Tier2Property::ALL {
        let grant = Tier2Grant::sign_off("owner", action).unwrap();
        let result = Tier2Write::authorize(grant, "node", PropertyValue::Int(1));
        if action == Tier2Property::AssetDefinition {
            assert!(matches!(
                result,
                Err(CommandError::UnsupportedOperation {
                    operation: "Asset Definition"
                })
            ));
        } else {
            let write = result.unwrap();
            assert_eq!(write.action(), action);
            assert_eq!(write.authority(), "owner");
            assert!(command_string(write.request()).starts_with("SetProperty,node,"));
        }
    }
}

#[test]
fn tier2_stringly_path_never_builds_a_write() {
    for property in ["Transmission Type", "Asset Definition", "Frobnicate"] {
        assert!(matches!(
            SetProperty::plan(vec!["node".into()], property, PropertyValue::Int(1)),
            Err(CommandError::Tier2Refused { .. })
        ));
    }
}

#[test]
fn canary_state_unlocks_one_target_at_a_time_and_records_progress() {
    let write =
        Tier1Write::camera_enabled(vec!["cam-1".into(), "cam-2".into(), "cam-3".into()], true)
            .unwrap();

    let first: CanaryStep = write.begin();
    assert!(first.is_canary());
    assert_eq!(first.target(), "cam-1");
    assert_eq!(
        command_string(first.set_request()),
        "SetProperty,cam-1,Enabled,1"
    );
    let second = first
        .verify_status(&status_ok(), &value("1"))
        .unwrap()
        .next()
        .unwrap();
    assert!(!second.is_canary());
    assert_eq!(second.target(), "cam-2");
    assert_eq!(
        command_string(second.set_request()),
        "SetProperty,cam-2,Enabled,1"
    );

    let third = second
        .verify_status(&status_ok(), &value("true"))
        .unwrap()
        .next()
        .unwrap();
    assert_eq!(third.target(), "cam-3");
    let done = third.verify_status(&status_ok(), &value("1")).unwrap();
    let log = done.finish().unwrap();
    assert_eq!(log.records().len(), 3);
    assert_eq!(log.records()[0].phase(), ProgressPhase::Canary);
    assert_eq!(log.records()[1].phase(), ProgressPhase::Target);
    assert_eq!(log.records()[0].status(), Some(0));
    assert_eq!(log.records()[2].target(), "cam-3");
}

#[test]
fn canary_status_failure_and_reread_mismatch_stop_progression() {
    let write = Tier1Write::camera_enabled(vec!["cam-1".into(), "cam-2".into()], true).unwrap();
    let step = write.begin();
    let err = step.verify_status(&CommandResponse::status_datagram(7), &value("1"));
    assert!(matches!(
        err,
        Err(CommandError::StatusFailure { ref target, status: 7 }) if target == "cam-1"
    ));

    let step = Tier1Write::camera_enabled(vec!["cam-1".into(), "cam-2".into()], true)
        .unwrap()
        .begin();
    assert!(matches!(
        step.verify_status(&status_ok(), &value("0")),
        Err(CommandError::RereadMismatch { ref target, .. }) if target == "cam-1"
    ));
}

#[test]
fn second_target_mismatch_retains_canary_progress() {
    let error = Tier1Write::camera_enabled(vec!["cam-1".into(), "cam-2".into()], true)
        .unwrap()
        .begin()
        .verify_status(&status_ok(), &value("1"))
        .unwrap()
        .next()
        .unwrap()
        .verify_status(&status_ok(), &value("0"))
        .expect_err("second target mismatch must stop progression");

    match &error {
        CommandError::VerificationFailed { source, progress } => {
            assert!(matches!(
                source.as_ref(),
                CommandError::RereadMismatch { target, .. } if target == "cam-2"
            ));
            assert_eq!(progress.len(), 1);
            assert_eq!(progress[0].target(), "cam-1");
        }
        other => panic!("expected retained verification progress, got {other:?}"),
    }
    assert_eq!(error.progress().len(), 1);
}

#[test]
fn response_decoder_keeps_four_byte_ambiguity_documented() {
    for (text, status) in [("abc", 6_513_249), ("1.0", 3_157_553), ("yes", 7_562_617)] {
        let reply = CommandResponse::value_datagram(text);
        assert_eq!(
            CommandResponse::parse(&reply).unwrap(),
            CommandResponse::Status(status)
        );
        assert_eq!(CommandResponse::parse_value(&reply).unwrap(), text);
    }
    assert_eq!(
        CommandResponse::parse(&CommandResponse::value_datagram("abcd")).unwrap(),
        CommandResponse::Value("abcd".into())
    );
}

#[test]
fn command_response_parses_status_and_value() {
    assert_eq!(
        CommandResponse::parse(&CommandResponse::status_datagram(0)).unwrap(),
        CommandResponse::Status(0)
    );
    assert_eq!(
        CommandResponse::parse(&CommandResponse::value_datagram("2")).unwrap(),
        CommandResponse::Value("2".into())
    );
}

#[test]
fn three_byte_values_have_the_documented_literal_status_codes() {
    for (text, status) in [
        ("abc", 6_513_249_i32),
        ("1.0", 3_157_553_i32),
        ("yes", 7_562_617_i32),
        ("off", 6_710_895_i32),
        ("100", 3_158_065_i32),
    ] {
        let reply = CommandResponse::value_datagram(text);
        assert_eq!(reply.len() - 4, 4);
        assert_eq!(
            CommandResponse::parse(&reply).unwrap(),
            CommandResponse::Status(status)
        );
    }
    for text in ["", "a", "ab", "abcd", "Enabled"] {
        assert_ne!(CommandResponse::value_datagram(text).len() - 4, 4);
    }
}

#[test]
fn ambiguity_population_collides_exactly_at_three_bytes() {
    const ALPHABET: [&[u8]; 6] = [b"a", b"1", b".", b"Z", "\u{e9}".as_bytes(), b"\x7f"];
    let mut words: Vec<Vec<u8>> = vec![Vec::new()];
    let mut all: Vec<Vec<u8>> = vec![Vec::new()];
    for _ in 0..5 {
        let mut next = Vec::new();
        for word in &words {
            for symbol in ALPHABET {
                let mut candidate = word.clone();
                candidate.extend_from_slice(symbol);
                next.push(candidate);
            }
        }
        all.extend(next.iter().cloned());
        words = next;
    }
    let mut checked = 0;
    let mut collided = 0;
    for raw in &all {
        let text = core::str::from_utf8(raw).unwrap();
        let parsed = CommandResponse::parse(&CommandResponse::value_datagram(text)).unwrap();
        checked += 1;
        if raw.len() == 3 {
            collided += 1;
            assert!(matches!(parsed, CommandResponse::Status(_)));
        } else {
            assert_eq!(parsed, CommandResponse::Value(text.into()));
        }
    }
    assert_eq!(checked, 9_331);
    assert_eq!(collided, 135);
}

#[test]
fn request_context_reads_value_at_every_payload_length() {
    for text in ["abc", "1.0", "yes", "off", "100", "", "a", "ab", "abcd"] {
        assert_eq!(
            CommandResponse::parse_value(&CommandResponse::value_datagram(text)).unwrap(),
            text
        );
    }
    assert_eq!(
        CommandResponse::parse_value(&CommandResponse::status_datagram(0)).unwrap(),
        ""
    );
    assert_eq!(
        CommandResponse::parse_value(&CommandResponse::status_datagram(49)).unwrap(),
        "1"
    );
    assert!(matches!(
        CommandResponse::parse_value(&CommandResponse::status_datagram(-1)),
        Err(Error::UnterminatedString { .. })
    ));
    assert!(matches!(
        CommandResponse::parse_value(command::connect_request().unwrap().as_bytes()),
        Err(Error::UnexpectedMessageId { expected: 3, .. })
    ));
    assert_eq!(
        CommandResponse::parse_value_payload(b"1.0\0").unwrap(),
        "1.0"
    );
}

#[test]
fn three_byte_canary_echo_uses_the_request_context_escape() {
    let plan = || Tier1Write::camera_video_mode(vec!["cam-1".into(), "cam-2".into()], 100).unwrap();
    let step = plan().begin();
    let reread = CommandResponse::value_datagram("100");
    assert!(matches!(
        step.verify(&reread),
        Err(CommandError::UnverifiableReply)
    ));

    let echo = CommandResponse::parse_value(&reread).unwrap();
    let verified = plan().begin().verify_echo(&echo).unwrap();
    assert_eq!(verified.next().unwrap().target(), "cam-2");

    let short = Tier1Write::camera_video_mode(vec!["cam-1".into(), "cam-2".into()], 10)
        .unwrap()
        .begin();
    assert!(short.verify(&CommandResponse::value_datagram("10")).is_ok());

    let text = SetProperty::plan(
        vec!["asset-1".into()],
        "Streaming",
        PropertyValue::Text("Yes".into()),
    )
    .unwrap()
    .begin();
    let text_reply = CommandResponse::value_datagram("Yes");
    assert!(matches!(
        text.verify(&text_reply),
        Err(CommandError::UnverifiableReply)
    ));

    let real_status = CommandResponse::status_datagram(0);
    assert_eq!(CommandResponse::parse_value(&real_status).unwrap(), "");
    assert!(matches!(
        plan().begin().verify_echo(""),
        Err(CommandError::RereadMismatch { ref found, .. }) if found.is_empty()
    ));
}

#[test]
fn verify_echo_matches_verify_on_unambiguous_replies() {
    let step = || {
        Tier1Write::camera_enabled(vec!["cam-1".into()], true)
            .unwrap()
            .begin()
    };
    for echo in ["1", "true", "TRUE", "0", "false", "", "wrong", "11111"] {
        let reply = CommandResponse::value_datagram(echo);
        let via_bytes = step().verify(&reply);
        let via_echo = step().verify_echo(echo);
        match (&via_bytes, &via_echo) {
            (Ok(left), Ok(right)) => assert_eq!(left.progress(), right.progress()),
            (
                Err(CommandError::RereadMismatch {
                    target: target_a,
                    property: property_a,
                    expected: expected_a,
                    found: found_a,
                }),
                Err(CommandError::RereadMismatch {
                    target: target_b,
                    property: property_b,
                    expected: expected_b,
                    found: found_b,
                }),
            ) => assert_eq!(
                (target_a, property_a, expected_a, found_a),
                (target_b, property_b, expected_b, found_b)
            ),
            other => panic!("verify and verify_echo disagreed: {other:?}"),
        }
    }
    assert!(step().verify_echo("1").is_ok());
    assert!(matches!(
        step().verify_echo("wrong"),
        Err(CommandError::RereadMismatch { .. })
    ));
    assert!(matches!(
        step().verify_echo(""),
        Err(CommandError::RereadMismatch { .. })
    ));
}

#[test]
fn command_reply_prefixes_and_random_bytes_never_panic() {
    let replies = [
        CommandResponse::status_datagram(0),
        CommandResponse::value_datagram("Enabled"),
        command::connect_request().unwrap().as_bytes().to_vec(),
        command::get_property("cam-1", "Enabled")
            .unwrap()
            .as_bytes()
            .to_vec(),
    ];
    for full in &replies {
        for length in 0..=full.len() {
            let prefix = full[..length].to_vec();
            assert!(catch_unwind(|| CommandResponse::parse(&prefix)).is_ok());
        }
    }

    let mut state: u64 = 0x0bad_c0de_dead_beef;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for _ in 0..4_000 {
        let length = (next() % 40) as usize;
        let mut bytes = vec![0; length];
        for byte in &mut bytes {
            *byte = (next() & 0xff) as u8;
        }
        assert!(catch_unwind(|| CommandResponse::parse(&bytes)).is_ok());
    }
}

#[test]
fn canary_decoding_is_total_on_bad_replies() {
    let make_step = || {
        Tier1Write::camera_enabled(vec!["cam-1".into()], true)
            .unwrap()
            .begin()
    };
    assert!(matches!(
        make_step().verify(&CommandResponse::status_datagram(0)),
        Err(CommandError::UnverifiableReply)
    ));
    assert!(matches!(
        make_step().verify(&[3, 0]),
        Err(CommandError::Wire(_))
    ));
}

#[test]
fn property_value_match_tolerates_common_echo_encodings() {
    assert!(PropertyValue::Bool(true).matches("1"));
    assert!(PropertyValue::Bool(true).matches("true"));
    assert!(PropertyValue::Bool(true).matches("TRUE"));
    assert!(!PropertyValue::Bool(true).matches("0"));
    assert!(PropertyValue::Int(2).matches(" 2 "));
    assert!(!PropertyValue::Int(2).matches("3"));
    assert!(PropertyValue::Text("Object".into()).matches("Object"));
    assert!(!PropertyValue::Text("Object".into()).matches(" Object"));
    assert!(!PropertyValue::Text("Object".into()).matches("Object "));
    assert!(PropertyValue::Text(" Object ".into()).matches(" Object "));
}
