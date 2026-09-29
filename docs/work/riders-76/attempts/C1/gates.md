# riders-76 C1 — gates

Candidate-bound gates ran from clean candidate `cb1495ba83458382264f1de05bffb34605fb7d57`,
tree `27e0ca06991cea4b29c8811b48957b9542658ef2`, in
`/home/andrej/ws_rs/natnet_wt/impl-riders-76`. Cargo target directories were
isolated under `/home/andrej/ws_rs/natnet_wt/target-riders-76-final*`.

## Focused and ordinary gates

| gate | exact command | result |
|---|---|---|
| focused imagery/net integration | `rtk proxy env CARGO_TARGET_DIR=/home/andrej/ws_rs/natnet_wt/target-riders-76-final cargo test --locked --features imagery,net --test imagery_net -- --test-threads=1` | exit 0; 23 passed, 0 failed |
| focused stability repeat | exact command below | 100/100 passed; 1,500 queue cases; loadavg 11.72/12.05/11.70 before and 11.74/12.05/11.70 after |
| default tests | `rtk proxy env CARGO_TARGET_DIR=/home/andrej/ws_rs/natnet_wt/target-riders-76-final cargo test --locked` | exit 0; all runtime and default doctest groups passed |
| all-feature tests | `rtk proxy env CARGO_TARGET_DIR=/home/andrej/ws_rs/natnet_wt/target-riders-76-final cargo test --locked --all-features` | exit 0; all runtime and all-feature doctest groups passed |
| fmt | `rtk proxy env CARGO_TARGET_DIR=/home/andrej/ws_rs/natnet_wt/target-riders-76-final cargo fmt --all -- --check` | exit 0 |
| all-feature Clippy | `rtk proxy env CARGO_TARGET_DIR=/home/andrej/ws_rs/natnet_wt/target-riders-76-final cargo clippy --locked --all-targets --all-features -- -D warnings` | exit 0 |
| net build | `rtk proxy env CARGO_TARGET_DIR=/home/andrej/ws_rs/natnet_wt/target-riders-76-final cargo build --locked --features net` | exit 0 |
| imagery example build | `rtk proxy env CARGO_TARGET_DIR=/home/andrej/ws_rs/natnet_wt/target-riders-76-final cargo build --locked --features imagery,net --example imagery_dump` | exit 0 |
| strict default docs | `rtk proxy env CARGO_TARGET_DIR=/home/andrej/ws_rs/natnet_wt/target-riders-76-final RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps` | exit 0 |
| strict all-feature docs | `rtk proxy env CARGO_TARGET_DIR=/home/andrej/ws_rs/natnet_wt/target-riders-76-final RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features` | exit 0 |

The retained 100-run command was:

```sh
rtk proxy env CARGO_TARGET_DIR=/home/andrej/ws_rs/natnet_wt/target-riders-76-final bash -c 'printf "load_before="; cat /proc/loadavg; fails=0; for i in $(seq 1 100); do if ! cargo test --locked --features imagery,net --test imagery_net a_record_too_long_for_the_receive_queue_is_not_completed_past_its_deadline -- --exact --test-threads=1 >/tmp/riders-76-final-queue-${i}.log 2>&1; then fails=$((fails + 1)); fi; done; printf "load_after="; cat /proc/loadavg; printf "queue_runs=100 failures=%s\n" "$fails"; [ "$fails" -eq 0 ]'
```

## Workflow feature-doc rows

The 16 rows used `cargo check --locked --all-targets --no-default-features`
and strict `cargo doc --locked --no-deps --no-default-features`, adding each
non-empty row's `--features` value. Rows were `default`, `imagery`, `net`,
`serde`, `nalgebra`, `imagery,net`, `imagery,serde`, `imagery,nalgebra`,
`net,serde`, `net,nalgebra`, `serde,nalgebra`, `imagery,net,serde`,
`imagery,net,nalgebra`, `imagery,serde,nalgebra`, `net,serde,nalgebra`, and
`imagery,net,serde,nalgebra`.

The exact candidate command loop printed `check=0 doc=0` for every row and
`feature_rows=16 failing_rows=0`.

The relevant workflow row also passed:

```text
rtk proxy env CARGO_TARGET_DIR=/home/andrej/ws_rs/natnet_wt/target-riders-76-final cargo clippy --locked --all-targets --features imagery,net -- -D warnings
rtk proxy env CARGO_TARGET_DIR=/home/andrej/ws_rs/natnet_wt/target-riders-76-final cargo test --locked --features imagery,net
```

Both exited 0; the feature test row reported 23 `imagery_net` cases passed.

## MSRV

Toolchain: `rustc 1.92.0 (ded5c06cf 2025-12-08)` and
`cargo 1.92.0 (344c4567c 2025-10-21)`.

```text
rtk proxy env CARGO_TARGET_DIR=/home/andrej/ws_rs/natnet_wt/target-riders-76-final-msrv cargo +1.92 test --locked
rtk proxy env CARGO_TARGET_DIR=/home/andrej/ws_rs/natnet_wt/target-riders-76-final-msrv cargo +1.92 test --locked --all-features
```

Both exited 0; default and all-feature runtime plus doctest groups passed.

## Dependency and committed-tree gates

- `rtk proxy cargo-deny --locked --all-features --manifest-path Cargo.toml check advisories bans licenses sources` — exit 0; advisories, bans, licenses, and sources passed.
- `rtk proxy cargo-machete` — exit 0; no unused dependencies.
- `rtk proxy git diff --check HEAD^ HEAD` — exit 0; candidate diff contains only `tests/imagery_net.rs`.
- `rtk proxy git status --short --branch` — clean `slice/riders-76` worktree.

## Controls and limits

In a detached temporary worktree, changing the post-deadline single read to a
repeated drain made the queue-bound test fail with `Ok(true)`; the two complete
prebuffered positive controls still passed. The temporary worktree was removed.
No Windows, Motive, hardware, benchmark, or live-network gate was applicable
or run.
