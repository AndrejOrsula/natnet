# Contributing to natnet

## Checks

A change passes the same checks CI runs:

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-features
cargo test --locked
RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps --all-features
cargo +1.92 test --locked --all-features
cargo deny check
```

Wire-format changes need a test at every supported layout (3.0, 3.1, 4.0 and
4.1) that proves the byte round trip, plus strict-mode rejection of truncated
and trailing input. Decoding must stay total: malformed input returns an
error, never a panic.

## Fuzzing

`fuzz/` holds [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) targets.
`decode` decodes arbitrary datagrams at every layout and policy and checks that
checked re-encoding is idempotent; `imagery` parses arbitrary AUGI/1 records.
Run a target after touching a decoder:

```sh
cargo +nightly fuzz run decode -- -max_total_time=300
cargo +nightly fuzz run imagery -- -max_total_time=300
```

A crash lands in `fuzz/artifacts/`; turn it into a regression test before
fixing it.

## Fixtures

Only synthetic, spec-derived fixtures are committed. Captures of real Motive
streams can identify a facility and its assets; keep them out of the
repository (`*.natnet`, `*.aucap` and `/captures/` are ignored) and derive a
synthetic fixture that reproduces the property under test. The layouts and the
generator in `src/generate.rs` follow the NatNet SDK 4.1.0.0 headers and
samples; say in the documentation when a layout has not been confirmed against
bytes from a real server.

## Published figures

A *measured* figure published anywhere in this crate (rustdoc, comments,
README, examples, commit messages) carries the statistic that produced it,
the repetition count, and the host load it was measured at. When the load was
not recorded, say so and name the record that was searched for it. Publish a
worst case only as a `max`: a median cannot show one. A *configured* value (a
timeout default, a test deadline, a protocol constant) is a choice, not a
reading, and needs none of this.
