# riders-67 C6 — builder report

Repository `natnet`. Branch `slice/riders-67-C6`, worktree
`/home/andrej/ws_rs/natnet_wt/impl-riders-67-C6`.

**Four sentences and three numbers. The allocation path and every test body
were left alone, and nothing C5 was verified on was reopened.**

## 0. Identity

| | |
|---|---|
| Base | `candidate(C5)` = `b48286771b3eb07d4dfbcc8354c67f391a6fe001`, tree `783fdeaea09314025f7c4161408a106b0463ae3a` |
| Base resolved | by me at **2026-09-18T21:05:09+02:00**, `git -C /home/andrej/ws_rs/natnet_wt/impl-riders-67-C6 rev-parse HEAD` and `rev-parse HEAD^{tree}`. Both match the C6 brief. The worktree was already at that commit and clean (`status --short --branch` printed `## slice/riders-67-C6` and nothing else). Not `dev` |
| `candidate(C6)` SHA | **`b03817b271ba0c9761577bbdeeeeebe44efd675b`** |
| `candidate(C6)` tree | **`e5fc608c2d999ad5d4dfe32102eb12674c7ae48b`** |
| `candidate(C6)` parents | `b03817b` → `0f3fc5734693df3790b0c7822df74ca8c113d9a4` (tree `d45a3a4e…`) → `b48286771b3eb07d4dfbcc8354c67f391a6fe001`. **Two commits on the base**, not a rebase of `dev`. The second is one sentence of `README.md`: the moved rule's load example said *"the same `MAX_RECORD_LEN` declaration on the same tree"*, which a reader can take as this crate's current code, where that declaration costs ~9 µs and not 202 ms. Both figures are `2c3041f`'s. I would rather add a commit than amend one |
| Files changed | `README.md`, `src/imagery/net.rs`, `tests/imagery_net.rs` (3 files, **+62 −22** over the base) |
| Code changed | **none.** Stripped of comments and doc comments by a real Rust lexer that preserves string and raw-string literals, `src/imagery/net.rs` and `tests/imagery_net.rs` are **byte-identical to `b482867`** (md5 `6d8cad89dbd1d525ff423088a00c4c0d` and `4c6cd4cb94ed11bda3eadb5c22ed4f63`, base and C6 alike). The only non-comment text added anywhere in the repository is the new `README.md` section |
| Commit message files | `/home/andrej/coord_scratch/riders67c6/msg.txt` (**2780 bytes**, carries the retractions) and `msg2.txt` (**739 bytes**), both committed with `git commit -F`; no `git commit -m` was run. `git log -2 --format=%B \| grep -c Co-Authored-By` → **0** |
| Staging | `git add README.md src/imagery/net.rs tests/imagery_net.rs`, then `git add README.md` — explicit paths, no `-A`. `git status --porcelain --untracked-files=no` after each commit is empty; the only untracked entry is `docs/` |
| Pushed | **no.** No `git push` was run in this slice, to any remote |
| Checkpoints | two commits, not squashed |
| Not interrupted | the session limit did not fire this attempt |

## 1. Load, first, because it is the variable that broke C5's figures

`nproc` = **32**. `/proc/loadavg` was read immediately before and after every
timing run and both readings are given. Port pressure was read as
`ss -tan | grep -c TIME-WAIT` before and after the campaign.

| run | load before | load after | ports |
|---|---|---|---|
| N4 `release`, base tree, 3×60 | 9.37 | 9.37 | — |
| N4 `debug`, base tree, 3×30 | 9.26 | 9.18 | — |
| N4 both profiles, C6 tree | 9.15 | 9.15 | — |
| MF-A 4 MiB paired, 8 pairs | 9.14 | 9.14 | — |
| flake campaign, 800 sequential runs | 8.94 | 9.32 | TIME_WAIT **4 → 9** |
| MF-B ladder, 6 runs | 9.93 | 9.93 | — |
| gates | 9.19 | 12.12 | — |

**The campaign did not pressure the ports**: 4 sockets in `TIME_WAIT` before,
9 after, against the 32768–60999 range. That matters because C5's Flake B
figures came from a host at 31 786 `TIME_WAIT`, and nothing here is in that
condition.

No orphan processes at close: `ps -eo pid,ppid,etimes,args | grep -E
'[i]magery_net|[z]z_c6_probe'` → empty.

## 2. The blocking finding — the axis test's three sentences

`a_peer_still_sending_past_the_deadline_is_a_timeout` no longer calls itself
deterministic. What was there — *"Deterministic, with no window and no
retry"*, *"the peer **cannot** have handed the whole record to the kernel, no
matter how fast it is or how the two threads were scheduled"*, *"A slow host
makes the peer **less** far along, not more, so it cannot turn this test
red"* — is replaced, in the same place, by three sentences:

> **What this holds is narrower than a claim of determinism, and it is not
> deterministic.** The record declares twice as many bytes as this connection
> will absorb with nothing read — a number the test measures rather than
> guesses — so the peer cannot have handed the whole record to the kernel *at
> the moment the queue stopped growing*; but `recv_record`'s own prefix read
> frees receive window, and the peer is parked inside one `write_all` ready to
> use it, so the rest of the record can still arrive between that read and the
> read that has to fail. Run sequentially on this tree with no port pressure,
> at loadavg 8.9-9.3 on 32 cores, it failed **3 of 400 in `release` and 2 of
> 400 in `debug`** — all five the panic below, reporting `Ok(true)` with
> 5 225 268 bytes delivered against the 2 612 608 this connection absorbs and
> the 2 612 608 that were queued. **A slow host makes the peer *further*
> along, not less**: it widens the gap between the last `peek` and the one read
> that must fail, which is the window the peer refills — so host speed pushes
> this test red rather than green, and pinning it is `riders-76`, not this
> comment.

**The three things the brief asked for, each with its instrument.**

**(a) What it actually holds.** The measured premise is true *at one instant*:
`payload_len = 2 * absorption` and `record.len() > absorption`, so when the
queue stops growing the peer is necessarily still holding bytes. What does not
follow is that it stays true through the call. `recv_record` reads the
12-byte prefix first, and that read *consumes* — it is the one place in this
sequence that returns receive window to a peer parked inside `write_all`. The
test's last observation of the queue is a `peek`, which does not consume, so
the premise is measured before the only event that can invalidate it.

**(b) The rate.** Measured on the extracted candidate tree (`git archive
b482867`, which is C6's code — my edits are comments only), the test binary run
**sequentially**, one process per run, no concurrency and no port pressure:

| profile | failures | runs |
|---|---|---|
| `release` | **3** | 400 |
| `debug` | **2** | 400 |

**5 in 800 = 1 in 160**, at loadavg 8.94 → 9.32, TIME_WAIT 4 → 9. All five
carry the identical panic at `tests/imagery_net.rs:1106`:

```
a peer still sending past the deadline was allowed to finish the record:
recv_record returned Ok(true) with 5225268 bytes, but this connection absorbs
only 2612608 bytes unread and 2612608 were queued, so the rest arrived after
the deadline
```

5 225 268 = 2 × 2 612 608 + 52, i.e. the **whole** record was delivered: the
peer refilled the queue with a second connection's worth of bytes *during* the
call. This is the same message C5's report recorded verbatim and the same one
the review's 1 200-run campaign saw, so all three campaigns are looking at one
defect. **This is the first time it has been forced sequentially in `debug` in
a builder's hands** — C5 got 0/150 there and declined to choose between its two
flakes on exactly that gap; 2/400 closes it, on the gate's own profile and
concurrency, and it agrees with the coordinator's ruling that Flake A is the
gate failure.

**(c) The direction.** The old third sentence is the inverse of the truth and
the mechanism says why. Slowing the host does not slow the peer relative to the
test: the peer is blocked in the kernel, and what unblocks it is window, not
CPU. Everything between the last `peek` and the read that must fail —
`from_stream`, `Vec::new()`, the prefix read, the call into `take_arrived` —
is *test-and-caller* work, and it is what a slow host stretches. Stretching it
gives the peer more wall-clock time against a window that the prefix read has
just opened. So slower host → peer further along → more bytes delivered → the
`Ok(true)` panic. The failure message is itself the evidence for the direction:
it reports *more* bytes arriving than the connection can hold, which is only
possible if the peer ran on during the call.

**I did not fix it**, per the brief. The assertion, the helper, the ladder,
the thread structure and the panic text are byte-identical to `b482867`; only
the doc comment moved. `riders-76` is named in the comment so the next reader
lands on the slice that owns it.

## 3. MF-A — re-derived as paired ratios, and the old band withdrawn

**The shipped band was a ratio of cross-run column ranges.** The review's
hypothesis is right and I can confirm it from C5's own report rather than
guess: report §2.2's table gives 4 MiB base p50 `40-55` and C5 p50 `87-98`
across four runs and both disciplines, and 91/55 = **1.65**, 95/40 = **2.38**
— the shipped band to two decimals. A within-run pairing was never taken for
this cell (§3/MF1 does pair correctly, but for 31 MiB).

**Re-derived, paired.** Same instrument as the shipped sentence names — median
of 60 timed calls, 4 MiB declared, `release`, one process per cell, base
(`2c3041f`'s `src/imagery/net.rs` on this tree) and candidate measured
back-to-back **within** each run, four runs × both disciplines = eight pairs.
**Loadavg 9.14 before and 9.14 after, unchanged across the whole set.**

| run | discipline | base p50 | C6 p50 | paired ratio |
|---|---|---|---|---|
| 1 | fresh | 37 | 91 | **2.46×** |
| 1 | reuse | 38 | 93 | **2.45×** |
| 2 | fresh | 37 | 90 | **2.43×** |
| 2 | reuse | 37 | 90 | **2.43×** |
| 3 | fresh | 37 | 88 | **2.38×** |
| 3 | reuse | 37 | 93 | **2.51×** |
| 4 | fresh | 37 | 91 | **2.46×** |
| 4 | reuse | 37 | 93 | **2.51×** |

Band **2.38–2.51×** (µs, medians of 60). The review measured **2.35–2.59×** at
loadavg 8.11 on the same instrument; mine sits inside that, and **neither
touches 1.65×**. What ships is my band, labelled as paired, with the load, and
one sentence withdrawing the old one:

> at 4 MiB this branch is **2.38-2.51× slower on within-run paired medians of
> 60** — eight pairs, four runs × both disciplines, base 37-38 µs against
> 88-93 µs here, at loadavg 9.14 before and after. An earlier revision
> published 1.65-2.38× for this cell; that was a ratio of cross-run column
> ranges rather than of paired runs, and it is withdrawn.

The retraction is also in the commit message, as C5 did for the 4.2×. The
following sentence — *"Other instruments on this host have put the 4 MiB
figure as high as 3.3×"* — is C4's review's figure, was not re-measured here,
and is left as it stands.

## 4. MF-C / N4 — restored to what reproduces, with the load

**C4's "202 ms" is the figure that reproduces, and I reproduced it myself.**
Instrument: `2c3041f`'s `src/imagery/net.rs` on this tree, `MAX_RECORD_LEN`
(67 108 864) declared, **fresh** destination buffer, one process per run, the
same probe the review used.

| | statistic | shipped by C5 | measured here | load |
|---|---|---|---|---|
| `debug` | median of 30 | 227 ms (band 210.1–227.1) | **202.7 / 202.1 / 202.0 ms** | 9.26 → 9.18 |
| `debug` | max of 30 | 312.4 ms | **211.3 / 209.8 / 202.7 ms** | 9.26 → 9.18 |
| `release` | median of 60 | ~25.4 ms | **20.49 / 20.47 / 20.69 ms** | 9.37 → 9.37 |
| `release` | max of 60 | 28.6 ms | **27.0 / 24.4 / 23.4 ms** | 9.37 → 9.37 |

Spreads under 1% on the medians in both profiles. The review's independent
figures (201.5 / 202.1 / 201.6 and 20.49–20.66, load 9.1–9.3) land on top of
mine. **The C5 brief's N4 correction tracked host load, not the base** — C5
measured at loadavg 13–32 and said so in its report §8, and 227/25.4 is what
this cell reads at that load.

**What changed in the tree.** Three figures and their load:

- module rustdoc: *"~25 ms"* → *"**~20.5 ms** for the same call writing them,
  medians of 60 **at loadavg 9.1-9.4 on 32 cores**"*.
- the `64 MiB resize fresh` cells in the two tables: median `25422` →
  `20466-20688`; max `28618` → `23441-26976`.
- the `debug` line: `64 MiB 210.1-227.1 ms (312.4)` →
  `202.0-202.7 ms (202.7-211.3)`; and the prose span *"1.3-25.4 ms there in
  `release` and 210-227 ms in `debug`"* → *"1.3-20.7 ms … and 202 ms in
  `debug`"*.
- the cost test's doc comment: *"227 ms … and 25 ms"* → *"**202 ms** over 30
  calls in `debug` and **20.5 ms** over 60 calls in `release` … both at loadavg
  9.2-9.4 on 32 cores — that load is part of the figure, and the same cells
  read 227 ms and 25 ms on the same tree at loadavg 13-32."*

**Load now ships beside every figure in that comment, which is what neither
set of numbers did.** The tables carry a new heading:

> **Load belongs to each figure and moves it.** These tables were taken at
> loadavg 13-32 on these 32 cores; the `64 MiB resize fresh` cells and the
> 4 MiB paired ratio below were re-measured at loadavg 9.1-9.4 and are the
> figures given here. A figure quoted without its load is not reproducible from
> this text.

I did **not** re-measure the other cells (1 MiB, 8 MiB, 31 MiB, 32 MiB, the max
rows, the tail counts). They are C5's, they were taken at loadavg 13–32, and
they now say so rather than being silently mixed with figures taken at 9.

The candidate side of the same cell, for completeness and because I edited the
sentence that carries it: 64 MiB on C6's tree, fresh, p50 **8-9 µs** in
`release` (max 13–19) and **9-10 µs** in `debug` (max 14–33), loadavg 9.15
unchanged. The shipped *"~9 µs"* stands.

## 5. MF-B — the span-versus-margin comparison, corrected

**This is report-side only; no line of the tree carried it**, and C5's
conclusion (leave the assertion, fix the comment) is right and stays.

**What C5's §4 did wrong.** It wrote *"any budget that passes this tree and
catches the base sits in [16.7, 28.6] — a **1.7× total span**, worse than the
base-side 2.13× the review already flagged as thin"*. Those are two different
quantities. `[16.7, 28.6] ms` is a **span between two absolute times**, taken
from a 60-rep single-length probe; 2.13× is a **margin** — worst cell over
budget — from the 9-rep ladder the test actually runs. A span and a margin
cannot be ordered against each other, so *"worse than"* does not follow from
1.7 < 2.13, and the sentence should not have been written.

**Computed properly, in the test's own shape.** The shipped test's exact
ladder (5 lengths × 2 disciplines, 9 reps, one process, budget
`small * 50 + 10 ms`), three runs per tree, each cell's median **and** max,
`release`. **Loadavg 9.93 before and after.**

| run | C6 worst med | C6 worst max | base worst med | base worst max |
|---|---|---|---|---|
| 1 | 692 | 11 037 | 20 402 | 22 801 |
| 2 | 720 | 11 020 | 21 324 | 22 395 |
| 3 | 700 | 10 884 | 20 408 | 22 028 |

(µs. Budget from the median small cell **10 250**; from a consistently
max-based small cell **10 350** on C6, **10 400–10 450** on base.)

| statistic | base-side margin | candidate-side margin |
|---|---|---|
| median (what ships) | 20 402/10 250 = **1.99–2.08× over** | 10 250/692 = **14.2–14.8× under** |
| `max` | 22 801/10 400 = **2.11–2.19× over** | 10 350/11 037 = **0.94–0.95× (fails)** |

**Switching to `max` leaves the base side where it was and destroys the
candidate side.** The review put the same two numbers at 2.09× and 14.5× →
0.96×; mine reproduce them. That, and not the 1.7×, is the argument for leaving
the assertion alone: a max-based assertion does not buy discrimination it does
not already have, and it costs all of the candidate's headroom.

**And *"fails 3/3"* is a figure that needs its load, which is the finding of
this whole attempt applied to itself.** The review got **2 of 3** at loadavg
8.42 (10 148 µs fell under the bar); I get **3 of 3** at loadavg 9.93 (10 884,
11 020, 11 037, all over). Both are true. The candidate's worst max sits within
**7%** of the budget, so which side of it a run lands on is decided by the
load, and neither "3/3" nor "2/3" is a property of the tree. The honest
statement is the one now in the shipped comment — *"the worst single call at
31 MiB is above that budget on this host, on both trees"* — which I also
reproduce (C6 10.9–11.0 ms, base 22.0–22.8 ms, budget 10.25 ms).

## 6. The rule — kept, moved, and given the variable that broke it

**Moved from** forty lines into `recv_record`'s `match` arm in
`src/imagery/net.rs` **to** `README.md`, its own section
**`## Performance figures in this crate's docs`**, placed immediately after the
paragraph describing the CI suite — the part of the README a contributor
already reads before changing anything:

> Any timing figure published anywhere in this crate — rustdoc, implementation
> comments, this file, examples, commit messages — carries the statistic that
> produced it, the repetition count, and **the host load it was measured at**.
> Measure a worst case with `max` or do not publish it: a median cannot see
> one, and an earlier revision of the imagery receive path published 2.8 ms as
> a ceiling when it was a median of 15 and the worst call was 4-6× above it.
> Load is not a footnote: measured against one unchanged revision on one
> 32-core host, the cost of a `MAX_RECORD_LEN` declaration reads as a `debug`
> median of **202 ms at loadavg 9** and **227 ms at loadavg 13-32** — the same
> code, the same statistic, a 12% difference — so a figure without its load
> cannot be reproduced from the text that carries it and will read as wrong to
> whoever tries. No millisecond ceiling for a record's past-deadline allocation
> cost ships anywhere in this crate, and none should be added without `max` and
> a load.

Three properties I kept deliberately. Its **correct output is never nothing** —
a figure that complies carries three visible things, so compliance and omission
do not look alike. Its **example is this slice's own accident**: the 202-vs-227
sentence is not a hypothetical, it is the defect that produced MF-C, and it
gives the reader the magnitude (12%) that load is worth here. And the
**universal** now sits where universals belong, with the crate rather than
inside one function.

**What stayed at the allocation**, because a retraction belongs at the locus of
the thing retracted: the 2.8 ms retraction and the local *"no millisecond
ceiling for this call ships anywhere in this crate"*, now ending with a pointer
—

> The rule that struck it — measure a worst case with `max`, name the statistic
> and the rep count, and name the load — is in `README.md` under *Performance
> figures in this crate's docs*, where someone adding a figure elsewhere in
> this crate will meet it.

**Why README and not a new `CONTRIBUTING.md`.** The repo has no contributor
document, no `docs/` directory and no PR template; adding a file for one rule
makes a home nobody visits. `README.md` is `Cargo.toml`'s declared `readme`,
it is not `include_str!`-ed into rustdoc (so no doctest surface is added — the
doc gates confirm rc 0), and its last section is already the contributor-facing
CI description.

**Should it become a gate?** *Yes, and I think it can be a cheap one* — but it
is not in this attempt and I have not written one. The shape I would propose,
if you queue it: a grep-based check that every line in `src`, `tests`,
`examples`, `README.md` matching a time-unit pattern (`\d+(\.\d+)?\s*(ns|µs|us|ms|s)\b`)
appears within N lines of `loadavg`/`load` **and** of a statistic word
(`median|max|p50|mean`), failing the job otherwise, with an explicit
allow-list file for the retraction sentences that name withdrawn figures on
purpose. Two things make me want a reviewer on it before it lands: the
allow-list is a hole big enough to walk the whole rule through, and the
population question is not settled — commit messages are named by the rule but
are not greppable from a working tree, so the gate would enforce over less than
the rule covers. **Nothing in the tree violates the rule today**; the review
verified that over README, `examples/`, `docs/` and `codebase-audit-docs/`, and
my own change adds three compliant figures and removes none.

## 7. Gates — every one, with its numbers

Full detail and provenance in `gates.md`. Script
`/home/andrej/coord_scratch/riders67c6/gates.sh` (C5's, path-substituted),
per-gate outputs `g_*.txt`. **Two runs, both reported**: `gates.log` on tree
`d45a3a4e` (the first commit) at loadavg 9.19 → 12.12, and `gates2.log` on the
**shipped tree `e5fc608c`** at loadavg 8.70 → 8.83. Every row below is from
`gates2.log`, and every row is identical between the two runs — including all
32 per-target cells, which is what a one-sentence README change should do.

Toolchain `cargo 1.97.1 (c980f4866 2026-06-30)`, `rustc 1.97.1 (8bab26f4f
2026-07-14)`, LLVM 22.1.6, Ubuntu 24.4.0 (noble), glibc 2.39; MSRV rows on
`cargo 1.92.0 (344c4567c 2025-10-21)`.

fmt rc **0** · clippy all-features rc **0** with **0** warning-or-error lines ·
`cargo test --locked --all-features` **183 passed / 0 failed** over **16**
targets, every per-target cell identical to C5 and C4 (5 · 23 · 7 · 11 · 28 ·
6 · **24** · 12 · 11 · 6 · 4 · 6 · 9 · 17 · 4 · 10 doc-tests) · no-features
**99 / 0** over 16 · `build --features net` rc **0** · example rc **0** · doc
no-features rc **0** · doc all-features rc **0** · MSRV no-features **99 / 0** ·
MSRV all-features **183 / 0** · features matrix `imagery` **127/0**, `net`
**121/0**, `serde` **101/0**, `nalgebra` **101/0**, `imagery,net` **179/0**,
`serde,nalgebra` **103/0**, all six clippy rc 0 with 0 warnings · feature-docs
**16 of 16 rows check_rc=0 doc_rc=0** · `cargo-deny` 0.20.2 rc **0**
(*advisories ok, bans ok, licenses ok, sources ok*, with exactly the two
pre-existing `license-not-encountered` warnings at `deny.toml:35,36`) ·
`cargo-machete` 0.9.2 rc **0**, *"didn't find any unused dependencies"*.

**Clean extraction.** `git archive b03817b` → `.../extract`: **14** files in
`tests/`, **0** matching `zz_`, **no `docs/` directory in the archive at all**;
`src/imagery/net.rs`, `tests/imagery_net.rs` and `README.md` all `cmp`-identical
to the worktree's.

**BASE comparison.** The same tree with `src/imagery/net.rs` from `2c3041f`,
`--no-fail-fast`, `imagery_net` target: **21 passed / 3 failed**, the three
being exactly `a_peer_still_sending_past_the_deadline_is_a_timeout`,
`the_worst_declared_length_past_the_deadline_stays_within_a_fixed_budget` and
`a_record_too_long_for_the_receive_queue_is_not_completed_past_its_deadline` —
cell for cell what C5 and the review got.

**Ablations were not re-run.** A1 and A2 are ablations of the allocation path
and the one-read bound, and this candidate's code is **byte-identical to
`b482867`** under the lexer check in §0 — an ablation of identical code cannot
return a different answer, and the review re-ran both against that code and got
C4's cells. Re-running them would have measured the same bytes a third time.
Say so if you want them anyway.

## 8. Scope compliance

- `docs/work/` : **not committed.** This report and `gates.md` are untracked;
  the commit staged three explicit paths and `git status --porcelain
  --untracked-files=no` after it is empty.
- `codebase-audit-docs/` : not edited, not read this attempt.
- No frozen packet edited. `docs/work/riders-67/attempts/C5/` read only.
- **The allocation path and every test body are byte-identical to C5's.** No
  assertion, budget, helper, thread or test name moved. The only non-comment
  change in the repository is the new README section.
- No `git push`, nothing installed, no `Co-Authored-By`, commit message written
  to a file and committed with `git commit -F`, no `git commit -m`.
- No `cd` into another repository or worktree; `git -C` throughout.
- `CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c6/target` for the
  worktree, and separate `target_cand` / `target_base` for the measurement
  trees.

## 9. What I am unsure about

- **The 1-in-160 rate is a rate for this host at loadavg ~9, and I have shipped
  it in a doc comment without that caveat being as loud as it should be.** The
  comment says "at loadavg 8.9-9.3 on 32 cores", which is the rule I was
  sent to enforce, but three campaigns now give 1/160 (mine), 1/240 (the
  review's `--exact` figure) and 1.3% (C5's `release` sequential). Those are
  compatible with one another at these sample sizes, and they are also
  compatible with the rate being a strong function of a variable none of us has
  isolated. I would not want a reader to treat 1 in 160 as a constant.
- **I attributed the mechanism from the test's own text and the failure
  message, not from a trace.** Nothing here instruments the prefix read
  directly: I have not shown by measurement that *that* read is the one freeing
  the window rather than, say, the socket's receive buffer auto-tuning upward
  during the call. Both mechanisms predict the same panic and the same
  direction for host speed, and the second would not change a sentence I wrote,
  but "the prefix read frees receive window" is a reading of the code, not an
  observation.
- **The direction claim is argued, not measured.** I did not run the campaign
  under an artificial slowdown (`cpulimit`, `nice`, a busy-loop bath) to show
  the rate rising with load. The argument is sound and the inverse claim is
  plainly false, but "slower host → more failures" would be worth a datapoint
  and does not have one. It is also the kind of experiment that would want the
  host to itself, which this host is not.
- **The 64 MiB table cells are now a different load from the cells above them
  in the same table.** I chose to label that rather than re-measure the whole
  table at loadavg 9, because re-measuring every cell is a wider attempt than
  the one I was sent to do. If you would rather have one table at one load, it
  is about twenty minutes of runs and I have the harness.
- **`grep -c` on the `cargo-deny` output counted one
  `license-not-encountered` line, not two**, because `gates.sh` pipes deny
  through `tail -12`; I confirmed the two warnings by their locations
  (`deny.toml:35:6` and `36:6`) instead. The claim "exactly two" is inherited
  from C5 and re-confirmed by location, not by a count over untruncated output.
- **Whether the rule belongs in README at all is a judgement I made alone.** A
  `CONTRIBUTING.md` would be met by more contributors and read by fewer users;
  I chose the file that certainly exists over the file that would certainly be
  correct. If you disagree, moving it is a two-line change and I have no stake
  in the choice.
- **My "3/3" against the review's "2/3" is a difference of 7% against a
  budget**, and I have presented both as load-dependent rather than picking
  one. If a reviewer's host is quieter still, they may get 1/3 or 0/3, and the
  sentence in §5 should survive that — but the underlying fact (a max-based
  assertion has no margin on this candidate) is what I would defend, not the
  fraction.
