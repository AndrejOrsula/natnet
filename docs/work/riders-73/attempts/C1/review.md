# riders-73 C1 — independent review

Reviewed 2026-09-20 in fresh read-only context. Candidate
`50cb951f5637b491e797baf1ea7546ab854a5e02`, parent
`bf1e1fda073662722a07fb57e2895da72a2a1a76`, tree
`cfeab6ed1af465276dd37b7dc2209e996cf05c49`.

Verdict: **accept config**, with no blocking finding. Only the authorized
workflow and deny graph changed. The workflow uses one canonical i686 target
value, installs that target on the stable toolchain used by the new check, and
checks the same value against the deny graph. A mismatched deny target fails
the static verifier. Existing jobs and gates are unchanged.

Independent YAML parsing, format, offline cargo-deny, cargo-machete, and
committed-tree whitespace checks passed. The builder's report records default
and all-feature tests and Clippy, net build, strict docs, six workflow feature
rows, sixteen feature-doc rows, and MSRV default/all as passed. `actionlint`
was unavailable. Local i686 compilation exited 101 with `E0463` because the
target is not installed; this is an environmental limit, not a passing gate.

The exact integrated-SHA CI i686 job must pass before the compile/config arm
can be called `accepted-build`. The change supplies no 32-bit runtime evidence;
that arm remains `blocked_external` until an equipped linker and runtime run
the tests. No hardware, Motive, benchmark, or live-network claim was made.
