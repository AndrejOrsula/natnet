# riders-67 C8 — the rule, obeyed by the commit that published it

**Slice:** riders-67, attempt **C8**. Repository **`natnet`**. **Base =
`candidate(C7)` `70d0a02`** (tree `c0be2b75`) — resolve it yourself. Branch
`slice/riders-67-C8`, worktree
`/home/andrej/ws_rs/natnet_wt/impl-riders-67-C8`. Freeze **C8**. Access
`remote`. **No code.**

**C7 was accepted with must-fix, nothing blocking.** Read
`docs/work/riders-67/attempts/C7/review.md` in full first.

**I am sizing this by scope, not by length** — your own critique was right that
a size in text units makes the honest edit the expensive one, and the reviewer
agreed and told me to size by scope. Scope: MF-1, N1, N2, N3, and the two
report-side notes. Use as many words as each needs.

## 1. What the reviewer says seven attempts have earned

*"The candidate does what it was sent to do."* The direction clause is hedged
correctly; **all six transcribed numbers are correct**, checked against the C6
review; the no-code claim survives **three** independent instruments, including
one you did not claim (no `include_str!` in `src/`, so the README edit cannot
move a doctest); the `2.8 ms` negative survives a population **wider** than the
one that produced it — every blob on the evidence branch carrying the figure,
and every commit matching it; both self-found defects are genuinely repaired;
and every gate figure re-run came back identical. **Reopen none of it.**

## 2. MF-1 — the commit that publishes the rule breaks it

`tests/imagery_net.rs:1032` ships *"a measured **1.79×** slowdown of the process
under test"*. Your `README.md` rule, **in the same commit**, requires every
measured timing figure to carry its statistic, its repetition count and its
load. This carries **none of the three**, and does not use the escape your own
second commit added.

It is in the population **on your own terms**: your exception paragraph names
that very file, and `src/imagery/net.rs:426` is your in-tree precedent that a
`×` ratio is bound — it carries all three fields plus the raw cells.

**Why your sweep could not see it: you enumerated every *time-unit* figure, and
a ratio carries no time unit.** That is the closed-list defect this slice has now
met four times, inside the instrument built to prevent it. Say so in the report.

Graded under (d) alone — the figure is **true**, so (c) does not fire — with
blocking considered and declined because a later author acts on the hedge and
the pointer, not on the magnitude.

**And add the provenance**, which the reviewer ruled for: a pointer to
riders-67 C6's review. It is the same clause as the fields, and it is the
standard your own rule sets.

## 3. N1 — the closed exception is not closed, and the paragraph contradicts itself

A ratio sweep returns `src/imagery/net.rs:435` — *"Other instruments on this
host have put the 4 MiB figure as high as 3.3×"* — measured, no statistic, no
rep count, no load, **not the 31 MiB tail, and a line you reflowed**. Your
commencement paragraph gives a reader two opposite answers: *"It is not
retroactive"* exempts it, *"That is the whole of the exception; anything else
non-compliant is a defect"* condemns it. **The fourth missed figure is a whole
class, not a line.** One clause in the same paragraph fixes it.

**N2** — `README.md:115`'s `4-6×` sits **inside the rule's own worked example**
with no statistic and no rep count of its own. Neither the C6 review's list nor
your sweep enumerated it.

## 4. N3 — the escape clause's one hole

*"Cannot now be recovered"* is **an unaudited negative**. A later author can
write *"at a load nobody recorded"* without looking, and the clause does not
require naming where they looked. `AGENTS.md` §7 is stricter about exactly this
class: *"an assertion about a named population — a universal, a **negative**, a
count … carries in its own sentence either the instrument that produced it or
the words *on my word alone*."* **Your own use of the escape is exemplary** —
you swept four blobs and the reviewer widened it further. Make the clause
require what you actually did.

## 5. Two report-side corrections

- **N7** — your §8(d) quotes my brief for *"a load on 2.8 ms"*; that phrase is
  in the **C6 review**, not the C7 brief. The parenthetical discloses the true
  source in the same sentence, so it is the safe kind. **The substance stands.**
- **N8** — §8(c)'s support is foreclosed: *"the cheapest compliant edit was to
  delete the direction claim"* is not available under a brief whose acceptance
  required the clause to exist. **The conclusion may still be right and the
  reviewer and I both think it is** — find it better support or state it as
  judgement.

## 6. Not yours, recorded so it survives

The C6 review's campaign had **four** arms and the comment ships three. The
omitted row — *"`-c 0,1` 0/400; `-c 0` 1/400"*, more contention and more
failures — points **toward** the claim the comment abandons. You inherited the
reviewer's own discounting of it, so this is not a fabrication. **I have put the
row in `riders-78`'s board entry.** Do not re-litigate it here.

## 7. Practical

- Draft `report.md` early, commit at each milestone. **No `sleep N; tail …`
  jobs.**
- **Do not commit to `docs/work/`.** Drafts under
  `docs/work/riders-67/attempts/C8/`, uncommitted. **Never push.** No
  `codebase-audit-docs/` edit, no frozen packet edit, **no code**.
- `export CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c8/target`.
- **State the load beside every number you publish.**
- Every gate of `natnet`'s workflow with its numbers.

## 8. Acceptance

- `1.79×` carrying its fields (or the escape) **and its provenance**.
- The commencement paragraph no longer answering two ways; `3.3×` and `4-6×`
  resolved.
- The escape clause requiring the instrument, not just the assertion.
- N7 and N8 corrected in the report, and the report saying why the sweep's
  population could not reach a ratio.
- Every gate with its numbers; `candidate(C8)` SHA and tree; what you are unsure
  about.
