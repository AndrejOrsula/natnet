# connect-options-2026-09-28 C1 — gates

Uncommitted worktree `/home/andrej/ws_rs/natnet` on `dev`
`50d8da207bdf73dcadb05d3ba262d05f4d46b57d`, `CARGO_TARGET_DIR=/tmp/int_natnet`,
script `/tmp/natnet_co_gates.sh`, logs `/tmp/natnet_co/<n>.log`. Linux x86-64
only; no Windows, Motive, hardware or benchmark run.

Fail-first: with only the tests added, `cargo test --all-features --test
command --test net_command` failed to compile (E0432 `ConnectOptions`, E0425
`connect_request_with` and `CONNECT_OPTIONS_PAYLOAD_LEN`, E0599
`connect_handshake_with`). After the change both suites passed (25 and 16).

| # | command | exit |
|---|---|---|
| 1 | `rtk cargo fmt --all --check` | 0 |
| 2 | `rtk proxy cargo clippy --locked --all-targets -- -D warnings` | 0 |
| 3 | `rtk proxy cargo clippy --locked --all-targets --all-features -- -D warnings` | 0 |
| 4 | `rtk proxy cargo test --locked` | 0 |
| 5 | `rtk proxy cargo test --locked --all-features` | 0 |
| 6 | `rtk proxy cargo test --locked --no-default-features --features net` | 0 |
| 7 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps` | 0 |
| 8 | `rtk proxy env RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps --all-features` | 0 |
| 9 | `rtk proxy cargo check --locked --all-targets --no-default-features` | 0 |
| 10 | `rtk proxy cargo +1.92 test --locked --all-features` | 0 |
| 11 | `rtk proxy cargo check --locked --all-targets --all-features --target i686-unknown-linux-gnu` | 0 |
| 12 | `rtk proxy cargo-machete` | 0 |
| 13 | `rtk proxy git diff --check` | 0 |

Test counts: default 111 passed (base 109), all features 204 (base 201),
`--no-default-features --features net` 140, MSRV 1.92 all features 204; 0
failed.

Consumers against this tree: natnet_zenoh with
`--config 'patch."https://github.com/AndrejOrsula/natnet.git".natnet.path="../natnet"'`
`cargo test` exit 0 (its `Cargo.lock` restored to the `50d8da2` pin
afterwards); motive-bridge `cargo +1.97.1 test --workspace --all-targets
--locked --offline` exit 0, 153 passed.
