# riders-73 C1 — compile the documented 32-bit surface in CI

Repository: `natnet`. Base `dev` `bf1e1fda073662722a07fb57e2895da72a2a1a76`. Candidate branch `slice/riders-73`; worktree `/home/andrej/ws_rs/natnet_wt/impl-riders-73`. Evidence branch `evidence/riders-73`. Access: remote. Read NatNet README/workflow, Auga STATUS/PLAN, and Auga `AGENTS.md` §§4–7.

## Outcome and scope

The crate documents 32-bit-only checked-arithmetic/error paths and carries assertions that branch on `cfg!(target_pointer_width = "32")`, but every CI/deny target is 64-bit. Add compile/config coverage for `i686-unknown-linux-gnu` without claiming runtime evidence.

Own only `.github/workflows/rust.yml` and `deny.toml`. Do not edit source, tests, manifests, locks, README, packets, or existing feature rows. No dependency/API/wire behavior change. Add the i686 target to the existing deny graph and a dedicated stable CI job which installs that exact target through the pinned toolchain action, then runs:

```text
cargo check --locked --all-targets --all-features --target i686-unknown-linux-gnu
```

The repository toolchain file overrides unqualified Cargo, so attach the target to the stable action actually used by the command. Keep all existing jobs/gates. Use one canonical target literal or an explicit workflow env value; add static controls proving install/check/deny agree and a mismatch is detected. Parse YAML and use actionlint if available.

This C1 can reach accepted-build/config only. `cargo test --locked --all-features --target i686-unknown-linux-gnu` remains `blocked_external` until an equipped linker/runtime exists. This workstation lacks the Rust target and 32-bit system linker objects; do not install them, browse, or reuse old `/tmp` evidence. A successful cross-check is not a runtime test.

## Verification and handoff

Show prechange absence from workflow and deny graph, then postchange static controls. Run fmt, default/all Clippy and tests, net build, strict docs and all feature rows, MSRV, cargo-deny over the expanded graph, cargo-machete, YAML/action checks, and committed-tree whitespace with isolated targets. The new i686 check may be unavailable locally; if so, CI is the acceptance instrument and must be inspected at the exact integrated SHA before closing build/config. Never mark runtime complete.

Commit only the two owned files as immutable `candidate(C1)`, with SHA/tree. Leave `docs/work/riders-73/attempts/C1/report.md` and `gates.md` uncommitted; no push. Fresh read-only review and coordinator integration are mandatory.
