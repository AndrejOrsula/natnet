//! SERVERINFO (message id 1) — the reply that establishes the server
//! identity, the negotiated protocol version, the clock frequency and the
//! data-channel connection parameters.
//!
//! Layout (checked against a real Motive 3.1.2.1 reply, 279-byte payload;
//! the capture is not distributed with the crate):
//!
//! | offset | field | notes |
//! |---|---|---|
//! | 0 | `application_name[256]` | NUL-terminated; **stale bytes follow the NUL** |
//! | 256 | `server_version[4]` | e.g. `03 01 02 01` = 3.1.2.1 |
//! | 260 | `protocol_version[4]` | bitstream, e.g. `04 01 00 00` = 4.1.0.0 |
//! | 264 | `high_res_clock_frequency` (`u64`) | e.g. 10 000 000 |
//! | 272 | `data_port` (`u16`) | e.g. 1511 |
//! | 274 | `multicast` (`u8` bool) | 1 = multicast |
//! | 275 | `multicast_group[4]` | e.g. `ef ff 2a 63` = 239.255.42.99 |
//!
//! The application-name gotcha (seen in that capture): the 256-byte field is a
//! reused server buffer. The real capture holds `Motive\0` followed by
//! non-zero stale bytes (an address-like `.195:30149` fragment); decoding
//! **must stop at the first NUL** — the fixed-width string decoder does.

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
    /// Whether the server streams multicast (else unicast).
    pub multicast: bool,
    /// Multicast group address octets (meaningful when `multicast`).
    pub multicast_group: [u8; 4],
}

/// A decoded SERVERINFO reply.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ServerInfo {
    /// Server application name, decoded NUL-stop (e.g. `Motive`).
    ///
    /// Decoding is lossy, so a decoded name may not re-encode through the
    /// checked [`ServerInfo::to_payload`], which accepts at most 255 UTF-8
    /// bytes: a 256-byte field with no NUL decodes to a 256-byte name, and
    /// each invalid UTF-8 byte becomes a 3-byte U+FFFD, so a name of up to
    /// 255 wire bytes can decode to as many as 765. Such a value returns
    /// [`Error::LengthOverflow`](crate::Error::LengthOverflow) on checked
    /// re-encoding; [`ServerInfo::to_payload_lossy`] truncates it instead.
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
    ///
    /// # Errors
    ///
    /// [`Error::UnexpectedMessageId`](crate::Error::UnexpectedMessageId) if
    /// the header is not id 1; [`Error::UnexpectedEof`](crate::Error::UnexpectedEof)
    /// if any field runs past the end.
    pub fn parse(datagram: &[u8]) -> Result<Self> {
        Self::parse_payload(message::expect(datagram, NAT_SERVERINFO)?)
    }

    /// Parses a datagram with explicit strict or compatibility validation.
    ///
    /// # Errors
    /// Returns header/field errors, or unexplained suffix bytes in strict mode.
    pub fn parse_with_policy(datagram: &[u8], policy: ParsePolicy) -> Result<Self> {
        Self::parse_payload_with_policy(message::expect(datagram, NAT_SERVERINFO)?, policy)
    }

    /// Parses a SERVERINFO payload (the bytes **after** the 4-byte header).
    ///
    /// # Errors
    ///
    /// [`Error::UnexpectedEof`](crate::Error::UnexpectedEof) if any field
    /// runs past the end of the payload.
    pub fn parse_payload(payload: &[u8]) -> Result<Self> {
        Self::parse_payload_with_policy(payload, ParsePolicy::Compatible)
    }

    /// Parses a payload with explicit validation of unexplained suffix bytes.
    ///
    /// The reported version is retained even when the crate cannot decode that
    /// version's frames. Call [`Version::validate`] before using it for frames.
    ///
    /// # Errors
    /// Returns field errors or [`crate::Error::TrailingBytes`] in strict mode.
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

    /// Serializes a SERVERINFO payload without truncating the application name.
    ///
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
    ///
    /// # Errors
    /// See [`Self::to_payload`].
    pub fn to_datagram(&self) -> Result<Vec<u8>> {
        message::frame_checked(NAT_SERVERINFO, &self.to_payload()?)
    }

    /// Historical lossy payload encoding: truncates the application name to
    /// 255 bytes and permits interior NUL. Prefer [`Self::to_payload`].
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

    /// Historical lossy datagram encoding. See [`Self::to_payload_lossy`].
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
