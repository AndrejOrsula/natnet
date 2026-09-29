# riders-67 C7 review — `natnet`, candidate `70d0a02`, tree `c0be2b75`

Diffstat `b03817b..70d0a02` = 3 files, +72 −39, matching the packet.
**Loadavg at session open 11.26 / 11.40 / 11.32** (32 cores, uptime 17 d);
stated per measurement below. The reviewer's own gate runs moved it 11.3 → 16.3.

## MUST-FIX 1 (tree) — the candidate publishes a new measured figure that violates the rule it publishes in the same commit

`tests/imagery_net.rs:1032`, in the axis test's doc comment:

> a measured 1.79× slowdown of the process under test

The rule the same commit ships in `README.md`: *"Any **measured** timing figure
published anywhere in this crate … carries the statistic that produced it, the
repetition count, and **the host load it was measured at**."* The `1.79×`
carries **none of the three**, and does not use the escape this candidate's own
second commit added (*"Where the load was not recorded and cannot now be
recovered, the figure says so in the load's place"*). Its source — C6 review,
*"60 runs pinned to core 0 took 4.90 s with no spinners and 8.75 s with three"*
— has the rep count; none of it shipped.

It is inside the rule's population **on the candidate's own terms**: the
exception paragraph names *"the cost test's doc comment in
`tests/imagery_net.rs`"*, and `src/imagery/net.rs:426` is the in-tree precedent
that a `×` ratio is bound — it carries all three fields plus the raw cells. It
is not the named exception (the 31 MiB tail), and the commencement clause binds
*"every figure published … from the revision that added it onward"* — **this
revision published it**.

**Why the builder's own instrument could not see it.** Report §5 swept *"every
time-unit figure"*. **A `×` ratio carries no time unit**, so the sweep's
population structurally excluded both `1.79×` and `3.3×`. That is §5(a) in
miniature — which the report names as a lesson and then repeats.

**Grading.** §7.1 **(d)** — *"a shipped quantity that a later author would act
on which you cannot reproduce from the shipped text alone."* (a) ∧ (b) ∧ (c)
does not fire: `1.79×` is **not false**, so (c) fails, and the bar is
*"`(a) ∧ (b) ∧ (c)`, or `(d)` — not a flat disjunction."* The false-premise rule
does not fire either, so **there is no conflict to resolve and (d) is applied
alone**.

**Blocking was considered under (d) and declined**, on the qualifier the bar
attaches to it: *"(d) carries *that a later author would act on* for the same
reason its three siblings do."* The conclusion this figure warrants is *"take
the direction as unknown"* — the safe direction — and the comment names the
slice that will measure it. A later author acts on the hedge and the pointer,
not on the magnitude. Same call as `docs/work/P4.3/attempts/C3/review.md` B3.

**Cheapest discharge:** one parenthetical, using the candidate's own escape —
`a measured 1.79× slowdown of the process under test (60 runs per arm, at a
load the review does not record)`.

## Notes

**N1 — the "closed" exception is not closed, and the commencement paragraph
contradicts itself about `3.3×`.** An independent ratio sweep over `README.md
src tests examples` returned `src/imagery/net.rs:435` — *"Other instruments on
this host have put the 4 MiB figure as high as 3.3×"* — measured, no statistic,
no rep count, no load, not the 31 MiB tail, and **the candidate reflowed that
line**. The paragraph gives a reader two opposite answers: *"It is not
retroactive"* exempts it; *"That is the whole of the exception; anything else
non-compliant is a defect"* condemns it. **This is the answer to "look for a
fourth": the fourth is a whole class, not a line.** The charitable reading is
available and the sentence order supports it, so: note. One clause fixes it.

**N2 — `4-6×` at `README.md:115`, inside the rule's own worked example.** No
statistic and no rep count of its own; it plausibly inherits *"median of 15"*
and *"at a load nobody recorded"* from the same sentence. Weaker than N1, but a
figure in the rule's own paragraph that **neither the C6 review's list nor the
builder's sweep enumerated**.

**N3 — the escape clause is the right shape with one hole.** Right shape: it
produces an artifact, so compliance and omission do not look alike. The hole:
*"cannot now be recovered"* is **an unaudited negative** — a later author can
write *"at a load nobody recorded"* without looking, and the clause does not
require naming where they looked. `AGENTS.md` §7 is stricter about exactly this
class: *"an assertion about a named population — a universal, a **negative**, a
count … carries in its own sentence either the instrument that produced it or
the words *on my word alone*."*

**N4 — attribution: yes, add it, and it is the same clause as MUST-FIX 1.** All
six transcribed numbers were checked against the C6 review and **every one is
correct**: 0/1200 @ loadavg 13.8–15.8, 2/1200, 5/3200 @ 18.2–22.9, 5/800 @
8.9–9.3, 1.79× (8.75/4.90 = 1.7857), Fisher p ≈ 0.27 one-tailed. Nothing is
misattributed — the comment is impersonal and never claims the builder measured
them. What is missing is **provenance**, which is the standard the rule itself
sets.

**N5 — the comment drops the one arm that pointed the builder's way.** The C6
review's table has four arms; the comment ships three. The omitted row is
*"earlier, unpaired | `-c 0,1` 0/400; `-c 0` 1/400 | 1/800 | 18.6 → 17.9"* —
more contention, more failures, i.e. **toward** the claim the comment abandons.
The shipped sentence *"The one campaign that has tested it ran against it"* is
stronger than the review's own *"it is the first datapoint and it does not point
the claimed way."* The builder inherited the review's framing, so this is not a
fabrication — but **`riders-78` should have the row**.

**N6 — the comparison inherits a population mismatch.** `5 in 800` is
release+debug combined; `5 in 3200` is the release binary only. The review drew
the comparison first and the comment repeats it. The conclusion is robust to it.

**N7 — report §8(d) quotes the brief for words it does not contain.** Searched
flattened: *"a load on 2.8 ms"* occurs **1×** in the C6 review and **0×** in the
C7 brief. The parenthetical discloses the true source in the same sentence, so
a reader can refute it from the document itself — the safe kind. **The substance
stands.**

**N8 — §8(c)'s support is foreclosed by the same brief.** *"the cheapest
compliant edit was to delete the direction claim altogether"* — the C7 brief's
acceptance bullet requires the clause to exist, so deletion fails acceptance.
The conclusion may still be right; the example offered for it is not available.

## Verified clean — the load-bearing ones

- **"No code" holds under three independent instruments**, not the builder's
  hashes: comment-stripped `cmp`-identical at **272 lines / `d1d7269b…`** and
  **1113 lines / `47f2a54f…`**; `git diff -U0` over `src tests` = **66** changed
  lines of which **0** fail the comment pattern; and a third check the builder
  did not claim — **no `include_str!` anywhere in `src/`**, so the README edit
  cannot move a doctest, which is why the doc-tests cell is unchanged.
- **The `2.8 ms` load negative holds, and holds wider than the builder claimed
  it.** All four C4 blobs: **0** hits for `loadavg|load average|/proc/loadavg`;
  reach check returns **4 / 24 / 7** over C6's gates, report and review. Then
  widened past the builder's stated weak point — **every** blob on
  `evidence/riders-67` carrying `2.8 ms`, and every commit matching
  `-S'2.8 ms'`. **No site anywhere attaches a load to that figure.** The §11
  hedge can be discharged.
- **The second commit repairs both self-found defects**, shown by the diff: the
  *"says so in the load's place"* clause added, and the uncounted *"exactly
  three figures"* replaced by a figure-named, two-locus clause.
- **Commencement is the right call.** *"A rule against inventing loads cannot be
  discharged by inventing three"* is correct, and reason 1 is checkable: the
  `loadavg 13-32` the builder would have had to borrow is **two inferences
  deep**, which §11 itself discloses as the weakest number in the diff.
- **The scope clause's three extra figures all exist** and are correctly
  exempted. The open query was the right instrument; its population was one
  class too narrow.
- **Every gate figure sampled reproduces exactly**, with loads: 183/0 over 16
  cell for cell @ 11.31→12.08; 99/0 @ 12.08; clippy 0 warning-or-error @
  12.08→12.29; `cargo-deny` rc 0 counted **untruncated** at warning-or-error 2
  and license-not-encountered 2 @ 12.19 — the `tail -12` weakness genuinely
  fixed in the instrument; feature rows 127 / 179 / 121 @ 12.6→16.3;
  `cargo-machete` rc 0, **found at the workflow's own `--root` install shape
  before concluding anything about its absence**.
- **BASE comparison reproduces cell for cell**: 21 passed / 3 failed in 7.24 s
  against the report's 7.25 s, the three being exactly the named tests, @
  12.00→12.82.
- **MSRV was actually exercised**: `rust-toolchain.toml` is `stable` (1.97.1
  here) and `gates.md` records `cargo 1.92.0` for the MSRV rows, so an explicit
  `+1.92` ran rather than the declared-and-ignored shape.

## On the coordinator's sizing language

**Partly yes, and the builder is right about the mechanism and wrong about the
evidence.** The mechanism is real: **hedging a claim is strictly longer than
asserting it**, so any size expressed in text units — sentences, clauses,
parentheticals — makes the honest edit the expensive one. The eight-sentence
paragraph is the predicted result. **Size by scope** (which claims may be
touched), which is how the builder in fact read it.

Where it fails is the support (N8). And on the inconsistency the builder is
right and **understated**: the C6 brief says *"four sentences"* once and *"three
sentences"* twice, so it adopted the **minority** reading.

## Verdict: accept with must-fix

One must-fix, tree-side, graded under **(d)** with blocking considered and
declined. Eight notes, of which **N1 is the one to fix in the same breath** —
both repairs are one clause each in text the candidate already wrote.

**This is not a report-only review** — MUST-FIX 1, N1 and N2 are against the
tree, found by an instrument the builder's sweep could not run.

But, plainly, what seven attempts have earned: **the candidate does what it was
sent to do.** The direction clause is hedged correctly, the numbers in it are
transcribed without a single error, the "no code" claim survives three
independent instruments, the `2.8 ms` negative survives a population wider than
the one that produced it, both self-found defects are genuinely repaired, and
every gate figure re-run came back identical. **The one thing it got wrong is
the thing this whole slice is about, on a figure nobody had enumerated —
including the coordinator.**
