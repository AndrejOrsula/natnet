# riders-73 E1 — native i686 runtime evidence

## 1. Result

E1 collected actual current-source native i686 runtime evidence. The source
input is commit `01302f32af0b6935f46fea4a690a4d35a6d5a407`, tree
`8bfa45cd18838adfc122a53b5371f907332ea5bd`, checked out in a clean detached
clone. No product source, manifest, lockfile, or shared document was changed.

The private Ubuntu biarch sysroot and Rust 1.98.1 target produced and executed
ELF32 Intel 80386 binaries directly under the current x86_64 Linux kernel. The
positive C control printed `sizeof(void*)=4 expected=4 uint32_wrap=0` and exited
0. The native host control, compiled from the same source, printed
`sizeof(void*)=8 expected=8 uint32_wrap=0` and exited 0.

The target `--all-targets --all-features` suite ran 176 runtime tests over 15
test binaries: 176 passed, 0 failed, 0 ignored, 0 measured, and 0 filtered.
The runner log contains 16 invocations including the zero-test example harness;
all 16 were inspected as ELF32/i386 with the private interpreter. The separate
target doctest command passed 1 executable doctest and 10 compile-fail
doctests. Its one rustdoc test executable was inspected by the runner as
ELF32/i386. The example smoke command `--max 0` exited 0 with no output.

The host differential used the same pinned Rust toolchain. Its all-target,
all-feature suite passed 176 runtime tests with 0 failures, and its separate
doctest command passed 1 executable doctest plus 10 compile-fail doctests.

## 2. Procedure and input identity

| field | value |
|---|---|
| procedure revision | `187f89d99ee808f771908268973a381fa6f52bcd` |
| source commit | `01302f32af0b6935f46fea4a690a4d35a6d5a407` |
| source tree | `8bfa45cd18838adfc122a53b5371f907332ea5bd` |
| source clone | `/home/andrej/coord_preserve/riders73-E1.ZUSSmh/source` |
| scratch | `/home/andrej/coord_preserve/riders73-E1.ZUSSmh` |
| source archive SHA-256 | `e2646aa3c98ea7a8c94d83c8ba3eaead6f9457ddfb99b7c10caffa1660c575e6` |
| `Cargo.lock` SHA-256 | `a8a5b8f5462663d6722466ba154369614c0fd4b693135d99cffbf2eaf1c150f0` |
| source state | detached `HEAD`; unstaged and staged `git diff --exit-code` both exit 0 |
| evidence branch state | only these E1 drafts are new; no packet commit or push |

The source archive excludes the clone's `.git` directory and is retained in
`manifests/source-01302f3.tar`. Existing historical runtime artifacts were not
used as current-source evidence.

## 3. Brief assumptions exposed during collection

The four exact procedure packages were available in local APT metadata, but
the configured Noble archive/security URLs returned HTTP 404 for the exact
glibc versions. `logs/apt-download.log` retains that failed attempt. The exact
glibc packages were fetched from the official Ubuntu Snapshot archive at
`20260212T000000Z`; the exact GCC packages came from the official Ubuntu
archive. No unreviewed mirror was substituted.

The specified `libc6-dev-i386` package depends on generic
`libc6-dev`, and the four specified packages alone did not provide the generic
headers required by the isolated C link. I therefore fetched the exact
official `libc6-dev:amd64=2.39-0ubuntu8.7` archive and extracted it into the
same private sysroot. This dependency addition is disclosed, hashed, and was
not installed system-wide. The sysroot also needed scratch-only `lib`/`lib32`
symlinks and the private loader path used by the wrapper; no system package
state was changed.

This is an explicit procedure deviation, not a silent replacement. The
retained `logs/package-provenance-addendum.log` records HTTP 200 responses for
the three exact glibc archives from the official Ubuntu Snapshot URL
`https://snapshot.ubuntu.com/ubuntu/20260212T000000Z/` and for the two exact
GCC archives from `fr.archive.ubuntu.com/ubuntu`. It also records
`dpkg-deb -f` output showing `libc6-dev-i386` depends on
`libc6-dev (= 2.39-0ubuntu8.7)` and the downloaded hashes for both development
archives. The package metadata and all five archive hashes are independently
retained in `logs/package-metadata.log` and
`manifests/package-download.sha256`.

The first target test attempt exposed a runner detail: Cargo's
`std::env::current_exe()` fallback saw the explicit loader executable rather
than the example path, so `tests/imagery_cli.rs` initially failed with
`ENOENT`. The target-only runner now exports
`CARGO_BIN_EXE_imagery_dump` to the built target example before invoking the
private loader. The initial failed runner logs remain beside the final logs;
the final procedure commands were rerun after this scratch-only harness fix.

The native host all-target command likewise needs an actual host example
binary. Cargo's test harness path is a hashed test executable, not the example
program. The host differential first recorded this harness failure, then built
the host example explicitly and reran with its real path in
`CARGO_BIN_EXE_imagery_dump`; that final run passed.

## 4. Environment and private packages

Final environment capture is `logs/environment-final.txt`.

| item | observed value |
|---|---|
| UTC capture | `2026-09-27T13:57:30Z` |
| kernel/host | Linux `6.8.0-101-generic`, x86_64 |
| OS | Ubuntu 24.04.4 LTS Noble |
| dpkg architectures | native `amd64`; foreign `i386` |
| C compiler | Ubuntu GCC `13.3.0-6ubuntu2~24.04.1` |
| Rust | `rustc 1.98.1 (48a229cea 2026-09-01)`, LLVM 22.1.8 |
| Cargo | `cargo 1.98.1 (797e8a9bc 2026-08-05)` |
| installed Rust targets | `i686-unknown-linux-gnu`, `x86_64-unknown-linux-gnu` |
| final disk | 343G free on `/` and `/home/andrej/coord_preserve` |

All five downloaded archives are Ubuntu `amd64` packages extracted with
`dpkg-deb -x` into the private sysroot. Expected metadata hashes and downloaded
hashes agree; `cd apt && sha256sum -c ../manifests/package-download.sha256`
returned `OK` for every row.

| package | version | expected/downloaded SHA-256 |
|---|---|---|
| `libc6-i386:amd64` | `2.39-0ubuntu8.7` | `a80dfe7331c74bda498aefcd49c20e531977736e287db224d53e9915bdd6d509` |
| `lib32gcc-s1:amd64` | `14.2.0-4ubuntu2~24.04.1` | `1802dc75345ccce457b44110a3819ca74ff8c33fc62394a78d0e59a11c0cf770` |
| `libc6-dev-i386:amd64` | `2.39-0ubuntu8.7` | `3502744587e191d2b6c5fd5ce7f64784c9f6c94d270b1110dbc0993f6da19815` |
| `lib32gcc-13-dev:amd64` | `13.3.0-6ubuntu2~24.04.1` | `0850e0555c86f2ce7a954158eb47df42d7645bfa4f12f29ea2df76459c930ac0` |
| extra `libc6-dev:amd64` | `2.39-0ubuntu8.7` | `bbf5a155039042634961a61276650631ee47b9e721f91f8dbb731b0bbe046df3` |

The private loader is itself an ELF32/i386 object:
`sysroot/usr/lib32/ld-linux.so.2`. No emulator, container, hardware, or
system-wide package installation was used.

## 5. Target wrappers and controls

The final linker wrapper is `bin/i686-cc.sh`, SHA-256
`631cc18f7191701a2ebbe836a0d1fb63d6acd43aaec163a5b032eacf26739a19`:

```sh
#!/bin/sh
set -eu
exec /usr/bin/cc \
  --sysroot=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/sysroot \
  -B/home/andrej/coord_preserve/riders73-E1.ZUSSmh/sysroot/usr/lib/gcc/x86_64-linux-gnu/13/32/ \
  -Wl,--dynamic-linker=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/sysroot/usr/lib32/ld-linux.so.2 \
  -L/home/andrej/coord_preserve/riders73-E1.ZUSSmh/sysroot/usr/lib/i386-linux-gnu \
  -L/home/andrej/coord_preserve/riders73-E1.ZUSSmh/sysroot/usr/lib32 \
  -L/home/andrej/coord_preserve/riders73-E1.ZUSSmh/sysroot/lib/i386-linux-gnu \
  -L/home/andrej/coord_preserve/riders73-E1.ZUSSmh/sysroot/lib32 \
  "$@"
```

The final target runner is `bin/i686-run.sh`, SHA-256
`609d83df0db0dfc29bde3a5682d73052618cca810fa218f31495b4bb6727fdd5`. It
records the invoked path and `file`/`readelf` ELF identity, exports the target
example path for subprocess tests, and invokes:

```sh
/home/andrej/coord_preserve/riders73-E1.ZUSSmh/sysroot/usr/lib32/ld-linux.so.2 \
  --library-path /home/andrej/coord_preserve/riders73-E1.ZUSSmh/sysroot/usr/lib32:/home/andrej/coord_preserve/riders73-E1.ZUSSmh/sysroot/usr/lib/i386-linux-gnu \
  "$@"
```

The private wrappers are target-specific environment variables; the host
linker was not replaced. `rustup +1.98.1 target add i686-unknown-linux-gnu`
completed successfully. `rustc +1.98.1 --target i686-unknown-linux-gnu --print
cfg` reported `target_pointer_width="32"`; native host cfg reported
`target_pointer_width="64"`.

The C control source is `controls/pointer_width.c`, SHA-256
`6c3a04261b77ca0f475f0ca3f95edead616e53fbc5760cf3b41db566d904322d`.

| control | identity | run | result |
|---|---|---|---|
| target | `file`: ELF 32-bit LSB pie, Intel 80386; `readelf`: `ELF32`, `Intel 80386`, private interpreter | private runner with argument `4` | `sizeof(void*)=4 expected=4 uint32_wrap=0`; exit 0 |
| host | `file`: ELF 64-bit LSB pie, x86-64; `readelf`: `ELF64`, `Advanced Micro Devices X86-64` | native binary with argument `8` | `sizeof(void*)=8 expected=8 uint32_wrap=0`; exit 0 |

Target control SHA-256 is
`154afb96c7958587f5377ac9efe4e7adba2e5478107875ed99ae583b82b3e940`;
host control SHA-256 is
`3dc41c1d8d2e4ad8c8d6c96a5d28772c625c70d29179e3851b064b67dcdf286d`.

## 6. Example build and smoke

Source `examples/imagery_dump.rs:57-60` returns success for `--max 0` before
any bind or connect call. The exact target build was:

```text
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/cargo-target-i686 \
  CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/bin/i686-cc.sh \
  CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_RUNNER=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/bin/i686-run.sh \
  cargo +1.98.1 build --locked --features imagery,net --example imagery_dump --target i686-unknown-linux-gnu
```

Exit was 0. The resulting `debug/examples/imagery_dump` is ELF32/i386 with
private interpreter and BuildID `944172e44ab55e30e2ebfe826bde3dd50d910470`;
SHA-256 is
`0b318809eff9e036f999e282e1db7c79e533d38fb56a13d6dffa0872aa4fa7c0`.

```text
rtk proxy /home/andrej/coord_preserve/riders73-E1.ZUSSmh/bin/i686-run.sh \
  /home/andrej/coord_preserve/riders73-E1.ZUSSmh/cargo-target-i686/i686-unknown-linux-gnu/debug/examples/imagery_dump --max 0
```

Exit was 0 and stdout/stderr were empty. This is a no-socket smoke only; it is
not live Motive or hardware evidence.

## 7. Target runtime tests

Final unfiltered output is `logs/target-tests-final-no-ld.log`; the runner
binding is `logs/runner-invocations-target-tests-final-no-ld.log`. The exact
commands were:

```text
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/cargo-target-i686 \
  CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/bin/i686-cc.sh \
  CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_RUNNER=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/bin/i686-run.sh \
  cargo +1.98.1 test --locked --all-targets --all-features --target i686-unknown-linux-gnu --no-fail-fast -- --test-threads=1

rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/cargo-target-i686 \
  CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/bin/i686-cc.sh \
  CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_RUNNER=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/bin/i686-run.sh \
  cargo +1.98.1 test --locked --all-features --doc --target i686-unknown-linux-gnu --no-fail-fast -- --test-threads=1
```

Both commands exited 0. The runtime count is over the 15 named test binaries
listed below, unit and integration tests included; the example harness
is a separate sixteenth runner invocation with zero tests.

| executed target binary | passed |
|---|---:|
| `natnet` unit | 5 |
| `command` | 23 |
| `decoder` | 8 |
| `frame_versions` | 12 |
| `imagery` | 28 |
| `imagery_cli` | 6 |
| `imagery_net` | 23 |
| `modeldef_versions` | 13 |
| `net_command` | 12 |
| `net_data` | 6 |
| `optional_features` | 4 |
| `payload_robustness` | 6 |
| `robustness` | 9 |
| `roundtrip` | 17 |
| `serverinfo_encoding` | 4 |
| `imagery_dump` example harness | 0 |
| **total runtime tests over the 15 test binaries** | **176** |

Every row in the runner log is `ELF32`, machine `Intel 80386`, dynamically
linked to the private `sysroot/usr/lib32/ld-linux.so.2`. The runtime binary
hashes are in `manifests/runtime-binaries.sha256`, which verifies successfully.

The separate doctest command emitted rustdoc's warning that no rustdoc doctest
environment variable was provided and doctests would run in the same process.
It nevertheless invoked one temporary `rust_out` through the target runner;
the runner log records `ELF32`, `Intel 80386`, and the private interpreter.
That executable doctest passed. The ten compile-fail doctests also passed, but
are compile-only and are not credited as runtime executions.

## 8. 32-bit arithmetic arms reached

The target cfg capture proves `target_pointer_width="32"`; the host capture
proves the differential is `target_pointer_width="64"`.

* `tests/payload_robustness.rs:398-404`, inside
  `structured_mutations_reach_meaningful_error_classes` at line 318, selected
  the 32-bit arm and required `Error::ImplausibleCount { count: usize::MAX,
  available: oversized_payload.len() }`. The target test passed. The host run
  selected lines 405-406 and passed the `UnexpectedEof` arm.
* `tests/imagery_net.rs:225-235`, inside
  `a_hostile_length_is_refused_before_allocating` at line 201, selected the
  32-bit arm and required `Error::ImplausibleCount { count: usize::MAX,
  available: PREFIX_LEN }`. The target test passed. The host run selected the
  `MAX_RECORD_LEN` text arm at lines 236-237 and passed.
* `src/imagery.rs:437-445` uses checked `usize` arithmetic for a hostile
  `65535 x 65535 x 24` ray grid; `src/imagery.rs:483-495` maps the overflow to
  `LengthOverflow`, and `src/imagery.rs:553-562` applies the same checked
  helper while decoding. `tests/imagery.rs:350-373` ran the hostile-grid
  assertion on target and host. Its assertion intentionally accepts either
  `UnexpectedEof` or `ImplausibleCount`; no narrower emitted variant is claimed
  from its quiet test output.
* `src/imagery.rs:776-800` checks prefix/header/payload addition overflow.
  The exact `u32::MAX` payload mutation is exercised by the
  `payload_robustness` test above, with the target and host branches recorded
  separately.

These are target test results, not host output relabeled as target output.
The ELF runner log, pointer-width control, and cfg output independently bind
the target arms.

## 9. Native host differential

The host example was built first with:

```text
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/cargo-target-host \
  cargo +1.98.1 build --locked --features imagery,net --example imagery_dump
```

It exited 0. `debug/examples/imagery_dump` is ELF64/x86-64, SHA-256
`0e9f3103c5d24751155984e330df50519d83665ea76980ec5d13e42a5dae1d18`, and its
`--max 0` smoke exited 0.

The final host all-target command used the same command shape as the target
without a target/linker/runner override and supplied the real host example via
`CARGO_BIN_EXE_imagery_dump`. It exited 0 with 176 runtime tests passed and no
failures. The separate host doctest command exited 0 with 1 executable and 10
compile-fail doctests passed. Full output is in
`logs/host-tests-final-pass.log` and `logs/host-doctests-final.log`.

## 10. Limits and unavailable checks

The first exact `cargo +1.98.1 fmt --all --check` attempt exited 1 because the
`rustfmt` component was not installed; that unchanged result is retained in
`logs/fmt-check.log`. Under the explicit bounded authorization,
`rustup +1.98.1 component add rustfmt` exited 0, and the exact fmt command was
rerun in `logs/fmt-check-after-rustfmt.log` with exit 0. The component
installation was toolchain-local; no system package state changed. Component
setup and state are retained in `logs/fmt-component-add.log` and
`logs/fmt-component-state.log`. The source remained clean throughout.

No hardware, Motive service, live bridge, external network fixture, emulator,
benchmark, ignored test, or system package installation was used. The example
smoke is deliberately `--max 0`; loopback socket coverage comes only from the
repository tests. Existing CI compile evidence and old runtime artifacts are
not substituted for this current native target evidence. Product lint,
dependency-audit, documentation, and unrelated feature gates were not rerun;
the E1 brief explicitly limits this procedure to the target runtime, host
differential, source fmt check, and integrity evidence.

The ten compile-fail doctests prove target compilation diagnostics, not target
execution. Only the one normal rustdoc doctest with the runner ELF inspection
is credited as target doctest runtime.

## 11. Reporting state and artifacts

The raw scratch remains unversioned under
`/home/andrej/coord_preserve/riders73-E1.ZUSSmh`. Package, wrapper, runtime
binary, and source hashes are retained under `manifests/`; each manifest
excludes itself and was verified with `sha256sum -c` after writing. The frozen
53-row raw-artifact manifest SHA-256 is
`b16c169bf4d9265a4946163165db868f50ecea4cf2a1cde7f301da624bff2cfc`.
Its 53 rows remain valid and self-excluding; verification is recorded in
`gates.md`. New fmt-log hashes are in
`manifests/fmt-supplement-20260927.sha256` (SHA-256
`bdc87a28ab41cfa6f27bfe0138a454233af9e83d28e1590f5e1db099191f3dfd`), and
the package-provenance addendum is self-excluding and verified by
`manifests/provenance-supplement-20260927.sha256` (SHA-256
`478dca6817f97adbc64780467e03dba1e88b81e14b75678a184997ebf7bc82c1`).

All published counts are bounded to the named final command outputs and
artifact populations; no grown-set claim reaches beyond those populations.
The runner log is a Cargo-runner invocation census only. The bounded
pre/post process-handle diagnostics were not evidence for runtime counts; no
system-wide process census is claimed.

This report and sibling `gates.md` are E1 drafts only. They are uncommitted;
coordinator freeze, packet commit, review, and push remain outside this task.
