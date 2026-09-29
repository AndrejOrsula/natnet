//! Parses arbitrary AUGI/1 records; parsing must never panic.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &[u8]| {
    let _ = natnet::imagery::parse_record(input);
});
