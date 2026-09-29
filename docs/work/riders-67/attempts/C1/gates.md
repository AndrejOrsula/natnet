# riders-67 C1 — gate evidence

Every command below was run in
`/home/andrej/ws_rs/natnet_wt/impl-riders-67` at
`candidate(C1) = e157774c85a425a84eee00fff188d739bcf64c39`
(tree `1a1c9bc69d344fe4f8c85b70217a687320eb2893`), with

```
export CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67/target
CARGO=/home/andrej/.cargo/bin/cargo     # absolute path; no shell rewriting
```

`git status --porcelain` was empty of tracked changes for every run.

The gate list is `natnet`'s own, from `.github/workflows/rust.yml` — jobs
`gate`, `features`, `feature-docs`, `deny`. It is **not** `auga`'s
`AGENTS.md` §6 list, which names workspace packages and scripts that do not
exist in this repository.

Toolchains: `rustc 1.97.1 (8bab26f4f 2026-07-14)` / `cargo 1.97.1` (selected by
`rust-toolchain.toml`, `channel = "stable"`); MSRV rows on
`cargo 1.92.0 (344c4567c 2025-10-21)` via the explicit `+1.92` selector, which
`rust-toolchain.toml` cannot override.

## job `gate`

| # | command | result |
|---|---|---|
| 1 | `cargo fmt --all --check` | rc **0** (see note) |
| 2 | `cargo clippy --locked --all-targets --all-features -- -D warnings` | rc **0**, `Finished dev profile in 6.66s`, 0 warnings |
| 3 | `cargo test --locked --all-features` | rc **0**, 16 targets, **177 passed, 0 failed**, 0 ignored |
| 4 | `cargo test --locked` (default features) | rc **0**, 16 targets, **99 passed, 0 failed**, 0 ignored |
| 5 | `cargo build --locked --features net` | rc **0**, `Finished dev profile in 0.35s` |
| 6 | `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps` | rc **0** |
| 7 | `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features` | rc **0** |
| 8 | `cargo build --locked --features imagery,net --example imagery_dump` | rc **0** |
| 9 | `cargo +1.92 test --locked` | rc **0**, 16 targets, **99 passed, 0 failed** |
| 10 | `cargo +1.92 test --locked --all-features` | rc **0**, 16 targets, **177 passed, 0 failed** |

Note on #1: the first invocation exited **1** on the new test helper —
rustfmt wanted `server_stream.write_all(bytes)` broken across three lines.
`cargo fmt --all` was applied and the check re-run to rc **0**; every gate in
this document, including #2 and #3, ran after that edit, and the formatted
text is what is in the candidate tree.

Per-target detail for #3 (`cargo test --locked --all-features`), 177 = 5 lib +
23 command + 7 decoder + 11 frame_versions + 28 imagery + 6 imagery_cli +
**18 imagery_net** + 12 modeldef_versions + 11 net_command + 6 net_data +
4 optional_features + 6 payload_robustness + 9 robustness + 17 roundtrip +
4 serverinfo_encoding + 10 doctests. `imagery_net` is 16 before this slice
plus the 2 new tests.

## job `features` — all 6 rows

Each row is `cargo clippy --locked --all-targets --features <row> -- -D warnings`
followed by `cargo test --locked --features <row>`.

| row | clippy rc | test rc | tests passed |
|---|---|---|---|
| `imagery` | 0 | 0 | 127 |
| `net` | 0 | 0 | 121 |
| `serde` | 0 | 0 | 101 |
| `nalgebra` | 0 | 0 | 101 |
| `imagery,net` | 0 | 0 | 173 |
| `serde,nalgebra` | 0 | 0 | 103 |

0 failed on every row. The totals differ by row because `#![cfg(...)]` gates
whole test files; they are not comparable to each other and are not a
truncation signal.

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

The brief allowed running only the rows the change can reach — the 4 rows
containing both `imagery` and `net`. All 16 were run instead.

## job `deny`

The change touches no manifest: `git status --porcelain Cargo.toml Cargo.lock
deny.toml` is empty. Run anyway.

```
cargo-deny 0.20.2
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

Two warnings, both pre-existing `deny.toml` allowances for licences no
dependency uses. Not introduced by this slice; the run exits 0.

```
cargo-machete 0.9.2
$ cargo-machete
Analyzing dependencies of crates in this directory...
cargo-machete didn't find any unused dependencies in this directory. Good job!
rc=0
```

`cargo-machete` was absent from this host; 0.9.2 was installed to
`/home/andrej/coord_scratch/riders67/machete` (outside the repository and
outside `~/.cargo/bin`) and invoked by absolute path.

## Mutation runs — `deadline-midread-off`

Not a gate; recorded here because the numbers belong with the command
evidence. Method and interpretation are in `report.md` §5.

```
NULL control A  candidate                  rc=0    passed=177 failed=0 total=177   Compiling natnet: 0 (warm)
mutant          mid-read check reinstated  rc=101  passed=176 failed=1 total=177   Compiling natnet: 1
NULL control B  candidate restored         rc=0    passed=177 failed=0 total=177   Compiling natnet: 1
```

Command for all three: `cargo test --locked --all-features --no-fail-fast`.
Single failure: `a_fully_buffered_record_is_delivered_when_its_deadline_expires_mid_read`.
Equal `pass+fail` totals across all three runs, so no run measured a different
suite than its controls.

Scout's survival claim re-derived separately, with the check disabled and the
**pre-existing** suite only (new tests reverted out of the tree):
`cargo test --locked --all-features` → 16 targets, **175 passed, 0 failed**.

Single-test repetition on the candidate, 40 consecutive invocations of
`cargo test --locked --features imagery,net --test imagery_net -- --exact
a_fully_buffered_record_is_delivered_when_its_deadline_expires_mid_read`:
**40 passed / 0 failed**. The same 20 invocations with the check present:
**20 failed / 0 passed**.

## Clean-extraction confirmation

The rows above ran in the implementation worktree, whose tracked content is
byte-identical to the candidate tree. As an independent check the tree object
itself was extracted and built from scratch in a separate target directory:

```
git archive 1a1c9bc69d344fe4f8c85b70217a687320eb2893 | tar -x -C <tmp>
CARGO_TARGET_DIR=<fresh> cargo test --locked --all-features --no-fail-fast
  → rc 0, 177 passed, 0 failed
```

`git archive` was given the **tree** object, not a commit-ish, so the
extraction stamps the extraction instant and the `AGENTS.md` §6 stale-mtime
hazard cannot apply; the fresh target directory makes the point moot in any
case. This confirms the frozen tree builds and passes on its own, with no
dependence on any untracked file in the worktree.
