# review-fixes-2026-09-28 C1 — integrator review

Reviewed 2026-09-28 against the uncommitted worktree on `88837ef`.

Verdict: **accept for the owner's decision**, with no must-fix found.

- RF1: the drain is deadline-bounded, tolerates `Interrupted` and Windows
  `ConnectionReset`, and restores blocking mode on every exit; the drain
  error takes precedence over a restore error. A reply racing between drain
  and fresh reply stays indistinguishable, as D54 already states.
- RF2: waits never rely on `SO_RCVTIMEO`. The price is polling at most once
  per millisecond while idle and up to 1 ms of added latency per record.
- RF3: the only consumer affected was natnet_zenoh's synthetic scene size
  check, which it now maps back to its own datagram-limit message; its
  native raw output already drops a frame that fails to re-encode.
- RF6: confirmed against `sdk/NatNetSDK/include/NatNetTypes.h` (b2 Live/Edit
  mode, b3 bitstream version changed) and `PacketClient.cpp` lines 3007–3008;
  only line 2874 treats `0x04` as bitstream changed. The owner should
  confirm.
- Open: apply RF1–RF6 to Auga D54; natnet_zenoh and motive-bridge must move
  their pins once this is committed.

Evidence is Linux loopback only.
