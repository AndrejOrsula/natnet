# riders-73 C1 — builder report

## 1. What was built

Added compile/config coverage for the documented 32-bit surface. The workflow
now defines `I686_TARGET = i686-unknown-linux-gnu`, installs that target on the
same pinned stable toolchain used by the repository's unqualified Cargo
commands, validates install/check/deny wiring with positive and mismatch
controls, and runs:

```text
cargo check --locked --all-targets --all-features --target i686-unknown-linux-gnu
```

The cargo-deny graph now includes the same i686 triple. Existing jobs and gates
remain unchanged. Only these candidate paths changed:

```text
.github/workflows/rust.yml
deny.toml
```

No source, test, manifest, lockfile, API, wire format, runtime behavior,
feature row, or dependency changed.

## 2. Brief and scope

The brief's explicit-env option was used: the target literal has one canonical
workflow env definition, while the deny graph retains its required explicit
triple. The brief's runtime boundary was preserved. The local workstation has
no i686 Rust target or 32-bit runtime objects; therefore the implementation
does not claim `cargo test` or any runtime evidence for i686.

## 3. Before, after, and mismatch controls

Before editing, both the workflow and deny graph returned no i686 match (exit
1, no output). Installed targets were only `wasm32-unknown-unknown`,
`x86_64-pc-windows-msvc`, and `x86_64-unknown-linux-gnu`.

After the candidate freeze, source controls verified the canonical env appears
in the pinned target-install input and Cargo check expression, and exactly one
matching deny graph line exists. A temporary deny copy with that line changed
to `mismatch-target` was rejected. Output:

```text
candidate static positive=pass; mismatch negative=pass
```

The candidate's in-workflow validator performs the same checks on CI after the
target action runs.

## 4. Gate result

Fmt, default/all Clippy and tests, net build, strict default/all docs, all six
workflow feature Clippy/test rows, all 16 feature-doc check/doc rows, MSRV
default/all, offline cargo-deny over the expanded target graph, cargo-machete,
YAML parsing, and committed-tree whitespace all passed. The exact commands,
exit codes, test populations, and output summaries are in `gates.md`.

The exact i686 check exited 101 with `E0463` because the target is absent
locally. `actionlint` was unavailable and not run. No target installation,
network access, linker/runtime execution, hardware test, benchmark, or live
Motive evidence was performed.

## 5. Candidate identity and state

| field | value |
|---|---|
| base | `bf1e1fda073662722a07fb57e2895da72a2a1a76` |
| candidate(C1) | `50cb951f5637b491e797baf1ea7546ab854a5e02` |
| candidate tree | `cfeab6ed1af465276dd37b7dc2209e996cf05c49` |
| branch | `slice/riders-73` |
| worktree | `/home/andrej/ws_rs/natnet_wt/impl-riders-73` |
| access | `remote` |
| committed paths | `.github/workflows/rust.yml`, `deny.toml` |
| push | none |
| packet state | these `report.md` and `gates.md` drafts are uncommitted |

Candidate review accepted this immutable identity. Integration and any packet
commit remain coordinator actions.
