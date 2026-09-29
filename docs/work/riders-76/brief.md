# riders-76 C1 — replace an unprovable late-peer socket assertion

Repository: `natnet`. Base `dev` `bf1e1fda073662722a07fb57e2895da72a2a1a76`. Candidate branch `slice/riders-76`; worktree `/home/andrej/ws_rs/natnet_wt/impl-riders-76`. Evidence branch `evidence/riders-76`. Access: remote loopback only. Read NatNet README/workflow, Auga STATUS/PLAN, and Auga `AGENTS.md` §§4–7.

## Finding and contract

`tests/imagery_net.rs::a_peer_still_sending_past_the_deadline_is_a_timeout` is flaky. Its peek buffer is capped at the socket absorption amount, so a full peek does not prove the whole receive queue is stable. The reader consumes the 12-byte prefix before its body read, freeing space while the writer remains active. The implementation deliberately performs one nonblocking read after deadline and cannot infer per-byte arrival timestamps. Thus the test's concurrent-writer premise is not a black-box observable contract. Historical rate was about 5/800 runs; do not widen a timeout or tolerance to hide it.

Own only `tests/imagery_net.rs`. Do not change `src/imagery/net.rs`, public deadline semantics, other tests/modules, workflow, manifests, README, or frozen packets. Active riders-73 owns workflow/deny; no overlap. Riders-78 separately investigates host-speed direction; do not substitute a larger undirected campaign for a deterministic control.

Replace the impossible assertion with a deterministic queue-bound property: complete peer `write_all` before calling `recv_record`; probe with a buffer at least the whole record size so `queued < total` is an informative observation; then require timeout and terminal client state for the incomplete record. Reuse or strengthen the existing queue-bound control near current lines 1272–1364. If the existing test is genuinely sufficient, remove only the redundant flaky assertion and strengthen the existing control's synchronization so the contract remains explicit. Prefer channel/barrier synchronization and bounded waits over an arbitrary 20 ms polling sleep. Keep fully-buffered-after-expiry delivery, incomplete-body timeout, drip deadline, and blocking-mode restoration controls intact.

Test first by reproducing or proving the flawed premise from current code, then show the deterministic assertion fails under a safe ablation of deadline/terminal handling and passes restored. Add a negative control ensuring a complete prebuffered record still delivers. No ignored tests, deleted property without replacement, timeout widening, benchmark, hardware, or production code edit.

## Gates and handoff

Use an isolated `CARGO_TARGET_DIR`. Run focused `imagery,net` integration tests, default/all-feature tests, fmt, all-feature Clippy `-D warnings`, net build, strict docs, relevant feature rows, MSRV default/all, cargo-deny and cargo-machete where available, and committed-tree whitespace. Repeat the focused test enough to demonstrate stability, but publish exact population/load and no timing-performance claim. No Windows/Motive evidence.

Commit only the owned test file as immutable `candidate(C1)` with SHA/tree. Draft `docs/work/riders-76/attempts/C1/report.md` and `gates.md` uncommitted. Never push. Fresh read-only review and coordinator integration are mandatory.
