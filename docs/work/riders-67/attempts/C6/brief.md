# riders-67 C6 — the test you proved flaky still calls itself deterministic

**Slice:** riders-67, attempt **C6**. Repository **`natnet`**. **Base =
`candidate(C5)` `b482867`** (tree `783fdea`) — resolve it yourself. Branch
`slice/riders-67-C6`, worktree
`/home/andrej/ws_rs/natnet_wt/impl-riders-67-C6`. Freeze **C6**. Access
`remote`. **This attempt is four sentences and three numbers. Do not widen it.**

Read `docs/work/riders-67/attempts/C5/review.md` in full first, then C5's
`report.md`.

## 1. Everything C5 was sent to do is done and verified

The reviewer's words. The comments-only property holds **under a real Rust
lexer**, not a line filter — both files byte-identical to C4 with 9 and 247
string literals compared verbatim. §2.4's attribution reproduces. The struck
figures are struck with a non-vacuous reach check. Every gate row re-run matched
exactly. **Reopen none of it, and do not touch the allocation path or any test
body.**

Both of the questions you left open are now answered, in your favour on the
process point: **Flake A is the gate failure** — forced sequentially in `debug`,
the gate's own profile, at 1/200, over a 1 200-run campaign with no port
pressure — and **Flake B does not block**, reproducing once in 800 and only
under 9 200 `AddrInUse`. Your refusal to choose was right on the evidence you
had.

## 2. What is blocking, and why I ruled it in scope

`a_peer_still_sending_past_the_deadline_is_a_timeout`'s doc comment says
*"Deterministic, with no window and no retry"*, *"the peer cannot have handed
the whole record to the kernel, no matter how… the two threads were
scheduled"*, and *"A slow host makes the peer less far along… so it cannot turn
this test red."*

All three are false. **The third is the exact inverse of the truth**: the test's
own prefix read frees receive window, so a slower host makes the peer *further*
along.

C3 introduced those sentences, not you. **I ruled it blocking anyway**, and the
reason is not the locus rule: **this candidate names Flake A in its own report
while shipping a doc comment that says the same test is deterministic.** That is
a contradiction internal to one candidate, on this slice's own subject. The
reviewer's scope argument is also right — you rewrote another test's doc comment
in this very commit, so the class of change was already in scope.

**Discharge is three sentences.** Say what the test actually holds, at what
rate it does not, and which direction host speed pushes it. **Do not fix the
flake** — that is `riders-76`, already queued, and its brief forbids pinning it
by widening a tolerance.

## 3. Three numbers

- **MF-A** — *"at 4 MiB this branch is 1.65–2.38× slower on the median of 60
  over four paired runs"*. Re-run at load 8.1: base p50 **37 µs in all eight
  runs**, candidate 87–96, **within-run paired ratios 2.35–2.59×**. Five of
  eight exceed your upper bound and none approaches 1.65. Your band looks like a
  ratio of **cross-run column ranges** (91/55, 95/40) rather than paired ratios.
  Re-derive it as paired ratios, or say what it is.
- **MF-C — and this one is partly my fault.** N4's figures do not reproduce:
  `debug` median of 30 measured **201.5 / 202.1 / 201.6** against your
  210.1–227.1; `release` median of 60 measured **20.49–20.66** against your
  ~25.4. Spreads under 1%. **C4's original "202 ms" is the figure that
  reproduces** — the correction my C5 brief asked for tracked **host load**, not
  the base. Restore what is true, and **ship the load beside every figure**,
  which neither set of numbers does.
- **MF-B (report-side)** — §4 compares a *span* to a *margin*. In the test's own
  shape, `max` leaves the base-side margin unchanged (**2.09×**) and destroys
  the candidate-side one (14.5× → **0.96×**); and *"fails 3/3"* is **2/3** on
  the reviewer's host. **Your conclusion — leave the assertion, fix the comment
  — is right and was measured.** Correct the comparison in C6's report.

## 4. The rule you shipped: keep it, and move it

*"Measure this with `max` or do not publish it"* is **the right instruction** —
its correct output is never nothing, so compliance is visible in the artifact.
Three things are wrong with where it sits: a universal about the whole crate
lives **forty lines into one function's `match` arm**; it is enforceable by no
gate; and **it is silent about load**, which is what actually broke its own
figures.

Nothing in the tree violates it today — verified over README, `examples/`,
`docs/` and `codebase-audit-docs/`, not just `src` and `tests`.

**Move it somewhere a contributor meets it, and add load to what it requires.**
Whether it should also become a gate is a real question and **not for this
attempt** — if you think it should, say so and I will queue it.

## 5. Practical

- **Assume you will be interrupted.** Draft `report.md` early, commit at each
  milestone. This session has lost runs to the limit eight times.
- **Do not commit to `docs/work/`.** Drafts under
  `docs/work/riders-67/attempts/C6/`, uncommitted. **Never push.** No
  `codebase-audit-docs/` edit, no frozen packet edit.
- `export CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c6/target`.
- **State the load beside every number you publish**, and re-check it before
  each timing run. The host is contended and two campaigns have saturated its
  ports.
- Every gate of `natnet`'s workflow with its numbers.

## 6. Acceptance

- The axis test's three sentences saying what is true, including the direction
  host speed actually pushes it.
- MF-A re-derived as paired ratios, or relabelled for what it is.
- N4 restored to the reproducing figures, with load beside them.
- MF-B's comparison corrected in the report.
- The rule moved where a contributor meets it, with load in it.
- Every gate with its numbers; `candidate(C6)` SHA and tree; what you are unsure
  about.
