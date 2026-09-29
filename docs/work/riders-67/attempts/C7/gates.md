# riders-67 C7 — gate evidence

`candidate(C7)` = `70d0a025419f3e0ea826c6a47509517e1ec501e4`, tree
`c0be2b7583d69fc1f379d2bdf2514b706ae201b6`. Worktree
`/home/andrej/ws_rs/natnet_wt/impl-riders-67-C7`, branch `slice/riders-67-C7`.

Script `/home/andrej/coord_scratch/riders67c7/gates.sh` — C6's, path-substituted,
with two changes: `cargo-deny`'s output is written whole to `g_deny.txt` and
counted **untruncated** (C6 §9 disclosed that `tail -12` could not count it),
and `/proc/loadavg` plus the `TIME_WAIT` count are taken at open and close.

`CARGO_TARGET_DIR=/home/andrej/coord_scratch/riders67c7/target`.

Three full runs, all in the scratch directory, all identical on every figure:
`gates.log` (tree `59bce514`, the first commit), `gates2.log` (an intermediate
tree), **`gates4.log` (the shipped tree `c0be2b7`)**. Everything below is
`gates4.log`. `gates3.log` was started against a tree amended before it
finished; it agrees, and is superseded.

| shipped-tree run | loadavg | TIME_WAIT |
|---|---|---|
| open | 11.37 11.50 11.31 | 4 |
| close | 11.45 11.50 11.32 | 1338 |

The `TIME_WAIT` growth is the gate suite's own socket tests, run sixteen times
over across profiles and feature sets; the next run opened at 4 again. No
figure in this attempt is a timing measurement.

## Totals, summed over every `test result` line in each file

| gate | passed | failed | result lines |
|---|---|---|---|
| `test --locked --all-features` | **183** | **0** | 16 |
| `test --locked` (no features) | **99** | **0** | 16 |
| MSRV 1.92 `test --locked` | **99** | **0** | 16 |
| MSRV 1.92 `test --locked --all-features` | **183** | **0** | 16 |
| `--features imagery` | **127** | **0** | 16 |
| `--features net` | **121** | **0** | 16 |
| `--features serde` | **101** | **0** | 16 |
| `--features nalgebra` | **101** | **0** | 16 |
| `--features imagery,net` | **179** | **0** | 16 |
| `--features serde,nalgebra` | **103** | **0** | 16 |

Per-target cells, all-features, in file order:
**5 · 23 · 7 · 11 · 28 · 6 · 24 · 12 · 11 · 6 · 4 · 6 · 9 · 17 · 4 · 10**
(the last is doc-tests). Identical to C6, C5 and C4.

Clippy warning-or-error line counts, all seven invocations (all-features and
the six feature combinations): **0 0 0 0 0 0 0**, rc 0 each.

## `cargo-deny`, counted untruncated

    deny untruncated: warning-or-error lines = 2
    deny untruncated: license-not-encountered = 2

Both are the pre-existing `deny.toml:35,36` entries. rc 0. Full output in
`g_deny.txt`.

## BASE comparison, on the shipped tree

`git archive 70d0a02` into `basecmp/`, then `src/imagery/net.rs` replaced by
`git show 2c3041f:src/imagery/net.rs` (md5 `0adf5f4d2c6312228dd1697114e28d7a`),
`cargo test --locked --all-features --no-fail-fast --test imagery_net`,
loadavg 11.45 → 11.50:

    test result: FAILED. 21 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.25s

        a_peer_still_sending_past_the_deadline_is_a_timeout
        a_record_too_long_for_the_receive_queue_is_not_completed_past_its_deadline
        the_worst_declared_length_past_the_deadline_stays_within_a_fixed_budget

## Clean extraction, on the shipped tree

    tests files: 14
    zz_ matches: 0
    docs dir present: NONE
    cmp identical: README.md
    cmp identical: src/imagery/net.rs
    cmp identical: tests/imagery_net.rs

## "No code changed", both instruments

    src/imagery/net.rs   comment-stripped: 272 lines  md5 d1d7269b55c3eb81edfcbc1d0abe5f97  -> IDENTICAL to b03817b
    tests/imagery_net.rs comment-stripped: 1113 lines md5 47f2a54feb5a41fe916a5bdee9ec11c1  -> IDENTICAL to b03817b
    git diff b03817b -U0 -- src tests, lines that are not comment lines: 0

    reach check 1: perturb one identifier in the stripped candidate -> DIFFER
    reach check 2: filter fed one code line and two comment lines   -> 1

## Raw log, every gate header and its result

    =====[loadavg open]=====
    11.37 11.50 11.31 13/1646 806571
    4
    =====[toolchain]=====
    cargo 1.97.1 (c980f4866 2026-06-30)
    =====[fmt]=====
    rc=0
    =====[clippy all-features]=====
    rc=0
    =====[clippy warncount]=====
    0
    =====[test all-features]=====
    =====[test no-features]=====
    =====[build net]=====
    rc=0
    =====[build example]=====
    rc=0
    =====[doc no-features]=====
    rc=0
    =====[doc all-features]=====
    rc=0
    =====[MSRV toolchain]=====
    cargo 1.92.0 (344c4567c 2025-10-21)
    =====[MSRV test no-features]=====
    =====[MSRV test all-features]=====
    =====[features imagery clippy]=====
    0
    rc=0
    =====[features imagery test]=====
    =====[features net clippy]=====
    0
    rc=0
    =====[features net test]=====
    =====[features serde clippy]=====
    0
    rc=0
    =====[features serde test]=====
    =====[features nalgebra clippy]=====
    0
    rc=0
    =====[features nalgebra test]=====
    =====[features imagery,net clippy]=====
    0
    rc=0
    =====[features imagery,net test]=====
    =====[features serde,nalgebra clippy]=====
    0
    rc=0
    =====[features serde,nalgebra test]=====
    =====[cargo-deny]=====
    deny binary: /home/andrej/.cargo/bin/cargo-deny
    rc=0
    =====[cargo-machete]=====
    0.9.2
    =====[loadavg close]=====
    11.45 11.50 11.32 12/1646 815567
    1338
    =====[done]=====
    featuredocs default check_rc=0 doc_rc=0
    featuredocs imagery check_rc=0 doc_rc=0
    featuredocs net check_rc=0 doc_rc=0
    featuredocs serde check_rc=0 doc_rc=0
    featuredocs nalgebra check_rc=0 doc_rc=0
    featuredocs imagery-net check_rc=0 doc_rc=0
    featuredocs imagery-serde check_rc=0 doc_rc=0
    featuredocs imagery-nalgebra check_rc=0 doc_rc=0
    featuredocs net-serde check_rc=0 doc_rc=0
    featuredocs net-nalgebra check_rc=0 doc_rc=0
    featuredocs serde-nalgebra check_rc=0 doc_rc=0
    featuredocs imagery-net-serde check_rc=0 doc_rc=0
    featuredocs imagery-net-nalgebra check_rc=0 doc_rc=0
    featuredocs imagery-serde-nalgebra check_rc=0 doc_rc=0
    featuredocs net-serde-nalgebra check_rc=0 doc_rc=0
    featuredocs all check_rc=0 doc_rc=0
