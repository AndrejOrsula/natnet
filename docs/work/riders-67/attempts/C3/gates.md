# riders-67 C3 — gates

Candidate `987f2ecc1d66a01ad80346efbf1a2bc8faf3ff90`, tree
`bf191d39d468431e985b55c228071a4890beb09c`, parent `2c3041f`.

Every row below was run by hand over this tree, in the order
`.github/workflows/rust.yml` declares, with
`CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c3/target` and the binary
named by absolute path `/home/andrej/.cargo/bin/cargo`. Raw output:
`/home/andrej/coord_scratch/riders67c3/{gates,features,featuredocs}.log`.

- stable `rustc 1.97.1 (8bab26f4f 2026-07-14)` / `cargo 1.97.1 (c980f4866 2026-06-30)`, selected by `rust-toolchain.toml` (`channel = "stable"`)
- MSRV `rustc 1.92.0 (ded5c06cf 2025-12-08)` / `cargo 1.92.0 (344c4567c 2025-10-21)`, selected by an explicit `+1.92`

## job `gate`

| # | command | rc | numbers |
|---|---|---|---|
| 1 | `cargo fmt --all --check` | **0** | no diff |
| 2 | `cargo clippy --locked --all-targets --all-features -- -D warnings` | **0** | **0 warnings** |
| 3 | `cargo test --locked --all-features` | **0** | **182 passed / 0 failed** over 16 targets |
| 4 | `cargo test --locked` | **0** | **99 / 0** over 16 targets |
| 5 | `cargo build --locked --features net` | **0** | — |
| 6 | `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps` | **0** | 0 warnings |
| 7 | `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --all-features` | **0** | 0 warnings |
| 8 | `cargo build --locked --features imagery,net --example imagery_dump` | **0** | — |
| 9 | `cargo +1.92 test --locked` | **0** | **99 / 0** |
| 10 | `cargo +1.92 test --locked --all-features` | **0** | **182 / 0** |

### row 3, cell by cell

| target | passed | failed |
|---|---|---|
| `unittests src/lib.rs` | 5 | 0 |
| `tests/command.rs` | 23 | 0 |
| `tests/decoder.rs` | 7 | 0 |
| `tests/frame_versions.rs` | 11 | 0 |
| `tests/imagery.rs` | 28 | 0 |
| `tests/imagery_cli.rs` | 6 | 0 |
| **`tests/imagery_net.rs`** | **23** | 0 |
| `tests/modeldef_versions.rs` | 12 | 0 |
| `tests/net_command.rs` | 11 | 0 |
| `tests/net_data.rs` | 6 | 0 |
| `tests/optional_features.rs` | 4 | 0 |
| `tests/payload_robustness.rs` | 6 | 0 |
| `tests/robustness.rs` | 9 | 0 |
| `tests/roundtrip.rs` | 17 | 0 |
| `tests/serverinfo_encoding.rs` | 4 | 0 |
| `Doc-tests natnet` | 10 | 0 |
| **total** | **182** | **0** |

5+23+7+11+28+6+23+12+11+6+4+6+9+17+4+10 = 182. Against C2's re-derived table
the only cell that moves is `imagery_net`, 21 → 23: the two tests added here.

## job `features` — all six rows

| features | `cargo clippy --locked --all-targets --features X -- -D warnings` | `cargo test --locked --features X` |
|---|---|---|
| `imagery` | rc **0** | rc 0, **127 / 0** |
| `net` | rc **0** | rc 0, **121 / 0** |
| `serde` | rc **0** | rc 0, **101 / 0** |
| `nalgebra` | rc **0** | rc 0, **101 / 0** |
| `imagery,net` | rc **0** | rc 0, **178 / 0** |
| `serde,nalgebra` | rc **0** | rc 0, **103 / 0** |

0 clippy warnings across the job. C2's row set was 127/121/101/101/**176**/103;
`imagery,net` moves by the same two tests.

## job `feature-docs` — all sixteen rows

`cargo check --locked --all-targets --no-default-features [--features X]` and
`RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features
[--features X]`. The `default` row takes the shell branch with no `--features`
argument at all, as the workflow does.

| row | features | check rc | doc rc |
|---|---|---|---|
| default | *(none)* | 0 | 0 |
| imagery | `imagery` | 0 | 0 |
| net | `net` | 0 | 0 |
| serde | `serde` | 0 | 0 |
| nalgebra | `nalgebra` | 0 | 0 |
| imagery-net | `imagery,net` | 0 | 0 |
| imagery-serde | `imagery,serde` | 0 | 0 |
| imagery-nalgebra | `imagery,nalgebra` | 0 | 0 |
| net-serde | `net,serde` | 0 | 0 |
| net-nalgebra | `net,nalgebra` | 0 | 0 |
| serde-nalgebra | `serde,nalgebra` | 0 | 0 |
| imagery-net-serde | `imagery,net,serde` | 0 | 0 |
| imagery-net-nalgebra | `imagery,net,nalgebra` | 0 | 0 |
| imagery-serde-nalgebra | `imagery,serde,nalgebra` | 0 | 0 |
| net-serde-nalgebra | `net,serde,nalgebra` | 0 | 0 |
| all | `imagery,net,serde,nalgebra` | 0 | 0 |

32 invocations, 32 × rc 0, 0 warnings.

## job `deny`

| step | result |
|---|---|
| `cargo-deny --version` | `cargo-deny 0.20.2` — the pinned `CARGO_DENY_VERSION` |
| `cargo-deny --locked --all-features --manifest-path Cargo.toml check advisories bans licenses sources` | rc **0**, `advisories ok, bans ok, licenses ok, sources ok` |
| warnings | exactly **2**, both pre-existing: `license-not-encountered` for `BSD-2-Clause` (`deny.toml:35`) and `BSD-3-Clause` (`deny.toml:36`) |
| `cargo-machete --version` | `0.9.2` — the pinned `CARGO_MACHETE_VERSION` |
| `cargo-machete` | rc **0**, *"didn't find any unused dependencies in this directory"* |

`cargo-machete` is **not** on this host's `PATH` and is **not** a `cargo`
subcommand here: `command -v cargo-machete` is empty and `cargo machete`
answers `error: no such command`. A bounded `find / -name cargo-machete -type f
-not -path '*/proc/*'` returned
`/home/andrej/coord_scratch/riders67/machete/bin/cargo-machete`, left by an
earlier attempt in this slice under a `cargo install --root` prefix — the
install shape neither `command -v` nor `ls ~/.cargo/bin` can see. That binary
was run. Nothing was installed.

## Beyond the workflow

| check | result |
|---|---|
| clean extraction (`git archive 987f2ec` into an empty dir, isolated target) | **14** files in `tests/`, **0** matching `zz_`; `cargo test --locked --all-features` **182 / 0**; extracted `src/imagery/net.rs` `cmp`-identical to the candidate blob |
| BASE comparison (candidate tree, `src/imagery/net.rs` from `2c3041f`) | **180 passed / 2 failed**; the two are exactly `a_peer_still_sending_past_the_deadline_is_a_timeout` and `a_declared_length_costs_nothing_past_the_deadline` |
| ablation **NULL** | 182 / 0, one `Compiling natnet` |
| ablation **A1** (drain restored) | 181 / 1 — `a_peer_still_sending_past_the_deadline_is_a_timeout` |
| ablation **A2** (late allocation removed) | 181 / 1 — `a_declared_length_costs_nothing_past_the_deadline` |
| ablation **A3** (loop top fails again) | 179 / 3 — the three delivery tests |
| ablation **A4** (blocking-mode restore deleted) | 181 / 1 — `a_record_finished_without_waiting_leaves_the_socket_blocking` |
| flake check, candidate `tests/imagery_net.rs` | 20 runs `debug` + 20 runs `release`, **40 / 40 at 23 passed / 0 failed** |
| refuted-citation sweep, `2c3041f 987f2ec`, 8 tokens, 3 legs | every hit read; no refuted sentence survives in the tree. `however` leg 2: no hit |

`passed + failed` is 182 in every ablation run and every ablation run shows a
`Compiling natnet` line.
