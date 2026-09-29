# riders-76 C1 — independent review

Reviewed 2026-09-20 in fresh read-only context. Candidate
`cb1495ba83458382264f1de05bffb34605fb7d57`, parent
`bf1e1fda073662722a07fb57e2895da72a2a1a76`, tree
`27e0ca06991cea4b29c8811b48957b9542658ef2`.

Verdict: **accept**, with no candidate must-fix. Only the authorized
`tests/imagery_net.rs` changed. The repaired control waits for the peer's
complete `write_all`, peeks with a whole-record buffer, requires the queued
prefix to be shorter than the record, then checks timeout, stale-buffer
clearing, and terminal retry. Complete-record, incomplete-body, byte-drip,
and blocking-mode restoration controls remain. Production source, deadline
tolerance, workflow, and manifest are untouched.

An independent focused `imagery,net` loopback test passed 23/23. The builder's
packet records default and all-feature tests, Clippy, format, builds, strict
docs, feature rows, MSRV, dependency checks, and a 100-run focused repeat as
passed. The one-read source ablation is builder-reported rather than replayed
by the reviewer. The builder supplied the retained exact 100-run command
after review; it is now in `gates.md`. The reviewer did not replay that repeat.

The queue-bound property remains dependent on host socket capacity; the test
fails with a diagnostic if the whole record queues. Evidence is Linux
loopback only. No Windows, Motive, hardware, benchmark, or live-network claim
was made.
