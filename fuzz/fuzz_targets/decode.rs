//! Decodes arbitrary datagrams at every supported layout and policy.
//!
//! Decoding must never panic. When a decoded message re-encodes, the encoding
//! must decode again and re-encode to the same bytes: checked encoding is
//! idempotent even for lossy input (non-UTF-8 names, NaNs). A frame section
//! that decodes into typed entries must re-encode to the same bytes.
#![no_main]

use libfuzzer_sys::fuzz_target;
use natnet::command::CommandResponse;
use natnet::{
    Decoder, FrameOfData, Message, ParsePolicy, RawSection, Version, parse_message,
    parse_message_string,
};

const VERSIONS: [Version; 4] = [
    Version::new(3, 0, 0, 0),
    Version::new(3, 1, 0, 0),
    Version::new(4, 0, 0, 0),
    Version::V4_1,
];

fuzz_target!(|input: &[u8]| {
    let Some((&selector, datagram)) = input.split_first() else {
        return;
    };
    let version = VERSIONS[usize::from(selector & 0x03)];
    let policy = if selector & 0x04 == 0 {
        ParsePolicy::Strict
    } else {
        ParsePolicy::Compatible
    };
    let decoder = Decoder::new(version)
        .expect("supported version")
        .with_policy(policy);

    let _ = parse_message(datagram, version);
    let _ = parse_message_string(datagram);
    let _ = CommandResponse::parse(datagram);

    let Ok(decoded) = decoder.parse(datagram) else {
        return;
    };
    if let Message::FrameOfData(frame) = decoded.message() {
        check_typed_sections(frame);
    }
    let Ok(first) = decoded.to_datagram() else {
        return;
    };
    let again = decoder
        .parse(&first)
        .expect("a checked encoding decodes under the policy that produced it");
    let second = again
        .to_datagram()
        .expect("a re-decoded checked encoding re-encodes");
    assert_eq!(first, second, "checked encoding is not idempotent");
});

/// Every raw section that decodes re-encodes to its exact count and bytes.
fn check_typed_sections(frame: &FrameOfData) {
    fn same(section: &RawSection, encoded: natnet::Result<RawSection>) {
        assert_eq!(
            &encoded.expect("decoded entries re-encode"),
            section,
            "typed section re-encoding changed the bytes"
        );
    }
    if let Ok(skeletons) = frame.decode_skeletons() {
        same(&frame.skeletons, RawSection::from_skeletons(&skeletons));
    }
    if let Ok(assets) = frame.decode_assets() {
        same(&frame.assets, RawSection::from_assets(&assets));
    }
    if let Ok(plates) = frame.decode_force_plates() {
        same(&frame.force_plates, RawSection::from_force_plates(&plates));
    }
    if let Ok(devices) = frame.decode_devices() {
        same(&frame.devices, RawSection::from_devices(&devices));
    }
}
