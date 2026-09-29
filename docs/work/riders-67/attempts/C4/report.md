# riders-67 C4 — builder report

Repository `natnet`. Branch `slice/riders-67-C4`, worktree
`/home/andrej/ws_rs/natnet_wt/impl-riders-67-C4`.

## 0. Identity

| | |
|---|---|
| Base | `candidate(C3)` = `987f2ecc1d66a01ad80346efbf1a2bc8faf3ff90`, tree `bf191d39d468431e985b55c228071a4890beb09c` |
| Base resolved | by me at **2026-09-17T16:22:47+02:00**, with `git -C /home/andrej/ws_rs/natnet_wt/impl-riders-67-C4 rev-parse HEAD` and `rev-parse HEAD^{tree}`. Both match the C4 brief. The worktree was already at that commit and clean. Not `dev` |
| `candidate(C4)` SHA | **`c5d1cce9519a425be227912b90da22be5e77a487`** |
| `candidate(C4)` tree | **`495974a3e34da2435a97166bccc594466b1a9961`** |
| `candidate(C4)` parent | `987f2ecc1d66a01ad80346efbf1a2bc8faf3ff90` — one commit on the base, not a rebase of `dev` |
| Files changed | `src/imagery/net.rs`, `tests/imagery_net.rs` (2 files, **+235 −38**) |
| Commit message file | `/home/andrej/coord_scratch/riders67c4/msg.txt`, **3349 bytes**, sha256 `a1bf5a6bb63f48c4681ba4079401d3d4eaf972bda1f19ca932abb67aed6f16e7`, committed with `git commit -F` (**N5**) |
| Checkpoints | one commit, not squashed |
| Pushed | no. No `git push` was run in this slice, to any remote |

Toolchain, `cargo-deny`/`cargo-machete` provenance and every gate number are in
`gates.md`. **Interrupted once by the session rate limit** and resumed; nothing
was lost — the coordinator preserved the working diff at
`/home/andrej/coord_preserve/riders-67-C4-killed/tracked.diff` and every
measurement was already on disk under `/home/andrej/coord_scratch/riders67c4/`.

**N6, recorded not repaired.** `origin/slice/riders-67-C3` was pushed by the
coordinator nine hours after C3's commit, same SHA. C3's *"Pushed: no"* was
true when written and I have not touched it.

## 1. B1 — the three sentences, and what replaced them

The review is right and I am not arguing with any of it. The measurement below
is mine, taken again on both trees, and it is **worse for the repair than the
review's**, so the corrected sentences are built on it rather than on the
review's two cells.

### 1.1 The instrument

`recv_record(&mut buf, Duration::from_nanos(1))` against a `peek`-confirmed
twelve-byte prefix declaring `total`, nothing behind it, each rep a fresh
loopback connection, median of 15. Two things the review's table does not
separate and that turn out to matter as much as the length:

- **each declared length in its own process.** The dynamic `mmap` threshold is
  process state — it rises as this branch frees mapped blocks — so a ladder run
  inside one process measures a different allocator than a single length does.
- **the destination-buffer discipline.** `fresh` = a new `Vec::new()` per call;
  `reuse` = one buffer across the run, which is how a caller in a receive loop
  actually holds it. The two differ by **4×** on the candidate at 31 MiB.

Probes: `/home/andrej/coord_scratch/riders67c4/zz_c4_one.rs` (one length per
process) and `zz_c4_cost.rs` (ladder). Trees: `git archive` extractions of
`2c3041f` and `987f2ec` at `.../base` and `.../cand`. Host: glibc 2.39,
x86-64, 32 cores.

### 1.2 The numbers, `release` / `debug`, median of 15

| declared | base fresh | base reuse | C4 fresh | C4 reuse |
|---|---|---|---|---|
| 64 KiB | 8.54 µs / 195.8 µs | 5.65 µs / 197.2 µs | 5.74 µs / 9.56 µs | 5.89 µs / 6.66 µs |
| 4 MiB | 38.2 µs / 11.674 ms | 37.7 µs / 11.813 ms | **90.2 µs** / 92.7 µs | **97.5 µs** / 101.5 µs |
| **31 MiB** | **715.7 µs** / 91.095 ms | **672.8 µs** / 90.657 ms | **704.0 µs** / 670.0 µs | **2.803 ms** / 3.266 ms |
| 32 MiB | 10.113 ms / 100.825 ms | 708.9 µs / 93.745 ms | 8.94 µs / 10.72 µs | 13.6 µs / 15.19 µs |
| 64 MiB | 20.625 ms / 201.566 ms | 1.458 ms / 187.342 ms | 9.95 µs / 9.86 µs | 13.4 µs / 15.05 µs |

Three further `release` runs of the two sub-threshold rows, to show they are not
one sample: base 4 MiB 37.5 / 37.9 / 37.6 µs fresh and 37.3 / 37.7 / 37.7 µs
reuse; C4 4 MiB 94.3 / 88.7 / 97.5 µs fresh and 86.7 / 92.2 / 88.0 µs reuse;
base 31 MiB 683 / 697 / 693 µs fresh and 660 / 701 / 668 µs reuse; C4 31 MiB
690 / 685 / 647 µs fresh and **2.108 / 3.106 / 1.858 ms** reuse.

**The 31 MiB row re-run against my candidate reproduces the review and then goes
past it.** With a fresh buffer the two trees are the same: **704 µs** against
**716 µs** (the review had 0.667 against 0.682 ms). With a *reused* buffer the
candidate is **2.803 ms against the base's 673 µs — 4.2× worse**. And at 4 MiB
in `release` the candidate is **2.4–2.6× worse** at every measurement I took.
The review said the candidate does *"the same proportional work"* below the cap;
below the cap it does **more**, and I would rather say so than let the ceiling
sentence rest on a row that is only a wash.

Why: `*buf = record` must allocate the new destination while the old one is
still alive, so it can never reuse it. `resize` on a buffer that already has
the capacity rewrites hot pages; this branch zeroes cold ones every call, and
below the threshold it pays a `memset` for them anyway. That is **N4**, and
§4 answers it.

### 1.3 What the three sentences now say

`src/imagery/net.rs`, at the same three places. Each is quoted as shipped.

**(a) `//!` module docs** (was *"no work is done in proportion to a length the
peer merely declared"*):

> What a peer can buy by *declaring* a length it has not sent is capped rather
> than removed: past the deadline the destination's zeroes are asked of the
> allocator instead of written here, so the worst declared length costs at most
> **2.8 ms** in `release` and **3.3 ms** in `debug` on this host, against
> **21 ms** and **202 ms** for the same call writing them — but the saving is
> the allocator's to give. Where it hands back memory that is already zero the
> length costs nothing at all (twelve bytes naming a 64 MiB record: ~10 µs
> against ~21 ms); where it does not, the zeroes are still written once, and for
> those lengths this call can be *slower* than writing them itself.

It then points at the implementation comment for the mechanism, and states the
narrowing with the number N1 asked for: the receive queue **settles at 98 304
bytes** on a cold loopback connection here, *"so the narrowing starts below
100 KB and not at multi-megabyte records"*.

**(b) `recv_record`'s public rustdoc** (was *"a length the peer declared but has
not delivered costs this call an allocation, not a write to every byte of it"*):

> And a length the peer declared but has not delivered is *capped* in what it
> can cost this call, because the destination's zeroes are requested from the
> allocator rather than written by this function. That is a ceiling, not an
> elimination, and the ceiling is the allocator's: where it can answer with
> memory that is already zero the declared length costs nothing to zero at all,
> and where it cannot the zeroes are written once anyway. Either way this call
> makes at most one pass over the declared length — never more than one, and
> never more than a pass over [`MAX_RECORD_LEN`], whatever the peer declares.

followed by the two measured worst cases and *"below the allocator's threshold
for handing back fresh memory there is no saving, and up to a 2.4× cost"*. The
same paragraph now gives 98 304 bytes for where the narrowing starts.

**The bound that does not name a libc.** The brief invited one, and *"at most
one pass over the declared length, never more than one and never more than a
pass over `MAX_RECORD_LEN`"* is it: it is true of any allocator, it is what a
caller can actually rely on, and it is strictly weaker than the reviewer's
`min(total, threshold)` form — which is the honest direction, because the
threshold is not a constant and the reviewer's form still needs glibc to read.
It is also a bound the **base** satisfies, so I did not let it stand alone: the
rustdoc pairs it with the two measured worst cases and with the sentence that
there is no saving below the threshold. The strong part of the claim is
measured, not asserted.

**(c) the comment at the allocation** (was *"the only pages that cost anything
are the ones the single read below actually fills"*). This is where the
mechanism goes, because the review's *"`grep` for `mmap|threshold|glibc|calloc`
in the shipped tree returns nothing"* is the §7.1(a) problem and a report
cannot fix it. It now names `alloc_zeroed`/`calloc`, the **dynamic** `mmap`
threshold, that it starts at 128 KiB, that **this branch's own `*buf = record`
frees a mapped block every call and so raises it**, that it is capped at
`DEFAULT_MMAP_THRESHOLD_MAX` = 32 MiB, and that `MAX_RECORD_LEN` is therefore
the one declared length guaranteed cheap. It carries §1.2's table, the
20.6 → 2.8 ms and 202 → 3.3 ms worst cases, and the regression:

> It is not free: below the threshold this branch is *slower* than the store it
> replaces in `release` (2.4-2.6× at 4 MiB, up to 4.2× at 31 MiB with a reused
> buffer), because it must allocate the new destination while the old one is
> still alive and so zeroes cold pages where `resize` rewrote hot ones.

`grep -cE "mmap|threshold|glibc|calloc|alloc_zeroed|DEFAULT_MMAP"` over
`src/imagery/net.rs` returns **0** at `987f2ec` — the review's *"returns
nothing"*, confirmed — and **7** at `c5d1cce`, with **2** more in
`tests/imagery_net.rs`. (The only other hit in the crate is a pre-existing
`src/frame.rs:608` *"threshold comparison"*, unrelated.)

**What I did not do.** I did not restore an unconditional claim. Grepping the
two files **flattened** for `eliminat|no work is done|O\(1\)|costs? nothing|not
a write to every|only the second is a timeout` returns four hits, all in
`src/imagery/net.rs` and **all four inside the clause that states their
condition**: two are *"a ceiling, not an elimination"* / *"a ceiling rather than
an elimination"*, one is *"where it hands back memory that is already zero the
length costs nothing at all"*, and one is *"`MAX_RECORD_LEN` (64 MiB) is the one
declared length guaranteed to cost nothing to zero"*. `tests/imagery_net.rs`
returns none.

## 2. MF1 — the cost test re-pointed

`a_declared_length_costs_nothing_past_the_deadline` is gone, name and all: the
name was itself one of the false sentences. It is now
**`the_worst_declared_length_past_the_deadline_stays_within_a_fixed_budget`**.

**A ladder, not a length.** Five declared lengths — 64 KiB, 4 MiB, 31 MiB,
32 MiB, `MAX_RECORD_LEN` — each under **both** buffer disciplines, ten cells,
medians of nine. The assertion is on the **worst** cell against
`small * 50 + 10 ms`, where `small` is the cheaper of the two 64 KiB cells. A
test that measures only `MAX_RECORD_LEN` cannot fail for the reason this one
can: the worst cell is free to be any of the ten, and on the candidate it is
**not** the one the old test used.

Run on the base (`2c3041f`'s `net.rs` in the C4 tree), both profiles — it fails,
and the ladder is in the failure message:

| declared | base `release` | base `debug` | C4 `release` | C4 `debug` |
|---|---|---|---|---|
| 64 KiB fresh | 6.131 µs | 187.191 µs | 5.83 µs | 6.493 µs |
| 4 MiB fresh | 38.201 µs | 11.861 ms | 96.891 µs | 89.166 µs |
| 31 MiB fresh | 700.050 µs | 91.068 ms | 675.374 µs | 701.302 µs |
| 32 MiB fresh | **10.012 ms** | 100.955 ms | 8.827 µs | 11.261 µs |
| 64 MiB fresh | **20.235 ms** | **201.293 ms** | 9.097 µs | 9.919 µs |
| 64 KiB reuse | 5.741 µs | 192.571 µs | 5.751 µs | 6.542 µs |
| 4 MiB reuse | 38.242 µs | 11.627 ms | 95.389 µs | 94.206 µs |
| 31 MiB reuse | 680.383 µs | 89.860 ms | 658.632 µs | 695.912 µs |
| 32 MiB reuse | 727.160 µs | 93.078 ms | **694.049 µs** | **740.776 µs** |
| 64 MiB reuse | 1.427 ms | 186.056 ms | 13.275 µs | 14.598 µs |
| **budget** | **10.287 ms** | **19.360 ms** | 10.288 ms | 10.325 ms |
| **verdict** | **FAIL** (20.235 ms) | **FAIL** (201.293 ms) | pass (0.694 ms, 14.8× margin) | pass (0.741 ms, 13.9× margin) |

**The base fails in both profiles.** It failed in `release` before only because
the old test picked 64 MiB; it fails here because the ladder *contains* 64 MiB
alongside four lengths at which the base is fine, and asserts on the worst.

**Where the base cannot be made to fail, and why I say so rather than hide it.**
At a sub-threshold length in `release` — 4 MiB, 31 MiB — **no** test can fail on
the base and pass on the candidate, because the candidate is not faster there;
it is 2.4–4.2× slower. The brief allowed *"or say why it cannot"* and that is
the why: in `release` the whole of the improvement lives above the 32 MiB cap.
In `debug` the candidate is faster at every length measured — 20–30× at
64 KiB, ~126× at 4 MiB, 28× (reused buffer) to 136× (fresh) at 31 MiB — so
there a sub-threshold test would discriminate
— I did not add one, because a test that only discriminates in one profile is
the mirror of the defect being repaired.

**The candidate's worst cell inside this test (0.69–0.74 ms) is smaller than its
worst cell in §1.2 (2.8 ms).** They are both real: §1.2 gives each length its
own process, and the ladder shares one, so the allocator arrives at 31 MiB in a
different state. The docs quote the **2.8 ms** figure — the worse one — and the
test's budget clears it four-fold either way. A reviewer who runs the test and
gets 0.7 ms has not caught the docs overstating; they have caught the two
instruments differing, which is this paragraph.

**Ablation.** The re-pointed test dies under **A2** (late-allocation branch
removed) and **survives A1** (one-read bound removed) — so it still isolates the
allocation axis and nothing else. `gates.md` has both runs.

## 3. MF2 — the narrowing test, shipped

C3's report said the one-read narrowing was caught by no test *"because on this
host the base loses the refill race anyway"*. **That was wrong**, and the
reviewer's construction is right. Shipped as
**`a_record_too_long_for_the_receive_queue_is_not_completed_past_its_deadline`**.

- The peer writes the **whole record** with one blocking `write_all` on its own
  thread, and the test **waits for that call to return** (`mpsc` + a 20 s
  `recv_timeout`) before it constructs the client. There is no concurrent
  writer, nothing for the peer to drive, and no race: every byte is already the
  kernel's.
- What makes the record uncompletable is that *this* end's receive queue does
  not hold it all at once — and the test **measures** that with `peek`, polling
  until two samples 20 ms apart agree, then asserts `queued < total` before it
  asserts anything else. **No socket-buffer constant appears in the test.**
- Sizes **256 KiB, 1 MiB, 2 MiB**, five reps each.

**Where the narrowing actually starts (N1).** The queue settles at **98 304
bytes — exactly, on all 15 reps, at every one of the three sizes.** The
reviewer's ~98 KB is confirmed to the byte. So the narrowing begins below
100 KB, not in the multi-megabyte range C3's report implied; that number is now
in the module docs, in `recv_record`'s rustdoc and in the test's own doc
comment. C3's *"a few megabytes"* figure was a **warm** connection's queue and
I have left that reading beside the cold one rather than striking it, because
both were measured — what was wrong was reporting only the warm one.

**Counts, from a 20-reps-per-size probe**
(`/home/andrej/coord_scratch/riders67c4/zz_c4_narrow.rs`, run on both trees in
both profiles):

| tree | profile | delivered | timed out |
|---|---|---|---|
| `2c3041f` | release | **60 / 60** | 0 |
| `2c3041f` | debug | **60 / 60** | 0 |
| C4 | release | **0 / 60** | 60 |
| C4 | debug | **0 / 60** | 60 |

The reviewer's 60/60 and 0/60 reproduce exactly, in both profiles. In the BASE
comparison the shipped test fails on `2c3041f` with
*"a record the receive queue cannot hold at once must not be completed past its
deadline: true"*.

**Ablation.** It dies under **A1** and survives **A2** — the exact mirror of the
cost test.

## 4. N4 — the buffer discard: intended, inherent, and now costed

The late path's `*buf = record` discards the caller's buffer on every call.
**Decision: intended, and inseparable from the mechanism — but it is what makes
the sub-threshold regression of §1.2, and that is now written down where the
branch is.**

It cannot be avoided while keeping the bound. The whole point is to obtain
zeroes the allocator already has; a fresh zeroed allocation is by definition not
the caller's existing one, so "reuse the buffer" and "do not write the zeroes"
are the same choice made two ways. Doing it only above some length would need
the magic constant C3's §2.2 already refused and the review endorsed refusing.

What changed is that it is no longer free in my account of it. C3's §8(2) spotted
the discard for the **live** path and treated it as a reason not to extend the
change there; the late path's discard has a **measured** price — 2.4–2.6× at
4 MiB and 4.2× at 31 MiB in `release` — and the comment at the branch now names
it, names its cause (allocating the new destination while the old is alive) and
gives the numbers. I did not change the code: the trade is right at the length
an adversary picks on the base (20.6 ms → 9.9 µs at 64 MiB) and in `debug`
everywhere, and reversing it would restore the 205 ms case the slice exists to
remove.

## 5. MF3 — the census is seven

The review is right and the extractor reproduced my population and numbering
exactly, so this is an omission from my table, not a disagreement.

**Claim #65**, `take_arrived`'s rustdoc at `2c3041f`
(`src/imagery/net.rs:357-368`):

> The deadline bounds how long this client will *wait*; it says nothing about
> bytes the kernel is already holding. So once it passes, **every remaining read
> is issued non-blocking**: each one either takes bytes that were there before
> it ran, or reports that none are, and **only the second is a timeout**.

Falsified twice over by C3's change, the same way as rows 1 and 4–6: there is at
most **one** remaining read, not "every remaining read"; and a read that finds
*some but not all* is now also a timeout, so "only the second is a timeout" no
longer covers the outcomes. C3 rewrote it — the candidate's `take_arrived`
rustdoc reads *"the rest of the destination is taken with **one** non-blocking
read"* — so **nothing false shipped**; the defect is that the sweep's table
claimed six and the correct count is **seven**.

**The reverse sweep's verdict, corrected: seven falsified, all in
`src/imagery/net.rs`, all rewritten.** Every count in C3's report that rests on
"six" reads seven: §4's *"Verdict: six falsified"*, its table's six rows, and
*"Five of the six I had not otherwise touched"* — which becomes six of the
seven, since #65 is also one I had not otherwise set out to touch. The two
further findings labelled separately in C3 §4 are unaffected; they were never
counted in the six.

I did not re-run the extractor. The review's reproduces my population and my
numbering, the sentence is quoted above from `git show 2c3041f:…`, and I
confirmed by reading the candidate that it no longer ships. **This is a
correction to the report only. No shipped text changes.**

## 6. What does not ship, and where it is

- `zz_c4_one.rs`, `zz_c4_cost.rs`, `zz_c4_narrow.rs` —
  `/home/andrej/coord_scratch/riders67c4/`. **None of them ships**: the clean
  extraction has 14 files in `tests/`, 0 matching `zz_`.
- Trees: `.../base` and `.../cand` (`git archive` of `2c3041f` / `987f2ec`),
  `.../extract` (C4), `.../basecmp2` (BASE comparison), `.../abl_A1`,
  `.../abl_A2`, `.../candprint` (a copy with the budget forced to zero so the
  passing ladder prints its cells).
- `.../gates.log`, `.../matrix.md`, `.../msg.txt`, `.../gates.sh`.

## 7. Scope compliance

- `docs/work/` : **not committed**. This report and `gates.md` are untracked;
  `git status --porcelain --untracked-files=no` after the candidate commit is
  empty.
- `codebase-audit-docs/` : not edited, not read this attempt.
- No frozen packet edited. `docs/work/riders-67/attempts/C3/` read only.
- Modules other than `src/imagery/net.rs` : not edited. Only
  `tests/imagery_net.rs` besides it.
- No `git push`. Nothing installed. No `Co-Authored-By` trailer.

## 8. What I am unsure about

- **The 2.8 ms figure in the public docs is the worst cell I could produce, not
  a proof.** It came from a process that allocated only 31 MiB; a different
  allocation history could produce a worse one, and I have no argument that it
  cannot. The bound I can defend without measurement is the weak one — at most
  one pass over the declared length — and that is why the rustdoc states both.
  **If a reviewer produces a cell above 2.8 ms I would treat the sentence as
  wrong, not the cell.**
- **The regression is a behaviour change I am shipping rather than fixing.**
  In `release`, a peer that repeatedly declares 4 MiB past the deadline now
  costs 2.4× what it cost on `2c3041f`. It is bounded and small in absolute
  terms (98 µs), and the alternative is the magic constant, but a reviewer could
  reasonably call it a defect rather than a trade and I would not argue.
- **My §1.2 disagrees with the review's 31 MiB candidate cell** (2.803 ms reused
  against its 0.667 ms). I think the difference is the buffer discipline and the
  per-process isolation, because my *fresh* cell is 0.704 ms and matches it; but
  I did not see the reviewer's harness, so that is an inference.
- **The ladder test's budget is `small * 50 + 10 ms` unchanged from C3.** It
  clears the candidate 14× and catches the base 2× in `release`. The 2× is the
  thin side, and it is thin because the base's 64 MiB fresh cell (20.2 ms) is
  only twice the floor. A host with slower memory widens it; a host with faster
  memory narrows it, and at roughly 2× faster than this one the base would start
  passing.
- **The narrowing test's `queued < total` precondition is an assert, not a
  skip.** On a host whose receive queue holds 2 MiB cold it would fail rather
  than adapt. I preferred a loud failure to a test that silently stops
  discriminating, but it is a host assumption and `absorption_of` shows this
  suite knows how to avoid one.
- **I did not re-derive the axis test's base discrimination** (the review's 12
  of 15 against C3's 29 of 30). It is the one figure the review could not
  reproduce and it is not in my scope; I accept the review's.
- **The `debug`/`release` asymmetry is now load-bearing and I have not explained
  it fully.** `calloc`'s `memset` is libc's in both profiles, while `resize` is
  a Rust loop that only vectorises in `release` — that accounts for the 20–130×
  `debug` wins, but I did not disassemble anything to confirm it.
