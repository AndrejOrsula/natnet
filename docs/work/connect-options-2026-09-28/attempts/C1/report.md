# connect-options-2026-09-28 C1 — report

Decision CO1 is in [`../../brief.md`](../../brief.md), written before the
source change.

## Change

- `src/command.rs`: `CONNECT_SENDER_LEN` (264), `CONNECT_OPTIONS_PAYLOAD_LEN`
  (270), `ConnectOptions { subscribed_data_only, bitstream_version }` with
  `requesting`, `parse` and `stream_version`, and `connect_request_with`.
  `connect_request()` is unchanged (empty payload) and its docs now say so.
- `src/net.rs`: `CommandClient::connect_handshake_with(options, timeout)`,
  which validates `options.stream_version(&info)`; `connect_handshake` shares
  the same private exchange-and-validate helper and keeps its behaviour.
- `README.md`: one paragraph on the default hello and the opt-in request.
- Tests: `connect_options_use_the_sdk_layout` and
  `stream_version_follows_the_sdk_rule` (`tests/command.rs`);
  `handshake_with_a_requested_version_validates_that_version`
  (`tests/net_command.rs`), in which a loopback server answering 4.2 is
  refused by the default handshake and accepted when 4.1 is requested.

## Not done

No consumer requests a version yet; natnet_zenoh and motive-bridge still send
the default hello. Enabling it needs a live Motive test of what Motive
streams and what its SERVERINFO reports after the request.
