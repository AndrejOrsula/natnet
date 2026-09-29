# riders-71 C1 report

## Candidate

- Base commit: `120330241ff05d55d7c47bf004c1d9678fc0d0cc`
- Base tree: `b125d77f1ea74042a7ea553f48a868db09102a1f`
- Candidate commit: `ea0bba5fc3b1cadea5ec567281a293051681fa7e`
- Candidate tree: `1b4db6495fdbf19b1d8b51f8a16ac9a5cd0b9641`
- Commit: `test(riders-71): pin NatNet boundary errors and UDP limit`
- Candidate tracked tree is clean. No push performed.

## Scope

Only the five authorized paths changed:

- `deny.toml`
- `tests/decoder.rs`
- `tests/frame_versions.rs`
- `tests/modeldef_versions.rs`
- `tests/net_command.rs`

No `src/**`, README, workflow, imagery-net test, or Auga file changed. No benchmark or hardware claim.

## Boundary controls and mutation evidence

The prechange file suites passed with 12 modeldef, 7 decoder, 11 frame-version, and 11 net-command tests (the net suite used `--features net`). Each new control was then run against a bounded mutation or ablation in an isolated target directory, with source restored before the candidate was frozen.

1. `hostile_dataset_count_is_implausible_before_a_truncated_entry` calls the public strict MODELDEF payload parser for legacy 3.0 and 4.1. A count of 1,000 with no remaining bytes returns `ImplausibleCount`, while one bounded rigid-body entry with a short body returns `UnexpectedEof`. Removing the `Reader::guard_count` remaining-byte check changed the hostile case to `UnexpectedEof { offset: 4, needed: 4, available: 0 }`; the focused test failed. Candidate modeldef suite: 13 passed.
2. `version_at_least_orders_major_and_minor_but_ignores_build_and_revision` covers both major/minor directions, equal minor, and high build/revision values. Changing the minor comparison from `>=` to `>` failed the equal `4.1` assertion. Candidate decoder suite: 8 passed.
3. `v41_labeled_marker_guard_rejects_a_short_entry_before_truncation` supplies one 25-byte 4.1 labeled-marker entry; strict parsing returns `ImplausibleCount { count: 1, available: 25 }`, pinning the 26-byte minimum. Removing the section minimum-entry guard changed the result to `UnexpectedEof { offset: 74, needed: 4, available: 3 }`; the focused test failed. Candidate frame-version suite: 12 passed.
4. `exchange_pins_the_ipv4_udp_payload_boundary_before_socket_io` uses loopback only. It accepts exactly `MAX_IPV4_UDP_PAYLOAD == 65_507`, then rejects 65,508 bytes with `InvalidInput` and proves no oversized datagram reached the peer. Changing the size comparison from `>` to `>=` rejected the exact-limit request; the focused test failed. Candidate net-command suite: 12 passed with `--features net`.

All ablated source was restored before freezing; no source path is in the candidate diff.

## License decision

`BSD-2-Clause` and `BSD-3-Clause` were removed from `deny.toml`. A controlled config with those two allowances restored produced two `license-not-encountered` warnings. The candidate's `cargo deny` run has no warnings and reports all four requested checks okay. No new allowance was added.

## Verification summary

Post-commit gates ran against the exact candidate with isolated `CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders71/candidate-post`:

- format, clippy, default tests, all-feature tests, net build, imagery+net example build, MSRV default/all-feature tests, cargo-deny, and committed-tree whitespace: exit 0.
- Default test gate: 97 runtime tests and 5 doctests passed; 0 failed, 0 ignored.
- All-feature test gate: 177 runtime tests and 10 doctests passed; 0 failed, 0 ignored.
- All six workflow feature rows passed clippy and tests: `imagery`, `net`, `serde`, `nalgebra`, `imagery,net`, `serde,nalgebra`.
- Strict rustdoc/check gates passed all 16 workflow feature combinations.
- MSRV `cargo +1.92 test --locked --no-fail-fast` and its all-feature form passed.
- `cargo deny ... check advisories bans licenses sources`: `advisories ok, bans ok, licenses ok, sources ok`.

The three-leg refuted-citation sweep was run with the exact resolved base and candidate, plus the widened base-parent leg. All legs reported no hit for the neutral token `boundaryc1`; the tool explicitly says a negative result proves nothing. No citation, name, or number was refuted. No approving description of unfixed behavior was added, and no closed enumeration grew; the deny allowlist only shrank.

## Residual risks

Evidence is host-side and loopback-only. It does not claim Windows behavior, Motive interoperability, or live-device behavior. The net test proves the IPv4 UDP payload contract through the command client seam. Future dependencies using BSD licenses would need an explicit reviewed policy decision.
