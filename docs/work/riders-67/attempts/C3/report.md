# riders-67 C3 — builder report

Repository `natnet`. Branch `slice/riders-67-C3`, worktree
`/home/andrej/ws_rs/natnet_wt/impl-riders-67-C3`.

## 0. Identity

| | |
|---|---|
| Base | `candidate(C2)` = `2c3041f890481f2df8790de78a03258170149bd8`, tree `54ea58d23c23ff01cdb5dd6202916636de044eb8`, subject *"fix(imagery/net): finish a record from bytes already in the queue"* |
| Base resolved | by me at **2026-09-13T20:48:11+02:00**, `git -C <worktree> rev-parse HEAD` and `rev-parse HEAD^{tree}`. Both match the C3 brief and the C2 review's own re-resolution. Not `dev` |
| `candidate(C3)` SHA | **`987f2ecc1d66a01ad80346efbf1a2bc8faf3ff90`** |
| `candidate(C3)` tree | **`bf191d39d468431e985b55c228071a4890beb09c`** |
| `candidate(C3)` parent | `2c3041f890481f2df8790de78a03258170149bd8` — one commit on the base, not a rebase of `dev` |
| Files changed | `src/imagery/net.rs`, `tests/imagery_net.rs` (2 files, **+343 −42**) |
| Commit message file | `/home/andrej/coord_scratch/riders67c3/msg.txt`, committed with `git commit -F` |
| Checkpoints | three, squashed; preserved reachable as tag **`riders-67-C3-checkpoint`** → `ab1b749`, whose tree is `bf191d3` — identical to the candidate's |
| Pushed | no. No `git push` was run in this slice, to any remote |

Toolchain `rustc 1.97.1 (8bab26f4f 2026-07-14)` / `cargo 1.97.1 (c980f4866
2026-06-30)` via `rust-toolchain.toml` (`channel = "stable"`). MSRV rows on
`rustc 1.92.0 (ded5c06cf 2025-12-08)` / `cargo 1.92.0 (344c4567c 2025-10-21)`.
Every cargo call ran with
`CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c3/target` (or a
per-tree sibling for the comparison runs) and `/home/andrej/.cargo/bin/cargo`.

**Interrupted once by a session rate limit** and resumed. Nothing was lost:
the probe was on disk and the coordinator preserved it at
`/home/andrej/coord_preserve/riders-67-C3-killed/`. Three checkpoint commits
followed.

## 1. B1a and B1b reproduced on the base, before anything was changed

My own instrument, `tests/zz_c3_probe.rs` — same construction as the
reviewer's (`peek`-confirm, 1 ns deadline, 43 reps), my own code, and two
cells and one snapshot the review did not have. It **does not ship**; see §9.

### B1a — the cost past an expired deadline is linear in a length the peer declares

Twelve bytes and nothing else: `MAGIC ‖ 99u16 ‖ 0u16 ‖ <declared>u32`,
`peek`-confirmed present, then `recv_record(&mut buf, Duration::from_nanos(1))`.
Five reps per row, each cell a separate connection.

| declared total | base `2c3041f` debug | base `2c3041f` release | outcome |
|---|---|---|---|
| `MAX_RECORD_LEN`, 64 MiB | 205.074 / 202.632 / 201.512 / 202.277 / 200.691 **ms** | 22.674 / 22.167 / 21.406 / 20.923 / 20.399 **ms** | `TimedOut` |
| 16 MiB | 50.056 / 49.894 / 47.154 / 46.803 / 48.783 **ms** | 4.696 / 4.757 / 0.365 / 0.363 / 0.355 **ms** | `TimedOut` |
| 4 MiB | 11.694 / 11.617 / 11.785 / 11.691 / 11.735 **ms** | 0.103 / 0.038 / 0.038 / 0.038 / 0.038 **ms** | `TimedOut` |
| 64 KiB | 0.193 / 0.196 / 0.195 / 0.190 / 0.183 **ms** | 0.006 **ms** ×5 | `TimedOut` |
| 0 B | 0.002 **ms** ×5 | 0.001 **ms** ×5 | `Ok(true)` |

The review's 213 ms debug and 22 ms release cells reproduce, over five
declared lengths rather than three. The release 16/4 MiB rows are not
monotonic, which is the allocator's mmap threshold moving after the 64 MiB
row frees — I say so because it is the same mechanism the repair uses, and
§2.2 is honest about its limit rather than resting on the tidy rows.

### B1b — a peer still sending past the deadline is delivered in full

A 4 MiB payload. The prefix is written and `peek`-confirmed; a writer thread
then pushes the payload in 64 KiB blocks publishing its progress in an
`AtomicUsize`; the test snapshots that counter immediately before calling
`recv_record(&mut buf, Duration::from_nanos(1))`, so the snapshot is an upper
bound on bytes handed over while the deadline was still live.

```
C3B1B rep=0  DELIVERED buf=4194316 missing_at_deadline=4194304
C3B1B rep=3  DELIVERED buf=4194316 missing_at_deadline=4194304
C3B1B rep=11 DELIVERED buf=4194316 missing_at_deadline=4194304
C3B1B total=4194316 payload=4194304 reps=43 delivered_in_full=3 worst_missing=4194304
```

**3 in 43 in `debug`, 0 in 43 in `release`,** with the *entire* 4 MiB payload
unwritten when the deadline expired. The review measured 4 in 43 with
1 572 916 unwritten. **I agree with the review's finding and my release column
is weaker than its 1/20** — the effect is scheduler-dependent, which is itself
part of the finding, and §3 builds a construction that removes the race.

## 2. The bound, and why this predicate

Two mechanisms were open, so there are two bounds. Both are in
`src/imagery/net.rs`; neither touches the live path.

### 2.1 B1b — `take_arrived` finishes its destination with one read

The reviewer's suggestion was *"break with `TimedOut` on the first read
returning fewer bytes than requested"*. **I took it, and restated it as the
property it actually is**: past the deadline there is **one** non-blocking
read per destination, and never a second.

That is not a rewording. Under the reviewer's phrasing the loop survives and
a reader must reason about when it exits; under "one read" the loop is gone —
the only remaining iteration is the `Interrupted` arm, which retries a read
that transferred **nothing at all**, so it cannot collect a byte. There is
nothing left for a peer to drive, which is why §3's axis is deterministic
rather than a 3-in-43 race.

**Why "fewer bytes than requested" is the right predicate.** The read asks for
the *whole* remaining destination. On a stream socket a read that returns
fewer bytes than it asked for has emptied the receive queue — `tcp_recvmsg`
copies until the requested length is satisfied or it runs out — so a short
return is a positive fact, not an absence: *the queue is now empty*. Every
byte after it is a byte that crossed into the queue **after** the deadline had
already passed, and a deadline whose meaning is "do not wait" cannot also mean
"collect what turns up next". The predicate is therefore exactly the line the
deadline draws, not a proxy for it.

**What it costs, stated rather than assumed.** Two things, and I found the
second only by running §4's sweep on my own change.

1. *A record whose bytes had all arrived but whose copy the kernel splits.*
   On Linux a TCP read returns short only when the queue is exhausted or a
   signal interrupts a copy that had already moved bytes. The signal case
   would turn a fully-queued record into a timeout. I did not construct one
   and I am not claiming it cannot happen; I am claiming it is the only case
   I could identify, and that on 40 consecutive suite runs (§7) no delivery
   test saw a short read.
2. *A record longer than the receive queue will hold at once is no longer
   completed after its deadline.* Bytes the peer handed to the kernel but
   which TCP flow control has parked in the **sender's** buffer are not in the
   receive queue, and collecting them means opening the window and waiting for
   the transfer — which is waiting. This is a real narrowing against C2 and it
   is stated in the module docs and in `recv_record`'s rustdoc rather than
   left for a reader to discover. It is still strictly better than `e157774`,
   which completed **no** bodied record late at all.

I measured the connection's capacity rather than assuming it, with two
instruments, and the two numbers below come from different ones. Standalone
(`/home/andrej/coord_scratch/riders67c3/rcvwin.rs`, one process, both arms):
cold, this host absorbs **2 612 608** bytes with nothing read, of which only
**112 704** sit in the receive queue; after one full-speed drain the queue
holds all **2 619 373** of them. In the shipped test's own helper, on the warm
connection it builds, the queue holds **3 929 404** (`release` run: 4 322 620)
— a different connection with a different autotuning history, which is why I
do not present these as one arrow. Both say the same thing: on a warm
connection — the state a record stream
is actually in — "in the receive queue" and "handed to the kernel" nearly
coincide, and the narrowing bites on the cold first record, not the stream.

### 2.2 B1a — the allocation decision, re-decided on the right question

C2's §6 refused to order a check before `buf.resize(total, 0)`, and its
reason 2 (*"it would bound nothing; the allocation is already bounded by
`MAX_RECORD_LEN`"*) was refuted by MF3 for answering the allocation's **size**
where the finding is about **time past the deadline**. Re-decided on that
question:

**Decision: past the deadline, do not *write* the zeroes for a peer-declared
length — ask the allocator for them.**

C2's §6 reason 1 still stands and rules out the obvious repair: a check that
*fails* before the resize would destroy a fully-queued bodied record at a new
boundary, and `a_fully_buffered_bodied_record_is_delivered_past_its_deadline`
would catch it — it does; ablation **A3** in §6 is that check, and it takes
down three delivery tests. So the destination must still be allocated. What
must not happen is paying for it:

```rust
buf.clear();
if Instant::now() < deadline {
    buf.extend_from_slice(&prefix);
    buf.resize(total, 0);
} else {
    let mut record = vec![0u8; total];
    record[..PREFIX_LEN].copy_from_slice(&prefix);
    *buf = record;
}
```

`vec![0u8; total]` is `alloc_zeroed`, which for a large request is a fresh
mapping of pages the kernel has already zeroed; the single read of §2.1 then
faults only the pages it fills. `resize` on a buffer that already has a length
cannot use it and stores every byte.

Measured, standalone, on this host (5 reps, min/max):

| | `vec![0u8; n]` | `clear + extend(12) + resize(n)` |
|---|---|---|
| n = 64 MiB, release | **0.020 / 0.080 µs** | 23 671 / 26 430 µs |
| n = 64 MiB, debug | **7.06 / 10.79 µs** | 152 118 / 156 840 µs |
| n = 64 MiB then touch 12 bytes, release | **0.020 / 0.031 µs** | — |

**And this is where the decision stops being tidy, which I would rather say
than have found for me.** The guarantee is the allocator's, not the language's.
Below glibc's mmap threshold `calloc` really does store the zeroes, and that
threshold moves at run time. It shows in the candidate's own rows, over
four runs of five reps in each profile against the frozen tree: 64 MiB costs
**0.009–0.019 ms** in steady state, but **4 MiB costs 0.091–0.154 ms** and one
16 MiB rep cost **1.236 ms**. The declared length that is cheapest is the
largest one, which is the allocator talking, not the code. So the honest claim
is **not** O(1). It is:

- the cost no longer grows with the declared length — the largest declaration
  is now the *cheapest* of the large rows;
- the worst case is bounded by a `memset` of `MAX_RECORD_LEN` with no page
  faults, which is the cost the live path has always paid for an accepted
  64 MiB record, and is ~1.5 ms at the 45 GB/s these rows imply;
- the measured 64 MiB case is **0.011–0.019 ms debug / 0.009–0.015 ms
  release** in steady state, against **205 ms / 22 ms** on the base.

**A correction to my own commit message, which cannot be amended.** It states
this cost as *"0.010 ms in `debug` and 0.009 ms in `release`"*. Those are two
single cells from the checkpoint build, and the debug one is below anything I
have since reproduced: over four runs of five reps against the frozen tree
`bf191d3`, debug ran 0.011–0.021 ms and release 0.009–0.023 ms, the first rep
of each run being the high end. The numbers in this section are the ones taken
against the frozen tree and they are the ones that count; the message's pair is
the same code measured once, quoted more precisely than a single sample can
carry. The three-order-of-magnitude claim the message rests on is unaffected.

A hard, allocator-independent bound would need either a cap on what may be
completed late — a magic number that silently refuses records a bigger-windowed
host could deliver — or reading into uninitialised memory, which needs `unsafe`
or unstable `read_buf` and the crate is `#![forbid(unsafe_code)]`. I chose the
measurable improvement with its limit written down over a constant I could not
derive. **This is the part of the repair I would most want a reviewer to
attack.**

### The candidate, same instruments

| | debug | release |
|---|---|---|
| B1a, 12 bytes declaring `MAX_RECORD_LEN`, 1 ns | 0.042 / 0.016 / 0.015 / 0.016 / **0.015 ms** | 0.030 / 0.013 / 0.009 / 0.009 / **0.009 ms** |
| B1b, 4 MiB refilling peer, 43 reps | **0 delivered in 43** | **0 in 43** |
| §3 axis, 43 reps | **0 delivered, 43 `TimedOut`** | **0 delivered, 43 `TimedOut`** |

Run against the frozen tree `bf191d3` extracted with `git archive`, with the
probe copied back in and removed again.

## 3. The ablation axis that did not exist

`a_peer_still_sending_past_the_deadline_is_a_timeout`, in
`tests/imagery_net.rs`.

`byte_drip_cannot_restart_the_total_record_deadline` states this property and
cannot test it: at 40 ms per byte any implementation finds the queue empty.
The axis needed a peer sending as fast as the kernel will take it — and my
first three attempts at that were the 3-in-43 race of §1, which is useless as
a test. Three things fixed it:

1. **Size the record against the host, not a constant.** `absorption_of`
   fills the connection until the kernel refuses more with nothing read, then
   drains it, and returns the count. The record declares **twice** that. So at
   the instant `recv_record` is called the peer *cannot* have handed the whole
   record over — the kernel had nowhere to put it — whatever the scheduler
   did. Any `Ok` is a record completed from post-deadline bytes. **The test
   cannot flake: its pass is guaranteed by construction, and a slow host makes
   the peer less far along, not more.**
2. **Warm the connection.** Calling `absorption_of` twice and keeping the
   second answer is what grows the receive window. Measured standalone on a
   single connection, the receive queue goes from **112 704** bytes cold to
   **2 619 373** warm, and `rounds = absorption / queue` from **23.2** to
   **1.0**. With a cold window the drain needs ~23 kernel refills to finish and
   loses the race; with a warm one it needs ~1, and wins it.
3. **One `write_all` for the body.** The peer parks *inside* the syscall once
   the buffers fill, so each refill is driven by the kernel as the reader
   drains rather than by the writer thread being rescheduled.

Measured on the base `2c3041f`: **43 of 43 reps delivered** a 7.86 MB record
over a connection that absorbs 3.93 MB unread. On the candidate: **0 of 43**,
in both profiles, over four separate runs. As a single shipped test on the
base it failed **29 of the 30 runs I measured** (10 alone, 5 in the full file,
5 in `release`, 10 with a two-test filter — the one pass was in that last
group). On the candidate it passed **40 of 40** full-file runs.

The second new test, `a_declared_length_costs_nothing_past_the_deadline`,
holds B1a. Twelve bytes declaring `MAX_RECORD_LEN` against twelve declaring
64 KiB, medians of nine, asserting `largest <= small * 50 + 10 ms`. **Both
rows time out on every tree**, so there is no outcome to flake; only the ratio
moves. On the base it fails with *"cost 212.492562ms against 197.349µs … a
peer must not be able to buy work by naming a length (allowed 19.86745ms)"*.

## 4. The reverse sweep — population and query

The C3 brief's §5, and the instrument this slice was missing. C2's brief asked
only *"which sentences did C1 ship that are false?"*; B1's two sentences were
true when they were written, so no forward sweep could return them.

**Population.** Every prose claim in the two touched files **at the base**
`2c3041f`, not only the ones I edited. Built mechanically, not by eye, by
`/home/andrej/coord_scratch/riders67c3/sweep/extract.py`, which for each file
at a named revision extracts three notations:

- every `//!`, `///` and `//` comment **block, flattened** (leaders and
  wrapping collapsed) and split into sentences — because corpus prose wraps
  mid-claim and a line-oriented read of a comment cannot see a whole sentence;
- every string literal of ≥12 characters containing prose — assertion and
  panic messages are claims, and C2's `"reject before payload allocation"` is
  one;
- every `#[test]` function **name** — a test name is a claim, and
  `byte_drip_cannot_restart_the_total_record_deadline` is the one the C3 brief
  itself calls *"literally this property"*.

```
src/imagery/net.rs   @ 2c3041f :  78 claims (71 comment sentences,  7 message strings,  0 test names)
tests/imagery_net.rs @ 2c3041f : 137 claims (42 comment sentences, 74 message strings, 21 test names)
TOTAL CLAIMS AT 2c3041f: 215
```

**Query.** Of each of the 215, in order, not filtered by keyword: *was this
true on `2c3041f`, and does my change make it false?* The listing is at
`/home/andrej/coord_scratch/riders67c3/sweep/base.txt`, one numbered line per
claim.

**Verdict: six falsified, all in `src/imagery/net.rs`.** Five of the six I had
not otherwise touched, and none of them is a sentence B1 named.

| # | claim, at `2c3041f` | why my change falsifies it |
|---|---|---|
| net.rs:16 | *"every remaining read of that record is issued non-blocking"* | there is now at most **one** remaining read per destination |
| net.rs:16 | *"delivered **whatever its kind and however long it is** — a `Heartbeat` and a multi-megabyte `ImageFrame` alike"* | length is now bounded by what one read returns |
| net.rs:47 | *"including **a record longer than any one read returns**"* | that is now exactly the case that times out |
| net.rs:48 | *"A record still missing bytes when a non-blocking read **finds none** fails … and that is the only timeout"* | a read that finds *some but not all* is now also a timeout, so the stated condition no longer covers them |
| net.rs:57 | *"only a read that **finds nothing there** is a timeout"* | same, at the loop top's own comment |
| net.rs:59 | *"a non-blocking read that **finds nothing** is the timeout"* | same, in the comment that sits where the next repair will look |

All six are rewritten. The module `//!` and `recv_record`'s rustdoc now each
carry a paragraph that states the bound **and what it costs**, because the
narrowing in 2.1(2) is the kind of fact a caller has to be able to check.

**Two further findings from the same population, which are not reverse-sweep
hits and I label them separately.** `tests/imagery_net.rs:25` said *"Both the
loop-top check and any mid-read check produce that kind"* — that was already
inaccurate at `2c3041f`, where the loop top no longer fails; it describes
`e157774`. I rewrote it. And `tests/imagery_net.rs:32`'s *"no read of bytes
that are present can fail"* remains true of that test (every byte is
`peek`-confirmed) but is now load-bearing on a fact it does not state; I left
it, and name it here rather than silently.

**Corpus leg.** The reverse question also has to be asked of files I did *not*
touch. `/home/andrej/coord_scratch/riders67c3/sweep/corpus.py` matched all
**42** tracked files, flattened, against an open pattern of 15 terms
(`deadline`, `receive queue`, `non-?blocking`, `arriv`, `MAX_RECORD_LEN`,
`drain`, `still sending`, …) — 13 files hit. Reading every sentence in the
11 files that are not the two touched ones: `src/net.rs`'s 80 hits are the UDP
transport's own timeouts and name neither `ImageClient` nor `recv_record`;
`examples/imagery_dump.rs` passes a timeout through and asserts nothing about
it; `codebase-audit-docs/IMPLEMENTATION_PLAN.md`'s *"A slow sender must not
reset the total deadline by sending occasional bytes"* is made **more** true;
`src/lib.rs`'s *"bounded records and deadlines"* still holds. **Nothing outside
the two touched files states the post-deadline contract.**

**Refuted-citation sweep** (`AGENTS.md`: *"over the diff, over the tree, and
over `git log <base>..HEAD --format=%B`"*), run with
`scripts/refuted_citation_sweep.py --repo . 2c3041f 987f2ec` over eight
tokens — `however`, `longer`, `finds`, `remaining`, `waiting`, `progress`,
`megabyte`, `drain`. Every hit read:

| token | leg 1 (diff) | leg 2 (tree at `987f2ec`) | leg 3 (messages) |
|---|---|---|---|
| `however` | one `[-]` in `net.rs` | **no hit** | no hit |
| `megabyte` | one `[-]` in `net.rs`, three `[+]` in tests | 3, all new test prose | no hit |
| `finds` | three `[-]` in `net.rs`, two `[+]` in tests | 2, both new test prose quoting the base's rule | no hit |
| `longer` | one `[-]`, one `[+]` in `net.rs` | 12, of which `net.rs:219` is the new narrowing sentence and `net.rs:228` the pre-existing `MAX_RECORD_LEN` error text | no hit |
| `remaining` | two `[-]`, one `[+]` | many, all the `remaining` identifier in other modules | no hit |
| `waiting` | — | `net.rs:45`, `:206` are the new *"bounds waiting, not reading"*; `net.rs` UDP and tests are pre-existing | no hit |
| `progress` | — | all `ReadProgress` | no hit |
| `drain` | 11 `[+]` | 15, all new prose or the pre-existing `drain_and_check` helper | 2, both in my own message describing the base |

No refuted sentence survives in the tree. Leg 3 carries the retraction
deliberately (§5).

## 5. The three report-side corrections, and the retraction

**MF1** — I do not write *"it costs no extra waiting"*, and I did not reason
about it: I re-ran C2's own nine-row population **and** the six rows it could
not contain. `release`, 40 calls per cell, base against candidate, median time
until `recv_record` returns:

| buffered | timeout | base `2c3041f` | candidate `987f2ec` |
|---|---|---|---|
| one prefix byte | 1 ns | 4.759 µs | 4.518 µs |
| one prefix byte | 1 µs | 5.129 µs | 5.18 µs |
| one prefix byte | 100 µs | 2.957 ms | 2.943 ms |
| prefix only | 1 ns | 5.26 µs | 6.452 µs |
| prefix only | 1 µs | 5.28 µs | 6.432 µs |
| prefix only | 100 µs | 2.945 ms | 2.938 ms |
| prefix + half payload | 1 ns | 5.711 µs | 5.34 µs |
| prefix + half payload | 1 µs | 5.731 µs | 5.43 µs |
| prefix + half payload | 100 µs | 2.953 ms | 2.941 ms |
| **deliver** payload 0 | 1 ns / 100 µs | 1.162 / 1.122 µs, 40/40 | 1.182 / 1.143 µs, 40/40 |
| **deliver** payload 4096 | 1 ns / 100 µs | 2.114 / 2.074 µs, 40/40 | 2.074 / 2.094 µs, 40/40 |
| **deliver** payload 65536 | 1 ns / 100 µs | 4.328 / 4.338 µs, 40/40 | 4.248 / 4.418 µs, 40/40 |

**The conclusion this table supports, with its population:** over a silent
peer with a few-kilobyte declared total, base and candidate are
indistinguishable — differences run to about 1 µs in both directions, and the
100 µs rows are floored by the platform's ~2 ms tick on both trees. **It
supports nothing outside that population**, and §1's two tables are the cases
that fall outside it: a peer-declared 64 MiB (205 ms) and a refilling peer
(3 in 43). That is precisely the step C2's report took and the review refuted.

**MF2** — the property. C2's report adopted the brief's **P**: *"A record every
byte of which has arrived is delivered, whatever its kind and whatever its
length; a record still missing bytes when the deadline passes fails, and only
that is a timeout."* The candidate satisfies clause two, which C2 did not:
§3's axis is a record still missing bytes at the deadline, and it fails, 43
of 43. **Clause one holds only under a stated reading of "arrived"**, and I
will not paper over which: *arrived* = **in the receive queue when the
post-deadline read runs**. Under that reading clause one holds, and every
delivery test passes. Under the wider reading *arrived* = *handed to the
kernel by the peer*, clause one is **false** on this candidate for a record
larger than the receive window, and I would rather report P as satisfied under
a defined term than claim a quantifier I cannot hold.

**MF3** — §2.2 decides the allocation on *time past the deadline*, which is
what N4 and the C2 brief raised. The size argument is not reused.

**The retraction, in the commit message**, which is the only repair available
for a frozen one — `AGENTS.md`: *"What it finds is repaired in the **next**
commit's message, never by amending the candidate: the SHA is the identity."*
`987f2ec`'s message withdraws C2's *"Measured, this costs no extra waiting"*,
says why the population cannot contain either cost, and keeps the nine medians
for the population they were taken over. It also names two further sentences
of C2's message that no longer describe this tree: *"only a read that finds
none is the timeout"* — the rule a refilling peer ran on — and *"it does not
depend on … the record's kind or length"*, now false of the length.

## 6. Ablation — break each axis, watch the right test die

Four ablations, each from a `git archive` of the candidate tree, each breaking
exactly one thing, each in its own `CARGO_TARGET_DIR`, with a NULL control.

| ablation | what it breaks | tests that fail |
|---|---|---|
| **NULL** | nothing | none — **182 passed, 0 failed** over 16 targets |
| **A1** | `take_arrived` drains until `WouldBlock` again (C2's rule; the B1b bound removed) | exactly `a_peer_still_sending_past_the_deadline_is_a_timeout` |
| **A2** | the late-allocation branch removed, always `resize` (the B1a bound removed) | exactly `a_declared_length_costs_nothing_past_the_deadline` |
| **A3** | the loop top returns `TimedOut` instead of calling `take_arrived` (the delivery half) | `a_fully_buffered_bodied_record_is_delivered_past_its_deadline`, `a_fully_buffered_record_is_delivered_when_its_deadline_expires_mid_read`, `a_record_finished_without_waiting_leaves_the_socket_blocking` |
| **A4** | `set_nonblocking(false)` restore deleted | exactly `a_record_finished_without_waiting_leaves_the_socket_blocking` |

`passed + failed` is 182 in all five runs, and every run shows a
`Compiling natnet` line, so no build was reused. **Each of the four axes has a
test that dies when that axis alone is removed, and A1 and A2 take down only
their own** — which is the claim C2's table could not make for the two bounds,
because neither existed.

A3 and A4 share a test. That is not a leak: `a_record_finished_without_
waiting_leaves_the_socket_blocking` reaches the non-blocking path only through
the delivery axis, so removing delivery removes its precondition. C2's report
recorded the same overlap.

## 7. Gates — every one, with its numbers

See `gates.md` for the command lines and the per-target cells. Summary:

- `cargo fmt --all --check` rc **0**.
- `cargo clippy --locked --all-targets --all-features -- -D warnings` rc **0**,
  **0 warnings**.
- `cargo test --locked --all-features` → **182 passed / 0 failed** over **16
  targets** (5, 23, 7, 11, 28, 6, **23**, 12, 11, 6, 4, 6, 9, 17, 4, 10 = 182).
  C2's table was 180 with 21 in `imagery_net`; the two added tests are the
  difference and every other cell is unchanged.
- `cargo test --locked` → **99 / 0** over 16 targets.
- `cargo build --locked --features net` rc **0**;
  `cargo build --locked --features imagery,net --example imagery_dump` rc **0**.
- `cargo doc --locked --no-deps` and `--all-features`, `RUSTDOCFLAGS=-D
  warnings`, rc **0** / rc **0**.
- MSRV `cargo +1.92 test --locked` → **99 / 0**;
  `+1.92 --locked --all-features` → **182 / 0**.
- `features` matrix, all six rows, clippy rc 0 and test rc 0:
  `imagery` **127/0**, `net` **121/0**, `serde` **101/0**, `nalgebra`
  **101/0**, `imagery,net` **178/0**, `serde,nalgebra` **103/0**. 0 clippy
  warnings across the job. Five rows are identical to C2's; `imagery,net`
  moves 176 → 178.
- `feature-docs`, all **16** rows: `cargo check` rc 0 and `cargo doc` rc 0 for
  every row, 0 warnings.
- `cargo-deny 0.20.2 … check advisories bans licenses sources` rc **0** —
  `advisories ok, bans ok, licenses ok, sources ok` — with exactly the two
  pre-existing `license-not-encountered` warnings (`BSD-2-Clause`,
  `BSD-3-Clause` at `deny.toml:35,36`).
- `cargo-machete 0.9.2` rc **0**, *"didn't find any unused dependencies"*.
- **Clean extraction** of `bf191d3` with `git archive`: **14** files in
  `tests/`, **0** matching `zz_`, `cargo test --locked --all-features`
  **182 / 0**, and the extracted `src/imagery/net.rs` is `cmp`-identical to the
  candidate blob.
- **BASE comparison** — the candidate tree with `src/imagery/net.rs` taken from
  `2c3041f`: **180 passed / 2 failed**, the two failures being exactly
  `a_peer_still_sending_past_the_deadline_is_a_timeout` and
  `a_declared_length_costs_nothing_past_the_deadline`. All four pre-existing
  lateness and delivery tests pass on both trees.
- **Flake check**: the full `imagery_net` target, 20 runs `debug` and 20 runs
  `release` on the candidate — **40 of 40 at 23/0**.

**One instrument I could not run the way CI does.** `cargo-machete` is not on
this host's `PATH` and is not a `cargo` subcommand here — `command -v` and
`cargo machete` both return nothing, which is the shape of absence claim C2's
own MF2 note warns about. A bounded `find / -name cargo-machete -type f`
located it at `/home/andrej/coord_scratch/riders67/machete/bin/cargo-machete`,
left by an earlier attempt in this slice's own scratch tree, at exactly the
pinned `0.9.2`. I ran that binary. I installed nothing.

## 8. Decisions the brief did not cover

1. **The bound is stated as "one read", not as the reviewer's predicate.** §2.1
   says why; the reviewer's phrasing and mine accept and reject the same
   inputs, and I would rather a future reader have the invariant than the
   branch condition.
2. **The live path is untouched.** `vec![0u8; total]` would help there too —
   §2.2's table implies ~8 ms per 4 MiB record in `debug` — but it discards the
   caller's buffer reuse on every record, which is a different change with a
   different justification and no finding behind it. Not attempted.
3. **No new error message.** The short-read arm produces the same
   `"record deadline expired"` string the `WouldBlock` arm does, so nothing
   API-visible is minted.
4. **A real `ImageFrame` for the axis, a synthetic prefix for the cost test.**
   The axis builds an `ImageFrame` whose payload is sized to the measured
   absorption. The cost test declares `MAX_RECORD_LEN` with a kind-99 prefix,
   as `exact_record_cap_is_accepted_and_one_byte_over_is_refused` already does:
   it never delivers a record, so the kind is never parsed, and building a
   64 MiB record to read twelve bytes of it would be the cost the test exists
   to refuse.
5. **A `debug_assert!` on `take_arrived`'s precondition.** The one-read form
   makes `destination[*read..]` being non-empty load-bearing — an empty slice
   would read `Ok(0)` and be reported as a clean EOF. `read_until`'s loop
   guarantees it; the assertion records that rather than trusting it.
6. **Nothing installed. Nothing pushed.**

## 9. What does not ship, and where it is

`tests/zz_c3_probe.rs` was moved to
`/home/andrej/coord_scratch/riders67c3/zz_c3_probe.rs` (sha256
`934ad2e409e173906816c0cbb176d7c6807c2b38353898c3957e481babf8cf55`) before
the gate runs. **It does not ship**: the frozen tree has 14 files in `tests/`
and none matching `zz_`, confirmed by `git ls-tree` and by the clean
extraction building and passing 182/0 without it. Copying that one file back
into `tests/` re-runs every number in §1 and §2's candidate table.
`zz_c3_cost.rs` (§5's table) is at
`/home/andrej/coord_scratch/riders67c3/zz_c3_cost.rs`, same treatment. The
allocation benchmark of §2.2 is `/home/andrej/coord_scratch/riders67c3/alloc/`
and the window measurement is `.../rcvwin.rs`. Ablation and comparison trees
are at `/home/andrej/coord_scratch/riders67c3/{abl/{NULL,A1,A2,A3,A4},extract,
basecmp}`. The sweep listings are under `.../sweep/`.

## 10. What I am unsure about

- **§2.2's bound is the allocator's, not the language's, and I say so there.**
  If a reviewer wants a bound that does not move with `M_MMAP_THRESHOLD`, the
  only shapes I could find are a cap (a constant I cannot derive) or `unsafe`
  (forbidden). I would treat a finding here as correct rather than argue.
- **The narrowing in §2.1(2) is a behaviour change against C2 that no test
  catches.** A record whose bytes the peer has handed to the kernel but which
  flow control has parked in the sender's buffer is now a timeout. I could not
  write a test that distinguishes it without asserting on a host's socket
  buffer sizes; on this host the base and the candidate behave identically in
  that case, because the base's drain loses the refill race about 98% of the
  time with a cold window. I documented it instead.
- **The signal-interrupted partial read of §2.1(1) is unexercised.** I could
  not construct one.
- **`a_peer_still_sending_past_the_deadline_is_a_timeout` measures the host and
  then acts on it.** Its assertion cannot be wrong — the record always declares
  more than the connection will hold — but if `absorption_of` ever returned a
  wildly wrong number the test would stop *discriminating* while still passing.
  It asserts `absorbed < 512 MiB` and that `record.len() > absorption`, which
  is the most I could make it check about itself.
- **The 2 ms floor is unchanged and is still a platform property.** My 100 µs
  rows in §5 reproduce it on both trees. I did not touch it in either
  direction and did not re-derive the review's `CONFIG_HZ` finding, which I
  accept.
- **I did not construct a production-timescale instance.** Everything here uses
  1 ns and 1 µs deadlines. The production shape — the remaining deadline is
  zero when a read is about to be issued — is what any stall spanning the
  deadline produces at any timeout; that is an argument, and I label it one,
  exactly as C2's report did and for the same reason.

## 11. Scope compliance

- `docs/work/` : **not committed**. This report and `gates.md` are untracked;
  `git status --porcelain --untracked-files=no` after the candidate commit is
  empty and `git status --short` shows only `?? docs/`.
- `codebase-audit-docs/` : not edited — read only, for §4's corpus leg.
- Modules other than `src/imagery/net.rs` : not edited. Only
  `tests/imagery_net.rs` besides it. `git diff --stat 2c3041f HEAD` is 2 files.
- No frozen packet touched; `docs/work/riders-67/attempts/C2/` and the C3 brief
  were read only.
- No `git push`, no publication, no part of this crate sent anywhere.
