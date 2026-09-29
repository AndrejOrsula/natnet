# riders-67 C5 — builder report

Repository `natnet`. Branch `slice/riders-67-C5`, worktree
`/home/andrej/ws_rs/natnet_wt/impl-riders-67-C5`.

## 0. Identity

| | |
|---|---|
| Base | `candidate(C4)` = `c5d1cce9519a425be227912b90da22be5e77a487`, tree `495974a3e34da2435a97166bccc594466b1a9961` |
| Base resolved | by me at **2026-09-17T21:31:32+02:00**, with `git -C /home/andrej/ws_rs/natnet_wt/impl-riders-67-C5 rev-parse HEAD` and `rev-parse HEAD^{tree}`. Both match the C5 brief. The worktree was already at that commit and clean. Not `dev` |
| `candidate(C5)` SHA | **`b48286771b3eb07d4dfbcc8354c67f391a6fe001`** |
| `candidate(C5)` tree | **`783fdeaea09314025f7c4161408a106b0463ae3a`** |
| `candidate(C5)` parents | `b482867` → `3c1ee647d230c8fc3c6b2091dc9b3d1f79ace800` (tree `2e8a4c1c…`) → `c5d1cce9519a425be227912b90da22be5e77a487`. **Two commits on the base**, not a rebase of `dev`. The second exists because naming the flake (§1) produced a fact that belongs beside the assertion, and I would rather add a commit than amend one the coordinator has already preserved |
| Files changed | `src/imagery/net.rs`, `tests/imagery_net.rs` (2 files, **+137 −51** over the base) |
| Commit message files | `/home/andrej/coord_scratch/riders67c5/msg.txt` (**4481 bytes**, carries the retractions) and `msg2.txt` (**1019 bytes**), both committed with `git commit -F`. `git log -1 --format=%B \| grep -c Co-Authored-By` → **0** |
| Checkpoints | two commits, not squashed |
| Pushed | no. No `git push` was run in this slice, to any remote |
| Code changed | **none.** `src/imagery/net.rs` is comments and rustdoc only; `tests/imagery_net.rs` is one doc comment only. No assertion, budget or test name moved |

**Interrupted once by the session rate limit** and resumed; nothing was lost.
The commit and this report were preserved by the coordinator at
`/home/andrej/coord_preserve/riders-67-C5-k1/` and every measurement was
already on disk under `/home/andrej/coord_scratch/riders67c5/`.

## 1. THE FLAKE — named, with its failure output, and there are two of them

**This is the first thing I found and the last thing I finished, and it is the
part of this attempt the coordinator should read hardest.**

### 1.1 What started it

The **first** run of `gates.sh` had one failure, in the
`cargo +1.92 test --locked --all-features` gate (MSRV, `debug`):

```
=====[MSRV test all-features]=====
...
test result: FAILED. 23 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.48s
rc=101
```

The target is `imagery_net` (seventh in this crate's order; 23+1 = 24 is its
count). **The name was lost** because my gate script filtered that gate through
`grep -E "^test result"`. That was my error. `gates.sh` now `tee`s every
`cargo test` gate in full and greps `failures:` and the indented names too.

### 1.2 Flake A — `a_peer_still_sending_past_the_deadline_is_a_timeout`

**This is C3's axis test. C4 did not touch it, C5 does not touch it, and it
produces exactly the `23 passed / 1 failed` signature of the gate failure.**

Failure output, verbatim, from `tests/imagery_net.rs:1106`:

```
a peer still sending past the deadline was allowed to finish the record:
recv_record returned Ok(true) with 5225268 bytes, but this connection absorbs
only 2612608 bytes unread and 2612608 were queued, so the rest arrived after
the deadline
```

and, under concurrency, the same assertion with 6025124/3012536 and
5762980/2881464.

**Rates I measured:**

| condition | failures |
|---|---|
| `--exact` this test, `release`, sequential | **2 / 150** |
| `--exact` this test, `debug`, sequential | 0 / 150 |
| `--exact` this test, `debug`, 4-way | 0 / 120 |
| full `imagery_net` target, `debug`, **4-way** | **2 / 40**, each giving `23 passed / 1 failed` |
| full `imagery_net` target, `debug`/`release`, sequential | 0 / 236 |

**The mechanism, from the test's own text.** It polls `peek` until the queued
count has not grown for 5 ms, concludes the connection is saturated at
`absorption` bytes, then adopts the stream and calls `recv_record` with a 1 ns
deadline, expecting `TimedOut` because the record is twice what the connection
holds. Between that last `peek` and the single non-blocking read of the payload
the test does real work — `from_stream`, a `Vec::new()`, the prefix read, which
itself frees receive window — and the peer is still sending. When the kernel
absorbs the rest in that gap, the one read finishes the record and the test's
premise is simply false for that run. It is a race between the test's
saturation measurement and its own setup, and 1.3% in `release` is what it
costs.

**I have not touched it.** It is C3's, it is outside C5's scope, and the review
re-derived it as sound. But *"every row times out on every tree, so there is no
outcome to flake"* is said of the **cost** test, not this one, and this one does
have an outcome to flake.

### 1.3 Flake B — the cost test, which is the one I am re-pointing

**Saying it plainly, as instructed: yes, the cost test can fail, and no, I have
not touched its budget.**
`the_worst_declared_length_past_the_deadline_stays_within_a_fixed_budget` fails
on its own assertion at **31 MiB with a *fresh* destination buffer**. All 24
occurrences, from `tests/imagery_net.rs:1231`:

```
declaring 32505856 bytes past an expired deadline cost 22.197945ms with a fresh
destination buffer, against 9.117µs for a 65536-byte declaration — a peer must
not be able to buy work by naming a length (allowed 10.45585ms)
```

The 24 costs run **10.913 / 10.962 / 11.025 / 11.034 / 11.038 / 11.045 /
11.266 / 11.351 / 11.496 / 11.564 / 11.681 / 11.724 / 13.123 / 13.392 / 13.813
/ 13.863 / 13.867 / 13.948 / 15.004 / 15.532 / 16.161 / 17.065 / 19.538 /
22.198 ms**, against budgets of **10.358–10.963 ms**.

**What that number is matters.** It is the **median of nine**, not one call.
Under enough load, five of the nine land in the 10–17 ms tail of §2.4 and the
median itself goes over. So the statistic the test asserts on is not robust to
the tail it was chosen to exclude — which is the same defect as B1, one level
down, and I would rather write that down than widen the budget past it.

**The condition, stated honestly, because it weakens the finding.** All 24 came
from one deliberately pathological run: 24 concurrent copies of the target
(16 `debug` + 8 `release`), 240 jobs, during which **3455 `bind` calls failed
with `AddrInUse`** because I exhausted this host's ephemeral port range
(32768–60999; `ss` showed 31 732 sockets in `TIME_WAIT`). That is my harness's
doing, not the crate's, and it is far past any gate condition.

| condition | cost-test failures |
|---|---|
| **24-way, 240 jobs (port-exhausting)** | **24** |
| 8-way, 40 runs | 0 |
| 4-way, 40 runs | 0 |
| `release` sequential, 40 runs | 0 |
| `debug` sequential, 40 runs | 0 |
| everything in §1.2's table and the 236 runs of §6 | 0 |

**0 in 400 runs at eight-way concurrency or less.**

### 1.4 Which one was the gate failure

**I do not know, and both fit the `23 / 1` signature.** Flake A is the
better-supported candidate: it reproduces at 4-way concurrency and at 1.3%
sequentially in `release`, with no port pressure at all, while Flake B needed a
host that was simultaneously failing thousands of binds. But the gate ran in
`debug`, where I could not make A fail sequentially either. I am not going to
pick one and present it as the diagnosis.

### 1.5 What I changed because of it

One thing, and it is a doc comment: the cost test now records §1.3 beside its
own assertion, with the counts and the condition, and says *"If this test fails
that way, the budget is not the thing to change — the tail is a property of the
host, and widening the budget until it passes buys nothing except a test that
discriminates less."* That is commit `b482867`. **The budget, the assertion, the
ladder, the rep count and the test names are byte-identical to C4's.** I did not
touch Flake A's test at all.

## 2. B1 — the ceiling. I dropped it, and here is why

**The choice: drop the numeric ceiling entirely.** Not re-derived with `max`
and republished. The reason is that I did re-derive it with `max`, and what
`max` returned is not a property of this crate.

### 2.1 The instrument

`recv_record(&mut buf, Duration::from_nanos(1))` against a `peek`-confirmed
twelve-byte prefix declaring `total`, nothing behind it, each rep a fresh
loopback connection. **Each declared length in its own process**, as C4 did.
`fresh` = a new `Vec::new()` per call; `reuse` = one buffer across the run.
Probes `/home/andrej/coord_scratch/riders67c5/zz_c5_max.rs` (one length per
process, reports min/p50/p90/p99/max and every raw cell) and `zz_c5_ladder.rs`
(the shipped cost test's exact shape, reporting each cell's max as well as its
median). Trees: `.../cand` = `git archive` of this candidate; `.../base` = the
same tree with `src/imagery/net.rs` taken from `2c3041f`. Host: glibc 2.39,
x86-64, 32 cores, **load average 13–32 throughout** — other work was running
and I have not pretended otherwise.

**N5, and it matters here.** C4's §1.1 named `987f2ec` as a measured tree while
§1.2's columns said "C4". Both were right, and the reason is worth recording:
`git diff 987f2ec c5d1cce -- src/imagery/net.rs` filtered of comment lines is
**empty** (I ran it), so measuring `987f2ec` *is* measuring C4's code. I have
used `2c3041f` for the base column and this candidate for the other, and said so.

### 2.2 What `max` returns

Release, four paired runs per length up to 31 MiB, one at 32 and 64 MiB,
**60 timed calls per cell**, µs:

| declared | resize fresh p50 | resize reuse p50 | here fresh p50 | here reuse p50 |
|---|---|---|---|---|
| 64 KiB | 8 | 7 | 8 | 8 |
| 1 MiB | 16–21 | 16–20 | 16–21 | 16–22 |
| 4 MiB | 40–55 | 40–54 | **91–98** | **87–95** |
| 8 MiB | 174–192 | 171–190 | 180–188 | 164–191 |
| 31 MiB | 635–718 | 652–685 | 648–693 | 677–727 |
| 32 MiB | 10 629 | 689 | **12** | **14** |
| 64 MiB | 25 422 | 1 299 | **9** | **14** |

**Max of 60, the same calls**, µs:

| declared | resize fresh | resize reuse | here fresh | here reuse |
|---|---|---|---|---|
| 64 KiB | 52 | 35 | 41 | 45 |
| 1 MiB | 330–540 | 330–463 | 293–405 | 359–504 |
| 4 MiB | 1 357–2 085 | 1 289–1 881 | 1 253–1 865 | 1 317–1 623 |
| 31 MiB | 10 018–13 626 | 10 208–13 604 | 10 274–14 156 | **11 721–16 706** |
| 32 MiB | 14 664 | 14 234 | 18 | 45 |
| 64 MiB | 28 618 | 28 251 | 13 | 29 |

**The shipped 2.803 ms is refuted, exactly as the review said.** The worst
single call at 31 MiB with a reused buffer is **11 721–16 706 µs over four
paired runs of 60 — 4.2–6.0× the figure that shipped.** The review found
13.37 ms; I find up to 16.71 ms. I am not arguing with any of it.

### 2.3 Why I dropped it rather than restating it with `max`

Three reasons, in order of weight.

**(a) The same cell's median moves by 4× with nothing but allocation history.**
At 31 MiB, reused buffer, `release`: **677–727 µs** over 60 reps with one
length per process; **2 262–2 907 µs** over 9 reps inside the cost test's
ladder (three runs); **2 803 µs** on C4's median of 15. Three instruments, one
host, one tree, one length — 0.68 ms to 2.9 ms. C4's §2 already noticed the two
instruments disagreeing and wrote a paragraph explaining it away. The
explanation was right and the conclusion drawn from it was not: a number that
moves 4× under the instrument is not a figure to publish under the word
*"ceiling"*, whichever statistic you take of it.

**(b) The tail is load, and load is not this crate's.** Every max cell above
4 MiB in both columns is 15–20× its own median. That is not an allocator
property; it is a loaded 32-core host faulting pages.

**(c) There is a bound that needs no stopwatch, and it was already in the
file.** Past the deadline the work is at most proportional to the length the
allocator actually has to zero — never more than the declared length, never
more than `MAX_RECORD_LEN`. That survives any host. It is what ships.

### 2.4 The finding the review did not have: the tail is not this branch's

The review's ablation was **64 MiB on the `mmap` path (15–44 µs) against 31 MiB
on the heap path (13.4 ms)**, both on the candidate, and concluded *"The tail is
the allocation."* The missing arm is **the base at 31 MiB**, and it has the same
tail:

| 31 MiB, release, 60 reps | max of 60 (four runs) |
|---|---|
| `2c3041f` fresh | 10 018 / 13 626 / 13 238 / 13 519 µs |
| `2c3041f` **reuse** | 10 208 / 13 604 / 13 011 / 12 654 µs |
| C5 fresh | 10 274 / 13 716 / 13 458 / 14 156 µs |
| C5 **reuse** | 14 245 / 11 721 / 15 337 / 16 706 µs |

**Both trees reach the same 10–17 ms worst call.** What this branch changes is
**how often**, and only on the reuse discipline. Counting cells over 10 ms out
of 60, two runs each:

| | over 10 ms |
|---|---|
| `2c3041f` fresh | 2/60, 2/60 |
| `2c3041f` reuse | 1/60, 1/60 |
| C5 fresh | 1/60, 1/60 |
| **C5 reuse** | **12/60, 9/60** |

So: same magnitude, ~10× the frequency, and only where the caller reuses its
buffer — which is what `*buf = record` freeing and re-allocating 31 MiB every
call would predict. **This does not rescue the 2.8 ms and I am not offering it
as a defence of it**; the ceiling is wrong either way. It corrects the
attribution, and it is now in the comment with its counts.

### 2.5 What ships instead

All four locations the review named.

**`//!` module docs** — the *"costs at most 2.8 ms / 3.3 ms against 21 ms and
202 ms"* sentence is gone. In its place: the work is *"proportional to what the
allocator actually has to zero — never more than the declared length, and never
more than [`MAX_RECORD_LEN`] — instead of to the declared length always"*,
followed by a paragraph of its own:

> **That is a bound on the work, not a bound in time, and these docs publish no
> millisecond ceiling for it.** What a declared length costs is the allocator's,
> the host's and the machine's load's to decide, and the worst single call is a
> tail this crate cannot promise anything about. The implementation comment at
> the allocation carries the measured shape, every figure labelled with the
> statistic and the rep count that produced it.

The one figure left in the module docs is the guaranteed-cheap case, and it
carries its statistic in the same sentence: *"twelve bytes naming a 64 MiB
record: ~9 µs here against ~25 ms for the same call writing them, medians of
60"*.

**`recv_record` public rustdoc** — same treatment, plus the two facts a caller
can act on: below the threshold there is no saving and a real cost, and at a
length just under the threshold with a reused buffer this branch enters a
multi-millisecond tail more often than `resize` does. No millisecond figure.

**The comment at the allocation** — carries both tables above, headed *"Every
figure below is µs over 60 timed calls: four paired runs per length up to
31 MiB, one at 32 and 64 MiB, and the ranges are across those runs"*, the debug
line with *"medians of 30 with the max in brackets"*, the regression band, the
tail-frequency counts, and a rule whose output is not nothing:

> **Nothing in this comment is a ceiling and no millisecond ceiling for this
> call ships anywhere in this crate.** An earlier revision of these files
> published 2.8 ms as the worst a declared length could cost. It was a median
> of 15, the worst call is 4-6× above it, and a median cannot see a worst case.
> Measure this with `max` or do not publish it.

**The cost test's doc comment** — §4 below.

**Sweep.** `grep -rn -E '2\.8 ms|3\.3 ms|4\.2|2\.4×|4×|21 ms|202 ms|20\.6|at
most one pass|pins the bound'` over `src` and `tests` returns, after the
change: **nothing**. Widening the pattern to a bare `2\.8 ms|3\.3 ms` returns
**three** hits in `src/imagery/net.rs`, all checked one by one: two are the
pattern matching *inside* `13.3 ms` and `102.8 ms` in the debug line, and the
third is the retraction sentence quoted above, which names the withdrawn figure
on purpose. `grep -rl` over the whole repo for `declared
length|past the deadline|mmap|calloc|alloc_zeroed` returns only
`src/imagery/net.rs`, `tests/imagery_net.rs` and two unrelated hits in
`src/decoder.rs` and `src/imagery.rs` (a `payload_len` ceiling and a record
offset). The population is those two files and nothing else.

## 3. MF1 and MF2 — the two numbers

### MF1 — the 4.2× is withdrawn, and so is its companion 4×

**Paired, medians of 60, four runs per discipline, 31 MiB, `release`**
(base / C5, µs): fresh **682/689, 673/679, 635/693, 718/648**; reuse **685/727,
652/677, 683/702, 667/695**. Ratio **0.90–1.09×**. There is no 4.2×, and the
2.803 ms cell it was computed from is §2.3(a)'s artefact.

**The conclusion it was offered for stands and is measured** — see MF2 — so the
number is struck and the conclusion kept. It stood at the comment at the
allocation and **in C4's commit message**, and C5's message carries the
retraction in full.

**Its companion.** The review cited `tests/imagery_net.rs:1148` for the same
4.2×; what actually stood there was *"the two differ by **4×** at 31 MiB"*,
about the two buffer disciplines rather than the two trees. It is refuted by the
same runs — fresh 648–693 µs against reuse 677–727 µs on this tree — and it is
gone too. `grep -n '4×\|4\.2'` over the base tree returns exactly those two
lines and over the candidate returns nothing.

### MF2 — *"up to a 2.4× cost"* removed from public rustdoc, band moved and labelled

The public rustdoc no longer bounds it numerically at all, which is the same
choice as B1 and for the same reason. The measured band is in the comment at
the allocation with its instrument:

| declared | base p50 of 60 (4 runs, both disciplines) | C5 p50 of 60 | ratio |
|---|---|---|---|
| 1 MiB | 16–21 µs | 16–22 µs | **0.95–1.22×** |
| 4 MiB | 40–55 µs | 87–98 µs | **1.65–2.38×** |
| 8 MiB | 171–192 µs | 164–191 µs | **0.89–1.05×** |
| 31 MiB | 635–718 µs | 648–727 µs | **0.90–1.09×** |

**The regression on my instrument is 4 MiB and nothing else.** 1 MiB, 8 MiB and
31 MiB are a wash. That disagrees with the review, which measured 1.4–1.6× at
1 MiB and 2.6–3.3× at 4 MiB, and with C4, which measured 2.4–2.6× at 4 MiB.
Three instruments on one host give 1.65×, 2.4× and 3.3× for the same cell, so
the comment says **"Other instruments on this host have put the 4 MiB figure as
high as 3.3×, so read it as a band and not a number"** rather than picking mine.
I have not restated the review's band as if it were mine.

## 4. MF3 — the comment changed, the assertion did not, and I measured why

**The brief offered two answers and I took the second, on measurement.**

**What making it assert on the worst cell would do.** I ran the shipped test's
*exact* shape — five lengths, two disciplines, one process, 9 reps per cell —
and recorded each cell's **max** as well as its median, three runs
(`zz_c5_ladder.rs`):

| run | worst cell median | **worst cell max** |
|---|---|---|
| 1 | 2 907 µs (31 MiB reuse) | **14 136 µs** |
| 2 | 2 262 µs | **10 574 µs** |
| 3 | 2 319 µs | **10 929 µs** |

The budget is `small * 50 + 10 ms` = **10.30 ms**. **A max-based assertion fails
on this candidate in all three runs.** And it would not be rescued by raising
the budget: the candidate's observed max over the single-length runs is
16.71 ms, and `2c3041f`'s worst single call is 28.6 ms at 64 MiB fresh, so any
budget that passes this tree and catches the base sits in [16.7, 28.6] — a
**1.7× total span**, worse than the base-side 2.13× the review already flagged
as thin, and with essentially no margin on the candidate side. A test with that
margin passes because it was tuned, not because the code is right.

**So: the assertion is byte-identical to C4's and the comment now says what it
pins.** At the comment at the allocation:

> `the_worst_declared_length_past_the_deadline_stays_within_a_fixed_budget`
> pins the *median* of this cost across the whole range and both buffer
> disciplines: no cell's median of nine may exceed 50× the 64 KiB cell plus
> 10 ms. It does not pin the worst single call and cannot — the worst single
> call at 31 MiB is above that budget on this host, on both trees.

And in the test's own doc comment, a new paragraph headed **"What this does not
pin, said plainly"** which states the budget in words, states that a cell is a
median of nine, and says: *"**this test cannot catch a wrong worst-case
figure**, and no worst-case figure is published for it to be wrong about"*. The
title sentence moved from *"What a record costs"* to *"What a record
**typically** costs"*, with *typically* flagged as load-bearing. The struck
*"4× at 31 MiB"* and *"2.8 ms … 3.3 ms against 202 ms"* sentences are replaced
by the tail-frequency fact (9–12 of 60 over 10 ms against 1 of 60), and the
`2c3041f` figures are corrected per N4.

**Everything the review said was sound about this test is untouched**: name,
ladder, both disciplines, medians of nine, `small * 50 + 10 ms`, the failure
message. A1/A2 and the BASE comparison in §6 confirm it still isolates its axis.

## 5. N3, N4, N5

**N3 — I am not defending it. It is withdrawn.** *"At most one pass over the
declared length"* is gone from the public rustdoc. The review is right that
sub-threshold the pages can be zeroed by the kernel on fault **and** memset by
`calloc`; and worse for the claim, a fault costs more than a pass, so *"one
pass"* understates rather than bounds. My own numbers put both cases on the
page: at 31 MiB the **median** is 648–727 µs = ~45 GB/s, which is one pass at
this host's single-pass rate, while the **tail** is 10–17 ms = ~2 GB/s, which is
not two passes but faulting. A constant-factor promise I can contradict with my
own instrument should not ship. What replaces it carries no constant: *"at most
proportional to the length the allocator has to zero — never more than the
declared length, and never more than [`MAX_RECORD_LEN`]"*. It is strictly
weaker, it is true of any allocator, and the strong part of the claim (the
32 MiB `DEFAULT_MMAP_THRESHOLD_MAX` cap making `MAX_RECORD_LEN` guaranteed
cheap on glibc) is measured and still stated.

**N4 — corrected, and by more than the review asked.** The docs said *"21 ms
and 202 ms"*. Re-measured at `MAX_RECORD_LEN` with a fresh buffer: `release`
**25 422 µs median of 60, max 28 618**; `debug` **227 075 µs median of 30, max
312 393**. The review's 24.2/26.6 and 206–217 are in the same place; mine are
slightly higher again, and the direction is the same — C4 understated the tree
it replaces. The module docs now say *"~25 ms"* with "medians of 60" in the same
sentence, the comment's tables carry the cells, and the test doc comment says
*"a median of 227 ms over 30 calls in `debug` and 25 ms over 60 calls in
`release`, both at `MAX_RECORD_LEN` with a fresh destination buffer"*.

**N5 — corrected in this report, §2.1.** Nothing shipped was wrong; the record
now says which tree each column came from and shows why `987f2ec` and `c5d1cce`
are the same code for this measurement.

## 6. Gates — every one, with its numbers

Full detail and provenance in `gates.md`. Raw log
`/home/andrej/coord_scratch/riders67c5/gates.log`, script `gates.sh`, per-gate
full outputs `g_*.txt`. Toolchain `cargo 1.97.1 (c980f4866 2026-06-30)`,
`rustc 1.97.1 (8bab26f4f 2026-07-14)`, LLVM 22.1.6, Ubuntu 24.4.0 (noble),
glibc 2.39; MSRV rows on `cargo 1.92.0 (344c4567c 2025-10-21)`.

fmt rc **0** · clippy rc **0** with **0** warning-or-error lines · `cargo test
--locked --all-features` **183 passed / 0 failed** over **16** targets, every
per-target cell identical to C4 (5 · 23 · 7 · 11 · 28 · 6 · **24** · 12 · 11 ·
6 · 4 · 6 · 9 · 17 · 4 · 10 doc-tests) · no-features **99 / 0** over 16 ·
`build --features net` rc **0** · example rc **0** · doc no-features rc **0** ·
doc all-features rc **0** · MSRV no-features **99 / 0** · MSRV all-features
**183 / 0** · features matrix `imagery` **127/0**, `net` **121/0**, `serde`
**101/0**, `nalgebra` **101/0**, `imagery,net` **179/0**, `serde,nalgebra`
**103/0**, all six clippy rc 0 with 0 warnings · feature-docs **16 of 16 rows
check_rc=0 doc_rc=0** · `cargo-deny` 0.20.2 rc **0** (*advisories ok, bans ok,
licenses ok, sources ok*, with exactly the two pre-existing
`license-not-encountered` warnings at `deny.toml:35,36`) · `cargo-machete`
0.9.2 rc **0**, *"didn't find any unused dependencies"*.

**This is the third run of `gates.sh` and the one that ran against the shipped
tree `783fdeae`.** The first run is §1.1's failure; the second, after
instrumenting the script, was green on tree `2e8a4c1c`; this one is green on
`783fdeae`. All three are reported rather than only the green ones.

**Clean extraction.** `git archive 2e8a4c1c…` → `.../extract`: **14** files in
`tests/`, **0** matching `zz_`, **no `docs/` directory in the archive at all**;
`src/imagery/net.rs` and `tests/imagery_net.rs` `cmp`-identical to the
worktree's; `cargo test --locked --all-features` **183 / 0** over 16 targets.

**BASE comparison.** The extracted tree with `src/imagery/net.rs` from
`2c3041f`, `--no-fail-fast`: **180 passed / 3 failed** over 16 targets, the
three being exactly `a_peer_still_sending_past_the_deadline_is_a_timeout`,
`the_worst_declared_length_past_the_deadline_stays_within_a_fixed_budget` and
`a_record_too_long_for_the_receive_queue_is_not_completed_past_its_deadline`.

**Ablations**, from the clean extraction, one edit each, own
`CARGO_TARGET_DIR`, `--no-fail-fast`:

| ablation | edit | result |
|---|---|---|
| **A1** | `take_arrived`'s `Ok(count)` arm loops again instead of returning `TimedOut` on a short read — the one-read bound removed | `imagery_net` **22 / 2**, failing exactly `a_peer_still_sending_past_the_deadline_is_a_timeout` and `a_record_too_long_for_the_receive_queue_is_not_completed_past_its_deadline` |
| **A2** | the `Instant::now() < deadline` branch removed, always `resize` — the allocation bound removed | `imagery_net` **23 / 1**, failing exactly `the_worst_declared_length_past_the_deadline_stays_within_a_fixed_budget` |

Both reproduce C4's ablations cell for cell: the cost test survives A1 and dies
under A2, the narrowing test does the reverse.

**Flake — see §1, which is where the numbers that matter are.** The
`imagery_net` binary from the clean extraction: **20/20 `debug` and 20/20
`release`** sequential, **36/36 under 12-way concurrency**, **40/40 sequential
in each profile** and **80/80 at 4- and 8-way** in the later batches. Against
that: **2/40 of `a_peer_still_sending_past_the_deadline_is_a_timeout` at 4-way**
and **2/150 of it sequentially in `release`**, and **24/240 of the cost test at
24-way with this host's port range exhausted**. The green counts are not the
finding; §1 is.

## 7. Scope compliance

- `docs/work/` : **not committed**. This report and `gates.md` are untracked;
  the candidate commit was made with `git add -A src tests` and
  `git status --porcelain --untracked-files=no` after it is empty.
- `codebase-audit-docs/` : not edited, not read this attempt.
- No frozen packet edited. `docs/work/riders-67/attempts/C4/` read only.
- Modules other than `src/imagery/net.rs` : not edited. Only
  `tests/imagery_net.rs` besides it, and only its doc comment.
- **The allocation path is byte-identical to C4's.** No `git push`. Nothing
  installed. No `Co-Authored-By` trailer. Commit message written to a file and
  committed with `git commit -F`.

## 8. What I am unsure about

- **§1, first and largest. Which flake was the gate failure.** Both fit the
  `23 / 1` signature and I could not force either in `debug` sequentially, which
  is the profile and the concurrency the gate ran at. I lean to Flake A on the
  evidence and I have not written that lean into any number.
- **Flake B's 24 occurrences all came from a host I had broken myself.** 3455
  concurrent `bind` failures is not a condition anyone will meet, and a reviewer
  could fairly say the cost test has not been shown to flake at all. My answer
  is that the *mechanism* is not conditional on ports — five of nine calls in
  the tail moves the median, and §2.4 shows the tail is real at 60 reps on an
  ordinary loaded host — but the *rate* I measured is an artefact of my harness
  and I have said so in the doc comment as well as here.
- **Flake A is a defect in a shipped test that I am reporting and not fixing.**
  That is the right scope call for this attempt — the brief says change numbers
  and sentences, not code — but it means `natnet` ships a test that fails about
  1.3% of the time in `release`, and the record of that is a paragraph in a
  builder report. If the coordinator wants it pinned, it needs its own slice.
- **My regression band disagrees with two other instruments on the same host.**
  I measure 4 MiB only, 1.65–2.38×; the review measured 4 MiB 2.6–3.3× *and*
  1 MiB 1.4–1.6×; C4 measured 4 MiB 2.4–2.6×. I have published mine with its
  instrument and cited the spread rather than reconciling them, because I do not
  know which is right and I could not see the review's harness. If the 1 MiB
  band is real and my instrument is blind to it, my *"1 MiB and 8 MiB are a
  wash"* sentence in the comment is wrong and should be struck.
- **§2.4's attribution rests on a paired max, which is a tail statistic.**
  Four paired runs of 60 is not many samples of a rare event. The
  *frequency* difference (12/60 vs 1/60, twice) I believe; the claim that the
  *magnitudes* are equal is four numbers against four numbers and could move.
- **The load on this host was 13–32 throughout and I did not control it.** Every
  tail figure in this report is a figure for a busy machine. That is the point
  of dropping the ceiling rather than restating it, but it also means the
  medians are probably a little high and the maxima are certainly instrument-
  and-load-bound. A quiet host would produce different tables and the same
  conclusion.
- **I did not re-derive the 98 304-byte receive-queue figure**, the narrowing
  test's counts (60/60 vs 0/60), or the census of seven. The review confirmed
  all three and they are outside C5's scope; I left the sentences that carry
  them exactly as C4 shipped them.
- **"No millisecond ceiling for this call ships anywhere in this crate"** is a
  universal I put in a shipped comment. I checked it with the grep in §2.5 over
  `src` and `tests`; I did not check `README.md`, `examples/` or
  `codebase-audit-docs/`, the last of which I am forbidden to read. The grep for
  `declared length|past the deadline|mmap|calloc|alloc_zeroed` over the whole
  repo returned only the two files I edited, which is my evidence, and it is a
  reach check rather than a proof.
