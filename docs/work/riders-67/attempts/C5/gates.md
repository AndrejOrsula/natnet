# riders-67 C5 — gates

Every cargo call ran from `/home/andrej/ws_rs/natnet_wt/impl-riders-67-C5` with
`CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c5/target` and
`/home/andrej/.cargo/bin/cargo`, except the extraction, comparison, ablation and
measurement runs, which used per-tree sibling target dirs (`target_extract`,
`target_basecmp`, `target_abl_A1`, `target_abl_A2`, `target_base`,
`target_cand`). Script `/home/andrej/coord_scratch/riders67c5/gates.sh`, raw log
`gates.log`, per-gate full outputs `g_test_all.txt`, `g_test_none.txt`,
`g_msrv_all.txt`, `g_msrv_none.txt`, `g_feat_*.txt`.

**Three runs of `gates.sh` exist and all three are reported.** The **first**
had one failure — `imagery_net` **23 passed / 1 failed** in the MSRV
all-features gate, name lost because the script filtered that gate to
`^test result`. The script was then instrumented to `tee` every `cargo test`
gate in full and to surface `failures:` blocks. The **second** run was green on
tree `2e8a4c1c`; the **third**, tabulated below, is green on the shipped tree
`783fdeae` — **no `FAILED` and no non-zero `rc` anywhere** in either.

**The failure has since been given two candidate names, both with their failure
output, in report §1** — `a_peer_still_sending_past_the_deadline_is_a_timeout`
(C3's axis test, 2/150 sequential in `release`, 2/40 at 4-way, and it produces
exactly `23 / 1`) and
`the_worst_declared_length_past_the_deadline_stays_within_a_fixed_budget` (the
cost test, 24/240 only under a port-exhausting 24-way run, 0/400 at eight-way
or less). **Read §1 rather than this table.** No budget, assertion or test was
changed in response to either.

Toolchain, from `cargo --version --verbose` / `rustc --version --verbose`:
`cargo 1.97.1 (c980f4866 2026-06-30)`, `rustc 1.97.1 (8bab26f4f 2026-07-14)`,
LLVM 22.1.6, host `x86_64-unknown-linux-gnu`, Ubuntu 24.4.0 (noble), selected by
`rust-toolchain.toml` (`channel = "stable"`). MSRV rows on
`cargo 1.92.0 (344c4567c 2025-10-21)`, an explicit `cargo +1.92`. glibc 2.39
(`ldd --version`: `ldd (Ubuntu GLIBC 2.39-0ubuntu8.7) 2.39`). Host load average
13–32 throughout — other work was running on this machine.

| gate | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | rc **0** |
| clippy | `cargo clippy --locked --all-targets --all-features -- -D warnings` | rc **0**, **0** warning-or-error lines |
| test (all features) | `cargo test --locked --all-features` | **183 passed / 0 failed** over **16** targets |
| test (no features) | `cargo test --locked` | **99 / 0** over 16 targets |
| build (net) | `cargo build --locked --features net` | rc **0** |
| build (example) | `cargo build --locked --features imagery,net --example imagery_dump` | rc **0** |
| doc (no features) | `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps` | rc **0** |
| doc (all features) | `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --all-features` | rc **0** |
| MSRV test (no features) | `cargo +1.92 test --locked` | **99 / 0** over 16 |
| MSRV test (all features) | `cargo +1.92 test --locked --all-features` | **183 / 0** over 16 |
| cargo-deny | `cargo-deny --locked --all-features --manifest-path Cargo.toml check advisories bans licenses sources` | rc **0**, *advisories ok, bans ok, licenses ok, sources ok*, with exactly the two pre-existing `license-not-encountered` warnings at `deny.toml:35,36` (`BSD-2-Clause`, `BSD-3-Clause`) |
| cargo-machete | `cargo-machete` 0.9.2 | rc **0**, *"didn't find any unused dependencies"* |

`cargo-deny` is **0.20.2**, the pinned version, on this host's `PATH` at
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

**Identical to C4 in every cell**, as it must be: C5 changes no test and no
code, only comments and doc comments.

## features matrix — `clippy --all-targets --features F -- -D warnings`, `test --features F`

| features | clippy | test |
|---|---|---|
| `imagery` | rc 0, 0 warnings | **127 / 0** |
| `net` | rc 0, 0 warnings | **121 / 0** |
| `serde` | rc 0, 0 warnings | **101 / 0** |
| `nalgebra` | rc 0, 0 warnings | **101 / 0** |
| `imagery,net` | rc 0, 0 warnings | **179 / 0** |
| `serde,nalgebra` | rc 0, 0 warnings | **103 / 0** |

All six identical to C4.

## feature-docs — all 16 rows

`cargo check --locked --all-targets --no-default-features [--features F]` and
`RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features
[--features F]`, for `default`, `imagery`, `net`, `serde`, `nalgebra`,
`imagery-net`, `imagery-serde`, `imagery-nalgebra`, `net-serde`,
`net-nalgebra`, `serde-nalgebra`, `imagery-net-serde`, `imagery-net-nalgebra`,
`imagery-serde-nalgebra`, `net-serde-nalgebra`, `all` — **check rc 0 and doc
rc 0 for every one of the 16**, 0 warnings. This gate matters more than usual
this attempt, because the change is rustdoc: the new `[`MAX_RECORD_LEN`]`
intra-doc link in the module docs resolves under every feature combination that
compiles the module.

## Clean extraction

`git archive 783fdeaea09314025f7c4161408a106b0463ae3a` into
`/home/andrej/coord_scratch/riders67c5/extract`: **14** files in `tests/`,
**0** matching `zz_`, and **no `docs/` directory in the archive at all**;
`src/imagery/net.rs` and `tests/imagery_net.rs` `cmp`-identical to the
worktree's; `cargo test --locked --all-features` → **183 passed / 0 failed**
over 16 targets.

## BASE comparison

The extracted C5 tree with `src/imagery/net.rs` taken from `2c3041f`
(`/home/andrej/coord_scratch/riders67c5/basecmp`),
`cargo test --locked --all-features --no-fail-fast`: **180 passed / 3 failed**
over 16 targets (`imagery_net` 21 / 3). The three failures are exactly

- `a_peer_still_sending_past_the_deadline_is_a_timeout` (C3's axis),
- `the_worst_declared_length_past_the_deadline_stays_within_a_fixed_budget`,
- `a_record_too_long_for_the_receive_queue_is_not_completed_past_its_deadline`.

## Ablations — each axis still dies alone

From the clean extraction, one edit each, own `CARGO_TARGET_DIR`,
`--no-fail-fast`:

| ablation | the edit | tests that fail |
|---|---|---|
| **A1** | in `take_arrived`, the `Ok(count)` arm loops again instead of breaking with `TimedOut` on a short read — the one-read bound removed | exactly `a_peer_still_sending_past_the_deadline_is_a_timeout` **and** `a_record_too_long_for_the_receive_queue_is_not_completed_past_its_deadline` — **22 passed / 2 failed** in `imagery_net` |
| **A2** | the `if Instant::now() < deadline` branch removed, always `resize` — the allocation bound removed | exactly `the_worst_declared_length_past_the_deadline_stays_within_a_fixed_budget` — **23 / 1** |

The cost test survives A1 and dies under A2; the narrowing test does the
reverse. Both reproduce C4's ablations cell for cell, which is the expected
result for an attempt that changes no code — and is the check that the comment
rewrite did not silently disturb the file.

## Flake

The full `imagery_net` target from the clean extraction, **20 runs `debug` and
20 runs `release`**: **40 of 40 at 24 passed / 0 failed**. Under **12-way
concurrency**, three rounds of twelve: **36 of 36**. Later batches, after the
ephemeral port range had been allowed to drain: **40/40 `debug` sequential**,
**40/40 `release` sequential**, **38/40 at 4-way**, **40/40 at 8-way**.

The two failures at 4-way are `a_peer_still_sending_past_the_deadline_is_a_timeout`
and are report §1.2; the cost test's 24 failures are §1.3 and came only from a
24-way run that exhausted this host's ephemeral ports. **The green counts here
are not the result — §1 is.**

## Measurement probes — none of them ship

`zz_c5_max.rs` (one declared length per process; min/p50/p90/p99/max and every
raw cell) and `zz_c5_ladder.rs` (the shipped cost test's exact shape, reporting
each cell's max as well as its median), both in
`/home/andrej/coord_scratch/riders67c5/` and copied into `.../cand/tests/` and
`.../base/tests/` only. The clean extraction has **14** files in `tests/` and
**0** matching `zz_`. Measurement logs: `tail_release.log`, `paired_release.log`,
`paired31_release.log`, `raw31.log`, `ladder_release.log`, `debug_sweep.log`.
