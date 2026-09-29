# riders-73 E1 — execute the current i686 surface

Work kind: evidence-only, remote workstation. Coordinator-owned evidence
branch/worktree: `evidence/riders-73`,
`/home/andrej/ws_rs/natnet_wt/evidence-riders-73`. One collector; no descendants.
The coordinator freezes the pre-review evidence commit after collection.

## Goal and fixed inputs

Determine whether the current NatNet revision passes actual native i686
execution, including its pointer-width-dependent arithmetic assertions.
Compile success and older runtime packets do not answer this question.

Source input: integrated NatNet
`01302f32af0b6935f46fea4a690a4d35a6d5a407`, tree
`8bfa45cd18838adfc122a53b5371f907332ea5bd` (`git show -s` at dispatch).
Accepted compile/config input: riders-73 C2 and CI run `35537300634`,
including i686 compile job `106148517661`, at that same revision. Read
`README.md`, `Cargo.toml`, `rust-toolchain.toml`, `.github/workflows/rust.yml`,
the riders-73 C1/C2 packets, and Auga `AGENTS.md` §§4–7 plus
`docs/09-implementation.md` §7. Inspect `tests/imagery_net.rs`,
`tests/payload_robustness.rs`, `tests/imagery.rs`, and the corresponding
checked-arithmetic implementations before interpreting runtime results.

Assumption to test: the native Linux kernel can execute ELF32 with an isolated
Ubuntu i386 userspace. The September 27 scout found the compiler's `-m32`
support but missing development startup objects and Rust target. Dependency:
official Rust target and Ubuntu archive availability; no hardware or Motive
access is required. Existing consumers remain pinned; this attempt changes no
source, manifest, lock, CI, contract, supported-platform matrix, or consumer.

## Ownership and safety

Draft only `report.md` and `gates.md` under this attempt directory. Retain raw
artifacts under a newly allocated `mktemp -d` directory within
`/home/andrej/coord_preserve/`; record the absolute path. Scratch scripts,
linker/runner configuration and a clean local clone of the exact source input
belong there. Create scripts with `apply_patch`. Do not commit, push, edit
implementation or shared documents, change system packages/configuration,
contact devices, deploy the bridge, run benchmarks, or delete older evidence.

The C1 brief's no-install constraint applied to that compile/config attempt.
This E1 procedure explicitly permits adding `i686-unknown-linux-gnu` to the
existing Rust 1.98.1 toolchain and downloading/extracting official Ubuntu
packages into isolated scratch. It does not authorize `apt install` or system
package upgrades. Preserve source-input cleanliness throughout collection.
Use `rtk` at the start of shell commands and an isolated Cargo target.

## Committed procedure

1. Record UTC time, source SHA/tree and clean state, source archive/lock hashes,
   kernel/OS/architecture, `cc`/linker identity, Rust/Cargo verbose versions,
   installed target state, environment relevant to compilation, and disk
   availability. Recheck package candidates and source trust with local APT
   metadata. If an exact archive below is unavailable, report the blocker;
   do not silently substitute an unreviewed package or external mirror.
2. Download the following official Ubuntu archives into the new scratch and
   extract with `dpkg-deb -x` into a private sysroot. Record package name,
   version, architecture, origin, expected and downloaded SHA-256, and
   extraction commands. No root/system installation:

   ```text
   libc6-i386:amd64=2.39-0ubuntu8.7
   lib32gcc-s1:amd64=14.2.0-4ubuntu2~24.04.1
   libc6-dev-i386:amd64=2.39-0ubuntu8.7
   lib32gcc-13-dev:amd64=13.3.0-6ubuntu2~24.04.1
   ```

3. Run `rtk proxy rustup +1.98.1 target add i686-unknown-linux-gnu`. Configure
   a target-specific linker wrapper around `/usr/bin/cc --sysroot=<root>`,
   `-B<root>/usr/lib/gcc/x86_64-linux-gnu/13/32/`, and private library search
   paths `<root>/usr/lib/i386-linux-gnu`, `<root>/usr/lib32`,
   `<root>/lib/i386-linux-gnu`, `<root>/lib32`. Record exact wrapper bytes and
   hashes. Target configuration must not alter the host linker. Invoke ELF32
   executables with the private `<root>/usr/lib32/ld-linux.so.2` and explicit
   `--library-path <root>/usr/lib32:<root>/usr/lib/i386-linux-gnu` through a
   target-specific Cargo runner; record that wrapper too.
4. Compile and execute a tiny C control with `-m32` that reports and checks
   `sizeof(void*) == 4`; inspect its ELF class/machine with `file` and
   `readelf`. Compile/run the same source with the native host compiler as
   an independently observed 8-byte-pointer control. No emulator may be used
   or silently substituted. A failed control blocks runtime claims.
5. From a clean local clone checked out at the fixed source input, build
   `imagery_dump` with Rust 1.98.1, `--locked --features imagery,net --example
   imagery_dump --target i686-unknown-linux-gnu`. Inspect ELF32 and run the
   resulting example through the explicit runner with `--max 0`. Confirm
   the source's early no-socket return before using this smoke command.
6. Run actual target tests with the recorded linker/runner configuration:

   ```text
   rtk proxy cargo +1.98.1 test --locked --all-targets --all-features --target i686-unknown-linux-gnu --no-fail-fast -- --test-threads=1
   rtk proxy cargo +1.98.1 test --locked --all-features --doc --target i686-unknown-linux-gnu --no-fail-fast -- --test-threads=1
   ```

   Add `--offline` only if the locked dependency cache permits it and record
   the exact executed command. Capture unfiltered stdout/stderr and exit
   status; identify every actually executed test binary and inspect ELF32.
   Use compiler message artifacts and/or runner invocation logs to bind
   binaries to commands. Explicitly identify named assertions taking the
   32-bit arms, their exact source lines and results. A skipped cross-target
   doctest or a host binary is not target runtime evidence. If rustdoc does
   not execute target examples, report that limitation rather than crediting
   a successful command with execution it did not perform.
7. Run the all-target/all-feature suite and separate doctests with the same
   fixed toolchain on the native host as a differential control. Keep host
   and target counts separate. Run source `cargo +1.98.1 fmt --all --check`,
   verify clean source/lock/config state, and hash retained artifacts.
   No product code changed, so unrelated lint, feature matrix, dependency
   audit and documentation compilation gates are not rerun by this evidence
   procedure; cite current CI only for what its exact jobs actually checked.

## Acceptance and proof

The report binds procedure/input identity, environment, tool/package versions,
wrapper bytes, raw hashes, command exits, runtime binary identities, and
separate passed/failed/ignored/skipped populations. Include the pointer-width
positive and host controls, example smoke, target unit/integration/doctest
execution, and source cleanliness. Hash manifests must not include themselves;
verify them after writing and report the verification command/result.

The report must say what was wrong or uncovered in the procedure, what could
not be verified, and what differs from historical evidence. Follow the AGENTS
reporting sweeps with actual commands and reach controls where applicable;
do not invent pre-commit proof or claim a process census without its command,
scope and verdict. Raw logs stay unversioned. Evidence drafts stay uncommitted
for the coordinator, who records their pre-review commit identity separately.

A different fresh-context reviewer checks that identity and six lenses before
`accepted-measurement`. Runtime acceptance requires successful actual i686
execution, not merely compile success. Findings require an E2 procedure or
recollection, never mutation of an evidence identity already sent to review.
The coordinator alone records acceptance, integrates the packet and updates
Auga's board. No empty implementation candidate is needed.
