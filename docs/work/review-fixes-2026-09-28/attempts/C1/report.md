# review-fixes-2026-09-28 C1 — builder report

Decisions RF1–RF6 are in [`../../brief.md`](../../brief.md), recorded before
the source changes (RF6 after the accessor was first drafted). They are a
draft amendment to Auga `docs/07-decisions.md` D54 that still has to be
applied there.

## Changes

- **A1** `src/net.rs`: `NAT_UNRECOGNIZED_REQUEST`; `exchange_with_deadline`
  drains queued datagrams before each send (`drain_queued`, deadline-bounded,
  blocking mode restored); a read timeout is always `TimedOut`; an id-100
  reply from the configured peer returns `Unsupported` at once (RF1).
- **A2** `src/imagery/net.rs`: `from_stream` sets the socket non-blocking and
  no receive timeout is used; `recv_record` and `read_body` poll against the
  deadline (50 µs doubling to 1 ms). An idle deadline is a non-terminal
  `TimedOut` with the buffer unchanged; a deadline inside a record stays
  terminal. Buffers grow in chunks as bytes arrive. New `is_terminal` (RF2).
- **A3** `message::frame_checked`, used by the checked encoders in
  `frame.rs`, `modeldef.rs`, `serverinfo.rs` and `decoder.rs` (RF3);
  `InvalidCount` for negative counts and sizes under both policies (RF4);
  `params` docs for bits 0–3 and `FrameOfData::bitstream_changed()` reading
  `0x08` (RF6); re-encode failure of long lossily decoded names documented.
- **A4** `src/net.rs`: `MIN_DATA_BUFFER`, a socket-options section,
  `DataClient::from_socket`, and `recv` returning `InvalidData` when a
  datagram fills a buffer shorter than 65 507 bytes (RF5).
- **A5** `bytes::vec_for_count` caps preallocation from untrusted counts at
  256; `checked_typed_section` writes in place and patches the size;
  `checked_raw_section` validates once; `tests/imagery_cli.rs` builds its
  example through `$CARGO` and reads the binary path from Cargo's JSON output.

## Public API

Added: `net::NAT_UNRECOGNIZED_REQUEST`, `net::MIN_DATA_BUFFER`,
`DataClient::from_socket`, `ImageClient::is_terminal`,
`FrameOfData::bitstream_changed`.

Behaviour changes: command `WouldBlock` becomes `TimedOut`; id 100 becomes
`Unsupported`; `DataClient::recv` into a filled buffer under 65 507 bytes
becomes `Err`; an imagery idle timeout is no longer terminal; checked
`to_datagram*` returns `LengthOverflow` for payloads over 65 535 bytes; a
negative frame count or size under `Compatible` is `InvalidCount` instead of
`ImplausibleCount`. AUGI/1 encoders and NAT_CONNECT are unchanged.

## Not verified

Windows socket behaviour of the drain and of non-blocking imagery polling;
live Motive; hardware; benchmarks.
