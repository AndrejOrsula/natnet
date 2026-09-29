# riders-72 C1 — gates

Tree under test for candidate-bound rows: `candidate(C1)`
`adbd2bcf157fb567e35d7b242fb94ec4529a3616`, tree
`74e3f9b730adfdcdfe58d6bb5d0f448f437b5cf5`, in
`/home/andrej/ws_rs/natnet_wt/impl-riders-72`. Each Cargo command used an
isolated `/tmp/natnet-riders-72-C1-*` target directory. No source was changed
after this identity was frozen.

## Focused and ordinary gates

| gate | exact command | result |
|---|---|---|
| default doctests | `CARGO_TARGET_DIR=/tmp/natnet-riders-72-C1-doc-default cargo test --locked --doc` | exit 0; 1 positive + 5 compile-fail cases passed in the default-feature doc-test population |
| all-feature doctests | `CARGO_TARGET_DIR=/tmp/natnet-riders-72-C1-doc-all cargo test --locked --doc --all-features` | exit 0; 1 positive + 10 compile-fail cases passed in the all-feature doc-test population |
| fmt | `cargo fmt --all --check` | exit 0 |
| Clippy default | `CARGO_TARGET_DIR=/tmp/natnet-riders-72-C1-clippy-default cargo clippy --locked --all-targets -- -D warnings` | exit 0 |
| Clippy all features | `CARGO_TARGET_DIR=/tmp/natnet-riders-72-C1-clippy-all cargo clippy --locked --all-targets --all-features -- -D warnings` | exit 0 |
| tests default | `CARGO_TARGET_DIR=/tmp/natnet-riders-72-C1-test-default cargo test --locked` | exit 0; 94 runtime tests plus 6 doctests passed, 0 failed, over the default-feature test population |
| tests all features | `CARGO_TARGET_DIR=/tmp/natnet-riders-72-C1-test-all cargo test --locked --all-features` | exit 0; 173 runtime tests plus 11 doctests passed, 0 failed, over the all-feature test population |
| net build | `CARGO_TARGET_DIR=/tmp/natnet-riders-72-C1-net cargo build --locked --features net` | exit 0 |
| strict docs default | `CARGO_TARGET_DIR=/tmp/natnet-riders-72-C1-doc-default RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps` | exit 0 |
| strict docs all features | `CARGO_TARGET_DIR=/tmp/natnet-riders-72-C1-doc-all RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features` | exit 0 |

## Workflow feature rows

For each listed set, the exact commands were:

```text
CARGO_TARGET_DIR=/tmp/natnet-riders-72-C1-feature-<name> cargo clippy --locked --all-targets --no-default-features --features <set> -- -D warnings
CARGO_TARGET_DIR=/tmp/natnet-riders-72-C1-feature-<name> cargo test --locked --no-default-features --features <set>
```

All six workflow `features` rows returned exit 0 for both commands. The test
case populations (runtime cases plus doctests) were: `imagery` 128,
`net` 117, `serde` 102, `nalgebra` 102, `imagery,net` 184, and
`serde,nalgebra` 104; every case passed and every Clippy row emitted no
warning or error.

The workflow `feature-docs` matrix used these exact row commands, with
`--no-default-features`, the row's feature set when non-empty, and
`RUSTDOCFLAGS="-D warnings"` on `cargo doc`:

```text
CARGO_TARGET_DIR=/tmp/natnet-riders-72-C1-feature-doc-<name> cargo check --locked --all-targets --no-default-features [--features <set>]
CARGO_TARGET_DIR=/tmp/natnet-riders-72-C1-feature-doc-<name> RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --no-default-features [--features <set>]
```

The matrix population was 16 named rows (`default`, `imagery`, `net`,
`serde`, `nalgebra`, `imagery-net`, `imagery-serde`, `imagery-nalgebra`,
`net-serde`, `net-nalgebra`, `serde-nalgebra`, `imagery-net-serde`,
`imagery-net-nalgebra`, `imagery-serde-nalgebra`, `net-serde-nalgebra`,
`all`). Every row printed `check_rc=0 doc_rc=0`.

## MSRV

Toolchain identity: `rustc 1.92.0 (ded5c06cf 2025-12-08)` and
`cargo 1.92.0 (344c4567c 2025-10-21)` on `x86_64-unknown-linux-gnu`.

- `CARGO_TARGET_DIR=/tmp/natnet-riders-72-C1-msrv-default cargo +1.92 test --locked`
  — exit 0; 94 runtime tests plus 6 doctests passed, 0 failed, over the
  default-feature MSRV test population.
- First `CARGO_TARGET_DIR=/tmp/natnet-riders-72-C1-msrv-all cargo +1.92 test --locked --all-features`
  — exit 101; 23 of 24 `imagery_net` runtime cases passed and one failed,
  `a_peer_still_sending_past_the_deadline_is_a_timeout`; the other runtime
  and doctest populations passed. This is a timing-sensitive pre-existing
  test, not a changed file.
- Retry `CARGO_TARGET_DIR=/tmp/natnet-riders-72-C1-msrv-all-retry cargo +1.92 test --locked --all-features`
  — exit 0; 173 runtime tests plus 11 doctests passed, 0 failed, over the
  all-feature MSRV test population.

## Dependency and tree gates

- `cargo-deny fetch db` — exit 0.
- `cargo-deny --locked --all-features --manifest-path Cargo.toml check advisories bans licenses sources` — exit 0; advisories, bans, licenses, and sources all passed. The full output carried 2 warnings, both pre-existing unmatched `BSD-2-Clause` and `BSD-3-Clause` allowances in `deny.toml:35-36`.
- `cargo-machete` (`0.9.2`) — exit 0; no unused dependencies.
- `git diff --check HEAD^ HEAD` — exit 0; the candidate diff path population was exactly one path: `src/lib.rs`.
- `git status --short --branch` after candidate freeze — clean tracked tree on `slice/riders-72`; the only later files are these uncommitted packet drafts.

No benchmark, Windows, hardware, or live Motive gate was applicable or run.
