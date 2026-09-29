# riders-67 C4 review — `natnet`, candidate `c5d1cce`, tree `495974a3`

## Verdict: reject — one blocking finding

**Nothing here is report-side.** The blocking finding is three shipped numeric
sentences, one of them public rustdoc; the two must-fixes are a public rustdoc
ceiling and a shipped test's own doc comment.

## The two questions the coordinator asked, answered first

**Is the trade acceptable? Yes — as a trade.** The regression band measured end
to end (release, reused buffer): 64 KiB 1.05×, 256 KiB 0.91×, 1 MiB ~1.5×,
**4 MiB 2.6–3.3×**, 8 MiB 1.02×, 16 MiB 0.99×, **31 MiB 0.86–1.08×**, 32 MiB
0.018×, 64 MiB ~0.009×. The regression is confined to roughly 1–8 MiB and its
absolute worst is **~110 µs added to a call that is already failing**, bought
against a worst case falling from **26.6 ms to 10.9 ms** in release and **217 ms
to 11 ms** in debug. Judged on lens 4 — *"Any unbounded queue, hot-loop
allocation…"*, the lens that makes a peer's declared length stop buying work —
that is clearly the right side.

The one alternative the builder did not test was tested here: `*buf =
Vec::new();` before `vec![0u8; total]`, so the old allocation is freed first. It
does **not** help (99.5 µs fresh / 99.1 µs reuse at 4 MiB, against the
candidate's 96–98 µs). Report §4's *"inseparable from the mechanism"* survives
the obvious counter-proposal. **The regression is not graded a defect.**

**Do the shipped sentences describe it honestly? Qualitatively yes,
quantitatively no.** The retraction is real, the mechanism is now in code
(`grep -cE "mmap|threshold|glibc|calloc|alloc_zeroed|DEFAULT_MMAP"`: **0** at
`987f2ec`, **7** at `c5d1cce`, **2** in tests — all four figures reproduce), and
the regression is named at the branch with its cause. No unconditional claim has
been restored in prose. **But C4 traded an unmeasurable universal for numeric
universals** — *"at most 2.8 ms"*, *"up to a 2.4× cost"*, *"at most one pass"* —
and those are still universals, now falsifiable. Two are false on this host.

## B1 — blocking. The shipped 2.8 ms / 3.3 ms ceiling is refuted on this host

Shipped at four places, one of them public rustdoc: `src/imagery/net.rs:61`
(`//!`), `:243` (**public rustdoc**), `:366,370` (implementation comment) and
`tests/imagery_net.rs:1149-1151` (test doc comment).

Candidate `c5d1cce`, **release**, declared length **31 MiB**, reused buffer,
3 × 60 = 180 timed `recv_record` calls: **six cells exceed 2.803 ms**, at
**11225 / 10966 / 10281 / 10655 / 13366 / 12153 µs**. Max **13.37 ms — 4.8× the
shipped ceiling**. In **debug**, max **11.14 ms** against the shipped 3.3 ms.
The same 10–12 ms tail appears at every sub-cap length scanned.

**Ablation, so this is not harness noise**: identical probe, same connection
churn, only the declared length differs — at **64 MiB** (the `mmap` path) the
max over 60 reps is **15–44 µs**; at **31 MiB** (the heap path) it is
**10.5–13.4 ms**. The tail is the allocation.

**The causal error is the instrument. The builder measured a *ceiling* with a
median of 15, which structurally cannot see a worst case.** The 2.803 ms cell
the public figure comes from is itself unreproducible here, and the true worst
is 4× above it.

**Consequence for the headline claim**: the measured release improvement in the
worst case is **26.6 ms → 10.9 ms (~2.4×)**, not the *"21 ms → 2.8 ms"* (7.5×)
the docs state.

**Grading.** §7.1's (a) ∧ (b) ∧ (c): (a) module `//!`, public rustdoc,
implementation comment and a test doc comment; (b) the sub-threshold late path
is the ordinary path for any declared length under 32 MiB; (c) a later author
sizing a receive loop's worst case acts on 2.8 ms and gets 13 ms.

**Conflict declared, as §7.1 requires.** (d) fires — *"a shipped quantity that a
later author would act on which you cannot reproduce from the shipped text
alone"* — and so does the false-premise rule, since the surrounding conclusion
(*"a ceiling, not an elimination"*) is true and measured. §7.1 records that
pairing as **open**, with `docs/work/riders-21/attempts/C1/review.md` F1 as the
precedent that returned blocking. **(a) ∧ (b) ∧ (c) is applied**, on riders-21
F1's ground and on the ground the C3 review already used in this slice: the
2.8 ms is not a reason offered for the ceiling conclusion, **it is the
conclusion** — C4 exists to replace *"no work is done"* with a true bound, and
the bound is the deliverable.

**The input that breaks it.** A peer that opens a connection, writes only the
twelve-byte prefix declaring **32 505 856 bytes** (any length in ≈[128 KiB,
32 MiB) does it), sends nothing more, and a caller whose deadline has already
expired. On this host that single `recv_record` costs up to **13.37 ms** in
release and **11.14 ms** in debug. The builder pre-committed in report §8: *"If
a reviewer produces a cell above 2.8 ms I would treat the sentence as wrong, not
the cell."*

## MF1 — *"up to 4.2× at 31 MiB with a reused buffer"* does not reproduce

`src/imagery/net.rs:374`, and the same 4.2× at `tests/imagery_net.rs:1148`, the
`2.8/3.3 ms` cell at `:366`, report §1.2 and the commit message.

Five paired runs, 31 MiB, reuse, release, median of 15 each (base / C4 µs):
**760/651, 680/669, 672/691, 677/733, 666/670** — ratio **0.86–1.08**. On the
60-rep ladder: base 700.6 vs C4 689.4. Fresh and reuse agree at that length.
**There is no 4.2×, and the 2.803 ms cell it rests on does not exist here.**

The conclusion it supports — that there *is* a sub-threshold regression — is
true and was reproduced: **4 MiB, 2.6–3.3×**. False premise under a true
measured conclusion; (d) also fires, and *"where (d) and the false-premise rule
both fire, the false-premise rule governs and the reviewer must say so"* —
**said: must-fix.**

## MF2 — a ceiling in public rustdoc that its own file and the measurement exceed

`src/imagery/net.rs:246` (**public rustdoc**) — *"below the allocator's
threshold for handing back fresh memory there is no saving, and **up to a 2.4×
cost**."* The implementation comment 128 lines below says **2.4–2.6×**.

Four paired runs at 4 MiB, reuse, release, median of 21 (base / C4 µs):
**50.8/166.3, 47.9/151.1, 49.7/142.3, 52.2/140.5** — **2.7–3.3×**. An earlier
quiet-host pair gives 2.6×. At 1 MiB, **1.4–1.6×**, a band the docs give no
figure for.

§7.1's mirror-case bullet was considered — *"A quantity a reader can refute from
the document itself … is the safe kind"* — and does not cover this: the
refutation is external as well as internal, and *"up to"* is a ceiling word in
public rustdoc. False-premise rule: **must-fix**.

## MF3 — the test the comment says *"pins the bound"* cannot see the bound

`src/imagery/net.rs:378`. The test's budget is `small * 50 + 10 ms` =
**10.30 ms** and it asserts on a **median of nine** per cell. The candidate's
own worst *single* call at 31 MiB is **10.5–13.4 ms** — above that budget. Only
the median hides it. It pins neither the 2.8 ms the docs publish nor the ~11 ms
that is true; it pins *"no cell's median exceeds 10 ms"*. Lens 5 — *"Would the
test fail if the code were wrong?"* — it would not fail if the docs' ceiling
were wrong by 4×, which is the state it shipped in.

**Otherwise the test is sound and discriminates for the right reason.** Base
fails in both profiles, both reproduced: release **21.999 ms vs a 10.300 ms
budget** (builder: 20.235 / 10.287), debug **216.925 ms vs 20.317 ms** (builder:
201.293 / 19.360). A2 kills it and A1 does not; the narrowing test does the
reverse; both ablations reproduce cell-for-cell (**22/2**, **23/1**). The
candidate passes **25/25** consecutive runs and **12/12** under 12-way
concurrency. The builder's *"cannot be made to fail at a sub-threshold length in
release"* is correct and correctly reasoned — and it is **not** true in debug,
where the base blows the budget at 31 MiB (97.7 ms) and 32 MiB (107.7 ms). The
report says so and declines a debug-only test, which is the right call.

## Notes

- **N1** — base-side margin confirmed thin: 21.999 / 10.300 = **2.13×**, and the
  10 ms floor does not scale with host memory speed, so a ~2× faster host lets
  the base pass. Disclosed in §8; recorded, not escalated.
- **N2 — 98 304 bytes confirmed to the byte.** 30 of 30 reps report
  `queued=98304`, consistent with this host's `tcp_rmem` default of 131072
  (3/4 = 98304). The narrowing test's construction is sound.
- **N3** — *"at most one pass over the declared length"* (public rustdoc): in
  the sub-threshold case the pages are zeroed by the kernel on fault **and**
  memset by `calloc` — 31 MiB in 11 ms is 2.8 GB/s against the ~46 GB/s
  single-pass rate this host shows at steady state, i.e. two passes' worth of
  work charged to this call. Defensible as written, and the weakest part of the
  libc-independent bound.
- **N4** — *"against 21 ms and 202 ms"*: the base measures **24.2 ms** (max
  26.6 ms) and **206–217 ms** debug. Understates the base, the conservative
  direction.
- **N5** — report §1.1 says the measured trees are `2c3041f` and `987f2ec` while
  §1.2's columns are headed "C4". Harmless — the diff is comments and rustdoc
  only — but worth fixing in the record.
- **N6** — census verified: `2c3041f` carries the old sentence, `c5d1cce` the
  new. Nothing false ships; **seven** is right.
- **N7** — commit message **3349 bytes**, sha256 `a1bf5a6b…`, exactly as the
  brief asked.

## Every other published number reproduces

fmt rc 0 · clippy rc 0, 0 warnings · **183 / 0 over 16 targets** with every
per-target cell identical · **99 / 0** no-features · **BASE comparison 180 / 3**,
the three being exactly the axis test, the re-pointed cost test and the new
narrowing test · **A1 22/2, A2 23/1** with exactly the named tests · all six
features rows **127/121/101/101/179/103** · all **16** feature-docs rows rc 0 ·
clean extraction 14 files in `tests/`, 0 `zz_` · 2 files, +235/−38, no
`docs/work/` in the tree, no `Co-Authored-By`.

## Shape of C5

Re-derive the ceiling with **max, not median**, over enough reps to see the
tail, and publish it — or drop the numeric ceiling and keep only what the test
can pin. Strike the 4.2× and the `2.8/3.3 ms` table cell. Correct *"up to a
2.4×"* to the measured band. Either make the cost test assert on the worst cell
rather than the median, or stop claiming it *"pins the bound"*. **The trade
itself is fine and the code need not change.**
