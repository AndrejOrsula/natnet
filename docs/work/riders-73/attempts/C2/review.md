# riders-73 C2 — independent review

Reviewed 2026-09-20 in fresh read-only context. Candidate
`c05b2b635d59e519752ae5c4a12478e97e6f9ce1`, parent
`46869eefb0609ac10af63f8488d838da7019bd12`. Verdict: **accept repair**;
no further must-fix.

The reviewer confirmed CI run `35536927717` installed the i686 target but
failed the verifier's negative control, leaving the actual cross-check
skipped. Bash suppresses `errexit` for a function invoked in `if`; the old
bare deny assertion failed but later greps succeeded. Each C2 assertion now
returns failure explicitly. A read-only harness reported `matching-pass` and
`mismatch-rejected`, exit 0. YAML parsing and diff whitespace passed.

The exact post-repair CI i686 job must pass before `accepted-build` can be
claimed. The local i686 target is absent, `actionlint` unavailable, and no
32-bit runtime, Motive, hardware, or benchmark proof exists.
