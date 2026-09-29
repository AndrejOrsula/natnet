# riders-67 C8 review — `natnet`, candidate `652039bc`, tree `e311a0f0`

Loadavg at session open **11.91 / 11.70 / 11.45**, at close **11.93 / 12.05 /
11.72** (32 cores); stated per measurement below.

## Blocking

**None.**

## Must-fix — one, report-side, one word

**MF-1 — §10's load sweep is labelled `39 lines`; 39 is the occurrence count and
the line count is 37.** The builder's instrument: *"every occurrence of the word
`load` … minus `payload`/`upload`/`download`/`load(`/`loaded`. **39 lines.**"*
Reproduced at `7a94509` — the pre-fix tree the sweep ran against — line count
**37**, occurrence count with the same exclusions **39**; two lines carry two
hits each. No line-based variant of the exclusion list reaches 39 (four tried:
38, 38, 37, 37). Loadavg 11.77.

**The conclusion is true and was verified**: the sweep's two hits are real, both
are the rule's own worked example, and `"at a load nobody recorded"` now returns
**0** over `README.md src tests examples`. Only the **unit** offered for the
population is wrong. `AGENTS.md` §7 is the operative standard: *"**State the
domain of every count you publish** — the population it was taken over, and the
unit it counts — **in the same sentence, and check that the domain you describe
is the domain you measured.**"* Fix: *"39 occurrences over 37 lines."* Not
blocking — (a) bars a report-only finding from blocking.

## Notes

**N1 — the sixth blind spot is a *locus*, not a notation, and the rule's own
first sentence names it.** The rule's population is *"rustdoc, implementation
comments, this file, examples, **commit messages**"*. Both sweeps' paths are
`README.md src tests examples`. The candidate's own four commit messages
publish, at sites carrying none of the three fields: `1.65-2.38x` and
`2.38-2.51x`, `2.8 ms`, `4-6x`. The mention/withdrawal defence is foreclosed by
the candidate's own new clause — *"a withdrawn figure is still a published one
and carries its fields"*. **Note, not must-fix**: (a) does not reach a commit
message; (d)'s qualifier is not met for figures the messages themselves label
non-compliant; and a frozen message cannot be amended, so the repair is a fifth
commit — and `riders-77` owns commit messages.

**N2 — the sixth *notation* exists, it has exactly one instance, and the
candidate's own new clause covers it correctly.** An independent notation-first
sweep (fraction/multiple words, `order of magnitude`, `N:1`, `per cent`,
spelled-out time units, throughput/`Hz`, `N in M`) returned one hit none of
C8's five notations reads: `tests/imagery_net.rs:1177` — *"at `MAX_RECORD_LEN` a
reused buffer costs `2c3041f` **a twentieth** of what a fresh one does"*.
Measured timing ratio, no statistic, no rep count, no load. `git blame` puts it
at **`3c1ee64` (2026-09-17)**, which predates the rule, so under the candidate's
own commencement reading it is grandfathered — and the clause the candidate
shipped calls it exactly what it is: *"a non-compliant figure in a notation
neither sweep reads is **unaudited rather than exempt**"*. **This is the
generalisation passing its own test, not failing it.** Everything else the sweep
returned is configured or non-timing.

**N3 — §14's stated consequence of the alternative reading does not check out.**
It says *"then the three grandfathered `tests/imagery_net.rs` lines become
defects"*. The three figures the exception names stay exempt under **both**
readings. A blame-plus-sweep of that file finds exactly **one** line the
alternative would newly condemn — and it is N2's `a twentieth`. The escalation
is right; its cost estimate is under-specified by two.

**N4 — one unaudited negative the strengthened clause does not reach.**
`README.md:134`: *"the loads of the older campaigns were mostly not recorded and
cannot be recovered"* — a negative over a named population with no instrument,
in the same paragraph as the clause requiring negatives to carry theirs. The
clause is scoped to *"the figure says so in the load's place"*, so it genuinely
does not bind this sentence, and `natnet` has no `AGENTS.md`. Recorded, not
asked for.

## Ruling requested — the commencement clause: the builder's reading stands. Do not reverse.

**The C7 review's own grading presupposes it.** Its MUST-FIX 1 establishes
population by *"the commencement clause binds 'every figure published … from the
revision that added it onward' — **this revision published it**"*, and its N1 by
*"**the candidate reflowed that line**"*. That touched-implies-bound test **is**
the builder's clause and has no work to do under the alternative, where
everything non-compliant is a defect regardless of touching. The alternative
also reads the *"It is not retroactive"* sentence out of the rule — and it is
not vacuous to do so: `tests/imagery_net.rs:1177` is a live figure the two
readings classify differently. **The builder's is the reading the slice has
already been decided under, twice.**

## Verified clean — with the reviewer's own instruments

- **No code.** Own comment strip, not the builder's hashes: **272 lines, md5
  `d1d7269b…`** and **1113 lines, md5 `47f2a54f…`**, both `cmp`-identical to
  `70d0a02`. Reach check: perturbing one identifier makes `cmp` DIFFER. Second
  instrument: a trailing-`//`-strip leaves the files differing only by blank
  lines. Third: `git diff -U0` has **0** changed lines failing the comment
  pattern. Fourth: **no `include_str!`** anywhere in `src/` or `tests/`.
- **The fifth ratio is real and was unenumerated.** `1.65-2.38×` is in C7's own
  added-lines list at line 39, exactly as claimed. Flattened, `1.65` occurs
  **0×** in C7's review, brief and report and in C8's brief.
- **`1.79×`** — C6 review gives 4.90 s and 8.75 s over 60 runs per arm;
  8.75/4.90 = 1.7857. Statistic, rep count and operands correct as shipped.
- **`4-6×`, and the choice between two notations is right.** The tree's max
  table gives 31 MiB reuse **11721–16706 µs**; the other commit's prose `10.3`
  is a **cross-column range** matching no single cell. 11721/2803 = **4.18**,
  16706/2803 = **5.96**. The builder's disclosure that the shipped figure *"is
  really 3.7–6.0× and rounds the wrong way"* under the alternative is correct.
- **`3.3×`** — C4 review cells 50.8/166.3 → 3.274, *"four paired runs … median
  of 21"*. Load negative over all four C4 files: **0, 0, 0, 0**; reach check
  over C6's: **1, 4, 26, 8**.
- **Both `2.8 ms` escape objects** carry **0** load strings each.
- **The self-found defect and its repair**: `"at a load nobody recorded"` → **0**
  occurrences in the shipped tree; both former sites now name two checkable
  objects.
- **Every gate re-run reproduces cell for cell**, with loads: clippy **0**
  warning-or-error @ 11.66→11.92; `test --all-features` **183/0** over **16**
  with all cells @ 11.92→12.08; no-features **99/0** @ 12.08; all six features
  rows @ 11.88→12.67; MSRV on **cargo 1.92.0** — an explicit `+1.92`, not the
  declared-and-ignored shape — **99/0** and **183/0** @ 12.67→12.81; deny rc 0
  with untruncated counts 2 and 2; machete rc 0 @ 12.60. **Two gates not run,
  and said so**: the 16-row feature-docs matrix and `cargo-deny fetch db`.
- **BASE comparison**: md5 as reported; **21 passed / 3 failed in 7.25 s**,
  exactly the three named tests @ 12.56→13.10.
- **Four gate runs, correctly disclosed** — `gates.md` provenances each to a
  tree and states plainly that run 1 spanned an edit and is not citable.

## The splitter question — well-posed once the terminator set is named; both counts are right

Both were reproduced. The C6 paragraph has **4** sentence-final periods, 1
colon, 1 semicolon; the C7 paragraph has **8**, 1, 1. Period-only gives **4 and
8** — C7's count, correct as a *sentence* count. Period plus colon plus
semicolon gives **6 and 10** — C8's, correct as a *clause* count. The
disagreement is exactly the one colon and one semicolon in each, not noise. The
builder is right that a text-unit size is instrument-dependent and right not to
publish 6 and 10 as *the* answer. **Nothing false shipped.**

## Verdict: accept with must-fix

One must-fix, **report-side, one word**. Four notes, none requiring an edit to
ship. **Nothing blocking, and no input or state that breaks this candidate.**

**Plainly: this is a clean candidate.** Every tree-side claim held under an
instrument built by the reviewer rather than inherited; the two self-found
defects are real and really repaired; the escape-clause defect §10 reports was
in the tree and not the diff, and is fixed; the fifth ratio exists where the
builder says, and no list on this slice had it.

The reviewer was sent to find the sixth notation the generalisation predicts,
and **it is there** — `a twentieth` at `tests/imagery_net.rs:1177` — plus a
sixth **locus** in the commit messages the rule's own first sentence names.
**Neither is a defect in this candidate**, because the clause it shipped —
*"unaudited rather than exempt"* — is precisely the clause that classifies them.
**That is the difference between a rule that closed a list and a rule that
stated how far its closure reaches, and it is the most valuable thing in the
packet.**

Everything against the tree is a note. **The only thing that must change is
report-side**, and the reviewer says so in those words.
