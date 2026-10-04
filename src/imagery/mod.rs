//! **AUGI/1** camera-image side channel (`imagery` feature). Not part of
//! NatNet. Parse records with [`parse_record`].
//!
//! Every record is little-endian:
//!
//! | offset | field | notes |
//! |---|---|---|
//! | 0 | `u32 magic` | [`MAGIC`] |
//! | 4 | `u16 kind` | [`KIND_HELLO`] to [`KIND_BYE`] |
//! | 6 | `u16 header_len` | version mechanism, see below |
//! | 8 | `u32 payload_len` | bytes after the header |
//! | 12 | `header[header_len]` | fixed-shape fields |
//! | 12 + `header_len` | `payload[payload_len]` | variable-length bytes |
//!
//! - **Versioning:** a minor version may append header fields. Readers skip
//!   bytes past the header length they know; a shorter header is
//!   [`Error::UnexpectedEof`]. Known `Hello` and `CameraDesc` payloads have
//!   fixed shapes and reject bytes beyond those fields.
//! - **No codecs:** [`ImageFrame`] carries encoded bytes and an
//!   [`ImageEncoding`] tag.
//! - **Checked encoding:** serializers reject values that would not parse
//!   back identically (see each `# Errors`).
//! - **Floats:** bits survive; `PartialEq` follows IEEE-754, so NaN != NaN.

use crate::bytes::{Reader, Writer};
use crate::error::{Error, Result};

#[cfg(feature = "net")]
pub mod net;

/// Record magic: the ASCII bytes `A U G I` read as a little-endian `u32`.
pub const MAGIC: u32 = 0x4947_5541;

/// Bytes before the header: magic, kind, `header_len`, `payload_len`.
pub const PREFIX_LEN: usize = 12;

/// [`Hello`] record kind.
pub const KIND_HELLO: u16 = 1;
/// [`CameraDesc`] record kind.
pub const KIND_CAMERA_DESC: u16 = 2;
/// [`ImageFrame`] record kind.
pub const KIND_IMAGE_FRAME: u16 = 3;
/// Heartbeat record kind (no header, no payload).
pub const KIND_HEARTBEAT: u16 = 4;
/// Bye record kind (no header, no payload).
pub const KIND_BYE: u16 = 5;

/// Header length this crate writes for [`Hello`].
pub const HELLO_HEADER_LEN: usize = 41;
/// Header length this crate writes for [`CameraDesc`].
pub const CAMERA_DESC_HEADER_LEN: usize = 72;
/// Header length this crate writes for [`ImageFrame`].
pub const IMAGE_FRAME_HEADER_LEN: usize = 40;

/// AUGI protocol major version this crate implements.
pub const PROTO_MAJOR: u16 = 1;
/// AUGI protocol minor version this crate implements.
pub const PROTO_MINOR: u16 = 0;

/// [`Hello::mode`]: the bridge accepted an inbound TCP connection.
pub const MODE_LISTEN: u8 = 0;
/// [`Hello::mode`]: the sender dialled the consumer.
pub const MODE_CONNECT: u8 = 1;

/// [`ImageFrame`] header flag bit 0: `camera_mid_exposure_ticks` is valid.
pub const FLAG_MID_EXPOSURE_VALID: u8 = 0x01;

/// Wire value for "this camera sends no imagery".
pub const ENCODING_NONE: u8 = 0;

/// How the bytes in an [`ImageFrame::payload`] are encoded.
///
/// Codec bytes are not decoded here; `Other` preserves unnamed wire values,
/// including [`ENCODING_NONE`].
///
/// `Other(1)` and `Other(2)` write wire bytes 1 and 2, which parse as
/// [`Mono8`](Self::Mono8) and [`Jpeg`](Self::Jpeg). Parsing chooses the named
/// variant; re-encoding preserves the wire byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ImageEncoding {
    /// One byte per pixel, row-major, `width * height` bytes, no padding.
    Mono8,
    /// A complete JPEG/MJPEG frame.
    Jpeg,
    /// Any other wire value, preserved verbatim.
    Other(u8),
}

impl ImageEncoding {
    /// Decodes the wire byte.
    #[must_use]
    pub const fn from_wire(value: u8) -> Self {
        match value {
            1 => Self::Mono8,
            2 => Self::Jpeg,
            other => Self::Other(other),
        }
    }

    /// The wire byte.
    #[must_use]
    pub const fn to_wire(self) -> u8 {
        match self {
            Self::Mono8 => 1,
            Self::Jpeg => 2,
            Self::Other(value) => value,
        }
    }
}

/// Stream preamble (kind [`KIND_HELLO`]): clock and version metadata; its
/// three `u16` length-prefixed UTF-8 strings are in the payload. Invalid UTF-8
/// is replaced lossily.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hello {
    /// AUGI major version ([`PROTO_MAJOR`]).
    pub proto_major: u16,
    /// AUGI minor version ([`PROTO_MINOR`]).
    pub proto_minor: u16,
    /// Ticks per second of the clock [`ImageFrame::camera_mid_exposure`] is
    /// expressed in — the NatNet `high_res_clock_frequency`, typically
    /// 10 MHz.
    pub clock_hz: u64,
    /// Ticks per second of the Windows performance counter
    /// ([`ImageFrame::grab_ticks`]).
    pub qpf: u64,
    /// NatNet bitstream major version the bridge re-serves.
    pub natnet_major: u16,
    /// NatNet bitstream minor version the bridge re-serves.
    pub natnet_minor: u16,
    /// Motive build number reported by the API.
    pub motive_build: u32,
    /// Host wall clock at Hello, as Unix seconds. **Provenance only — never a
    /// correlation key.** A Motive host's clock may be deliberately wrong.
    pub host_wall_unix_secs: u64,
    /// [`MODE_LISTEN`] or [`MODE_CONNECT`].
    pub mode: u8,
    /// Number of [`CameraDesc`] records that follow.
    pub camera_count: u32,
    /// Operator-chosen identity of this bridge instance.
    pub bridge_id: String,
    /// Path of the Motive profile the API auto-loaded (provenance).
    pub profile: String,
    /// Path of the calibration in force (provenance).
    pub calibration: String,
}

impl Hello {
    /// Serializes to a complete record.
    ///
    /// # Errors
    ///
    /// [`Error::LengthOverflow`] if [`Hello::bridge_id`], [`Hello::profile`],
    /// or [`Hello::calibration`] exceeds the `u16` length limit (65 535 bytes).
    pub fn to_record(&self) -> Result<Vec<u8>> {
        let mut h = Writer::new();
        h.u16(self.proto_major);
        h.u16(self.proto_minor);
        h.u64(self.clock_hz);
        h.u64(self.qpf);
        h.u16(self.natnet_major);
        h.u16(self.natnet_minor);
        h.u32(self.motive_build);
        h.u64(self.host_wall_unix_secs);
        h.u8(self.mode);
        h.u32(self.camera_count);

        let mut p = Writer::new();
        write_lp_str(&mut p, &self.bridge_id, "Hello::bridge_id")?;
        write_lp_str(&mut p, &self.profile, "Hello::profile")?;
        write_lp_str(&mut p, &self.calibration, "Hello::calibration")?;

        encode_record(KIND_HELLO, &h.into_vec(), &p.into_vec())
    }

    fn parse(header: &[u8], payload: &[u8]) -> Result<Self> {
        let mut r = Reader::new(header);
        let proto_major = r.u16()?;
        let proto_minor = r.u16()?;
        let clock_hz = r.u64()?;
        let qpf = r.u64()?;
        let natnet_major = r.u16()?;
        let natnet_minor = r.u16()?;
        let motive_build = r.u32()?;
        let host_wall_unix_secs = r.u64()?;
        let mode = r.u8()?;
        let camera_count = r.u32()?;

        let mut p = Reader::new(payload);
        let bridge_id = read_lp_str(&mut p)?;
        let profile = read_lp_str(&mut p)?;
        let calibration = read_lp_str(&mut p)?;
        p.finish()?;

        Ok(Self {
            proto_major,
            proto_minor,
            clock_hz,
            qpf,
            natnet_major,
            natnet_minor,
            motive_build,
            host_wall_unix_secs,
            mode,
            camera_count,
            bridge_id,
            profile,
            calibration,
        })
    }
}

/// One camera's static description (kind [`KIND_CAMERA_DESC`]), emitted once
/// per camera after [`Hello`]. Payload: `ray_grid_w * ray_grid_h` samples of six little-endian
/// world-coordinate `f32`s (start x/y/z, end x/y/z), then a NUL-terminated
/// name. Rays are borrowed as bytes; use [`CameraDesc::ray`] to decode one.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraDesc<'a> {
    /// Motive camera index (0-based).
    pub camera_index: u32,
    /// Motive camera id.
    pub camera_id: u32,
    /// Camera serial number — the only stable identity across sessions.
    pub serial: u32,
    /// Motive `eVideoType` in force: `Segment=0, Grayscale=1, Object=2,
    /// Precision=4, MJPEG=6, ColorH264=9`.
    pub video_mode: u8,
    /// Encoding of the [`ImageFrame`]s this camera will send.
    /// [`ImageEncoding::Other`]`(`[`ENCODING_NONE`]`)` means "none".
    pub encoding: ImageEncoding,
    /// Whether this camera contributes to the Motive solve.
    pub contributes_to_solve: bool,
    /// Imaging frame-rate divisor (1 = every frame).
    pub frame_rate_divisor: u8,
    /// Image width in pixels.
    pub width: u16,
    /// Image height in pixels.
    pub height: u16,
    /// Camera position in world coordinates.
    pub position: [f32; 3],
    /// Camera orientation, a row-major 3x3 matrix.
    pub orientation: [f32; 9],
    /// Ray-grid columns.
    pub ray_grid_w: u16,
    /// Ray-grid rows.
    pub ray_grid_h: u16,
    /// Raw ray-grid bytes, `ray_grid_w * ray_grid_h * 24` of them.
    pub rays: &'a [u8],
    /// Camera name, borrowed from the payload. Truncated at the first invalid
    /// UTF-8 byte (the borrowed analogue of this crate's lossy decoding).
    pub name: &'a str,
}

/// Bytes per ray sample: six little-endian `f32`.
const RAY_BYTES: usize = 24;

/// Bytes of [`CameraDesc::rays`] a `w * h` grid declares.
///
/// Computes the grid's byte length. `available` is included in overflow
/// errors; overflow is possible on 32-bit targets.
fn ray_grid_bytes(ray_grid_w: u16, ray_grid_h: u16, available: usize) -> Result<usize> {
    usize::from(ray_grid_w)
        .checked_mul(usize::from(ray_grid_h))
        .and_then(|cells| cells.checked_mul(RAY_BYTES))
        .ok_or(Error::ImplausibleCount {
            count: usize::from(ray_grid_w).saturating_mul(usize::from(ray_grid_h)),
            available,
        })
}

impl<'a> CameraDesc<'a> {
    /// Ray at column `x`, row `y` as `[start_x, start_y, start_z, end_x, end_y, end_z]` in world
    /// coordinates; [`None`] if out of grid or short.
    #[must_use]
    pub fn ray(&self, x: usize, y: usize) -> Option<[f32; 6]> {
        if x >= usize::from(self.ray_grid_w) || y >= usize::from(self.ray_grid_h) {
            return None;
        }
        let start = (y * usize::from(self.ray_grid_w) + x).checked_mul(RAY_BYTES)?;
        let chunk = self.rays.get(start..start.checked_add(RAY_BYTES)?)?;
        let mut r = Reader::new(chunk);
        let mut out = [0.0f32; 6];
        for slot in &mut out {
            *slot = r.f32().ok()?;
        }
        Some(out)
    }

    /// Serializes to a complete record.
    ///
    /// # Errors
    ///
    /// [`Error::SectionSizeMismatch`] if [`CameraDesc::rays`] is not exactly
    /// `ray_grid_w * ray_grid_h * 24` bytes (`declared` is grid size;
    /// `consumed` is slice length). Parsing uses that size to locate the name.
    ///
    /// [`Error::InteriorNul`] for an interior NUL in [`CameraDesc::name`] (`offset` is its byte
    /// index);
    /// [`Error::LengthOverflow`] if the ray-grid size overflows `usize` (32-bit only) or payload
    /// exceeds `u32`.
    pub fn to_record(&self) -> Result<Vec<u8>> {
        let declared = match ray_grid_bytes(self.ray_grid_w, self.ray_grid_h, self.rays.len()) {
            Ok(declared) => declared,
            Err(Error::ImplausibleCount { count, .. }) => {
                return Err(Error::LengthOverflow {
                    field: "CameraDesc::rays",
                    length: count.saturating_mul(RAY_BYTES),
                    max: usize::MAX,
                });
            }
            Err(error) => return Err(error),
        };
        if declared != self.rays.len() {
            return Err(Error::SectionSizeMismatch {
                declared,
                consumed: self.rays.len(),
            });
        }
        if let Some(offset) = self.name.bytes().position(|b| b == 0) {
            return Err(Error::InteriorNul {
                field: "CameraDesc::name",
                offset,
            });
        }

        let mut h = Writer::new();
        h.u32(self.camera_index);
        h.u32(self.camera_id);
        h.u32(self.serial);
        h.u8(self.video_mode);
        h.u8(self.encoding.to_wire());
        h.u8(u8::from(self.contributes_to_solve));
        h.u8(self.frame_rate_divisor);
        h.u16(self.width);
        h.u16(self.height);
        h.vec3(self.position);
        for value in self.orientation {
            h.f32(value);
        }
        h.u16(self.ray_grid_w);
        h.u16(self.ray_grid_h);

        let mut p = Writer::new();
        p.bytes(self.rays);
        p.cstr(self.name);

        encode_record(KIND_CAMERA_DESC, &h.into_vec(), &p.into_vec())
    }

    fn parse(header: &[u8], payload: &'a [u8]) -> Result<Self> {
        let mut r = Reader::new(header);
        let camera_index = r.u32()?;
        let camera_id = r.u32()?;
        let serial = r.u32()?;
        let video_mode = r.u8()?;
        let encoding = ImageEncoding::from_wire(r.u8()?);
        let contributes_to_solve = r.u8()? != 0;
        let frame_rate_divisor = r.u8()?;
        let width = r.u16()?;
        let height = r.u16()?;
        let position = r.vec3()?;
        let mut orientation = [0.0f32; 9];
        for slot in &mut orientation {
            *slot = r.f32()?;
        }
        let ray_grid_w = r.u16()?;
        let ray_grid_h = r.u16()?;

        // Check the declared grid against the payload before slicing. This
        // rejects oversized grids without allocation; the encoder uses the
        // same size calculation.
        let cells = ray_grid_bytes(ray_grid_w, ray_grid_h, payload.len())?;
        let rays = payload.get(..cells).ok_or(Error::UnexpectedEof {
            offset: 0,
            needed: cells,
            available: payload.len(),
        })?;
        let rest = payload.get(cells..).unwrap_or_default();
        let end = rest
            .iter()
            .position(|&b| b == 0)
            .ok_or(Error::UnterminatedString { offset: cells })?;
        if end + 1 != rest.len() {
            return Err(Error::TrailingBytes {
                remaining: rest.len() - end - 1,
            });
        }
        let name = utf8_prefix(rest.get(..end).unwrap_or_default());

        Ok(Self {
            camera_index,
            camera_id,
            serial,
            video_mode,
            encoding,
            contributes_to_solve,
            frame_rate_divisor,
            width,
            height,
            position,
            orientation,
            ray_grid_w,
            ray_grid_h,
            rays,
            name,
        })
    }
}

/// One camera image (kind [`KIND_IMAGE_FRAME`]).
#[derive(Debug, Clone, PartialEq)]
pub struct ImageFrame<'a> {
    /// Frame number, sharing bits with NatNet's signed frame number. Use
    /// [`Self::matches_natnet_frame`] or [`Self::natnet_frame_number`] to join
    /// across the sign wrap; `u32::try_from` on the NatNet value fails past 2^31.
    pub frame_number: u32,
    /// Motive camera index; matches a [`CameraDesc::camera_index`].
    pub camera_index: u32,
    /// Authoritative mid-exposure timestamp in [`Hello::clock_hz`] ticks,
    /// copied from the NatNet frame of the same number.
    ///
    /// [`None`] (wire flag clear) means no correlated timestamp was available,
    /// e.g. the bridge's NatNet subscription is down; it is not zero ticks.
    pub camera_mid_exposure: Option<u64>,
    /// Windows `QueryPerformanceCounter` reading around the buffer copy, in
    /// [`Hello::qpf`] ticks. Ordering and latency only, never correlation.
    pub grab_ticks: u64,
    /// The Motive API's own frame timestamp, in seconds.
    pub api_timestamp_secs: f64,
    /// Image width in pixels.
    pub width: u16,
    /// Image height in pixels.
    pub height: u16,
    /// How [`ImageFrame::payload`] is encoded. This crate decodes none of it.
    pub encoding: ImageEncoding,
    /// The image bytes, borrowed verbatim.
    pub payload: &'a [u8],
}

impl<'a> ImageFrame<'a> {
    /// Frame number in NatNet's signed type, comparable with
    /// [`FrameOfData::frame_number`](crate::FrameOfData::frame_number).
    /// Reinterprets the same 32 bits, including values across the 2^31 wrap.
    /// Use [`ImageFrame::matches_natnet_frame`] for a pairwise join.
    #[must_use]
    pub const fn natnet_frame_number(&self) -> i32 {
        self.frame_number as i32
    }

    /// Whether this image belongs to NatNet frame `natnet_frame_number`; pass
    /// [`FrameOfData::frame_number`](crate::FrameOfData::frame_number).
    #[must_use]
    pub const fn matches_natnet_frame(&self, natnet_frame_number: i32) -> bool {
        self.natnet_frame_number() == natnet_frame_number
    }

    /// The record's 12-byte prefix and 40-byte header, without the payload. Send it with the
    /// borrowed payload to avoid copying image bytes.
    ///
    /// # Errors
    ///
    /// [`Error::LengthOverflow`] if [`ImageFrame::payload`] exceeds the `u32`
    /// `payload_len` limit (4 GiB).
    pub fn to_record_head(&self) -> Result<Vec<u8>> {
        let mut h = Writer::new();
        h.u32(self.frame_number);
        h.u32(self.camera_index);
        h.u64(self.camera_mid_exposure.unwrap_or(0));
        h.u64(self.grab_ticks);
        h.f64(self.api_timestamp_secs);
        h.u16(self.width);
        h.u16(self.height);
        h.u8(self.encoding.to_wire());
        h.u8(if self.camera_mid_exposure.is_some() {
            FLAG_MID_EXPOSURE_VALID
        } else {
            0
        });
        h.u16(0); // reserved
        let header = h.into_vec();

        let (header_len, payload_len) = wire_lens(header.len(), self.payload.len())?;
        let mut out = Writer::new();
        write_prefix(&mut out, KIND_IMAGE_FRAME, header_len, payload_len);
        out.bytes(&header);
        Ok(out.into_vec())
    }

    /// Serializes to a complete record (head followed by the payload).
    ///
    /// # Errors
    ///
    /// As [`ImageFrame::to_record_head`].
    pub fn to_record(&self) -> Result<Vec<u8>> {
        let mut out = self.to_record_head()?;
        out.extend_from_slice(self.payload);
        Ok(out)
    }

    fn parse(header: &[u8], payload: &'a [u8]) -> Result<Self> {
        let mut r = Reader::new(header);
        let frame_number = r.u32()?;
        let camera_index = r.u32()?;
        let ticks = r.u64()?;
        let grab_ticks = r.u64()?;
        let api_timestamp_secs = r.f64()?;
        let width = r.u16()?;
        let height = r.u16()?;
        let encoding = ImageEncoding::from_wire(r.u8()?);
        let flags = r.u8()?;
        let _reserved = r.u16()?;

        Ok(Self {
            frame_number,
            camera_index,
            camera_mid_exposure: (flags & FLAG_MID_EXPOSURE_VALID != 0).then_some(ticks),
            grab_ticks,
            api_timestamp_secs,
            width,
            height,
            encoding,
            payload,
        })
    }
}

/// One decoded AUGI/1 record.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Record<'a> {
    /// Stream preamble.
    Hello(Hello),
    /// One camera's static description.
    CameraDesc(CameraDesc<'a>),
    /// One camera image.
    ImageFrame(ImageFrame<'a>),
    /// Keep-alive. Any header or payload bytes are ignored (forward
    /// compatibility).
    Heartbeat,
    /// Orderly end of stream. Any header or payload bytes are ignored.
    Bye,
    /// A record kind this crate does not know, preserved verbatim.
    Other {
        /// The kind from the record prefix.
        kind: u16,
        /// The header bytes.
        header: &'a [u8],
        /// The payload bytes.
        payload: &'a [u8],
    },
}

/// Total length of the record beginning at `prefix`, when enough bytes exist.
///
/// `Ok(None)` if fewer than [`PREFIX_LEN`] bytes are available; otherwise the full prefix, header,
/// and payload length.
///
/// # Errors
///
/// [`Error::InvalidMagic`] if the magic is not [`MAGIC`].
///
/// [`Error::ImplausibleCount`] if the total length overflows `usize` (32-bit
/// targets only).
pub fn record_len(prefix: &[u8]) -> Result<Option<usize>> {
    if prefix.len() < PREFIX_LEN {
        return Ok(None);
    }
    let mut r = Reader::new(prefix);
    let magic = r.u32()?;
    if magic != MAGIC {
        return Err(Error::InvalidMagic {
            found: magic,
            expected: MAGIC,
        });
    }
    let _kind = r.u16()?;
    let header_len = usize::from(r.u16()?);
    let payload_len = r.u32()? as usize;
    let total = PREFIX_LEN
        .checked_add(header_len)
        .and_then(|n| n.checked_add(payload_len))
        .ok_or(Error::ImplausibleCount {
            count: payload_len,
            available: prefix.len(),
        })?;
    Ok(Some(total))
}

/// Parses the record at the front of `bytes`.
///
/// Bytes beyond the record are ignored; use [`record_len`] to find the next.
///
/// Extra trailing header bytes are skipped. A header shorter than the known
/// layout fails.
///
/// # Errors
///
/// [`Error::InvalidMagic`] for bad magic; [`Error::UnexpectedEof`] for incomplete records, short
/// headers, or oversized ray grids; [`Error::UnterminatedString`] for an unterminated name;
/// [`Error::ImplausibleCount`] for arithmetic overflow.
pub fn parse_record(bytes: &[u8]) -> Result<Record<'_>> {
    let Some(total) = record_len(bytes)? else {
        return Err(Error::UnexpectedEof {
            offset: 0,
            needed: PREFIX_LEN,
            available: bytes.len(),
        });
    };
    if bytes.len() < total {
        return Err(Error::UnexpectedEof {
            offset: 0,
            needed: total,
            available: bytes.len(),
        });
    }

    let mut r = Reader::new(bytes);
    let _magic = r.u32()?;
    let kind = r.u16()?;
    let header_len = usize::from(r.u16()?);
    let payload_len = r.u32()? as usize;
    let header = r.take(header_len)?;
    let payload = r.take(payload_len)?;

    match kind {
        KIND_HELLO => Ok(Record::Hello(Hello::parse(header, payload)?)),
        KIND_CAMERA_DESC => Ok(Record::CameraDesc(CameraDesc::parse(header, payload)?)),
        KIND_IMAGE_FRAME => Ok(Record::ImageFrame(ImageFrame::parse(header, payload)?)),
        KIND_HEARTBEAT => Ok(Record::Heartbeat),
        KIND_BYE => Ok(Record::Bye),
        kind => Ok(Record::Other {
            kind,
            header,
            payload,
        }),
    }
}

/// Encodes a record from its kind, header, and payload.
///
/// Public so callers can re-emit unknown [`Record::Other`] values.
///
/// # Errors
///
/// [`Error::LengthOverflow`] if the header exceeds 65 535 bytes or payload
/// exceeds 4 GiB.
pub fn encode_record(kind: u16, header: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
    let (header_len, payload_len) = wire_lens(header.len(), payload.len())?;
    let mut w = Writer::new();
    write_prefix(&mut w, kind, header_len, payload_len);
    w.bytes(header);
    w.bytes(payload);
    Ok(w.into_vec())
}

/// A complete heartbeat record (kind [`KIND_HEARTBEAT`], empty).
#[must_use]
pub fn heartbeat_record() -> Vec<u8> {
    empty_record(KIND_HEARTBEAT)
}

/// A complete bye record (kind [`KIND_BYE`], empty).
#[must_use]
pub fn bye_record() -> Vec<u8> {
    empty_record(KIND_BYE)
}

/// A bare prefix with both lengths zero — nothing to narrow, so unlike
/// [`encode_record`] this cannot fail and needs no [`Result`].
fn empty_record(kind: u16) -> Vec<u8> {
    let mut w = Writer::new();
    write_prefix(&mut w, kind, 0, 0);
    w.into_vec()
}

/// Narrows a header and payload length to the widths their wire fields have,
/// or fails rather than emit a record that declares less than it occupies.
fn wire_lens(header_len: usize, payload_len: usize) -> Result<(u16, u32)> {
    let header = u16::try_from(header_len).map_err(|_| Error::LengthOverflow {
        field: "header",
        length: header_len,
        max: usize::from(u16::MAX),
    })?;
    let payload = u32::try_from(payload_len).map_err(|_| Error::LengthOverflow {
        field: "payload",
        length: payload_len,
        max: usize::try_from(u32::MAX).unwrap_or(usize::MAX),
    })?;
    Ok((header, payload))
}

/// Writes the 12-byte record prefix from already-narrowed wire lengths.
fn write_prefix(w: &mut Writer, kind: u16, header_len: u16, payload_len: u32) {
    w.u32(MAGIC);
    w.u16(kind);
    w.u16(header_len);
    w.u32(payload_len);
}

/// Writes a `u16` length-prefixed UTF-8 string.
fn write_lp_str(w: &mut Writer, s: &str, field: &'static str) -> Result<()> {
    let len = u16::try_from(s.len()).map_err(|_| Error::LengthOverflow {
        field,
        length: s.len(),
        max: usize::from(u16::MAX),
    })?;
    w.u16(len);
    w.bytes(s.as_bytes());
    Ok(())
}

/// Reads a `u16` length-prefixed UTF-8 string, lossily.
fn read_lp_str(r: &mut Reader<'_>) -> Result<String> {
    let len = usize::from(r.u16()?);
    Ok(String::from_utf8_lossy(r.take(len)?).into_owned())
}

/// The longest valid UTF-8 prefix of `bytes`, borrowed.
///
/// Returns the longest valid prefix; borrowed `&str` cannot insert lossy-decoding replacement
/// characters.
fn utf8_prefix(bytes: &[u8]) -> &str {
    match core::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => bytes
            .get(..e.valid_up_to())
            .and_then(|b| core::str::from_utf8(b).ok())
            .unwrap_or(""),
    }
}
