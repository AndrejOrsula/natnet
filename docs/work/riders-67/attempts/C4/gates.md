# riders-67 C4 — gates

Every cargo call ran from `/home/andrej/ws_rs/natnet_wt/impl-riders-67-C4` with
`CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c4/target` and
`/home/andrej/.cargo/bin/cargo`, except the comparison/ablation/extraction runs,
which used per-tree sibling target dirs (`target_extract`, `target_basecmp2`,
`target_abl_A1`, `target_abl_A2`, `target_base`, `target_cand`,
`target_candprint`). Raw log: `/home/andrej/coord_scratch/riders67c4/gates.log`.

Toolchain, from `cargo --version --verbose` / `rustc --version --verbose`:
`cargo 1.97.1 (c980f4866 2026-06-30)`, `rustc 1.97.1 (8bab26f4f 2026-07-14)`,
LLVM 22.1.6, host `x86_64-unknown-linux-gnu`, Ubuntu 24.4.0 (noble), selected by
`rust-toolchain.toml` (`channel = "stable"`). MSRV rows on
`cargo 1.92.0 (344c4567c 2025-10-21)`. glibc 2.39 (`ldd --version`).

| gate | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | rc **0** |
| clippy | `cargo clippy --locked --all-targets --all-features -- -D warnings` | rc **0**, **0** warnings |
| test (all features) | `cargo test --locked --all-features` | **183 passed / 0 failed** over **16** targets |
| test (no features) | `cargo test --locked` | **99 / 0** over 16 targets |
| build (net) | `cargo build --locked --features net` | rc **0** |
| build (example) | `cargo build --locked --features imagery,net --example imagery_dump` | rc **0** |
| doc (no features) | `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps` | rc **0** |
| doc (all features) | `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --all-features` | rc **0** |
| MSRV test (no features) | `cargo +1.92 test --locked` | **99 / 0** |
| MSRV test (all features) | `cargo +1.92 test --locked --all-features` | **183 / 0** |
| cargo-deny | `cargo-deny --locked --all-features --manifest-path Cargo.toml check advisories bans licenses sources` | rc **0**, *advisories ok, bans ok, licenses ok, sources ok*, with exactly the two pre-existing `license-not-encountered` warnings at `deny.toml:35,36` (`BSD-2-Clause`, `BSD-3-Clause`) |
| cargo-machete | `cargo-machete` 0.9.2 | rc **0**, *"didn't find any unused dependencies"* |

`cargo-deny` is **0.20.2**, the pinned version, and is on this host's `PATH` at
`/home/andrej/.cargo/bin/cargo-deny`. `cargo-machete` is **not** on `PATH` and
is not a `cargo` subcommand here; I ran the pinned **0.9.2** binary left in this
slice's own scratch tree by an earlier attempt,
`/home/andrej/coord_scratch/riders67/machete/bin/cargo-machete`. I installed
nothing.

## Per-target cells, `cargo test --locked --all-features` (183)

lib 5 · command 23 · decoder 7 · frame_versions 11 · imagery 28 · imagery_cli 6
· **imagery_net 24** · modeldef_versions 12 · net_command 11 · net_data 6 ·
optional_features 4 · payload_robustness 6 · robustness 9 · roundtrip 17 ·
serverinfo_encoding 4 · doc-tests 10 = **183**.

`cargo test --locked` (99): 5 · 23 · 7 · 11 · 0 · 0 · **0** · 12 · 0 · 0 · 0 · 6
· 9 · 17 · 4 · 5 = **99**.

Against C3's 182: `imagery_net` moves **23 → 24**. One test was renamed
(`a_declared_length_costs_nothing_past_the_deadline` →
`the_worst_declared_length_past_the_deadline_stays_within_a_fixed_budget`) and
one added
(`a_record_too_long_for_the_receive_queue_is_not_completed_past_its_deadline`).
Every other cell is unchanged.

## features matrix — `clippy --all-targets --features F -- -D warnings`, `test --features F`

| features | clippy | test |
|---|---|---|
| `imagery` | rc 0, 0 warnings | **127 / 0** |
| `net` | rc 0, 0 warnings | **121 / 0** |
| `serde` | rc 0, 0 warnings | **101 / 0** |
| `nalgebra` | rc 0, 0 warnings | **101 / 0** |
| `imagery,net` | rc 0, 0 warnings | **179 / 0** |
| `serde,nalgebra` | rc 0, 0 warnings | **103 / 0** |

Five rows identical to C3; `imagery,net` moves **178 → 179**, the one added
test.

## feature-docs — all 16 rows, `cargo check --locked --all-targets --no-default-features [--features F]` and `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features [--features F]`

`default`, `imagery`, `net`, `serde`, `nalgebra`, `imagery-net`,
`imagery-serde`, `imagery-nalgebra`, `net-serde`, `net-nalgebra`,
`serde-nalgebra`, `imagery-net-serde`, `imagery-net-nalgebra`,
`imagery-serde-nalgebra`, `net-serde-nalgebra`, `all` — **check rc 0 and doc rc
0 for every one of the 16**, 0 warnings.

## Clean extraction

`git archive 495974a3…` into `/home/andrej/coord_scratch/riders67c4/extract`:
**14** files in `tests/`, **0** matching `zz_`; `src/imagery/net.rs` and
`tests/imagery_net.rs` `cmp`-identical to the worktree's;
`cargo test --locked --all-features` → **183 passed / 0 failed** over 16
targets.

## BASE comparison

The extracted C4 tree with `src/imagery/net.rs` taken from `2c3041f`
(`/home/andrej/coord_scratch/riders67c4/basecmp2`),
`cargo test --locked --all-features --no-fail-fast`: **180 passed / 3 failed**
over 16 targets. The three failures are exactly

- `a_peer_still_sending_past_the_deadline_is_a_timeout` (C3's axis),
- `the_worst_declared_length_past_the_deadline_stays_within_a_fixed_budget`,
- `a_record_too_long_for_the_receive_queue_is_not_completed_past_its_deadline`.

All four pre-existing lateness and delivery tests pass on both trees.

## Ablations — each new axis dies alone

From the clean extraction, one edit each, own `CARGO_TARGET_DIR`,
`--no-fail-fast`, `Compiling natnet` present in both:

| ablation | what it breaks | tests that fail |
|---|---|---|
| **A1** | `take_arrived` drains until `WouldBlock` again (the one-read bound removed) | exactly `a_peer_still_sending_past_the_deadline_is_a_timeout` **and** `a_record_too_long_for_the_receive_queue_is_not_completed_past_its_deadline` — **22 passed / 2 failed** in `imagery_net` |
| **A2** | the late-allocation branch removed, always `resize` (the allocation bound removed) | exactly `the_worst_declared_length_past_the_deadline_stays_within_a_fixed_budget` — **23 / 1** |

**The re-pointed cost test still isolates its own axis**: it survives A1 and
dies only under A2, and the new narrowing test does the reverse. That is the
check the old test could also have passed; what it could not do was fail on the
base at a length the base is actually slow at, which is the point of the
re-pointing.

## Flake

The full `imagery_net` target from the clean extraction, **20 runs `debug` and
20 runs `release`**: **40 of 40 at 24 passed / 0 failed**, 0.38 s and 0.33 s per
run.
