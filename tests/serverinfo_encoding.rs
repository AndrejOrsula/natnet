//! Public SERVERINFO encoding and strict/compatibility decoding contracts.
#![allow(clippy::unwrap_used)]

use natnet::{ConnectionInfo, Error, ParsePolicy, ServerInfo, Version};

fn info(name: &str) -> ServerInfo {
    ServerInfo {
        application_name: name.into(),
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
fn checked_name_encoding_refuses_interior_nul() {
    assert!(matches!(
        info("Motive\0hidden").to_payload(),
        Err(Error::InteriorNul { offset: 6, .. })
    ));
}

#[test]
fn checked_name_encoding_refuses_byte_truncation() {
    for name in ["a".repeat(256), "é".repeat(128)] {
        assert!(matches!(
            info(&name).to_payload(),
            Err(Error::LengthOverflow {
                length: 256,
                max: 255,
                ..
            })
        ));
    }
    let value = info(&"é".repeat(127));
    assert_eq!(
        ServerInfo::parse(&value.to_datagram().unwrap()).unwrap(),
        value
    );
}

#[test]
fn strict_parser_rejects_suffix_while_compatibility_preserves_acceptance() {
    let value = info("Motive");
    let mut payload = value.to_payload().unwrap();
    payload.push(7);
    assert_eq!(ServerInfo::parse_payload(&payload).unwrap(), value);
    assert_eq!(
        ServerInfo::parse_payload_with_policy(&payload, ParsePolicy::Strict),
        Err(Error::TrailingBytes { remaining: 1 })
    );
}

#[test]
fn server_information_retains_unrecognized_version_for_diagnostics() {
    let mut value = info("future server");
    value.protocol_version = Version::new(99, 0, 7, 8);
    assert_eq!(
        ServerInfo::parse(&value.to_datagram().unwrap()).unwrap(),
        value
    );
    assert!(value.protocol_version.validate().is_err());
}

#[test]
fn a_lossily_decoded_name_longer_than_255_bytes_refuses_checked_reencode() {
    let payload = info("Motive").to_payload().unwrap();
    for (field, length) in [
        (vec![b'a'; 256], 256),
        ([vec![0xff; 255], vec![0]].concat(), 765),
    ] {
        let mut wire = payload.clone();
        wire[..256].copy_from_slice(&field);
        for policy in [ParsePolicy::Strict, ParsePolicy::Compatible] {
            let decoded = ServerInfo::parse_payload_with_policy(&wire, policy).unwrap();
            assert_eq!(decoded.application_name.len(), length);
            assert_eq!(
                decoded.to_datagram(),
                Err(Error::LengthOverflow {
                    field: "application_name",
                    length,
                    max: 255,
                })
            );
            let lossy = ServerInfo::parse(&decoded.to_datagram_lossy()).unwrap();
            assert!(lossy.application_name.len() <= 255);
        }
    }
}
