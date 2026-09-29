# riders-67 C5 review — `natnet`, candidate `b482867`, tree `783fdea`

No orphan processes; load back to 8.26 at close.

## Verdict: reject — on one finding, and on nothing else

**Everything C5 was sent to do is done and verified.**

## The coordinator's two questions, first

### Which flake was the gate failure — Flake A

The builder declined to choose because it could not force A sequentially in
`debug`, the gate's profile. This review did: **1/200**. Full campaign on the
extracted candidate, **no port pressure** (TIME_WAIT 841–845, loadavg
7.9–10.3): release seq **1/200**, **debug seq 1/200**, release 4-way 2/200,
release 8-way 1/200, debug 4- and 8-way 0/200 — **5/1200**, every one carrying
the builder's §1.2 message verbatim.

Flake B is real but reproduced only **1 in 800 runs**, and only after this
review's own campaign drove the host to **9 200 `AddrInUse` and 31 786 TIME_WAIT
against the 32768–60999 range** — independently corroborating the builder's
3455/31732 condition report. **Flake B does not block.** The builder's refusal
to choose was right on its evidence and is now resolved against it.

### The rule — right instruction, wrong scope, omits the variable that matters, unenforced

*"Measure this with `max` or do not publish it"* is genuinely the right shape:
its correct output is never nothing, so compliance is visible in the artifact.
But the universal *"anywhere in this crate"* lives **forty lines into one
function's `match` arm** — a contributor adding a ceiling in `src/decoder.rs`
never sees it.

**Nothing in the tree violates it today**, verified over the whole tree (README,
`examples/`, `docs/`, `codebase-audit-docs/` — not just the `src`+`tests` the
builder checked), and the grep has reach, returning live hits in
`examples/imagery_dump.rs` and `src/generate.rs`. It is enforceable by **no
gate** — not fmt, clippy, the 16 feature-docs rows, deny or machete. And it
omits **load**, which is what actually breaks its own figures (MF-C).

## BLOCKING — B1. The axis test's doc comment claims determinism it does not have

`a_peer_still_sending_past_the_deadline_is_a_timeout` says *"Deterministic, with
no window and no retry"*, *"the peer cannot have handed the whole record to the
kernel, no matter how… the two threads were scheduled"*, and *"A slow host makes
the peer less far along… so it cannot turn this test red."*

All three are false, and **the third is the inverse of the truth** — the test's
own prefix read frees receive window, so a slower host makes the peer *further*
along.

§7.1 (a) ∧ (b) ∧ (c), applying the clause list; §7.1 states outright *"(a) is
satisfied by a rustdoc sentence"*, and the riders-21 C1 F1 precedent returned
blocking for the same over-claiming direction. **Input that breaks it:** that
test, run repeatedly — **~1 in 240**.

**C5 did not introduce it** (`git log -S` puts it at `87e5b51`, C3). The scope
argument is the weak part of exempting it: the brief says *"this attempt changes
numbers and sentences"*, and **C5 rewrote another test's doc comment in this
very commit**. Discharge is three sentences, not a slice.

## MUST-FIX

- **MF-A (tree)** — shipped *"at 4 MiB this branch is 1.65–2.38× slower on the
  median of 60 over four paired runs"*. That exact instrument re-run at load
  8.1: base p50 **37 µs in all eight runs**, candidate 87–96 → **within-run
  paired ratios 2.35–2.59×**. Five of eight exceed the upper bound; none
  approaches 1.65. The shipped band looks like a ratio of **cross-run column
  ranges** (91/55 = 1.65, 95/40 = 2.38) rather than paired ratios.
- **MF-C (tree)** — N4's figures do not reproduce and **load ships nowhere**.
  `debug` median of 30: shipped band 210.1–227.1 ms, measured
  **201.5 / 202.1 / 201.6**. `release` median of 60: shipped ~25.4 ms, measured
  **20.49–20.66**. Spreads under 1%. Sharply: **C4's original "202 ms" is the
  figure that reproduces — the N4 correction tracked host load, not the base.**
- **MF-B (report-only)** — §4's *"1.7× span, worse than the base-side 2.13×"*
  compares a span to a margin. In the test's own shape, `max` leaves the
  base-side margin unchanged (**2.09×**) and destroys the candidate-side one
  (14.5× → **0.96×**). And *"fails 3/3"* is **2/3** on this host. The conclusion
  — leave the assertion, fix the comment — is right and was measured.

## Verified clean

The load-bearing safety property holds under a **real Rust lexer**, not the
builder's line filter: comments and doc comments (including nested block
comments) stripped, string and raw-string literals preserved and compared
verbatim (9 and 247 literals) — both files **byte-identical** to C4.

§2.4's attribution **reproduces**: both trees reach the same order of worst
call; only the *frequency* differs, reuse-only, **6–10/60 against 1/60**.

The 4.2×, its companion 4×, the 2.4× and *"at most one pass"* are struck from
the tree, with a non-vacuous reach check, and the retraction is in the commit
message.

Every gate row re-run matched **exactly**: 183/0 over 16 with all per-target
cells, 99/0, six features rows, 16/16 feature-docs, BASE 180/3 (`imagery_net`
21/3), A1 22/2, A2 23/1, each failing exactly the named tests.

**This is not report-side** — B1, MF-A and MF-C are against the tree; MF-B is the
only report-only finding and is graded as such.

## One call the reviewer expects to be overruled

§7.1's clauses are **locus-based** and say nothing about authorship, so B1 blocks
because it sits in the candidate tree. If the coordinator rules that a blocking
finding must be attributable to the candidate, this is **accept-with-must-fix**
on MF-A/B/C.

*Coordinator's ruling, recorded when this review was committed:* **B1 stands as
blocking.** Not on the locus rule alone, but because this candidate **names
Flake A in its own report while shipping a doc comment that says the same test
is deterministic** — a contradiction internal to one candidate, on the slice's
own subject. The reviewer's scope argument is also correct: C5 rewrote another
test's doc comment in this very commit, so the class of change was in scope.
