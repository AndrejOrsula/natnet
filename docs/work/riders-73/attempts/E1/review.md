# riders-73 E1 independent evidence review

Fresh-context reviewer: `r73_i686_review`. Coordinator transcription.
**Disposition: accept, accepted-measurement.** No material blocker or E2
requirement was found.

Evidence identity `4edaf89636c1ce1370b9d2185064944f33aab852`, tree
`fb5967271501d4de0e262737875ba94c157ac333`; procedure
`187f89d99ee808f771908268973a381fa6f52bcd`; source
`01302f32af0b6935f46fea4a690a4d35a6d5a407`, tree
`8bfa45cd18838adfc122a53b5371f907332ea5bd`. Packet and detached source were
clean at review. This acceptance applies to that source/environment tuple.

## Independent verification and six lenses

1. **Correctness:** the reviewer recomputed 16 target runner invocations,
   all ELF32/i386 with the private interpreter, and 176 passing runtime tests.
   A direct private-loader rerun of the six-test `imagery_cli` subprocess
   suite passed. Pointer-width and target arithmetic controls agree.
2. **Honesty:** one executable doctest is runtime evidence; ten compile-fail
   doctests are compile-only. The independently checked doctest runner log
   contains one ELF32 invocation. Host, target, initial failed setup and final
   passing results remain distinct.
3. **Determinism:** fixed source/toolchain/target, package versions, wrapper
   bytes and retained hashes bind the measurement; no cross-machine or
   broader determinism claim follows.
4. **Runtime behavior:** the target example is itself ELF32 and linked to
   the private sysroot. `tests/imagery_cli.rs:23-46` explicitly supports the
   exact-example environment override; independently rerunning its subprocess
   suite showed that the wrapper does not replace product behavior.
5. **Test quality:** target payload, network length and ray-grid controls
   independently passed at the relevant 32-bit branches. Target/host counts
   are not interchangeable; loopback and no-socket smoke scopes are explicit.
6. **Scope/seams:** no product source change, system package installation,
   emulator, hardware, live bridge or benchmark. Exact official Snapshot
   archives, the necessary generic development dependency and the later
   authorized formatter are disclosed tooling corrections, not silent
   substitutions. Their preserved failed attempts and final hashes make the
   correction reviewable; no re-collection is required.

Raw reviewed evidence remains under
`/home/andrej/coord_preserve/riders73-E1.ZUSSmh`; the source and all frozen
manifests and named reviewed logs remain unchanged. The reviewer disclosed
that one direct wrapper check appended the generic untracked
`logs/runner-invocations.log`. That file is excluded from this evidence
identity; it must not be mistaken for any named final runner log in the
packet. The coordinator rechecked the frozen raw and supplemental manifests
after review and their verification returned 0.

The host differential records 176 runtime tests, one executable doctest and
ten compile-fail doctests. This corroborates current-source behavior but is
not substituted for target execution. No physical or live-system acceptance
is implied by this source/runtime measurement.
