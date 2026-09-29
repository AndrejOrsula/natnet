# riders-72 C1 — execute one positive NatNet use in rustdoc

Repository: `natnet`. Base: `dev` `120330241ff05d55d7c47bf004c1d9678fc0d0cc` (resolve and report). Candidate branch: `slice/riders-72`; worktree: `/home/andrej/ws_rs/natnet_wt/impl-riders-72`. Evidence branch: `evidence/riders-72`. Access: `remote`. Read `natnet/README.md`, the workflow, and Auga `AGENTS.md` §§4–7.

## Finding and outcome

Every current Rust doctest is negative: ten `compile_fail` blocks; the remaining source fences are text. Add one small executable crate-level example proving the ordinary sans-io purpose: generate a deterministic supported FrameOfData datagram, decode it strictly, assert the frame/body identity, and encode it back to the same bytes. The public seams already exist: `SyntheticServer`, `PlannedBody`, `Decoder`, `Message`, and `Version` are re-exported from `src/lib.rs`.

Own **only `src/lib.rs`**. Do not edit implementation modules, tests, README, workflow, manifests, deny policy, or `docs/work/**`. Active riders-71 owns four test files and `deny.toml`; no overlap is permitted. Preserve all existing negative doctests. No dependency, feature, API, wire-format, runtime, socket, clock, randomness, benchmark, or hardware change.

The example must be deterministic and useful, not merely compile. Prefer a hidden `fn main() -> natnet::Result<()>`, `SyntheticServer::new(Version::V4_1, ...)`, one tracked rigid body with fixed values, strict `Decoder::new(Version::V4_1)?.parse(&wire)?`, an assertion over `Message::FrameOfData`, and `assert_eq!(decoded.to_datagram()?, wire)`. Adjust only if the current public API requires it; report any mismatch instead of broadening scope.

## Proof and gates

Test-first control: before editing, record `cargo test --locked --doc` showing ten negative doctests and zero positive examples. After editing, it must show eleven passing doctests, including the named positive example. Demonstrate a safe negative control by changing one asserted identity in an uncommitted temporary mutation and showing the positive doctest fails, then restore it. Do not weaken existing compile-fail blocks.

Use an isolated `CARGO_TARGET_DIR`. Run focused default and all-feature doctests; fmt; Clippy and tests at default/all features; net build; strict docs; the workflow's feature rows relevant to `src/lib.rs` (the crate root is compiled under every one of the 16 combinations, so execute or explicitly account for all); MSRV; cargo-deny and cargo-machete when available; committed-tree whitespace. Record exact commands, exits, counts, and unavailable checks. No Windows/hardware claim.

Commit only `src/lib.rs` as immutable `candidate(C1)`, recording SHA and tree. Draft `docs/work/riders-72/attempts/C1/report.md` and `gates.md` uncommitted in the implementation worktree. Never push. The coordinator will commission a fresh read-only review and integrate only an accepted candidate.
