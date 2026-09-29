# riders-67 C7 — builder report

Repository `natnet`. Branch `slice/riders-67-C7`, worktree
`/home/andrej/ws_rs/natnet_wt/impl-riders-67-C7`.

**One clause and three parentheticals, plus the two rule clauses §4 and §5 were
asked for. No code, no test body, no assertion, no figure measured.**

## 0. Identity

| | |
|---|---|
| Base | `candidate(C6)` = `b03817b271ba0c9761577bbdeeeeebe44efd675b`, tree `e5fc608c2d999ad5d4dfe32102eb12674c7ae48b` |
| Base resolved | by me at **2026-09-18T21:49:51+02:00**, re-confirmed at **2026-09-19T02:47:31+02:00** after the session rate limit reset, both times with `git -C /home/andrej/ws_rs/natnet_wt/impl-riders-67-C7 rev-parse HEAD` and `rev-parse HEAD^{tree}`. Both match the C7 brief. Worktree clean: `status --short --branch` printed `## slice/riders-67-C7` and nothing else. Not `dev` |
| **`candidate(C7)` SHA** | **`70d0a025419f3e0ea826c6a47509517e1ec501e4`** |
| **`candidate(C7)` tree** | **`c0be2b7583d69fc1f379d2bdf2514b706ae201b6`** |
| Parents | `70d0a02` → `3a61310` (tree `59bce514…`) → `b03817b`. **Two commits on the base**, not a rebase of `dev`. The second fixes two defects of the first, both found by reading my own shipped rule back against the tree rather than against the review's list — see §4 |
| Files changed | `README.md`, `src/imagery/net.rs`, `tests/imagery_net.rs` (3 files, **+72 −39** over the base). Part of the `net.rs` hunk is a reflow of the two comment paragraphs it edits, to the file's width — no word changed by the reflow |
| Code changed | **none.** With whole-line comments (`^[[:space:]]*//`) removed, `src/imagery/net.rs` and `tests/imagery_net.rs` are byte-identical to `b03817b` — **272** lines, md5 `d1d7269b55c3eb81edfcbc1d0abe5f97`, and **1113** lines, md5 `47f2a54feb5a41fe916a5bdee9ec11c1`. Separately, `git diff b03817b -U0 -- src tests` has **0** changed lines that are not comment lines. Both instruments reach-checked below |
| Commit message files | `/home/andrej/coord_scratch/riders67c7/msg.txt` (**3562** bytes) and `msg2.txt` (**1516** bytes), both committed with `git commit -F`. **No `git commit -m` was run.** `git log -2 --format=%B \| grep -c Co-Authored-By` → **0** |
| Staging | `git add README.md src/imagery/net.rs tests/imagery_net.rs`, then `git add README.md src/imagery/net.rs` — explicit paths, no `-A`. `git status --porcelain --untracked-files=no` after each commit → **0 lines**; the only untracked entry is `docs/` |
| Pushed | **no.** No `git push` was run in this slice, to any remote |
| Amended | yes, once: the second commit was amended twice before it was final, while it was unpushed and uncited. The first commit was never amended |
| Interrupted | **yes, once.** The session rate limit fired after the C6 packet was read and the corpus sweep had started; nothing had been committed and nothing was lost, because nothing had been written. On resume this report's skeleton was written **before** the first edit |

**Reach checks on the two "no code changed" instruments**, because a general
negative with no reach check is a vacuous oracle:

- **Strip-and-compare.** Perturbing a single identifier in the candidate's
  comment-stripped `tests/imagery_net.rs` (`fn ` → `fN `, first occurrence)
  makes `cmp` report a difference. It is not comparing nothing.
- **Diff-line counter.** Fed `+    let x = 1;`, `+/// a doc line` and
  `+        // a code comment`, the filter `grep -vE '^[+-][[:space:]]*//'`
  returns **1** — the code line, and only the code line.

## 1. Load

`nproc` = **32**. **Nothing in this attempt is a timing measurement**, so no
figure here depends on the load. It is recorded because the rule this slice is
about says to record it.

| event | loadavg | TIME_WAIT |
|---|---|---|
| session open | 10.92 | 4 |
| gate run on the shipped tree, open → close | 11.37 → 11.45 | 4 → **1338** |
| BASE comparison, open → close | 11.45 → 11.50 | — |

**The 1338 is the gate suite's own and it is not port pressure.** The socket
tests run sixteen times over (all-features, no-features, MSRV ×2, six feature
combinations) and each leaves sockets in `TIME_WAIT`. Against the 32768–60999
ephemeral range that is 4% of it, an order of magnitude below the 31 786 that
broke C5's Flake B figures, and it drains: the next gate run opened at **4**
again. Nothing here is contaminated by it — I record it because a builder who
starts a campaign straight after a gate run should know the host is not quiet.

## 2. The clause — the direction claim, hedged to what is known

MUST-FIX 2. What shipped on `b03817b`, bold and flat, one sentence after a
measured rate:

> **A slow host makes the peer *further* along, not less**: it widens the gap
> between the last `peek` and the one read that must fail, which is the window
> the peer refills — so host speed pushes this test red rather than green, and
> pinning it is `riders-76`, not this comment.

**The review's diagnosis is right and it is the one I could not make about
myself.** C5 was rejected for shipping the opposite direction as a flat
unmeasured assertion; C6 flipped the sign and left the epistemic status where
it was. My own C6 §9 said *"the direction claim is argued, not measured"* — in
the file that does not ship, while the file that does ship put the claim one
sentence after a rate that **was** measured, which is the worst available place
for it.

**What ships now**, `tests/imagery_net.rs`, the axis test's doc comment, sliced
from the file rather than retyped:

> `Ok(true)` with 5 225 268 bytes delivered against the 2 612 608 this
> connection absorbs and the 2 612 608 that were queued. **Which way host
> speed moves this is argued from the mechanism and not measured.** The
> argument is that a slow host leaves the peer *further* along, not less,
> because it widens the gap between the last `peek` and the one read that
> must fail, which is the window the peer refills. The one campaign that has
> tested it ran against it: **0 failures in 1200 runs** pinned to a core
> contended by three busy spinners — a measured 1.79× slowdown of the process
> under test — against **2 in 1200** on an idle core interleaved with it, at
> loadavg 13.8-15.8; and **5 in 3200** unconstrained at loadavg 18.2-22.9
> against the 5 in 800 above at loadavg 8.9-9.3. That is too small a
> difference to refute the argument (0 against 2 is a one-tailed Fisher
> p ≈ 0.27, and the spinners slow the peer thread as well as the test thread)
> and it is no support for it, so take the direction as unknown and the rate
> as a reading of one host at one load. Pinning this test is `riders-76`, not

**The numbers in it are the C6 review's, and I did not reproduce any of them.**
Per the brief I ran no campaign. They are published because a reader who meets
a hedge deserves the datapoint that produced it, and because under this slice's
own rule a claim about load that cites no load is the defect being repaired.
Each carries its rep count and its load: 0/1200 and 2/1200 at loadavg
13.8–15.8, 5/3200 at loadavg 18.2–22.9, against the 5/800 at loadavg 8.9–9.3
that the comment already carried.

**I do want the direction measured properly, and I am asking for it as its own
slice.** The review's own instrument is not the one that would settle it: three
spinners on the shared core slow the peer thread as well as the test thread, so
it moves both sides of the quantity in question. What is wanted is a slowdown
*asymmetric* between the two — the peer on an idle core with the test thread
alone contended, and the reverse — interleaved, with enough runs to resolve a
difference against a base rate near 1 in 160. That is a few thousand runs on a
host that is not shared. **Queue it and I will run it.**

**What I did not weaken.** 3/400 and 2/400 are mine, the review reproduced the
panic and its three numbers, and they stay. What changed is that the comment
now says the rate is a reading of one host at one load — which C6 also said
only in the file that does not ship.

## 3. The parentheticals — the rule's three own figures, `:420`, `:441`

MUST-FIX 1, tree side. The rule demands a statistic, a repetition count and a
load, and each of its three own worked figures omitted exactly one.

| figure | had | was missing | what it now says |
|---|---|---|---|
| `2.8 ms` | median, 15 | **load** | *"at a load nobody recorded, which is the other half of why that figure cannot be reproduced"* |
| `202 ms at loadavg 9` | median, load | **rep count** | *"medians of 30 in both cases"* |
| `227 ms at loadavg 13-32` | median, load | **rep count** | the same clause |

**The 2.8 ms load does not exist and could not be added.** This is the one part
of the must-fix that could not be discharged in the form the review's *"cheapest
discharge"* proposed (*"a load on 2.8 ms"*). I looked for it: **no `loadavg`
string appears anywhere in the C4 packet** — report, review, brief or gates,
which is where that campaign is written down — nor in the commit that shipped
the figure (`c5d1cce`) nor in the one that retracted it (`3c1ee64`, whose
message says the tail would belong to *"this host and this machine's load"* and
gives no number). **Reach check on that negative:** the identical
`grep -rl loadavg` against the **C6** packet returns **3** files. The instrument
sees the string when the string is there.

So the honest parenthetical is that the load was never recorded — and it is the
better worked example. The rule's first figure is now one that is *permanently*
unreproducible **because** of the omission the rule forbids, which argues for
the rule harder than any number would have.

**`src/imagery/net.rs:445`** (`:441` on the base), the same retraction at the
allocation:

> this crate.** An earlier revision of these files published 2.8 ms as
> the worst a declared length could cost. It was a median of 15 taken
> at a load nobody recorded, the worst call is 4-6× above it, and a
> median cannot see a worst case. The rule that struck it — measure a
> worst case with `max`, name the statistic and the rep count, and
> name the load — is in `README.md` under *Performance figures in this

**`src/imagery/net.rs:420`, the line C6 itself rewrote.** *"1.3-20.7 ms there
in `release`"* is not a range: `1.3` is the 64 MiB `resize reuse` cell
(1299 µs, loadavg 13–32) and `20.7` is the 64 MiB `resize fresh` cell
(20688 µs, loadavg 9.1–9.4), the one cell in that column C6 re-measured. **One
range, two cells, two loads, and the range carried neither load nor statistic.**
I checked both ends against the tables twenty lines above the sentence rather
than taking them from the review's prose. What ships:

> What the tables say, and all they say. Above the threshold the
> declared length stops being bought at all — 64 MiB costs 9-14 µs
> here (medians of 60 at loadavg 13-32) against 1.3 ms there with a
> reused buffer (median of 60 at loadavg 13-32) and 20.7 ms with a
> fresh one (median of 60 at loadavg 9.1-9.4) in `release`, and 202 ms
> in `debug` (median of 30 at loadavg 9.2-9.4). Those two `release`
> cells are two loads and not one range, which is why they are given
> apart. That is the whole of the claim. Below it there is no saving:
> at 4 MiB this branch is **2.38-2.51× slower on within-run paired

The sentence no longer contains a range, because the two numbers were never
one. It gives two cells, each with its statistic and its load, and says why.

## 4. The `tests/imagery_net.rs` three — decided: the rule declares its own commencement

The review named three pre-existing, untouched lines in the cost test's doc
comment — `:1165`, `:1172`, `:1182` on `b03817b`, which are `:1175`, `:1182`,
`:1192` on the C7 tree, because my edit sits ten lines above them. (That drift
is why the rule now cites them **by their text**, not by line.) C7 could fix
them or the rule could declare its commencement. **I chose commencement and did
not touch the three lines.** Three reasons, in the order that decided it:

1. **Their loads cannot be recovered, only invented.** `:1182`'s campaign
   recorded *concurrency* — 24 in 240 at twenty-four-way, 0 in 400 at eight-way
   or less — where a loadavg would go, and nothing in the corpus carries a
   loadavg for it. For `:1165` and `:1172` I could have argued my way to
   loadavg 13–32, since those cells look like the ones in `net.rs`'s tables and
   the tables carry that load — but the tables carry it because **C6**
   attributed it to them out of C5's report §8, so attaching the same number to
   another file's prose is a second inference stacked on the first. Publishing
   an inferred load as a measured one is exactly what this rule exists to stop.
   **A rule against inventing loads cannot be discharged by inventing three.**
2. **Commencement covers figures nobody enumerated; a line-by-line fix covers
   only the list.** §5's open sweep returned figures the review's list did not
   have — and §4's own second defect, below, is a figure *the review did not
   flag* that my first clause then mis-counted. A rule with a named, closed
   exception is auditable; a rule hand-patched at three sites is exactly as
   violated as before anywhere nobody looked.
3. **It is inside the size I was given and touching the three is not.** They
   are in a doc comment the review verified clean, on a test this slice does
   not own.

**The clause was wrong twice before it was right, and both defects were mine.**
I wrote it, committed it (`3a61310`), then read it back against the tree instead
of against the review's list, which is how both were found:

- It demanded a load, named the exception set, and **left `2.8 ms` complying
  with neither** — in the tree twice, load never recorded, and not old enough to
  be grandfathered since the rule's own paragraph carries it. The rule now says
  a figure whose load cannot be recovered **says so in the load's place**, which
  is what both copies already do. That restores the property the rule exists for:
  *a missing load is never silent*, so compliance and omission do not look alike.
- It said *"exactly three figures already in the tree"*, **a census I had not
  taken**. The same two 31 MiB tail figures are also quoted at the allocation in
  `src/imagery/net.rs` (`10-17 ms` and `9-12 of 60 calls over 10 ms`), outside
  the *"These tables"* heading that gives the tables their load — the review
  flagged `:441` in that paragraph on exactly that reasoning and did not flag
  these two. The clause now names the **figures** and both places they are
  quoted, instead of counting occurrences.

**The cost of the choice, stated.** A commencement clause is a rule admitting it
does not hold over its own tree, and that is weaker than a rule that does. I
bought it down by making the exception closed, named and text-addressed. If you
would rather the three figures were struck than grandfathered, striking them is
a three-line deletion and I have no stake in it — **but I will not attach loads
to them.**

## 5. The scope clause — and the review's N4 list was not the whole set

N4. Read literally the rule caught figures that cannot comply because they are
not measurements. What ships, in the rule's first paragraph:

> A configured figure is not a measured one and carries none of the three: a
> timeout default, a test's deadline, a drip interval and a protocol constant
> are choices, not readings.

**I ran the open query instead of working from the review's list** — every
time-unit figure in `src`, `tests`, `examples` and `README.md` by pattern, not
by enumeration, because a list of the hits I already believe in cannot return
the one I do not. **34 hits.** The review's N4 named four kinds; the sweep
returned three the list did not have:

| figure | where | why it cannot comply |
|---|---|---|
| `5-second connect and 30-second record defaults` | `README.md` | configured — in N4 |
| `1 ns` / `1 µs` deadlines | `tests/imagery_net.rs:716, 831, 836, 838, 914` | configured — in N4 |
| `40 ms per byte` drip | `tests/imagery_net.rs:1009` | configured — in N4 |
| `connect timeout 5000 ms, record timeout 30000 ms` | `examples/imagery_dump.rs:216` | configured — in N4 |
| **`with_clock_offset_secs(0.030)` — `30 ms, the 3.5 example`** | **`src/generate.rs:263`** | **a protocol constant out of the spec — *not* in N4** |
| **`the true timeline (1.0 s)`** | **`src/generate.rs:271`** | **a protocol constant — *not* in N4** |
| **`the 64 KiB cell plus 10 ms`** | **`tests/imagery_net.rs:1180`** | **the asserted budget's own constant, quoted — *not* in N4** |

That is why the clause names *"a protocol constant"* as well as N4's three
kinds: two of the figures it must exempt are in a file N4 never reached.

**The rule as it now reads, in full** (sliced from `README.md`):

> Any *measured* timing figure published anywhere in this crate — rustdoc,
> implementation comments, this file, examples, commit messages — carries the
> statistic that produced it, the repetition count, and **the host load it was
> measured at**. Where the load was not recorded and cannot now be recovered,
> the figure says so in the load's place: a missing load is never silent. A
> configured figure is not a measured one and carries none of the three: a
> timeout default, a test's deadline, a drip interval and a protocol constant
> are choices, not readings. Measure a worst case with `max` or do not publish
> it: a median cannot see one, and an earlier revision of the imagery receive
> path published 2.8 ms as a ceiling when it was a median of 15 and the worst
> call was 4-6× above it (at a load nobody recorded, which is the other half of
> why that figure cannot be reproduced). Load is not a footnote: measured
> against one unchanged revision on one 32-core host, the cost of a
> `MAX_RECORD_LEN` declaration reads as a `debug` median of **202 ms at loadavg
> 9** and **227 ms at loadavg 13-32** — medians of 30 in both cases, the same
> code, the same statistic, a 12% difference — so a figure without its load
> cannot be reproduced from the text that carries it and will read as wrong to
> whoever tries. No millisecond ceiling for a record's past-deadline allocation
> cost ships anywhere in this crate, and none should be added without `max` and
> a load.
>
> This rule binds every figure published or re-measured from the revision that
> added it onward. It is not retroactive, because the loads of the older
> campaigns were mostly not recorded and cannot be recovered, and inventing them
> is the thing this rule exists to prevent. Once the *measured*/*configured*
> line above is drawn, one set of figures already in the tree still falls short
> of it: the 31 MiB tail — `9-12 of 60 calls over 10 ms`, the `10-17 ms` worst
> call, and the `24 times in 240 runs` line, which records concurrency in place
> of a load — wherever the cost test's doc comment in `tests/imagery_net.rs` and
> the comment at the allocation in `src/imagery/net.rs` quote it. That is the
> whole of the exception; anything else non-compliant is a defect.

## 6. The compliance sentence, replaced with what was actually verified

C6's report §6 said: *"**Nothing in the tree violates the rule today**; the
review verified that over README, `examples/`, `docs/` and
`codebase-audit-docs/`."* **That sentence is withdrawn.** What it cited was the
C5 review's *"Nothing in the tree violates it today, verified over the whole
tree"* — a verification of the rule **as it then read**, i.e. the rule *without*
the load requirement, which is C6's entire edit to it. The same C5 review
sentence continues two sentences later with *"And it **omits load**, which is
what actually breaks its own figures"*. So the verification I transplanted was
explicitly bounded by the very gap I then filled, and I carried it across that
boundary anyway. **It was a false conclusion in the report, and the tree it
described was violating the rule in eight places.**

What has actually been verified, on what, and by whom:

- **The C5 review** verified the *pre-load* rule over the whole tree. That
  result stands, and it covers the `max` / statistic / rep-count clauses only.
- **The C6 review** ran the *post-load* rule over the candidate tree and found
  eight violations: the rule's own three figures, `net.rs:420` and `:441`, and
  three lines in `tests/imagery_net.rs`.
- **C7 (me)** fixed the first five; exempted the last three by a named
  commencement clause; and re-ran the population sweep myself over `src`,
  `tests`, `examples` and `README.md`, which reproduced the review's eight, added
  three configured figures the scope clause now excludes (§5), and added two
  quotations of the exempted figures at the allocation that the review had not
  flagged (§4).
- **Nobody has verified the rule over `codebase-audit-docs/` or over commit
  messages on this candidate.** C6's sentence claimed `codebase-audit-docs/`
  coverage by transplant from the C5 review; I did not re-run it there — it is
  outside this attempt's edit scope — and I claim nothing about it. Commit
  messages are named by the rule and are not greppable from a working tree; the
  gap I flagged in C6 §6 is still open and `riders-77` owns it.

## 7. N1 and N2, corrected

- **N1.** C6's report §2(b) wrote *"the extracted candidate tree (`git archive
  b482867` …)"*. **`b482867` is the base**, `candidate(C5)`, not the candidate.
  The sentence self-corrected inside its own parenthetical and byte-identical
  code carries the result, so no figure moves — but the description was wrong,
  and it is the kind of wrong arithmetic cannot catch. **Correctly: the
  5-in-800 campaign ran on `b482867`'s tree, whose code is byte-identical to
  `candidate(C6)`'s and to `candidate(C7)`'s.**
- **N2.** C6's report §2(b) put the panic at `tests/imagery_net.rs:1106`. The
  `panic!` line is **1106** on `b482867`, **1113** on `candidate(C6)`, and
  **1123 on `candidate(C7)`** (its message string starting one line below each),
  because this attempt adds ten comment lines above it. The figure is a line
  number in a file that grows whenever anyone writes a sentence, which is the
  argument for citing it by test name: the panic is in
  `a_peer_still_sending_past_the_deadline_is_a_timeout`, and that does not move.

## 8. What in the brief was wrong

`AGENTS.md` §7 item 2: *"**What in the brief was wrong**, with quoted evidence
from the source. The most valuable section"*. C6 had the substance without the
heading; it is restated first, under the heading.

**(a) The C5 brief's N4 correction was tracking host load, not the base.**
(C6 §4, restated.) The brief directed C5 to correct 202 ms to 227 ms as though
the earlier figure were wrong about the tree. It was not: the same cell on the
same unchanged code reads 202 ms at loadavg 9 and 227 ms at loadavg 13–32. C6
measured 202.0–202.7; the C6 review measured 201.5–202.1 independently, then
204.8–209.5 at loadavg 16.7, and the three readings are monotone in load. **A
brief that corrects a number without its load is committing the defect the
slice is about.**

**(b) My C6 brief contradicted itself about its own size and I adopted half of
it silently.** Its header said *"four sentences and three numbers"*; its §2 said
*"Discharge is three sentences"*. My report's epigraph said *"Four sentences and
three numbers"*: I took the header's count, did not say the two disagreed, and
so converted a brief's internal inconsistency into a report's unsourced
assertion. (For the record the shipped C6 comment was four sentences, so the
header was the operative count — which is not the point. The point is that I
chose between two instructions without saying I was choosing.)

**(c) A sentence count is the wrong instrument for sizing a doc comment, and
this attempt shows why.** The review and the C7 brief both sized this as *"one
clause and three parentheticals"*, and I have kept inside that in substance —
one claim hedged, three figures given their missing field, plus the two clauses
§§4–5 were explicitly asked for. But the axis comment's paragraph now runs to
**eight sentences** where C6's ran to four, because hedging a claim honestly
costs more words than asserting it. **A size expressed in sentences prices
honesty above its value**: read literally, the cheapest compliant edit was to
delete the direction claim altogether, which would have hidden the disagreement
rather than recorded it. I read the sizing as being about scope, and say so here
rather than quietly exceed it.

**(d) The C7 brief §3 asks for "a load on 2.8 ms"** (taking the review's
*"cheapest discharge"* phrasing). **There is no load on 2.8 ms to be had** —
§3. The fix had to be a statement that it was never recorded, not a number. **A
brief can specify the *form* of a discharge that the evidence cannot supply.**

**(e) The review's N4 list, which the C7 brief §4 repeats, is not the whole
set** — §5 finds two more figures in `src/generate.rs` and one more in
`tests/imagery_net.rs`. The scope clause had to be written against the sweep,
not against the list. **This is (a) in miniature and it is as much mine as
anyone's**: I would have written the clause to fit the four examples I was
handed if I had not run the query.

**(f) The review's list of non-compliant figures is also not the whole set** —
§4 — and my first commit inherited the gap by turning that list into a count.
Neither the review nor the brief is at fault for the omission; the fault is
mine, for treating a reviewer's enumeration as a census.

**(g) Nothing else. On everything load-bearing the C7 brief was right**,
including the part that was least comfortable: the direction claim was the
defect C5 was rejected for, wearing the opposite sign, and I did not see it
until the review said so.

## 9. Gates — every one, with its numbers

Full detail and provenance in `gates.md`. Script
`/home/andrej/coord_scratch/riders67c7/gates.sh` — C6's, path-substituted, with
one instrument repaired (below) and `/proc/loadavg` plus the `TIME_WAIT` count
taken at open and close. Per-gate outputs `g_*.txt`.

**Three full runs, all reported, all identical.** `gates.log` on tree
`59bce514` (the first commit), `gates2.log` on an intermediate tree, and
**`gates4.log` on the shipped tree `c0be2b7`** at loadavg **11.37 → 11.45**,
TIME_WAIT **4 → 1338**. Every figure below is from `gates4.log` and every one
of them is identical in all three runs, including all sixteen per-target cells.
(`gates3.log` was started against a tree I then amended and is superseded by
`gates4.log`; it is in the scratch directory and agrees with it.)

Toolchain `cargo 1.97.1 (c980f4866 2026-06-30)`, `rustc 1.97.1`, Ubuntu 24.4.0
(noble), glibc 2.39; MSRV rows on `cargo 1.92.0 (344c4567c 2025-10-21)`.

fmt rc **0** · clippy all-features rc **0** with **0** warning-or-error lines ·
`cargo test --locked --all-features` **183 passed / 0 failed** over **16**
targets, per-target cells **5 · 23 · 7 · 11 · 28 · 6 · 24 · 12 · 11 · 6 · 4 ·
6 · 9 · 17 · 4 · 10** (the last is doc-tests), every cell identical to C6, C5
and C4 · no-features **99 / 0** over 16 · `build --features net` rc **0** ·
example rc **0** · doc no-features rc **0** · doc all-features rc **0** · MSRV
no-features **99 / 0** · MSRV all-features **183 / 0** · features matrix
`imagery` **127 / 0**, `net` **121 / 0**, `serde` **101 / 0**, `nalgebra`
**101 / 0**, `imagery,net` **179 / 0**, `serde,nalgebra` **103 / 0**, all six
clippy rc 0 with **0** warning-or-error lines · feature-docs **16 of 16** rows
`check_rc=0 doc_rc=0` · `cargo-deny` 0.20.2 rc **0** · `cargo-machete` 0.9.2
rc **0**, *"Done!"*, no unused dependency reported.

**The `tail -12` weakness I disclosed in C6 §9 is fixed in the instrument, not
in the write-up.** `gates.sh` now writes `cargo-deny`'s output whole to
`g_deny.txt` and counts over the untruncated file: **warning-or-error lines =
2**, **`license-not-encountered` = 2** — the two pre-existing `deny.toml:35,36`
entries. *"Exactly two"* is a count over the whole output, which is what it
should always have been; the C6 review reached the same 2 independently.

**Clean extraction.** `git archive 70d0a02` → `extract/`: **14** files in
`tests/`, **0** matching `zz_`, **no `docs/` directory in the archive at all**;
`README.md`, `src/imagery/net.rs` and `tests/imagery_net.rs` all `cmp`-identical
to the worktree's.

**BASE comparison, re-run on the shipped tree.** `git archive 70d0a02` with
`src/imagery/net.rs` replaced by `git show 2c3041f:src/imagery/net.rs` (md5
`0adf5f4d2c6312228dd1697114e28d7a`), `--all-features --no-fail-fast`,
`imagery_net` target, loadavg **11.45 → 11.50**: **21 passed / 3 failed**, the
three being exactly `a_peer_still_sending_past_the_deadline_is_a_timeout`,
`a_record_too_long_for_the_receive_queue_is_not_completed_past_its_deadline`
and `the_worst_declared_length_past_the_deadline_stays_within_a_fixed_budget` —
cell for cell what C6, C5 and the C6 review got.

**Ablations were not re-run.** A1 and A2 ablate the allocation path and the
one-read bound, and this candidate's code is byte-identical to `b03817b` and so
to `b482867` (§0). An ablation of identical code cannot return a different
answer; the C6 review re-ran both against that code and got C4's cells; nothing
in this attempt is executable. Say so if you want them anyway.

## 10. Scope compliance

- `docs/work/` : **not committed.** This report and `gates.md` are untracked;
  the two commits staged explicit paths and `git status --porcelain
  --untracked-files=no` after each is empty.
- `codebase-audit-docs/` : **not edited, not read** this attempt.
- **No frozen packet edited.** `docs/work/riders-67/attempts/C6/` and
  `C7/brief.md` read only, from the `evidence-riders-67` worktree.
- **No code.** §0's two instruments, both reach-checked.
- No `git push`, nothing installed, **no `Co-Authored-By` trailer**, both commit
  messages written to files and committed with `git commit -F`, **no `git commit
  -m`**.
- No `cd` into another repository or worktree in a compound command; `git -C`
  and absolute paths throughout, `/usr/bin/git` and `/usr/bin/grep`.
- `CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c7/target` for the gate
  runs, `target_base` for the BASE comparison.
- **No campaign was run to rescue the direction claim**, per the brief. The only
  executions in this attempt are the gate runs, the BASE comparison and the
  extraction check.

## 11. What I am unsure about

- **I am shipping a reviewer's measurements in a doc comment without having
  reproduced any of them.** The 0/1200, 2/1200, 5/3200 and the 1.79× ablation
  are the C6 review's, and I was told not to run a campaign. I believe them —
  the ablation makes the instrument non-vacuous and the review states its own
  limits — but a doc comment does not distinguish *measured here* from
  *measured by whoever reviewed this*, and a later reader will not know that
  four of the six numbers in that paragraph come from a different hand than the
  other two. If you would rather the comment attribute them, that is one more
  clause and I will add it.
- **The commencement clause is the judgement I am least sure of.** It makes the
  rule admit it does not hold over its own tree. I think a named, closed
  exception beats three invented loads and I would make the call again — but a
  reviewer who holds that a rule must bind everywhere the moment it lands would
  strike the three figures instead, and would not be wrong.
- **The `2.8 ms` load negative rests on the C4 packet plus two commit
  messages.** The instrument is reach-checked and not vacuous, but the
  population is *"where that campaign is written down as far as I know it"*. If
  a load survives somewhere I did not look — that attempt's scratch directory,
  say — then *"at a load nobody recorded"* is too strong and should read *"at a
  load this crate's record does not carry"*. One-word fix; I would take it.
- **`loadavg 13-32` as the load of `net.rs`'s tables is inherited, not measured
  by me.** My `:420` parenthetical gives `1.3 ms` that load on C6's attribution
  of it to those tables, which came in turn from C5's report §8. The `20.7 ms`
  and `202 ms` ends are C6's own readings at loads C6 took itself. So one of the
  three loads in that sentence is two inferences deep. I published it anyway
  because the alternative was to leave the cell loadless — **it is the weakest
  number in my diff**, and it is the reason §4 refuses to do the same thing to
  the `tests/` three, where the inference would be a third step.
- **The rule still covers commit messages and no instrument can check them from
  a working tree.** Unchanged from C6 §6, and the scope clause does not help:
  commit messages carry measured figures, so they are in the population.
  `riders-77` will have to decide whether the rule shrinks to what a gate can
  see or the gate reaches into `git log`.
- **I reflowed two comment paragraphs in `net.rs` that I had only partly
  edited.** No word changed and the check in §0 covers it, but the diff is
  wider than the edit, and one line in the first paragraph was over-long before
  this attempt and is now not. If you want the diff minimal rather than the file
  tidy, that is a revert of the reflow hunks.
- **The eight-sentence paragraph is longer than a doc comment ought to be, and I
  cannot tell from inside it whether it is over the line.** Every sentence earns
  its place by a test I can name, which is exactly what someone who has written
  too much would say. If you want it cut, the two I would drop first are the
  Fisher parenthetical and the `5 in 3200` clause, in that order.
