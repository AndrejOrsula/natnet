# riders-67 C6 review — `natnet`, candidate `b03817b`, tree `e5fc608c`

Host, stated because this attempt's subject is that figures need it: loadavg
**8.89** at open, **16.10** at close, 32 cores; TIME_WAIT **5 → 5** (two
campaigns ran and ended during the review; per-block loads are given beside
every number below).

## Verdict: accept-with-must-fix — two must-fixes, no blocking finding

**This candidate is largely clean, and the parts it was told to push hardest on
are the parts that held.**

## MUST-FIX 1 (tree + report) — the rule's compliance was verified for the *pre-load* rule and re-asserted for the *post-load* one

Report §6: *"**Nothing in the tree violates the rule today**; the review
verified that over README, `examples/`, `docs/` and `codebase-audit-docs/`."*
The C5 review sentence it cites reads *"Nothing in the tree violates it today,
verified over the whole tree"* — and two sentences later, *"And it **omits
load**, which is what actually breaks its own figures."* **Adding the load
clause is C6's entire edit to the rule.** The transplanted verification cannot
cover the clause C6 added, and does not.

Run over the candidate tree, measured figures missing one of the three things
the new rule requires:

- **`README.md` — the rule's own paragraph**, and the rule names *"this file"*
  in its own population. `2.8 ms` carries "a median of 15" and **no load**.
  `202 ms at loadavg 9` and `227 ms at loadavg 13-32` carry statistic and load
  and **no repetition count** (they are medians of 30). **Each of the rule's
  three own figures omits exactly one of the three things it demands.**
- **`src/imagery/net.rs:441`** — the same 2.8 ms retraction, **no load**; not
  covered by the new *"Load belongs to each figure"* heading, which scopes
  itself to *"These tables"*.
- **`tests/imagery_net.rs:1165`** *"9-12 of 60 calls over 10 ms, against 1 of
  60"* — no load. **`:1172`** *"both produce calls of 10-17 ms"* — no statistic,
  no rep count, no load. **`:1182`** gives concurrency, not loadavg. All three
  pre-existing and untouched by C6.
- **`src/imagery/net.rs:420`, a line C6 itself rewrote**: *"1.3-20.7 ms there in
  `release`"*. 1.3 is the 64 MiB `resize reuse` cell (1299 µs, loadavg 13-32);
  20.7 is the 64 MiB `resize fresh` cell (20688 µs, loadavg 9.1-9.4). **One
  range, two loads, no load on the range.**

**Grade, and which rule applied.** The report sentence is a false conclusion
that touches no shipped line, so §7.1's third-pairing ruling governs: *"a false
conclusion that touches no shipped line is `must-fix`, not blocking"*. The
tree-side half does not reach blocking because (c) — *"a false statement a later
author would act on"* — is not satisfied: the rule's text is normative, not
false; the defect is that **its own worked examples do not satisfy it**.

**Cheapest discharge:** three parentheticals (rep counts on 202/227, a load on
2.8 ms) plus replacing the report sentence with what was actually verified.

## MUST-FIX 2 (tree) — the direction claim ships unhedged, and the reviewer ran the measurement the builder said it did not

Shipped, in bold, flat, in the axis test's doc comment: *"**A slow host makes
the peer *further* along, not less**: … so host speed pushes this test red
rather than green."* Report §9 discloses *"The direction claim is argued, not
measured"* — **but §9 does not ship; the comment does**, one sentence after a
measured rate, so a reader takes both as measured.

Release `imagery_net` binary built from the candidate tree, run sequentially,
`--exact`, one process per run:

| arm | instrument | failures | load / TIME_WAIT |
| --- | --- | --- | --- |
| unconstrained | no pinning | **5 / 3200** | 18.2 → 22.9, TW 70 → 343 |
| slow | `taskset -c 0`, that core also carrying 3 busy spinners | **0 / 1200** | 13.8 → 15.8, TW 5 → 5 |
| fast (paired, interleaved 200-run blocks, same session) | `taskset -c 1`, idle core | **2 / 1200** | same |
| earlier, unpaired | `-c 0,1` 0/400; `-c 0` 1/400 | 1 / 800 | 18.6 → 17.9 |

**Ablation proving the slowdown instrument is non-vacuous:** 60 runs pinned to
core 0 took **4.90 s** with no spinners and **8.75 s** with three — a 1.79×
slowdown of the process under test.

Two readings, both against the claim. (1) Arms differing only in whether the
shared core is contended: **0/1200 slowed against 2/1200 not**. Slowing did not
push it red. (2) Unconstrained at roughly twice the builder's load: **5/3200 =
1/640** against its **5/800 = 1/160** at loadavg 8.9–9.3; under Poisson at that
rate, P(≤5 | λ=20) ≈ 5e-4. **On this host higher load went with fewer
failures.**

**This is not a refutation** — 0 against 2 is one-tailed Fisher p ≈ 0.27, and
the spinners slow the peer thread as well as the test thread. **It is the first
datapoint and it does not point the claimed way.**

**Grade:** the false-premise rule — the conclusion stands and was reproduced;
the reason offered for it is what the measurement does not support. Not
blocking: (c) requires a false statement a later author would act on, and
falsity cannot be established at this power.

**Why it matters more than its grade.** C5 was rejected for shipping *"A slow
host makes the peer **less** far along"* as a flat unmeasured assertion. **C6
ships the inverse as a flat unmeasured assertion. The sign flipped; the
epistemic status did not.** Fix is one clause: say the direction is argued from
the mechanism and not measured — which the report already says, in the file that
does not ship.

## Verified clean — and these are the load-bearing ones

- **"Code unchanged" holds under an independent Rust lexer, not the builder's
  md5.** Nested block comments, `///`/`//!`, raw strings with `#` hashes, byte
  and c strings, char-vs-lifetime. Token streams **identical** between
  `b482867` and `b03817b`, literals compared verbatim: **1756 tokens / 9
  literals** and **8718 tokens / 247 literals** — the literal counts match the
  builder's independently. **Reach check, non-vacuous:** perturbing one
  identifier *and* one character **inside a string literal** each return DIFFER
  at the right token.
- **The rate, the panic and its three numbers reproduce.** Identical panic text,
  identical `Ok(true)` / **5225268** / **2612608** / **2612608**; 5225268 =
  2 × 2612608 + 52 as stated.
- **Every gate number reproduces exactly**, in an isolated target dir: fmt rc 0;
  clippy 0 warning-or-error lines; **183/0** over 16 targets with **all sixteen
  per-target cells** matching; **99/0**; doc ×2; features matrix
  **127 / 121 / 101 / 101 / 179 / 103**; MSRV **99** and **183**; **feature-docs
  16 of 16**; deny rc 0; machete rc 0.
- **BASE comparison reproduces**: 21 passed / 3 failed, exactly the three named
  tests.
- **The disclosed `tail -12` weakness is resolved, not inherited.** Untruncated:
  `grep -c license-not-encountered` → **2**, the only two `warning` lines in the
  whole output. *"Exactly two"* is now confirmed by a count, not by location.
- **The restored 202 ms, at a third load.** The base's `net.rs` on the candidate
  tree, `debug`, 64 MiB fresh, the ladder test's own **median of nine** — a
  different statistic from the builder's median of 30, **named so it does not
  travel as a fourth notation** — five runs: **204.8 / 204.9 / 205.8 / 209.0 /
  209.5 ms** at loadavg 16.67 → 16.81. **Monotone between the builder's 202 at
  load 9 and C5's 227 at load 13–32** — independent support for restoring 202
  *and* for the README's worked example.
- **Refuted-citation sweep, all three legs, flattened.** The only surviving
  `1.65` is inside its own withdrawal; the only surviving `227`/`25 ms` are
  load-qualified historical citations; commit messages carry all three
  retractions. *"First sequential `debug` reproduction in a builder's hands"*
  checks out — C5's table row reads `debug, sequential | 0 / 150`.
- **Placement of the rule is sound.** No `CONTRIBUTING.md`, no PR template, no
  `include_str!` of README anywhere; `readme = "README.md"` confirmed.
- **MF-A's internal arithmetic checks**: all eight paired ratios recompute, band
  2.38–2.51 correct, and it sits inside the reviewer's independent 2.35–2.59.

## Notes

- **N1** — report §2(b) calls `git archive b482867` *"the extracted candidate
  tree"*; that is the **base**. The parenthetical discloses the revision in the
  same sentence, so it self-corrects, and byte-identical code carries the
  result.
- **N2** — report §2(b) puts the panic at `tests/imagery_net.rs:1106`; on the
  shipped tree it is **1113** — 1106 is the base's line, the doc-comment hunk
  adding 7 above it.
- **N3** — MF-B's *"which side of it a run lands on is decided by the load"*
  rests on two points from two sessions. Directionally consistent, but the same
  argued-not-measured shape as MF-2. Report-side only; the 7 per cent arithmetic
  is right (11037/10350 = 6.6 over).
- **N4** — **the rule does not distinguish a *measured* figure from a
  *configured* one**, so read literally it catches README's *"5-second connect
  and 30-second record defaults"*, the tests' `1 ns`/`1 µs` deadlines and the
  `40 ms per byte` drip, and `examples/imagery_dump.rs:216`. None of those can
  carry a statistic, a rep count or a load. **A scope clause fixes it.**
- **N5** — `readme = "README.md"` means this section ships as the crates.io
  front page, and it is now the last prose section before the licence line. The
  builder weighed the tradeoff only in the other direction. Correctly disclosed
  as a judgement call.
- **N6** — no *"what in the brief was wrong"* section, which `AGENTS.md` §7
  item 2 calls *"The most valuable section"*. The substance is there — §4
  attributes the C5 brief's N4 correction to host load rather than to the base —
  just not under that heading. **The C6 brief is itself inconsistent**: its
  header says *"four sentences and three numbers"* against §2's *"Discharge is
  three sentences"*, and the report adopted "Four" rather than flagging it.

## On disposition

**This is not report-side.** MF-2 is against a shipped doc comment; MF-1's
figure-level half is against `README.md` and `src/imagery/net.rs`, and only its
*"Nothing in the tree violates the rule today"* half is report-only. But
together they are **one clause and three parentheticals — a C7 materially
smaller than C6**.

**On the coordinator's C5 instruction.** The N4 correction C6 undoes was the
brief's, and **C6 undoes it correctly and for the right reason**: 202
reproduced independently, at a third load, and the load-monotonicity the README
asserts holds across all three readings. That one is settled.
