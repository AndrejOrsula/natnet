//! **AUGI/1** — the imagery side-channel (`imagery` feature, default off).
//!
//! NatNet carries no imagery. `motive-bridge` (a sibling repo) replaces the
//! Motive GUI on the Windows host, pulls camera frame buffers through the
//! Motive API, and re-serves NatNet unchanged via `StreamNP(true)`. The
//! imagery it pulls travels beside NatNet on a separate TCP stream in the
//! record format decoded here.
//!
//! # Design
//!
//! Same rules as the rest of the crate: **sans-io** (`&[u8]` in, typed
//! values out), **zero-dependency**, **total decoding** (truncated or
//! malformed input returns [`Err`], never a panic; `#![forbid(unsafe_code)]`
//! is crate-wide), and **round-trip**
//! (`parse_record(&x.to_record()?)? == x` for every `x` a serializer accepts,
//! modulo IEEE-754 float equality and the one documented alias — the two
//! exceptions in [Encoding refuses what it cannot
//! round-trip](#encoding-refuses-what-it-cannot-round-trip)).
//!
//! Three further rules are specific to imagery:
//!
//! - **This module decodes no codec.** An [`ImageFrame`] hands out its bytes
//!   as `&[u8]` plus an [`ImageEncoding`] tag. Turning JPEG into pixels is the
//!   consumer's job and pulls in the consumer's decoder — not this crate's.
//! - **Encoding is fallible on purpose.** Every serializer that frames a
//!   header or payload returns [`Result`], and for each condition in the
//!   enumerated table below it *refuses* rather than render a record that
//!   parses back as a different value or desynchronises the stream. That
//!   table is the whole guarantee — it is a closed list, and its last two
//!   rows record what is **not** refused. See [Encoding refuses what it
//!   cannot round-trip](#encoding-refuses-what-it-cannot-round-trip).
//! - **Errors retain wire context.** Encoding faults use
//!   [`Error::InteriorNul`] and [`Error::LengthOverflow`] with the logical
//!   field that could not be represented. A bad magic is reported as
//!   [`Error::InvalidMagic`] with all four bytes retained. Decode-side count
//!   guards continue to use [`Error::ImplausibleCount`], and a missing
//!   terminator continues to use [`Error::UnterminatedString`].
//!
//! # Record framing
//!
//! Little-endian throughout, matching NatNet. Every record is
//!
//! | offset | field | notes |
//! |---|---|---|
//! | 0 | `u32 magic` | [`MAGIC`] — `"AUGI"` in wire order |
//! | 4 | `u16 kind` | [`KIND_HELLO`] … [`KIND_BYE`] |
//! | 6 | `u16 header_len` | the versioning mechanism, see below |
//! | 8 | `u32 payload_len` | bytes after the header |
//! | 12 | `header[header_len]` | fixed-shape fields |
//! | 12 + `header_len` | `payload[payload_len]` | variable-length bytes |
//!
//! `header_len` is how the format versions. A reader knows the header length
//! it was written against ([`HELLO_HEADER_LEN`] etc.) and **skips
//! `header_len - known_len` trailing bytes**, so a minor version may append
//! header fields without breaking older readers. A `header_len` *smaller*
//! than the known length is a truncation and fails with
//! [`Error::UnexpectedEof`] — appending is safe, removing is not.
//!
//! # Encoding refuses what it cannot round-trip
//!
//! The serializers that frame a header or payload are fallible, and each
//! rejection below — every row but the last two — used to be an `Ok` that
//! produced a record parsing back as a **different value**, or one that
//! **desynchronised** the stream. All of them are reachable from a
//! hand-constructed value, which is exactly what a Rust sender builds (the
//! ray-grid overflow additionally needs a 32-bit target); so are the last two
//! rows, which is why they are listed here rather than left implicit.
//!
//! | serializer | refused when | error |
//! |---|---|---|
//! | [`CameraDesc::to_record`] | `rays.len() != ray_grid_w * ray_grid_h * 24` | [`Error::SectionSizeMismatch`] |
//! | [`CameraDesc::to_record`] | [`name`](CameraDesc::name) holds an interior NUL | [`Error::InteriorNul`] |
//! | [`CameraDesc::to_record`] | the declared ray grid overflows `usize` (32-bit targets only) | [`Error::LengthOverflow`] |
//! | [`Hello::to_record`] | a payload string exceeds its `u16` length prefix | [`Error::LengthOverflow`] |
//! | [`encode_record`], [`ImageFrame::to_record_head`] | header > 65 535 B or payload > 4 GiB | [`Error::LengthOverflow`] |
//! | [`CameraDesc::to_record`], [`ImageFrame::to_record_head`] | **not refused:** `encoding` is [`Other(1)`](ImageEncoding::Other) or `Other(2)`; the wire canonicalizes it to `Mono8` or `Jpeg` | none — [`Ok`], see below |
//! | [`CameraDesc::to_record`], [`ImageFrame::to_record_head`] | **not refused:** a float field holds a NaN | none — [`Ok`], see below |
//!
//! The declared ray grid is the one the parser slices by, so a `rays` slice
//! of any other length silently ate into (or ran past) the camera name.
//! A length field narrower than the bytes actually appended left a reader
//! that advanced by the declared length standing inside the record.
//!
//! The framer row is where those two length limits are checked, and it covers
//! every typed serializer: [`Hello::to_record`] and [`CameraDesc::to_record`]
//! reach it by delegating to [`encode_record`], which is why the `u32`
//! `payload_len` ceiling that [`CameraDesc::to_record`] documents in its own
//! `# Errors` section has no row of its own. [`ImageFrame::to_record_head`] is
//! named beside [`encode_record`] because it writes the prefix itself, so it
//! checks both lengths itself.
//!
//! The last two rows are the exceptions to the round-trip rule, and both are
//! left open deliberately.
//!
//! The alias row first. [`ImageEncoding::Other`] aliases the two named wire
//! values — `Other(1).to_wire() == Mono8.to_wire()` and `Other(2).to_wire()
//! == Jpeg.to_wire()` — so a hand-built [`CameraDesc`] or [`ImageFrame`]
//! carrying either serializes to [`Ok`] and parses back as `Mono8` or
//! `Jpeg`: the same bytes, an unequal value. Every other `Other(n)`
//! round-trips, and nothing that came off the wire can reach the alias,
//! because [`ImageEncoding::from_wire`] never yields `Other(1)` or
//! `Other(2)`. Closing it would mean rejecting an encoding tag at
//! serialization time or making the alias unconstructible; both are
//! behaviour changes rather than documentation, so both are deferred (see
//! the `TODO` above [`ImageEncoding`] in the source).
//!
//! The NaN row is the wider of the two, in exactly the respect the sentence
//! above leans on: **a NaN can come off the wire.** Thirteen slots in AUGI/1
//! are float-typed and so enter the derived [`PartialEq`] —
//! [`CameraDesc::position`] (three `f32`), [`CameraDesc::orientation`] (nine
//! `f32`) and [`ImageFrame::api_timestamp_secs`] (one `f64`). No other field
//! of any AUGI/1 type holds a float, which is why [`Hello`] can derive [`Eq`]
//! and those two structs cannot. A NaN in any of the thirteen serializes to
//! [`Ok`] and survives the wire **bit for bit** — `from_le_bytes` reproduces
//! the pattern `to_le_bytes` wrote — and yet
//! `parse_record(&x.to_record()?)? == x` is `false`, because `NaN != NaN`:
//! the same bytes, an unequal value, and a value that is not even equal to
//! itself. Unlike the alias there is no escape through the parser: all
//! 16 777 214 `f32` bit patterns with a saturated exponent and a non-zero
//! mantissa decode to a NaN, so a peer's record can carry one, and the value
//! it parses back to fails the comparison exactly as a hand-built one does.
//! This is IEEE-754 semantics inherited through the derive, not a framing
//! defect: the byte-level round-trip is exact, and closing the value-level one
//! would mean comparing floats by their bits instead of with `==` — again a
//! behaviour change rather than documentation.
//!
//! Two neighbours of that row are **not** affected. `-0.0` keeps its own bit
//! pattern across the wire *and* `-0.0 == 0.0` holds, so it round-trips. And a
//! NaN among the ray samples in [`CameraDesc::rays`] is compared as the
//! `&[u8]` that field is, byte for byte, so it too round-trips; only the
//! float-*typed* slots named above are affected.
//!
//! [`heartbeat_record`] and [`bye_record`] frame no header and no payload —
//! each writes the 12-byte record prefix alone, with `header_len` and
//! `payload_len` both zero — so the rule above does not reach them and they
//! stay infallible.
//!
//! # Joining an image to its NatNet frame
//!
//! The whole side-channel rests on one join: [`ImageFrame::frame_number`]
//! against [`FrameOfData::frame_number`](crate::FrameOfData::frame_number).
//! The two fields hold **the same bits in different Rust types** — `u32` here,
//! `i32` there, each matching what its protocol writes — and Motive's counter
//! wraps at 2^31, so past that point one reads negative while the other reads
//! large and positive.
//!
//! ```text
//! if image.matches_natnet_frame(fod.frame_number) { /* same frame */ }
//! ```
//!
//! [`ImageFrame::matches_natnet_frame`] and
//! [`ImageFrame::natnet_frame_number`] reinterpret the bits and are correct
//! across the wrap. `u32::try_from(fod.frame_number)` is the trap: it looks
//! like the safe conversion and silently drops every frame past 2^31
//! (about 207 days at 120 Hz).
//!
//! # Timestamps, and the two clocks that are not the wall clock
//!
//! [`ImageFrame::camera_mid_exposure`] is the authoritative correlation key:
//! the bridge subscribes to its *own* re-served NatNet stream and copies
//! `FrameTiming::camera_mid_exposure` for frame *N* into the image record for
//! frame *N*, in [`Hello::clock_hz`] ticks (10 MHz on a typical rig). It is
//! [`Option`] on purpose: when the bridge's subscription is down the wire
//! flag is clear and this field is [`None`]. **"Absent" and "zero ticks" are
//! never the same artifact.**
//!
//! [`ImageFrame::grab_ticks`] is a `QueryPerformanceCounter` reading on the
//! Windows host ([`Hello::qpf`] ticks/second) taken around the buffer copy —
//! useful for ordering and latency, useless for correlation.
//!
//! [`Hello::host_wall_unix_secs`] is **provenance only, never a key**: a
//! Motive host's wall clock may be deliberately wrong.

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
/// [`Hello::mode`]: the bridge dialled the consumer (reverse mode — an
/// outbound connection needs no inbound firewall rule on the Windows host).
pub const MODE_CONNECT: u8 = 1;

/// [`ImageFrame`] header flag bit 0: `camera_mid_exposure_ticks` is valid.
pub const FLAG_MID_EXPOSURE_VALID: u8 = 0x01;

/// Wire value for "this camera sends no imagery".
pub const ENCODING_NONE: u8 = 0;

/// How the bytes in an [`ImageFrame::payload`] are encoded.
///
/// This crate decodes none of them — the tag exists so a consumer can pick a
/// decoder. `Other` preserves any wire value this crate does not name,
/// including [`ENCODING_NONE`].
///
/// The mapping is *not* injective in the `Other` direction:
/// `Other(1).to_wire() == Mono8.to_wire()` while `Other(1) != Mono8`.
/// [`ImageEncoding::from_wire`] never produces `Other(1)` or `Other(2)`, so a
/// value that came off the wire always compares correctly. A hand-built one
/// can hold the alias, and the serializers intentionally accept it: the wire
/// byte is canonicalized back to `Mono8` or `Jpeg`. This is one of the two
/// exceptions to this module's round-trip rule (the other is a NaN in a float
/// field, which *can* come off the wire), both listed in its refusal table.
// TODO: the alias is documented, not closed. Closing it needs one of two
// behaviour changes, both out of scope for a documentation pass and both
// source-breaking for callers that construct or match `Other`: make
// `to_wire` (and every serializer above it) fallible so `Other(1)`/`Other(2)`
// is rejected, or make the alias unconstructible with a `#[non_exhaustive]`
// variant or a private inner type. Revisit when the AUGI wire version moves;
// neither is worth a breaking change on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ImageEncoding {
    /// One byte per pixel, row-major, `width * height` bytes, no padding.
    Mono8,
    /// A complete JPEG/MJPEG frame (Motive's MJPEG reference video is
    /// 640x512 at roughly 23 kB per frame).
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

/// The stream preamble (kind [`KIND_HELLO`]) — everything a consumer needs to
/// interpret the ticks and versions in the records that follow.
///
/// The three strings live in the **payload** as `u16` length-prefixed UTF-8
/// (not in the header: `header_len` versioning requires a fixed-shape header).
/// Invalid UTF-8 is replaced lossily, as everywhere else in this crate.
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
    /// [`Error::LengthOverflow`] if [`Hello::bridge_id`], [`Hello::profile`]
    /// or [`Hello::calibration`] is longer than the `u16` length prefix that
    /// must carry it (65 535 bytes). Truncating instead would parse back as a
    /// different string, and could cut a UTF-8 sequence in half.
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
/// after [`Hello`] for every camera in the system.
///
/// The payload is `ray_grid_w * ray_grid_h` ray samples of six `f32`
/// (start x/y/z then end x/y/z, world coordinates, little-endian) followed by
/// the NUL-terminated camera name. The rays are handed out as raw bytes:
/// borrowing them costs nothing and imposes no alignment on the caller's
/// buffer. Use [`CameraDesc::ray`] to decode one.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraDesc<'a> {
    /// Motive camera index (0-based, the argument every API call takes).
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
/// The single place the grid-to-bytes arithmetic happens, so the encoder and
/// the decoder cannot disagree about it. `available` only decorates the
/// overflow error (reachable on a 32-bit target, where a 65535x65535 grid
/// exceeds `usize`).
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
    /// The ray sample at grid column `x`, row `y`, as
    /// `[start_x, start_y, start_z, end_x, end_y, end_z]` in world
    /// coordinates, or [`None`] if the index is outside the grid or the
    /// borrowed [`CameraDesc::rays`] slice is short.
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
    /// `ray_grid_w * ray_grid_h * 24` bytes (`declared` is what the grid
    /// implies, `consumed` what `rays` holds). The parser slices the rays by
    /// the **declared grid**, not by the payload, so any other length would
    /// eat into — or run past — the camera name that follows them and parse
    /// back as a different [`CameraDesc`].
    ///
    /// [`Error::InteriorNul`] if [`CameraDesc::name`] holds an interior
    /// NUL: the name is NUL-terminated on the wire, so a parser stops at the
    /// first one and returns a shorter name. `offset` is the byte's index
    /// within the name.
    ///
    /// [`Error::LengthOverflow`] if the ray grid overflows `usize` (32-bit
    /// targets only), or if the payload exceeds the `u32` `payload_len` field.
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

        // Guard the declared grid against the payload before slicing: a
        // hostile 65535x65535 grid must fail, not allocate. The encoder
        // computes `cells` with the same helper and refuses to write a `rays`
        // slice of any other length.
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
    /// Motive frame number this image belongs to — the join key against a
    /// NatNet [`FrameOfData`](crate::FrameOfData).
    ///
    /// **The two sides of the join carry the same bits in different Rust
    /// types.** This field is `u32`; NatNet's
    /// [`FrameOfData::frame_number`](crate::FrameOfData::frame_number) is
    /// `i32`. Neither is wrong — each matches what its protocol writes — but
    /// Motive's counter is a wrapping 32-bit value, so once it passes 2^31
    /// (roughly 207 days at 120 Hz) the NatNet side reads *negative* while
    /// this one reads large and positive. The bit patterns still agree; only
    /// the interpretation differs.
    ///
    /// Join by reinterpreting the bits —
    /// [`ImageFrame::matches_natnet_frame`] or
    /// [`ImageFrame::natnet_frame_number`] — and never with
    /// `u32::try_from(fod.frame_number)`, which looks like the safe
    /// conversion and silently discards every frame past the wrap.
    pub frame_number: u32,
    /// Motive camera index; matches a [`CameraDesc::camera_index`].
    pub camera_index: u32,
    /// Authoritative mid-exposure timestamp in [`Hello::clock_hz`] ticks,
    /// copied from the NatNet frame of the same number.
    ///
    /// [`None`] means the bridge's own NatNet subscription was down when this
    /// image was emitted — **not** "zero ticks".
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
    /// This image's frame number in the type NatNet uses, ready to compare
    /// with [`FrameOfData::frame_number`](crate::FrameOfData::frame_number).
    ///
    /// The conversion reinterprets the same 32 bits and is never a range
    /// check: it is total, allocation-free, and stays correct across the wrap
    /// at 2^31 that `u32::try_from` would reject. Use it to key a map of
    /// NatNet frames; use [`ImageFrame::matches_natnet_frame`] for a
    /// pairwise join.
    #[must_use]
    pub const fn natnet_frame_number(&self) -> i32 {
        self.frame_number as i32
    }

    /// Whether this image belongs to the NatNet frame numbered
    /// `natnet_frame_number` — the AUGI/1-to-NatNet join predicate.
    ///
    /// Take the argument straight from
    /// [`FrameOfData::frame_number`](crate::FrameOfData::frame_number); see
    /// [`ImageFrame::frame_number`] for why the two types differ and why
    /// converting rather than reinterpreting loses frames.
    #[must_use]
    pub const fn matches_natnet_frame(&self, natnet_frame_number: i32) -> bool {
        self.natnet_frame_number() == natnet_frame_number
    }

    /// The record's 12-byte prefix and 40-byte header, **without** the
    /// payload.
    ///
    /// A sender uses this to put an image on the wire with two `write_all`
    /// calls and no copy of the (large) payload:
    /// `w.write_all(&f.to_record_head()?)?; w.write_all(f.payload)?;`.
    ///
    /// # Errors
    ///
    /// [`Error::LengthOverflow`] if [`ImageFrame::payload`] is longer than
    /// the `u32` `payload_len` field can express (4 GiB). Truncating the field
    /// while the sender still writes the whole payload would desynchronise the
    /// stream. Unreachable on a 32-bit target and two orders of magnitude
    /// above any camera frame.
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

/// Total length of the record `prefix` begins, if `prefix` is long enough to
/// say.
///
/// Returns `Ok(None)` when fewer than [`PREFIX_LEN`] bytes are available —
/// the caller should read more and ask again. Returns `Ok(Some(n))` with the
/// full record length (prefix + header + payload) otherwise.
///
/// # Errors
///
/// [`Error::InvalidMagic`] if the magic is not [`MAGIC`]. The full `u32`
/// values are retained so callers can identify the producer that sent the
/// record without reconstructing discarded bytes.
///
/// [`Error::ImplausibleCount`] if prefix + header + payload overflows `usize`
/// (reachable only on a 32-bit target).
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
/// Bytes beyond the record are **ignored**, so a stream buffer can be passed
/// directly: use [`record_len`] to find where the next record starts.
///
/// Forward compatibility: a header longer than the length this crate knows
/// has its extra trailing bytes skipped. A header *shorter* than the known
/// length is a truncation and fails.
///
/// # Errors
///
/// [`Error::InvalidMagic`] for a bad magic, [`Error::UnexpectedEof`]
/// if `bytes` does not hold the whole record or a header is short,
/// [`Error::UnterminatedString`] for a [`CameraDesc`] name with no NUL, and
/// [`Error::UnexpectedEof`] for a ray grid larger than its supplied payload,
/// or [`Error::ImplausibleCount`] if record-length or grid-size arithmetic
/// overflows `usize`.
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

/// Frames `header` and `payload` as a record of `kind`.
///
/// This is the generic framer the typed serializers use; it is public so an
/// unknown [`Record::Other`] can be re-emitted verbatim.
///
/// # Errors
///
/// [`Error::LengthOverflow`] if either length is too large for the wire
/// field that must carry it — `header_len` is a `u16` (65 535 bytes) and
/// `payload_len` a `u32` (4 GiB). The error names the offered field length and
/// the largest value that field can express.
///
/// The alternative is what this function used to do: narrow the length field
/// while still appending every byte. A 70 000-byte header then produced a
/// 70 016-byte record declaring a 65 551-byte one, leaving a reader that
/// advances by the declared length standing 4 465 bytes inside the record —
/// a silent, unrecoverable desynchronisation. No record this crate's typed
/// serializers write comes close to either limit.
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

/// Writes a `u16` length-prefixed UTF-8 string (payload strings in [`Hello`]).
///
/// Fails with [`Error::LengthOverflow`] rather than truncate: the old
/// truncation wrote a shorter string than it was given, which parsed back as
/// a different value and could split a UTF-8 sequence while doing it.
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
/// The rest of the crate decodes strings with `from_utf8_lossy`, which must
/// allocate to insert replacement characters. A borrowed `&str` cannot, so
/// the borrowed analogue is to stop at the first invalid byte.
fn utf8_prefix(bytes: &[u8]) -> &str {
    match core::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => bytes
            .get(..e.valid_up_to())
            .and_then(|b| core::str::from_utf8(b).ok())
            .unwrap_or(""),
    }
}
