# connect-options-2026-09-28 C1 — self-review

Same-context review of the complete diff; not an independent review.

- Wire bytes of every existing request are unchanged; the new request is
  reachable only through the new functions.
- Offsets follow the SDK samples cited in the brief (sender block 264 bytes,
  option byte at 264, version at 265..=268). The payload length follows the
  Python sample (270); the C++ sample declares 273 with four unset trailing
  bytes, which a server reading fixed offsets ignores (inference).
- `stream_version` encodes the `PacketClient` rule but cannot see whether the
  data path is multicast; the docs put that on the caller.
- Open: live Motive behaviour; an independent review before a consumer
  enables the request.
