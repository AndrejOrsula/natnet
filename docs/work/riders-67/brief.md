# riders-67 — `recv_record` throws away a record it has fully received, and kills the session doing it

**Slice:** riders-67, attempt **C1**. **Repository: `natnet`**, not `auga`.
**Base = `dev`** (`3a6400e` when I resolved it on 2026-09-13 — resolve it
yourself and say what you got). Branch `slice/riders-67`, worktree
`/home/andrej/ws_rs/natnet_wt/impl-riders-67`. Freeze **C1**. Access `remote`.

`natnet` is a sibling repository of the `auga` effort and carries no
`AGENTS.md` of its own. The process rules that bind you are in
`/home/andrej/ws_rs/auga/AGENTS.md` — read §4 (Workflow), §5 (Git), §6 (Gates)
and §7 (Reporting) — but **`natnet`'s gate list is `natnet`'s own**, in
`.github/workflows/rust.yml`, not `auga`'s. **A sibling repository has its own
corpus; do not import a rule from `auga` that contradicts something written
here.** If you find such a contradiction, report it rather than resolving it.

## 1. The finding, which is a scout's and not yet yours

A read-only scout reported this and I confirmed the code shape myself. **Neither
of us wrote the test. Reproduce it before you change a line**, and report the
reproduction as its own result — if it does not reproduce, that is the finding
and the slice stops there.

`Reader`-side: `src/imagery/net.rs`, `read_until`. After a successful read the
function checks the deadline **without re-testing whether the read completed**:

```rust
Ok(count) => {
    *read += count;
    if Instant::now() >= deadline {
        return Err(io::Error::new(
            ErrorKind::TimedOut,
            "record deadline expired while receiving bytes",
        ));
    }
}
```

When the syscall that delivers the **last** byte of a record returns at or
after the deadline, the caller gets `TimedOut`, the buffer is cleared, and
`fail()` shuts the connection down — the client is permanently terminal —
even though every byte of the record was in hand.

The scout's instrument: a 12-byte `imagery::heartbeat_record()` (`total ==
PREFIX_LEN`, so the whole record is the prefix) written to the socket and left
in the client's receive buffer for 2 ms, then read with timeouts swept from
1 µs to 60 µs, 40 runs each.

```
  Ok(true) (record delivered)                : 2358
  Err before any read (loop-top deadline)    : 0
  Err AFTER a complete read (mid-read check) : 42
  first mid-read hit: timeout=1us  err="record deadline expired while receiving bytes"  buf.len()=0
```

The distinct message string is what separates this branch from the loop-top
check; `buf.len()==0` is the record being discarded. With the block deleted,
the same sweep gave `2400 / 0 / 0`.

**Numbers in this brief are the scout's. Re-derive them or refute them.**

## 2. The question I am not deciding for you

The block is either **redundant** or **harmful**, and the scout argues it is
never useful: if bytes remain, the loop re-enters and the loop-top
`remaining.is_zero()` check returns `TimedOut` anyway, with the same
`ErrorKind` and a different message. On that reading the only behaviour unique
to this branch is turning a complete read into a failure.

**Decide whether the repair is to guard the check (`*read <
destination.len() && …`) or to delete the block, and say why.** They are not
the same change: one keeps a second notation of the deadline and one removes
it, and the error *message* a caller sees differs. Show what each does to the
observable behaviour, and state which callers can tell them apart.

**Do not weaken the deadline.** A record that genuinely does not arrive in time
must still fail, and the client must still become terminal on a real timeout.
Show that separately, with a test.

## 3. Sweep, do not spot-fix

The same shape — *check a deadline after a read without asking whether the read
finished* — may appear elsewhere in this crate. **Search for it before you
conclude there is one site**, with an open query rather than a list of the
functions you already suspect, and report what the search covered. One strike
is not a discharge.

## 4. What the test must do

- **Fail before your change and pass after it.** Show both runs, with output.
- Pre-buffer a complete record, call with a deadline that has already expired
  or is about to, and assert the record is **delivered**.
- Bound the flake: this is a timing test on a loopback socket, and this project
  has just spent four days on a loopback race in a sibling repository. Make it
  deterministic or make it bounded-retry with a stated bound, and **say which
  and why**. A test that passes because it got lucky is worse than none.
- Add the coverage the scout found missing: mutation `deadline-midread-off`
  **survived the entire suite**, and
  `tests/imagery_net.rs::byte_drip_cannot_restart_the_total_record_deadline`
  goes through the loop-top path, not this one. Your test must kill that
  mutant. **Show the mutant killed**, by running it, not by arguing it.

## 5. Scope

- Owned: `src/imagery/net.rs` and the tests that cover it.
- **Touch no other module** unless the §3 sweep finds the same defect there, in
  which case report it first and fix it in the same attempt only if it is the
  same one-line shape.
- **Do not edit `codebase-audit-docs/`.**
- **Do not commit to `docs/work/`.** Draft `report.md` and `gates.md` under
  `docs/work/riders-67/attempts/C1/` in your worktree and leave them
  **uncommitted** for the coordinator. A commit cannot contain a report citing
  its own SHA.
- **Never push.**

## 6. Gates live for you — `natnet`'s, from its own workflow

Run what `.github/workflows/rust.yml` runs, and report each with its output,
never with the word "green":

- `cargo fmt --all --check`
- `cargo clippy --locked --all-targets --all-features -- -D warnings`
- `cargo test --locked --all-features` and the default-feature run
- `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps` **per feature row**
  — this crate's `feature-docs` job covers **all 16 combinations of the four
  flags**, and a partial-feature doc break is invisible to `--all-features`.
  You need not run all 16 by hand; run the rows your change can reach and say
  which those are and why.
- `cargo deny check advisories bans licenses sources` if your change touches
  the dependency graph. It should not.

Use an isolated target directory: `export
CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67/target`.

## 7. Report

Base resolved and when; the reproduction (or its failure); the §2 decision with
its reason; the §3 sweep with its query and population; the tests, with the
before/after runs and the killed mutant; every gate with its numbers;
`candidate(C1)` commit SHA and tree hash; and what you are unsure about.
