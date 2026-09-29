# riders-67 C5 — a ceiling measured with a median is not a ceiling

**Slice:** riders-67, attempt **C5**. Repository **`natnet`**. **Base =
`candidate(C4)` `c5d1cce`** (tree `495974a3`) — resolve it yourself. Branch
`slice/riders-67-C5`, worktree `/home/andrej/ws_rs/natnet_wt/impl-riders-67-C5`.
Freeze **C5**. Access `remote`.

Read `docs/work/riders-67/attempts/C4/review.md` in full first, **B1 first**,
then C4's `report.md`.

## 1. The code is right. Do not change it.

The reviewer's ruling, and mine: **the trade is acceptable and the regression is
not a defect.** The band is confined to roughly 1–8 MiB; its absolute worst is
**~110 µs added to a call that is already failing**; it buys a worst case
falling from **26.6 ms to 10.9 ms** release and **217 ms to 11 ms** debug. The
reviewer also tested the one alternative you did not — freeing the old
allocation first — and it does **not** help.

**This attempt changes numbers and sentences. If you find yourself editing the
allocation path, stop and report.**

## 2. What is blocking, and the one-word cause

**You measured a ceiling with a median of fifteen.** A median structurally
cannot see a worst case, and a ceiling is a claim about the worst case.

Over 180 timed calls at 31 MiB, release, reused buffer: **six cells exceed the
shipped 2.803 ms**, at 11 225 / 10 966 / 10 281 / 10 655 / 13 366 / 12 153 µs —
**max 13.37 ms, 4.8× the shipped ceiling**. Debug max **11.14 ms** against 3.3.
The ablation rules out harness noise: same probe, same connection churn, only
the declared length differs — 64 MiB on the `mmap` path maxes at **15–44 µs**,
31 MiB on the heap path reaches **13.4 ms**.

Four shipped locations carry it, one of them **public rustdoc**:
`src/imagery/net.rs:61`, `:243`, `:366`/`:370`, and
`tests/imagery_net.rs:1149-1151`.

**And the headline is wrong too**: the measured release improvement in the
worst case is **26.6 ms → 10.9 ms (~2.4×)**, not the *"21 ms → 2.8 ms"* (7.5×)
the docs state.

**You pre-committed to this outcome** in report §8: *"If a reviewer produces a
cell above 2.8 ms I would treat the sentence as wrong, not the cell."* Honour
it.

## 3. The choice, and I am not making it for you

Either **re-derive the ceiling with `max`, not a median, over enough reps to see
the tail, and publish that** — or **drop the numeric ceiling entirely and keep
only what a test can pin.** Both are acceptable; the second may be the stronger
answer, since a tail figure is a property of this host and the corpus has just
spent three attempts learning what host-bound numbers cost. Say which you chose
and why, and if you publish a number, **say what statistic it is and over how
many reps** in the same sentence.

## 4. Two numbers to strike, one band to correct

- **MF1 — the 4.2× does not reproduce.** Five paired runs at 31 MiB give
  **0.86–1.08**; the 60-rep ladder gives base 700.6 against C4 689.4. The
  `2.803 ms` cell it rests on does not exist on the reviewer's host. **The
  sub-threshold regression it was offered as evidence for is real** — 4 MiB,
  2.6–3.3× — so strike the number, keep the conclusion. It appears at
  `src/imagery/net.rs:374`, `tests/imagery_net.rs:1148`, `:366`, report §1.2
  **and your commit message**, so C5's message carries the retraction.
- **MF2 — *"up to a 2.4× cost"* in public rustdoc** (`:246`) is exceeded by its
  own file 128 lines later (2.4–2.6×) and by measurement (**2.7–3.3×** at
  4 MiB). There is also a **1.4–1.6× band at 1 MiB** that the docs give no
  figure for. Correct to the measured band, or stop bounding it numerically.

## 5. MF3 — the test does not pin what its comment says it pins

`src/imagery/net.rs:378` says the cost test *"pins the bound over the whole
range and both buffer disciplines"*. Its budget is 10.30 ms and it asserts on a
**median of nine**, while the candidate's own worst single call at 31 MiB is
**10.5–13.4 ms**. It would not have failed if the shipped ceiling were wrong by
4×, **which is the state it shipped in**.

**Either make it assert on the worst cell, or stop claiming it pins the bound.**
If you make it assert on the worst cell, say what that does to its flake margin
— the reviewer measured the base-side margin at only **2.13×** already.

**Everything else about that test is sound and was re-derived**: base fails in
both profiles, A1/A2 kill exactly the right tests (22/2 and 23/1), 25/25
consecutive and 12/12 under 12-way concurrency. Leave that alone.

## 6. Three notes

- **N3** — *"at most one pass over the declared length"* is the weakest part of
  your libc-independent bound: sub-threshold, the kernel zeroes on fault **and**
  `calloc` memsets, which the reviewer measures as two passes' work charged to
  this call. Defensible as written; decide whether you want to defend it.
- **N4** — the base is **24.2 ms** (max 26.6) and 206–217 ms, against your
  *"21 ms and 202 ms"*. You understate the base, which is the conservative
  direction; correct it anyway.
- **N5** — report §1.1 names `987f2ec` as a measured tree while §1.2's columns
  say "C4". Harmless, and worth fixing in the record.

## 7. Practical

- **Assume you will be interrupted.** Draft `report.md` early, commit at each
  milestone. This session has lost many runs to rate limits.
- **Do not commit to `docs/work/`.** Drafts under
  `docs/work/riders-67/attempts/C5/`, uncommitted. **Never push.** No
  `codebase-audit-docs/` edit, no frozen packet edit.
- `export CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c5/target`.
- Every gate of `natnet`'s workflow with its numbers.

## 8. Acceptance

- The ceiling either re-derived with `max` and published with its statistic and
  rep count, or removed — with the reason.
- The 4.2× struck everywhere including a retraction in your commit message; the
  2.4× corrected to the measured band.
- The cost test's comment and its assertion agreeing.
- N3 decided, N4 and N5 corrected.
- Every gate with its numbers; `candidate(C5)` SHA and tree; what you are unsure
  about.
