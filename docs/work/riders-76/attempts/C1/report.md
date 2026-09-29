# riders-76 C1 — builder report

## 1. Change

Candidate `cb1495ba83458382264f1de05bffb34605fb7d57` changes only
`tests/imagery_net.rs`.

- Removes `a_peer_still_sending_past_the_deadline_is_a_timeout` and its
  absorption helper. The old test started a peer `write_all` concurrently with
  `recv_record`, used a peek buffer capped at the measured absorption amount,
  and could not establish which bytes arrived before the deadline.
- Strengthens the existing queue-bound control. A channel waits for the peer's
  complete blocking `write_all`; a whole-record peek proves `queued < total` is
  informative; the call must return `TimedOut`, clear a stale buffer, and
  retain the terminal error on retry.
- Leaves the fully buffered positive controls, incomplete-body timeout,
  byte-drip deadline, and blocking-mode restoration controls intact.

No production source, public deadline behavior, workflow, manifest, README,
other test, benchmark, hardware, or feature surface changed. No test is
ignored.

## 2. Premise and controls

Before editing, the base flaky test passed 100/100 serial invocations. The
flaw is structural in the base: its writer thread remained active while
`recv_record` consumed the prefix, and its capped peek could not prove queue
stability. The implementation's post-deadline read has no per-byte arrival
timestamps, so that premise is not observable through the public API.

The repaired queue test passed 100/100 serial invocations after the candidate
freeze. Each invocation exercises three record sizes and five repetitions (15
queue-bound cases per invocation; 1,500 cases total). Load average was
`11.72 12.05 11.70` before and `11.74 12.05 11.70` after the run on the
32-core x86-64 Linux host. These counts are stability evidence, not a timing
or performance claim.

In a detached temporary worktree, a safe source ablation changed the
post-deadline single read into a repeated drain. The repaired queue test then
failed with `Ok(true)` at its timeout assertion, while both complete-prebuffered
positive controls passed. The ablation worktree was removed; the candidate
source stayed unchanged.

## 3. Candidate identity

| field | value |
|---|---|
| base | `bf1e1fda073662722a07fb57e2895da72a2a1a76` |
| candidate(C1) | `cb1495ba83458382264f1de05bffb34605fb7d57` |
| candidate tree | `27e0ca06991cea4b29c8811b48957b9542658ef2` |
| branch | `slice/riders-76` |
| worktree | `/home/andrej/ws_rs/natnet_wt/impl-riders-76` |
| committed paths | `tests/imagery_net.rs` only |
| toolchain | `rustc 1.98.1`, `cargo 1.98.1`, `x86_64-unknown-linux-gnu` |
| push | none |

The implementation worktree is clean after the candidate freeze. Gate details
are in `gates.md`; both packet drafts remain uncommitted for coordinator
review.

## 4. Residual risk and limits

The queue-bound assertion is host/socket-capacity dependent by design. It
fails with a diagnostic if a whole record is observed queued, because that
environment cannot distinguish the one-read property. No Windows, Motive,
hardware, benchmark, or live-network evidence was run.
