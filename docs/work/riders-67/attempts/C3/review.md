# riders-67 C3 review — `natnet`, candidate `987f2ec`, tree `bf191d3`

## Verdict: reject — one blocking finding

**Nothing here is report-only**: the blocking finding is three shipped
sentences, one of them public rustdoc, and MF1 is a shipped test.

## B1 — blocking. The B1a bound is the allocator's, and the peer picks the length that escapes it

Three shipped sentences state a universal that is false:

- `src/imagery/net.rs:318-320` — *"the only pages that cost anything are the
  ones the single read below actually fills"*
- `src/imagery/net.rs:220-222` (**public rustdoc**) — *"a length the peer
  declared but has not delivered costs this call an allocation, not a write to
  every byte of it"*
- `src/imagery/net.rs:58-60` (`//!`) — *"no work is done in proportion to a
  length the peer merely declared"*

`vec![0u8; n]` skips the store only above glibc's **dynamic** mmap threshold,
capped at `DEFAULT_MMAP_THRESHOLD_MAX` = 32 MiB. `MAX_RECORD_LEN` (64 MiB) is
therefore the one declaration that is *guaranteed* cheap — and the cheapest
there is. Below the cap `calloc` memsets, and **the threshold rises by itself:
declaring the same length twice is enough**, because the candidate's
`*buf = record` frees the previous mmap'd buffer.

Reproduced end-to-end through `recv_record` against `git archive` extractions of
both trees (release, one reused buffer, 15 reps):

| declared | base `2c3041f` | candidate `987f2ec` |
| --- | --- | --- |
| 31 MiB | median **0.682 ms** | median **0.667 ms** |
| 64 MiB | median 1.405 ms (21.3 ms with a fresh buffer) | median **0.013 ms** |

**At the length an adversary picks, the candidate does the same proportional
work as the base, in `release`.** The improvement is real — worst case over all
lengths falls ~21 ms → ~0.7 ms release, ~205 ms → ~0.7 ms debug — but it is a
~30× reduction, not the elimination the docs state.

`grep` for `mmap|threshold|glibc|calloc` in the shipped tree returns nothing:
the limit is disclosed only in `report.md` §2.2, which is exactly what §7.1 (a)
is written for. Graded on (a) ∧ (b) ∧ (c) plus the first general rule, with the
false-premise conflict declared — the sentences **are** the contract, not a
reason offered for one (riders-21 C1 F1's ground).

The report's own bullet *"the cost no longer grows with the declared length"* is
refuted by its own neighbouring numbers.

## MF1 — the new cost test is aimed at the single best length

`a_declared_length_costs_nothing_past_the_deadline` uses `MAX_RECORD_LEN`, and
its `small * 50 + 10 ms` floor is above the worst case. Changing that one line
to 31 MiB and running it on both trees: candidate pass/pass, **base release
pass**, base debug fail. **Its discrimination comes entirely from the length it
chose.**

## MF2 — *"no test catches it"* is false; the reviewer wrote it on the first attempt

Drop the writer thread and let a blocking `write_all` return before the call —
no race, no socket-buffer constant. Base **60/60 delivered**, candidate
**0/60**, at 256 KiB / 1 MiB / 2 MiB. Report-only → must-fix under the
2026-09-04 third-pairing ruling.

## MF3 — the reverse sweep's census is seven, not six

An independent extractor reproduces the population **exactly** (net.rs
78 = 71 + 7 + 0; tests 42 comment sentences, 21 test names) and its numbering —
all six table rows and both extra findings land on identical indices. Claim
**#65**, `take_arrived`'s base rustdoc (*"every remaining read is issued
non-blocking … and only the second is a timeout"*), is falsified the same way as
rows 1 and 4–6 and is absent from the table; the report never quotes it. **It
was rewritten anyway, so nothing false ships.**

## Notes

- **N1** — the narrowing bites at **~98 KB** on a cold connection here, not
  "multi-megabyte", and `buffered_peer` plateaus at 127 KB so the existing
  harness **structurally cannot reach it** (largest tested payload: 4096 B).
- **N3** — the false universal is frozen in `987f2ec`'s message too.
- **N4** — the late path discards the caller's buffer every call; report §8(2)
  spots this for the live path only.
- **N5** — `msg.txt`'s byte count and sha256 are missing from the report. They
  check out: 3915 B, `ac441fdd…`.
- **N6** — `origin/slice/riders-67-C3` exists, `update by push` at 2026-09-14
  10:48:28, nine hours after the commit — same SHA, not attributable to the
  builder, but *"Pushed: no"* no longer describes the repository. *(Coordinator:
  that push is mine.)*

## Every published number reproduces

fmt/clippy rc 0 · **182/0 over 16 targets with identical cells** · 99/0 · doc ×2
rc 0 · MSRV 1.92.0 99/0 and 182/0 · features 127/121/101/101/178/103 · all 16
feature-docs rows · deny 0.20.2 with the same two `deny.toml:35,36` warnings ·
machete 0.9.2 (and it genuinely is not on `PATH`) · clean extraction 14/0 ·
**BASE comparison 180/2 with exactly the two new tests** · identity, parent and
checkpoint tree all match.

The only figure that does not reproduce is the axis test's discrimination on the
base — **12 of 15** here against the report's 29 of 30, same direction.

## The shape of C4

The mechanism is right and C4 is small: **qualify the three sentences** (or
state the true bound — at most one `memset` of `min(total, threshold)` with no
page faults), **re-point the cost test at a sub-threshold length**, **ship the
narrowing test**, and **correct the census to seven**.
