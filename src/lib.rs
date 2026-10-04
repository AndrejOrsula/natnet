//! A sans-io OptiTrack NatNet protocol core.
//!
//! Decodes [`ServerInfo`] (id 1), [`ModelDef`] (id 5), and [`FrameOfData`]
//! (id 7). [`Decoder`] validates the negotiated bitstream version, defaults to
//! [`ParsePolicy::Strict`], and retains the source layout in [`DecodedMessage`].
//! Supported major/minor layouts are 3.0, 3.1, 4.0, and 4.1; build and revision
//! are retained without changing field selection. Other layouts are refused.
//!
//! # Encoding and compatibility
//!
//! Frame, MODELDEF, and SERVERINFO encoders return [`Result`] and reject values that cannot be
//! represented faithfully: interior NULs, overflowing lengths (including a
//! datagram payload longer than the 65 535 bytes its header can declare),
//! malformed raw sections, and fields unavailable in the selected layout.
//! Explicitly named `*_lossy` methods retain historical unchecked encoding
//! behavior.
//!
//! [`DecodedMessage::to_datagram`] uses the source layout. Explicit conversion
//! refuses opaque layout-dependent data until the caller reconstructs it.
//! Floating-point bits, including NaNs, are preserved; NaNs still compare
//! unequal. Decoding strings uses lossy UTF-8, so arbitrary wire bytes need not
//! survive string decoding and re-encoding byte for byte.
//!
//! Strict parsing rejects invalid counts, terminators, and unexplained residue.
//! [`ParsePolicy::Compatible`] selects extension-tolerant parsing. Both policies
//! treat the outer header size as telemetry and preserve unknown message payloads.
//!
//! # Dependencies and transport
//!
//! The default runtime dependency graph is empty. Optional `serde` derives and
//! `nalgebra` conversions leave the wire model as plain arrays. Coordinates and
//! quaternions retain the sender's convention; no axis conversion is performed.
//! The crate forbids unsafe code.
//!
//! With `net`, the data client receives UDP. The typed command client sends
//! validated requests with peer filtering and bounded transactions; an explicit
//! raw client permits unrestricted commands. Tier-1 execution checks a canary
//! and verifies each subsequent write. Peer filtering is not authentication.
//! Replies already queued are discarded before each request, but NatNet has no
//! transaction identifier to distinguish a late peer reply that arrives after
//! that.
//!
//! With `imagery`, AUGI/1 decoding handles the separate motive-bridge image
//! stream without decoding image codecs. Combining `imagery` and `net` adds a
//! TCP client with bounded records and deadlines.
//!
//! # Generate and round-trip a frame
//!
//! ```
//! # fn main() -> natnet::Result<()> {
//! use natnet::{Decoder, Message, PlannedBody, SyntheticServer, Version};
//!
//! let server = SyntheticServer::new(Version::V4_1, 1_000_000);
//! let body = PlannedBody::tracked(7, [1.0, 2.0, 3.0], [0.0, 0.0, 0.0, 1.0]);
//! let wire = server.datagram(42, 0.5, &[body])?;
//! let decoded = Decoder::new(Version::V4_1)?.parse(&wire)?;
//!
//! let Message::FrameOfData(frame) = decoded.message() else {
//!     panic!("expected a frame");
//! };
//! assert_eq!(frame.frame_number, 42);
//! assert_eq!(frame.rigid_bodies[0].id, 7);
//! assert_eq!(decoded.to_datagram()?, wire);
//! # Ok(())
//! # }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod bytes;
pub mod command;
mod decoder;
mod error;
pub mod frame;
pub mod generate;
mod message;
pub mod modeldef;
pub mod serverinfo;
mod version;

#[cfg(feature = "imagery")]
pub mod imagery;

#[cfg(feature = "nalgebra")]
mod nalgebra_conv;

#[cfg(feature = "net")]
pub mod net;

pub use decoder::{DecodedMessage, Decoder};
pub use error::{Error, Result};
pub use frame::{
    Asset, Device, ForcePlate, FrameOfData, FrameTiming, LabeledMarker, MarkerSet, RawSection,
    RigidBody, Skeleton,
};
pub use generate::{PlannedBody, PlannedSections, SyntheticServer};
pub use message::{
    HEADER_LEN, MessageHeader, NAT_CONNECT, NAT_DISCONNECT, NAT_DISCONNECTBYTIMEOUT, NAT_DISCOVERY,
    NAT_ECHOREQUEST, NAT_ECHORESPONSE, NAT_FRAMEOFDATA, NAT_KEEPALIVE, NAT_MESSAGESTRING,
    NAT_MODELDEF, NAT_REQUEST, NAT_REQUEST_FRAMEOFDATA, NAT_REQUEST_MODELDEF, NAT_RESPONSE,
    NAT_SERVERINFO, NAT_UNRECOGNIZED_REQUEST, parse_message_string,
};
pub use modeldef::{
    AssetDef, CameraDef, DataSet, DeviceDef, ForcePlateDef, MarkerDef, MarkerSetDef, ModelDef,
    RigidBodyDef, SkeletonDef,
};
pub use serverinfo::{ConnectionInfo, ServerInfo};
pub use version::{ParsePolicy, Version};

#[cfg(feature = "nalgebra")]
pub use nalgebra_conv::marker_vector;

/// A decoded NatNet message — the ones this crate understands, plus a
/// verbatim fallback for any other message id.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Message {
    /// SERVERINFO (id 1).
    ServerInfo(ServerInfo),
    /// MODELDEF (id 5).
    ModelDef(ModelDef),
    /// FrameOfData (id 7). Boxed: a frame is far larger than the other
    /// variants, so boxing keeps [`Message`] small to move around.
    FrameOfData(Box<FrameOfData>),
    /// Any other message id, payload preserved verbatim.
    Other {
        /// The message id from the header.
        id: u16,
        /// The payload bytes (after the 4-byte header).
        payload: Vec<u8>,
    },
}

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
