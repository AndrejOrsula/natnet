# riders-67 C4 — say what the bound actually is

**Slice:** riders-67, attempt **C4**. Repository **`natnet`**. **Base =
`candidate(C3)` `987f2ec`** (tree `bf191d3`) — resolve it yourself. Branch
`slice/riders-67-C4`, worktree `/home/andrej/ws_rs/natnet_wt/impl-riders-67-C4`.
Freeze **C4**. Access `remote`.

Read `docs/work/riders-67/attempts/C3/review.md` in full first, **B1 first**,
then C3's `report.md`. The reviewer named the shape of this attempt and it is
small; **do not widen it.**

## 1. What is settled — do not reopen any of it

**The mechanism is right.** One non-blocking read past the deadline, and zeroes
from the allocator rather than written, both stand. The reviewer re-derived
**every** published number except one and they matched, including 182/0 over 16
targets with identical cells and the BASE comparison at 180/2 with exactly the
two new tests. The axis test's construction holds; only its base discrimination
differs (12 of 15 against 29 of 30, same direction).

## 2. What is blocking: three sentences claim an elimination you achieved a thirty-fold reduction of

`vec![0u8; n]` skips the store **only above glibc's dynamic mmap threshold**,
capped at `DEFAULT_MMAP_THRESHOLD_MAX` = 32 MiB. So `MAX_RECORD_LEN` at 64 MiB
is the one declared length that is *guaranteed* cheap, and below the cap
`calloc` memsets — **and the threshold rises by itself**, because declaring the
same length twice frees the previous mmap'd buffer.

Measured through `recv_record` on extractions of both trees, release, 15 reps:
at **31 MiB** the candidate is **0.667 ms** against the base's **0.682 ms**. At
the length an adversary picks, **your repair does the same proportional work as
the base.**

These three shipped sentences say otherwise, and one is public rustdoc:
`src/imagery/net.rs:318-320`, `:220-222`, `:58-60`. The shipped tree contains no
mention of `mmap`, `threshold`, `glibc` or `calloc`; the limit lives only in
`report.md` §2.2.

**Say what is true.** The reviewer offers one form — *at most one `memset` of
`min(total, threshold)` with no page faults* — and that is **a suggestion, not a
specification**. If you can state a bound that does not name a libc's internals,
better. If the honest sentence is that the bound is the allocator's and names
the condition under which it holds, write that. **Do not restore an
unconditional claim in different words.**

The same universal is frozen in `987f2ec`'s message, so **your C4 message
carries the retraction.**

## 3. MF1 — your cost test is aimed at the single best length

`a_declared_length_costs_nothing_past_the_deadline` uses `MAX_RECORD_LEN`, the
one length that is guaranteed cheap, and its `small * 50 + 10 ms` floor sits
above the worst case. At **31 MiB** the base passes in release. **Its
discrimination comes entirely from the length it chose.** Re-point it, and show
the base failing at the new length in both profiles or say why it cannot.

## 4. MF2 — the test you said could not be written, written

Your report says the one-read narrowing is caught by no test *"because on this
host the base loses the refill race anyway"*. The reviewer wrote it on its first
attempt: **drop the writer thread and let a blocking `write_all` return before
the call** — no race, no socket-buffer constant. Base **60/60 delivered**,
candidate **0/60**, at 256 KiB / 1 MiB / 2 MiB.

**Ship it.** And note N1: the narrowing bites at **~98 KB** on a cold connection
here, not multi-megabyte, and `buffered_peer` plateaus at 127 KB, so the
existing harness structurally cannot reach it — the largest payload it tests is
4096 B. Your docs should say where the narrowing actually starts.

## 5. MF3 — the census is seven

Claim **#65**, `take_arrived`'s base rustdoc, is falsified the same way as four
of your six rows and is absent from your table. The reviewer's extractor
reproduces your population and your numbering exactly, so this is an omission
from the table rather than a disagreement about the population. **Nothing false
ships** — you rewrote it anyway — so this is a correction to the report and to
any count that depends on it.

## 6. Two notes to act on, one to record

- **N4** — the late path discards the caller's buffer every call; your §8(2)
  spots this for the live path only. Decide whether that is a defect or
  intended, and say which.
- **N5** — `msg.txt`'s byte count and sha256 are missing from your report; they
  check out at 3915 B and `ac441fdd…`. Include them this time.
- **N6** — `origin/slice/riders-67-C3` was pushed **by me**, nine hours after
  your commit, same SHA. Your *"Pushed: no"* was true when written. Record, do
  not repair.

## 7. Practical

- **Assume you will be interrupted.** Draft `report.md` early, commit at each
  milestone. This session has lost several runs to rate limits.
- **Do not commit to `docs/work/`.** Drafts under
  `docs/work/riders-67/attempts/C4/`, uncommitted. **Never push.** No
  `codebase-audit-docs/` edit, no frozen packet edit.
- `export CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c4/target`.
- Every gate of `natnet`'s workflow with its numbers.

## 8. Acceptance

- The three sentences true as written, with **the 31 MiB measurement re-run
  against your candidate** so the corrected claim is the measured one.
- The cost test re-pointed, with the base failing at the new length.
- The narrowing test shipped, with its base/candidate counts.
- The census corrected to seven.
- The retraction in your commit message.
- Every gate with its numbers; `candidate(C4)` SHA and tree; what you are unsure
  about.
