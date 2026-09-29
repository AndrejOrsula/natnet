# riders-71 C1 gates

All commands below ran after commit `ea0bba5fc3b1cadea5ec567281a293051681fa7e` with isolated target directory `/home/andrej/coord_scratch/riders71/candidate-post`, unless noted. Every command exited 0.

## Core gates

```text
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders71/candidate-post cargo fmt --all --check
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders71/candidate-post cargo clippy --locked --all-targets --all-features -- -D warnings
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders71/candidate-post cargo test --locked --all-features --no-fail-fast
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders71/candidate-post cargo test --locked --no-fail-fast
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders71/candidate-post cargo build --locked --features net
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders71/candidate-post cargo build --locked --features imagery,net --example imagery_dump
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders71/candidate-post cargo +1.92 test --locked --no-fail-fast
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders71/candidate-post cargo +1.92 test --locked --all-features --no-fail-fast
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders71/candidate-post cargo deny --locked --all-features --manifest-path Cargo.toml check advisories bans licenses sources
rtk proxy git diff-tree --check HEAD^ HEAD
```

Results: default tests 97 runtime plus 5 doctests; all-feature tests 177 runtime plus 10 doctests; all passed with zero failures and zero ignored. Cargo-deny printed `advisories ok, bans ok, licenses ok, sources ok`.

## Feature matrix

For each row, this exact gate passed:

```text
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders71/candidate-post cargo clippy --locked --quiet --all-targets --features <row> -- -D warnings && rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders71/candidate-post cargo test --locked --quiet --features <row> --no-fail-fast
```

Rows: `imagery`, `net`, `serde`, `nalgebra`, `imagery,net`, `serde,nalgebra`.

## Strict documentation matrix

For each row, both commands passed:

```text
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders71/candidate-post cargo check --locked --all-targets --no-default-features [--features <row>]
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders71/candidate-post env RUSTDOCFLAGS=-Dwarnings cargo doc --locked --no-deps --no-default-features [--features <row>]
```

Rows: `default`, `imagery`, `net`, `serde`, `nalgebra`, `imagery-net`, `imagery-serde`, `imagery-nalgebra`, `net-serde`, `net-nalgebra`, `serde-nalgebra`, `imagery-net-serde`, `imagery-net-nalgebra`, `imagery-serde-nalgebra`, `net-serde-nalgebra`, `all`.

## Negative controls and review checks

- Prechange focused suites: modeldef 12, decoder 7, frame-version 11, net-command 11; all passed.
- Guard/comparison ablations listed in `report.md` each failed its new focused test; all source was restored before commit.
- Refuted-citation sweep used exact base/candidate and widened base-parent/candidate legs; exit 0, no `boundaryc1` hits on any leg. The script says this negative result proves nothing.
- `git diff-tree --check HEAD^ HEAD` passed. Tracked candidate status is clean; only the uncommitted C1 draft directory remains.
