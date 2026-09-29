# riders-67 C6 — gates

**Tree under test: `e5fc608c2d999ad5d4dfe32102eb12674c7ae48b`**
(`candidate(C6)` = `b03817b271ba0c9761577bbdeeeeebe44efd675b`).

**Two runs, both reported.** The first ran against tree `d45a3a4e` (the first
of this attempt's two commits) at loadavg 9.19 → 12.12; I then changed one
sentence of `README.md` and ran the whole suite again against the shipped tree
`e5fc608c` at loadavg 8.70 → 8.83. **Every row is identical between the two
runs**, including all 32 per-target cells. The numbers below are the second
run's. Nothing was re-run to make a row green — no row was ever not green.

- Script: `/home/andrej/coord_scratch/riders67c6/gates.sh` — C5's `gates.sh`
  with `riders-67-C5` → `riders-67-C6` and `riders67c5` → `riders67c6`
  substituted, nothing else changed.
- Raw logs: `/home/andrej/coord_scratch/riders67c6/gates.log` (run 1, tree
  `d45a3a4e`) and `gates2.log` (run 2, shipped tree `e5fc608c`); per-gate test
  output in `g_test_all.txt`, `g_test_none.txt`, `g_msrv_*.txt`, `g_feat_*.txt`
  (overwritten by run 2).
- Working directory `/home/andrej/ws_rs/natnet_wt/impl-riders-67-C6`,
  `CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c6/target`.
- **Load: `/proc/loadavg` 8.70 before the shipped run, 8.83 after**, 32 cores
  (run 1: 9.19 → 12.12, the run itself being most of that rise). No figure in
  this document depends on load; every row is a return code or a test count —
  which is why the two runs agree cell for cell across a 3.3-point difference
  in load.
- Rows are every step of `.github/workflows/rust.yml`: the `stable` job, the
  `features` matrix (6), the `feature docs` matrix (16), `deny`, `machete`.

## Toolchain

| | |
|---|---|
| cargo | `1.97.1 (c980f4866 2026-06-30)` |
| rustc | `1.97.1 (8bab26f4f 2026-07-14)`, LLVM 22.1.6 |
| host | `x86_64-unknown-linux-gnu`, Ubuntu 24.4.0 (noble) |
| glibc | `2.39-0ubuntu8.7` |
| MSRV rows | `cargo 1.92.0 (344c4567c 2025-10-21)` — the `rust-version = "1.92"` declared in `Cargo.toml`, invoked explicitly as `cargo +1.92` |

## Stable job

| gate | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | **rc 0** |
| clippy | `cargo clippy --locked --all-targets --all-features -- -D warnings` | **rc 0**, **0** lines matching `^(warning\|error)` |
| test (all features) | `cargo test --locked --all-features` | **183 passed / 0 failed** over **16** targets, rc 0 |
| test (no features) | `cargo test --locked` | **99 passed / 0 failed** over 16 targets, rc 0 |
| build (net) | `cargo build --locked --features net` | **rc 0** |
| build example | `cargo build --locked --features imagery,net --example imagery_dump` | **rc 0** |
| doc (no features) | `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps` | **rc 0** |
| doc (all features) | `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --all-features` | **rc 0** |
| MSRV test (no features) | `cargo +1.92 test --locked` | **99 / 0** over 16, rc 0 |
| MSRV test (all features) | `cargo +1.92 test --locked --all-features` | **183 / 0** over 16, rc 0 |

**Per-target cells, `--all-features`** (the order `cargo` ran them in): lib **5**
· command **23** · decoder **7** · frame_versions **11** · imagery **28** ·
imagery_cli **6** · **imagery_net 24** · modeldef_versions **12** ·
net_command **11** · net_data **6** · optional_features **4** ·
payload_robustness **6** · robustness **9** · roundtrip **17** ·
serverinfo_encoding **4** · doc-tests **10**. Sum 183.

**Per-target cells, no features**: 5 · 23 · 7 · 11 · 0 · 0 · **0** · 12 · 0 · 0
· 0 · 6 · 9 · 17 · 4 · 5 doc-tests. Sum 99.

Both sets are **identical, cell for cell, to C5's and C4's**. `imagery_net`
carries 24 tests in both this candidate and its base; no test was added,
removed, renamed or ignored.

## features matrix (6 rows)

| features | clippy | test |
|---|---|---|
| `imagery` | rc 0, **0** warnings | **127 / 0** |
| `net` | rc 0, **0** warnings | **121 / 0** |
| `serde` | rc 0, **0** warnings | **101 / 0** |
| `nalgebra` | rc 0, **0** warnings | **101 / 0** |
| `imagery,net` | rc 0, **0** warnings | **179 / 0** |
| `serde,nalgebra` | rc 0, **0** warnings | **103 / 0** |

## feature docs matrix (16 rows)

`cargo check --locked --all-targets --no-default-features [--features F]` and
`RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features
[--features F]`, for `default`, `imagery`, `net`, `serde`, `nalgebra`,
`imagery-net`, `imagery-serde`, `imagery-nalgebra`, `net-serde`,
`net-nalgebra`, `serde-nalgebra`, `imagery-net-serde`, `imagery-net-nalgebra`,
`imagery-serde-nalgebra`, `net-serde-nalgebra`, `all`.

**16 of 16 rows `check_rc=0 doc_rc=0`.** No row returned anything else. The new
`README.md` section adds no rustdoc surface: `README.md` is `Cargo.toml`'s
`readme`, it is not `include_str!`-ed anywhere in `src`, and both doc rows and
all 16 feature-doc rows are rc 0 with `-D warnings`.

## supply chain

| gate | result |
|---|---|
| `cargo-deny` **0.20.2** (`/home/andrej/.cargo/bin/cargo-deny`), `check advisories bans licenses sources` | **rc 0** — *advisories ok, bans ok, licenses ok, sources ok* |
| `cargo-machete` **0.9.2** | **rc 0** — *"didn't find any unused dependencies in this directory"* |

`cargo-deny` emits the two pre-existing `license-not-encountered` warnings,
located at **`deny.toml:35:6`** and **`deny.toml:36:6`** (`Unicode-3.0` and
`BSD-3-Clause` allowances that no dependency matches). Both are inherited
unchanged from C5 and C4; `deny.toml` was not touched. **Provenance caveat:**
`gates.sh` pipes `cargo-deny` through `tail -12`, so the log holds only the
last of the two warning blocks in full; I confirmed there are exactly two by
their two distinct `deny.toml` line numbers in the retained output, not by a
count over the full stream.

## outside the workflow, kept because C5 and the review reported them

| check | result |
|---|---|
| clean extraction | `git archive b03817b` → 14 files in `tests/`, **0** matching `zz_`, **no `docs/` in the archive**; `src/imagery/net.rs`, `tests/imagery_net.rs`, `README.md` all `cmp`-identical to the worktree |
| BASE comparison (`src/imagery/net.rs` from `2c3041f`) | `imagery_net` **21 passed / 3 failed**, failing exactly `a_peer_still_sending_past_the_deadline_is_a_timeout`, `the_worst_declared_length_past_the_deadline_stays_within_a_fixed_budget`, `a_record_too_long_for_the_receive_queue_is_not_completed_past_its_deadline` |
| comments-only property | both edited `.rs` files **byte-identical to `b482867`** once a Rust lexer strips comments and doc comments while preserving string and raw-string literals (md5 `6d8cad89…` and `4c6cd4cb…` on both sides) |
| orphan processes | none. `ps -eo pid,ppid,etimes,args \| grep -E '[i]magery_net\|[z]z_c6_probe'` empty at close |
| port pressure | TIME_WAIT **4** before the 800-run campaign, **9** after; 642 after the gate run, against the 32768–60999 ephemeral range |

**A1/A2 ablations were not re-run.** They ablate the allocation path and the
one-read bound; this candidate's code is byte-identical to `b482867` under the
lexer check above, and the review re-ran both against that code and reproduced
C4's cells (`imagery_net` 22/2 and 23/1, failing exactly the named tests). An
ablation of identical bytes cannot return a different answer.

## the one flake this tree is known to carry

`a_peer_still_sending_past_the_deadline_is_a_timeout` fails **3 of 400
sequential `release` runs and 2 of 400 sequential `debug` runs** on this tree,
at loadavg 8.94 → 9.32 with TIME_WAIT 4 → 9 — no concurrency and no port
pressure. It is **not fixed here** (`riders-76` owns it) and it did **not** fire
in the gate run above, which is one execution of each row. A gate run is a
sample of size one against a 1-in-160 event; the row `imagery_net 24 / 0` above
is true and is not evidence that the flake is gone.
