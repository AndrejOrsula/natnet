# riders-67 C1 review — `natnet`, candidate `e157774`, tree `1a1c9bc`

Re-resolved by the reviewer: `slice/riders-67` tip
`e157774c85a425a84eee00fff188d739bcf64c39`, tree
`1a1c9bc69d344fe4f8c85b70217a687320eb2893`, parent `3a6400e`. Builder worktree
HEAD matches; `git status --porcelain` there shows only untracked `docs/`, so
the packet was not committed, as the brief required.

**This review does not return only report-side findings.** The most severe
finding lands on three shipped lines of `src/imagery/net.rs`.

## BLOCKING

### B1 — the shipped deadline contract is false for every record with a body, and the candidate does not change the behaviour of a single bodied record

**Where, in code:** `src/imagery/net.rs:44-49` (module `//!`),
`src/imagery/net.rs:192-194` (`recv_record` rustdoc, rendered), and
`src/imagery/net.rs:319-330` (the comment that replaced the deleted block).

The module doc now says:

> `//! a byte does not restart it. The deadline bounds *waiting*: a record whose`
> `//! every byte has arrived is delivered even when the read that collects the`
> `//! last of them returns at or after the deadline, while a record still`
> `//! missing bytes at the deadline fails. Only the second case is a timeout —`
> `//! discarding a record already in hand would end the session over bytes that`
> `//! were never late.`

**"Only the second case is a timeout" is false**, and so is `recv_record`'s
*"It bounds waiting for bytes that have not arrived"*.

`recv_record` calls `read_until` **twice** with one deadline — once for the
12-byte prefix (`net.rs:239`) and once for the body (`net.rs:289`). The deleted
block sat inside `read_until`, so deleting it lets the *prefix* call return
`Complete` past the deadline. Control then falls through `record_len`, the
`MAX_RECORD_LEN` check and `buf.resize(total, 0)` into the **body**
`read_until`, whose loop-top `remaining.is_zero()` check (`net.rs:305-310`)
fires and returns `TimedOut`. The buffer is cleared, `fail()` shuts the socket
down, the client is permanently terminal — with every byte of the record
already in the kernel receive queue. The record was not missing a byte; it is a
timeout; the doc says that cannot happen.

**Measured, not argued.** The instrument is the candidate's own
`buffered_peer` construction — `peek` confirms every byte is in the client's
receive buffer before `ImageClient` is constructed — over a record with a real
body (12-byte prefix + payload of `0x5a`). 200 calls per cell, reviewer's host:

| record | timeout | delivered on **base** `3a6400e` | delivered on **candidate** `e157774` |
| --- | --- | --- | --- |
| payload 1 B | 1 us | 0/200 | **0/200** |
| payload 64 B | 1 us | 0/200 | **0/200** |
| payload 4096 B | 1 us | 0/200 | **0/200** |
| heartbeat (0 B body) | 1 us | 0/200 | 200/200 |

and a timeout sweep, 100 calls per cell:

| record | 1 us | 2 us | 3 us | 5 us | 10 us | 20 us | 50 us | 100 us | 1 ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| heartbeat, **base** | 0 | 99 | 99 | 100 | 100 | 100 | 100 | 100 | 100 |
| heartbeat, **candidate** | **100** | 100 | 100 | 100 | 100 | 100 | 100 | 100 | 100 |
| payload 4096, **base** | 0 | 0 | 0 | 0 | 0 | 100 | 100 | 100 | 100 |
| payload 4096, **candidate** | **0** | **0** | **0** | **0** | **0** | 100 | 100 | 100 | 100 |

The bodied rows are **identical between base and candidate at every timeout
tested**. On the candidate the discarding error carries the loop-top message
`record deadline expired` with `buf.len()==0`, confirming it is the body call's
loop top and not a survivor of the deleted block. The heartbeat control
delivered 200/200 in the same process, which rules out the prefix call's own
loop top as the cause.

So the repair's entire measured effect is on records whose body is empty.
`src/imagery.rs:133` names those exactly: *"`heartbeat_record` and `bye_record`
frame no header and no payload"*. Two of the six `Record` variants. `Hello`,
`CameraDesc`, `ImageFrame` and `Other`-with-content — including the
payload-carrying record the crate exists for — are untouched, and the shipped
prose asserts a universal over all of them.

**Grade, and which rule was applied.** §7.1: *"**A finding is blocking if it is
*all of*:** **(a)** in code — the candidate tree, doc comments and rendered
rustdoc included — not only in the report; **(b)** reachable in normal
operation; **(c)** a false statement a later author would act on"*, read as
§7.1 requires: *"It is `(a) ∧ (b) ∧ (c)`, or `(d)` — not a flat disjunction."*

(a) is satisfied on the terms §7.1 spells out — a `//!` sentence and a rendered
rustdoc sentence. (c) is satisfied: a later author reading *"Only the second
case is a timeout"* concludes the discard-a-complete-record class is closed and
stops looking; the instrument they would forgo is the one this review ran.

§7.1 also requires the rule to be declared when the false-premise rule could
co-fire: *"(a) ∧ (b) ∧ (c) and the false-premise rule can both fire … Whichever
you apply, say which."* **(a) ∧ (b) ∧ (c) is applied**, following the recorded
precedent it names — `docs/work/riders-21/attempts/C1/review.md` F1, *"It is a
false statement about what an instrument covers, not a false premise under a
true conclusion"*. This sentence is the shipped specification of the API's
behaviour, not a reason offered for a measured result. This is also the
direction §7.1 flags as open — *"clause (c) has no direction"* — and it is
riders-21's direction: the sentence **over**-claims.

**(b), stated honestly rather than asserted.** What was *measured* is 1--10 us
record timeouts, where the failure is 100% of calls. Those are not production
timeouts. But they are the same timeouts the slice itself used to establish the
defect it repairs: the brief's scout swept 1--60 us, and the builder's own
reproduction is *"400 of 400 calls at a 1 us timeout"*. If the window is
reachable enough to justify the slice, it is reachable enough here — and
strictly more so, because the prefix/body boundary sits *earlier* in the call
than "the read that collects the last byte", so more ways of losing time reach
it. The production shape is not "timeout below 20 us" but "the remaining
deadline is zero when the prefix completes", which any scheduling stall
spanning the deadline produces at any timeout. No millisecond-scale instance
was constructed, and that last sentence is labelled an argument, not a
measurement.

**The input that breaks it:** any record with a non-empty body, every byte of
it already in the client's receive buffer, read with a record timeout short
enough that the prefix `read_until` returns at or after the deadline.
Concretely: `MAGIC ‖ kind=99 ‖ 0u16 ‖ payload_len=4096` followed by 4096 bytes,
`peek`-confirmed present, `recv_record(&mut buf, Duration::from_micros(1))` —
0 delivered in 200 calls, `TimedOut`, `buf` empty, client terminal.

The code comment at `:319-330` carries the same error in the place most likely
to stop the next repair: *"A second check here could therefore only differ from
the first by rejecting a read that **completed** the record — discarding bytes
already in hand and … ending the session with the whole record received."* For
the prefix call, the second check differs by rejecting a read that completed
the **prefix**, and removing it does not deliver the record. Counted as the
third location of B1 rather than a separate finding.

## MUST-FIX

### MF1 — `report.md` §8: *"The code path is the same `read_until` call with a different `destination`"*

Stated as fact and false. For a bodied record `recv_record` makes a **second**
`read_until` call, and that second call's loop-top check — not the deleted
block — is what destroys the record. The builder was right to refuse to call
the gap closed (*"'the same function with a longer slice' is an argument, not a
measurement, and I am recording it as one"* — the single most useful sentence
in the packet). But the premise it offers for why the gap is narrow is the
thing that made the gap look cosmetic.

§7.1: *"A false **premise under a true measured conclusion** is **must-fix** —
the result stands, the reason or the number offered for it does not."* The
conclusion it sits under — "I did not test this" — is true. **The false-premise
rule is applied**, and per §7.1's third-pairing ruling (*"a false conclusion
that touches no shipped line is `must-fix`, not blocking"*) a report-only
defect could not be blocking regardless.

### MF2 — `report.md` §5: the `git grep -i mutant` parenthetical

> *"there is no mutation harness checked into this crate (`git grep -i mutant`
> returns only prose in `tests/payload_robustness.rs`, `tests/robustness.rs`
> and `codebase-audit-docs/`)"*

`grep -ril mutant` over the full extracted candidate tree returns **nothing**.
The string "mutant" does not occur in the tree. The three files carry
`mutation` / `mutated` — which `mutant` does not match. The conclusion is true
(confirmed independently: no `cargo-mutants` config in `Cargo.toml` or
`.github/`), the instrument offered for it returns the opposite of what is
claimed. False premise under a true measured conclusion → **must-fix**.

### MF3 — `report.md` §3: *"the `deadline|elapsed` query adds only `src/lib.rs`"*

`git grep -l -E "Instant|deadline|elapsed" 3a6400e -- '*.rs'` returns **five**
files: `src/imagery/net.rs`, `src/lib.rs`, `src/net.rs`, `tests/imagery_net.rs`,
`tests/net_command.rs`. Relative to the three `Instant::now()` files it adds
`src/lib.rs` **and** `tests/imagery_net.rs`. As literally transcribed in the
report — without `-E` — the query returns nothing at all, since `|` is not an
alternation in `git grep`'s default mode. The §3 conclusion ("one site")
survives: `src/net.rs::exchange_with_deadline` was read independently and the
builder's description of it is exact — one `remaining(deadline)?` at the loop
top, immediate return of a matching datagram, `continue` arms that re-enter.
False premise under a true conclusion → **must-fix**.

### MF4 — the commit message subject over-claims, and is frozen

`fix(imagery/net): deliver a record whose last byte lands on the deadline`.
Measured, it delivers such a record only when that last byte completes the
**final** `read_until` destination — i.e. only for `Heartbeat` and `Bye`. The
body also says *"discarding a record every byte of which was already in hand"*
is what the change stops; for four of six record kinds it does not stop it. Not
code, so (a) fails and this cannot be blocking; false-premise rule,
**must-fix**. Per `AGENTS.md` §7 this is repairable only in a later message —
*"What it finds is repaired in the **next** commit's message, never by amending
the candidate: the SHA is the identity."*

## NOTES

**N1 — the flake bound. Verified by construction, twice; the claim holds.** Not
relied on a single run.

- *Source-side* (the decisive one, run against the builder's actual shipped
  test): `recv_record` patched in a copy of the candidate tree to `sleep(1 ms)`
  immediately after computing `deadline`, making the mid-read window
  unreachable. `a_fully_buffered_record_is_delivered_when_its_deadline_expires_mid_read`
  **FAILED**, at `tests/imagery_net.rs:739`, with `none of 64 attempts reached
  the mid-read window; 64 expired at the loop top`.
- *Input-side*: a replica of the test at `Duration::from_nanos(1)` gave
  `delivered=0 loop_top=64` and panicked with the same message.

It fails loudly and names the cause. It cannot pass vacuously. The builder's §8
self-warning about a host where the window never opens is accurate and
correctly phrased.

**N2 — the mutant polarity. What was done is stronger than a kill, not
weaker.** The builder is right that "kill the mutant" loses its usual meaning
when the repair *is* the mutation, and the substitute is the right one.
Re-derived from scratch: reinstating the block in a copy of the candidate tree
gives **176 passed, 1 failed, total 177**, the single failure being the new
test — matching `gates.md` exactly, including the total, so no run measured a
different suite. The pre-existing suite against the repaired source gives **175
passed, 0 failed** (= 177 − 2), confirming the scout's survival claim: the
mutation was invisible to the suite as it stood. What a killed mutant
establishes is "a test discriminates these two source states"; that is exactly
what these runs establish, in the only direction available. The substitution is
accepted. The builder's framing of the alternative — that under the guard
repair the mutant would be killable only by minting `"record deadline expired
while receiving bytes"` as an API contract — is correct, and declining to do
that was the right call.

**N3 — the deletion-safety attack. Negative, with its population.** No input or
interleaving was found where the deleted block refuses something the loop-top
check admits, *within `read_until`*. What was tried: the `Ok(0)` arm returns
before either check; `Err(Interrupted)` falls to the loop top, which checks;
`Err(WouldBlock)` returns `TimedOut` regardless; `Err(other)` returns. That
leaves only `Ok(count)`, where the post-increment state is either `*read <
destination.len()` — the loop re-enters and `saturating_duration_since` yields
zero on a monotonic `Instant`, so `TimedOut` before any further syscall — or
`*read == destination.len()`, where the loop exits. **The difference is exactly
`{destination completed}`, and nothing else.** Measured confirmation: the 5-of-12
partial case in the report reproduces
(`an_incomplete_record_past_its_deadline_still_fails_and_is_terminal` passes on
base and candidate), and a byte-drip probe on the base took the loop-top path 0
times in 20.

But "destination completed" is not "record completed", and that is where B1
comes from. The builder's argument is sound about `read_until` and was never
lifted to `recv_record`. The `--all-targets`-visible consequence is that the
deletion is safe; the doc sentences generalise it to a scope it does not have.

**N4 — the repair moves one allocation inside the deadline, and the guard
variant would too.** On the base, a prefix completing past the deadline
returned before `record_len`, before the `MAX_RECORD_LEN` check and before
`buf.resize(total, 0)`. On the candidate it reaches all three, so a peer that
gets a prefix declaring up to `MAX_RECORD_LEN = 64 << 20` across the deadline
now causes a 64 MiB zero-filling allocation before the body loop top fails it.
One per connection, since receive failures are terminal. Not a reason to
reject, and the `guard` alternative is identical here — but `report.md` §2's
*"Guard and delete are identical in `ErrorKind`, in `buf` (empty), in terminal
state, and in the number of syscalls"* is a base-vs-repair comparison the table
does not make, and this is the axis it omits.

**N5 — the brief. Three defects beyond the one the builder refuted.** All three
point the same way, at B1.

- **§3 defines the sweep shape as *"check a deadline after a read without
  asking whether the read finished"*.** That definition is a closed query: the
  second instance of the defect is a deadline check that runs *before* a read,
  so no sweep obeying §3 could return it. The brief told the builder to
  *"Search for it before you conclude there is one site"* and then handed over
  the one shape that guarantees the miss. The site it missed is eight lines
  from the site it found, in the same function's caller.
- **§2 offers a binary choice set** — *"guard the check (`*read <
  destination.len() && …`) or … delete the block"* — and both members leave
  bodied records exactly as broken as the base. The repair that would close the
  class (asking whether the *record*, not the destination, is outstanding) is
  outside the set the brief permitted.
- **§4 says *"Pre-buffer a complete record"*** without saying "and one with a
  body". The builder's §8 flagged the omission anyway, and was right to.

The sentence the builder refuted (§4's *"byte_drip … goes through the loop-top
path, not this one"*) was re-run independently on the base: **19 of 20** through
the mid-read branch, **1 of 20** through `WouldBlock`, **0 of 20** through the
loop top. The load-bearing half — zero loop-top — reproduces exactly; the
builder's "20 of 20" is a per-host count, not a property, and its own candidate
column already showed the same jitter. Note only.

**N6 — every number. What reproduced.** All on the `git archive 1a1c9bc | tar
-x` extraction, in isolated target dirs.

Reproduced exactly: `cargo fmt --all --check` rc 0 · clippy rc 0, no warnings ·
`test --all-features` **177 passed / 0 failed** over 16 targets, and the
per-target breakdown in `gates.md` is right cell by cell (5, 23, 7, 11, 28, 6,
18, 12, 11, 6, 4, 6, 9, 17, 4, 10 = 177) · `test --locked` **99** · MSRV
`+1.92` rows **99** and **177** · all six `features` rows **127 / 121 / 101 /
101 / 173 / 103** · **all 16 `feature-docs` rows**, `check_rc=0 doc_rc=0` each,
matching the workflow's 16-row `include:` list · `cargo-deny` rc 0 with exactly
the two pre-existing `license-not-encountered` warnings · `cargo-machete` clean
· mutant run 176/1/177 · pre-existing-suite-only 175 · base `imagery_net` 16,
candidate 18 · 30 tracked `.rs` files split 15/14/1 · seven `Instant::now()`
sites on base, distributed 3/2/2 as stated · `"while receiving bytes"` exactly
one hit on base, zero on the candidate · `zz_riders67` absent from the tree ·
`msg.txt` present, **2147 bytes**, sha256 `145b7774…6b01d6` as reported, and
`git log -1 --format=%B` is 2148 bytes, the one-byte difference being the
trailing newline.

Both coordinator-named checks pass: **all 16 `feature-docs` rows really were
runnable and all really are rc 0**, and **the clean `git archive` extraction
really does build and pass** — the entire gate re-derivation ran on that
extraction, 177/0.

Could not re-derive, and why: the scout's `2358 / 0 / 42` and the builder's
`2360 / 0 / 40` (different hosts, different instants — the direction reproduced:
base, heartbeat, 1 us, 200 calls, 0 delivered) · `400 of 400` and `load average
8.22 on 32 cores` (host/time-bound) · the `40 passed / 0 failed` and `20 failed
/ 0 passed` repetition counts (the check-present case was run once and it
failed; not repeated 40x or 20x) · `gates.md` #1's fmt *history* (first
invocation rc 1, then corrected) — only the final rc 0 is checkable · the
`Compiling natnet: 0 / 1 / 1` invalidation counts (sidestepped by rebuilding
the mutant from scratch in a fresh target dir, which reproduces the verdict).

## Verdict

**Reject.**

The shipped code change is correct as far as it goes and nothing was found
wrong with it: within `read_until` the deletion is provably equivalent to the
guard, the tests are honest, the bound is real, the mutant substitution is
sound, and every gate number in the packet reproduces. This is a careful
attempt and the builder's §8 named the exact hole it fell into.

But the packet ships a universal statement of the deadline contract in `//!`
and rustdoc that is false for four of the six record kinds, including
`ImageFrame`; the candidate does not change the delivered/discarded outcome for
a single bodied record at any timeout measured; and the code comment sitting
where the block used to be tells the next reader the class is closed. B1 is
blocking on (a) ∧ (b) ∧ (c), so §9's *"an independent `accept` review with no
unresolved blocking finding"* is not met.

The cheapest disposition for C2: the repair the brief's choice set excluded —
carry "is the record still outstanding?" across both `read_until` calls, or
check the deadline once at the top of `recv_record` rather than at the top of
each destination — plus the bodied-record instrument, which is about 40 lines
and is already written. It is at
`/home/andrej/coord_scratch/review67/work/tests/zz_review67.rs` and
`zz_sweep67.rs`; the mutant, window-closed and base trees are at
`/home/andrej/coord_scratch/review67/{mutant,nowindow,base,pre}`. Nothing in
`natnet` or in the builder's worktree was modified.
