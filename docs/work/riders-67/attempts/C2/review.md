# riders-67 C2 review — `natnet`, candidate `2c3041f`, tree `54ea58d`

Re-resolved by the reviewer: `candidate/riders-67-C2` and `slice/riders-67-C2`
both → `2c3041f890481f2df8790de78a03258170149bd8`, tree
`54ea58d23c23ff01cdb5dd6202916636de044eb8`, parent `e157774`, one commit, 2
files, +291 −50. `riders-67-C2-checkpoint` → `73dfc2b56b21…`, a real commit.
Nothing in `natnet`, the builder's worktree or the evidence branch was touched.

**This review does not return only report-side findings.** B1 lands on two
shipped lines of `src/imagery/net.rs`.

## BLOCKING

### B1 — the record timeout no longer bounds `recv_record`, and two doc sentences this commit *retained* say it does

The locus is two sentences C2 did **not** rewrite, sitting inside the two doc
blocks it did: the module `//!` *"The timeout is an **absolute deadline for the
whole record**, including all prefix, header, and payload bytes; receiving a
byte does not restart it"*, and `recv_record`'s rendered rustdoc *"the deadline
covers the complete record and cannot be extended by fragmented or slow
delivery."* **Both were true on `e157774`.** Two independent mechanisms falsify
them.

**B1a, deterministic.** Past the deadline the loop top no longer fails, so
control always reaches `buf.resize(total, 0)` with `total` peer-chosen. A peer
sends **twelve bytes** — `MAGIC ‖ 99u16 ‖ 0u16 ‖ 67108852u32`, exactly
`MAX_RECORD_LEN` — and nothing else. `recv_record(&mut buf,
Duration::from_nanos(1))`, 5 reps:

| tree | debug | release |
| --- | --- | --- |
| `e157774` | 5.25 / 3.78 / 3.64 / 3.89 / 3.59 **µs** | 5.37 / 3.55 / 3.31 / 3.31 / 3.28 **µs** |
| `2c3041f` | **214.96 / 213.52 / 212.75 / 212.19 / 212.81 ms** | **22.28 / 20.56 / 25.15 / 22.01 / 19.96 ms** |

Twelve bytes buy 22 ms (release) / 213 ms (debug) under a one-nanosecond
deadline, and the caller gets `TimedOut` for it. Linear in the declared length,
confirmed at 1/4/16 MiB.

**B1b, reproducible.** `take_arrived` does not bound the drain to bytes that had
arrived, so a peer refilling the queue within the reader's syscall gap keeps it
going. A 4 MiB `ImageFrame` with **1 572 916 bytes not yet written by the peer**
when the deadline expired was delivered in full — 2/20 debug, 1/20 release, plus
1/3 in an earlier run: **4 in 43**. Base 0/20. It also makes the outcome
scheduler-dependent where the base was deterministic.
`tests/imagery_net.rs:637 byte_drip_cannot_restart_the_total_record_deadline`
is literally this property; it passes only because its drip is 40 ms per byte.

Graded on §7.1's *"(a) ∧ (b) ∧ (c)"*, read as §7.1 requires (*"not a flat
disjunction"*), with the rule declared as §7.1 demands since the false-premise
rule co-fires on MF1. **(b) stated honestly**: measured at 1 ns; the production
shape is "deadline already passed when the prefix is in hand", which any stall
spanning the deadline produces at any timeout — **strictly more reachable than
the window C1 repaired, since it needs no window.**

## MUST-FIX

- **MF1** — `report.md` §2's *"It does not"* (the drain does not make
  `recv_record` slower to give up) is a false conclusion over a population that
  cannot contain the refuting case: all nine rows use a peer that will never
  send again **and** a few-kilobyte declared total. Clause list applied →
  must-fix (report-only). The same sentence is in the commit message as
  *"Measured, this costs no extra waiting"*.
- **MF2** — the candidate fails the second clause of the property the brief
  states and the report adopts as **P** (*"a record still missing bytes when the
  deadline passes fails"*). Clause list applied.
- **MF3** — `report.md` §6 reason 2 (*"It would bound nothing. The allocation is
  already bounded by `MAX_RECORD_LEN`"*) substitutes the allocation's **size**
  for the **time spent past the deadline**, which is what N4 and brief §4
  raised. False-premise rule applied.

## Notes — the three pressure points

- **N1 — the restore-failure arm is safe by construction.** Every route that
  could leave `O_NONBLOCK` set returns `Err` from `read_until`; both call sites
  route `Err` to `fail()` → terminal; a terminal `recv_record` returns without
  touching the stream; `stream` is private and never handed back. **No path
  leaves a *usable* client non-blocking.** One real note: `from_stream` adopts a
  caller's `TcpStream`, and `O_NONBLOCK` lives on the open file description, so
  a retained `try_clone()` sees the mode flip — `from_stream`'s rustdoc does not
  say so.
- **N2 — no spin.** Bounded by `destination.len()`; every pass advances or
  breaks.
- **N3 — the 2 ms floor reproduces and is a platform property.** The builder's
  three rows reproduce exactly. The two extra rows are the informative ones:
  requested **100 µs** and **1 ms** also return a 2 ms median, so it is
  `CONFIG_HZ=1000` jiffy granularity, not a `std` `timeval` artifact. On
  `CONFIG_HZ_250` kernels it is worse. **R5's rejection stands and is
  understated.**
- **N4** — the 1 ns discriminator reproduces independently: base 0/100,
  candidate 100/100 for payloads 0, 1, 4096.
- **N5 — the C2 brief, four items.** §6 names a **closed** list of three prose
  locations and only ever asks *"which sentences did C1 ship that are false?"*,
  never *"which true sentences does my repair falsify?"* — **neither the
  brief's sweep shape nor the builder's could return B1's two retained
  sentences, and that is causal.** §4's *"do not weaken the deadline"* does not
  distinguish *peer silent* from *peer still sending*, and only the first is
  tested. §1's *"provably equivalent"* cites a review that graded it *"Negative,
  with its population"*. §1's *"Every gate number in C1's packet reproduced"* is
  contradicted by that review's N6, which names two `gates.md` items it could
  not re-derive. And §2's digest travels without its population — the 200-call
  table tested one timeout; the every-timeout sweep was 100 calls.

## Gates

Everything re-derived on a clean `git archive` extraction in isolated target
dirs. Reproduced exactly: fmt rc 0 · clippy rc 0 / 0 warnings · `--all-features`
**180/0 over 16 targets** and the per-target table cell by cell (5, 23, 7, 11,
28, 6, **21**, 12, 11, 6, 4, 6, 9, 17, 4, 10 = 180) · `--locked` 99/0 · MSRV
`+1.92` 99 and 180 · **all six `features` rows** (127/121/101/101/**176**/103) ·
**all 16 `feature-docs` rows** rc 0/0 · `cargo-deny` rc 0 with exactly the two
pre-existing warnings · clean extraction 14 test files, 0 `zz_`, 180/0 · the §5
flattened sweep row for row · the probe sha256 · BASE 177/3/180 with the three
named failures, **A1** identical to BASE, **A2** the three lateness tests,
**A3** exactly one — all with one `Compiling natnet`.

Could not reach: the builder's per-cell timing medians and 200-call cells
(host/instant-bound — the same axes were re-derived and the delivery counts
agree, the latency conclusion does not), the `site`-labelled diagnostic tree,
the warm NULL controls (built cold), and `report.md` §10's `git status` in the
builder's worktree, which the reviewer was told not to touch.

**One self-correction, recorded.** The first pass attributed the candidate's
extra wall time to the drain loop. It is dominated by the `debug` zero-fill;
`p7` and the release re-runs separate them, and **B1 is written from the
separated instrument.**

## Verdict: reject

§9 admits only an independent `accept` with no unresolved blocking finding.

**R6 is the right idea and R1–R5 are correctly rejected**; what is missing is a
*bound* on what happens once the decision moves onto the socket. Cheapest C3:

1. In `take_arrived`, break with `TimedOut` on the first read returning fewer
   bytes than requested — on a stream socket that read emptied the queue. **This
   patch was applied and run: all 180 shipped tests still pass and B1b goes to
   0/20**, which is also the measure of how little the suite discriminates here.
2. For B1a, do not zero-fill a peer-declared length on a call whose deadline has
   already passed — §6's own dropped alternative.
3. Add the ablation axis that does not exist: **a still-sending peer past the
   deadline must be a timeout.**
