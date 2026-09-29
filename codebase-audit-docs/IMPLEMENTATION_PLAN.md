# Audit remediation implementation plan

**Date:** 2026-09-08
**Baseline:** `eb8631b378808bbc542f3f61252eb77cd51f995e`
**Input:** [Master codebase audit](MASTER_CODEBASE_AUDIT.md)
**Status:** Implementation in progress under the owner's 2026-09-08 request. D54 contract amendment recorded before code changes; candidate uncommitted. Planning text below defines the implementation scope.

## Objective and boundaries

Resolve B01–B11 and give every additional architecture, coverage, infrastructure and external-evidence finding an explicit implementation or evidence work item. Preserve the dependency-free, convention-neutral core, Rust 1.92, optional features, deterministic generation, documented opaque data, and private-capture provenance. Success requires evidence for the audit's R1–R12 criteria, not merely another green run of the existing tests.

This document proposes changes; it does not enact API changes, authorize hardware writes, update consumer pins, alter runner settings, or publish a release. The audit remains a historical baseline. No source, tests, configuration, lockfile, existing status file, or sibling decision record is changed during planning.

Current source was checked against the audit: HEAD matches; MODELDEF has no version parameter; frame sections always use sizes; the assets gate is 4.0; command components are interpolated; `CommandClient::send` accepts arbitrary bytes; the handshake discards the peer; TCP uses two `read_exact` calls; the CLI treats any `UnexpectedEof` as success and invalid `--max` as unlimited. Rust-LSP confirmed the MODELDEF signature. Historical test results in the audit are baseline evidence, not test results for the future implementation.

The chosen approach is a series of bounded changes with regression tests accompanying each behavioral fix. Shared parsing and public-contract decisions precede dependent work. Independent command, imagery and documentation work can proceed after those decisions; final integration remains sequential. No speculative subsystem rewrite, async runtime, codec decoder, database, GPU work, or production dependency is needed.

## Contract decisions to record before implementation

Use the existing [D47 command policy](/home/andrej/ws_rs/auga/docs/07-decisions.md:733) and [D54 crate boundary](/home/andrej/ws_rs/auga/docs/07-decisions.md:852). Draft the amendments and consumer migration details first; apply them through Auga's existing dispatch convention before changing those contracts. Do not create a competing architecture or status system in this repository.

| Decision | Proposed approach | Constraint / unresolved evidence |
|---|---|---|
| Supported layouts | Implement the advertised legacy family with independently specified 3.0, 4.0 and 4.1 cases; enumerate other supported 3.x versions only after verifying their fields. Separate raw `Version` from a validated internal layout | Do not equate `major >= 3` with support or silently reduce the crate to 4.1. Exact minor/build/revision equivalence needs vendor evidence in W0 |
| Unknown versions | Reject unrecognized layouts on the corrected version-aware path, including 99.0; preserve raw version reporting from SERVERINFO | A comparison helper such as `Version::at_least` is not a support validator. Rejection changes existing permissive behavior and needs migration notes |
| MODELDEF compatibility | Add explicit version-aware parsing/encoding; make `parse_message` propagate its version to MODELDEF. Existing versionless entry points may remain as clearly documented 4.1 conveniences | No layout guessing from payload bytes; unknown unframed legacy dataset types cannot be safely skipped |
| Parser strictness | Bounded readers always; the new layout-aware decoder defaults to strict policy. Existing standalone parser entry points retain documented compatibility behavior during migration; explicit policy entry points expose both | Strict rejects negative counts, nonzero frame terminators and unexplained residue; compatibility retains these audited acceptances. Both retain deliberate outer-header length tolerance and known opaque extensions |
| Safe command boundary | Immutable validated command requests, consuming write authorization/state, and a transport API accepting only those requests | A public `From<Vec<u8>>`, mutable byte access, broad `AsRef<[u8]>` send argument, or deserialization into a trusted request would erase the guarantee |
| Raw transmission | Separate `RawCommandClient` from the safe client; raw bytes, unrestricted receive and socket controls belong only to the raw type. Migrate the ambiguous existing API in a coordinated breaking change | The safe type exposes no raw socket; any conversion to raw consumes the idle safe client. Deprecation is an intermediate migration aid, not closure of B04. Rust memory `unsafe` is not the right marker for this policy distinction |
| Canary execution | Immutable sans-io steps plus an exclusive transport executor that sends, re-reads and records each target sequentially | The sans-io caller can fabricate observations; a grant is an explicit assertion, not authentication. NatNet replies do not supply a general transaction ID; freshness cannot be guaranteed against stale same-peer packets |
| TCP failure lifecycle | Clean close is a distinct outcome; partial/framing failure terminates the client. Use bounded connect/record operations rather than implementing resumable partial records | Preserve the 64 MiB cap. Reconnect requires a new client. Partial failure must not leave reusable stream state |
| Encoding | Add checked NatNet encoding that refuses unrepresentable lengths, interior NUL and implicit data loss; migrate internal callers | Existing lossy/infallible behavior must not be silently changed into panics. Outer header saturation is a documented exception, handled separately from true count/length corruption |
| Scope boundaries | Continue caller ownership of clocks, tracking-gap filtering, coordinates, bridge session identity and logging persistence | Document required consumer behavior; do not pull Auga subsystems into natnet to make the library appear complete |

Public API changes are expected for command errors/requests, transaction ownership, stream EOF outcomes and some checked serialization. W0 must inventory actual callers before final signatures are frozen. Source compatibility, wire compatibility and parsed-value compatibility are separate acceptance questions.

## Work packages and dependency order

Owners below identify expertise, not assigned people. Sizes describe relative complexity, not calendar estimates. Each package includes source review, its regression tests, documentation updates and targeted verification.

| Package | Owner / size | Dependencies | Deliverable |
|---|---|---|---|
| W0 | Protocol/API maintainer; medium | None | Contract matrix, source-backed layout table, migration inventory, evidence plan |
| W1 | Parser maintainer; medium | W0 | Bounded section readers, explicit validation policy, accurate error context |
| W2 | Protocol maintainer; large | W0, W1 | Correct legacy/4.1 frames and MODELDEF, supported-layout dispatch |
| W3 | Protocol/API maintainer; medium | W1, W2 | Checked serialization and version-preserving decode/encode seam |
| W4 | Command maintainer; medium | W0 | Validated command grammar and immutable request/authorization state |
| W5 | Transport maintainer; large | W4 | Safe/raw API separation, peer/deadline ownership, canary execution |
| W6 | Imagery maintainer; medium | W0; coordinate shared errors with W1 | Terminal TCP failures, clean EOF, strict CLI |
| W7 | QA maintainer; medium | Tests land with W1–W6; final sweep after them | Deep regression, payload mutation and optional-feature coverage |
| W8 | Docs/CI maintainer; medium | W0; final API docs after W2–W6 | Accurate docs, strict feature documentation, reproducible CI inputs |
| W9 | Infrastructure/integration owners; external | Environment inventory starts at W0; candidate after W2–W8 | Platform, multicast, runner and real-producer evidence |
| W10 | Performance/consumer maintainer; small disposition | W0; implementation only if separately justified | Explicit allocation/cache disposition and consumer responsibilities |
| W11 | Release/integration coordinator; medium | W1–W10 evidence | Fresh R1–R12 closure, consumer migration, release readiness decision |

Critical path: W0, W1, W2, W3, final W7/W8 checks, W9 acceptance, W11. W4/W5 and W6 can proceed independently of legacy parser implementation, while W8 can repair current rustdoc failures early. Shared edits to `error.rs`, `lib.rs`, manifest and workflow require a single integration owner. Keep partial migrations buildable; do not merge a state in which the documented safe path still accepts raw bytes.

### W0 — Establish exact contracts and migration scope

1. Recheck worktree, revision, existing instructions, manifest, workflow and relevant D47/D54 amendments. Preserve unrelated changes. Record candidate identities as repository plus SHA.
2. Derive a field-by-field frame and MODELDEF table from the audited official SDK samples. Include section count/size prefixes, assets, precision timestamps, rigid-body/skeleton details, device/force-plate channel nesting and each known dataset's extent. Resolve sample disagreements against independent bytes or stronger vendor evidence; do not copy a sample's bug into the implementation.
3. Record supported major/minor combinations and the meaning of build/revision bytes. Test 2.x rejection, 3.0, 4.0, 4.1, each additional advertised 3.x layout, and unknown future versions. No claim about an unverified minor version becomes release documentation.
4. Use Rust-LSP references for changed public APIs and inspect known Auga and motive-bridge consumer manifests/callers. Capture current signatures, feature usage, serialization/error matches, grant/canary usage and pin locations. No claim of exhaustive external usage from local references.
5. Prepare the D47/D54 amendments and a migration table: old call, replacement call, behavior/error change, wire effect, consumer tests, integration owner. Keep unrelated sibling changes outside the implementation; necessary coordinated changes go through the established integration process.
6. Arrange access to existing private fixture replay, native targets, isolated multicast interfaces and the real bridge. Missing environments remain named external gates rather than disappearing late in the project.

**Acceptance:** one unambiguous layout/strictness/trust/API table; every breaking change has a migration path; fixture provenance and all external gates identified. W0 does not close a bug by rewriting the advertised scope.

### W1 — Bound parsing and define malformed-input behavior

**Files:** `src/bytes.rs`, `src/frame.rs`, `src/modeldef.rs`, `src/error.rs`; `tests/robustness.rs`, `tests/roundtrip.rs`.

1. Introduce the smallest shared reader operation for an exact bounded byte region. Preserve absolute payload offsets in nested errors. Validate a declared size against remaining bytes before allocating or parsing entries.
2. Parse typed sections within that region; derive count feasibility from the region, not the whole message. Require exact region consumption where the format requires it. Reuse the primitive for size-prefixed MODELDEF bodies without forcing legacy count-only records into it.
3. Add strict parsing policy with explicit handling of negative raw counts, count/size inconsistencies, nonzero frame terminators and unexplained message/body tails. Never scan arbitrary bytes to guess the next section. Keep compatibility parsing behavior explicitly tested where public compatibility requires it.
4. Preserve `DataSet::Other` bodies and rigid-body tails. Strict marker-set residue rejection must not accidentally reject the intentionally opaque rigid-body tail. Decide SERVERINFO suffix policy separately from known fixed fields; retain documented canonicalization where intentional.
5. Use `TrailingBytes` where strict policy needs it. Add precise error variants/context for unsupported legacy dataset extent, invalid signed fields, invalid terminator and encoding faults as needed. Keep the existing non-exhaustive error boundary; preserve existing variants for compatibility wrappers where useful.

**Regression cases:** negative/zero/huge counts; negative/truncated/oversized section sizes; small declared region followed by ample bytes; nested count exceeding its own region; exact region versus one-byte residue; frame terminator and suffix; MODELDEF top-level and marker-set residue; known rigid tail and unknown length-delimited dataset preserved; header declared-length mismatch still tolerated. Assert meaningful error class and offsets, not just `is_err()`.

**Acceptance:** malformed typed entries cannot consume subsequent sections; no new panic or count-driven excessive allocation relative to input; strict and compatibility behavior both specified. B10 closes through this explicit contract, not a claim that every accepted extension is corrupt.

### W2 — Repair protocol version handling

**Files:** `src/version.rs`, `src/frame.rs`, `src/modeldef.rs`, `src/lib.rs`, `src/generate.rs`; version fixtures and `tests/roundtrip.rs`.

1. Add a validated layout selection used by frame decode, MODELDEF decode and their encoders. Keep `Version` as the raw four-byte value; use a compact internal layout description rather than duplicating threshold checks throughout the crate.
2. Decode/encode section byte sizes only for 4.1+ supported layouts; gate assets at 4.1, and retain the already-correct precision timestamp threshold. Audit every other field against W0's table, not only the reproduced offsets.
3. For count-only legacy raw sections, structurally walk every nested record to discover the exact byte span, then preserve it as raw data. Implement the necessary skeleton, force-plate and device extent logic; avoid adding public fully typed models solely to find boundaries.
4. Add version-aware MODELDEF entry points and route `parse_message` through them. Legacy datasets require version-specific structural extents, including the suffixes currently kept opaque. Preserve known untyped datasets by discovered extent. Reject an unknown legacy type when there is no trustworthy length; keep 4.1 unknown size-prefixed datasets round-trippable.
5. Make generator output obey the same correct layout. Preserve deterministic poses, dropout bits and timing. Remove the unreachable pre-2.7 decoder branch once the support table proves it unnecessary; align checked encoder rejection with decoder support.
6. Keep versionless MODELDEF helpers documented as 4.1 conveniences if retained. Do not auto-detect by trying several decoders. Correct the claim that MODELDEF is version-independent.

**Regression cases:** independent literal/spec-derived empty 78-byte legacy payloads; nonempty marker sets, rigid bodies, skeletons, labeled markers, force plates and devices; 4.0 without assets versus 4.1 with nonempty assets; nonzero precision values; mixed MODELDEF dataset types; unknown framed versus unframed types; future-version refusal. Use independently constructed expected bytes, not only serializer-to-parser round trips.

**Acceptance:** B01/B08 regressions enforce the chosen policy; every advertised version has independent layout evidence; all existing 4.1 private subsets still parse and value-round-trip; opaque bytes remain intact. Do not replace the failing legacy fixtures with the crate's currently incorrect format.

### W3 — Make encoding and negotiated layout explicit

**Files:** `src/bytes.rs`, `src/message.rs`, `src/serverinfo.rs`, `src/frame.rs`, `src/modeldef.rs`, `src/version.rs`, `src/lib.rs`, `src/generate.rs`, `src/error.rs`.

1. Add checked encoding entry points, tentatively `try_to_payload` and `try_to_datagram`, with a refusal table. Reject count/size overflow, interior NUL, fixed-field string truncation and invalid raw count/length combinations before emitting a record. Test multibyte UTF-8 near fixed-field boundaries.
2. Distinguish faithful checked encoding from current documented lossy wrappers. Retain old behavior only under truthful compatibility documentation and deprecation where appropriate; no new panic, silent default, saturated internal count or partial-success output.
3. Preserve the intentional NatNet outer-header length policy as a named exception. Payload-only serialization, full datagram representation and actual UDP payload limits are distinct; validate UDP send limits at the transport boundary instead of imposing an invented parser limit.
4. Add a lightweight decoder/configuration seam holding the negotiated supported layout. Its parsed frame and MODELDEF wrappers retain source layout without changing existing plain model fields or serde representation. Their default checked re-encoding uses that layout. Extracting the plain value is explicit; re-encoding that value requires an explicit layout. A legacy MODELDEF parsed through this seam must never silently serialize as 4.1 through a versionless wrapper.
5. Explicit cross-version conversion must reject unsupported non-default assets, precision timestamps or opaque content unless the caller selects a documented lossy conversion. Raw section bytes from one layout are not automatically valid in another. Keep the core usable without a socket/session.
6. Improve encoding errors: interior NUL must not display as missing terminator, overflow must name the offered length and limit. Imagery error mapping should retain all four bad-magic bytes and the underlying cause through `io::Error` where applicable.

**Acceptance:** checked encode/decode contracts hold for valid values; unsupported conversions fail deliberately; existing qualified NaN/string round-trip caveats remain precise. Length boundary tests exercise arithmetic helpers without allocating multi-gigabyte buffers. Changes to public errors or serialization derive shapes are included in W0's migration review.

**Migration completion:** adding `try_*` is only the transition. At the coordinated API cutover, rename retained lossy/infallible serializers to explicitly `*_lossy` compatibility methods, or remove them after consumer migration; ordinary encoding names must not silently saturate internal counts or truncate strings. Preserve valid supported wire output. The intentional outer-header convention remains documented and tested separately, not used as justification for internal length corruption.

### W4 — Validate commands and preserve authorization metadata

**Files:** `src/command.rs`, `tests/command.rs`; compile-fail documentation tests.

1. Validate every node/property/text value before command formatting: reject comma, interior NUL and command-breaking controls. Define ASCII/Unicode acceptance from the actual command grammar; do not invent escaping unsupported by Motive. Preserve legitimate empty root targets only where the command permits them; reject empty camera targets.
2. Canonicalize allowlisted property names before classification and emission. Apply the same validation to typed constructors, stringly `SetProperty::plan`, `GetProperty` rereads and Tier-2 builders. Validate the complete request length, including separators and NUL, before framing.
3. Return explicit construction errors. Validate the entire target batch before exposing its first request; reject duplicate exact target identities rather than silently repeating a write. Do not silently trim node identity or coerce invalid value text. Treat unknown or ambiguous write properties as fail-closed. Do not fabricate numeric Video Mode ranges without source evidence. An enum variant for an operation unavailable over NatNet, such as asset definition, must return an unsupported-operation error rather than manufacture a plausible `SetProperty` command.
4. Introduce private-field immutable requests carrying validated bytes and request metadata for connect, MODELDEF, GetProperty, Tier-1 and Tier-2 operations. Expose bytes read-only for sans-io consumers, with explicit consumer responsibility when sent through their own sockets. No mutable access, arbitrary byte constructor, trusted-state serde derive, or generic safe-send argument.
5. Bind Tier-2 authority metadata to the consumed grant/request rather than returning a freely substitutable authority string. Make the explicit per-action scope inspectable. Empty or malformed metadata fails validation; caller identity remains outside the crate's authentication capabilities.
6. Remove cloning and public request mutation from progressing canary state. Unlock only the next target after the preceding target's reread succeeds, preserving a first-target canary. Expose structured progress/failure records so the consumer can persist the D47 log.

**Regression cases:** exact B02 exploit; comma/NUL/control injection in each component; whitespace/case property variants; valid ordinary names and empty root cases; unknown property; oversized request; all Tier-1 property constructors and Tier-2 enum cases; empty/duplicate target policy; mismatch, status failure and malformed response prevent progression. Compile-fail tests prove request construction/mutation and state reuse are unavailable, not merely that a private grant field exists.

**Acceptance:** B02 rejected before any send; unchanged benign commands produce expected bytes; authority and target metadata cannot diverge from the immutable safe request. Tests never transmit Tier-2 commands to real hardware.

Command payload length includes its terminating NUL and must fit the `u16` wire field exactly: test `u16::MAX` and one over at construction. The safe command encoder never uses outer-header saturation. The transport separately refuses requests exceeding the actual IPv4 UDP payload limit before socket I/O; fitting the NatNet header field alone does not prove sendability.

### W5 — Enforce transport ownership and bounded command exchanges

**Files:** `src/net.rs`, `src/command.rs`, `tests/net_command.rs`, command examples/docs and consumers identified in W0.

1. Introduce safe `CommandClient` operations that accept only W4 requests. Move arbitrary-byte transmission and unrestricted socket operations into `RawCommandClient`. Migrate old `send(&[u8])`, `recv(&self, …)` and shared timeout-setter callers and documentation together. Remove the old unbounded `connect_handshake` entry point from the safe client at cutover. Merely adding a typed overload alongside the falsely documented old path is insufficient.
2. Validate the expected peer on receive. Start with configured IP and port; confirm the actual Motive reply-port contract in vendor/private evidence before finalizing that rule. Do not weaken to any source merely to make a fixture pass.
3. Give handshake/command transactions exclusive socket ownership, through mutable operations or an owned transaction guard. Prevent concurrent raw receive or timeout mutation while an exchange is active. Avoid a background reader and synchronization framework for this small synchronous client.
4. Require an explicit nonzero timeout for the new transaction API; use an absolute monotonic deadline and recompute remaining read time after unrelated packets. Define malformed expected-peer response, unrelated message, undersized caller buffer, timeout, interruption and socket-error behavior. Reject invalid durations and deadline overflow. Restore previous socket options or keep ownership of options fully within the operation.
5. Prefer a fresh exclusively owned command socket for the validated handshake, then validate SERVERINFO and supported layout. Return raw server information separately when the version is unsupported, so diagnostics do not lose the negotiated value.
6. Add the sequential Tier-1 executor: write one target, interpret status, issue its reread, verify with request-specific response decoding, emit an auditable result, then advance. A mismatch, malformed response or timeout stops the sequence. Read/verification failures after sending a write have an **unknown remote outcome**; do not automatically retry or roll back writes.
7. Document correlation limits: peer filtering is not cryptographic authentication, and a matching late reply from the same peer cannot always be distinguished on this protocol. A transport-owned observation is stronger than an arbitrary caller-supplied echo, but does not create a transaction identifier. Discard/reset a failed transaction context before a new operation; log uncertainty.

**Regression cases:** foreign-port SERVERINFO followed by genuine reply; only foreign replies; unrelated-packet flood bounded by deadline; wrong ID; malformed same-peer payload; timeout without traffic; unsupported version; concurrent-reader exclusion; option cleanup; no remaining target sent after failure; delayed/reordered and stale same-peer replies and failed write outcome recorded correctly. Include compile-fail tests for raw receive/timeout mutation during a safe transaction and for reuse after consuming conversion to raw. Test `CommandResponse::parse_value` request context for the documented four-byte ambiguity. Same-peer stale-response tests must pin the documented uncertainty, not assert impossible freshness detection.

**Acceptance:** B03 rejects the reproduced foreign peer; B04's safe API guarantee is true; raw API limitations explicit; D47 sequence protected against accidental misuse within the executor. Closure does not claim protection from a malicious in-process caller, forged authority or on-path UDP attacker.

### W6 — Correct imagery stream failures and CLI validation

**Files:** `src/imagery/net.rs`, `src/imagery.rs`, `examples/imagery_dump.rs`, `tests/imagery_net.rs`; a focused CLI test harness.

1. Read the prefix with explicit progress accounting. Zero bytes followed by EOF means clean closure; EOF after any prefix/header/payload byte is truncation. Return a distinct close/record outcome, tentatively `Result<Option<…>>`, adapted to the existing caller-owned buffer API.
2. After partial read, invalid magic, impossible length or oversize declaration, mark the client terminal and prevent another record read from interpreting residual bytes as a prefix. Return the original contextual failure; subsequent calls consistently report closed/failed state. Release or clear incomplete caller-buffer contents.
3. Bound connection establishment and whole-record receipt with caller-supplied nonzero time limits in the new API; give the CLI a documented finite default and override. A slow sender must not reset the total deadline by sending occasional bytes. Keep the 64 MiB record limit, validate before allocation, and retain caller buffer reuse for successful records. Any retained unbounded compatibility method must be labelled as such and excluded from the bounded-operation guarantee.
4. Parse all CLI options before binding/connecting. Missing, negative, malformed or overflowing `--max` returns usage exit code 2. Preserve omitted `--max` as unlimited; define zero as zero records and return without establishing a connection. Preserve valid max-one behavior and mutually exclusive connect/listen handling. Validate missing values for the other options consistently.
5. Clean close/Bye/limit reached return success; partial close, malformed record and transport failure return failure. Keep codec interpretation out of the tool. Reverse mode documents first-peer trust and operator network ownership.
6. Document borrowed output lifetime, failure/reconnect lifecycle and session responsibilities. Do not conflate successful framing with Hello ordering, bridge identity or image-size semantic validation. If a consumer needs those, provide documented checks without changing forward-compatible record parsing.

**Regression cases:** clean EOF before prefix; every partial prefix boundary; partial header/payload; zero-payload records; multiple records; deliberately fragmented delivery without relying on TCP packet boundaries; stall/slow drip deadline; retry after terminal error; cap boundary and oversize before body allocation; full four-byte bad magic; outbound/reverse controls. CLI subprocess tests assert exit code, stderr and number of records for exact audit inputs, invalid/missing/overflow/zero max, help and option conflicts. Synchronize loopback peers with channels/barriers and bounded waits, not arbitrary sleeps.

**Acceptance:** B06/B07 fail correctly; no stream reuse after desynchronizing error; normal/reverse delivery and valid extension records remain compatible. Real bridge compatibility is still W9 evidence.

### W7 — Close persistent regression and coverage gaps

Regression tests belong with each preceding fix; this package completes cross-cutting coverage rather than postponing all testing until the end.

| Area | Required additional proof |
|---|---|
| Dispatch | Valid SERVERINFO, MODELDEF and frame envelopes plus unknown ID byte preservation; declared header-size mismatch policy |
| Payload robustness / B09 | Call seven audit payload entry points directly; mutate valid envelopes with IDs 1/5/7 and command ID 3; truncate valid seeds at every byte; mutate nested lengths/counts and strings |
| Reach | Deterministically ensure every intended parser is invoked with valid and malformed bodies; assert error classes and successful seeds. Random IDs alone are not a coverage claim |
| Fuzzing | Add corpus-driven targets if tooling is selected in W0; include structured seeds and bounded allocations. Record engine/version/seed/corpus/run duration and minimize crashes into ordinary regression tests. Keep fuzz-only dependencies outside the default library graph |
| DataClient | Unicast data/source/local address, nonblocking empty receive, read timeout, bind errors, selected-interface multicast join/delivery and failure cleanup |
| Serde | Inventory actual derives; test representative core structures/enums and opaque bytes with a small dev-only test serializer such as `serde_test` after MSRV/license review. Document excluded Message/header/imagery types rather than inventing new public derives |
| Nalgebra | Distinct `[x,y,z,w]` components map to `(w,x,y,z)`; positions and marker vectors preserve components. Nonunit quaternion is not silently normalized; no reverse conversion or axis transform implied |
| Imagery | Explicit NaN and `Other(1)`/`Other(2)` serializer contract cases, short ray slices, invalid UTF-8, ray dimensions/counts, unknown records and longer headers |
| Generator | Unsupported version/domain behavior, non-default fields across layouts, dropout bodies retained, reproducibility unchanged |
| Arithmetic | Exact/one-over wire limits via pure checked-length helpers; 32-bit overflow tests on an equipped target without multi-gigabyte allocation |
| Dead code | Recheck `TrailingBytes` and obsolete timestamp branches with Rust-LSP after changes. Remove only proven unreachable private code; public local nonuse is not dead-code proof |

**Acceptance:** every B01–B10 behavioral finding has a persistent regression or, for policy choices, paired strict/compatibility tests. Report numeric coverage only if actually measured; inspect uncovered critical branches rather than using a made-up percentage threshold. Sustained fuzz evidence remains separate from deterministic CI smoke tests.

### W8 — Repair documentation and strengthen CI/dependency evidence

**Files:** `README.md`, `Cargo.toml` comments/description, relevant module docs, `.github/workflows/rust.yml`, `deny.toml` if justified.

1. Fix feature-dependent rustdoc links with feature-aware prose or plain code references where targets are absent. Run strict docs for default, all features, each optional feature and `imagery,net`; add those checks to CI and remove the stale intentional-failure explanation.
2. Correct B11 and the wider documentation drift: NAT_CONNECT transmission, separate data/command clients, imagery TCP connection, retained tracking-invalid bodies, 260-byte datagram versus 256-byte fixture payload, optional serde scope and one-way nalgebra conversion. Explain checked versus compatibility encoding and supported layout/trust/EOF policies.
3. Pin all actions to reviewed immutable commit SHAs, recording upstream release labels for readability. Resolve and verify SHAs at implementation time; do not reuse unverified values from this plan. Preserve read-only permissions, no Docker-socket requirement, current cache-write restrictions and intended runner labels.
4. Retain stable as a moving compatibility gate, but record actual `rustc`/Cargo versions and invoke MSRV explicitly. Preserve Rust 1.92; run default and all-feature tests on it. Use `--locked` in acceptance/CI commands; use offline only when dependencies are already provisioned.
5. With only four independent feature flags, enumerate all 16 combinations for compile and strict docs. Retain the existing five lint/test rows and default/all-feature endpoints; add focused serde/nalgebra combined behavioral coverage. Broaden runtime tests further if the cfg inventory finds a unique combination. Do not mistake all-features success for every combination working.
6. Refresh RustSec before dependency acceptance and record database identity. Run all-feature deny and dependency-use checks. Review unused BSD allowances against the full declared target graph before removal; do not delete an allowance solely because the host graph omits it. Keep duplicate-version warnings and wildcard policy only with an explicit rationale; no automatic dependency upgrades or blanket suppression.
7. Verify intended repository metadata with the owner before publication; a private canonical repository redirect alone does not prove the eventual public URL is wrong.

**Acceptance:** B05/B11 closed, strict docs green for all configurations tested, no default runtime dependency added, MSRV verified, action provenance and advisory freshness recorded. A passing dependency scan does not establish absence of every possible vulnerability.

### W9 — Complete infrastructure and external acceptance

| Gate | Implementation/evidence procedure | Closure requirement |
|---|---|---|
| Runner trust | Infrastructure owner inspects runner group access, persistence/reuse, credential cleanup, network reach and which workflow authors can execute code | Recorded controls for actual runners; no claim from repository runner count alone |
| Repository protection | Recheck rulesets/branch protection and action policy with sufficient access; document the observed 403 if still applicable | Actual policy evidence or explicit owner disposition; do not purchase a plan or change organization settings implicitly |
| Platforms | Declare runtime-supported OS/architectures separately from dependency target filters; preserve existing compile coverage. Attempt missing ARM/macOS compile evidence on equipped environments without declaring new runtime support | Native tests on every declared runtime platform, cross-check logs labelled compilation only; new platform commitments require an explicit contract decision |
| 32-bit | Use an equipped i686 target/runtime for arithmetic paths; if emulated, label it accurately | Overflow/error assertions execute; target absence cannot be reported as pass |
| Multicast | Run selected-interface join/delivery/error tests on an isolated authorized interface on declared platforms | Real socket/interface evidence, no accidental lab traffic, resource cleanup verified |
| Small private replay | Auga-side harness replays the existing 25-frame, SERVERINFO and MODELDEF subsets against the candidate | Hash-bound inputs, current candidate SHA, parsed values and qualified round-trip comparison |
| Full private replay | Restore the audit's absent full capture in private evidence; verify SHA256 `9ed89bea5c2c0cf54d0c0d19cf2c5b054dd054c9a10f5453ddb43a7f85401860` before replay | Current-candidate packet/parse/error counts and comparison; no copy into public fixtures |
| Older versions | Independent spec fixtures plus genuine older-version captures or producer execution where available | Separate specification confidence from actual producer compatibility; no invented hardware evidence |
| Real AUGI bridge | Run exact candidate against an identified Windows producer, normal/reverse modes and safe disconnect/reconnect scenarios | Actual producer/build/interface versions and record/join/error observations; no claim from synthetic TCP alone |
| Live Tier-1 | Authorized operator chooses one reversible canary and lab window; capture pre-state, send once, reread, log result, stop on uncertainty, then verify each further target | Fresh per-run authorization and D47 evidence; no Tier-2 test, automatic write retry or unapproved rollback |

Private fixtures remain Auga evidence. Use the existing harness if it can select the candidate; otherwise implement the smallest Auga-side adapter through its integration process. The old capture's body names/IDs are historical input, not current lab configuration.

### W10 — Dispose of unmeasured performance and boundary debt

The audit demonstrated copies and linear lookup, not a measured regression. Record these as retained tradeoffs pending explicit profiling authorization: owned NatNet models, temporary serializer section buffers, datagram copy and linear name resolution. They are not silently dropped findings and are not presented as optimized.

If profiling is later authorized, first measure an agreed representative corpus, packet rate, allocation count and latency target. Prefer caller-buffer/write-into support and capacity reuse over new model layers. Preserve failure atomicity: a failed checked encode must not leave a seemingly valid partial record in the caller's output. Consumer-side name caches must invalidate on MODELDEF generation changes and preserve current duplicate-name semantics. Do not add a hidden mutable global cache.

Document memory cap versus total deadline, bridge identity/session checks, backpressure and reconnection ownership. Keep tracking-gap filtering, clocks and coordinate transforms in Auga. Lack of SQL/XSS/CSRF/GPU/DB machinery is not a missing implementation item.

**Acceptance:** each tradeoff has an explicit rationale, owner and trigger for reopening. If the owner requires measured optimization for release, W10 becomes an external authorization/evidence dependency; it cannot be called complete without that evidence.

## Complete finding traceability

| Audit item | Work packages | Required closure evidence |
|---|---|---|
| B01 legacy framing/assets/MODELDEF | W0–W3 | Independent legacy/nonempty fixtures and current 4.1 private replay |
| B02 component injection | W4, W5 | Exact exploit refused at every construction path; no datagram sent |
| B03 handshake peer | W5 | Foreign-peer rejection, real-peer control and bounded unrelated-traffic tests |
| B04 raw-send guarantee | W0, W4, W5, W8 | Migrated API, compile-fail tests, explicit raw contract and correct docs |
| B05 strict rustdoc | W8 | Default, individual and combined-feature strict doc logs |
| B06 partial EOF | W6 | Prefix/header/body truncation failure, clean EOF success and terminal lifecycle |
| B07 max validation | W6 | Missing/invalid/overflow errors before sockets; zero/one/omitted controls |
| B08 future versions | W0, W2, W3 | Explicit layout policy in decode, encode and negotiation |
| B09 ineffective randomness | W7 | Payload-entry reach and structured mutation evidence |
| B10 validation debt | W1, W2, W7 | Bounded readers, strict/compat contract and malformed-region matrix |
| B11 stale prose | W8 | Source-matched docs and fixture-length assertion |
| Negotiated layout / lossy encoding | W0, W3 | Source-layout retention and explicit conversion/refusal tests |
| Mutable canary/grant / missing per-target reread | W4, W5, W9 | Immutable state, exclusive executor, stopped progression and authorized live evidence |
| Partial TCP/resource lifecycle / lossy errors | W1, W3, W6 | Context-preserving failures, total deadlines, no desynchronized reuse |
| Allocation/copy / linear lookup | W10 | Explicit retained tradeoff or separately authorized measured change |
| Default/serde/nalgebra feature truth | W7, W8 | Default graph and behavior/derive scope tests |
| Dispatch/DataClient/imagery/generator gaps | W2, W6, W7, W9 | Persistent tests, independent bytes and actual platform evidence |
| Dead code/dependencies/secrets | W7, W8, W11 | Semantic recheck, refreshed deny/machete, scoped secret-pattern scan; no unsupported exhaustive claim |
| Mutable actions/toolchain / missing feature gates | W8 | SHA provenance, explicit MSRV and full compile/doc feature matrix |
| Runner isolation/protection / metadata | W9, W11 | Owner evidence, accurate unknowns and approved intended metadata |
| Missing native/32-bit/multicast/bridge/full capture/live evidence | W9 | Candidate-bound results or explicitly unresolved external gates |
| D54 release timing/private fixtures | W0, W9, W11 | Private evidence retained; publication separately gated by Auga v1 |

## Verification commands and evidence format

These commands are planned, not newly executed results. Start with the affected test target, then run the applicable feature/MSRV gates. Set `CARGO_TARGET_DIR` to a task-specific build directory if preserving a clean source checkout is useful. Do not run benchmarks. New dependencies require an intentional lockfile update before `--locked` verification.

```sh
rtk proxy cargo fmt --all --check
rtk proxy cargo test --locked --test roundtrip
rtk proxy cargo test --locked --test robustness
rtk proxy cargo test --locked --test command
rtk proxy cargo test --locked --features net --test net_command
rtk proxy cargo test --locked --features imagery --test imagery
rtk proxy cargo test --locked --features imagery,net --test imagery_net
rtk proxy cargo clippy --locked --all-targets --all-features -- -D warnings
rtk proxy cargo test --locked
rtk proxy cargo test --locked --all-features
rtk proxy cargo build --locked --features net
rtk proxy cargo build --locked --features imagery,net --example imagery_dump
rtk proxy cargo +1.92 test --locked
rtk proxy cargo +1.92 test --locked --all-features
rtk proxy env RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
rtk proxy env RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features
rtk proxy cargo deny fetch db
rtk proxy cargo-deny --locked --all-features --manifest-path Cargo.toml check advisories bans licenses sources
rtk proxy cargo machete
rtk proxy cargo tree --locked --no-default-features --edges normal
rtk proxy git diff --check
```

Run test and strict Clippy forms for `imagery`, `net`, `serde`, `nalgebra`, and `imagery,net`. Use the following templates for each nonempty subset of the four features; default is the same command with no `--features` argument. W8 should encode the 16-row matrix in CI or its existing reusable command convention, not rely on manual memory.

```sh
rtk proxy cargo check --locked --all-targets --no-default-features --features serde,nalgebra
rtk proxy env RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --no-default-features --features serde,nalgebra
rtk proxy cargo test --locked --features serde,nalgebra
rtk proxy cargo clippy --locked --all-targets --features imagery,net -- -D warnings
```

On equipped environments, compile each of the seven `deny.toml` triples using the first template, then run tests natively for declared runtime support. The 32-bit command requires the corresponding runtime/linker or explicitly identified emulation. A cross-check never substitutes for native network evidence.

```sh
rtk proxy cargo check --locked --all-targets --all-features --target x86_64-pc-windows-msvc
rtk proxy cargo test --locked --all-features
rtk proxy cargo test --locked --all-features --target i686-unknown-linux-gnu
```

The seven existing graph targets are `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `x86_64-apple-darwin`, `x86_64-pc-windows-gnu`, and `x86_64-pc-windows-msvc`. Exact new CLI, private-replay and fuzz commands must be recorded when those harnesses are selected; this plan does not invent an existing script or a producer command.

For each decisive check record: package/finding, candidate SHA, command, toolchain/features/target, exit code, decisive output/counts, log/artifact location, input hash where relevant, and one of passed/failed/skipped/unavailable/not applicable. Review each complete diff for contracts, error paths, cleanup, concurrency, allocation and documentation truth; fix material defects and rerun affected checks. Record baseline failures separately from new regressions.

## W11 — Final acceptance and handoff

1. Rerun all applicable gates against the integrated candidate, not intermediate revisions. Check default dependency isolation, format, strict lint, tests/doctests, docs, feature matrix, MSRV, dependencies and platform evidence. Recheck the diff for accidental private fixtures, secrets or unrelated changes.
2. Reassess every B01–B11 row and every unnumbered debt/gap row above. A deprecated vulnerable/ambiguous path is not closed merely because an alternative exists. Explicit compatibility behavior is documented and tested; retained performance tradeoffs remain labelled as such.
3. Recompute R1–R12 using the audit's original definitions: R1 tests/MSRV; R2 format/lint; R3 features; R4 dependencies; R5 current real 4.1 replay; R6 AUGI framing/join/stream; R7 advertised versions; R8 command boundary and live acceptance; R9 handshake; R10 stream/CLI failures; R11 docs; R12 critical regressions. Do not carry forward the old six passes without fresh candidate evidence.
4. Keep unavailable hardware, missing capture, protection visibility or target execution explicitly open. Do not claim 100% while a required R criterion lacks evidence. Ancillary findings can receive a reasoned owner disposition; that cannot substitute for a required test or the missing live canary.
5. Prepare integration for **both Auga and motive_bridge**, with exact natnet candidate identity, build/test/doc results and migration notes through their existing processes. The bridge uses a sibling path dependency with a pairing ledger and CI `NATNET_REV`; update those together during authorized integration. Its imagery tests match exact `Error` variants and display text, so W3 error improvements require coordinated test/error-adapter migration. Verify the bridge's existing host tests, strict lint/docs, Windows compile/lint and MSRV gates; retain real Windows producer execution as separate W9 evidence. Auga needs its candidate pin, affected consumer tests and private replay. No source reset, shared-history rewrite, commit, push or PR is implied by this plan. During future implementation update existing handoff/status records only within the authorized integration scope.
6. Publication remains separately governed by D54 and Auga v1. Completing the technical audit checklist does not authorize a public repository change, crates.io publication or live Tier-2 operation.

## Planning verification

The plan was checked against current source and every numbered and unnumbered audit section. No implementation tests were run during planning; they would not validate a document or future code. Document checks passed for B01–B11/W0–W11/R1–R12 traceability, local links, balanced Markdown fences and preservation of tracked files. Dependency ordering and closure criteria were reviewed manually.

Fable 5.1 reported its included-usage limit; the permitted Opus preflight refused, so no Claude critique was obtained and no reset was awaited. One independent Luna review completed. Accepted refinements: explicit serializer cutover, layout-carrying decoded wrappers, separate safe/raw clients including receive controls, strict-default versus compatibility parser mapping, exact command-length boundary, stale same-peer tests, and both consumer integration gates. The bridge dependency/error-match claims were verified in its manifest, workflow and source. Independent legacy fixtures were already mandatory and remain so. The review's platform caution is incorporated: graph targets do not automatically become native-runtime commitments.

## Implementation evidence ledger

The verified source was committed as `969af81`, followed by audit whitespace
cleanup in `b5a1259c144e26ffff6a22ce60595be475376df0`, after explicit owner
authorisation. Baseline: `eb8631b378808bbc542f3f61252eb77cd51f995e`.
Publication and native producer acceptance are not claimed. Consumer pin
integration follows this committed source; see the existing Auga STATUS record. Final source verification is recorded below. Logs live under
`/tmp/natnet-remediation-core/` and the companion evidence directories.

- Official SDK 4.1.1 source review resolved layouts to 3.0, 3.1, 4.0 and 4.1.
  Build/revision are retained; unknown major/minor pairs are rejected.
- Intermediate default tests: passed, exit 0, `default-integration-02.log`.
  Previous bounded-reader assertion failures were corrected to assert the
  earlier rejection inside the declared section, before an overread.
- Intermediate all-feature compile: passed, exit 0, `check-integration-01.log`.
- Strict lint/docs: failed at intermediate revisions; owned public API error
  documentation and command links are being corrected. Not final gate evidence.
- `cargo deny fetch db`: passed, exit 0, `deny-fetch.log`.
- `cargo-deny --locked --all-features --manifest-path Cargo.toml check advisories bans licenses sources`:
  passed, exit 0, `deny-check.log`; only unused BSD license allowances warned.
- `cargo machete`: passed, exit 0, `machete.log`, no unused dependencies found.
- `cargo tree --locked --no-default-features --edges normal`: passed, exit 0,
  only the root package. `serde_test` is a development dependency.

### W10 disposition

Owned models, temporary section buffers, the datagram copy, and linear name
resolution are retained. They keep ownership and failure atomicity explicit;
no measured regression currently justifies a new abstraction or cache. The
NatNet maintainer owns serializer/model tradeoffs; consumers own name caching.
Reopen only after authorized profiling establishes a representative corpus,
rate and allocation/latency criterion. No benchmark was run or optimization
claimed. Any consumer cache must invalidate on MODELDEF generation changes
and preserve duplicate-name behavior.

The imagery client owns the record memory cap and transport deadline; consumers
own session/identity validation, backpressure, and deliberate reconnection.
A record failure closes the current connection. Tracking-gap filtering, clock
estimation and coordinate transforms remain Auga responsibilities.

### Consumer/API migration map

| Previous surface | Replacement | Behavior and verification |
|---|---|---|
| `FrameOfData::to_payload/to_datagram(version)` and `SyntheticServer::datagram` returning bytes | Handle `Result<Vec<u8>>` | Correct selected layout; invalid values fail without emitting a partial datagram |
| Versionless MODELDEF decode followed by implicit 4.1 encode | `Decoder::new(version)?.parse(bytes)?`, then `DecodedMessage::to_datagram()` | Retains source layout; explicit destination conversion refuses opaque layout-dependent data |
| Historical permissive standalone parsing | Standalone compatibility wrappers, or explicit `ParsePolicy` | Strict decoder rejects invalid counts/terminators/residue; outer size remains telemetry |
| Infallible `ServerInfo` encoding | Handle checked encoder result | Interior NUL and names exceeding 255 UTF-8 bytes are refused |
| Raw command byte builders | Handle validated `Request` results; immutable `as_bytes()` only for explicit raw/sans-io consumers | Complete request length and component grammar validated before exposure |
| Mutable grant/canary fields and remaining-write batch | `Tier2Grant::sign_off` and consuming Tier-1 steps | Authority/action bound; each next target requires a verified previous step |
| `CommandClient` arbitrary send/receive/socket controls | Typed bounded exchange, or separate `RawCommandClient` | Safe client owns peer-filtered transactions; Tier-1 writes use the executor |
| Unbounded handshake | `connect_handshake(timeout)` | Nonzero deadline and supported negotiated layout required; unsupported SERVERINFO retained in error |
| Imagery `connect/from_stream/recv_record` without total deadline | Explicit positive durations; `recv_record` returns `Ok(true)` record / `Ok(false)` clean EOF | Partial/framing/I/O failures terminate reuse; clean EOF differs from truncation |
| Imagery encoding faults reported as decode errors | `InteriorNul`, `LengthOverflow`, full `InvalidMagic` | Bridge exact error assertions require migration |

Versionless MODELDEF wrappers remain explicitly 4.1-compatible conveniences.
A valid mandatory rigid-body marker prefix may carry an opaque suffix in 4.1;
legacy tails must end exactly because their envelope has no body size.

### Private and consumer evidence

The private replay harness is `/tmp/natnet-remediation-replay`; no private
capture was copied into this repository. Strict 4.1 replay passed for one
SERVERINFO, 25 consecutive frames and one 17-dataset MODELDEF. Checked frame
and MODELDEF roundtrips preserve bytes and float bits. SERVERINFO preserves
semantics while canonicalizing 185 stale application-name padding bytes.
Fixture hashes remain those recorded in W0. Full 138,946-packet capture and
live producer/canary acceptance remain unavailable, independent gates.

The concurrently edited motive_bridge worktree was not modified. Its isolated
compatibility snapshot is `/tmp/natnet-remediation-bridge/snapshot`; details
and the pending commit/pairing procedure are in `COMPATIBILITY_PATCH.md` and
`PAIRING_MIGRATION_PROPOSAL.md` under that temporary evidence directory.
Host check, 99 library tests, strict docs, Windows target check and MSRV host
check passed for the snapshot. Full tests exposed an existing mock heartbeat
ordering failure; host/Windows lint exposed existing bridge lints. These
failures are not waived, and isolated success is not a completed sibling pin
migration. Source manifests are recorded in the final ledger below.


### Final candidate verification — 2026-09-08

This is a source implementation handoff, not full external acceptance. The
40-file source manifest (excluding this audit directory) is
`/tmp/natnet-remediation-core/final-source-manifest.sha256`, SHA-256
`0206dd580ed8e512f28b564baef759adff964eb0eb1f6a2fdb736c48f5362393`.
It identifies the source in `b5a1259c144e26ffff6a22ce60595be475376df0`;
the baseline SHA is not the candidate commit. Later audit-only commits do not
change this source manifest. Temporary evidence must be retained before `/tmp` cleanup.

All commands below were run through `rtk proxy` (Git checks through `rtk git`).
Core stable target directory was `/tmp/natnet-remediation-core/target`;
MSRV target directory was `/tmp/natnet-remediation-msrv`.

| Check | Result | Evidence |
|---|---|---|
| `cargo test --locked` | Passed, exit 0; 99 tests including doctests | `final-default.log` |
| `cargo test --locked --all-features` | Passed, exit 0; 175 tests including doctests | `final-all.log` |
| `cargo +1.92 test --locked` | Passed, exit 0; 99 tests | `final-msrv-default.log` |
| `cargo +1.92 test --locked --all-features` | Passed, exit 0; 175 tests | `final-msrv-all.log` |
| `cargo fmt --all --check`; `git diff --check` | Passed, exit 0 each | Final handoff tool output |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | Passed, exit 0 | `clippy-integration-05.log`; matrix final all-feature row |
| `cargo build --locked --features net` | Passed, exit 0 | `final-net-build.log` |
| `cargo build --locked --release --features imagery,net --example imagery_dump`; `cargo test --locked --release --features imagery,net --test imagery_cli` | Passed, exit 0 each; 6 CLI tests | `final-cli-release-build.log`, `final-cli-release-test.log` |
| All 16 feature subsets: `cargo check --locked --all-targets --no-default-features [--features SET]` and `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --no-default-features [--features SET]` | Passed, 16/16 each, exit 0 | `/tmp/natnet-remediation-matrix/results-final-feature.tsv`, `results-docs-final.tsv`, corresponding logs |
| Seven focused feature rows: strict all-target Clippy and tests | Passed, 7/7 each, exit 0 | Matrix `results-final-focused.tsv`; imagery, net, serde, nalgebra, imagery+net, serde+nalgebra, all |
| Seven deny targets plus i686: `cargo check --locked --all-targets --all-features --target TRIPLE` | Passed, 8/8, exit 0; compilation only | `/tmp/natnet-remediation-platform/final-check-summary.log` and `final-check-*.log` |
| Private 4.1 replay and all-prefix sweeps | Passed, exit 0; 27 packets, no panic | `/tmp/natnet-remediation-replay/replay-driver-command.txt`, `replay-command.txt`, `replay.log`, `fixture-manifest.sha256`, `production-manifest.sha256` |
| Linux loopback multicast join and delivery | Passed, exit 0; actual socket delivery, no skip | `multicast-runtime-01.log` |
| Dependency advisory/license/source checks, unused dependencies, empty default runtime dependency tree | Passed, exit 0 each | `deny-fetch.log`, `deny-check.log`, `machete.log`; dependency ledger above |

The final Request documentation change split mutation and forging into separate
compile-fail cases. All four stable/MSRV default/all-feature test runs and all
16 strict docs were rerun afterward. Earlier platform/replay evidence covers
identical runtime code; the only other platform-window change was import
formatting. No benchmark, GPU claim, generated-source gate or visual artifact
was required or produced. Scoped secret-pattern checks and complete diff review
found no private fixture or secret addition; this is not a universal secret audit.

### Finding and readiness reassessment

| Findings | Disposition and current proof |
|---|---|
| B01, B08 | Fixed: independent nonempty 3.0/3.1/4.0/4.1 fixtures, layout-carrying decoder, unsupported-version refusal including lossy encoding; private 4.1 replay |
| B02 | Fixed: validated immutable requests, exact exploit and component/length boundary regressions |
| B03 | Fixed: connected peer filtering and bounded handshake tests; UDP freshness/authentication limitations remain explicit |
| B04 | Fixed at API boundary: separate safe/raw clients, consuming state, forbidden socket/mutation/reuse compile-fail proofs, sequential executor; live acceptance remains open |
| B05, B11 | Fixed: current contract documentation and all 16 strict rustdoc configurations |
| B06, B07 | Fixed: terminal partial EOF/error handling, total deadline, exact retry context, record cap, CLI validation and release-profile tests |
| B09, B10 | Fixed: direct payload mutation/truncation, bounded readers, strict/compatible paired tests, checked encoders; sustained fuzzing is not claimed |

W0–W8 source work and W10 documented dispositions are complete. W9 remains
partially verified. W11 has completed candidate verification and prepared
consumer migrations; actual pins and external acceptance remain open.

R1, R2, R3, R4, R5, R6, R7, R9, R10, R11 and R12 meet their audited criteria
with the candidate evidence above. R8 remains unmet because the required live
Tier-1 acceptance is absent, despite the repaired command boundary. Result:
**11/12 readiness criteria met; not 100% accepted.** Full-capture replay, genuine
older producers, real Windows AUGI producer, native Windows/macOS/musl runtime,
i686 runtime, runner isolation and repository protection evidence remain
separately open. i686 runtime lacks `Scrt1.o`, `crti.o` and `-lgcc`; cross-check
success does not replace `cargo test --locked --all-features --target
i686-unknown-linux-gnu` on an equipped host. Protection visibility previously
returned 403; no settings were changed. Live work requires an authorized
operator and per-run reversible canary; historical fixtures are not lab targets.

### Final consumer handoff

Auga migration is prepared in `/tmp/auga-natnet-check.lAXpdk`, with source patch
`/tmp/auga-natnet-migration-source.patch` (SHA-256
`d922f1756bd19a8d5eec4f22ebb66e3072398027f25f9fe2e14e8c4fa774c9e1`).
Snapshot archive `/tmp/auga-natnet-migration-snapshot.tar.gz` has SHA-256
`9d9620acb0d7ed4f6b6066d45f9ed31d886640fc5b1efb1f2f070937da7d6159`.
Exact commands and results are in
`/tmp/auga-natnet-migration-verification-ledger.txt`: 129 tests, auga_io and
params_scan strict Clippy, scanner self-test and locked offline check passed.
Strict no-feature auga_io docs failed with existing unresolved `ros`,
`ros::cdr`, `ros::transport` links; the same docs with `--features ros` passed.
This existing documentation failure is not waived. The real Auga checkout
retains its old source/pin; only its existing D54 and STATUS records were
updated. This avoids landing new API calls against an old dependency revision.

Bridge patch `/tmp/natnet-remediation-bridge/compatibility.patch` was dry-run
and apply/hash verified. Exact gates, known failures and snapshot manifests
are in `/tmp/natnet-remediation-bridge/VERIFICATION_LEDGER.md` and
`final-snapshot-hashes.sha256`. Full tests fail on an existing mock heartbeat
ordering case; strict host/Windows Clippy fails on 12 existing lints, and
formatting has existing drift. Those are consumer gates, not waived passes.
The real concurrently edited bridge checkout was preserved.

The owner subsequently authorised commits. Source and audit whitespace commits
are recorded above; all 175 all-feature tests passed again at `b5a1259` with
exit 0. Consumer integration applies the prepared changes, updates Auga's
actual pin and the bridge pairing ledger/CI `NATNET_REV` consistently, and
reruns consumer gates against the committed source. The consumer snapshots
and pending-pin descriptions above record the pre-commit handoff, not a
claim about subsequent integration. Current consumer state is recorded in
Auga's existing STATUS and each consumer verification ledger.
No push, PR, hardware write or publication was performed. D54's Auga
v1 publication prerequisite remains independent.


### Non-lab completion scope — 2026-09-08

The owner authorised remaining work and explicitly assigned lab execution to
a different future agent because this agent has no lab access. This pass
finishes accessible local/CI gates and preserves the W9 lab procedure; it does
not report the deferred hardware criteria as passed.

Committed approach: repair the failed published CI workflow's missing explicit
`toolchain: stable` input in the feature-docs matrix; verify the action contract
and rerun the exact committed workflow. Independently repair Auga's default
rustdoc links to its feature-gated ROS module, and attempt actual i686 runtime
verification with isolated local tooling. Preserve source contracts, consumer
pins and concurrent work. No benchmark or lab traffic is needed. Acceptance
for this pass is fresh CI evidence, default/ROS strict docs, an honest i686
runtime result, and an updated handoff distinguishing completed work from
future lab/native evidence and unavailable repository policy controls.

CI failure reproduced from GitHub run `34212963327` at NatNet `a61ea38`:
feature-docs setup exits 1 with `'toolchain' is a required input`. Local compile
and rustdoc tests did not exercise that action setup, so their earlier success
did not establish workflow success.

Workflow lint additionally reproduced SC2086 on the cargo-machete executable
path: YAML quotes did not preserve shell quotes. A literal run block now
quotes the executable path for the shell. Both repairs preserve the existing
job matrix and tool versions.

Actual i686 execution exposed two 64-bit-only test expectations. Both oversized
records are rejected before allocation on both targets: 32-bit record-length
arithmetic returns `ImplausibleCount`, while 64-bit framing can represent the
length and reaches the transport cap or truncated-input check. Tests now
assert the exact intended error path for each pointer width, and parse_record
documentation names record-length overflow. Runtime behavior is unchanged.

After the assertion repairs, actual i686 all-feature/all-target execution passed
165 tests, and a separate doctest invocation passed 10 tests (both exit 0, no
skips). The ELF32 imagery_dump example built and ran with `--max 0` (exit 0).
Logs and isolated sysroot details are in `/tmp/natnet-remediation-platform/`
(`i686-final-*`). Host targeted regression tests passed 22 tests; strict
targeted Clippy, all-feature rustdoc, formatting and diff whitespace checks
passed. Independent review found no material defects in the assertion or
documentation changes. The earlier musl run passed 175 tests at `58782649`;
that run predates these test/rustdoc-only changes and remains identified by
its original source manifest. Current CI and consumer pin evidence is recorded
in Auga's existing `docs/STATUS.md` after publication.
