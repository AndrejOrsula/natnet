# riders-67 C2 — builder report

Repository `natnet`. Branch `slice/riders-67-C2`, worktree
`/home/andrej/ws_rs/natnet_wt/impl-riders-67-C2`.

## 0. Identity

| | |
|---|---|
| Base | `candidate(C1)` = `e157774c85a425a84eee00fff188d739bcf64c39`, tree `1a1c9bc69d344fe4f8c85b70217a687320eb2893`, parent `3a6400e`, subject *"fix(imagery/net): deliver a record whose last byte lands on the deadline"* |
| Base resolved | by me at **2026-09-13T10:44:22+02:00**, two ways: `git -C <worktree> rev-parse HEAD` and `git -C /home/andrej/ws_rs/natnet rev-parse slice/riders-67`. Both give `e157774c85…`, which is the value the C2 brief recorded and the value the C1 review re-resolved. Not `dev` |
| `candidate(C2)` SHA | **`2c3041f890481f2df8790de78a03258170149bd8`** |
| `candidate(C2)` tree | **`54ea58d23c23ff01cdb5dd6202916636de044eb8`** |
| `candidate(C2)` parent | `e157774c85a425a84eee00fff188d739bcf64c39` — one commit on top of the base, not a rebase of `dev` |
| Files changed | `src/imagery/net.rs`, `tests/imagery_net.rs` (2 files, +291 −50) |
| Commit message file | `/home/andrej/coord_scratch/riders67c2/msg.txt`, committed with `git commit -F` |
| Pushed | no. No `git push` was run in this slice, to any remote |
| Maturity | complete; no blockers |

Toolchain `rustc 1.97.1 (8bab26f4f 2026-07-14)` / `cargo 1.97.1 (c980f4866
2026-06-30)` via `rust-toolchain.toml` (`channel = "stable"`); MSRV rows on
`cargo 1.92.0 (344c4567c 2025-10-21)`. All cargo calls ran with
`CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c2/target` (or a
per-tree sibling of it for the comparison runs) and the binary named by
absolute path `/home/andrej/.cargo/bin/cargo`.

**This attempt was interrupted once by a session rate limit** and resumed. The
brief's instruction to commit at each milestone is why nothing was lost: the
checkpoint commit `73dfc2b` held the whole repair. That checkpoint was
squashed into the single candidate commit at the end, and is preserved as the
reachable tag **`riders-67-C2-checkpoint`** so that an amended-away commit is
not one `git gc` from gone. `git rev-parse 73dfc2b:src/imagery/net.rs`,
`2dacc1f:…` and `2c3041f:…` are all `d994553e6851c38df163892f3e5c9c953eab91fb`
— the source file is byte-identical across all three, so every measurement
taken before the squash applies to the frozen tree, and the probe numbers in
§3 were re-run against the frozen tree anyway.

## 1. The defect re-derived on `e157774`, with my own instrument

The review's B1 says the candidate changes nothing for any record with a body.
I wrote my own instrument rather than running the reviewer's — same
construction (`peek`-confirm every byte is in the client's receive queue
before the client is built), my own code, my own record builder, and one cell
the review did not have (65536 bytes). 200 calls per cell, 1 µs record
timeout, on `e157774`:

```
C2PROBE payload=     0 n=200 t=1us delivered=200 timed_out=  0 msgs=[]
C2PROBE payload=     1 n=200 t=1us delivered=  0 timed_out=200 msgs=["record deadline expired |buf=0|"]
C2PROBE payload=    64 n=200 t=1us delivered=  0 timed_out=200 msgs=["record deadline expired |buf=0|"]
C2PROBE payload=  4096 n=200 t=1us delivered=  0 timed_out=200 msgs=["record deadline expired |buf=0|"]
C2PROBE payload= 65536 n=200 t=1us delivered=  0 timed_out=200 msgs=["record deadline expired |buf=0|"]
```

and the timeout sweep, 100 per cell, on `e157774`:

```
payload 0:     1us 100   2us 100   3us 100   5us 100  10us 100  20us 100  50us 100  100us 100  1ms 100
payload 4096:  1us   0   2us   0   3us   0   5us   0  10us   0  20us 100  50us 100  100us 100  1ms 100
```

My sweep agrees with the review's candidate column cell for cell. **I have no
disagreement with the review's table.**

### The discriminator the review argued, measured instead

B1 concludes the failure is the **body** call's loop top, from `buf.len()==0`
plus the heartbeat control. Both are sound, but they are inferences. I built a
third notation: a copy of the `e157774` tree in which `read_until` takes a
`site: &str` and the loop-top message carries it, `"prefix"` at `net.rs:239`
and `"body"` at `net.rs:289`. Same 200-call cells:

```
C2PROBE payload=     0 delivered=200 msgs=[]
C2PROBE payload=     1 delivered=  0 msgs=["record deadline expired [loop-top of body] |buf=0|"]
C2PROBE payload=    64 delivered=  0 msgs=["record deadline expired [loop-top of body] |buf=0|"]
C2PROBE payload=  4096 delivered=  0 msgs=["record deadline expired [loop-top of body] |buf=0|"]
C2PROBE payload= 65536 delivered=  0 msgs=["record deadline expired [loop-top of body] |buf=0|"]
```

800 of 800 discards name the body call. B1's mechanism is confirmed by an
instrument that reports the answer rather than by an argument that implies it.

## 2. The repairs I enumerated, and why I picked the one I picked

The brief states a property and withholds a choice set. The enumeration below
is what I could find; the two the review named are in it as **R2** and **R3**.

> **P.** A record every byte of which has arrived is delivered, whatever its
> kind and whatever its length; a record still missing bytes when the deadline
> passes fails, and only that is a timeout.

**R1 — guard the deleted mid-read check** (`*read < destination.len() && …`),
the original brief's option A. Rejected: the check that destroys bodied
records is the *body call's loop top*, not the mid-read check, so this is
inert for them. Measured, §1: identical to `e157774` at every timeout.

**R2 — carry "is the record still outstanding?" across both calls** (review's
example (a)): give `read_until` a flag meaning "bytes of this record have
already been read", and let the loop top decline to fail when it is set.
Closes the prefix→body boundary. **Does not close the class**, and the reason
is measurable rather than rhetorical: at the *prefix* call the flag is false by
construction, so a deadline spent before the first read still destroys a fully
arrived record at the prefix loop top. See §3's 1 ns row — `e157774` discards
a fully buffered record 100 times in 100 there, including the `Heartbeat`,
which is the kind C1's own test asserted was safe.

**R3 — check the deadline once at the top of `recv_record`** (review's example
(b)). Same objection in a sharper form: it *keeps exactly one* pre-read check
and puts it at the earliest point in the call, which is the one the 1 ns row
fires. It is also not a complete mechanism on its own — something still has to
happen when `remaining` is zero at a mid-record loop top, and
`set_read_timeout(Some(Duration::ZERO))` is not available: measured,
`Err(InvalidInput, "cannot set a 0 duration timeout")`.

**R4 — merge the two `read_until` calls into one.** My own; the third notation
the brief asked for. Read into a growable `buf` and re-target the destination
once `total` is known, so there is no prefix/body boundary to get wrong.
Rejected, and instructively: the destination is still only `PREFIX_LEN` bytes
long until the prefix is parsed, so a bodied record still needs at least two
read syscalls with a loop top between them, and that loop top is still a
pre-read check. R4 converts "always broken for bodied records" into "broken
whenever more than one read is needed" — narrower, still open. I did not
implement it; this is an argument from the code's shape and I label it one.

**R5 — clamp the read timeout instead of failing.** Delete the loop-top
failure outright and issue every read with `remaining.max(<minimum>)`, letting
`WouldBlock` be the only timeout. This *does* close the class — no pre-read
check survives — and it is two lines. Rejected on a measurement I did not
expect. I assumed the cost was the one-microsecond floor `std` imposes when it
converts a sub-microsecond `Duration` to a `timeval`. It is not:

```
C2STD read timeout requested=1ns     n=200  min=998.378µs median=2.000622ms max=3.021462ms
C2STD read timeout requested=100ns   n=200  min=999.901µs median=1.999851ms max=3.001775ms
C2STD read timeout requested=1000ns  n=200  min=1.92956ms  median=1.999561ms max=3.003739ms
```

A read asked to wait 1 ns waits a **median of 2 ms** on this host. R5 would
therefore buy class closure at a cost of up to one such interval *per read
that finds the queue momentarily empty*, past a deadline that has already
expired. "The deadline bounds waiting" would become "bounds waiting to within
a couple of milliseconds per outstanding byte", which is not a sentence I
would want to put in the module docs.

**R6 — decide on the socket, not at a boundary. Chosen.** Past the deadline,
`read_until` stops answering the question itself and hands the rest of its
destination to `take_arrived`, which sets `O_NONBLOCK` and reads: a read that
returns bytes makes progress, a read that returns `WouldBlock` is the timeout.

### Why R6 closes the class rather than one more member of it

The class is **pre-read deadline checks**. Every member of it — the mid-read
check the base had, the loop top of the prefix call, the loop top of the body
call, and any check a future author adds between `record_len` and the payload
read — fails for the same reason: *it runs before the read that would tell it
whether the bytes are there.* R1–R4 relocate which pre-read checks survive.
They cannot change what a surviving one is able to know.

R6 removes the question from every boundary and puts it on the only instrument
that can answer it. The consequences are the three quantifiers in P:

- **independent of how many calls** — the decision is re-made at each read, so
  it does not matter that `recv_record` calls `read_until` twice, or that a
  future version calls it three times. This is the brief's *"what is the next
  call site that would break it?"*: there is no per-call-site decision left to
  get wrong. I put that to the test rather than asserting it — the module doc
  and the code comment at `net.rs:329` both now say it, and `take_arrived` is
  reached from the single loop top that every destination goes through.
- **independent of kind** — nothing in the mechanism reads the record kind.
  Measured over `Heartbeat` (empty body) and `ImageFrame` with 1, 64, 4096 and
  65536-byte payloads.
- **independent of length** — the loop drains until the destination is full or
  the queue is empty, so a record longer than one read returns is no different
  from one that fits.

And it terminates without waiting: each pass either consumes at least one of
the finitely many outstanding bytes or ends the loop, and no pass blocks.

### What R6 costs, measured because it is the obvious objection

A non-blocking drain could in principle make `recv_record` slower to give up,
which would be a weakened deadline that no delivery count would reveal. It
does not. Against a peer that will never send enough, 40 calls per cell,
`e157774` against the candidate, median time until `recv_record` returns
`Err`:

| buffered | timeout | `e157774` median | candidate median |
|---|---|---|---|
| one prefix byte | 1 ns | 3.54 µs | 5.03 µs |
| one prefix byte | 1 µs | 4.61 µs | 5.40 µs |
| one prefix byte | 100 µs | 1.963 ms | 2.940 ms |
| prefix only | 1 ns | 3.48 µs | 17.47 µs |
| prefix only | 1 µs | 16.73 µs | 17.53 µs |
| prefix only | 100 µs | 2.931 ms | 2.939 ms |
| prefix + half payload | 1 ns | 3.53 µs | 18.02 µs |
| prefix + half payload | 1 µs | 16.48 µs | 18.03 µs |
| prefix + half payload | 100 µs | 2.933 ms | 2.939 ms |

The candidate gives up at most ~14 µs later, and only in the 1 ns rows, where
`e157774` bails at the very first loop top without issuing a single read while
the candidate issues the non-blocking reads that are the whole point. The
100 µs rows are indistinguishable, because on **both** trees they are floored
by the ~2 ms receive-timeout granularity measured above. Which is worth saying
plainly: neither tree enforces a record deadline more finely than a timer
tick, and this change does not make that worse.

## 3. The test that fails on `e157774` and passes on the candidate

Three tests were added to `tests/imagery_net.rs`, and one of C1's was
strengthened.

**`a_fully_buffered_bodied_record_is_delivered_past_its_deadline`** — the
coverage whose absence is the whole finding. A real `ImageFrame` (not a
synthetic kind-99 record: it is the record the review named as untouched),
payloads of 1 and 4096 bytes, every byte `peek`-confirmed present, 32
attempts at each of two deadlines. The delivered bytes are compared to the
bytes sent *and* re-parsed, so a zero-padded or truncated buffer cannot pass.

**`a_record_missing_payload_bytes_past_its_deadline_still_fails_and_is_terminal`**
— the still-fails-when-genuinely-late test, sited at the boundary the repair
moved: a complete prefix (so `recv_record` gets past `record_len`, past the
`MAX_RECORD_LEN` check and into the payload read) and half a payload, with a
peer that sends no more. `TimedOut`, empty buffer, terminal client, and the
terminal error replays identically.

**`a_record_finished_without_waiting_leaves_the_socket_blocking`** — reads two
records over one client: the first past a 1 µs deadline (through the
non-blocking path), the second with its bytes written 50 ms *after* the first
call returned, so it can only be collected by a read that waits.

### The deterministic row, and why it is better than the 1 µs one

Both delivery tests carry a **1 ns** deadline row as well as the 1 µs one. A
1 ns deadline is spent before the first loop-top check runs, so that row needs
no timing window at all — it is deterministic on every host, on both trees:

```
                              e157774            candidate 2c3041f
C2FIRST payload=0    n=100    delivered=  0      delivered=100
C2FIRST payload=1    n=100    delivered=  0      delivered=100
C2FIRST payload=4096 n=100    delivered=  0      delivered=100
```

This row is also what refutes R2 and R3 in §2: on `e157774` it destroys the
**heartbeat** 100/100, at the prefix call's loop top, where no repair scoped to
the prefix→body boundary is looking.

### Both runs, at suite scope

`src/imagery/net.rs` taken byte-for-byte from `e157774` (confirmed with `cmp`
against `git show e157774:src/imagery/net.rs`), `tests/imagery_net.rs` taken
byte-for-byte from the frozen tree, `cargo test --locked --all-features
--no-fail-fast`:

```
BASE  e157774   rc=101  passed=177  failed=3  total=180
  FAILED  a_fully_buffered_bodied_record_is_delivered_past_its_deadline
  FAILED  a_fully_buffered_record_is_delivered_when_its_deadline_expires_mid_read
  FAILED  a_record_finished_without_waiting_leaves_the_socket_blocking
  passed  an_incomplete_record_past_its_deadline_still_fails_and_is_terminal
  passed  a_record_missing_payload_bytes_past_its_deadline_still_fails_and_is_terminal
  passed  byte_drip_cannot_restart_the_total_record_deadline

  thread 'a_fully_buffered_bodied_record_is_delivered_past_its_deadline' panicked at
  tests/imagery_net.rs:859:31:
  payload_len 1, timeout 1ns, attempt 0: a complete record was received and then
  discarded: record deadline expired (kind TimedOut, buf.len() 0)

CANDIDATE 2c3041f  rc=0  passed=180  failed=0  total=180
```

The three lateness tests pass on **both** trees, which is the point: the
candidate does not buy delivery by weakening the deadline.

**One honest qualification.**
`a_record_finished_without_waiting_leaves_the_socket_blocking` fails on
`e157774` for the *wrong* reason — it never gets past its first `recv_record`,
because that call is the bodied delivery this slice repairs. It is not
evidence about blocking-mode restoration on the base (the base never sets
`O_NONBLOCK`, so there is nothing there to restore). Its evidence is ablation
A3 below, which is scoped to exactly that.

### C1's test, strengthened rather than reverted

`a_fully_buffered_record_is_delivered_when_its_deadline_expires_mid_read` kept
its construction. Its retry arm did not: under this repair a fully buffered
record at the loop top is no longer a timeout, so an arm that counted one and
tried again would have been dead code sitting under a doc comment saying it
was reachable. The arm now panics and the assertion is `delivered == 2 *
ATTEMPTS`. That is a strengthening, and I want to be exact about what it is
not: the 1 µs half of it does not discriminate the trees (`e157774` delivers
the heartbeat 200/200 there). The 1 ns half does, 100/100 against 0/100.

I read the C2 brief's *"do not re-litigate it and do not revert it"* as
covering the deletion inside `read_until`, the flake bound's soundness and the
mutant substitution — none of which I touch or dispute. Changing an assertion
that this attempt makes false is a different act, and I am flagging it rather
than burying it.

## 4. Ablation — break the repair, watch the tests fail

Three ablations, each built from the frozen tree and each breaking exactly one
thing, with candidate NULL controls on both sides. Full numbers in
`gates.md`.

| ablation | what it breaks | fails |
|---|---|---|
| **A1** loop top returns `TimedOut` again instead of calling `take_arrived` | the delivery half of P | the 3 delivery tests; all 3 lateness tests still pass |
| **A2** `take_arrived` breaks `Ok(Complete)` on `WouldBlock` instead of `Err` | the lateness half of P | the 3 lateness tests, including the pre-existing `byte_drip_cannot_restart_the_total_record_deadline`; all 3 delivery tests still pass |
| **A3** `set_nonblocking(false)` restore deleted | mode restoration only | exactly `a_record_finished_without_waiting_leaves_the_socket_blocking` |

`pass+fail` is 180 in all six runs including both NULL controls, and every
ablation run shows a `Compiling natnet` line, so the build was invalidated
rather than reused.

Read as an ablation this is the table I wanted: **no test passes without the
mechanism it is there for.** Each of the three axes of the repair has at least
one test that dies when that axis alone is removed, and none of the three
ablations takes down a test belonging to another axis. A1 also serves as its
own control — it was built to reproduce `e157774`'s predicate inside C2's
structure, and it fails the identical three tests the real `e157774` tree
does.

## 5. The prose, which was three of the shipped locations — and a fourth

### L1 — the module `//!` (`src/imagery/net.rs`, the *Timeouts* section)

```rust
//! a byte does not restart it. The deadline bounds *waiting*, not reading:
//! once it has passed, every remaining read of that record is issued
//! non-blocking, so a record whose bytes have all reached the receive queue
//! is delivered **whatever its kind and however long it is** — a `Heartbeat`
//! and a multi-megabyte `ImageFrame` alike, and including the case where the
//! deadline falls between the read that collects the prefix and the read
//! that collects the payload. What fails is a record whose outstanding bytes
//! are not there when a non-blocking read looks for them, and only that is a
//! timeout — discarding a record already in hand would end the session over
//! bytes that were never late.
```

The universal is still a universal, because it is now true; what changed is
that it names the mechanism that makes it true and names the two kinds the old
sentence quietly excluded.

### L2 — `recv_record`'s rustdoc, rendered

```rust
/// slow delivery. It bounds *waiting*, not reading. This call reads the
/// prefix and the payload separately under that one deadline, and once the
/// deadline passes both are finished from bytes already in the receive
/// queue, without blocking. A record every byte of which has arrived is
/// therefore delivered whatever its kind and length — including when the
/// deadline falls between those two reads, and including a record longer
/// than any one read returns. A record still missing bytes when a
/// non-blocking read finds none fails with [`io::ErrorKind::TimedOut`],
/// and that is the only timeout.
```

The old text's failure was not that it was vague but that it described a
one-read function. This one says "two reads" out loud, because that is the
fact a reader needs in order to check the claim.

### L3 — the comment where the deleted block used to be

The review called this the most dangerous of the three, because it sits where
the next repair will look and told the reader the class was closed. It is now
the place that tells them why a check there is wrong *and* forecloses the exact
argument C1 used:

```rust
// Deliberately no deadline check here, and none at the
// loop top either: neither place can tell a late record from
// an arrived one, because both run before the read that would
// answer the question. `take_arrived` above is the only thing
// that decides, and it decides on the socket — a
// non-blocking read that finds nothing is the timeout.
//
// Do not restore a check here or at the loop top on the
// reasoning that `Instant` is monotonic and the next pass
// would fail anyway. That argument is about *this* function
// finishing *this* destination, and `recv_record` calls it
// twice under one deadline, once for the prefix and once for
// the body. Any deadline check that runs before a read is a
// check at one of those boundaries, and a record whose every
// byte is already in the kernel receive queue would be
// destroyed there — with the connection shut down, because
// receive failures are terminal.
```

### L4 — a fourth location, which the brief did not name and I nearly shipped

The brief named three. There were four. `tests/imagery_net.rs:698` opened C1's
test doc with the refuted rustdoc sentence verbatim — *"The record deadline
bounds \*waiting\* for bytes that have not arrived"*.

I want to be precise about how I found it, because it is not a flattering
story. My first corpus sweep was line-oriented and **did** return this file; I
read lines 698–700, judged them true of the candidate, and moved on. They
*were* true. What I missed is that a sentence can be true and still be the
fourth copy of a contract stated in wording a review has just condemned for
under-specifying its scope. What made me look again was leg 2 of the
refuted-citation sweep, run flattened over every tracked file, which printed
`*** STILL PRESENT ***` for a phrase I had already told myself was handled. It
now reads:

```rust
/// The record deadline bounds *waiting*, not reading. A record whose every
/// byte is already in the receive buffer must be delivered whenever the
/// deadline passes — before the read that collects it, during it, or after
/// it — because none of those is the client waiting for anything. Discarding
/// such a record would throw away a complete one and, because receive
/// failures are terminal, end the session with the record in hand.
```

### The sweep, run as a sweep

`git ls-files`, every tracked file, matched **flattened** (whitespace and
comment leaders collapsed) because corpus prose wraps mid-claim and a
line-oriented `grep` returns zero for text that is present:

```
phrase                                                    leg1 removed  leg2 tree  leg3 msgs
"Only the second case is a timeout"                            Y         absent     absent
"bounds waiting for bytes that have not arrived"               Y         absent     absent
"could therefore only differ from the first by rejecting…"     Y         absent     absent
"the loop-top check, which is a legitimate timeout and is…"    Y         absent     absent
"none of {ATTEMPTS} attempts reached the mid-read window"      Y         absent     absent
"deliver a record whose last byte lands on the deadline"       n         absent     PRESENT
```

Leg 1 is `git diff e157774..2c3041f` restricted to removed lines; leg 2 is the
tree at the candidate; leg 3 is `git log e157774..2c3041f --format=%B`, one
commit, and the widened `e157774^..` leg adds only `e157774`'s own message
(`e157774` has one parent, `3a6400e`). The last row is the intended one: C1's
subject appears in my commit message because §7 of the brief requires it to.
No refuted sentence survives anywhere in the tracked tree.

Outside these two files, nothing states the record-deadline contract:
`grep -rn "record deadline\|record_timeout\|bounds waiting"` over `README.md`,
`codebase-audit-docs/`, `src/` and `examples/` returns only
`examples/imagery_dump.rs`, which passes a timeout through and asserts nothing
about it. `codebase-audit-docs/` was read, never edited.

## 6. The `MAX_RECORD_LEN` allocation decision (review N4)

**Decision: do not order a deadline check before `buf.resize(total, 0)`.**
Three reasons, in the order they bind.

1. **Such a check is the defect.** It would run between the `MAX_RECORD_LEN`
   check and the payload read — before the read that could tell it whether the
   payload has arrived — so it would destroy a record whose every byte is in
   the receive queue, at a *new* boundary. It is a fresh member of the class
   §2 exists to close, and the test in §3 would catch it, which is the point of
   having asked "what is the next call site that would break it?".
2. **It would bound nothing.** The allocation is already bounded, by
   `MAX_RECORD_LEN = 64 << 20`, checked immediately before the resize
   (`src/imagery/net.rs`, `if total > MAX_RECORD_LEN` → `InvalidData`, covered
   by `a_hostile_length_is_refused_before_allocating`). The deadline bounds
   neither the size of the allocation nor its rate. It was never the guard.
3. **N4's accounting changes under this repair, and I should say how.** N4
   observed "one per connection, since receive failures are terminal". On C2
   that is no longer the right sentence: the record is now *delivered*, so the
   connection survives and a peer can cause the allocation again. But that is
   not a new exposure, because it is not the deadline that gates it — on every
   tree including the base, a prefix that completes *before* the deadline (the
   overwhelmingly normal case) reaches `record_len`, the cap check and the
   resize. A peer that wants to force repeated 64 MiB zero-fills only has to
   send its prefixes promptly, which is easier than landing them past a
   deadline. The change relative to the base is not "a new allocation" but
   "the same allocation the base already performs for every record it
   delivers, now also performed for records it used to discard" — which is
   what delivering them means.

Not attempted, and out of scope: reducing the zero-fill itself.
`buf.resize(total, 0)` writes 64 MiB of zeroes for a maximal record before any
payload byte is read. Reading into spare capacity instead needs `unsafe` or
unstable `read_buf`, and neither belongs in this slice.

## 7. Report-side must-fixes carried from C1 (brief §7)

**MF1** — C1's *"the same `read_until` call with a different `destination`"* is
not repeated. It is false and §1 of this report says the opposite, measured
with the `site`-labelled tree: 800 of 800 discards name the **body** call.

**MF2 and MF3** — I reuse no C1 sentence without re-running its instrument, and
I have not reproduced either query. Where I state a search result in this
report I ran it: the `git ls-files` flattened sweep in §5, the `grep -rn` over
`README.md`/`codebase-audit-docs/`/`src/`/`examples/` in §5, the `cmp` checks
in §3, and the `find`-then-correct for `cargo-deny` recorded in `gates.md`.
That last one is the MF2 shape caught before it shipped: my first instrument
(`find /home/andrej/coord_scratch -name cargo-deny`) returned nothing, which
would have supported a false "not available on this host"; the binary is in
`~/.cargo/bin`, which that search cannot see.

**MF4** — repaired in this commit's message, per `AGENTS.md` §7: *"What it
finds is repaired in the **next** commit's message, never by amending the
candidate: the SHA is the identity."* The message states that `e157774`'s
subject over-claims, quotes it, and says what it actually delivers (only when
the last byte completes the *final* `read_until` destination — `Heartbeat` and
`Bye`, two of six kinds), and that its body's *"discarding a record every byte
of which was already in hand"* holds for those two kinds only.

## 8. Decisions the brief did not cover

1. **The scratch instrument does not ship.** `tests/zz_c2_probe.rs` was moved
   to `/home/andrej/coord_scratch/riders67c2/zz_c2_probe.rs` (sha256
   `a3ad947af054e6436635f58a6502801b31ee7e9beaaf32c2c6133f53f2f547f5`) before
   the gate runs. The frozen tree has 14 files in `tests/` and none matching
   `zz_`; copying that one file back into `tests/` re-runs every number in §1,
   §2 and §3. The diagnostic trees are at
   `/home/andrej/coord_scratch/riders67c2/{diag,basetests,abl_A1,abl_A2,abl_A3,extract,extract_pure}`.
2. **One commit, not two.** The checkpoint `73dfc2b` was squashed so the
   candidate is a single commit whose message carries the MF4 repair where a
   reader of the tip will see it. The checkpoint is tagged
   `riders-67-C2-checkpoint` rather than left dangling.
3. **A real `ImageFrame`, not a synthetic kind-99 record**, in the shipped
   test. The review's instrument used kind 99 and that is fine for a probe;
   what the review said was untouched was `ImageFrame` specifically, so that is
   what the suite should now assert.
4. **Not minting the error message as a contract.** `take_arrived` produces the
   same `"record deadline expired"` string the loop top produced, so no new
   API-visible message exists. The strengthened heartbeat test no longer
   compares against that string at all, which removes the coupling C1's §4
   flagged as real.
5. **Nothing installed.** `cargo-deny` and `cargo-machete` were already on this
   host; no binary was installed into the repository or `~/.cargo/bin`.

## 9. What I could not verify, and what I am unsure about

- **The ~2 ms receive-timeout floor is a fact about this host.** min 0.998 ms /
  median 2.00 ms / max 3.02 ms over 200 reads, for any requested timeout of
  1 ns, 100 ns or 1 µs. It is the load-bearing measurement under my rejection
  of R5, and I ran it on one 32-core Linux machine, in a `debug` build, on
  loopback. On a host with finer timer granularity R5 would look much better
  than it does here. I did not test any other platform, and on Windows
  `SO_RCVTIMEO` semantics differ.
- **A consequence of that floor which is not mine but is worth flagging.**
  Because a sub-tick `remaining` still buys a full tick of blocking, a record
  whose bytes arrive up to a few milliseconds *after* a short deadline can be
  accepted, on the base and on this candidate alike. The deadline has never
  been enforced more finely than a tick on any of the three trees, and I did
  not change that in either direction — §2's table is the evidence that I did
  not make it worse. Making it sharper is a different slice and would need a
  different mechanism.
- **The 1 µs rows remain host-sensitive; the 1 ns rows are not.** That is why
  every delivery assertion now has a 1 ns row. If CI ever goes red here I
  would look at the 1 µs rows first — but unlike C1's bound, a failure there
  would now be a genuine regression rather than a host on which the window
  never opened, because the 1 ns row proves the path is reachable.
- **`a_record_finished_without_waiting_leaves_the_socket_blocking` has a
  50 ms sleep in it.** It is a real sleep and I am not going to pretend
  otherwise. It is not a race in the usual direction — the second
  `recv_record` has a 5 s deadline, so a slow host makes the test *more*
  likely to pass, not less — but it does mean the test would still pass if
  `O_NONBLOCK` were cleared by some other path. Ablation A3 is what shows it
  discriminates.
- **`take_arrived`'s behaviour under a socket error during restore.** If the
  drain succeeds but `set_nonblocking(false)` fails, I propagate the restore
  error; if the drain fails, I keep the receive failure and drop the restore
  error. That matches `fail()`'s existing *"Preserve the original receive
  failure even if shutdown also fails"*, but I could not construct a case
  where `set_nonblocking` fails on a live socket, so that arm is unexercised.
- **A `std` behaviour I hit and did not chase.** `TcpStream::peek` with a
  **zero-length** buffer intermittently blocks past its `SO_RCVTIMEO` — it
  hung my scratch probe for over 20 s on the fifth iteration of a cell where
  four had returned instantly, with a 5 s read timeout set. It cost me the run
  the rate limit then interrupted. It is a bug in my probe (confirming that
  nothing has arrived is a meaningless thing to ask), not in the crate: no
  shipped test calls `buffered_peer` with an empty slice, and the helper now
  asserts against it. I did not investigate further and I am not claiming a
  diagnosis.
- **I did not construct a production-timescale instance.** Everything measured
  here uses 1 ns and 1 µs deadlines. The production shape of this defect is
  "the remaining deadline is zero when a read is about to be issued", which
  any scheduling stall spanning the deadline produces at any timeout — that
  sentence is the review's argument and mine, and it is an argument, not a
  measurement.

## 10. Scope compliance

- `docs/work/` : **not committed**. This report and `gates.md` are untracked in
  the worktree. `git status --porcelain --untracked-files=no` after the
  candidate commit is empty.
- `codebase-audit-docs/` : not edited (read only, for §5's sweep).
- Modules other than `src/imagery/net.rs` : not edited. Only
  `tests/imagery_net.rs` besides it.
- No frozen packet touched; `docs/work/riders-67/attempts/C1/` and
  `docs/work/riders-67/brief.md` were read only.
- Temporary instruments are not in the tree at `54ea58d`:
  `git ls-tree -r --name-only` shows no `zz_` path, and the clean extraction
  builds and passes 180/0 without them.
- No `git push`, no publication, no part of this crate sent anywhere.
