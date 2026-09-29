# riders-67 C8 — builder report

Repository `natnet`. Branch `slice/riders-67-C8`, worktree
`/home/andrej/ws_rs/natnet_wt/impl-riders-67-C8`.

**MF-1, N1, N2, N3 and the two report-side notes — plus two things the brief
could not have listed: a fifth ratio no list on this slice contained, and two
uses of the escape clause that my own strengthening of it broke, both found by
running the new instrument over the whole tree instead of over my diff. Four
commits, no code.**

## 0. Identity

| | |
|---|---|
| Base | `candidate(C7)` = `70d0a025419f3e0ea826c6a47509517e1ec501e4`, tree `c0be2b7583d69fc1f379d2bdf2514b706ae201b6` |
| Base resolved | by me at **2026-09-19T03:18:00+02:00**, with `git -C /home/andrej/ws_rs/natnet_wt/impl-riders-67-C8 rev-parse HEAD` and `rev-parse HEAD^{tree}`. Both match the C8 brief. Worktree clean at that moment: `status --short` printed only `?? docs/`. Branch `slice/riders-67-C8`, not `dev` |
| **`candidate(C8)` SHA** | **`652039bc7db3605f66657cb3986edf32d09dec26`** |
| **`candidate(C8)` tree** | **`e311a0f09b3fe9ed1aafc74c627776a7b40548e5`** |
| Parents | `652039b` → `0d3272b` (tree `8a5f4c45…`) → `7a94509` (tree `869bb7c5…`) → `8002192` (tree `476a9e16…`) → `70d0a02`. **Four commits on the base**, not a rebase of `dev`. The first does the brief's work; the second fixes two defects the first did to the text around its own insertions (§11); the third makes the tree obey the escape clause the first one strengthened (§10); the fourth reflows two paragraphs the second and third left ragged, which is the same defect the second commit named, so it is fixed rather than tolerated |
| Files changed | `README.md`, `src/imagery/net.rs`, `tests/imagery_net.rs` — **3 files, +55 −30** over `70d0a02` (the extra insertions and deletions over the earlier count are the fourth commit's rewrap, which moves words between lines) |
| Code changed | **none.** With whole-line comments (`^[[:space:]]*//`, which also catches `///`) removed, `src/imagery/net.rs` and `tests/imagery_net.rs` are byte-identical to `70d0a02`: **272** lines, md5 `d1d7269b55c3eb81edfcbc1d0abe5f97`, and **1113** lines, md5 `47f2a54feb5a41fe916a5bdee9ec11c1` — the same two md5s C7 published and the C7 review reproduced. Separately, `git diff 70d0a02 -U0 -- src tests` has **0** changed lines that are not comment lines. Both instruments reach-checked below |
| Commit message files | `/home/andrej/coord_scratch/riders67c8/msg.txt` (**3059** bytes), `msg2.txt`, `msg3.txt`, `msg4.txt`, all four committed with `git commit -F`. **No `git commit -m` was run in this attempt.** `git log -4 --format=%B \| grep -c Co-Authored-By` → **0** |
| Staging | `git add README.md src/imagery/net.rs tests/imagery_net.rs`, then three times `git add README.md src/imagery/net.rs` — explicit paths, no `-A`, no `-u`. `git status --porcelain --untracked-files=no` after each commit → **0 lines**; the only untracked entry is `docs/` |
| Pushed | **no.** No `git push` was run in this slice, to any remote |
| Amended | **no.** No commit in this attempt was amended |

**Reach checks on the two "no code" instruments**, because a general negative
with no reach check is a vacuous oracle:

- **Strip-and-compare.** Perturbing a single identifier in the candidate's
  comment-stripped `tests/imagery_net.rs` (`fn ` → `fN `, first occurrence)
  makes `cmp` report **DIFFER**. It is not comparing nothing.
- **Diff-line counter.** Fed `+    let x = 1;`, `+/// a doc line` and
  `+        // a code comment`, the filter `grep -vcE '^[+-][[:space:]]*(//|///)'`
  returns **1** — the code line, and only the code line.

## 1. Load

`nproc` = **32**, uptime 17 d. **Nothing in this attempt is a timing
measurement**; no figure I publish depends on the load. It is recorded because
the rule this slice is about says to record it.

| event | loadavg |
|---|---|
| session open / base resolved | **10.82** |
| ratio sweep (§3) | **10.89 → 10.91** |
| load-negative audits (§2, §4) | **10.86** |
| no-code instruments | **11.17** |
| gate run 4, on the shipped tree, open → close | **11.57 → 11.95** |
| BASE comparison on the shipped tree, open → close | **11.97 → 12.05** |
| escape-clause sweep and the 2.8 ms audit | **11.70 → 11.89** |
| ratio census re-run on the frozen tree (§8) | **11.94** |

## 2. MF-1 — the commit that published the rule now obeys it

`tests/imagery_net.rs` shipped *"a measured **1.79×** slowdown of the process
under test"* with none of the three fields its own `README.md` rule demands and
without the escape its own second commit added. **The review is right and the
grading is right**: the figure is true, so (c) does not fire; it is in the
population on the candidate's own terms, since the exception paragraph names
that file and `src/imagery/net.rs:426` is the in-tree precedent that a `×`
ratio is bound.

**What it says now**, sliced from the file:

> tested it ran against it: **0 failures in 1200 runs** pinned to a core
> contended by three busy spinners — a measured 1.79× slowdown of the
> process under test: 8.75 s of elapsed time against 4.90 s, 60 runs per
> arm, at a load `riders-67`'s C6 review records beside each campaign arm
> and not beside this ablation — against **2 in 1200** on an idle core
> interleaved with it, at loadavg 13.8-15.8; and **5 in 3200** unconstrained
> at loadavg 18.2-22.9 against the 5 in 800 above at loadavg 8.9-9.3. Those
> four counts and the 1.79× are that review's own measurements and were not
> re-measured here; the 3 of 400 and 2 of 400 above are this tree's.

**Statistic and rep count are recoverable and were recovered**, from the C6
review: *"60 runs pinned to core 0 took **4.90 s** with no spinners and
**8.75 s** with three — a 1.79× slowdown of the process under test."* The
statistic is total elapsed time per arm, the rep count is 60 runs per arm, and
8.75 / 4.90 = 1.7857, which is the shipped 1.79×. **The reviewer's cheapest
discharge said "60 runs per arm" and that is what the source supports**, so it
ships.

**The load is not recoverable, and the negative is audited rather than
asserted.** Instrument: `grep -icE 'loadavg|load average|/proc/loadavg|host
load'` over the C6 review, then reading every hit in place. The review carries
loads in exactly three kinds of place — a session bracket in its header
(*"loadavg 8.89 at open, 16.10 at close"*), the campaign table's per-arm
`load / TIME_WAIT` column, and the 202 ms block at loadavg 16.67 → 16.81.
**The ablation block carries none**, and the header's own promise is *"per-block
loads are given beside every number below"*, which for this block is not kept.
Attaching the slow arm's 13.8–15.8 to it would be inventing a load, which is
the defect this slice exists to repair. So the shipped words are *"at a load
`riders-67`'s C6 review records beside each campaign arm and not beside this
ablation"* — which is the escape, and which names the record it searched, as
§7's strengthened clause now requires of anyone using it — and which §10 then
had to enforce against the tree itself.

**Provenance, N4.** The same clause carries it: *"Those four counts and the
1.79× are that review's own measurements and were not re-measured here; the
3 of 400 and 2 of 400 above are this tree's."* That is the line the review
asked for — not because anything was misattributed (**all six numbers are
correct**, which the review checked) but because a doc comment does not
otherwise distinguish *measured here* from *measured by whoever reviewed this*,
and the rule that demands a figure's load has no principled reason to let it
omit its hand.

## 3. Why the sweep could not reach a ratio — the fourth closed list, inside the instrument built against closed lists

This is the part of the attempt worth a later reader's time, so it is stated
plainly rather than repaired quietly.

**The sweep C7 ran was the right idea and the wrong population.** C7 §5 says it:
*"I ran the open query instead of working from the review's list — every
time-unit figure in `src`, `tests`, `examples` and `README.md` by pattern, not
by enumeration, because a list of the hits I already believe in cannot return
the one I do not."* That reasoning is correct, and the instrument it produced
was built precisely to defeat the failure mode this slice had already met three
times. It then failed in exactly that mode.

**The mechanism.** *"Every time-unit figure"* is an open query over a **closed
notation**. A ratio is a measured timing figure — it is a quotient of two
timings, it moves with the load, and it is unreproducible without its fields in
exactly the way a millisecond is — but it carries **no time unit**. `1.79×`,
`3.3×`, `4-6×` and `1.65-2.38×` contain no `ms`, no `µs`, no `s`, no `ns`. A
pattern that reads *"a number next to a unit"* cannot return any of them, at any
width of the alternation. **Widening the list of units would never have helped;
the list that needed widening was the list of notations.**

**Four times now, and the fourth is the instructive one.** The first three were
lists of *sites*: the C6 review's enumeration of non-compliant lines was not a
census (C7 §4); the review's N4 list of configured figures was not the whole set
(C7 §5); C7's own *"exactly three figures already in the tree"* was a count it
had not taken (C7 §4). Each was repaired by replacing a list of sites with a
query. **This one is a closed list hiding inside a query** — and that is why it
survived a repair aimed at closed lists. The general form:

> A pattern is open over the values of whatever it varies and closed over
> everything it holds fixed. C7's pattern varied the number and held the
> *notation* fixed. Nothing about running it feels like enumerating, which is
> why it was not audited as an enumeration.

**What I did about it, and what it cost.** I enumerated the notations first and
published the list beside the result (§7): `×`, a bare `2x`, `%`, *twice* /
*fold* / *times faster* / *slower*, and a ratio written as a division of two
cells. That sweep over the same four paths returns **six occurrences of five
figures**, and it returned one — `1.65-2.38×` — that **no list on this slice
contained**: not the C7 review's own ratio sweep, which named only `3.3×`
(`grep` for `1.65` over `C7/review.md` returns **0**), and not the C8 brief
(**0**). §5 has it.

**And this sweep is closed too**, over whatever *it* holds fixed — its paths are
`README.md src tests examples`, so it cannot see `codebase-audit-docs/`, `docs/`
or a commit message, and its notations are the five I thought of. That is not a
rhetorical hedge: it is why the rule text now says how far its own exception's
closure reaches (§6) instead of saying *"that is the whole of the exception"*
flat. **A closure claim needs its instrument beside it or it is the same defect
with better manners.**

## 4. N1 — `3.3×` given its fields, and the paragraph that answered two ways

**The figure.** `src/imagery/net.rs` said *"Other instruments on this host have
put the 4 MiB figure as high as 3.3×, so read it as a band and not a number"* —
measured, no statistic, no rep count, no load, not the 31 MiB tail, **and a line
C7 reflowed**, so it is inside *"published … from the revision that added it
onward"* on the rule's own words. It now reads:

> so read it as a band and not a number: that end is `riders-67`'s C4
> review's, 166.3 µs against 50.8 µs — within-run paired medians of
> 21, four paired runs, at a load no file of that packet records.

**Provenance and fields, recovered rather than guessed.** C4's review: *"Four
paired runs at 4 MiB, reuse, release, median of 21 (base / C4 µs): 50.8/166.3,
47.9/151.1, 49.7/142.3, 52.2/140.5 — **2.7–3.3×**."* The four ratios recompute
to 3.27 / 3.15 / 2.86 / 2.69, so the band is right and `3.3×` is its top cell,
166.3 / 50.8. Those two cells ship, so the ratio is now reproducible from the
shipped text, which is what clause (d) asks of any published quantity.

**The load negative, audited.** `grep -icE 'loadavg|load average|/proc/loadavg|
host load'` over **all four files of the C4 packet** — `brief.md`, `gates.md`,
`report.md`, `review.md` — returns **0, 0, 0, 0**. A bare case-insensitive
`grep -i load` over `C4/review.md` also returns **0**: the string does not occur
in that file at all. **Reach check:** the identical pattern over the four files
of the C6 packet returns **1 / 4 / 26 / 8**. The instrument sees the string when
the string is there.

**The paragraph.** It gave a reader two opposite answers — *"It is not
retroactive"* exempting this line, *"anything else non-compliant is a defect"*
condemning it. Both halves were defensible and the charitable reading was
available, which is exactly what makes it a defect: a rule a reader can satisfy
two ways is not a rule. The last sentence now reads:

> That is the whole of the exception, and *whole* reaches exactly as far as
> the two sweeps that produced it — every time-unit figure and every ratio in
> `README.md`, `src/`, `tests/` and `examples/`; a non-compliant figure in a
> notation neither sweep reads is unaudited rather than exempt, and saying so
> is the difference between a closed list and a list that only looks closed.
> Anything else non-compliant that this revision or a later one publishes or
> re-measures is a defect; a figure older than this rule that no revision
> since has touched is covered by *not retroactive* above and not by this
> list.

**It answers one way now**: non-retroactivity governs figures no revision has
touched; the named exception governs the 31 MiB tail; everything a revision
publishes or re-measures is bound. Under that reading `3.3×` *was* a defect —
C7 reflowed it — which is why §4 fixes the figure instead of exempting it. **The
rule and the tree now agree about it rather than the rule having to be read
kindly.**

## 5. The fifth ratio — `1.65-2.38×`, which no list contained

`src/imagery/net.rs` published *"An earlier revision published 1.65-2.38× for
this cell; that was a ratio of cross-run column ranges rather than of paired
runs, and it is withdrawn"* — **in a line C7 itself published** (it is line 39
of C7's own added-lines list). It names a statistic of a kind, and carries **no
rep count and no load**. It is not in the C7 review's must-fix, not in its
notes, and not in the C8 brief.

**It is in the population, and the reason is the rule's own worked example.**
The rule's headline case is `2.8 ms` — a figure that exists in this crate *only
inside its own retraction*, and which C7 nonetheless gave a statistic, a rep
count and the escape. If a withdrawn figure were out of the population, the
rule's own worked example would be out of the population. It ships as:

> earlier revision published 1.65-2.38× for this cell; that was a
> ratio of cross-run column ranges — medians of 60, four paired runs,
> at the loadavg 13-32 those tables carry — rather than of paired
> runs, and it is withdrawn; a withdrawn figure is still a published
> one and carries its fields, or a reader cannot see what was wrong
> with it.

**Provenance**: commit `3c1ee64`'s message — *"the band that replaces it is
measured … **1.65-2.38× at 4 MiB on the median of 60 over four paired runs**"*.
The load is the one the tables immediately above the sentence carry in their own
heading (*"These tables were taken at loadavg 13-32"*), and the sentence points
at that heading rather than restating the number as if it were measured here.

## 6. N2 — `4-6×`, inside the rule's own worked example, at both its sites

**Both sites**, because C7's second self-found defect was a figure quoted at a
site its clause did not count, and I am not repeating it: `README.md` and
`src/imagery/net.rs`.

**Its fields are recoverable and its arithmetic checks.** Commit `3c1ee64`:
*"Re-measured with `max` over 60 timed calls per cell, four paired runs per
length, the worst single call at 31 MiB with a reused destination buffer is
10.3-16.7 ms in release: **4-6x** the figure that shipped."* The tree's own max
table gives the `here reuse` 31 MiB cell as **11721–16706 µs**, and
11.7 / 2.803 = 4.18, 16.7 / 2.803 = 5.96 — **exactly 4-6×**. So the number needs
no correction; it needed its operands.

**A disagreement I did not paper over.** That commit message says the cell is
*"10.3-16.7 ms"* while the C5 packet message and the table say **11.7–16.7 ms**;
`10.3` is the `here fresh` column's low cell (10274 µs), not `here reuse`. I
took the **table**, because it is in the tree and a reader can check it, and
because only the table's low end makes the shipped `4-6×` arithmetic come out.
The prose figure is one notation of the same cell and it is the wrong one.

`README.md` now reads *"the worst call was 4-6× above it — the 11.7-16.7 ms
31 MiB reused-buffer cell of the max table in `src/imagery/net.rs`, a max over
60 timed calls per cell across four paired runs at the loadavg 13-32 that table
carries, against a 2.8 ms median of 15 at a load neither `c5d1cce`, which
published the figure, nor `3c1ee64`, which retracted it, records"* — the last
clause from §10, one commit later — and `src/imagery/net.rs` carries the same
fields pointing at *"the max table above"*.
**The old parenthetical attached "at a load nobody recorded" to the 2.8 ms
denominator only; a ratio has two ends and the reader could not tell which end
the disclaimer covered.** Now both ends carry their own.

## 7. N3 — the escape clause requires the instrument, not the assertion

*"Cannot now be recovered"* was an unaudited negative, and a negative with no
named population is an oracle that agrees with whoever writes it. The clause now
reads:

> Where the load was not recorded and cannot now be recovered, the figure
> says so in the load's place **and names the record it searched**: a missing
> load is never silent, and *cannot now be recovered* is a negative, so it
> carries the population that was searched for it rather than the assertion
> alone — which is what lets a reader who holds that record refute it.

**It requires what C7 actually did** — C7 swept the C4 packet and two commit
messages, the review widened that to every blob on the evidence branch, and both
negatives held. **It also binds me in this attempt**, which is the test of
whether a clause is a rule or a decoration: the two escapes I used in §2 and §4
each name their record in the shipped text (*"`riders-67`'s C6 review … not
beside this ablation"*, *"no file of that packet records"*), and the audits are
above. **I did not cite `AGENTS.md` §7 in the README, though it is the stricter
statement of the same principle: `natnet` has no `AGENTS.md`**, so a pointer to
it from this repository's README would not resolve for a reader of this crate.

## 8. The ratio census — the population, the notations, and the rule that produced it

Instrument, over `README.md`, `src/`, `tests/`, `examples/` **at the shipped
tree `e311a0f0`** — re-run on the frozen candidate and not only on the worktree
I was editing — with the notations enumerated **before** the query rather than
after the result:

| notation | pattern | return |
|---|---|---|
| `×` sign | bare `×` | **16 lines.** 5 are `src/frame.rs`'s wire-layout table (`3 × f32` field counts); **2 are `×` as multiplication, not ratio** — *"four runs × both disciplines"* and *"20 reps × 3 sizes"*; the other 9 are the ratio figures below |
| bare `Nx` | `[0-9](\.[0-9]+)?x\b` | **1** — `0x{found:08x}`, a format string in `src/error.rs`. Not a ratio |
| percent | `[0-9]+[[:space:]]*(%\|percent)` | **1** — `README.md`'s `12%` |
| comparative words | `twice\|thrice\|N-fold\|N times faster\|faster\|slower` | **8 lines**, every one prose or `twice` as a count. No figure |
| a ratio written as a division of two cells | read by eye over every `×` hit's sentence | **0** beyond the above |

**The two multiplication hits are why the pattern is a bare `×` and nothing
cleverer.** A tighter pattern demanding *number–dash–number* before the `×`
would have dropped `1.79×` itself. **A pattern tuned until the noise goes away
is a closed list wearing a regex**, which is the shape of the defect this
attempt exists to repair, so I would rather read seven false hits.

**The measured comparative figures, all of them, and where each stands:**

| figure | site | status |
|---|---|---|
| `1.79×` | `tests/imagery_net.rs` | **MF-1** — fixed, §2 |
| `3.3×` | `src/imagery/net.rs` | **N1** — fixed, §4 |
| `4-6×` | `README.md` and `src/imagery/net.rs` | **N2** — fixed, §6, both sites |
| `1.65-2.38×` | `src/imagery/net.rs` | **unlisted by anyone** — fixed, §5 |
| `2.38-2.51×` | `src/imagery/net.rs` | **already compliant** — statistic (*within-run paired medians of 60*), rep count (*eight pairs, four runs × both disciplines*), load (*loadavg 9.14 before and after*), plus the raw cells. Untouched. It is the in-tree precedent the review cited |
| `12%` | `README.md` | derived in its own sentence from two figures that each carry all three fields there (`202 ms at loadavg 9` and `227 ms at loadavg 13-32`, medians of 30). Compliant by derivation; untouched |
| `50×` | `src/imagery/net.rs`, `tests/imagery_net.rs` | the cost test's own budget constant, quoted. **Configured, not measured** — exempt by the scope clause C7 added. Untouched |
| `1 ×`, `3 ×`, `3 + 4 + 1 ×` | `src/frame.rs` | field counts in a wire-layout table. Not timings at all |

**Every one of these ratios was in a line C7's own diff added.** Its added-lines
list carries **six ratio occurrences of five distinct figures** — `4-6×` twice,
`2.38-2.51×`, `1.65-2.38×`, `3.3×`, `1.79×` — and **four of the five carried
fewer than three fields**. That is the size of this finding: the revision that
published the rule published five ratios the rule binds, enumerated none of
them, and its report's sweep could not have returned one if it had tried.

## 9. What in the brief was wrong, and the two report-side corrections

### (a) N7 — C7's §8(d) attributed the phrase to the wrong document. Corrected.

C7 §8(d) wrote *"**The C7 brief §3 asks for "a load on 2.8 ms"** (taking the
review's *"cheapest discharge"* phrasing)"*. **The phrase is the C6 review's,
not the C7 brief's**: flattened, *"a load on 2.8 ms"* occurs **1×** in
`C6/review.md` (in its *"Cheapest discharge: three parentheticals (rep counts on
202/227, a load on 2.8 ms…)"*) and **0×** in `C7/brief.md`. The review's N7 is
right, and right about the mitigation too — C7's own parenthetical disclosed the
true source in the same sentence, so a reader could refute it from the document.
**The substance stands unchanged**: a brief (or a review) can specify the *form*
of a discharge that the evidence cannot supply, and that is what happened. Only
the attribution moves. **The correct sentence is: the C6 *review* asked for a
load on 2.8 ms; there was none to be had.**

### (b) N8 — C7's §8(c) rested on an example its own brief forecloses. Restated with support that is available.

C7 §8(c) argued that sizing a doc comment in sentences prices honesty above its
value, and supported it with *"read literally, the cheapest compliant edit was
to delete the direction claim altogether"*. **That support is not available**:
the C7 brief's acceptance required the clause to exist, so deletion fails
acceptance and was never a compliant edit. The review is right.

**The conclusion is right and here is support that the brief does not
foreclose.** Take the *same* paragraph, at the *same* scope, in two epistemic
states, both of which this slice actually produced — no counterfactual edit is
needed, because the experiment was run twice by accident and the two candidates
are the two arms:

- **`b03817b` (C6)**, the direction asserted flat: *"**A slow host makes the
  peer *further* along, not less**: … so host speed pushes this test red rather
  than green."*
- **`70d0a02` (C7)**, the same claim with its epistemic status attached: the
  hedge, the campaign that ran against it with its counts and loads, the Fisher
  parenthetical, and the instruction to take the direction as unknown.

Nothing was deleted in either and the scope is identical, so **whatever the
paragraph grew by is the price of the hedge and nothing else.** C7's report
counted the growth as **four sentences → eight**, and the C7 review accepted
that count (*"The eight-sentence paragraph is the predicted result"*).

**And here is the part that makes the argument better than C7's did.** I tried
to reproduce those two counts with my own splitter over the same two blobs and
got **6 and 10**, not 4 and 8. I have not chased the difference and I am not
publishing 6 and 10 as the right answer — the point is that **a sentence count
is notation-dependent**: two honest instruments over the same two blobs
disagree by two on each. A size expressed in a unit whose *measurement* is
ambiguous cannot be checked by either party, which is a second and independent
reason not to size in text units, on top of the review's mechanism that
*"hedging a claim is strictly longer than asserting it."* **The ratio is what
survives both instruments: 8/4 and 10/6 both say the honest paragraph is
substantially the longer one, and a sentence budget charges for exactly that.**

I also record the part of §8(c) that was never in dispute and is now settled by
this brief: the coordinator sized C8 **by scope** and said so, and the reviewer
told them to. **There is nothing left of my complaint and I withdraw the rest of
it.**

### (c) The brief's premise about the sweep is right, and it understates the reach of the consequence.

The brief says the sweep enumerated every time-unit figure and a ratio carries no
time unit. That is exactly right (§3). What it does not say — and could not,
since no list on the slice had the figure — is that **the same blind spot hid a
fifth ratio, `1.65-2.38×`, in C7's own added lines** (§5). The brief's diagnosis
is sound; its population is one figure short, for the same structural reason the
C7 review's N4 list was three figures short. **This is (a) in miniature for the
third consecutive attempt**, and it is not a complaint about the brief: a brief
that hands me a diagnosis rather than a list is what let me find it.

### (d) Nothing else. On everything load-bearing the C8 brief was right —

including the instruction I would have most liked to argue with, that the report
should say plainly why the sweep could not reach a ratio rather than repair it
quietly. Writing §3 is what produced the notation enumeration in §8, and the
notation enumeration is what returned the fifth ratio.

## 10. The clause I strengthened broke the tree that ships it — found and fixed before freezing

**This is the defect of C7 exactly, one revision later, and I nearly shipped
it.** §7's strengthened escape requires a figure whose load cannot be recovered
to *name the record it searched*. Before freezing I ran the obvious audit — the
one whose absence is what made MF-1 possible — over the whole tree rather than
over the lines I had touched: every occurrence of the word `load` in
`README.md`, `src/`, `tests/` and `examples/`, minus `payload`/`upload`/
`download`/`load(`/`loaded`. **39 lines.** Two of them use the escape and name
nothing, and **both are the rule's own worked example**:

- `README.md` — *"a 2.8 ms median of 15 **at a load nobody recorded**"*
- `src/imagery/net.rs` at the allocation — *"It was a median of 15 taken **at a
  load nobody recorded**"*

*"Nobody recorded"* is an unaudited universal negative naming no population —
the precise thing N3 is about. Both now read *"at a load neither `c5d1cce`,
which published the figure, nor `3c1ee64`, which retracted it, records"*: two
objects a reader of this repository can check for themselves.

**The audit behind those words, run by me on this attempt** rather than
inherited from C7's:

- `git log -1 --format=%B` over **`c5d1cce`** (published the figure) and
  **`3c1ee64`** (retracted it), pattern `loadavg|load average|/proc/loadavg`:
  **0 and 0**.
- Every blob on `evidence/riders-67`, **read flattened** so a claim wrapped
  across a line break is still found: **10** carry `2.8 ms`. The four that
  predate the load rule — C4's report and review, C5's brief, and C8's brief —
  carry **0** load strings each. The six later ones carry 1, 24, 7, 3, 26 and 4,
  and every one of those is this slice discussing the figure's *missing* load,
  not supplying it.
- `git log --all -S'2.8 ms'`: **14** commits, of which the two above are the
  only ones that publish or retract the figure itself; the rest are packets and
  briefs.
- **Reach check:** the same flattened blob loop returns the C6 review's
  *"209.5 ms at loadavg 16.67 → 16.81"*, so it does find a load beside a figure
  when there is one.

**Why this matters more than the fix.** C7's must-fix was *a rule broken by the
commit that published it*. A strengthened clause is a new rule, and it has its
own population, and that population is **the whole tree and not the diff**. The
sweep that catches it is the audit of the candidate, not of the base —
which is the one I would not have run if I had stopped when the brief's list was
discharged.

## 11. My own defects in this attempt

The first two are the second commit's, both found by reading the committed file
back rather than the diff:

- **The `4-6×` parenthetical pushed the following line to 100 characters** in a
  comment block whose every other line wraps at 79. Rewrapped. **No gate would
  ever have told me**: `rustfmt` does not wrap comments, so `cargo fmt --check`
  passes on it — I checked the file's own wrap width by measuring it (`awk
  length`) rather than assuming one.
- **The sentence I inserted stole a pronoun.** *"…and it is withdrawn. It is
  slower because it must allocate the new destination…"* — *"It"* meant *this
  branch*; with *"A withdrawn figure is still a published one"* between them it
  read as the withdrawn figure. The subject is named (*"This branch is
  slower"*) and my sentence is joined to the clause it comments on rather than
  standing between two sentences that belong together. **A comment edit can
  break text it does not touch**, which is the comment-side version of the
  reflow risk C7 disclosed.

A README rewrap rode along, from the same cause: the worked example's new clause
left a four-word line in the middle of a justified paragraph.

**And then I did it twice more, and the fourth commit is the ratchet.** §10's
two edits left the same artefact — a 47-character line inside `README.md`'s
worked example and a 43-character one in the comment at the allocation. I found
them by running the measurement rather than reading for it: `git diff -U1`,
every context and added line shorter than 65 characters printed. **Four of the
seven hits are paragraph-final lines, which is what a wrapped paragraph should
look like; the other two were mid-paragraph and are gone.** The instrument is
in the report because *"I reflowed it"* is not checkable and *"no mid-paragraph
line under 65 characters"* is.

## 12. Gates — every one, with its numbers

Full provenance in `gates.md`. Script
`/home/andrej/coord_scratch/riders67c8/gates.sh` — C7's, path-substituted, with
one line changed (`cargo-deny`'s fallback path, because C7's no longer exists).
`CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c8/target`.

**Four full runs, all reported, all agreeing.** `gates.log` spans the second
commit's edits and is **superseded**; `gates2.log` (`869bb7c5`) and `gates3.log`
(`8a5f4c45`) were each superseded by a later commit; **`gates4.log` ran start to
finish against the shipped tree `e311a0f0`** at loadavg **11.57 → 11.95**,
TIME_WAIT **4 → 1335**. Filtered of loads, timings and cargo chatter, runs 2, 3
and 4 are byte-identical to one another and run 1 differs by one
`Checking serde_test` line. Every figure below is run 4's and is identical in
all four, including all sixteen per-target cells.

Toolchain `cargo 1.97.1 (c980f4866 2026-06-30)`, `rustc 1.97.1 (8bab26f4f
2026-07-14)`, Ubuntu GLIBC 2.39-0ubuntu8.7; **MSRV rows on `cargo 1.92.0
(344c4567c 2025-10-21)`**, so an explicit `cargo +1.92` ran rather than the
declared-and-ignored shape `rust-toolchain.toml` would otherwise give.

fmt rc **0** · clippy all-features rc **0** with **0** warning-or-error lines ·
`cargo test --locked --all-features` **183 passed / 0 failed** over **16**
targets, per-target cells **5 · 23 · 7 · 11 · 28 · 6 · 24 · 12 · 11 · 6 · 4 ·
6 · 9 · 17 · 4 · 10** (the last is doc-tests), every cell identical to C7, C6,
C5 and C4 · no-features **99 / 0** over 16 · `build --features net` rc **0** ·
example rc **0** · doc no-features rc **0** · doc all-features rc **0** · MSRV
no-features **99 / 0** · MSRV all-features **183 / 0** · features matrix
`imagery` **127 / 0**, `net` **121 / 0**, `serde` **101 / 0**, `nalgebra`
**101 / 0**, `imagery,net` **179 / 0**, `serde,nalgebra` **103 / 0**, all six
clippy rc **0** with **0** warning-or-error lines · feature-docs **16 of 16**
rows `check_rc=0 doc_rc=0`, counted as 16 matching against 16 total so the count
is over the whole population · `cargo-deny` 0.20.2 rc **0**, counted
**untruncated** at warning-or-error **2** and `license-not-encountered` **2** ·
`cargo-machete` 0.9.2 rc **0**, *"Done!"*, no unused dependency.

**`cargo-deny` was located before anything was concluded from its absence.**
C7's fallback path `/home/andrej/coord_scratch/riders67/deny/bin/cargo-deny` no
longer exists; I enumerated install shapes — `command -v`, `~/.cargo/bin`,
`/usr/bin`, `/usr/local/bin`, and a `find` over `coord_scratch` — and it is the
`~/.cargo/bin` shape, version 0.20.2, the version the workflow pins.

**One gate step I did not run, stated rather than glossed:** the workflow's
`cargo-deny fetch db` advisory-database refresh. I checked against the database
already on this host, so an advisory published since it was fetched would not be
seen. **`Cargo.toml` and `Cargo.lock` are untouched by this candidate**, so the
check's input is identical to C7's, whose run the C7 review reproduced.

**Clean extraction.** `git archive 652039b` → `extract/`: **14** files in
`tests/`, **0** matching `zz_`, **no `docs/` directory in the archive at all**;
`README.md`, `src/imagery/net.rs` and `tests/imagery_net.rs` all `cmp`-identical
to the worktree's.

**BASE comparison**, `git archive 652039b` with `src/imagery/net.rs` replaced by
`git show 2c3041f:src/imagery/net.rs` (md5 `0adf5f4d2c6312228dd1697114e28d7a`,
C7's md5), `--all-features --no-fail-fast --test imagery_net`, loadavg **11.97 →
12.05**: **21 passed / 3 failed in 7.25 s**, against C7's 7.25 s and the C6
review's 7.24 s, the three being exactly the named tests. Run three times in
this attempt, on three trees, same cells and 7.25–7.26 s each time.

**Ablations not re-run.** A1 and A2 ablate the allocation path and the one-read
bound; this candidate's code is byte-identical to `70d0a02` and so to `b03817b`
and `b482867` (§0), and an ablation of identical code cannot return a different
answer. Say so if you want them anyway.

## 13. Scope compliance

- `docs/work/` : **not committed.** This report and `gates.md` are untracked;
  both commits staged explicit paths and `git status --porcelain
  --untracked-files=no` after each is empty.
- `codebase-audit-docs/` : **not edited, not read** this attempt.
- **No frozen packet edited.** `C4/`, `C5/`, `C6/`, `C7/` and `C8/brief.md` were
  read only, via `git -C …/evidence-riders-67 show evidence/riders-67:…`, which
  cannot write.
- **No code.** §0's two instruments, both reach-checked.
- No `git push`, nothing installed, **no `Co-Authored-By` trailer**, both commit
  messages written to files and committed with `git commit -F`, **no
  `git commit -m`**.
- No `cd` into another repository or worktree in a compound command; `git -C`
  and absolute paths (`/usr/bin/git`, `/usr/bin/grep`) throughout.
- `CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c8/target` for the gate
  runs.
- **No campaign was run.** The only executions in this attempt are the gate
  runs, the BASE comparison and the extraction check.

## 14. What I am unsure about

- **`loadavg 13-32`, attached to `4-6×` and to `1.65-2.38×`, is inherited and
  not measured by me.** It is the load the tables' own heading in
  `src/imagery/net.rs` gives, so both sentences point at that heading rather
  than asserting a fresh reading — but C7 §11 already flagged that number as
  *two inferences deep* (C5 report §8 → C6 → the heading) and **the weakest
  number in its diff**, and I have now attached it to two more figures. If you
  would rather those two ratios said *"at the load those tables carry"* with no
  number, that is a two-site deletion and I would take it. I did not re-measure
  and I was not to.
- **`4-6×` rests on a disagreement between two of C5's own notations and I
  chose one.** Commit `3c1ee64` calls the cell *"10.3-16.7 ms"*; the C5 packet
  message and the tree's max table both say **11.7–16.7 ms** (`here reuse`,
  11721–16706 µs). I took the table, because it is in the tree, a reader can
  check it, and only its low end makes `4-6×` come out (11.7/2.803 = 4.18). **If
  the commit message's `10.3` is the right cell, the shipped `4-6×` is really
  3.7–6.0× and rounds the wrong way**, and the right repair is a re-measurement
  rather than a parenthetical.
- **`1.65-2.38×` was outside the brief's named scope and I fixed it anyway.** My
  ground is that the brief's own diagnosis — a closed list inside an instrument
  — demands the whole of the sweep's return, and that this figure sits in a line
  C7's own diff added. A reviewer who reads the scope strictly would say I
  exceeded it. **I would rather be told I overreached than ship the sixth ratio
  the sweep found and say nothing.**
- **The commencement clause now rules on a contradiction the review only
  noted.** I made *not retroactive* govern figures no revision has touched, and
  the exception list govern the rest. That is a substantive choice: if the
  coordinator meant the other side — that the named exception is exhaustive over
  the whole tree — then the three grandfathered `tests/imagery_net.rs` lines
  become defects and my clause is wrong rather than merely different. **It is
  the judgement in this attempt I am least sure of**, and it is one clause to
  reverse.
- **My ratio sweep is itself closed over what it holds fixed.** Its paths are
  `README.md src tests examples`, so it cannot see `codebase-audit-docs/`,
  `docs/` or a commit message — and the rule binds commit messages, which
  `riders-77` owns. Its notations are the five I enumerated (`×`, bare `2x`,
  `%`, the *twice*/*fold*/*faster*/*slower* words, a division of two cells). **A
  sixth notation would be a sixth missed figure**, and the rule text now says so
  in its own words rather than leaving me to say it here.
- **I shipped an attribution clause for figures I did not reproduce**, and the
  same caveat C7 raised still applies: the comment now says the four counts and
  the `1.79×` are the C6 review's, which is more than C7 said and less than a
  reader gets from having the review. I was told not to run a campaign and did
  not.
- **The C6 review's fourth campaign arm is still not in the comment.** The brief
  says it is in `riders-78`'s board entry and not to re-litigate it, so I have
  not — but the shipped sentence *"The one campaign that has tested it ran
  against it"* is still stronger than a four-arm table one of whose arms points
  the other way. **It is recorded here so the gap is visible from the
  candidate's own packet.**
