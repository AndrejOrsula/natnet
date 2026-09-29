# riders-73 C2 — CI verifier repair

The C1 configuration reached `dev` at `09e89aab15b63345403c82a838685bdcb3fe97e1`.
Its CI run `35536927717` failed the new `i686 compile` job before the
`cargo check` step. The stable action installed the i686 target successfully,
but the negative wiring control printed `target mismatch was accepted`.

The cause is Bash's `errexit` exception for a function called as an `if`
condition. `check_target_wiring` used bare `test` and `grep` commands. A
failed deny comparison did not end the function in that context; a later
successful grep made the whole function return success. The C1 local positive
and mismatch check did not execute the workflow step with the same `if`
context. The independent C1 review also missed that control-flow defect.

C2 adds explicit `|| return 1` to each assertion in the verifier. It changes
only `.github/workflows/rust.yml`; the candidate target, install, check,
deny graph, runtime boundary, and ordinary jobs are unchanged.

Candidate C2 is `c05b2b635d59e519752ae5c4a12478e97e6f9ce1`, tree
`2a3e1edff87104d3079c5612df34fd3099dde613`, parent
`46869eefb0609ac10af63f8488d838da7019bd12`.

The exact checked-in `run:` block was parsed with PyYAML and executed under
`bash -e`, substituting only a one-line installed-target response because this
workstation lacks i686. With `TARGET=i686-unknown-linux-gnu` and a temporary
`RUNNER_TEMP`, the positive check and mismatched-deny negative control both
passed; the block exited 0. This is local control-flow proof, not a cross-build.
The exact integrated-SHA CI job must now reach and pass `cargo check` before
the compile/config arm can be marked `accepted-build`. 32-bit runtime tests
remain `blocked_external`.
