# riders-67 C2 — close the class, not the instance

**Slice:** riders-67, attempt **C2**. Repository **`natnet`**. **Base =
`candidate(C1)` `e157774`** (tree `1a1c9bc`), not `dev` — resolve it yourself
and say what you got. Branch `slice/riders-67-C2`, worktree
`/home/andrej/ws_rs/natnet_wt/impl-riders-67-C2`. Freeze **C2**. Access
`remote`.

**C1 was rejected on a blocking finding.** Read, in this order:
`docs/work/riders-67/attempts/C1/review.md` (the whole thing — B1 first),
then C1's `report.md`, then the original `docs/work/riders-67/brief.md`
**knowing that three of its sentences are defective and named as such in the
review's N5**. Process rules: `/home/andrej/ws_rs/auga/AGENTS.md` §4, §5, §6,
§7. Gates are `natnet`'s own.

## 1. What C1 got right, so you do not undo it

Within `read_until`, deleting the mid-read deadline check is **provably
equivalent** to guarding it — the reviewer attacked that independently and could
not break it, and the difference is exactly `{destination completed}`. The new
test's flake bound holds under construction: with the window made unreachable
the test **fails loudly** and names the cause. The mutant substitution is sound.
Every gate number in C1's packet reproduced.

**None of that is in question. Do not re-litigate it and do not revert it.**

## 2. What is blocking

`recv_record` calls `read_until` **twice with one deadline** — once for the
12-byte prefix, once for the body. C1's change lives inside `read_until`, so a
prefix that completes at or after the deadline now falls through `record_len`,
the `MAX_RECORD_LEN` check and `buf.resize(total, 0)` into the **body** call,
whose loop-top check fires at once: buffer cleared, socket shut down, client
terminal — with every byte already in the kernel receive queue.

Measured by the reviewer, 200 calls per cell, base against candidate: the
**bodied rows are identical at every timeout tested**. The repair's entire
measured effect is on records with an empty body — `Heartbeat` and `Bye`, two
of six variants. `ImageFrame`, the record the crate exists for, is untouched,
and the module `//!` now asserts a universal over all of them.

I confirmed the two-call structure myself before dispatching this.

## 3. The property to achieve — and I am not handing you a choice set again

**My C1 brief offered exactly two repairs and both of them leave bodied records
as broken as the base.** That was the defect that produced this attempt, and
repeating it would produce another. So this brief states a **property** and
leaves the mechanism to you:

> A record every byte of which has arrived is delivered, **whatever its kind and
> whatever its length**; a record still missing bytes when the deadline passes
> fails, and only that is a timeout.

**Enumerate the repairs that could achieve it before choosing one**, and say
why the one you pick closes the *class* rather than one more member of it. The
reviewer named two possibilities — carrying "is the record still outstanding?"
across both calls, or checking the deadline once at the top of `recv_record` —
and **those are examples, not the set**. If your enumeration finds a third,
that is the outcome I would most like.

Ask of each candidate repair: **what is the next call site that would break it?**
The defect survived C1 because a fix correct about `read_until` was assumed
correct about its caller.

## 4. Two things the repair must not quietly do

- **`MAX_RECORD_LEN` is `64 << 20`.** Review N4: C1 moved the 64 MiB
  zero-filling `resize` inside the deadline window, where the base returned
  before reaching it. One per connection, since receive failures are terminal.
  Decide whether your repair should order the deadline check before the resize,
  and say what you decided either way.
- **Do not weaken the deadline.** A record genuinely missing bytes must still
  fail and the client must still become terminal. Hold that with its own test,
  and show the test fails if you break your own fix.

## 5. The instrument

The reviewer wrote a bodied-record instrument and left it at
`/home/andrej/coord_scratch/review67/work/tests/zz_review67.rs` and
`zz_sweep67.rs`, with base, mutant and window-closed trees at
`/home/andrej/coord_scratch/review67/{base,mutant,nowindow,pre}`.

**Read them. Do not paste them in.** Re-derive the measurement yourself and
report your own numbers; if yours disagree with the review's table, say so and
yours are the ones that count. What must end up in the suite is a test that
**fails on `e157774` and passes on your candidate, for a record with a
non-empty body** — the coverage whose absence is the whole finding.

## 6. The prose is part of the defect

Three shipped locations state the false contract: the module `//!`
(`src/imagery/net.rs:44-49`), `recv_record`'s rustdoc (`:192-194`), and the
comment that replaced the deleted block (`:319-330`). **Rewrite all three to
say what is true of your candidate.** The comment is the most dangerous of the
three, because it sits exactly where the next reader looks and currently tells
them the class is closed.

**A sentence that is true only of the empty-body case must say so.**

## 7. Report-side must-fixes carried from C1 — do not reproduce them

MF1, MF2 and MF3 are all false premises under true conclusions in C1's report:
the "same `read_until` call with a different `destination`" claim, a
`git grep -i mutant` parenthetical whose pattern returns nothing at all in the
tree, and a `deadline|elapsed` query that adds two files rather than one and
returns nothing as literally transcribed (no `-E`). **Do not repeat them in
C2's report. Where you reuse a C1 sentence, re-run its instrument first.**

**MF4 is in C1's commit message and cannot be amended.** Per `AGENTS.md` §7
it is repaired in the next message: **your C2 commit message must state that
C1's subject line over-claims** — it delivers such a record only when the last
byte completes the final destination, i.e. only for two of six kinds.

## 8. Practical

- **Assume you will be interrupted.** Two sessions of this project have been
  cut off mid-run by a rate limit. **Draft `report.md` early and keep it
  current, and commit at each milestone** rather than once at the end. Your
  worktree is what I measure if you stop.
- **Do not commit to `docs/work/`.** Draft `report.md` and `gates.md` under
  `docs/work/riders-67/attempts/C2/`, uncommitted, for me.
- **Never push.** `export
  CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c2/target`.
- Gates: `natnet`'s own, every one, reported **with its numbers**. C1's
  `gates.md` is the format to follow — the reviewer re-derived it cell by cell
  and it held.

## 9. Acceptance

- The §3 enumeration, the choice, and why it closes the class.
- A test that **fails on `e157774` and passes on your candidate** for a bodied
  record, plus the still-fails-when-genuinely-late test, each with both runs.
- An ablation: break your own repair, show the test fails.
- The three prose locations rewritten and quoted in the report.
- The N4 allocation decision, either way, with its reason.
- Every gate with its numbers; `candidate(C2)` commit SHA and tree hash; and
  what you are unsure about.
