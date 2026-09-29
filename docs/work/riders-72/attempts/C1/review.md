# riders-72 C1 — independent review

Reviewed 2026-09-20 in fresh read-only context. Candidate
`adbd2bcf157fb567e35d7b242fb94ec4529a3616`, parent
`120330241ff05d55d7c47bf004c1d9678fc0d0cc`, tree
`74e3f9b730adfdcdfe58d6bb5d0f448f437b5cf5`.

Verdict: **accept**, with no source must-fix. The diff is 29 additions to the
single authorized path, `src/lib.rs`. Fixed NatNet 4.1 inputs generate one
tracked body, strict decoding checks the frame and body identities, and
checked re-encoding must reproduce the bytes. All referenced APIs and the
`Message::FrameOfData` variant are unconditional public exports.

Independent default doctests passed: one positive plus five compile-fail
cases. All-feature doctests passed: one positive plus ten compile-fail cases,
eleven total. The ten existing compile-fail fences are unchanged from the
base. Format, default/all tests and Clippy, net build, strict docs, imagery
example, sixteen feature rows, MSRV retry, cargo-deny, cargo-machete, and
whitespace checks passed. The builder's temporary body-id `7` to `8` mutation
made the positive doctest fail and is absent from the candidate.

The brief's unqualified count of ten pre-change negative doctests applies to
the all-feature population; default features expose five. The packet records
both populations explicitly. The first MSRV all-feature run hit the known
timing-sensitive imagery deadline test; an immediate exact-candidate retry
passed. This remains riders-76 risk and is not evidence against the
`src/lib.rs` change. No Windows, hardware, network, or benchmark evidence was
run or claimed.
