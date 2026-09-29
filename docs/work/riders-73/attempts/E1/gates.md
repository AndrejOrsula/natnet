# riders-73 E1 — gates and raw evidence index

All commands below were run with `rtk` at the shell-command start. Raw output
is retained under `/home/andrej/coord_preserve/riders73-E1.ZUSSmh`; these gate
and report files are uncommitted drafts.

## Identity and source state

| item | result |
|---|---|
| procedure | `187f89d99ee808f771908268973a381fa6f52bcd` |
| source | commit `01302f32af0b6935f46fea4a690a4d35a6d5a407`, tree `8bfa45cd18838adfc122a53b5371f907332ea5bd` |
| source clone | detached clean clone at `/home/andrej/coord_preserve/riders73-E1.ZUSSmh/source` |
| source archive | `manifests/source-01302f3.tar`, SHA-256 `e2646aa3c98ea7a8c94d83c8ba3eaead6f9457ddfb99b7c10caffa1660c575e6` |
| lockfile | SHA-256 `a8a5b8f5462663d6722466ba154369614c0fd4b693135d99cffbf2eaf1c150f0` |
| source diff | `git diff --exit-code`: 0; cached diff: 0 |
| package install | none; only private extraction under scratch |
| packet state | no commit, push, or shared-doc mutation |

`logs/environment-final.txt` records UTC, kernel, OS, architecture, compiler,
Rust/Cargo versions, installed targets, disk, source hashes, private loader,
and package-hash verification. Final capture time was `2026-09-27T13:57:30Z`.

## Packages and sysroot

The first exact APT download used the configured Ubuntu archive/security URLs
and exited nonzero because the two exact glibc URLs returned 404; complete
output is `logs/apt-download.log`. The exact GCC archives were then fetched
from official Ubuntu archive URLs and the exact glibc archives from official
Ubuntu Snapshot `20260212T000000Z`. `logs/package-metadata.log` records local
APT candidate metadata. No third-party mirror or system install was used.

| archive | version | SHA-256 |
|---|---|---|
| `libc6-i386_2.39-0ubuntu8.7_amd64.deb` | `2.39-0ubuntu8.7` | `a80dfe7331c74bda498aefcd49c20e531977736e287db224d53e9915bdd6d509` |
| `lib32gcc-s1_14.2.0-4ubuntu2~24.04.1_amd64.deb` | `14.2.0-4ubuntu2~24.04.1` | `1802dc75345ccce457b44110a3819ca74ff8c33fc62394a78d0e59a11c0cf770` |
| `libc6-dev-i386_2.39-0ubuntu8.7_amd64.deb` | `2.39-0ubuntu8.7` | `3502744587e191d2b6c5fd5ce7f64784c9f6c94d270b1110dbc0993f6da19815` |
| `lib32gcc-13-dev_13.3.0-6ubuntu2~24.04.1_amd64.deb` | `13.3.0-6ubuntu2~24.04.1` | `0850e0555c86f2ce7a954158eb47df42d7645bfa4f12f29ea2df76459c930ac0` |
| extra `libc6-dev_2.39-0ubuntu8.7_amd64.deb` | `2.39-0ubuntu8.7` | `bbf5a155039042634961a61276650631ee47b9e721f91f8dbb731b0bbe046df3` |

Extraction used `dpkg-deb -x <archive> /home/andrej/coord_preserve/riders73-E1.ZUSSmh/sysroot` for each archive. The extra generic development archive was required because the specified `libc6-dev-i386` package declares `libc6-dev (= 2.39-0ubuntu8.7)` and the isolated root otherwise lacked generic headers. This is an explicit procedure deviation, not a silent substitution. `logs/package-provenance-addendum.log` records HTTP 200 for the exact three glibc paths under official Snapshot `https://snapshot.ubuntu.com/ubuntu/20260212T000000Z/`, HTTP 200 for the two exact GCC paths under official `fr.archive.ubuntu.com/ubuntu`, the `dpkg-deb -f` dependency fields, and the two development-package hashes. Scratch-only links were added for `sysroot/lib32 -> usr/lib32`, `sysroot/lib -> usr/lib`, and `sysroot/usr/lib/ld-linux.so.2 -> ../lib32/ld-linux.so.2`.

Verification:

```text
rtk proxy bash -c 'cd /home/andrej/coord_preserve/riders73-E1.ZUSSmh/apt && sha256sum -c ../manifests/package-download.sha256'
```

Result: all five rows `OK`.

## Rust target and wrappers

```text
rtk proxy rustup +1.98.1 target add i686-unknown-linux-gnu
```

Exit 0. Final installed list: `i686-unknown-linux-gnu`,
`x86_64-unknown-linux-gnu`.

`bin/i686-cc.sh` wraps `/usr/bin/cc` with the private `--sysroot`, GCC 13
32-bit `-B` directory, private dynamic linker, and the four required private
library directories. SHA-256:
`631cc18f7191701a2ebbe836a0d1fb63d6acd43aaec163a5b032eacf26739a19`.

`bin/i686-run.sh` invokes the private
`sysroot/usr/lib32/ld-linux.so.2 --library-path
sysroot/usr/lib32:sysroot/usr/lib/i386-linux-gnu`, records the invoked
executable's `file`/`readelf` identity, and exports the target example path
for `imagery_cli`. SHA-256:
`609d83df0db0dfc29bde3a5682d73052618cca810fa218f31495b4bb6727fdd5`.

The exact wrapper and C-control source hashes verify with:

```text
rtk proxy bash -c 'cd /home/andrej/coord_preserve/riders73-E1.ZUSSmh && sha256sum -c manifests/wrapper-sha256'
```

Result: three rows `OK`.

## Pointer-width controls

Source: `controls/pointer_width.c`, SHA-256
`6c3a04261b77ca0f475f0ca3f95edead616e53fbc5760cf3b41db566d904322d`.

| command | exit | decisive output |
|---|---:|---|
| `rtk proxy bin/i686-cc.sh -m32 -O2 controls/pointer_width.c -o controls/pointer_width-i686` | 0 | target binary built |
| `rtk proxy bin/i686-run.sh controls/pointer_width-i686 4` | 0 | `sizeof(void*)=4 expected=4 uint32_wrap=0` |
| `rtk proxy /usr/bin/cc -O2 controls/pointer_width.c -o controls/pointer_width-host` | 0 | host binary built |
| `rtk proxy env -u LD_LIBRARY_PATH controls/pointer_width-host 8` | 0 | `sizeof(void*)=8 expected=8 uint32_wrap=0` |

Target control SHA-256 is
`154afb96c7958587f5377ac9efe4e7adba2e5478107875ed99ae583b82b3e940`;
host control SHA-256 is
`3dc41c1d8d2e4ad8c8d6c96a5d28772c625c70d29179e3851b064b67dcdf286d`.
`logs/elf-inspection-final.txt` records both `file` and `readelf` output:
target `ELF32`/`Intel 80386` with the private interpreter; host `ELF64`/x86-64
with `/lib64/ld-linux-x86-64.so.2`.

`logs/target-cfg.txt` exits 0 and contains `target_pointer_width="32"`.
`logs/host-cfg.txt` exits 0 and contains `target_pointer_width="64"`.

## Example build and smoke

```text
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/cargo-target-i686 CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/bin/i686-cc.sh CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_RUNNER=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/bin/i686-run.sh cargo +1.98.1 build --locked --features imagery,net --example imagery_dump --target i686-unknown-linux-gnu
```

Exit 0; full output `logs/example-build-final.log`. The resulting target
example is ELF32/i386, private interpreter, BuildID
`944172e44ab55e30e2ebfe826bde3dd50d910470`, SHA-256
`0b318809eff9e036f999e282e1db7c79e533d38fb56a13d6dffa0872aa4fa7c0`.

```text
rtk proxy /home/andrej/coord_preserve/riders73-E1.ZUSSmh/bin/i686-run.sh /home/andrej/coord_preserve/riders73-E1.ZUSSmh/cargo-target-i686/i686-unknown-linux-gnu/debug/examples/imagery_dump --max 0
```

Final no-`LD_LIBRARY_PATH` smoke exit: 0; output length: 0 bytes. Raw output
and runner identity: `logs/example-smoke-final-no-ld.log` and
`logs/runner-invocations-example-smoke-final-no-ld.log`.

## Target all-target/all-feature runtime

Exact final command:

```text
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/cargo-target-i686 CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/bin/i686-cc.sh CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_RUNNER=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/bin/i686-run.sh cargo +1.98.1 test --locked --all-targets --all-features --target i686-unknown-linux-gnu --no-fail-fast -- --test-threads=1
```

Exit 0. Full unfiltered output: `logs/target-tests-final-no-ld.log`.

| executed target binary | passed | failed | ignored | measured | filtered |
|---|---:|---:|---:|---:|---:|
| `natnet` unit | 5 | 0 | 0 | 0 | 0 |
| `command` | 23 | 0 | 0 | 0 | 0 |
| `decoder` | 8 | 0 | 0 | 0 | 0 |
| `frame_versions` | 12 | 0 | 0 | 0 | 0 |
| `imagery` | 28 | 0 | 0 | 0 | 0 |
| `imagery_cli` | 6 | 0 | 0 | 0 | 0 |
| `imagery_net` | 23 | 0 | 0 | 0 | 0 |
| `modeldef_versions` | 13 | 0 | 0 | 0 | 0 |
| `net_command` | 12 | 0 | 0 | 0 | 0 |
| `net_data` | 6 | 0 | 0 | 0 | 0 |
| `optional_features` | 4 | 0 | 0 | 0 | 0 |
| `payload_robustness` | 6 | 0 | 0 | 0 | 0 |
| `robustness` | 9 | 0 | 0 | 0 | 0 |
| `roundtrip` | 17 | 0 | 0 | 0 | 0 |
| `serverinfo_encoding` | 4 | 0 | 0 | 0 | 0 |
| `imagery_dump` example harness | 0 | 0 | 0 | 0 | 0 |
| **runtime total over the 15 test binaries** | **176** | **0** | **0** | **0** | **0** |

The runner log has 16 invocations and 16 `Class: ELF32` records. Every
invocation reports machine `Intel 80386` and the private interpreter. The
runner log itself is `logs/runner-invocations-target-tests-final-no-ld.log`.

The manifest's 20 rows are partitioned as 15 test binaries plus the zero-test
example harness (16 runner-bound test artifacts), then the standalone target
example, two controls, and the host example (4 additional artifacts). The
manifest is verified by:

```text
rtk proxy bash -c 'cd /home/andrej/coord_preserve/riders73-E1.ZUSSmh && sha256sum -c manifests/runtime-binaries.sha256'
```

Result: 20 rows `OK`; the 16 runner-bound test artifacts and 4 additional
artifacts are distinct populations.

## Target doctests

Exact final command:

```text
rtk proxy env CARGO_TARGET_DIR=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/cargo-target-i686 CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/bin/i686-cc.sh CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_RUNNER=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/bin/i686-run.sh cargo +1.98.1 test --locked --all-features --doc --target i686-unknown-linux-gnu --no-fail-fast -- --test-threads=1
```

Exit 0; full output `logs/doctests-i686-final-no-ld.log`:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The first line is the executable doctest `src/lib.rs - (line 45)`. The other
ten are compile-fail doctests. `logs/runner-invocations-doctest-final-no-ld.log`
records the temporary `rust_out` as ELF32/i386 with the private interpreter;
therefore one target doctest executable actually ran. The ten compile-fail
cases are not runtime evidence.

## Host differential

The host example build:

```text
rtk proxy env -u LD_LIBRARY_PATH CARGO_TARGET_DIR=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/cargo-target-host cargo +1.98.1 build --locked --features imagery,net --example imagery_dump
```

Exit 0; host example is ELF64/x86-64, SHA-256
`0e9f3103c5d24751155984e330df50519d83665ea76980ec5d13e42a5dae1d18`.

Final host all-target command supplied the built host example path through
`CARGO_BIN_EXE_imagery_dump` and removed all target linker/runner overrides:

```text
rtk proxy env -u LD_LIBRARY_PATH -u CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER -u CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_RUNNER CARGO_BIN_EXE_imagery_dump=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/cargo-target-host/debug/examples/imagery_dump CARGO_TARGET_DIR=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/cargo-target-host cargo +1.98.1 test --locked --all-targets --all-features --no-fail-fast -- --test-threads=1
```

Exit 0; `logs/host-tests-final-pass.log` records 176 runtime tests passed,
0 failed, 0 ignored, 0 measured, 0 filtered.

Separate host doctests:

```text
rtk proxy env -u LD_LIBRARY_PATH -u CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER -u CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_RUNNER CARGO_TARGET_DIR=/home/andrej/coord_preserve/riders73-E1.ZUSSmh/cargo-target-host cargo +1.98.1 test --locked --all-features --doc --no-fail-fast -- --test-threads=1
```

Exit 0; `logs/host-doctests-final.log` records 1 executable and 10
compile-fail doctests passed.

The first host all-target attempt exited 101 because the example fallback
path did not exist; the second attempt exited 101 because it selected Cargo's
hashed example test harness. Those raw outputs are preserved as
`logs/host-tests-final.log` and `logs/host-tests-final-with-example-env.log`.
They are harness setup failures, not final host product results.

## 32-bit arms and source reach

| source arm | target result | host result |
|---|---|---|
| `tests/payload_robustness.rs:398-404` (`structured_mutations_reach_meaningful_error_classes`, line 318) | `target_pointer_width=32`; exact `ImplausibleCount` assertion passed | `target_pointer_width=64`; `UnexpectedEof` arm at 405-406 passed |
| `tests/imagery_net.rs:225-235` (`a_hostile_length_is_refused_before_allocating`, line 201) | exact `ImplausibleCount { count: usize::MAX, available: PREFIX_LEN }` arm passed | `MAX_RECORD_LEN` text arm at 236-237 passed |
| `src/imagery.rs:437-445`, `483-495`, `553-562`; `tests/imagery.rs:350-373` | checked hostile ray-grid arithmetic test passed; assertion permits `UnexpectedEof` or `ImplausibleCount` | same test passed; no narrower variant claimed |
| `src/imagery.rs:776-800` record-length checked additions | reached by target payload mutation and passed | reached by host payload mutation and passed |

The source line sweep was read from the fixed clone after runtime execution;
no source repair or test edit was made.

## Fmt, integrity, and limits

```text
rtk proxy cargo +1.98.1 fmt --all --check
```

The first exact attempt exited 1 because `cargo-fmt` was not installed for
toolchain 1.98.1; raw output is retained in `logs/fmt-check.log`. Under the
explicit bounded authorization, `rtk proxy rustup +1.98.1 component add
rustfmt` exited 0 (`logs/fmt-component-add.log`), and the exact fmt command
was rerun with exit 0 (`logs/fmt-check-after-rustfmt.log`). Installed-component
state is retained in `logs/fmt-component-state.log`. The source remained
unchanged and clean.

No benchmark or ignored-test command was run. No hardware, Motive, live
bridge, emulator, or system package installation was used. Existing CI and
historical runtime records are not counted as E1 target runtime evidence.

Hash checks after writing the package, wrapper, and runtime manifests returned
all rows `OK`. The original 53-row raw-artifact manifest is intentionally
unchanged after the authorized addenda; it excludes itself and is verified
with the command below. New logs are covered by the supplemental manifests
below rather than retroactively added to this frozen hash list.

```text
rtk proxy bash -c 'cd /home/andrej/coord_preserve/riders73-E1.ZUSSmh && sha256sum -c manifests/raw-artifacts.sha256'
```

Result: 53 rows `OK`, including `source/Cargo.lock`; the manifest does not
hash itself. The frozen raw manifest SHA-256 is
`b16c169bf4d9265a4946163165db868f50ecea4cf2a1cde7f301da624bff2cfc`.

New fmt logs are covered by self-excluding
`manifests/fmt-supplement-20260927.sha256`, SHA-256
`bdc87a28ab41cfa6f27bfe0138a454233af9e83d28e1590f5e1db099191f3dfd`; all
three rows verify `OK`. The package provenance addendum is covered by
self-excluding `manifests/provenance-supplement-20260927.sha256`, SHA-256
`478dca6817f97adbc64780467e03dba1e88b81e14b75678a184997ebf7bc82c1`; its
one row verifies `OK`.

## Reporting sweep

The E1 drafts contain no candidate code change and no finding that retracts a
prior citation, name, or number. Therefore the three-leg refuted-citation
sweep is not applicable: there is no candidate diff, no candidate commit
message object, and no historical claim being repaired here. Counts in these
drafts state their domain and unit in the same sentence or table row: target
versus host, test binary population, runtime versus compile-only doctest, and
ELF invocation population. These counts are bounded to the final command
outputs; no grown-set claim reaches beyond them. The runner log is a
Cargo-runner invocation census only. Bounded pre/post process-handle
diagnostics were not evidence for runtime counts; no system-wide process
census is claimed. No historical evidence is invented.
