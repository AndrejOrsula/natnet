# riders-71 C1 — pin untested NatNet boundaries

Repository: `natnet`. Base: `dev` `120330241ff05d55d7c47bf004c1d9678fc0d0cc` (resolve and report again). Candidate branch: `slice/riders-71`; worktree: `/home/andrej/ws_rs/natnet_wt/impl-riders-71`. Evidence branch: `evidence/riders-71`. Access: `remote`. Read `natnet/README.md`, workflow, and Auga `AGENTS.md` §§4–7. The Auga STATUS row is dispatch authority; NatNet's own workflow supplies its gates.

## Outcome and constraints

Add mutation-killing regression controls for four live but weakly witnessed boundaries. No production-code change is authorized. Own only `tests/modeldef_versions.rs`, `tests/decoder.rs`, `tests/frame_versions.rs`, `tests/net_command.rs`, and `deny.toml`. Do not edit `src/**`, `README.md`, workflows, `tests/imagery_net.rs`, or Auga files. If a test exposes a production defect, report it and stop that subcase; do not silently expand scope. Keep behavior, API, wire layout, and features unchanged. No benchmark or hardware claim.

1. `Reader::guard_count` in `src/bytes.rs` rejects a MODELDEF count that cannot fit remaining bytes, before `Vec::with_capacity` in `src/modeldef.rs`. Current hostile-count tests mainly exercise frame counts. Add a direct public MODELDEF decode test that distinguishes `ImplausibleCount` from `Truncated`. Use a bounded hostile value (for example 1,000 with a short payload), not an OOM-sized count. Exercise the relevant legacy and 4.1 layouts where feasible. Show guard-bypass ablation safely in an isolated build, and restore source before freezing.
2. Pin the boundary behavior of exported `Version::at_least` with positive and negative major/minor cases, including build/revision treatment. Place it in an owned test file.
3. Pin the 26-byte minimum for 4.1 labeled markers in `src/frame.rs` with a packet whose count exceeds what the remaining bytes can hold; distinguish the guard error from a later truncation. Demonstrate a plausible mutation that the test kills.
4. Pin the UDP datagram-size boundary in `src/net.rs` with an owned test, using loopback only and deterministic bounded setup. Demonstrate a plausible mutation killed. Do not rely on Motive or a Windows host.
5. `cargo deny check licenses` currently reports two unmatched BSD allowances. Decide whether to remove them or retain them with precise rationale grounded in the current dependency graph; run cargo-deny and report its warnings and result. Do not add a new allowance.

For each test, show prechange behavior, the mutation/ablation result, and restored candidate result. If a mutation would be unsafe or cannot be made deterministic, state that and provide the strongest safe negative control. Search existing tests first to avoid duplicates.

## Gates and handoff

Use an isolated `CARGO_TARGET_DIR`. Run focused tests, `cargo fmt --all --check`, `cargo clippy --locked --all-targets --all-features -- -D warnings`, default and all-feature tests, `cargo build --locked --features net`, strict rustdoc for the affected feature combinations (justify rows against the workflow's 16-combination matrix), MSRV checks, `cargo deny check advisories bans licenses sources`, and committed-tree whitespace check. Record exact command, exit code, and result; do not equate unavailable rows with passed. Host-only evidence is not Windows evidence.

Commit only owned code/test/config paths as immutable `candidate(C1)`. Record SHA and tree. Draft `docs/work/riders-71/attempts/C1/report.md` and `gates.md` in the worktree **uncommitted** for coordinator, with any residual concern. Do not push or edit a frozen packet. Coordinator will commission a separate read-only review, commit evidence, then integrate only an accepted candidate.
