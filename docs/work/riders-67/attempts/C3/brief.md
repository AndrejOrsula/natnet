# riders-67 C3 — bound what the socket decides, and sweep the sentences your change makes false

**Slice:** riders-67, attempt **C3**. Repository **`natnet`**. **Base =
`candidate(C2)` `2c3041f`** (tree `54ea58d`) — resolve it yourself. Branch
`slice/riders-67-C3`, worktree `/home/andrej/ws_rs/natnet_wt/impl-riders-67-C3`.
Freeze **C3**. Access `remote`.

Read `docs/work/riders-67/attempts/C2/review.md` in full first — **B1 first** —
then C2's `report.md`, then `attempts/C2/brief.md` **knowing its §6 is defective
and named as such in the review's N5**.

## 1. What is settled

**R6 is the right idea and R1–R5 are correctly rejected.** The reviewer says so
explicitly and re-derived the reasoning. Do not re-open the choice.

Also confirmed: the socket-mode restore is **safe by construction** — every
route that could leave `O_NONBLOCK` set returns `Err`, both call sites route
`Err` to `fail()`, a terminal client never touches the stream again, and
`stream` is private. There is **no spin**. The 1 ns discriminator reproduces
independently. And the **2 ms floor is a platform property, not a `std`
artifact** — requested 100 µs and 1 ms also return a 2 ms median, so it is
`CONFIG_HZ=1000` jiffy granularity, and R5's rejection is understated rather
than doubtful.

## 2. What is blocking

**The repair removed the bound it was protecting.** Two sentences C2 did *not*
rewrite now say something false, and they were true on `e157774`:
the module `//!`'s *"absolute deadline for the whole record … receiving a byte
does not restart it"*, and `recv_record`'s *"the deadline covers the complete
record and cannot be extended by fragmented or slow delivery."*

**B1a** — past the deadline the loop top no longer fails, so control always
reaches `buf.resize(total, 0)` with `total` peer-chosen. Twelve bytes declaring
`MAX_RECORD_LEN` buy **22 ms release / 213 ms debug** under a one-nanosecond
deadline, against microseconds on the base, and the caller gets `TimedOut` for
it. Linear in the declared length.

**B1b** — `take_arrived` does not bound the drain to bytes that had *already
arrived*, so a peer refilling the queue inside the reader's syscall gap keeps it
going. A 4 MiB frame with **1 572 916 bytes not yet written** when the deadline
expired was delivered in full, **4 times in 43**, against **0 in 20** on the
base — and the outcome is now scheduler-dependent where the base was
deterministic.

## 3. The repairs, and what is yours to decide

The reviewer named three and **applied the first itself**: break with `TimedOut`
in `take_arrived` on the first read returning fewer bytes than requested, since
on a stream socket that read emptied the queue. It measured **all 180 shipped
tests still passing and B1b at 0/20**.

**That is a suggestion with a measurement attached, not a specification.** Take
it, improve it, or replace it — but if you take it, say why "fewer bytes than
requested" is the right predicate and what it costs when a legitimate record
arrives split across exactly that boundary. If you replace it, show B1b at 0 in
at least the reviewer's sample size.

For **B1a**, the reviewer's shape is: do not zero-fill a peer-declared length on
a call whose deadline has already passed. **Your §6 dropped that alternative and
its stated reason was refuted** (MF3: it answers the allocation's *size* where
the finding is about *time past the deadline*). Decide it again on the right
question.

## 4. The ablation axis that does not exist

**A still-sending peer past the deadline must be a timeout.** Nothing in the
suite holds it — `byte_drip_cannot_restart_the_total_record_deadline` is
literally this property and passes only because its drip is 40 ms per byte.
Build that axis and show it fails without your bound.

## 5. My brief's defect, which is causal and which you inherit the fix for

C2's brief §6 named a **closed list of three prose locations** and asked only
*"which sentences did C1 ship that are false?"* It never asked *"which true
sentences does my repair falsify?"* — and B1's two sentences are exactly that
second kind, so **neither my sweep shape nor yours could have returned them**.

**So C3 owes a sweep in the other direction**: enumerate the sentences in the
touched files that were **true on the base** and ask, of each, whether your
change makes it false. Report the population and the query, not just the
verdict. This is the instrument the slice has been missing for two attempts.

## 6. Three report-side corrections

MF1 (*"It does not"* — a false conclusion over a population of nine rows that
all use a silent peer and a few-kilobyte total, and the same sentence is in the
commit message as *"Measured, this costs no extra waiting"*, so your C3 message
carries the retraction), MF2 (the candidate fails clause two of the property the
report adopts as **P**), MF3 (above). Do not reproduce them.

## 7. Practical

- **Assume you will be interrupted.** Draft `report.md` early, commit at each
  milestone. Four runs this session were cut off mid-work.
- **Do not commit to `docs/work/`.** Drafts under
  `docs/work/riders-67/attempts/C3/`, uncommitted. **Never push.** No
  `codebase-audit-docs/` edit, no frozen packet edit.
- `export CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c3/target`.
- Every gate of `natnet`'s workflow with its numbers.

## 8. Acceptance

- **B1a and B1b closed**, each with the reviewer's own instrument re-run against
  your candidate: the twelve-byte `MAX_RECORD_LEN` timing, and the refilling-peer
  delivery count at the reviewer's sample size or larger.
- The §4 axis, failing without your bound.
- The §5 reverse sweep, with its population and query.
- The three report-side corrections, and the retraction in your commit message.
- Every gate with its numbers; `candidate(C3)` SHA and tree; what you are unsure
  about.
