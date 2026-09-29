//! The NatNet protocol (bitstream) version and the version-negotiation
//! surface: never assume a fixed version; branch on the bitstream.

use crate::{Error, Result};

/// Validation of otherwise bounded, structurally decodable input.
///
/// Both policies tolerate the NatNet outer header's telemetry-only size and
/// preserve explicitly opaque extensions. Strict decoding additionally rejects
/// negative raw counts, a nonzero frame terminator and unexplained residue.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ParsePolicy {
    /// Reject invalid counts, terminators and unexplained trailing bytes.
    #[default]
    Strict,
    /// Preserve the historical standalone parsers' extension tolerance.
    Compatible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Layout {
    Legacy3,
    Legacy4,
    V4_1,
}

impl Layout {
    pub(crate) fn has_section_sizes(self) -> bool {
        self == Self::V4_1
    }

    pub(crate) fn has_assets(self) -> bool {
        self == Self::V4_1
    }

    pub(crate) fn has_precision(self) -> bool {
        self == Self::V4_1
    }

    pub(crate) fn rigid_body_marker_names(self) -> bool {
        self != Self::Legacy3
    }
}

/// A four-part version quadruple `major.minor.build.revision`.
///
/// The NatNet **protocol** version is carried as the four bitstream bytes of
/// a SERVERINFO reply (e.g. `04 01 00 00` = 4.1.0.0). The same shape holds
/// the server application version (e.g. Motive `03 01 02 01` = 3.1.2.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Version {
    /// Major version.
    pub major: u8,
    /// Minor version.
    pub minor: u8,
    /// Build number.
    pub build: u8,
    /// Revision.
    pub revision: u8,
}

impl Version {
    /// The NatNet 4.1.0.0 protocol version — the bitstream this crate is
    /// checked against real Motive captures at.
    pub const V4_1: Self = Self::new(4, 1, 0, 0);

    /// Verifies that the major/minor pair names a supported wire layout.
    ///
    /// Build and revision are retained as provenance; supported layouts use
    /// the vendor sample's major/minor field selection.
    ///
    /// # Errors
    /// Returns [`Error::UnsupportedVersion`] for an unlisted layout.
    pub fn validate(self) -> Result<()> {
        self.layout().map(|_| ())
    }

    pub(crate) fn layout(self) -> Result<Layout> {
        match (self.major, self.minor) {
            (3, 0 | 1) => Ok(Layout::Legacy3),
            (4, 0) => Ok(Layout::Legacy4),
            (4, 1) => Ok(Layout::V4_1),
            _ => Err(Error::UnsupportedVersion {
                major: self.major,
                minor: self.minor,
            }),
        }
    }

    /// Builds a version from its four parts.
    #[must_use]
    pub const fn new(major: u8, minor: u8, build: u8, revision: u8) -> Self {
        Self {
            major,
            minor,
            build,
            revision,
        }
    }

    /// Decodes the four little-endian bitstream bytes as they appear on the
    /// wire (`[major, minor, build, revision]`).
    #[must_use]
    pub const fn from_bytes(b: [u8; 4]) -> Self {
        Self::new(b[0], b[1], b[2], b[3])
    }

    /// The four wire bytes.
    #[must_use]
    pub const fn to_bytes(self) -> [u8; 4] {
        [self.major, self.minor, self.build, self.revision]
    }

    /// Whether this version is at least `major.minor`. This comparison does
    /// not imply that the version is a supported wire layout.
    #[must_use]
    pub const fn at_least(self, major: u8, minor: u8) -> bool {
        self.major > major || (self.major == major && self.minor >= minor)
    }
}

impl core::fmt::Display for Version {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.major, self.minor, self.build, self.revision
        )
    }
}
