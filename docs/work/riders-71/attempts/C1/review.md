# riders-71 C1 — independent review

Reviewed 2026-09-20 in fresh read-only context. Candidate
`ea0bba5fc3b1cadea5ec567281a293051681fa7e`, parent
`120330241ff05d55d7c47bf004c1d9678fc0d0cc`, tree
`1b4db6495fdbf19b1d8b51f8a16ac9a5cd0b9641`.

Verdict: **accept**. Zero blocking or must-fix findings in the candidate tree.
The diff contains only the five paths authorized by the brief.

The reviewer confirmed that the MODELDEF tests cover legacy and 4.1 layouts,
distinguish `ImplausibleCount` from later `UnexpectedEof`, and reach the guard
before allocation. Version tests pin major/minor ordering and the intended
build/revision treatment. The 4.1 labeled-marker test pins the 26-byte minimum
with a 25-byte remainder. The UDP control covers the exact and over-limit
datagram boundaries and confirms the rejected request is not sent. The two
unmatched BSD allowances are removed; cargo-deny passes without warnings.

Independent focused suites passed with 13, 8, 12, and 12 tests. Format,
Clippy, default and all-feature tests, net build, imagery example, Rust 1.92
default/all-feature tests, all six feature rows, all sixteen strict-doc rows,
cargo-deny, and committed-tree whitespace passed. No repository-tracked file
was changed by review.

The draft packet initially said “102 runtime plus 5 doctests” for the default
run. The exact output enumerates **97 runtime plus 5 doctests**. Coordinator
corrected that report-only transcription in `report.md` and `gates.md` before
freezing this packet; the candidate identity is unchanged. All-feature
`177 runtime plus 10 doctests` was correct.

The mutation/ablation and restored-license controls are builder evidence and
were not replayed by the read-only reviewer. No Windows, Motive, or hardware
evidence exists or is claimed.
