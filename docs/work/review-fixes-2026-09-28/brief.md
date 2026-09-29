# review-fixes-2026-09-28 — command freshness, imagery idle waits, checked framing

Repository: `natnet`. Base `dev` `88837ef24765743d122cf496cc1d7610f7883076`
(a docs-only descendant of the reviewed `01302f32af0b6935f46fea4a690a4d35a6d5a407`;
`git diff --stat 01302f3 88837ef` touches only `docs/work/riders-73/`).
Worktree `/home/andrej/ws_rs/natnet`, uncommitted; nothing is staged, committed
or pushed. Access: loopback only. No Motive, hardware, benchmark or Windows run.

The owner authorised the review fixes A1-A5 below on 2026-09-28. Consumers:
`motive-bridge` (path dependency; `imagery` encoders, `net` `DataClient`) and
`natnet_zenoh` (frame/MODELDEF/SERVERINFO decode, checked `to_datagram*`
re-encode, synthetic server). The AUGI/1 encoders and wire format are out of
scope and unchanged. NAT_CONNECT payload changes are out of scope.

## Contract decisions (draft amendment to Auga D54)

The canonical decisions record for this crate is Auga
`docs/07-decisions.md` D54 (with D46/D47). This work is not permitted to edit
another repository, so the entries below are recorded here first, before any
source change, as the text the coordinator applies to D54 as an amendment
dated 2026-09-28. Until then this packet is the decision record for these
changes.

**RF1 — Command replies: drain before send, fail fast on refusal, one timeout
kind.** Before each typed command transmission the safe client discards every
datagram already queued on its socket (non-blocking reads until
`WouldBlock`, bounded by the transaction deadline, blocking mode restored).
A reply that arrived after an earlier transaction's deadline therefore cannot
answer the next request. A reply that arrives after the drain but before the
fresh reply remains indistinguishable (no NatNet transaction id); D54's
"UDP peer filtering cannot prove same-peer reply freshness" still holds.
A `NAT_UNRECOGNIZED_REQUEST` (id 100) datagram from the configured peer ends
the transaction immediately with `io::ErrorKind::Unsupported`. A read-timeout
expiry is always reported as `io::ErrorKind::TimedOut` (Linux reports
`WouldBlock` for `SO_RCVTIMEO`; that kind is no longer surfaced).

**RF2 — Imagery idle waits are non-fatal; partial-record timeouts stay
fatal.** `ImageClient` performs every wait with non-blocking reads polled
against the absolute record deadline (sleep at most 1 ms between polls); it
never relies on `SO_RCVTIMEO`, because Microsoft documents that a blocking
`recv` timed out by `SO_RCVTIMEO` leaves the connection "in an indeterminate
state" that "should be closed". `from_stream` sets the adopted socket's mode
explicitly, so an inherited non-blocking socket works. When a record deadline
passes before any byte of the next record has been read, `recv_record`
returns `TimedOut` without entering the terminal state and leaves the caller's
buffer unchanged; the next call may wait again. A deadline that passes after
any byte of a record was consumed remains terminal (connection shut down).
An additive `ImageClient::is_terminal()` distinguishes the two. Past the
deadline, the record is still finished only from bytes already queued, with a
single read, now of at most the larger of 1 MiB and the space already
allocated. Receive buffers grow in bounded chunks with bytes actually
received (never zero-filling the declared length up front); `MAX_RECORD_LEN`
is unchanged.

**RF3 — Checked NatNet framing refuses payloads the header cannot declare.**
Checked `to_datagram*` encoders (`FrameOfData`, `ModelDef`, `ServerInfo`,
`DecodedMessage`) return `Error::LengthOverflow { field: "message payload
size", length, max: 65535 }` for a payload longer than 65 535 bytes instead of
stamping a saturated header. Only `*_lossy` encoders keep saturation. This
changes a previously successful (`Ok`) result into `Err` for such payloads;
datagrams received over IPv4 UDP (at most 65 507 bytes) re-encode unchanged
unless lossy UTF-8 replacement grows them.

**RF4 — Negative frame counts are `InvalidCount` under both policies.** A
negative typed count, or a negative 4.1 section byte size, in a FrameOfData
payload returns `Error::InvalidCount { count, offset }` under both
`ParsePolicy::Strict` and `ParsePolicy::Compatible`, matching MODELDEF. It was
previously `ImplausibleCount` with a sign-extended `usize` count under
`Compatible`. A negative count on an opaque 4.1 raw section remains retained
under `Compatible`, as before.

**RF5 — Data-socket configuration belongs to the caller.** `DataClient`
keeps its zero-dependency `bind_*` constructors, which cannot set
`SO_REUSEADDR` or `SO_RCVBUF` through `std`. An additive
`DataClient::from_socket(UdpSocket)` adopts a caller-configured socket.
`DataClient::recv` returns `io::ErrorKind::InvalidData` when a datagram fills
a caller buffer shorter than the exported `MIN_DATA_BUFFER` (65 507 bytes),
because such a datagram may have been truncated; buffers of at least that size
are unaffected (`Ok` becomes `Err` only in that ambiguous case).

**RF6 — `bitstream_changed()` reads bit 3 (`0x08`), per the SDK header.**
The review asked for frame params bit `0x04` to be documented and a
`FrameOfData::bitstream_changed()` accessor added. The NatNet SDK 4.1.0.0
sources disagree about that bit: `include/NatNetTypes.h`
(`sFrameOfMocapData::params`) documents b0 recording, b1 model list changed,
b2 Live/Edit mode and b3 bitstream version changed; `PacketClient.cpp`'s
receive loop (lines 3007-3008) tests `0x04` as Live/Edit and `0x08` as
bitstream changed, matching the header; only its frame unpacker (line 2874)
tests `0x04` as bitstream changed, next to a `0x03` Live-mode mask that cannot
be a single bit. The accessor follows the header (two of three sources, and
the only normative one). The `params` docs record all four bits, the sample's
disagreement, and that no bit has been observed set (144 356 frames, Auga
P2.3b). The raw field stays public. This entry was written after the
accessor was first drafted against `0x04` and before it was finalised; the
owner should confirm or reverse it.

## Tasks

- A1 `src/net.rs` command exchange: RF1, named constant, tests tightened to
  `TimedOut`; loopback regression where a late reply to A must not answer B.
- A2 `src/imagery/net.rs`: RF2; tests for an inherited non-blocking socket,
  an idle poll followed by a delivered heartbeat, and a bare 12-byte prefix
  that must not grow the buffer to 64 MiB.
- A3 `src/message.rs`, `frame.rs`, `modeldef.rs`, `serverinfo.rs`,
  `decoder.rs`: RF3, RF4; document frame params bit `0x04` with an additive
  `FrameOfData::bitstream_changed()` (RF6); document re-encode failure of a
  lossily decoded name longer than 255 bytes.
- A4 `src/net.rs` data client: RF5.
- A5 self-sufficient `tests/imagery_cli.rs`; bounded preallocation from
  untrusted counts; no temporary body `Vec` or duplicate raw validation on the
  checked frame encoder.

## Gates

fmt; clippy `-D warnings` default and all features and the CI feature rows;
tests default, all features and the CI feature rows; strict docs default and
all features and the 16 CI feature-doc rows; `net` build; example build;
MSRV 1.92 build/test; i686 check if the target is installed; cargo-deny and
cargo-machete if installed. Every command, exit code and result goes to
`attempts/C1/gates.md`; the change summary to `attempts/C1/report.md`.
