//! SERVERINFO (message id 1): server identity, protocol version, clock
//! frequency and data-channel parameters.
//!
//! Layout, confirmed against a real Motive 3.1.2.1 reply:
//!
//! | offset | field | notes |
//! |---|---|---|
//! | 0 | `application_name[256]` | Decoding stops at the first NUL; stale bytes may follow |
//! | 256 | `server_version[4]` | major, minor, build, revision |
//! | 260 | `protocol_version[4]` | bitstream version |
//! | 264 | `high_res_clock_frequency` (`u64`) | ticks per second |
//! | 272 | `data_port` (`u16`) | UDP data port |
//! | 274 | `multicast` (`u8` bool) | 1 = multicast |
//! | 275 | `multicast_group[4]` | IPv4 multicast group |

use crate::bytes::{Reader, Writer};
use crate::error::Result;
use crate::message::{self, NAT_SERVERINFO};
use crate::version::{ParsePolicy, Version};

/// Width of the application-name field.
const NAME_FIELD: usize = 256;

/// The data-channel connection parameters advertised in SERVERINFO.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConnectionInfo {
    /// UDP data-stream port (e.g. 1511).
    pub data_port: u16,
    /// Whether the server streams multicast; `false` means unicast.
    pub multicast: bool,
    /// Multicast group address octets (meaningful when `multicast`).
    pub multicast_group: [u8; 4],
}

/// A decoded SERVERINFO reply.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ServerInfo {
    /// NUL-stop lossy UTF-8; a 256-byte field can expand to 768. [`Self::to_payload`]
    /// rejects names over 255 bytes with [`Error::LengthOverflow`](crate::Error::LengthOverflow);
    /// [`Self::to_payload_lossy`] truncates.
    pub application_name: String,
    /// Server application version (e.g. Motive 3.1.2.1).
    pub server_version: Version,
    /// NatNet **protocol** (bitstream) version to decode subsequent frames
    /// with (e.g. 4.1.0.0).
    pub protocol_version: Version,
    /// High-resolution clock frequency in ticks per second (e.g. 10 MHz).
    /// The frame's camera timestamps are expressed in these ticks.
    pub high_res_clock_frequency: u64,
    /// Data-channel connection parameters.
    pub connection: ConnectionInfo,
}

impl ServerInfo {
    /// Parses a SERVERINFO **datagram** (including the 4-byte header).
    /// # Errors
    /// [`Error::UnexpectedMessageId`](crate::Error::UnexpectedMessageId) if
    /// the header is not id 1; [`Error::UnexpectedEof`](crate::Error::UnexpectedEof)
    /// if any field runs past the end.
    pub fn parse(datagram: &[u8]) -> Result<Self> {
        Self::parse_payload(message::expect(datagram, NAT_SERVERINFO)?)
    }

    /// Parses a datagram with explicit validation policy.
    /// # Errors
    /// Header or field errors, including [`Error::TrailingBytes`](crate::Error::TrailingBytes)
    /// for unexplained suffix bytes in strict mode.
    pub fn parse_with_policy(datagram: &[u8], policy: ParsePolicy) -> Result<Self> {
        Self::parse_payload_with_policy(message::expect(datagram, NAT_SERVERINFO)?, policy)
    }

    /// Parses a SERVERINFO payload (the bytes **after** the 4-byte header).
    /// # Errors
    /// [`Error::UnexpectedEof`](crate::Error::UnexpectedEof) if any field
    /// runs past the end of the payload.
    pub fn parse_payload(payload: &[u8]) -> Result<Self> {
        Self::parse_payload_with_policy(payload, ParsePolicy::Compatible)
    }

    /// Parses a payload and applies the selected suffix-byte policy. The
    /// reported version is retained even if frame decoding does not support it;
    /// call [`Version::validate`] before decoding frames.
    /// # Errors
    /// Field errors or [`crate::Error::TrailingBytes`] in strict mode.
    pub fn parse_payload_with_policy(payload: &[u8], policy: ParsePolicy) -> Result<Self> {
        let mut r = Reader::new(payload);
        let application_name = r.fixed_cstr(NAME_FIELD)?;
        let server_version = Version::from_bytes(r.array()?);
        let protocol_version = Version::from_bytes(r.array()?);
        let high_res_clock_frequency = r.u64()?;
        let data_port = r.u16()?;
        let multicast = r.u8()? != 0;
        let multicast_group = r.array()?;
        if policy == ParsePolicy::Strict {
            r.finish()?;
        }
        Ok(Self {
            application_name,
            server_version,
            protocol_version,
            high_res_clock_frequency,
            connection: ConnectionInfo {
                data_port,
                multicast,
                multicast_group,
            },
        })
    }

    /// Serializes a SERVERINFO payload without truncating the name.
    /// # Errors
    /// Refuses interior NUL or a name longer than 255 UTF-8 bytes. A value
    /// decoded from the wire can exceed that limit; see
    /// [`Self::application_name`].
    pub fn to_payload(&self) -> Result<Vec<u8>> {
        let mut w = Writer::new();
        w.try_fixed_cstr(&self.application_name, NAME_FIELD, "application_name")?;
        w.bytes(&self.server_version.to_bytes());
        w.bytes(&self.protocol_version.to_bytes());
        w.u64(self.high_res_clock_frequency);
        w.u16(self.connection.data_port);
        w.u8(u8::from(self.connection.multicast));
        w.bytes(&self.connection.multicast_group);
        Ok(w.into_vec())
    }

    /// Serializes to a full SERVERINFO datagram (4-byte header + payload).
    /// # Errors
    /// See [`Self::to_payload`].
    pub fn to_datagram(&self) -> Result<Vec<u8>> {
        message::frame_checked(NAT_SERVERINFO, &self.to_payload()?)
    }

    /// Encodes without name validation, truncating to 255 bytes and allowing
    /// interior NUL. Prefer [`Self::to_payload`].
    #[must_use]
    pub fn to_payload_lossy(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.fixed_cstr(&self.application_name, NAME_FIELD);
        w.bytes(&self.server_version.to_bytes());
        w.bytes(&self.protocol_version.to_bytes());
        w.u64(self.high_res_clock_frequency);
        w.u16(self.connection.data_port);
        w.u8(u8::from(self.connection.multicast));
        w.bytes(&self.connection.multicast_group);
        w.into_vec()
    }

    /// Encodes a datagram without name validation.
    #[must_use]
    pub fn to_datagram_lossy(&self) -> Vec<u8> {
        message::frame(NAT_SERVERINFO, &self.to_payload_lossy())
    }

    /// The multicast group as an [`Ipv4Addr`](std::net::Ipv4Addr).
    #[must_use]
    pub fn multicast_group(&self) -> std::net::Ipv4Addr {
        self.connection.multicast_group.into()
    }
}
