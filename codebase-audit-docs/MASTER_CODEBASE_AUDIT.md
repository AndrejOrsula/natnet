# Master Codebase Audit

**Date:** 2026-09-08
**Root:** `/home/andrej/ws_rs/natnet`
**Revision:** `eb8631b378808bbc542f3f61252eb77cd51f995e`
**Mode:** Architect and QA. Source, tests, configuration, lockfile and status files unchanged.

## Executive summary

The crate has a coherent dependency-free core and working NatNet 4.1 and AUGI/1 paths. Current code passed existing real 4.1 fixture replay and every configured CI build, lint and test gate. Its broad version compatibility and command-safety claims exceed what the implementation enforces.

| Measure | Result | Meaning |
|---|---|---|
| Health | **5.5 / 10** | Audit judgment using the rubric below, not a measured reliability probability |
| Release readiness | **50%: 6 of 12 criteria met** | Explicit technical acceptance checklist, not a percentage of all possible product features |
| Findings | **3 P1, 7 P2, 1 P3** | Includes confirmed defects, contract risks and test gaps |
| All-feature tests | **80 ordinary tests + 1 compile-fail doctest passed** | Also passed on MSRV 1.92; none ignored |
| Default tests | **49 ordinary tests + 1 doctest passed** | Zero optional dependencies activated |
| Real fixture replay | **25 frames + SERVERINFO + MODELDEF passed** | Current revision, existing private 4.1 subsets |
| Strict documentation | **Five feature profiles failed** | Default, imagery, net, serde, nalgebra |
| Cross-target checks | **Three passed** | Windows GNU/MSVC and Linux musl compilation; no native execution |
| Dependencies | **Audit passed against refreshed RustSec HEAD** | No unused direct dependency; two unmatched-license allowances |

The highest-priority work is version-correct frame/MODELDEF decoding, command-component validation, and handshake peer validation. All findings remain open because this audit intentionally changes no implementation.

### Scoring and readiness

| Health dimension | Score / 2 | Evidence |
|---|---:|---|
| Structure | 1.50 | Shared checked primitives, plain models, optional transports; policy/protocol coupling |
| Protocol correctness | 1.00 | Current 4.1 replay passes; independent older-version layouts fail |
| Safety/trust | 0.50 | Command injection, raw-send contract bypass and foreign-peer acceptance reproduced |
| Verification | 1.25 | Feature/MSRV gates strong; random tests miss typed payloads; new defects lack regressions |
| Infrastructure/docs | 1.25 | Exact-revision CI and dependencies healthy; partial docs broken, action refs mutable |
| **Total** | **5.50 / 10** | Judgment at the audited revision |

| Criterion | State | Proof |
|---|---|---|
| R1: default/all/MSRV tests | Met | 50 / 81 / 81 results including doctests |
| R2: format and configured strict lint gates | Met | Host and five feature rows pass |
| R3: configured optional feature combinations | Met | imagery, net, serde, nalgebra, imagery,net |
| R4: current dependency policy/use | Met | Refreshed cargo-deny and cargo-machete |
| R5: current real 4.1 subset compatibility | Met | 27 packets replayed and value-round-tripped |
| R6: AUGI framing/join/normal stream | Met | Integration tests and CLI loopback controls |
| R7: independently verified advertised versions | Unmet | B01, B08 |
| R8: command-safety boundary | Unmet | B02, B04; live Tier-1 acceptance also outstanding |
| R9: handshake peer identification | Unmet | B03 |
| R10: stream/CLI error behavior | Unmet | B06, B07 |
| R11: strict docs in supported profiles | Unmet | B05 |
| R12: meaningful critical-path regressions | Unmet | B09, B10 and coverage gaps |

100% means meeting these twelve criteria with fresh evidence. It does not prove absence of every possible defect. Publication additionally follows the existing D54 release decision.

## Scope and specification

| Area | Inspected scope |
|---|---|
| Inventory | 31 tracked files; 14 source files / 4,085 lines; six integration tests / 2,608 lines; one example / 180 lines |
| Source | `bytes`, `command`, `error`, `frame`, `generate`, `imagery`, `imagery/net`, `lib`, `message`, `modeldef`, `nalgebra_conv`, `net`, `serverinfo`, `version` |
| Tests | All six test files, generator unit tests and grant compile-fail doctest |
| Documentation/config | README, Cargo manifest/lockfile, toolchain, Clippy, deny policy, workflow, ignore policy, license files |
| Planning | No local PRD, architecture, contributor or status file. Parent `../STATUS.md` is a prior global lint handoff |
| Referenced decisions | [D47](/home/andrej/ws_rs/auga/docs/07-decisions.md:733), [D54](/home/andrej/ws_rs/auga/docs/07-decisions.md:852), relevant P2.1b/P2.3b evidence |
| Independent authority | Official OptiTrack SDK 4.1.1 samples, downloaded outside the repository and hashed |
| Runtime | Linux host, loopback peers, deterministic malformed payloads, existing small private capture subsets |
| Not applicable | HTTP routes, UI, DB/schema/migrations, application deployment, GPU/visual verification, generated-source drift |
| Not performed | Source fixes, benchmarks, installations, hardware commands, commits, pushes, PRs, full sibling-codebase audit |

D54 assigns this crate the protocol seam, optional UDP client and D47 command tiers. Clock estimation, pose buffering, coordinate transforms and filtering tracking-invalid bodies belong to `auga_io`. It keeps real captures in private Auga evidence and delays public release until Auga v1. Public real-capture absence and package version `0.0.0` are intentional development state. [D54 scope and release policy](/home/andrej/ws_rs/auga/docs/07-decisions.md:866).

Eleven bounded Luna investigations covered infrastructure, three feature areas, protocol sources, two architecture areas, host gates, coverage, platforms and external evidence. The primary verified consequential claims and ran independent reproductions. A final independent Luna review checked finding counts, acceptance arithmetic, logs, reproduction commands and source consistency. Its wording correction is incorporated.

## Infrastructure and dependency health

| Component | Current evidence | Assessment |
|---|---|---|
| Stack | Single Rust 2024 Cargo library; MSRV 1.92; [manifest](/home/andrej/ws_rs/natnet/Cargo.toml:1) | Appropriate small stack |
| Toolchain | `stable` in [toolchain file](/home/andrej/ws_rs/natnet/rust-toolchain.toml:2); audited version 1.98.0 | Moving stable can introduce lint drift; explicit `cargo +1.92 test --all-features` genuinely checks MSRV |
| Default dependencies | Committed lockfile; default cargo tree contains only natnet | Zero-dependency claim verified |
| Optional dependencies | serde 1.0.229, nalgebra 0.35.0 and transitives | Both used; cargo-machete finds no unused direct dependency. Locked glam records are absent from the activated all-feature graph |
| Dependency policy | [deny.toml](/home/andrej/ws_rs/natnet/deny.toml:8), seven target graph filters | Advisories/bans/licenses/sources pass; BSD-2-Clause/BSD-3-Clause allowances unused |
| Advisory freshness | Explicit fetch; local and remote HEAD both `8a1eb4f933fb5821add5b4e98601ebd90b8b3538` | Current when checked; not a guarantee about future advisories |
| CI | Push main/dev, PR/manual; fmt, Clippy, tests, feature matrix, net build, all-feature docs, MSRV, deny | [Workflow](/home/andrej/ws_rs/natnet/.github/workflows/rust.yml:26) implements these gates |
| Actual CI | Exact audited SHA; all seven jobs and step conclusions successful | [Run 34054099546](https://github.com/SpaceRL/natnet_private/actions/runs/34054099546) |
| Remote identity | API resolves manifest URL to private `SpaceRL/natnet_private`; dev default; `allow_forking=false` | Review intended metadata before publication; arbitrary public-fork exposure not demonstrated |
| Actions trust | API: enabled, `allowed_actions=all`, `sha_pinning_required=false`; mutable `@master`/`@v5`/`@v2` refs | Pin reviewed action SHAs |
| Self-hosted fleet | Successful jobs on six Linux X64 hosts; repository-level runners endpoint lists zero | Zero repository runners does not mean no fleet. Machine isolation/network reach not audited |
| Protections | Branch-protection API HTTP 403, upgrade required; fork approval HTTP 422 for private repo | Protection settings verification unavailable |
| Platforms | Linux runtime; Windows GNU/MSVC and musl compile checks pass | No native cross-platform or multicast runtime evidence |
| Absent infrastructure | No Docker/deployment/env templates | Appropriate to a library; unrelated to identified defects |

GitHub recommends immutable action SHA references and protecting self-hosted environments from untrusted workflow code. The observed private/no-fork settings reduce the public-fork scenario; they do not prove runner isolation. [GitHub secure-use guidance](https://docs.github.com/en/actions/reference/security/secure-use).

## Feature completion matrix

**Done:** implemented/verified in stated scope. **WIP:** present with a material correctness, documentation or acceptance gap. **Missing:** absent audit-required regression evidence.

| Feature | State | Evidence / limits |
|---|---|---|
| Dependency-free, convention-neutral core | Done | [Manifest](/home/andrej/ws_rs/natnet/Cargo.toml:13), [reader](/home/andrej/ws_rs/natnet/src/bytes.rs:35); crate forbids unsafe code |
| Header/dispatch/unknown IDs | Done | [Dispatch](/home/andrej/ws_rs/natnet/src/lib.rs:154); real id 1/5/7 replay. Unknown payload preserved; declared header length deliberately telemetry-only |
| SERVERINFO fixture shape | Done | [Parser](/home/andrej/ws_rs/natnet/src/serverinfo.rs:79); current real fixture and value round-trip pass |
| Frame versions | WIP | [Version gate](/home/andrej/ws_rs/natnet/src/frame.rs:348); 4.1 works, independent 3.0/4.0 fails |
| MODELDEF/name resolution | WIP | [Parser](/home/andrej/ws_rs/natnet/src/modeldef.rs:165); 4.1 real fixture passes; no version argument for size-prefixed layout |
| Qualified parsed-value round-trip | Done | [Contract](/home/andrej/ws_rs/natnet/src/lib.rs:22), 17 tests and private replay; NaN/string/constructed-value caveats explicit |
| Synthetic generation | Done for 4.1 | [Generator](/home/andrej/ws_rs/natnet/src/generate.rs:136); deterministic poses, dropout bits, clock offset; legacy output shares B01 |
| Optional serde | WIP in docs/verification | Core derives compile; broad “public wire types” wording excludes Message, MessageHeader and imagery types; no dedicated behavior tests |
| Optional nalgebra | Done in one-way scope | [Conversions](/home/andrej/ws_rs/natnet/src/nalgebra_conv.rs:14); Point3/Quaternion from RigidBody, marker vector; no reverse conversion or ordering regression |
| Connect/modeldef/GetProperty builders | Done at API/byte level | [Builders](/home/andrej/ws_rs/natnet/src/command.rs:55), loopback tests; no fresh live execution |
| Tier-1 canary/re-read | WIP | [State machine](/home/andrej/ws_rs/natnet/src/command.rs:334); normal mocks pass; injection breaks property gate; logging/per-target reread caller-owned |
| Tier-2 authorization | WIP | [Grant](/home/andrej/ws_rs/natnet/src/command.rs:244); explicit token path exists; raw transport unrestricted |
| Response decoding | Done with documented ambiguity | [Response API](/home/andrej/ws_rs/natnet/src/command.rs:594); three-byte collisions and context-driven parser tested |
| DataClient | WIP in coverage | [Client](/home/andrej/ws_rs/natnet/src/net.rs:35); unicast/source/nonblocking audit smoke passes; multicast runtime not verified |
| Command UDP/handshake | WIP | [Handshake](/home/andrej/ws_rs/natnet/src/net.rs:202); intended-peer control passes; foreign peer accepted |
| AUGI records/extensions | Done | [Parser](/home/andrej/ws_rs/natnet/src/imagery.rs:811); typed and unknown records, appended headers tested |
| AUGI fallible encoding | Done within refusal table | [Contract](/home/andrej/ws_rs/natnet/src/imagery.rs:64); length/grid/NUL checks; NaN/Other aliases intentionally accepted |
| AUGI/NatNet frame join | Done | [Helpers](/home/andrej/ws_rs/natnet/src/imagery.rs:625); signed-boundary bit reinterpretation tested |
| Imagery TCP/CLI | WIP | [Client](/home/andrej/ws_rs/natnet/src/imagery/net.rs:60), [example](/home/andrej/ws_rs/natnet/examples/imagery_dump.rs:20); normal/reverse pass; error handling defects |
| Independent compatibility evidence | WIP | Real 4.1 subsets and one literal AUGI golden; older versions, nonempty raw sections and actual bridge need stronger evidence |
| New defect regressions | Missing | No committed tests protecting B01–B08; audit probes do not modify regression suites |

Skeletons, assets, force plates, devices, camera datasets and rigid-body tails are documented opaque/structural scope. Do not present them as fully typed decoding. Codec decoding is expressly out of scope.

## Architecture and technical debt

| Boundary | Current design | Assessment |
|---|---|---|
| Primitives | Checked slice cursor and append-only writer | Strong shared foundation; input arithmetic bounded |
| NatNet models | Owned strings/vectors, plain arrays, RawSection, boxed frame | Clear ownership; per-message allocations |
| Generator | Uses frame serializer, no RNG/clock/socket | Deterministic; shared serializer/parser defect can fool self-roundtrips |
| Imagery | Owned Hello strings; borrowed image/ray/name data; separate record head | Useful zero-copy seam |
| Commands | Wire formatting plus lab-specific tiers | Provenance erased by mutable Vec/raw slice output |
| Transport | Private synchronous sockets; no worker/queue/DB | Appropriate scope; transaction ownership and partial TCP progress not modeled |
| Features | Default-off modules/dependencies | Default graph isolated; behavioral tests uneven |

Inspected imports and the single-package graph show no apparent dependency cycle. Rust-LSP initially failed readiness, then recovered for narrow symbol/reference and signature checks. No exhaustive semantic graph claim is made.

| Debt | Evidence | Recommended direction |
|---|---|---|
| Caller owns negotiated layout | [parse_message](/home/andrej/ws_rs/natnet/src/lib.rs:154), [to_payload](/home/andrej/ws_rs/natnet/src/frame.rs:465) | Small supported-layout/session seam; parsed frames do not retain version and can be reserialized with incompatible settings |
| Typed section checked after parsing | [typed_section](/home/andrej/ws_rs/natnet/src/frame.rs:281) | Size-limited sub-reader. Current code remains input-bounded but may read later sections/allocate before rejecting |
| Inconsistent strictness | [raw_section](/home/andrej/ws_rs/natnet/src/frame.rs:303), [MODELDEF](/home/andrej/ws_rs/natnet/src/modeldef.rs:187) | Specify count/terminator/tail policy; retain intentional compatibility |
| Infallible NatNet vs fallible AUGI encoding | [framer](/home/andrej/ws_rs/natnet/src/message.rs:87), [string writer](/home/andrej/ws_rs/natnet/src/bytes.rs:201) | Add validated/fallible output for faithful arbitrary-value encoding, with compatible migration |
| Cooperative canary/grant | [authority](/home/andrej/ws_rs/natnet/src/command.rs:248), [mutable packets](/home/andrej/ws_rs/natnet/src/command.rs:433) | Bind validated metadata to safe transport; state authentication/logging responsibilities honestly |
| TCP partial errors | [timeout docs](/home/andrej/ws_rs/natnet/src/imagery/net.rs:114), [reads](/home/andrej/ws_rs/natnet/src/imagery/net.rs:138) | Retain incremental progress or explicitly terminate client after partial failure; distinguish clean EOF |
| Lossy error taxonomy | [Error](/home/andrej/ws_rs/natnet/src/error.rs:14), [magic handling](/home/andrej/ws_rs/natnet/src/imagery.rs:771) | Preserve encode/decode context and full magic; interior NUL currently displays “unterminated string” |
| Allocation/copy path | [section writer](/home/andrej/ws_rs/natnet/src/frame.rs:311), [dataset writer](/home/andrej/ws_rs/natnet/src/modeldef.rs:227), [datagram copy](/home/andrej/ws_rs/natnet/src/message.rs:87) | Consider reused buffers/write-into APIs after authorized profiling |
| Linear name lookup | [resolve_rigid_body](/home/andrej/ws_rs/natnet/src/modeldef.rs:263) | Consumer cache by model generation if needed; preserve invalidation |
| External/stale prose | D47/D54 references, receive-only introductions | Local support matrix, trust boundary, error lifecycle and consumer responsibilities |

ImageClient bounds a record at 64 MiB, then zero-fills and blocks for its declared bytes. A per-read timeout is not a total transaction deadline. Backpressure, reconnect and session identity remain consumer responsibilities. No measured latency/throughput regression is claimed; benchmarks were prohibited.

## Prioritized bug and security tracker

**P1:** before broad compatibility/safety reliance. **P2:** before release acceptance or explicit bounded disposition. **P3:** maintenance follow-up. All findings are **Open**.

| ID | Priority / kind | Location | Reproduction / impact | Resolution |
|---|---|---|---|---|
| B01 | **P1: protocol correctness** | [frame.rs:261](/home/andrej/ws_rs/natnet/src/frame.rs:261), [frame.rs:386](/home/andrej/ws_rs/natnet/src/frame.rs:386), [modeldef.rs:165](/home/andrej/ws_rs/natnet/src/modeldef.rs:165), [lib.rs:154](/home/andrej/ws_rs/natnet/src/lib.rs:154) | Implementation uses section sizes for all accepted versions and gates assets at 4.0; the official layout introduces both at 4.1. Independent 78-byte empty 3.0/4.0 frames fail at offset 76; old 14-byte marker-set MODELDEF fails at offset 12 | Versioned framing and legacy structural walkers; explicit unsupported-layout policy. Support narrowing requires a compatibility decision |
| B02 | **P1: command injection** | [command.rs:79](/home/andrej/ws_rs/natnet/src/command.rs:79), [command.rs:89](/home/andrej/ws_rs/natnet/src/command.rs:89), [command.rs:377](/home/andrej/ws_rs/natnet/src/command.rs:377), [command.rs:580](/home/andrej/ws_rs/natnet/src/command.rs:580) | Allowed Enabled write, target `,Transmission Type,1\0`, produces effective `SetProperty,,Transmission Type,1` without grant. Unsafe caller input changes the actual wire property | Reject comma/NUL-bearing components before encoding; validated grammar and canonical property names; typed/stringly regression cases |
| B03 | **P1: peer validation** | [net.rs:202](/home/andrej/ws_rs/natnet/src/net.rs:202) | SERVERINFO queued by a different loopback port accepted as configured server; `_from` discarded | Peer filtering, total deadline and transaction/read ownership; unrelated packets must not alter negotiated identity/version |
| B04 | **P2: contract mismatch** | [net.rs:16](/home/andrej/ws_rs/natnet/src/net.rs:16), [net.rs:119](/home/andrej/ws_rs/natnet/src/net.rs:119), [net.rs:178](/home/andrej/ws_rs/natnet/src/net.rs:178) | Manually built Tier-2 bytes transmitted to loopback without grant despite stronger docs | Validated request type for safe send or explicit unrestricted raw API and corrected guarantee |
| B05 | **P2: docs build** | [lib.rs:12](/home/andrej/ws_rs/natnet/src/lib.rs:12), [lib.rs:68](/home/andrej/ws_rs/natnet/src/lib.rs:68), [lib.rs:73](/home/andrej/ws_rs/natnet/src/lib.rs:73), [lib.rs:81](/home/andrej/ws_rs/natnet/src/lib.rs:81), [lib.rs:85](/home/andrej/ws_rs/natnet/src/lib.rs:85), [command.rs:8](/home/andrej/ws_rs/natnet/src/command.rs:8) | Strict rustdoc: default 6, imagery 5, net 2, serde 6, nalgebra 6 unresolved links. CI deliberately omits them | Feature-aware links/prose; strict docs matrix |
| B06 | **P2: false success** | [imagery_dump.rs:95](/home/andrej/ws_rs/natnet/examples/imagery_dump.rs:95), [imagery/net.rs:138](/home/andrej/ws_rs/natnet/src/imagery/net.rs:138) | Four prefix bytes then EOF, or missing declared image payload, prints `stream closed` and exits 0 | Distinguish clean EOF from partial record; truncation must fail |
| B07 | **P2: CLI validation** | [imagery_dump.rs:28](/home/andrej/ws_rs/natnet/examples/imagery_dump.rs:28), [imagery_dump.rs:39](/home/andrej/ws_rs/natnet/examples/imagery_dump.rs:39) | `--max garbage` and absent value become usize::MAX; process entire stream and exit 0 | Reject missing/invalid/overflow values before socket setup |
| B08 | **P2: version policy** | [frame.rs:348](/home/andrej/ws_rs/natnet/src/frame.rs:348), [version.rs:51](/home/andrej/ws_rs/natnet/src/version.rs:51), [error.rs:82](/home/andrej/ws_rs/natnet/src/error.rs:82) | Current-layout packet succeeds as Version 99.0; this proves permissive acceptance, not that every future layout differs | Enumerate supported layouts or explicit opt-in future policy; align UnsupportedVersion docs |
| B09 | **P2: test blind spot** | [robustness.rs:84](/home/andrej/ws_rs/natnet/tests/robustness.rs:84), [command.rs:547](/home/andrej/ws_rs/natnet/tests/command.rs:547) | PRNG replay: zero ids 1/5/7 in 2,000 core trials; zero id 3 in 4,000 command trials. Typed payload logic not entered | Payload fuzzing and valid-envelope mutation; assert meaningful reach and error classes |
| B10 | **P2: validation debt** | [frame.rs:303](/home/andrej/ws_rs/natnet/src/frame.rs:303), [frame.rs:426](/home/andrej/ws_rs/natnet/src/frame.rs:426), [modeldef.rs:187](/home/andrej/ws_rs/natnet/src/modeldef.rs:187) | Negative raw count, nonzero terminator and frame suffix junk accepted; MODELDEF marker-set residue dropped. No OOB demonstrated | Define/test strict vs lenient policy; bound typed sections before parsing; preserve outer-length tolerance |
| B11 | **P3: docs drift** | [message.rs:12](/home/andrej/ws_rs/natnet/src/message.rs:12), [generate.rs:32](/home/andrej/ws_rs/natnet/src/generate.rs:32), [generate.rs:75](/home/andrej/ws_rs/natnet/src/generate.rs:75), [frame.rs:3](/home/andrej/ws_rs/natnet/src/frame.rs:3) | NAT_CONNECT is sent; dropout bodies retained; fixture frames are 260-byte datagrams / 256-byte payloads, not 260-byte payloads | Correct public descriptions; gap filtering belongs to Auga |

Official SDK evidence for B01: `PacketClient.cpp:2656–2671` gates section sizes at 4.1+, `2126–2130` gates assets at 4.1+, and `PythonClient/NatNetClient.py:1390–1405` gates MODELDEF sizes. Current 4.1 layout and precision timestamps agree with the C++ sample. [Official SDK archive](https://s3.amazonaws.com/naturalpoint/software/NatNetSDK/NatNet_SDK_4.1.1.zip).

B02 follows the documented comma-delimited command grammar. The exact restricted command is also in the D47 incident record. Only bytes and loopback transmission were tested; no live disruption was attempted. [Official command syntax](https://docs.optitrack.com/developer-tools/natnet-sdk/natnet-remote-requests-commands), [D47](/home/andrej/ws_rs/auga/docs/07-decisions.md:747).

### Security conclusions

| Surface | Conclusion |
|---|---|
| Memory safety | No panic observed in current tests or 10,000 extra payload cases; not a proof against every input/allocation failure |
| Credentials | No common hardcoded credential, key, URL credential or provider-token pattern found in current inspected files; history/secret stores/unknown encodings not cleared |
| Authorization | Cooperative grants/canaries are not authentication. Mutable packets, cloned state and separately supplied authority permit caller misuse |
| UDP | Handshake source filtering absent; source filtering would still not provide cryptographic on-path authentication |
| Imagery | Connected peer is not authenticated bridge identity; Hello/order/session validation remains consumer-owned |
| Resources | Finite image cap; blocking/partial-error handling and avoidable allocation amplification remain concerns |
| Web OWASP | SQL injection, XSS, CSRF and HTTP access-control findings not applicable |
| CI | Mutable refs and unverified self-hosted isolation are risks; no runner compromise or exposed secret demonstrated |

## Test coverage and dead code

No numeric line/branch coverage measured. Counts are actual test results, not branch coverage.

| Suite | Passed | Coverage | Missing protection |
|---|---:|---|---|
| Generator unit | 5 | Determinism, dropout, clock offset | Extreme domain/version limits |
| command | 18 | Grants, canary/mismatch, response ambiguity | Injection, wire/state binding, meaningful payload fuzz |
| imagery | 24 | Records, truncation, refusals, extensions, literal golden, wrap join | Explicit NaN/Other serializer cases, short rays, 32-bit branches |
| imagery_net | 5 | Normal/reverse, wrong interface, cap, bad magic | Partial EOF, timeout/recovery, deterministic fragmentation |
| net_command | 2 | Intended-peer hello/GetProperty | Foreign/stale/malformed peers, concurrent reads, timeouts; DataClient suite absent |
| robustness | 9 | Typed-section size controls, hostile count, no-panic wrappers | Deep MODELDEF/SERVERINFO mutation, raw counts/terminators/tails |
| roundtrip | 17 | Synthetic values and explicit caveats | Independent legacy packets and nonempty raw version boundaries |
| doctest | 1 | Grant private field cannot be constructed | Does not constrain raw send or grammar |
| **Total** | **81** | **80 ordinary + 1 doctest** | None ignored |

Core random trials: 1,991 complete headers, all unknown ids, plus nine short headers. Command trials: 3,620 complete headers, none id 3, plus 380 short headers. The frame-prefix sweep also supplies id 7 to SERVERINFO/MODELDEF, so those payload parsers are not reached. Counts independently recomputed from exact PRNGs.

Audit probes strengthen current evidence for payload panic resistance, private id 1/5/7 dispatch, DataClient unicast, malformed CLI closure and command defects. They do not repair persistent regression coverage. Serde derives compile but have no dedicated behavior tests; nalgebra lacks ordering tests.

| Question | Result |
|---|---|
| Unused direct dependencies | None found by cargo-machete or feature/source inspection |
| Error::TrailingBytes | Rust-LSP references only declaration line 93 and Display arm line 128; no construction in this workspace |
| Old f32 timestamp decode | Unreachable behind major < 3 rejection; corresponding serializer branch is reachable |
| Public helpers unused locally | Missing local reference is not proof of no external users; no deletion proposed from grep alone |
| Source generation | None; generate.rs emits runtime packets, not generated source |

## Verification ledger

Shell invocations used `rtk`. Cargo build/test/check outputs went outside the repository. Host logs: `/tmp/natnet-qa.2W2Y9N/`. Stable target: `target` under that directory; MSRV: `target-msrv`. Delegate used `rtk proxy bash -c` wrappers, preserved the actual command exit status, and recorded these effective commands:

| Log | Effective command | Exit | Result |
|---|---|---:|---|
| 01 | `cargo fmt --all --check` | 0 | Pass |
| 02 | `cargo clippy --locked --offline --all-targets --all-features -- -D warnings` | 0 | Pass |
| 03 | `cargo test --locked --offline --all-features` | 0 | 81 passed |
| 04 | `cargo test --locked --offline` | 0 | 50 passed |
| 05 | `cargo build --locked --offline --features net` | 0 | Pass |
| 06 | `RUSTDOCFLAGS="-D warnings" cargo doc --locked --offline --no-deps --all-features` | 0 | Pass |
| 07 | `cargo +1.92 test --locked --offline --all-features` | 0 | 81 passed |
| 08–09 | Strict Clippy + test, `--features imagery` | 0 / 0 | 74 tests |
| 10–11 | Strict Clippy + test, `--features net` | 0 / 0 | 52 tests |
| 12–13 | Strict Clippy + test, `--features serde` | 0 / 0 | 50 tests |
| 14–15 | Strict Clippy + test, `--features nalgebra` | 0 / 0 | 50 tests |
| 16–17 | Strict Clippy + test, `--features imagery,net` | 0 / 0 | 81 tests |
| 18 | Strict doc default | 101 | 6 broken links |
| 19 | Strict doc imagery | 101 | 5 broken links |
| 20 | Strict doc net | 101 | 2 broken links |
| 21 | Strict doc serde | 101 | 6 broken links |
| 22 | Strict doc nalgebra | 101 | 6 broken links |
| 23 | Strict doc imagery,net | 0 | Pass |
| 24 | `cargo-deny --locked --all-features --manifest-path Cargo.toml check advisories bans licenses sources` | 0 | Four pass; two unmatched BSD allowances |
| 25 | `cargo machete` | 0 | No unused direct dependency |

Exact matrix command forms, shown with imagery; substitute each listed feature value. Omit feature option for default docs:

```sh
rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-qa.2W2Y9N/target cargo clippy --locked --offline --all-targets --features imagery -- -D warnings
rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-qa.2W2Y9N/target cargo test --locked --offline --features imagery
rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-qa.2W2Y9N/target RUSTDOCFLAGS="-D warnings" cargo doc --locked --offline --no-deps --features imagery
```

| Extra check | Command / evidence | Result |
|---|---|---|
| Default graph | `rtk proxy cargo tree --locked --offline --no-default-features` | Exit 0, natnet only |
| Optional graph | `rtk proxy cargo tree --locked --offline --all-features` | Exit 0 |
| Advisory refresh | `rtk proxy cargo deny fetch db` | Exit 0 |
| Local advisory identity | `rtk proxy git -C /home/andrej/.cargo/advisory-dbs/advisory-db-3157b0e258782691 log -1 --format='%H %cI'` | Exit 0, `8a1eb4f933fb5821add5b4e98601ebd90b8b3538`, 2026-09-07 |
| Remote identity | `rtk proxy git ls-remote https://github.com/RustSec/advisory-db.git HEAD` | Exit 0, same hash |
| Fresh dependency check | `rtk proxy env CARGO_TARGET_DIR=/tmp/natnet-audit-probes-20260908-6l4uyjk1/deny-target cargo-deny --locked --all-features --manifest-path Cargo.toml check advisories bans licenses sources` | Exit 0, four pass |
| Windows GNU | `rtk proxy env CARGO_TARGET_DIR=/tmp/natnet_phase4_platform_20260908/cargo-target cargo check --locked --offline --all-targets --all-features --target x86_64-pc-windows-gnu` | Exit 0 |
| Windows MSVC | Same command, `--target x86_64-pc-windows-msvc` | Exit 0 |
| Linux musl | Same command, `--target x86_64-unknown-linux-musl` | Exit 0 |
| Source preservation | `rtk proxy git diff --exit-code`; final status inspection | Tracked source/config unchanged |

Platform command/log ledger: `/tmp/natnet_phase4_platform_20260908/commands.txt` and `results.txt`. Checks do not establish linking or native runtime behavior. Initial dependency check did not prove cache freshness; explicit refresh and matching remote HEAD closed that gap.

### Runtime observations

Artifacts: `/tmp/natnet-audit-probes-20260908-6l4uyjk1/`. Probe exit 0 means assertions about observed behavior succeeded; some assertions intentionally confirm defects.

| Probe | Result |
|---|---|
| Q1/Q2 | Tier-1 component injection produces restricted command; raw Tier-2 bytes sent to isolated loopback without grant |
| Q3 | Foreign loopback SERVERINFO accepted |
| Q4/Q5 | Negative raw count/nonzero terminator/tail accepted; current payload accepted as 99.0 |
| Q6 | Old empty frame fails: `UnexpectedEof { offset: 76, needed: 8, available: 2 }` |
| Q7 | Old MODELDEF fails: `UnexpectedEof { offset: 12, needed: 97, available: 2 }` |
| Q8 | 10,000 deterministic small payload cases over seven parser entry points, no panic |
| Q9 | Documented Other(1) imagery alias canonicalizes |
| CLI controls | Bye exits 0; --max 1 consumes one heartbeat |
| CLI defects | Partial prefix/payload, invalid max and missing max all exit 0 |
| Private replay | 25 real frames, SERVERINFO, MODELDEF parse/value-round-trip |
| DataClient | Unicast bytes/source preserved; empty nonblocking recv returns WouldBlock |

Library/example compiled from existing source; probes supplied to rustc via stdin. No .rs file created or modified. Logs: `probes.log`, `cli-probes.json`, `private-replay.log`, `random-reach.json`, `dependency-refresh.txt`.

CLI truncation inputs were exactly `b"AUGI"`, and `b"AUGI" + little_endian_u16(3) + little_endian_u16(40) + little_endian_u32(1) + 40 zero bytes`. The latter declares one payload byte that never arrives. Invalid/missing max controls used two empty Heartbeat records then Bye; valid max-one stops after the first.

## Private/external evidence provenance

| Evidence | Hash / result | Limit |
|---|---|---|
| [Frame subset](/home/andrej/ws_rs/auga/docs/work/P2.1b/attempts/E1/fixtures/mcast_subset.aucap) | 6,962 B; SHA256 `58ab29c6bde195876146c98cb063ef93532fa058d1b8d162ff75239165822aba`; replay passes | 25 consecutive frames 15,732,032–15,732,056; 260 B datagram / 256 B payload |
| [Hello subset](/home/andrej/ws_rs/auga/docs/work/P2.1b/attempts/E1/fixtures/hello_subset.aucap) | 313 B; SHA256 `b815b2166d745594a0e3ab489669faa5ceb0ef2e522ce517efa1bbe1f4764051`; replay passes | One 283 B datagram; Motive 3.1.2.1, NatNet 4.1.0.0, 10 MHz |
| [MODELDEF](/home/andrej/ws_rs/auga/crates/auga_io/tests/fixtures/modeldef_reply.bin) | 1,016 B; SHA256 `fe5d289449cad576badd24f70039952f70be1b28297070c6961b9abc5a3ec468`; replay passes | 17 datasets, historical CW-500 wand id 3; no current-lab identity claim |
| [Container](/home/andrej/ws_rs/auga/crates/auga_bench/src/capture/container.rs:3) | AUGACAP v1; 12 B header, 18 B record overhead | No fixture modified or republished |
| [Historical live report](/home/andrej/ws_rs/auga/docs/work/P2.3b/live-2026-08-28.md:425) | Tier-1 canary explicitly outstanding; full 138,946-packet raw artifact absent locally | Older-SHA results do not prove current full-corpus behavior |
| [Official SDK archive](https://s3.amazonaws.com/naturalpoint/software/NatNetSDK/NatNet_SDK_4.1.1.zip) | SHA256 `af6a81ab6b5fde30bf1ee0fda0f59f62fd7cc7f6dcf6ad14eab7f82e5bb0a50a` | Inspected source, did not run vendor binaries |
| SDK PacketClient.cpp | SHA256 `50013ea1ce0789cd5c05a03d73a994362cf6b57fb6b67a8ee2d0b9afb4bbe4ab` | C++ frame-size/assets/timestamp gates |
| SDK PythonClient/NatNetClient.py | SHA256 `07c134e61b77b91cfc6791d258921f3b2f21d1d54fd3f77340d517e125f26eba` | MODELDEF version gate cross-check |

No private capture was copied here. Hashes and results make the report interpretable if temporary logs disappear.

## Roadmap to the stated 100% acceptance

No implementation or live authorization is implied by this report.

| Step | Owner / dependency | Action | Acceptance evidence |
|---|---|---|---|
| 1 | Protocol maintainer | Record supported-version/layout policy in existing decision system and public docs | Supported/unsupported/future matrix fixed; retain 4.1 behavior |
| 2 | Protocol maintainer, step 1 | Independent 3.0/4.0/4.1 fixtures; legacy structural walkers; versioned MODELDEF; correct assets gate | B01 inputs succeed; B08 policy enforced; current replay/round-trips still pass |
| 3 | Command maintainer | Validate command component grammar and canonicalize allowed names | Injection refused in typed/stringly APIs; benign values unchanged |
| 4 | Command/transport, step 3 | Safe typed request boundary or explicit raw policy; bind metadata to canary/grant | B02/B04 regressions pass; enforceable guarantee and caller duties explicit |
| 5 | Transport maintainer | Peer validation, total deadline, transaction ownership | Q3 rejected; valid peer succeeds; unrelated traffic cannot extend deadline indefinitely |
| 6 | Imagery maintainer | Clean EOF distinction, partial-error lifecycle, strict --max | B06/B07 fail correctly; normal/reverse/fragmented controls pass |
| 7 | Parser/QA | Explicit raw-count/terminator/tail policy and bounded sub-readers | Malformed sizes/residue tests; compatibility cases intentionally retained |
| 8 | QA, steps 2–7 | Persistent defect regressions; payload-aware fuzz; DataClient/dispatch/serde/nalgebra/imagery errors | Meaningful parser reach; conversion ordering verified; no shared-generator-only compatibility proof |
| 9 | Docs/CI maintainer | Feature-aware rustdoc, stale prose fixes, strict matrix, reviewed action SHA pins | Default/five feature docs pass with -D warnings; docs match actual behavior |
| 10 | Infrastructure owner | Verify runner isolation/protection settings; declare runtime platform scope | Native network tests on declared platforms; dependency graph not substituted for execution |
| 11 | Integration owner / authorized operator | Restore full capture and replay current candidate; outstanding Tier-1 canary only with fresh authorization and suitable lab window | Hash-bound current results, reread and logged canary evidence |
| 12 | Release coordinator | Re-run gates; reassess twelve criteria; correct intended public metadata; honor D54/Auga-v1 timing | R1–R12 met; no unresolved P1; explicit disposition of P2/P3/external constraints |

Allocation/caching work follows authorized profiling. Silently changing tests to match the incorrect legacy wire format would not satisfy version claims. Any deliberate support narrowing must be explicit and coordinated with consumers.

## Skipped/unavailable checks

| Check | Status | Next command / evidence |
|---|---|---|
| ARM Linux/macOS compile | Targets absent; no install | On equipped environment: `rtk proxy cargo check --locked --offline --all-targets --all-features --target aarch64-unknown-linux-gnu`; likewise `aarch64-apple-darwin` / `x86_64-apple-darwin` |
| Native Windows/macOS/musl | Not executed on Linux GNU host | On each declared native platform: `rtk proxy cargo test --locked --all-features` plus interface/multicast tests |
| 32-bit overflow | No 32-bit target installed | Equipped target: `rtk proxy cargo test --locked --all-features --target i686-unknown-linux-gnu`; add explicit overflow assertions |
| Multicast interface | No current runtime evidence | Deterministic isolated-interface test; no audit traffic to lab hardware |
| Full P2.3b capture | Absent locally | Restore artifact; verify SHA256 `9ed89bea5c2c0cf54d0c0d19cf2c5b054dd054c9a10f5453ddb43a7f85401860`; replay current candidate |
| Live canary / real bridge | Not run, read-only audit | Authorized operator, exact candidate/producer versions, suitable window, reread/log evidence |
| Numeric coverage / sustained fuzz | Not measured | Meaningful coverage/fuzz instrumentation; test counts are not branch coverage |
| Benchmarks | Skipped by instruction | Separate explicit authorization; no performance acceptance claim |
| UI/GPU/WASM/DB/deployment/generation drift | Not applicable | No extra systems required |
| Status updates | Intentionally unchanged | No status file created |

## Reproduction appendix

The probe source below was compiled from stdin against the audited source, with only net and imagery enabled. It sends only to loopback. The assertions for Q1–Q7 document current defects/policies; exit 0 means the observed behavior was reproduced.

```sh
rtk proxy rustc --edition=2024 --crate-name natnet --crate-type rlib --cfg 'feature="net"' --cfg 'feature="imagery"' src/lib.rs -o /tmp/natnet-audit-probes-20260908-6l4uyjk1/libnatnet.rlib
rtk proxy rustc --edition=2024 --crate-name audit_probes --extern natnet=/tmp/natnet-audit-probes-20260908-6l4uyjk1/libnatnet.rlib - -o /tmp/natnet-audit-probes-20260908-6l4uyjk1/audit_probes
rtk proxy /tmp/natnet-audit-probes-20260908-6l4uyjk1/audit_probes
```

Supply the following Rust as stdin to the second command. The external directory existed during the audit; create a fresh temporary directory and substitute it if reproducing elsewhere.

```rust

use natnet::{Version, FrameOfData, ModelDef, ServerInfo, ConnectionInfo, SyntheticServer};
use natnet::command::{SetProperty,PropertyValue,WriteTier,classify_property};
use natnet::net::CommandClient;
use std::net::{Ipv4Addr,SocketAddr,SocketAddrV4,UdpSocket};
use std::time::Duration;

fn v4(addr:SocketAddr)->SocketAddrV4 { match addr {SocketAddr::V4(a)=>a,_=>panic!("IPv4 only")} }
fn main() {
    // Byte construction only: no hardware request is sent.
    let step=SetProperty::plan(
        vec![",Transmission Type,1\0".into()],
        "Enabled", PropertyValue::Bool(true)).unwrap().begin();
    let effective=step.set_request[4..].split(|b|*b==0).next().unwrap();
    assert_eq!(classify_property("Enabled"),WriteTier::Tier1);
    assert_eq!(classify_property("Transmission Type"),WriteTier::Tier2);
    assert_eq!(effective,b"SetProperty,,Transmission Type,1");
    println!("Q1 CONFIRMED: Tier1 accepted node emits effective command {:?}",String::from_utf8_lossy(effective));

    let intended=UdpSocket::bind((Ipv4Addr::LOCALHOST,0)).unwrap();
    intended.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let client=CommandClient::bind(Ipv4Addr::LOCALHOST,v4(intended.local_addr().unwrap())).unwrap();
    client.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let raw_text=b"SetProperty,,Transmission Type,1\0";
    let mut raw=vec![2,0]; raw.extend_from_slice(&(raw_text.len() as u16).to_le_bytes());raw.extend_from_slice(raw_text);
    client.send(&raw).unwrap();
    let mut buf=[0u8;2048];
    let(n,_)=intended.recv_from(&mut buf).unwrap();
    assert_eq!(&buf[..n],raw);
    println!("Q2 CONFIRMED: raw Tier2 datagram transmitted through CommandClient to loopback without grant");

    let foreign=UdpSocket::bind((Ipv4Addr::LOCALHOST,0)).unwrap();
    assert_ne!(foreign.local_addr().unwrap(),intended.local_addr().unwrap());
    let info=ServerInfo {
        application_name:"FOREIGN".into(),server_version:Version::new(3,1,2,1),protocol_version:Version::V4_1,
        high_res_clock_frequency:10_000_000,
        connection:ConnectionInfo{data_port:1511,multicast:true,multicast_group:[239,255,42,99]}
    };
    foreign.send_to(&info.to_datagram(),client.local_addr().unwrap()).unwrap();
    let got=client.connect_handshake(&mut buf).unwrap();
    assert_eq!(got.application_name,"FOREIGN");
    println!("Q3 CONFIRMED: connect_handshake accepted queued SERVERINFO from foreign UDP port");

    let server=SyntheticServer::new(Version::V4_1,1_000_000);
    let payload=server.frame(1,0.0,&[]).to_payload(Version::V4_1);
    assert_eq!(payload.len(),122);
    let mut bad=payload.clone();
    bad[28..32].copy_from_slice(&(-1i32).to_le_bytes());
    assert_eq!(FrameOfData::parse_payload(&bad,Version::V4_1).unwrap().skeletons.count,-1);
    let mut bad_end=payload.clone();
    let last=bad_end.len()-4;
    bad_end[last..].copy_from_slice(&123i32.to_le_bytes());
    assert!(FrameOfData::parse_payload(&bad_end,Version::V4_1).is_ok());
    bad_end.extend_from_slice(b"trailing junk");
    assert!(FrameOfData::parse_payload(&bad_end,Version::V4_1).is_ok());
    println!("Q4 CONFIRMED: negative raw count, nonzero terminator, and trailing frame bytes accepted");

    let future=FrameOfData::parse_payload(&payload,Version::new(99,0,0,0)).unwrap();
    assert_eq!(future.frame_number,1);
    println!("Q5 CONFIRMED: current-layout packet accepted as NatNet 99.0");

    // Independently specified pre-4.1 empty layout: count fields, no section byte counts.
    let mut old=vec![0u8;78];old[0..4].copy_from_slice(&1i32.to_le_bytes());
    for v in [Version::new(3,0,0,0),Version::new(4,0,0,0)] {
        let result=FrameOfData::parse_payload(&old,v);
        assert!(result.is_err());
        println!("Q6 pre-4.1 count-only empty frame at {v}: {result:?}");
    }
    let model=[1u8,0,0,0,0,0,0,0,b'a',0,0,0,0,0];
    let result=ModelDef::parse_payload(&model);
    assert!(result.is_err());
    println!("Q7 pre-4.1 count/type/name/count marker-set MODELDEF: {result:?}");

    let mut state=0x1234_5678_9abc_def0u64;
    let mut next=||{state^=state<<13;state^=state>>7;state^=state<<17;state};
    for _ in 0..10000 {
        let n=(next()%600)as usize;
        let mut bytes=vec![0u8;n];
        for b in &mut bytes {*b=next()as u8;}
        let r=std::panic::catch_unwind(||{
            let _=FrameOfData::parse_payload(&bytes,Version::V4_1);
            let _=ModelDef::parse_payload(&bytes);
            let _=ServerInfo::parse_payload(&bytes);
            let _=natnet::command::CommandResponse::parse_payload(&bytes);
            let _=natnet::command::CommandResponse::parse_value_payload(&bytes);
            let _=natnet::imagery::record_len(&bytes);
            let _=natnet::imagery::parse_record(&bytes);
        });
        assert!(r.is_ok());
    }
    println!("Q8 PASS: 10000 deterministic payload-level malformed inputs across seven parser entry points; no panic");

    let image=natnet::imagery::ImageFrame {
        frame_number:0,camera_index:0,camera_mid_exposure:None,grab_ticks:0,
        api_timestamp_secs:0.0,width:1,height:1,
        encoding:natnet::imagery::ImageEncoding::Other(1),payload:&[0]
    };
    let record=image.to_record().unwrap();
    assert!(matches!(natnet::imagery::parse_record(&record).unwrap(),natnet::imagery::Record::ImageFrame(f) if f.encoding!=image.encoding));
    println!("Q9 PASS: documented ImageEncoding Other(1) canonicalization reproduced");
}
```

## Completion record

The four phase reports were saved in order and read before synthesis. This master incorporates later private-fixture verification and recovered Rust-LSP evidence. The four temporary phase reports were removed after final review. Source, tests, manifest, lockfile, workflow and existing status files remain unchanged. No commit, push or PR was created.
