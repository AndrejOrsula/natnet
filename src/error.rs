//! Crate errors, defined here to keep the core dependency-free.

/// A NatNet decode/encode failure.
///
/// Offsets and count/size context identify where decoding or encoding failed.
/// Malformed input returns an error rather than panicking.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// A signed wire count or size is negative. Typed fields reject this under
    /// every policy; compatible 4.1 decoding retains opaque raw-frame counts.
    InvalidCount {
        /// Signed value read from the wire.
        count: i32,
        /// Absolute payload offset of the count or size field.
        offset: usize,
    },
    /// A strict frame decoder found a nonzero end marker.
    InvalidTerminator {
        /// Value read from the end-marker field.
        value: i32,
        /// Absolute payload offset of that field.
        offset: usize,
    },
    /// An unframed legacy dataset has no known structural extent.
    UnsupportedDataSet {
        /// Dataset type from the wire.
        type_id: i32,
        /// Protocol major version.
        major: u8,
        /// Protocol minor version.
        minor: u8,
    },
    /// An encoding length cannot be represented by its wire field.
    LengthOverflow {
        /// Field being encoded.
        field: &'static str,
        /// Offered length or count.
        length: usize,
        /// Largest representable value.
        max: usize,
    },
    /// A string offered for encoding contains an interior NUL.
    InteriorNul {
        /// String field being encoded.
        field: &'static str,
        /// Byte index within the offered string.
        offset: usize,
    },
    /// A value cannot be represented faithfully in the selected layout.
    InvalidValue {
        /// Field being encoded or validated.
        field: &'static str,
        /// Explanation of the violated constraint.
        reason: &'static str,
    },
    /// A record prefix contains the wrong four-byte magic.
    InvalidMagic {
        /// Full magic found on the wire.
        found: u32,
        /// Required magic.
        expected: u32,
    },
    /// The input ended before a field of `needed` bytes could be read at
    /// byte `offset` (only `available` bytes remained).
    UnexpectedEof {
        /// Byte offset at which the read was attempted.
        offset: usize,
        /// Bytes the field required.
        needed: usize,
        /// Bytes actually remaining from `offset`.
        available: usize,
    },
    /// A NUL-terminated string ran to the end of the buffer without a
    /// terminator, starting at `offset`.
    UnterminatedString {
        /// Byte offset at which the string started.
        offset: usize,
    },
    /// A section or dataset size differs from bytes consumed.
    SectionSizeMismatch {
        /// What the section header declared.
        declared: usize,
        /// What parsing the entries consumed.
        consumed: usize,
    },
    /// A count cannot fit the remaining input.
    ImplausibleCount {
        /// The declared entry count.
        count: usize,
        /// Bytes remaining when it was read.
        available: usize,
    },
    /// The message id differs from the expected id.
    UnexpectedMessageId {
        /// The id found in the header.
        found: u16,
        /// The id the caller expected.
        expected: u16,
    },
    /// The negotiated major/minor pair is not a supported wire layout.
    UnsupportedVersion {
        /// Major protocol version found.
        major: u8,
        /// Minor protocol version found.
        minor: u8,
    },
    /// Strict decoding found trailing bytes after a payload, including SERVERINFO.
    TrailingBytes {
        /// Bytes left unconsumed.
        remaining: usize,
    },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidCount { count, offset } => {
                write!(f, "invalid count {count} at byte {offset}")
            }
            Self::InvalidTerminator { value, offset } => {
                write!(f, "invalid frame terminator {value} at byte {offset}")
            }
            Self::UnsupportedDataSet {
                type_id,
                major,
                minor,
            } => {
                write!(
                    f,
                    "unsupported unframed dataset {type_id} for NatNet {major}.{minor}"
                )
            }
            Self::LengthOverflow { field, length, max } => {
                write!(f, "{field} length {length} exceeds wire limit {max}")
            }
            Self::InteriorNul { field, offset } => {
                write!(f, "interior NUL in {field} at byte {offset}")
            }
            Self::InvalidValue { field, reason } => write!(f, "invalid {field}: {reason}"),
            Self::InvalidMagic { found, expected } => {
                write!(
                    f,
                    "invalid record magic 0x{found:08x}, expected 0x{expected:08x}"
                )
            }
            Self::UnexpectedEof {
                offset,
                needed,
                available,
            } => write!(
                f,
                "unexpected end of input at byte {offset}: needed {needed}, {available} available"
            ),
            Self::UnterminatedString { offset } => {
                write!(f, "unterminated string starting at byte {offset}")
            }
            Self::SectionSizeMismatch { declared, consumed } => write!(
                f,
                "section size mismatch: header declared {declared} bytes, entries consumed {consumed}"
            ),
            Self::ImplausibleCount { count, available } => write!(
                f,
                "implausible entry count {count} with only {available} bytes available"
            ),
            Self::UnexpectedMessageId { found, expected } => {
                write!(f, "unexpected message id {found} (expected {expected})")
            }
            Self::UnsupportedVersion { major, minor } => {
                write!(f, "unsupported NatNet protocol version {major}.{minor}")
            }
            Self::TrailingBytes { remaining } => {
                write!(f, "{remaining} trailing bytes after message")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Convenience alias for a [`Result`](core::result::Result) with this
/// crate's [`Error`].
pub type Result<T> = core::result::Result<T, Error>;
