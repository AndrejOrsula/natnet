//! The 4-byte NatNet message framing header and message-id constants.
//!
//! Every NatNet datagram begins with `u16 message_id` then `u16
//! declared_payload_size`, both little-endian. The declared size is **not a
//! contract** (it varies with Motive streaming settings), so the parsers here
//! read the *actual* remaining bytes and use the declared size only when
//! serializing.

use crate::bytes::Reader;
use crate::error::{Error, Result};

/// NAT_CONNECT (client hello) request, sent through the command transport.
pub const NAT_CONNECT: u16 = 0;
/// SERVERINFO — reply to a connect.
pub const NAT_SERVERINFO: u16 = 1;
/// NAT_REQUEST — generic request.
pub const NAT_REQUEST: u16 = 2;
/// NAT_RESPONSE — generic reply.
pub const NAT_RESPONSE: u16 = 3;
/// NAT_REQUEST_MODELDEF — request for the data descriptions.
pub const NAT_REQUEST_MODELDEF: u16 = 4;
/// NAT_MODELDEF — data-description (model definition) reply.
pub const NAT_MODELDEF: u16 = 5;
/// NAT_REQUEST_FRAMEOFDATA — request one frame.
pub const NAT_REQUEST_FRAMEOFDATA: u16 = 6;
/// NAT_FRAMEOFDATA — a streamed data frame.
pub const NAT_FRAMEOFDATA: u16 = 7;
/// NAT_MESSAGESTRING — an asynchronous string message; see
/// [`parse_message_string`].
pub const NAT_MESSAGESTRING: u16 = 8;
/// NAT_DISCONNECT — a client announcing that it leaves. Carries no payload
/// and has no reply.
pub const NAT_DISCONNECT: u16 = 9;
/// NAT_KEEPALIVE — the periodic liveness message a unicast client sends on the
/// command channel. Carries no payload and has no reply.
pub const NAT_KEEPALIVE: u16 = 10;
/// NAT_DISCONNECTBYTIMEOUT — defined by the NatNet SDK 4.1 `NatNetTypes.h`;
/// the SDK samples neither send nor handle it.
pub const NAT_DISCONNECTBYTIMEOUT: u16 = 11;
/// NAT_ECHOREQUEST — defined by the NatNet SDK 4.1 `NatNetTypes.h`. The SDK
/// documents no payload layout for it; this crate only names the id.
pub const NAT_ECHOREQUEST: u16 = 12;
/// NAT_ECHORESPONSE — the reply to [`NAT_ECHOREQUEST`]; id only, as above.
pub const NAT_ECHORESPONSE: u16 = 13;
/// NAT_DISCOVERY — defined by the NatNet SDK 4.1 `NatNetTypes.h`; id only.
pub const NAT_DISCOVERY: u16 = 14;
/// NAT_UNRECOGNIZED_REQUEST — the reply a NatNet server sends to a request it
/// does not recognize (`NatNetTypes.h` and the SDK samples define it as 100).
pub const NAT_UNRECOGNIZED_REQUEST: u16 = 100;

/// The fixed 4-byte header size.
pub const HEADER_LEN: usize = 4;

/// The parsed 4-byte framing header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageHeader {
    /// Little-endian message id at offset 0.
    pub message_id: u16,
    /// Little-endian declared payload size at offset 2. **Not a contract**;
    /// recorded but never used to bound decoding.
    pub declared_size: u16,
}

impl MessageHeader {
    /// Parses the leading four bytes.
    ///
    /// # Errors
    ///
    /// [`Error::UnexpectedEof`] if `data` is shorter than four bytes.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        Ok(Self {
            message_id: r.u16()?,
            declared_size: r.u16()?,
        })
    }
}

/// Splits a datagram into its header and the payload bytes that follow it.
///
/// The payload is the *actual* trailing bytes (`data[4..]`), not the header's
/// declared size — declared size is telemetry only.
///
/// # Errors
///
/// [`Error::UnexpectedEof`] for a datagram shorter than the 4-byte header.
pub fn split(data: &[u8]) -> Result<(MessageHeader, &[u8])> {
    let header = MessageHeader::parse(data)?;
    Ok((header, &data[HEADER_LEN..]))
}

/// Parses a NAT_MESSAGESTRING datagram (including the 4-byte header) into its
/// text.
///
/// The payload is a NUL-terminated string, as the NatNet SDK 4.1
/// `PacketClient.cpp` sample prints it; the text ends at the first NUL, or at
/// the end of the payload when there is none. Bytes that are not UTF-8 are
/// replaced with U+FFFD.
///
/// # Errors
///
/// [`Error::UnexpectedEof`] for a datagram shorter than the header;
/// [`Error::UnexpectedMessageId`] when the header is not
/// [`NAT_MESSAGESTRING`].
pub fn parse_message_string(datagram: &[u8]) -> Result<String> {
    let payload = expect(datagram, NAT_MESSAGESTRING)?;
    let end = payload
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(payload.len());
    Ok(String::from_utf8_lossy(&payload[..end]).into_owned())
}

/// Confirms the datagram's header carries `expected` and returns the payload.
pub(crate) fn expect(data: &[u8], expected: u16) -> Result<&[u8]> {
    let (header, payload) = split(data)?;
    if header.message_id != expected {
        return Err(Error::UnexpectedMessageId {
            found: header.message_id,
            expected,
        });
    }
    Ok(payload)
}

/// Prepends the 4-byte framing header to a payload, stamping the declared
/// size with the true payload length (the honest value; the serializers do
/// not fabricate a mismatched size).
///
/// This is the lossy framer, used by the `*_lossy` encoders and by command
/// paths whose payload is already bounded: a payload longer than `u16::MAX`
/// gets a saturated declared size (non-contractual on decode) and is still
/// emitted in full. Checked encoders use [`frame_checked`].
pub(crate) fn frame(message_id: u16, payload: &[u8]) -> Vec<u8> {
    let declared = u16::try_from(payload.len()).unwrap_or(u16::MAX);
    frame_declared(message_id, declared, payload)
}

/// Checked framing: refuses a payload the `u16` declared size cannot state.
///
/// # Errors
/// [`Error::LengthOverflow`] (`field: "message payload size"`) for a payload
/// longer than 65 535 bytes.
pub(crate) fn frame_checked(message_id: u16, payload: &[u8]) -> Result<Vec<u8>> {
    let declared = u16::try_from(payload.len()).map_err(|_| Error::LengthOverflow {
        field: "message payload size",
        length: payload.len(),
        max: usize::from(u16::MAX),
    })?;
    Ok(frame_declared(message_id, declared, payload))
}

fn frame_declared(message_id: u16, declared: u16, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER_LEN + payload.len());
    out.extend_from_slice(&message_id.to_le_bytes());
    out.extend_from_slice(&declared.to_le_bytes());
    out.extend_from_slice(payload);
    out
}
