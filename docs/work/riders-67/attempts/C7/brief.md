# riders-67 C7 — one clause and three parentheticals

**Slice:** riders-67, attempt **C7**. Repository **`natnet`**. **Base =
`candidate(C6)` `b03817b`** (tree `e5fc608c`) — resolve it yourself. Branch
`slice/riders-67-C7`, worktree
`/home/andrej/ws_rs/natnet_wt/impl-riders-67-C7`. Freeze **C7**. Access
`remote`.

**C6 was accepted with must-fix, nothing blocking.** Read
`docs/work/riders-67/attempts/C6/review.md` in full first, then C6's
`report.md`. **The reviewer's own sizing is "one clause and three
parentheticals". Do not exceed it.**

## 1. What held, verified harder than you verified it

Code-unchanged was re-derived under an **independent Rust lexer** with a
non-vacuous reach check — token streams identical, **9 and 247 literals**
compared verbatim, and perturbing one character *inside a string literal*
returns DIFFER at the right token. The rate, the panic and its three numbers
reproduce. **Every gate figure reproduces exactly.** The `tail -12` weakness you
disclosed is **resolved** — untruncated, `grep -c` returns 2. Your restored
**202 ms reproduced at a third load** (204.8–209.5 at loadavg ~16.7, median of
nine, named as a different statistic), so the README's load-monotonicity holds
across three independent readings. **Touch none of it, and no code.**

## 2. MF-2 — the sign flipped and the epistemic status did not

You ship, bold and flat, in a doc comment: *"**A slow host makes the peer
*further* along, not less**… so host speed pushes this test red rather than
green."* Your §9 says the direction is **argued, not measured** — but **§9 does
not ship and the comment does**, one sentence after a measured rate, so a reader
takes both as measured.

**The reviewer ran the measurement you said you had not**: `0 / 1200` slowed
(`taskset` onto a core carrying three spinners) against `2 / 1200` fast
(interleaved 200-run blocks, same session), with an ablation showing the
slowdown is real — **4.90 s against 8.75 s for 60 runs, 1.79×**. Unconstrained
at roughly twice your load: **5 / 3200** against your **5 / 800**. It says
plainly this is **not** a refutation (Fisher p ≈ 0.27, and the spinners slow the
peer thread too) — but it is the first datapoint and **it does not point your
way**.

**C5 was rejected for shipping the opposite direction as a flat unmeasured
assertion.** Fix is one clause: say the direction is argued from the mechanism
and not measured, and that the one campaign run against it did not support it.
**Do not run a bigger campaign to rescue the claim** — if you want to measure it
properly that is its own slice and I will queue it.

## 3. MF-1 — the rule's own worked examples do not satisfy the rule

Each of the three figures in the rule's own paragraph omits **exactly one** of
the three things it demands: `2.8 ms` has a statistic and **no load**; `202 ms`
and `227 ms` have statistic and load and **no rep count**.

Also missing a load: `src/imagery/net.rs:441` (the same 2.8 ms retraction, not
covered by the *"These tables"* heading), and **`:420`, a line you rewrote** —
*"1.3-20.7 ms there in `release`"*, where 1.3 is a loadavg-13-32 cell and 20.7 a
loadavg-9.1-9.4 cell: **one range, two loads, no load on the range**.

Three pre-existing lines in `tests/imagery_net.rs` (`:1165`, `:1172`, `:1182`)
are also non-compliant. **Decide** whether C7 fixes them or the rule declares
its own commencement; either is acceptable, **say which and why**.

And **replace the compliance sentence with what was actually verified**: the
review you cite verified the rule **before** you added the load clause, which is
your entire edit to it.

## 4. N4 — the rule catches things that cannot comply

Read literally it covers README's *"5-second connect and 30-second record
defaults"*, the tests' `1 ns`/`1 µs` deadlines, the `40 ms per byte` drip and
`examples/imagery_dump.rs:216`. **None of those can carry a statistic, a rep
count or a load**, because none is a measurement. **A scope clause fixes it** —
the rule is about *measured* figures.

## 5. Three notes, cheap

- **N1** — report §2(b) calls `git archive b482867` *"the extracted candidate
  tree"*; that is the **base**. Self-correcting in the same sentence, and
  byte-identical code carries the result. Fix in C7's report.
- **N2** — §2(b) puts the panic at `:1106`; on the shipped tree it is **1113**.
- **N6** — your report has no *"what in the brief got wrong"* section, which
  `AGENTS.md` §7 item 2 calls *"The most valuable section"*. The substance is in
  your §4; put it under that heading. **And my C6 brief is inconsistent** — its
  header says *"four sentences and three numbers"* against its own §2's *"three
  sentences"*. You adopted "Four" rather than flagging it; flagging it is the
  section's job.

## 6. Practical

- **Assume you will be interrupted.** Draft `report.md` early, commit at each
  milestone.
- **Do not commit to `docs/work/`.** Drafts under
  `docs/work/riders-67/attempts/C7/`, uncommitted. **Never push.** No
  `codebase-audit-docs/` edit, no frozen packet edit, **no code**.
- `export CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c7/target`.
- **State the load beside every number you publish.** Your own rule, and your
  packet is in its population.
- Every gate of `natnet`'s workflow with its numbers.

## 7. Acceptance

- The direction clause hedged to what is known, including that a campaign ran
  against it.
- The rule's three own figures compliant; `:420` and `:441` carrying their load;
  the `tests/` three decided either way with the reason.
- The compliance sentence replaced with what was verified.
- A scope clause separating measured from configured figures.
- N1, N2 and the brief-critique section.
- Every gate with its numbers; `candidate(C7)` SHA and tree; what you are unsure
  about.
