# Changelog

All notable changes to this crate are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the crate
follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

First public release. Everything below is new.

### Protocol core

- SERVERINFO, MODELDEF and FrameOfData decoding and checked encoding for the
  NatNet 3.0, 3.1, 4.0 and 4.1 layouts; other versions are refused, not
  guessed.
- `Decoder` with strict parsing by default and `ParsePolicy::Compatible` for
  extension-tolerant input; decoded messages re-encode in their source layout.
- `*_lossy` encoders for intentionally non-canonical output.
- Typed skeleton, asset, force-plate and device frame data
  (`FrameOfData::decode_*`, `RawSection::from_*`) and typed MODELDEF
  skeleton, force-plate, device, camera and asset descriptions (`DataSet`
  variants). Compatible parsing keeps a description it cannot represent
  exactly as `DataSet::Other`. These layouts are spec-derived from the NatNet
  SDK 4.1.0.0 samples.
- `frame::decode_id` and `frame::encode_id` for NatNet composite ids.
- All NatNet SDK 4.1 message ids, including NAT_DISCONNECT, NAT_KEEPALIVE,
  the echo and discovery ids, and NAT_UNRECOGNIZED_REQUEST at the crate root.
- `parse_message_string` for NAT_MESSAGESTRING text.
- `SyntheticServer`, `PlannedBody` and `PlannedSections` for generating
  spec-derived datagrams.

### Commands

- Validated command requests: NAT_CONNECT with optional `ConnectOptions`
  (subscribed-data-only and a requested bitstream version), model-definition
  and single-frame requests, and `GetProperty`.
- Tier-1 `SetProperty` state machine with a canary and read-back; Tier-2
  writes require explicit authorization.
- `keepalive_datagram` and `disconnect_datagram` for unicast clients.

### Optional features

- `net`: receive-only `DataClient` (unicast, multicast, or an adopted socket),
  `CommandClient` with peer filtering and bounded transactions, and
  `RawCommandClient`.
- `imagery`: AUGI/1 image side-channel decoding and encoding; with `net`, a
  TCP `ImageClient`.
- `serde` derives and `nalgebra` conversions.

### Examples

- `probe`: handshake, model definitions and frames from a live server.
- `imagery_dump`: prints AUGI/1 records from a `motive-bridge`.
