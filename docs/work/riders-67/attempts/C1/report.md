# riders-67 C1 — builder report

Repository `natnet`. Branch `slice/riders-67`, worktree
`/home/andrej/ws_rs/natnet_wt/impl-riders-67`.

## 0. Identity

| | |
|---|---|
| Base | `dev` = `3a6400efb02e27bc8ab507ee5985624a5655e823`, resolved by me at **2026-09-13T01:00:28+02:00** (`git -C /home/andrej/ws_rs/natnet rev-parse dev`); the same value the brief recorded |
| Base shape | one parent (`3a6400e^` = `587826496e327e380f305e41f4d90306397f06ff`) |
| `candidate(C1)` SHA | `e157774c85a425a84eee00fff188d739bcf64c39` |
| `candidate(C1)` tree | `1a1c9bc69d344fe4f8c85b70217a687320eb2893` |
| Files changed | `src/imagery/net.rs`, `tests/imagery_net.rs` (2 files, +128 −12) |
| Commit message file | `/home/andrej/coord_scratch/riders67/msg.txt`, 2147 bytes, sha256 `145b7774d4d532ec7ef9e449c80eafc89c49a0473c0e3e02e38297d6ee6b01d6`, committed with `git commit -F` |
| Pushed | no. No `git push` was run in this slice, to any remote |
| Maturity | complete; no blockers |

Toolchain: `rustc 1.97.1 (8bab26f4f 2026-07-14)` via `rust-toolchain.toml`
(`channel = "stable"`), MSRV rows on `cargo 1.92.0 (344c4567c 2025-10-21)`.
All cargo calls ran with `CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67/target`
and the binary named by absolute path `/home/andrej/.cargo/bin/cargo`.

## 1. The reproduction, before a line was changed

It reproduces. The scout's instrument re-derived on this host against the
**unmodified base tree**: a 12-byte `imagery::heartbeat_record()` written to a
loopback peer, left in the client's receive buffer for 2 ms, then read with
`recv_record` at record timeouts swept 1 µs…60 µs, 40 runs each (2400 calls):

```
--- riders-67 sweep: 1us..=60us, 40 runs each ---
  timeout=1us  delivered=0  loop_top=0  mid_read=40
  Ok(true) (record delivered)                : 2360
  Err before any read (loop-top deadline)    : 0
  Err AFTER a complete read (mid-read check) : 40
  Other                                      : 0
  first mid-read hit: Some("timeout=1us record deadline expired while receiving bytes  kind=TimedOut  buf.len()=0")
```

The scout reported `2358 / 0 / 42` over the same sweep; I measured
`2360 / 0 / 40` — the same three buckets, the same discriminating message, and
the same `buf.len()==0`. The two extra hits the scout saw were at timeouts
above 1 µs; on this host every hit was at 1 µs exactly. Concentrating the
sweep there, over **400 calls at a 1 µs timeout** (not 40):

```
  timeout=1us  delivered=0  loop_top=0  mid_read=400
  timeout=2us  delivered=373  loop_top=0  mid_read=27
  timeout=3us  delivered=396  loop_top=0  mid_read=4
```

So on this host the defect is not rare: at a 1 µs record timeout a fully
buffered record is destroyed on **400 of 400** calls, and the loop-top check
never once fires first. The system was under load average 8.22 on 32 cores.

Every count above is over calls to `ImageClient::recv_record` on a freshly
adopted loopback client, one call per run, with the complete record already
written by the peer.

## 2. The decision: delete the block, not guard it

**Deleted.** The reason is not that the guard is wrong — it is that the guard
keeps a second notation of one deadline whose only observable is an error
message that nothing in this crate reads.

### What each does, measured rather than argued

I built all three source states and ran two instruments against each. The
second instrument is the case where the two repairs can differ at all: only
**5 of the 12** prefix bytes are pre-buffered, the peer sends no more, and the
read that lands past the deadline therefore did *not* finish the record.

| instrument | base | guard (`*read < destination.len() && …`) | delete |
|---|---|---|---|
| complete record buffered, 1 µs, 400 calls | 0 delivered, **400** `TimedOut` "…while receiving bytes", `buf.len()==0` | **400 delivered** | **400 delivered** |
| 5-of-12 buffered, 1 µs, 200 calls | 200 × `TimedOut` "record deadline expired while receiving bytes" | 200 × `TimedOut` "record deadline expired **while receiving bytes**" | 200 × `TimedOut` "record deadline expired" |

Guard and delete are identical in `ErrorKind`, in `buf` (empty), in terminal
state, and in the number of syscalls. They differ in exactly one byte string.

### Why the loop-top check already covers it

`Instant` is monotonic. If `Instant::now() >= deadline` holds where the deleted
block stood, then at the loop top — a strictly later instant —
`deadline.saturating_duration_since(Instant::now())` is zero and the function
returns `TimedOut` before `set_read_timeout` or any further read. The deleted
check could therefore never produce a failure the loop-top check would not
produce, **except** in the one case where the loop is not re-entered: when the
read just completed the record. That case is the defect. The row above is the
measurement of that argument: with the block deleted, the partial case still
fails 200/200.

### Which callers can tell the two repairs apart

- A caller matching on `error.kind()`: **cannot**. `TimedOut` either way.
- A caller reading `error.to_string()` or logging it: **can** — this is the
  only difference. The terminal state stores `message`, so every later
  `recv_record` on the dead client replays the same difference.
- A caller reading `raw_os_error()` or downcasting to `natnet::Error`:
  **cannot**; both are `None` on this path either way.
- Inside this repository, the set of readers of that string is empty. Before
  the change, `git grep "while receiving bytes"` returned exactly one line —
  its own definition. No test, no doc comment, no README, no
  `codebase-audit-docs/` page mentioned it.

A second notation of a deadline that no instrument reads can only drift from
the first. Deleting it is what makes the remaining notation the only one.

### The deadline is not weakened

A record still missing bytes when the deadline passes fails exactly as before:
`TimedOut`, buffer cleared, client terminal. That is asserted by the new
`an_incomplete_record_past_its_deadline_still_fails_and_is_terminal` (§4) and
by the two pre-existing tests `record_deadline_is_total_and_timeout_is_terminal`
and `byte_drip_cannot_restart_the_total_record_deadline`, which pass unchanged.

The rule is now written down rather than implied — the module docs and
`recv_record`'s doc comment say the deadline bounds *waiting* for bytes that
have not arrived, and the code carries a comment saying why there is no check
there, so the block is not re-added by the next reader who notices its absence.

## 3. The sweep: one site, and the query has a positive control

**Query.** Not a list of suspected functions. The defect needs a clock, so I
enumerated every clock read in the repository and read each one:

```
git grep -n "Instant::now()" -- '*.rs'       → 7 sites
git grep -n "SystemTime|Instant|.elapsed()|monotonic" -- src/ examples/
git grep -l "Instant|deadline|elapsed" -- '*.rs'
```

**Population.** All 30 tracked `.rs` files (15 `src/`, 14 `tests/`, 1
`examples/`); no tracked `.rs` file lies outside those three directories, and
`codebase-audit-docs/` holds none. The seven `Instant::now()` sites are
`src/imagery/net.rs` ×3, `src/net.rs` ×2, `tests/net_command.rs` ×2; the
`deadline|elapsed` query adds only `src/lib.rs`, whose single hit is the prose
"TCP client with bounded records and deadlines."

**Result: one site.** The crate has exactly two deadline-bounded receive loops.
The other is `src/net.rs::CommandClient::exchange_with_deadline`, and it has
the *correct* shape: its `remaining(deadline)?` check is at the loop top only,
and a datagram that matches source, length and message id is returned
immediately with no further clock read. Its `continue` arms — foreign source,
short datagram, wrong id — re-enter the loop and re-check, which is right,
because in those arms the operation is not finished.

That second loop is the reach check: the query returned an artifact where a
second instance *could* have lived and I read it, so the negative is a finding
rather than a vacuous oracle.

## 4. The tests

Both are in `tests/imagery_net.rs`, behind the file's existing
`#![cfg(all(feature = "imagery", feature = "net"))]`.

`a_fully_buffered_record_is_delivered_when_its_deadline_expires_mid_read` —
pre-buffers a complete heartbeat record and asserts it is delivered under a
1 µs deadline.

`an_incomplete_record_past_its_deadline_still_fails_and_is_terminal` —
pre-buffers 5 of the 12 prefix bytes and asserts `TimedOut`, an empty buffer,
and a terminal client. This one passes on the base too, by design: it is the
ablation that proves the repair did not blanket-disable the deadline.

Both use a new `buffered_peer` helper.

### How the flake is bounded, and why that shape

Two timing facts are involved and they are handled differently.

1. *Are the bytes there?* Made **deterministic**, not slept on. `buffered_peer`
   confirms every pre-buffered byte is in the client's receive buffer with
   `TcpStream::peek` — which consumes nothing — before the client is
   constructed, under a 5 s bound whose expiry is an explicit assertion. When
   `recv_record` is called the data is already present, so nothing in either
   test depends on how fast loopback is. The scout's 2 ms sleep is replaced by
   this.
2. *Does the 1 µs deadline fall before the first read or during it?* This one
   cannot be made deterministic without changing the shape of the code under
   test, so it is **bounded retry with a stated bound**: `ATTEMPTS = 64`. A run
   in which the deadline expired at the loop top is a legitimate timeout, is
   counted, and is retried; the test fails if no attempt ever reaches the
   window, so it cannot pass by never testing anything. The bound is set
   against a measured rate: 0 loop-top hits in 840 measured calls at 1 µs
   (400 + 400 + 40), and 40/40 consecutive passes of the finished test.

Measured stability of the finished test, run alone, 40 consecutive
invocations on the candidate: **40 passed / 0 failed**. Run as part of the
parallel suite it has passed in every suite run recorded in `gates.md`.

Failure mode if someone later edits the loop-top message string: the retry arm
matches `error.to_string() == "record deadline expired"` exactly, so a changed
message turns a retry into a loud panic naming the message it saw. It fails
safe rather than silently passing, but it is a real coupling and is called out
here as one.

### Before and after

Before — new tests against the **unmodified** base source:

```
running 4 tests
test an_incomplete_record_past_its_deadline_still_fails_and_is_terminal ... ok
test a_fully_buffered_record_is_delivered_when_its_deadline_expires_mid_read ... FAILED
test record_deadline_is_total_and_timeout_is_terminal ... ok
test byte_drip_cannot_restart_the_total_record_deadline ... ok

---- a_fully_buffered_record_is_delivered_when_its_deadline_expires_mid_read stdout ----
thread '...' panicked at tests/imagery_net.rs:730:27:
a complete record was received and then discarded: record deadline expired while receiving bytes (kind TimedOut, buf.len() 0)

test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 14 filtered out
```

After — same tests on the candidate:

```
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s
```

## 5. The mutant, killed by running it

`deadline-midread-off` is the scout's label for a hand-applied edit; there is
no mutation harness checked into this crate (`git grep -i mutant` returns only
prose in `tests/payload_robustness.rs`, `tests/robustness.rs` and
`codebase-audit-docs/`). The coordinator confirmed this mid-slice.

**A polarity that matters, stated plainly.** The mutation *is* disabling the
mid-read check, and the repair I chose *is* disabling the mid-read check. After
this change the mutant is not a variant of the shipped code — it is the shipped
code. So "kill the mutant" is discharged in the only direction that has
meaning: the two source states differ by that block, my suite passes on the
state without it and fails on the state with it, and the pair of runs that
proves it is the pair below. Under the guard repair the mutant would have
survived as a live variant, killable only by a test asserting the exact string
`"record deadline expired while receiving bytes"` — minting an error-message
contract in order to have something to test. I judged that the wrong trade and
am flagging the judgement, not hiding it.

**Re-derivation of the scout's survival claim** — the check disabled, the
**pre-existing** suite only (my tests reverted out of the tree),
`cargo test --locked --all-features`: 16 targets, **175 passed, 0 failed**.
The mutation is invisible to the suite as it stood. Confirmed, not assumed.

**The kill**, with NULL controls interleaved and `--no-fail-fast`, per
`AGENTS.md` §6 (*"a NULL control belongs in the protocol, not beside it"* and
*"Read the control's total, not only its verdict"*):

| run | source state | rc | passed | failed | pass+fail |
|---|---|---|---|---|---|
| NULL control A | candidate | 0 | 177 | 0 | **177** |
| mutant | mid-read check reinstated | 101 | 176 | 1 | **177** |
| NULL control B | candidate restored | 0 | 177 | 0 | **177** |

The single failure is
`a_fully_buffered_record_is_delivered_when_its_deadline_expires_mid_read`.
Totals agree across all three runs, so no run measured a different suite than
the controls did. The build-invalidation trap in `AGENTS.md` §6
(*"Byte-identical source is not sufficient; the build must be invalidated
too"*) is discharged by construction: the mutation script rewrites the file
in place and calls `os.utime(path, None)`, and the mutant and restore runs
each show a `Compiling natnet v0.0.0` line (1 and 1) while the already-built
control shows 0.

Run alone, 20 consecutive invocations with the check present: **20 failed /
0 passed**. The kill is not a coin flip.

## 6. What in the brief was wrong

**§4's claim about which path the existing test takes is refuted.** The brief
says `tests/imagery_net.rs::byte_drip_cannot_restart_the_total_record_deadline`
*"goes through the loop-top path, not this one."* I ran that test's exact body
20 times on the base source, printing the message instead of only the kind:

```
######## base shape (mid-read check present) ########
   20x  TimedOut | record deadline expired while receiving bytes
######## candidate (check deleted) ########
    2x  TimedOut | Resource temporarily unavailable (os error 11)
   18x  TimedOut | record deadline expired
```

On the base it went through the **defective branch on 20 of 20 runs** and
through the loop-top path on none. The arithmetic is straightforward once
looked at: the drip delivers a byte every 40 ms against a 120 ms deadline, so
the fourth byte arrives at the deadline, the socket read timeout and the data
race each other, and whichever wins, the loop top is not where the call ends.

This changes the diagnosis, and in a way worth keeping. The mutant did not
survive because no test reached the branch — a test reached it every single
run. It survived because the test that reached it asserts `error.kind()`, and
all three exits from that function carry `TimedOut`. The missing coverage was
never reach; it was a **discriminating** assertion. That is also the reason the
brief's own §1 instrument had to lean on the message string to separate the
branches.

Two smaller ones:

- **§1's numbers.** `2358 / 0 / 42` on the scout's host; `2360 / 0 / 40` on
  mine. Not a refutation — same shape, same discriminator, the difference is
  where in the sweep the hits fall. Recorded because the brief said to
  re-derive or refute.
- **§4's "Your test must kill that mutant"** is not satisfiable as written
  under the repair §2 asks me to consider, for the polarity reason in §5.
  Reported rather than resolved silently.

`auga`'s `AGENTS.md` and `natnet`'s corpus: **no contradiction found**. The two
touch in one place — `AGENTS.md` §6's gate list is `auga`'s workspace list and
names packages (`auga_core`, `auga_bench`) and scripts that do not exist here;
`natnet`'s gate list is `.github/workflows/rust.yml`, and that is the list I
ran (`gates.md`). That is the brief's own instruction, not a conflict. The
`AGENTS.md` clauses I did import are the ones about *method* — the NULL
control, the totals check, and mtime invalidation in §6, and §7's reporting
shape — none of which `natnet` contradicts.

## 7. Decisions the brief did not cover

1. **Documenting the rule, not only removing the check.** The brief scoped the
   repair to one block. I also added three sentences to the module docs and two
   to `recv_record`'s doc comment stating that the deadline bounds waiting, and
   an eleven-line comment where the block stood. Without them the next reader
   sees a post-read path with no deadline check and restores the defect. Both
   files are inside the brief's owned scope. No intra-doc link was added, so no
   rustdoc row can break on it — and all 16 doc rows were run anyway.
2. **`peek` instead of the scout's 2 ms sleep.** Strictly stronger: it removes
   the "were the bytes there yet" question from the test entirely instead of
   making it improbable.
3. **Holding the peer stream alive.** `buffered_peer` returns it and the tests
   bind it as `_peer`. Dropping it would turn the incomplete-record test from a
   timeout into a clean EOF and quietly test something else.
4. **Not asserting the loop-top message as a contract.** The test compares it
   only to classify a retry, and §2 deliberately declines to mint the other
   message as an API guarantee.
5. **Running all 16 `feature-docs` rows** rather than the 4 my change can reach
   (`imagery,net` + the two three-flag rows containing both + `all`). They were
   cheap; a partial-feature break is exactly the thing that is invisible to
   `--all-features`, and I would rather report 16 rc=0 than an argument about
   which four mattered.
6. **`cargo-machete` was not installed on this host**, so I installed 0.9.2 to
   `/home/andrej/coord_scratch/riders67/machete` and ran it rather than
   skipping the gate. It reports no unused dependencies. Nothing was installed
   into the repository or into `~/.cargo/bin`.

## 8. What I could not verify, and what I am unsure about

- **The timing window is a property of this host, not of the code.** Every
  reproduction number here (400/400, 40/40, 20/20) was measured on one
  32-core Linux machine under load average ~8, in a `debug` build. On a
  machine where `set_read_timeout` + `read` complete inside 1 µs, the new
  test's window would never open and it would fail on its `delivered > 0`
  assertion — a loud failure, not a silent pass, and the message names the
  loop-top count so the cause is readable. I did not run it on any other
  host, in `--release`, or on a non-Linux target. **This is the thing I would
  look at first if CI ever goes red here**, and it is the reason the assertion
  is phrased as "no attempt reached the window" rather than as a pass.
- **I did not test the case the defect would bite in production**: a record
  whose *body* — not its prefix — is completed by a read landing past the
  deadline, i.e. a real `ImageFrame` rather than a 12-byte heartbeat. The code
  path is the same `read_until` call with a different `destination`, and the
  brief's instrument and the scout's both use the heartbeat, but "the same
  function with a longer slice" is an argument, not a measurement, and I am
  recording it as one.
- **`Instant` monotonicity is the load-bearing assumption** of the §2
  argument. Rust's `Instant` guarantees a monotonically nondecreasing clock
  across platforms, which is what the argument needs; I did not attempt to
  construct a violation, and if one existed the consequence would be a
  `recv_record` that hangs up to one full `remaining` past its deadline, not a
  lost record.
- **The `--no-fail-fast` totals check is sound only across the runs I paired.**
  The three-run table in §5 shares one command line; the earlier per-row
  feature runs in `gates.md` do not, and their differing totals are feature
  gating, not truncation.
- **`cargo-deny` emitted two `license-not-encountered` warnings** for
  `BSD-2-Clause` and `BSD-3-Clause` in `deny.toml`. They are pre-existing, the
  run exits 0, and my change touches no manifest (`git status` on
  `Cargo.toml`, `Cargo.lock`, `deny.toml` is empty), so they are not mine — but
  they are in the output and I am not going to let them read as clean.
- **I could not run the `codebase-audit-docs/` cross-check as an instrument.**
  I read it only to confirm it says nothing about the record deadline
  (`grep -rn "record deadline\|record_timeout"` over `README.md` and
  `codebase-audit-docs/` returns nothing), and I edited nothing there.

## 9. Refuted-citation sweep

`natnet` carries no `scripts/refuted_citation_sweep.py`; the three legs were
run by hand against `3a6400e e157774` for the literal the change removes,
`"record deadline expired while receiving bytes"`:

```
leg 1  git diff 3a6400e..e157774           → 1 record, a deletion:
         39:-                            "record deadline expired while receiving bytes",
leg 2  git grep -n "…" e157774              → (no hit in tree at candidate)
leg 3  git log 3a6400e..e157774 --format=%B → (no hit)
leg 3 widened  git log 3a6400e^..e157774    → (no hit); range is 2 commits
```

Base shape: **one parent**, so per `AGENTS.md` §7 the widened leg adds exactly
`3a6400e`'s own message and nothing else; it carries the token nowhere. Leg 2
is the one that matters here and it is empty: the removed string survives
nowhere in the candidate tree. The commit message was drafted to
`/home/andrej/coord_scratch/riders67/msg.txt` (2147 bytes, sha256 above),
`grep -F`-checked, and committed with `git commit -F`; the file is kept for the
reviewer, which makes the step checkable now and falsifiable later, and is all
it can do.

## 10. Scope compliance

- `docs/work/` : **not committed**. This report and `gates.md` are untracked
  files in the worktree, left for the coordinator. `git status --porcelain`
  after the candidate commit was empty of tracked changes.
- `codebase-audit-docs/` : not edited (read only).
- Modules other than `src/imagery/net.rs` : not edited. The §3 sweep found no
  second site, so the brief's conditional licence to touch another module was
  never exercised.
- Temporary instruments were removed before the candidate commit and are not
  in the tree at `e157774`: `git grep -l zz_riders67 e157774` returns nothing.
  Both are kept unversioned as `/home/andrej/coord_scratch/riders67/`
  `zz_riders67_sweep.rs` (§1 and §2's sweeps) and `zz_riders67_drip.rs` (§6's
  probe); copying either into `tests/` re-runs it.
- No `git push`, no publication, no part of this crate sent anywhere.
