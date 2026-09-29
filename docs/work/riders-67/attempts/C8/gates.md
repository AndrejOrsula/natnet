# riders-67 C8 — gates

**Tree under test: `candidate(C8)` = `652039bc7db3605f66657cb3986edf32d09dec26`,
tree `e311a0f09b3fe9ed1aafc74c627776a7b40548e5`.**

Script `/home/andrej/coord_scratch/riders67c8/gates.sh` — C7's, path-substituted
to `riders67c8` and `impl-riders-67-C8`, with one line changed: `cargo-deny`'s
fallback path. `diff` against C7's script is those substitutions and that line
and nothing else. `CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c8/target`
throughout; `target_base` for the BASE comparison. Per-gate outputs `g_*.txt`.

## Four full runs. The shipped figures are run 4's.

| run | log | tree it ran against | loadavg open → close | TIME_WAIT open → close |
|---|---|---|---|---|
| 1 | `gates.log` | **mixed — superseded, do not cite** | 12.0 → 12.07 | 3 → 1768 |
| 2 | `gates2.log` | `869bb7c5` (commit 2) — superseded by commit 3 | 11.65 → 11.95 | 3 → 1219 |
| 3 | `gates3.log` | `8a5f4c45` (commit 3) — superseded by commit 4 | 11.29 → 11.60 | 4 → 1322 |
| **4** | **`gates4.log`** | **`e311a0f0`, the shipped tree, frozen before it started** | **11.57 → 11.95** | **4 → 1335** |

**All four agree.** Filtered of loadavg, `TIME_WAIT`, per-test `finished in`
timings and cargo's `Compiling` / `Checking` / `Finished` / `Blocking` chatter,
`gates4.log`, `gates3.log` and `gates2.log` are **byte-identical to one
another**, and `gates.log` differs from them by **one line**
(`Checking serde_test v1.0.177`, a cache-warmth artefact). Every figure below is
run 4's and every one of them is identical in all four runs, **including all
sixteen per-target cells**.

**Why run 1 is disclosed and not cited.** I launched it against commit 1's tree
and then, while it was running, made commit 2's edits in the same worktree.
Comment-only edits cannot move a pass/fail cell — and did not — but a run that
spans an edit is not provenanced to a tree. Runs 2 and 3 are each provenanced
and each was then superseded by a later commit. **Run 4 is the one that ran,
start to finish, against the tree that ships.**

**The `TIME_WAIT` close figures are the suite's own and are not port pressure.**
The socket tests run sixteen times over, each leaving sockets in `TIME_WAIT`;
against the 32768–60999 ephemeral range 1335 is 4% of it, an order of magnitude
below the 31 786 that broke C5's figures, and it drains — the next run opened at
**4**.

## Toolchain

`cargo 1.97.1 (c980f4866 2026-06-30)`, `rustc 1.97.1 (8bab26f4f 2026-07-14)`,
Ubuntu GLIBC 2.39-0ubuntu8.7. **MSRV rows on `cargo 1.92.0 (344c4567c
2025-10-21)`**, i.e. an explicit `cargo +1.92` ran — `rust-toolchain.toml` pins
`stable`, so a declared-and-ignored MSRV would have shown 1.97.1 in these rows.

## Job `gate`

| gate | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | **rc 0** |
| clippy, all features | `cargo clippy --locked --all-targets --all-features -- -D warnings` | **rc 0**, **0** lines matching `^(warning\|error)` |
| test, all features | `cargo test --locked --all-features` | **183 passed / 0 failed** over **16** targets |
| | per-target cells | **5 · 23 · 7 · 11 · 28 · 6 · 24 · 12 · 11 · 6 · 4 · 6 · 9 · 17 · 4 · 10** (last is doc-tests) — **identical cell for cell to C7, C6, C5 and C4** |
| test, no features | `cargo test --locked` | **99 passed / 0 failed** over 16 |
| build, `net` | `cargo build --locked --features net` | **rc 0** |
| doc, no features | `RUSTDOCFLAGS=-D warnings cargo doc --locked --no-deps` | **rc 0** |
| doc, all features | same, `--all-features` | **rc 0** |
| example | `cargo build --locked --features imagery,net --example imagery_dump` | **rc 0** |
| MSRV test, no features | `cargo +1.92 test --locked` | **99 passed / 0 failed** |
| MSRV test, all features | `cargo +1.92 test --locked --all-features` | **183 passed / 0 failed** |

## Job `features` — six sets, clippy and test each

| features | clippy | test |
|---|---|---|
| `imagery` | rc 0, **0** warning-or-error lines | **127 / 0** |
| `net` | rc 0, **0** | **121 / 0** |
| `serde` | rc 0, **0** | **101 / 0** |
| `nalgebra` | rc 0, **0** | **101 / 0** |
| `imagery,net` | rc 0, **0** | **179 / 0** |
| `serde,nalgebra` | rc 0, **0** | **103 / 0** |

## Job `feature-docs` — sixteen rows

`check` and `doc` (with `RUSTDOCFLAGS=-D warnings`), `--no-default-features`
plus the row's set, over `default`, `imagery`, `net`, `serde`, `nalgebra`,
`imagery-net`, `imagery-serde`, `imagery-nalgebra`, `net-serde`,
`net-nalgebra`, `serde-nalgebra`, `imagery-net-serde`, `imagery-net-nalgebra`,
`imagery-serde-nalgebra`, `net-serde-nalgebra`, `all`.

**16 of 16 rows `check_rc=0 doc_rc=0`** — counted with
`grep -c 'featuredocs .* check_rc=0 doc_rc=0'` = **16** against
`grep -c '^featuredocs'` = **16**, so the count is over the whole population and
not over the rows that happened to pass.

## Job `deny`

- **`cargo-deny 0.20.2`**, found on `PATH` at `/home/andrej/.cargo/bin/cargo-deny`.
  **C7's fallback path `/home/andrej/coord_scratch/riders67/deny/bin/cargo-deny`
  no longer exists**; I enumerated install shapes (`command -v`, `~/.cargo/bin`,
  `/usr/bin`, `/usr/local/bin`, a `find` over `coord_scratch`) before concluding
  anything, and it is the `~/.cargo/bin` shape.
  `check advisories bans licenses sources`, `--locked --all-features`: **rc 0**.
- Counted **untruncated**, over the whole `g_deny.txt` and not a `tail`:
  **warning-or-error lines = 2**, **`license-not-encountered` = 2** — the two
  pre-existing `deny.toml:35,36` entries. Same as C7 and as the C7 review's
  independent count.
- **`cargo-machete 0.9.2`** at
  `/home/andrej/coord_scratch/riders67/machete/bin/cargo-machete`: **rc 0**,
  *"cargo-machete didn't find any unused dependencies in this directory. Good
  job!"*, *"Done!"*.
- **Not run: `cargo-deny fetch db`.** The workflow refreshes the RustSec
  advisory database before checking; I ran against the database already on this
  host. An advisory published since that database was fetched would not be seen
  here. **Nothing in this candidate adds, removes or moves a dependency** —
  `Cargo.toml` and `Cargo.lock` are untouched — so the check's input is the same
  as C7's, whose run the C7 review reproduced.

## Clean extraction

`git archive 652039b` → `extract/`: **14** files in `tests/`, **0** matching
`zz_`, **no `docs/` directory in the archive at all**; `README.md`,
`src/imagery/net.rs` and `tests/imagery_net.rs` all `cmp`-identical to the
worktree's. Nothing under `docs/work/` is in the commit.

## BASE comparison

`git archive 652039b` with `src/imagery/net.rs` replaced by
`git show 2c3041f:src/imagery/net.rs` (md5 `0adf5f4d2c6312228dd1697114e28d7a`,
the same md5 C7 recorded), `--all-features --no-fail-fast`, `--test imagery_net`,
loadavg **11.97 → 12.05**:

**21 passed / 3 failed in 7.25 s** — against C7's **7.25 s** and the C6 review's
**7.24 s** — the three being exactly
`a_peer_still_sending_past_the_deadline_is_a_timeout`,
`a_record_too_long_for_the_receive_queue_is_not_completed_past_its_deadline` and
`the_worst_declared_length_past_the_deadline_stays_within_a_fixed_budget`. **Cell
for cell what C7, C6, C5 and the C6 review got.** Run three times in this
attempt — `869bb7c5` 7.26 s, `8a5f4c45` 7.25 s, shipped `e311a0f0` 7.25 s —
same cells every time.

## Ablations not re-run

A1 and A2 ablate the allocation path and the one-read bound. This candidate's
code is byte-identical to `70d0a02` and so to `b03817b` and `b482867` — the
comment-stripped `cmp` in report §0, at the same two md5s C7 published. **An
ablation of identical code cannot return a different answer**, and nothing in
this attempt is executable. Say so if you want them anyway.
