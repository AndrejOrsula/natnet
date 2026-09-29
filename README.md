# natnet

`natnet` is a sans-io Rust implementation of the OptiTrack **NatNet** protocol
spoken by Motive. It parses and serializes SERVERINFO, MODELDEF and
FrameOfData messages, builds the command-channel requests, and keeps transport
and application policy at the edge of the crate.

- **Sans-io core.** Bytes in, typed values out, and back. The default feature
  set has no runtime dependencies and the crate forbids unsafe code.
- **Version-aware.** Layouts are selected from the negotiated protocol version;
  an unknown or unverified layout is refused rather than guessed.
- **Checked encoding.** Encoders reject values the wire cannot carry
  faithfully; explicitly named `*_lossy` encoders keep the old unchecked
  behavior.
- **Optional std clients** (`net`): a receive-only data client for unicast or
  multicast, a command client with validated requests and bounded
  transactions, and an explicitly unrestricted raw client.
- **Convention-neutral.** Positions and quaternions are plain arrays in the
  server's frame. Coordinate transforms, clock estimation, buffering and the
  handling of tracking-invalid bodies are the consumer's.

## Example

```rust
use natnet::{Decoder, Message, PlannedBody, SyntheticServer, Version};

fn main() -> natnet::Result<()> {
    // A synthetic NatNet 4.1 server produces a real FrameOfData datagram.
    let server = SyntheticServer::new(Version::V4_1, 1_000_000);
    let body = PlannedBody::tracked(7, [1.0, 2.0, 3.0], [0.0, 0.0, 0.0, 1.0]);
    let wire = server.datagram(42, 0.5, &[body])?;

    let decoded = Decoder::new(Version::V4_1)?.parse(&wire)?;
    if let Message::FrameOfData(frame) = decoded.message() {
        assert_eq!(frame.frame_number, 42);
        assert!(frame.rigid_bodies[0].tracking_valid());
    }
    assert_eq!(decoded.to_datagram()?, wire);
    Ok(())
}
```

Against a live server, the `probe` example performs the NAT_CONNECT
handshake, prints SERVERINFO and the model definitions, and decodes a few
frames from multicast, or from unicast while sending keepalives:

```sh
cargo run --features net --example probe -- --server 192.168.1.10 --interface 192.168.1.20
```

## Protocol layouts

The negotiated protocol version comes from SERVERINFO. Version-aware decoding
and encoding use an explicit layout:

| Layout | Status | Wire distinction |
| --- | --- | --- |
| NatNet 3.0 | Supported | Legacy count-only sections; no assets or precision timestamp |
| NatNet 3.1 | Supported | Legacy count-only sections; no assets or precision timestamp |
| NatNet 4.0 | Supported | Legacy count-only sections; no assets or precision timestamp |
| NatNet 4.1 | Supported | Count-and-size sections, assets, and precision timestamp |
| Other 3.x minors | Pending independent protocol evidence | No compatibility claim until the field layout is verified |

This matrix describes parser layout support. It does not certify every Motive
producer or firmware combination. Unsupported versions remain available as
raw SERVERINFO data for diagnostics, but are not accepted as a known frame
layout. Build and revision bytes remain available as version provenance; for
the listed layouts, layout selection uses the vendor-verified major/minor pair
and ignores build and revision. Vendor documentation lists newer 4.2 and 4.3
versions, but their layouts are unaudited here and are explicitly unsupported;
the crate does not accept every version sharing a supported major number. See
the [vendor version table](https://docs.optitrack.com/v3.3/developer-tools/natnet-sdk/natnet-4.0.md#natnet-versioning)
for the advertised version history.

Every FrameOfData section and every MODELDEF dataset type the NatNet SDK 4.1
decodes has a typed representation. Skeleton, asset, force-plate and device
frame sections are kept as their wire bytes, so a round trip never loses them,
and decode on request (`FrameOfData::decode_skeletons` and its siblings);
`RawSection::from_skeletons` and its siblings encode typed values back.
MODELDEF skeletons, force plates, devices, cameras and assets parse into
`DataSet` variants. These layouts follow the NatNet SDK 4.1.0.0 samples and
are not yet confirmed against bytes from a real Motive server; the marker-set,
rigid-body and timing layouts are.

## Validation and encoding

Strict decoding rejects invalid counts, terminators, and unexplained residue.
Compatibility parsing remains available for historical extension tolerance, and
the outer message length is treated as telemetry under both policies. A
negative typed count or section size is `InvalidCount` under both policies.
`Decoder` defaults to strict parsing and keeps the source layout, so a decoded
message re-encodes in the layout it arrived in.

Frame decoding preserves rigid-body entries whose tracking-valid bit is clear.
Consumers decide whether such entries represent a gap for their application;
the crate does not remove them.

Checked encoders reject values that cannot be represented faithfully, such as
wire-length overflow, interior NULs in terminated strings, or content that the
selected layout cannot carry. Floating-point bits, NaNs included, survive a
round trip; string decoding is lossy UTF-8, so arbitrary non-UTF-8 name bytes
do not.

## Networking (`net`)

The data client receives NatNet data and has no datagram-send surface. The
command client owns command-channel transmission: NAT_CONNECT negotiation,
model-definition, frame and property requests. Typed requests are validated
before they are framed; the client filters the configured peer and bounds
every transaction by a deadline. Replies already queued are discarded before
each request, so a reply that missed an earlier deadline cannot answer the next
one, and a server's NAT_UNRECOGNIZED_REQUEST ends the transaction at once.
NatNet has no transaction identifier, so a late reply from the same peer that
arrives after the send is still indistinguishable. Peer filtering is transport
hygiene, not authentication.

The default NAT_CONNECT hello carries no options, so Motive streams its current
bitstream; `command::ConnectOptions` with `connect_request_with` (and
`CommandClient::connect_handshake_with`) request a specific one in the SDK
sample layout. Motive's handling of that request is not yet verified live.
`command::keepalive_datagram` and `command::disconnect_datagram` are the
fire-and-forget messages a unicast client sends on its command socket.

Property writes are split into tiers. Tier-1 writes run as a consuming state
machine that writes a canary first and re-reads every value before the next
target is unlocked; Tier-2 writes are refused without explicit authorization.
An explicitly separate raw command client permits unrestricted bytes for
consumers that accept that responsibility.

## Optional features

| Feature | Adds |
| --- | --- |
| `net` | Standard-library UDP data and command clients with explicit local-interface binding. |
| `serde` | `Serialize`/`Deserialize` derives on the protocol value types that document them. |
| `nalgebra` | One-way conversions from the plain-array poses and markers into nalgebra types, with no normalization or axis transform. |
| `imagery` | The AUGI/1 image side channel of `motive-bridge`; with `net`, a TCP client for it. |

NatNet carries no imagery. The `imagery` feature decodes the codec-agnostic
AUGI/1 stream a `motive-bridge` host emits: hello, camera descriptions, image
frames, heartbeat and bye records. Image bytes are returned with an encoding
tag; no image codec is decoded and no dependency is added. With `net`, the
image client connects to the bridge or accepts a bridge-initiated reverse
connection. A record timeout that expires before any byte of the next record
arrives is an idle poll, not a failure; a deadline inside a record is
terminal. Reverse mode trusts the first accepted peer, so listener exposure
and session validation remain the consumer's.

```sh
cargo run --features imagery,net --example imagery_dump -- --connect 127.0.0.1:1516
```

## Minimum supported Rust version

Rust 1.92. CI tests the MSRV, the moving stable toolchain, and every
combination of the optional features for compilation and documentation.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the checks a change must pass and the
rules for fixtures and published figures.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option. Unless you explicitly state otherwise, any contribution
intentionally submitted for inclusion in this crate by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any additional
terms or conditions.
