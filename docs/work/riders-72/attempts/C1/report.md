# riders-72 C1 — builder report

## 1. What was built

Added one crate-level executable rustdoc example in `src/lib.rs:43-70`.
The example uses `SyntheticServer::new(Version::V4_1, 1_000_000)` to generate
one deterministic tracked rigid body (id `7`), decodes the datagram with
`Decoder::new(Version::V4_1)?.parse(&wire)?`, checks the
`Message::FrameOfData` frame/body identity, and asserts checked byte-for-byte
re-encoding with `decoded.to_datagram()?`.

Only `src/lib.rs` changed in the candidate: 29 added lines in the candidate
diff, one owned path. No implementation module, test, manifest, workflow,
feature, API, wire format, runtime, socket, clock, randomness, benchmark, or
hardware behavior changed. Existing compile-fail examples remain unchanged.

## 2. Brief correction

The brief says: “before editing, record `cargo test --locked --doc` showing
ten negative doctests and zero positive examples” and later requires “eleven
passing doctests.” On the base, the default-feature doc-test population had 5
compile-fail cases and 0 positive cases (`cargo test --locked --doc`, exit 0);
the all-feature population had 10 compile-fail cases and 0 positive cases
(`cargo test --locked --doc --all-features`, exit 0). The five additional
cases are the feature-gated `src/net.rs` examples, so plain default features
cannot report ten. The all-feature interpretation supplies the requested
eleven after this change. No source was broadened to compensate.

## 3. Before, after, and negative control

Before editing, with base `120330241ff05d55d7c47bf004c1d9678fc0d0cc`:

- `CARGO_TARGET_DIR=/tmp/natnet-riders-72-target-base cargo test --locked --doc`
  — exit 0; the default-feature doc-test population was 5 compile-fail cases,
  0 positive cases.
- `CARGO_TARGET_DIR=/tmp/natnet-riders-72-target-base-all cargo test --locked --doc --all-features`
  — exit 0; the all-feature doc-test population was 10 compile-fail cases,
  0 positive cases.

After the final source was restored and before freezing the candidate:

- `CARGO_TARGET_DIR=/tmp/natnet-riders-72-target-final-doc cargo test --locked --doc`
  — exit 0; the default-feature doc-test population was 1 positive plus 5
  compile-fail cases, all passing.
- `CARGO_TARGET_DIR=/tmp/natnet-riders-72-target-final-doc-all cargo test --locked --doc --all-features`
  — exit 0; the all-feature doc-test population was 1 positive plus 10
  compile-fail cases, all passing (11 cases total).

Negative control: in an uncommitted temporary mutation, the body-id assertion
at `src/lib.rs:62` was changed from `7` to `8`. The default doc-test command
returned exit 101; the positive test failed with `left: 7` and `right: 8`,
while the separate population of 5 compile-fail cases still passed. The line
was restored with `apply_patch`, `src/lib.rs` was touched to invalidate the
build, and the two passing final doc-test runs above were rerun. The mutation
is absent from the candidate.

## 4. Candidate identity and state

| field | value |
|---|---|
| base | `120330241ff05d55d7c47bf004c1d9678fc0d0cc` |
| candidate(C1) | `adbd2bcf157fb567e35d7b242fb94ec4529a3616` |
| candidate tree | `74e3f9b730adfdcdfe58d6bb5d0f448f437b5cf5` |
| branch | `slice/riders-72` |
| worktree | `/home/andrej/ws_rs/natnet_wt/impl-riders-72` |
| access | `remote` |
| committed paths | `src/lib.rs` only |
| push | none |

The commit-message file used with `git commit -F` is
`/tmp/natnet-riders-72-C1-message.txt`: one file, 217 bytes,
SHA-256 `945e23fc6f3c17f70bca9862fd2c63f59a3858b40e08f0c59e1ed2a3e980c1d5`.
The candidate is immutable; no amend or follow-up source edit was made.

## 5. Decisions and limits

Used fixed literal `Version::V4_1` at both generation and strict-decoder
seams, fixed body values, and no extra fixture or dependency. The negative
control and the brief count mismatch were recorded rather than changing
scope. No benchmark, Windows run, hardware run, or live Motive claim was made.

The first candidate MSRV all-feature run hit the existing timing-sensitive
`tests/imagery_net.rs::a_peer_still_sending_past_the_deadline_is_a_timeout`
test (23 passed, 1 failed in that runtime-test population); an immediate
second run on the same candidate passed all 24 runtime tests and all 11
doc-test cases. Both attempts are recorded in `gates.md`; this remains a
residual timing-flake risk unrelated to `src/lib.rs`.

Exact gate commands and outputs are in `gates.md`.
