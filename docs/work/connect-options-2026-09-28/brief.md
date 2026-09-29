# connect-options-2026-09-28 — NAT_CONNECT connection options

Repository: `natnet`. Base `dev` `50d8da207bdf73dcadb05d3ba262d05f4d46b57d`.
Worktree `/home/andrej/ws_rs/natnet`, uncommitted. Access: loopback only; no
Motive, hardware, benchmark or Windows run. The owner asked on 2026-09-28 for
the review follow-ups, one of which is the versioned connect deferred from
`review-fixes-2026-09-28`.

## Finding

`command::connect_request()` sends NAT_CONNECT with an empty payload. The SDK
4.1.0.0 clients send a sender block followed by `sConnectionOptions`:

- `Samples/PacketClient/PacketClient.cpp` lines 1073-1096: a zeroed 264-byte
  `sSender` (256-byte name, application version, NatNet version), then
  `subscribedDataOnly` (one byte) and `BitstreamVersion[4]`, with a comment
  that an unset (zero) version makes Motive send "the most current version".
  It declares `sizeof(sSender) + sizeof(sConnectionOptions) + 4` payload
  bytes.
- `Samples/PythonClient/NatNetClient.py` lines 1714-1736: 270 bytes, "Ping"
  in the name, byte 264 = 0, bytes 265-268 = 4.1.0.0.
- `PacketClient.cpp` lines 398-415: on SERVERINFO the client keeps its
  requested version when it asked for one, and otherwise adopts the server's
  NatNet version; bitstream changes need a NatNet 3+ server and unicast data.

Consequence: this crate cannot ask Motive for a bitstream it can decode. A
server whose current stream is newer than 4.1 fails the handshake's version
check (inference; no such server has been tested).

## Decision (draft amendment to Auga D54)

**CO1 — Connection options are additive; the default hello is unchanged.**
`command::ConnectOptions` (`subscribed_data_only`, `bitstream_version:
Option<Version>`) and `command::connect_request_with(options)` build the SDK
layout: 264 zero sender bytes, the option byte at 264, the version at
265..=268 (0.0.0.0 when `None`), and one zero byte, 270 bytes in all, as the
Python sample sends. `connect_request()` keeps its empty payload, so no
existing caller's wire bytes change. `ConnectOptions::parse` reads the same
layout for servers and returns `None` for a payload shorter than 269 bytes.
`ConnectOptions::stream_version(&ServerInfo)` applies the sample's rule: the
requested version when one was requested and the server's NatNet major is at
least 3, otherwise the server's. `CommandClient::connect_handshake_with`
validates that version instead of the server's. The sample restricts bitstream
changes to unicast data; with multicast data the caller should not request a
version. Whether Motive honours the request, and what its SERVERINFO then
reports, is unverified against live Motive and must be tested there before a
consumer enables it by default.

## Gates

fmt; clippy `-D warnings` default and all features; tests default and all
features; strict docs all features; MSRV 1.92 test; `git diff --check`.
