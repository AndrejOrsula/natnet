# review-fixes-2026-09-28 C1 — gates

Uncommitted worktree `/home/andrej/ws_rs/natnet` on base `dev`
`88837ef24765743d122cf496cc1d7610f7883076`. Every command ran with
`CARGO_TARGET_DIR=/tmp/impl_natnet` from the gate script
`/tmp/rf_gates/run.sh` (per-gate logs `/tmp/rf_gates/<n>.log`, table
`/tmp/rf_gates/results.tsv`). Linux x86-64 only; no Windows, Motive, hardware
or benchmark run.

## Builder gate run (final tree)

| # | command | exit |
|---|---|---|
| 1 | `rtk cargo fmt --all --check` | 0 |
| 2 | `rtk proxy cargo clippy --locked --all-targets -- -D warnings` | 0 |
| 3 | `rtk proxy cargo clippy --locked --all-targets --all-features -- -D warnings` | 0 |
| 4 | `rtk proxy cargo clippy --locked --all-targets --features imagery -- -D warnings` | 0 |
| 5 | `rtk proxy cargo clippy --locked --all-targets --features net -- -D warnings` | 0 |
| 6 | `rtk proxy cargo clippy --locked --all-targets --features serde -- -D warnings` | 0 |
| 7 | `rtk proxy cargo clippy --locked --all-targets --features nalgebra -- -D warnings` | 0 |
| 8 | `rtk proxy cargo clippy --locked --all-targets --features imagery,net -- -D warnings` | 0 |
| 9 | `rtk proxy cargo clippy --locked --all-targets --features serde,nalgebra -- -D warnings` | 0 |
| 10 | `rtk proxy cargo test --locked` | 0 |
| 11 | `rtk proxy cargo test --locked --all-features` | 0 |
| 12 | `rtk proxy cargo test --locked --features imagery` | 0 |
| 13 | `rtk proxy cargo test --locked --features net` | 0 |
| 14 | `rtk proxy cargo test --locked --features serde` | 0 |
| 15 | `rtk proxy cargo test --locked --features nalgebra` | 0 |
| 16 | `rtk proxy cargo test --locked --features imagery,net` | 0 |
| 17 | `rtk proxy cargo test --locked --features serde,nalgebra` | 0 |
| 18 | `rtk proxy cargo build --locked --features net` | 0 |
| 19 | `rtk proxy cargo build --locked --features imagery,net --example imagery_dump` | 0 |
| 20 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps` | 0 |
| 21 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --all-features` | 0 |
| 22 | `rtk proxy cargo check --locked --all-targets --no-default-features` | 0 |
| 23 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features` | 0 |
| 24 | `rtk proxy cargo check --locked --all-targets --no-default-features --features imagery` | 0 |
| 25 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features --features imagery` | 0 |
| 26 | `rtk proxy cargo check --locked --all-targets --no-default-features --features net` | 0 |
| 27 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features --features net` | 0 |
| 28 | `rtk proxy cargo check --locked --all-targets --no-default-features --features serde` | 0 |
| 29 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features --features serde` | 0 |
| 30 | `rtk proxy cargo check --locked --all-targets --no-default-features --features nalgebra` | 0 |
| 31 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features --features nalgebra` | 0 |
| 32 | `rtk proxy cargo check --locked --all-targets --no-default-features --features imagery,net` | 0 |
| 33 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features --features imagery,net` | 0 |
| 34 | `rtk proxy cargo check --locked --all-targets --no-default-features --features imagery,serde` | 0 |
| 35 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features --features imagery,serde` | 0 |
| 36 | `rtk proxy cargo check --locked --all-targets --no-default-features --features imagery,nalgebra` | 0 |
| 37 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features --features imagery,nalgebra` | 0 |
| 38 | `rtk proxy cargo check --locked --all-targets --no-default-features --features net,serde` | 0 |
| 39 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features --features net,serde` | 0 |
| 40 | `rtk proxy cargo check --locked --all-targets --no-default-features --features net,nalgebra` | 0 |
| 41 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features --features net,nalgebra` | 0 |
| 42 | `rtk proxy cargo check --locked --all-targets --no-default-features --features serde,nalgebra` | 0 |
| 43 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features --features serde,nalgebra` | 0 |
| 44 | `rtk proxy cargo check --locked --all-targets --no-default-features --features imagery,net,serde` | 0 |
| 45 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features --features imagery,net,serde` | 0 |
| 46 | `rtk proxy cargo check --locked --all-targets --no-default-features --features imagery,net,nalgebra` | 0 |
| 47 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features --features imagery,net,nalgebra` | 0 |
| 48 | `rtk proxy cargo check --locked --all-targets --no-default-features --features imagery,serde,nalgebra` | 0 |
| 49 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features --features imagery,serde,nalgebra` | 0 |
| 50 | `rtk proxy cargo check --locked --all-targets --no-default-features --features net,serde,nalgebra` | 0 |
| 51 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features --features net,serde,nalgebra` | 0 |
| 52 | `rtk proxy cargo check --locked --all-targets --no-default-features --features imagery,net,serde,nalgebra` | 0 |
| 53 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --no-default-features --features imagery,net,serde,nalgebra` | 0 |
| 54 | `rtk proxy cargo +1.92 build --locked --all-targets --all-features` | 0 |
| 55 | `rtk proxy cargo +1.92 test --locked` | 0 |
| 56 | `rtk proxy cargo +1.92 test --locked --all-features` | 0 |
| 57 | `rtk proxy cargo check --locked --all-targets --all-features --target i686-unknown-linux-gnu` | 0 |
| 58 | `rtk proxy cargo-deny --locked --all-features --manifest-path Cargo.toml check advisories bans licenses sources` | 0 |
| 59 | `rtk proxy cargo-machete` | 0 |
| 60 | `rtk proxy git diff --check` | 0 |

Test counts: default 109 passed, all features 201 passed, `imagery,net` 197
passed; MSRV 1.92 default 109 and all features 201 passed. cargo-deny
reported advisories, bans, licenses and sources ok; cargo-machete found no
unused dependency.

## Fail-first evidence

As reported by the builder: each regression test failed before its fix (exit 101, or a compile error for
the new accessor): the late command reply answered the next request ("A" for
B); an id-100 reply waited for `TimedOut`; a bare 12-byte imagery prefix grew
the buffer to 67 108 864 bytes; negative counts gave `ImplausibleCount`;
oversized frame / ModelDef / `Other` payloads encoded `Ok`; `imagery_cli` run
alone with the example deleted failed with `NotFound`. The exact exit codes
of the A1/A2/A4 fail-first runs were not retained; only the failures are.

## Integrator re-run and consumer builds (2026-09-28)

| command | exit | result |
|---|---|---|
| `rtk cargo fmt --all --check` | 0 | clean |
| `rtk cargo clippy --locked --all-targets --all-features -- -D warnings` | 0 | no issues |
| `rtk cargo test --locked` | 0 | 109 passed |
| `rtk cargo test --locked --all-features` | 0 | 201 passed |
| `RUSTDOCFLAGS="-D warnings" rtk cargo doc --locked --no-deps --all-features` | 0 | clean |
| `rtk git diff --check` | 0 | clean |
| motive-bridge (path dependency on this tree): `rtk cargo clippy --locked --all-targets -- -D warnings`, the same for `--target x86_64-pc-windows-msvc`, `rtk cargo test --locked`, `cargo +1.92 build --locked --all-targets` | 0 each | 151 tests passed |
| natnet_zenoh with `--config 'patch."https://github.com/AndrejOrsula/natnet.git".natnet.path="../natnet"'`: `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings` | 0 each | after adapting its synthetic size check to RF3; `Cargo.lock` restored afterwards |

Integrator re-runs used `CARGO_TARGET_DIR=/tmp/int_natnet` (natnet) and
`/tmp/int_mb` (motive-bridge); natnet_zenoh used its own `target/`.
