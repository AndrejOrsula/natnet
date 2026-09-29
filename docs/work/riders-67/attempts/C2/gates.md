# riders-67 C2 — gate evidence

Every command below was run in
`/home/andrej/ws_rs/natnet_wt/impl-riders-67-C2` at
`candidate(C2) = 2c3041f890481f2df8790de78a03258170149bd8`
(tree `54ea58d23c23ff01cdb5dd6202916636de044eb8`), with

```
export CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c2/target
CARGO=/home/andrej/.cargo/bin/cargo     # absolute path; no shell rewriting
```

`git status --porcelain --untracked-files=no` was empty for every run (the
only untracked path is `docs/`, which is this packet). The scratch instrument
`tests/zz_c2_probe.rs` was moved out of the worktree to
`/home/andrej/coord_scratch/riders67c2/zz_c2_probe.rs` **before** these runs;
`git ls-tree` on the frozen tree shows 14 files in `tests/` and none matching
`zz_`.

The gate list is `natnet`'s own, from `.github/workflows/rust.yml` — jobs
`gate`, `features`, `feature-docs`, `deny`. It is **not** `auga`'s
`AGENTS.md` §6 list, which names workspace packages and scripts that do not
exist in this repository.

Toolchains: `rustc 1.97.1 (8bab26f4f 2026-07-14)` / `cargo 1.97.1 (c980f4866
2026-06-30)`, selected by `rust-toolchain.toml` (`channel = "stable"`); MSRV
rows on `cargo 1.92.0 (344c4567c 2025-10-21)` via the explicit `+1.92`
selector, which `rust-toolchain.toml` cannot override.

## job `gate`

| # | command | result |
|---|---|---|
| 1 | `cargo fmt --all --check` | rc **0** |
| 2 | `cargo clippy --locked --all-targets --all-features -- -D warnings` | rc **0**, **0** lines matching `^warning` |
| 3 | `cargo test --locked --all-features` | rc **0**, **16** targets, **180 passed, 0 failed**, 0 ignored |
| 4 | `cargo test --locked` (default features) | rc **0**, **16** targets, **99 passed, 0 failed** |
| 5 | `cargo build --locked --features net` | rc **0** |
| 6 | `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps` | rc **0** |
| 7 | `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features` | rc **0** |
| 8 | `cargo build --locked --features imagery,net --example imagery_dump` | rc **0** |
| 9 | `cargo +1.92 test --locked` | rc **0**, 16 targets, **99 passed, 0 failed** |
| 10 | `cargo +1.92 test --locked --all-features` | rc **0**, 16 targets, **180 passed, 0 failed** |

Per-target detail for #3, read off the run rather than retyped:

```
src/lib.rs               5     modeldef_versions.rs   12
command.rs              23     net_command.rs         11
decoder.rs               7     net_data.rs             6
frame_versions.rs       11     optional_features.rs    4
imagery.rs              28     payload_robustness.rs   6
imagery_cli.rs           6     robustness.rs           9
imagery_net.rs          21     roundtrip.rs           17
                               serverinfo_encoding.rs  4
                               doctests               10
                                              sum = 180
```

**180 = 177 + 3.** C1 shipped 177 with `imagery_net` at 18; this attempt adds
three tests to `imagery_net` (21) and changes no other target's count. Every
other cell is identical to C1's table.

## job `features` — all 6 rows

Each row is `cargo clippy --locked --all-targets --features <row> -- -D warnings`
followed by `cargo test --locked --features <row>`.

| row | clippy rc | test rc | tests passed | failed | vs C1 |
|---|---|---|---|---|---|
| `imagery` | 0 | 0 | 127 | 0 | = |
| `net` | 0 | 0 | 121 | 0 | = |
| `serde` | 0 | 0 | 101 | 0 | = |
| `nalgebra` | 0 | 0 | 101 | 0 | = |
| `imagery,net` | 0 | 0 | **176** | 0 | 173 + 3 |
| `serde,nalgebra` | 0 | 0 | 103 | 0 | = |

`imagery,net` is the only row that moves, and it moves by exactly the three
new tests — which is the right answer, since `tests/imagery_net.rs` carries
`#![cfg(all(feature = "imagery", feature = "net"))]`. The totals differ by row
because `#![cfg(...)]` gates whole test files; they are not comparable to each
other and are not a truncation signal.

## job `feature-docs` — all 16 rows

Each row is
`cargo check --locked --all-targets --no-default-features [--features <row>]`
then `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --no-default-features [--features <row>]`.
The empty row omits `--features` entirely rather than passing an empty value.

```
row='default'                      check_rc=0  doc_rc=0
row='imagery'                      check_rc=0  doc_rc=0
row='net'                          check_rc=0  doc_rc=0
row='serde'                        check_rc=0  doc_rc=0
row='nalgebra'                     check_rc=0  doc_rc=0
row='imagery,net'                  check_rc=0  doc_rc=0
row='imagery,serde'                check_rc=0  doc_rc=0
row='imagery,nalgebra'             check_rc=0  doc_rc=0
row='net,serde'                    check_rc=0  doc_rc=0
row='net,nalgebra'                 check_rc=0  doc_rc=0
row='serde,nalgebra'               check_rc=0  doc_rc=0
row='imagery,net,serde'            check_rc=0  doc_rc=0
row='imagery,net,nalgebra'         check_rc=0  doc_rc=0
row='imagery,serde,nalgebra'       check_rc=0  doc_rc=0
row='net,serde,nalgebra'           check_rc=0  doc_rc=0
row='imagery,net,serde,nalgebra'   check_rc=0  doc_rc=0
```

All 16, matching the workflow's 16-row `include:` list. This slice rewrites
rendered rustdoc (`recv_record`'s doc comment and the module `//!`), so these
rows are load-bearing here rather than precautionary.

## job `deny`

The change touches no manifest: `git status --porcelain Cargo.toml Cargo.lock
deny.toml` returns **0** lines. Run anyway.

```
cargo-deny 0.20.2   (/home/andrej/.cargo/bin/cargo-deny)
$ cargo-deny --locked --all-features --manifest-path Cargo.toml check advisories bans licenses sources
warning[license-not-encountered]: license was not encountered
   ┌─ deny.toml:35:6
35 │     "BSD-2-Clause",   # https://spdx.org/licenses/BSD-2-Clause.html
   │      ━━━━━━━━━━━━ unmatched license allowance
warning[license-not-encountered]: license was not encountered
   ┌─ deny.toml:36:6
36 │     "BSD-3-Clause",   # https://spdx.org/licenses/BSD-3-Clause.html
   │      ━━━━━━━━━━━━ unmatched license allowance
advisories ok, bans ok, licenses ok, sources ok
rc=0
```

**2** warnings, both pre-existing `deny.toml` allowances for licences no
dependency uses — the same two C1 reported and the reviewer re-derived. Not
introduced by this slice; the run exits 0.

```
cargo-machete 0.9.2   (/home/andrej/coord_scratch/riders67/machete/bin/cargo-machete)
$ cargo-machete
Analyzing dependencies of crates in this directory...
cargo-machete didn't find any unused dependencies in this directory. Good job!
rc=0
```

`cargo-machete` is the 0.9.2 C1 installed outside the repository and outside
`~/.cargo/bin`; nothing was installed by this attempt. `cargo-deny` **is** in
`~/.cargo/bin`, which is worth recording because my first search for it —
`find /home/andrej/coord_scratch -name cargo-deny` — returned nothing and
would have supported a false "not available on this host". The absence
instrument has to match the install shape.

## Discrimination runs — base and three ablations, NULL controls interleaved

Not a gate; recorded here because the numbers belong with the command
evidence. Method and interpretation are in `report.md` §4 and §6.

Command for all six rows: `cargo test --locked --all-features --no-fail-fast`,
each tree in its own `CARGO_TARGET_DIR`.

```
run              source state                                rc    passed failed total  Compiling natnet
NULL control A   candidate 2c3041f                            0     180     0    180    0 (warm)
BASE             src/imagery/net.rs from e157774, C2 tests  101     177     3    180    1
ABL-A1           deadline check back in front of the read   101     177     3    180    1
ABL-A2           WouldBlock past the deadline = complete    101     177     3    180    1
ABL-A3           blocking mode never restored               101     179     1    180    1
NULL control B   candidate 2c3041f restored                   0     180     0    180    0 (warm)
```

`pass+fail` is **180 in every row**, so no run measured a different suite from
its controls. Each non-control row shows a `Compiling natnet` line, so the
build was invalidated rather than reused — `AGENTS.md` §6's
*"Byte-identical source is not sufficient; the build must be invalidated
too"*. Which tests failed, per row:

```
BASE    / ABL-A1  a_fully_buffered_bodied_record_is_delivered_past_its_deadline
                  a_fully_buffered_record_is_delivered_when_its_deadline_expires_mid_read
                  a_record_finished_without_waiting_leaves_the_socket_blocking
ABL-A2            an_incomplete_record_past_its_deadline_still_fails_and_is_terminal
                  a_record_missing_payload_bytes_past_its_deadline_still_fails_and_is_terminal
                  byte_drip_cannot_restart_the_total_record_deadline
ABL-A3            a_record_finished_without_waiting_leaves_the_socket_blocking
```

BASE and ABL-A1 fail the identical three, which is the check on A1: it was
built by hand to reproduce `e157774`'s predicate inside C2's structure, and
it does.

The BASE row is the required *fails on `e157774`, passes on the candidate*
evidence. Its `src/imagery/net.rs` was confirmed byte-identical to
`git show e157774:src/imagery/net.rs` with `cmp`, and its
`tests/imagery_net.rs` byte-identical to the frozen
`git show 54ea58d:tests/imagery_net.rs`, before the run.

## Clean-extraction confirmation

```
git archive 54ea58d23c23ff01cdb5dd6202916636de044eb8 | tar -x -C <tmp>
  → tests/ holds 14 files, 0 matching zz_
CARGO_TARGET_DIR=<fresh> cargo test --locked --all-features --no-fail-fast
  → rc 0, 16 targets, 180 passed, 0 failed
```

`git archive` was given the **tree** object, not a commit-ish, so the
extraction stamps the extraction instant and the `AGENTS.md` §6 stale-mtime
hazard cannot apply; the fresh target directory makes the point moot in any
case. The frozen tree builds and passes on its own, with no dependence on any
untracked file in the worktree — in particular none on the scratch probe.
