//! Version-carrying NatNet decoding and checked re-encoding.

use crate::message;
use crate::{Error, FrameOfData, Message, ModelDef, ParsePolicy, Result, ServerInfo, Version};

/// A supported protocol layout and parsing policy, independent of transport.
///
/// Construct from the version reported by [`ServerInfo`]. Strict validation is
/// the default; [`Self::with_policy`] selects historical compatibility behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decoder {
    version: Version,
    policy: ParsePolicy,
}

impl Decoder {
    /// Selects a supported layout with strict validation.
    ///
    /// # Errors
    /// Returns [`Error::UnsupportedVersion`] for an unlisted major/minor pair.
    pub fn new(version: Version) -> Result<Self> {
        version.validate()?;
        Ok(Self {
            version,
            policy: ParsePolicy::Strict,
        })
    }

    /// Selects the explicit parsing policy while retaining the same layout.
    #[must_use]
    pub const fn with_policy(mut self, policy: ParsePolicy) -> Self {
        self.policy = policy;
        self
    }

    /// The reported version whose layout this decoder uses.
    #[must_use]
    pub const fn version(self) -> Version {
        self.version
    }

    /// The validation policy used for known messages.
    #[must_use]
    pub const fn policy(self) -> ParsePolicy {
        self.policy
    }

    /// Decodes one datagram, retaining the layout for subsequent encoding.
    ///
    /// Unknown message IDs remain opaque. The outer declared length remains
    /// telemetry-only under both policies.
    ///
    /// # Errors
    /// Returns header, field or strict-validation errors for known messages.
    pub fn parse(self, datagram: &[u8]) -> Result<DecodedMessage> {
        let (header, payload) = message::split(datagram)?;
        let message = match header.message_id {
            message::NAT_SERVERINFO => {
                Message::ServerInfo(ServerInfo::parse_payload_with_policy(payload, self.policy)?)
            }
            message::NAT_MODELDEF => Message::ModelDef(ModelDef::parse_payload_with_policy(
                payload,
                self.version,
                self.policy,
            )?),
            message::NAT_FRAMEOFDATA => Message::FrameOfData(Box::new(
                FrameOfData::parse_payload_with_policy(payload, self.version, self.policy)?,
            )),
            id => Message::Other {
                id,
                payload: payload.to_vec(),
            },
        };
        Ok(DecodedMessage {
            message,
            version: self.version,
        })
    }
}

/// A decoded message retaining the layout selected for its interpretation.
///
/// Read-only access preserves that pairing. [`Self::into_message`] deliberately
/// removes it so a caller can reconstruct or explicitly convert plain values.
#[derive(Debug, Clone, PartialEq)]
pub struct DecodedMessage {
    message: Message,
    version: Version,
}

impl DecodedMessage {
    /// The decoded plain value, without mutable access to its source pairing.
    #[must_use]
    pub const fn message(&self) -> &Message {
        &self.message
    }

    /// The version selected by the decoder, including build/revision provenance.
    #[must_use]
    pub const fn version(&self) -> Version {
        self.version
    }

    /// Removes the source-layout pairing and returns the plain model.
    #[must_use]
    pub fn into_message(self) -> Message {
        self.message
    }

    /// Re-encodes using the original source layout.
    ///
    /// Decoding is not always reversible through the checked encoders. Names
    /// are decoded with lossy UTF-8 replacement, so a decoded SERVERINFO
    /// application name can be longer than the 255 bytes its field accepts
    /// (a 256-byte field without NUL, or invalid bytes each widened to a
    /// 3-byte U+FFFD; see [`ServerInfo::application_name`](crate::ServerInfo::application_name)),
    /// and replacement can also grow a FrameOfData or MODELDEF payload past
    /// the 65 535 bytes a header can declare. A message decoded from a buffer
    /// longer than a UDP datagram can exceed that limit too. Each such case
    /// returns an error rather than a datagram that differs from the value.
    ///
    /// # Errors
    /// Returns checked encoding errors, including values accepted only by
    /// compatibility decoding that cannot be represented faithfully, such as
    /// [`Error::LengthOverflow`](crate::Error::LengthOverflow) for the cases
    /// above.
    pub fn to_datagram(&self) -> Result<Vec<u8>> {
        self.to_datagram_for(self.version)
    }

    /// Explicitly re-encodes in a supported destination layout.
    ///
    /// Cross-layout non-empty raw frame sections, MODELDEF rigid-body tails
    /// (including skeleton bones and asset rigid bodies) and
    /// [`DataSet::Other`](crate::DataSet::Other) datasets require explicit
    /// reconstruction through [`Self::into_message`]. Unsupported
    /// assets or precision fields are refused by the destination encoder.
    ///
    /// # Errors
    /// Returns unsupported-version, opaque-conversion or checked encoder errors.
    pub fn to_datagram_for(&self, version: Version) -> Result<Vec<u8>> {
        let changes_layout = version.layout()? != self.version.layout()?;
        match &self.message {
            Message::ServerInfo(info) => info.to_datagram(),
            Message::FrameOfData(frame) => {
                if changes_layout
                    && [
                        &frame.skeletons,
                        &frame.assets,
                        &frame.force_plates,
                        &frame.devices,
                    ]
                    .iter()
                    .any(|section| section.count != 0 || !section.payload.is_empty())
                {
                    return Err(Error::InvalidValue {
                        field: "opaque frame sections",
                        reason: "cross-layout encoding requires explicit reconstruction",
                    });
                }
                frame.to_datagram(version)
            }
            Message::ModelDef(model) => {
                if changes_layout
                    && model.datasets.iter().any(|dataset| match dataset {
                        crate::DataSet::RigidBody(body) => !body.tail.is_empty(),
                        // Bone and asset rigid-body tails carry the source
                        // layout's marker description.
                        crate::DataSet::Skeleton(skeleton) => skeleton
                            .rigid_bodies
                            .iter()
                            .any(|body| !body.tail.is_empty()),
                        crate::DataSet::Asset(asset) => {
                            asset.rigid_bodies.iter().any(|body| !body.tail.is_empty())
                        }
                        crate::DataSet::Other { .. } => true,
                        // Identical layouts in NatNet 3.0 through 4.1.
                        crate::DataSet::MarkerSet(_)
                        | crate::DataSet::ForcePlate(_)
                        | crate::DataSet::Device(_)
                        | crate::DataSet::Camera(_) => false,
                    })
                {
                    return Err(Error::InvalidValue {
                        field: "opaque MODELDEF data",
                        reason: "cross-layout encoding requires explicit reconstruction",
                    });
                }
                model.to_datagram_version(version)
            }
            Message::Other { id, payload } => message::frame_checked(*id, payload),
        }
    }
}
