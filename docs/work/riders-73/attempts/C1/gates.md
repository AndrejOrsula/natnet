# riders-73 C1 — gates

Candidate under test: `candidate(C1)`
`50cb951f5637b491e797baf1ea7546ab854a5e02`, tree
`cfeab6ed1af465276dd37b7dc2209e996cf05c49`, in
`/home/andrej/ws_rs/natnet_wt/impl-riders-73`. The candidate commit contains
only `.github/workflows/rust.yml` and `deny.toml`. These gate drafts are
uncommitted and were written after the candidate was frozen; no candidate file
was changed afterward. Cargo target directories were isolated per row and
removed after each result was read.

## Before-edit controls

The base was `bf1e1fda073662722a07fb57e2895da72a2a1a76`.

| control | exact command | result |
|---|---|---|
| workflow absence | `rtk proxy rg -n 'i686-unknown-linux-gnu|I686_TARGET' .github/workflows/rust.yml` | exit 1; no output, no i686 target or env entry |
| deny absence | `rtk proxy rg -n 'i686-unknown-linux-gnu|I686_TARGET' deny.toml` | exit 1; no output, no i686 graph entry |
| local targets | `rtk proxy rustup target list --installed` | exit 0; `wasm32-unknown-unknown`, `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`; no i686 target |

## Static and workflow-shape controls

The candidate's `verify i686 target wiring` step checks the stable target
installation, the workflow action input, the Cargo `--target` expression, and
the exact `deny.toml` graph line. Its positive control passed. A temporary
copy of `deny.toml` with the i686 line changed to `mismatch-target` failed the
same validator, so the negative mismatch control passed. The post-freeze
source control printed:

```text
candidate static positive=pass; mismatch negative=pass
```

| gate | exact command | result |
|---|---|---|
| YAML parse | `rtk proxy python3 -c 'from pathlib import Path; import yaml; p=Path(".github/workflows/rust.yml"); yaml.safe_load(p.read_text()); print("candidate YAML parse=pass")'` | exit 0; `candidate YAML parse=pass` |
| fmt | `rtk proxy cargo fmt --all --check` | exit 0 |
| candidate whitespace | `rtk proxy git diff --check HEAD^ HEAD` | exit 0 |
| actionlint | `rtk proxy sh -c 'command -v actionlint || true'` | unavailable; command produced no path, so actionlint was not run |

## Ordinary Cargo gates

| gate | exact command | result |
|---|---|---|
| Clippy default | `rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-riders-73-C1-clippy-default cargo clippy --locked --all-targets -- -D warnings` | exit 0; `Finished dev profile` |
| Clippy all features | `rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-riders-73-C1-clippy-all cargo clippy --locked --all-targets --all-features -- -D warnings` | exit 0; `Finished dev profile` |
| tests default | `rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-riders-73-C1-test-default cargo test --locked` | exit 0; 97 runtime tests + 6 doctests passed, 0 failed |
| tests all features | `rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-riders-73-C1-test-all cargo test --locked --all-features` | exit 0; 177 runtime tests + 11 doctests passed, 0 failed |
| net build | `rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-riders-73-C1-net cargo build --locked --features net` | exit 0; `Finished dev profile` |
| strict docs default | `rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-riders-73-C1-doc-default RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps` | exit 0; documentation generated |
| strict docs all features | `rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-riders-73-C1-doc-all RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps --all-features` | exit 0; documentation generated |

## Workflow `features` rows

Each row ran both commands below, with a fresh target directory and the row's
feature value:

```text
rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-riders-73-C1-feature-clippy-<row> cargo clippy --locked --all-targets --features <set> -- -D warnings
rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-riders-73-C1-feature-test-<row> cargo test --locked --features <set>
```

All six Clippy and test commands exited 0 with no warnings or failures:

| row / feature set | test population |
|---|---:|
| `imagery` / `imagery` | 125 runtime + 6 doctests = 131 |
| `net` / `net` | 115 runtime + 11 doctests = 126 |
| `serde` / `serde` | 99 runtime + 6 doctests = 105 |
| `nalgebra` / `nalgebra` | 99 runtime + 6 doctests = 105 |
| `imagery-net` / `imagery,net` | 173 runtime + 11 doctests = 184 |
| `serde-nalgebra` / `serde,nalgebra` | 101 runtime + 6 doctests = 107 |

## `feature-docs` matrix

Each row ran `cargo check --locked --all-targets --no-default-features` and
strict `cargo doc --locked --no-deps --no-default-features`, adding the row's
feature set when non-empty. Every check/doc pair exited 0:

```text
rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-riders-73-C1-feature-check-<row> cargo check --locked --all-targets --no-default-features [--features <set>]
rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-riders-73-C1-feature-doc-<row> RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps --no-default-features [--features <set>]
```

Rows: `default`, `imagery`, `net`, `serde`, `nalgebra`, `imagery-net`,
`imagery-serde`, `imagery-nalgebra`, `net-serde`, `net-nalgebra`,
`serde-nalgebra`, `imagery-net-serde`, `imagery-net-nalgebra`,
`imagery-serde-nalgebra`, `net-serde-nalgebra`, `all`.

## MSRV

Toolchain: `rustc 1.92.0` / `cargo 1.92.0`, host
`x86_64-unknown-linux-gnu`.

| gate | exact command | result |
|---|---|---|
| MSRV default | `rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-riders-73-C1-msrv-default cargo +1.92 test --locked` | exit 0; 97 runtime tests + 6 doctests passed, 0 failed |
| MSRV all features | `rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-riders-73-C1-msrv-all cargo +1.92 test --locked --all-features` | exit 0; 177 runtime tests + 11 doctests passed, 0 failed |

## Dependency and tree gates

| gate | exact command | result |
|---|---|---|
| cargo-deny expanded graph | `rtk proxy cargo-deny --offline --locked --all-features --manifest-path Cargo.toml check advisories bans licenses sources` | exit 0; `advisories ok, bans ok, licenses ok, sources ok` |
| cargo-machete | `rtk proxy cargo-machete` | exit 0; no unused dependencies |

## i686 boundary

The exact candidate check was attempted without installing a target:

```text
rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-riders-73-C1-i686-unavailable-final cargo check --locked --all-targets --all-features --target i686-unknown-linux-gnu
```

It exited 101 with `error[E0463]: can't find crate for core` and
`the i686-unknown-linux-gnu target may not be installed`. This is
`blocked_external` local evidence, not a candidate failure. No target install,
network access, linker/runtime test, hardware test, or benchmark was run.

No runtime claim is made. CI's stable action-installed target and its
compile-only `cargo check` are the acceptance instrument; 32-bit runtime tests
remain external.
