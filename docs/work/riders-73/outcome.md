# riders-73 outcome

The compile/config arm is **accepted-build**: C2
`c05b2b635d59e519752ae5c4a12478e97e6f9ce1` repaired the verifier, and CI run
`35537300634`, including i686 job `106148517661`, passed at integrated NatNet
`01302f32af0b6935f46fea4a690a4d35a6d5a407`.

The runtime arm is **accepted-measurement**: E1 evidence
`4edaf89636c1ce1370b9d2185064944f33aab852`, tree
`fb5967271501d4de0e262737875ba94c157ac333`, received independent acceptance
recorded in `18b11eda30a60d4a7d916be09164509d9ebbd1e8`. The measured source is
the exact integrated revision above, tree
`8bfa45cd18838adfc122a53b5371f907332ea5bd`; the procedure is
`187f89d99ee808f771908268973a381fa6f52bcd`.

The frozen target logs record 176 passing native i686 runtime tests, one
executable doctest and ten compile-fail doctests. The independent reviewer
verified the ELF32 runner records and target arithmetic controls and reran
the six-test imagery subprocess suite. Separate host controls passed. The
private Ubuntu userspace, fixed Rust 1.98.1 toolchain, official package
provenance, example-path override and preserved initial setup failures are
part of the measurement identity, not interchangeable implementation inputs.

Raw artifacts remain in `/home/andrej/coord_preserve/riders73-E1.ZUSSmh`.
The coordinator rechecked the frozen and supplemental manifests after review.
The review's appended generic runner log is not part of the accepted raw
identity. No system package installation, emulator, live bridge, hardware or
benchmark result is claimed. This evidence does not add a runtime CI job or
broaden the supported-platform contract.

The coordinator accepts the packet for integration. The packet-only change
preserves product source, tests, manifests, lockfile and workflow. Before
advancing the main NatNet checkout, reconcile motive-bridge's exact-HEAD
pairing against the final packet-integration identity; do not mistake the
old pairing pin for evidence about a changed checkout. Auga's board and
integration plan record the remaining coordination separately. Earlier
compile-only or unavailable-runtime observations remain frozen historical
records, not current runtime acceptance.
