//! FrameOfData (message id 7) — one streamed data frame.
//!
//! # Layout
//!
//! NatNet 3.0, 3.1 and 4.0 use count-only sections. NatNet 4.1 adds an
//! `int32 section_byte_size` after each count. Skeleton, asset, force-plate
//! and device sections use [`RawSection`]; typed decode and encode methods
//! are provided on [`FrameOfData`] and [`RawSection`]. Legacy layouts and
//! strict 4.1 parsing validate their nested grammar. Compatible 4.1 parsing
//! preserves raw bodies without validation. Validators and typed decoders
//! share an entry grammar.
//!
//! ```text
//! int32 frame_number
//! section marker_sets     : entry = cstr name, int32 nMarkers, nMarkers*[f32;3]
//! section other_markers   : entry = [f32;3]                       (legacy unlabelled)
//! section rigid_bodies    : entry = int32 id, [f32;3] pos, [f32;4] quat,
//!                                    f32 mean_marker_error, u16 params
//! section skeletons       : entry = int32 id, int32 nBodies, nBodies*rigid body
//! section assets          : 4.1 only; entry = int32 id, int32 nBodies,
//!                           nBodies*rigid body, int32 nMarkers, nMarkers*labeled marker
//! section labeled_markers : entry = int32 id, [f32;3] pos, f32 size,
//!                                    u16 params, f32 residual   (NatNet >= 3.0)
//! section force_plates    : entry = int32 id, int32 nChannels,
//!                           nChannels*(int32 nFrames, nFrames*f32)
//! section devices         : entry = same shape as force_plates
//! u32 timecode, u32 timecode_subframe
//! f64 timestamp, u64 camera_mid_exposure, u64 camera_data_received,
//! u64 transmit                                                   (>= 3.0)
//! u32 precision_secs, u32 precision_fractional_secs               (4.1 only)
//! u16 frame params
//! int32 end-of-data marker (0)
//! ```
//!
//! Marker-set, rigid-body and timing layouts (up to `transmit`) are confirmed
//! against Motive.
//! Other-marker, skeleton, asset, labeled-marker, force-plate, device and
//! precision-timestamp layouts are **spec-derived**, not server-confirmed.
//!
//! # Round-trip and float equality
//!
//! Checked same-version serialization preserves float bits. Derived
//! [`PartialEq`] uses IEEE-754 equality, so any NaN makes the frame unequal
//! to itself; every accepted frame has a float slot in
//! [`FrameTiming::timestamp`]. `-0.0` preserves its bits and equals `0.0`.
//! Raw sections retain bytes, while typed views use ordinary float equality.
//! Invalid UTF-8 marker-set names decode to U+FFFD and cannot round-trip.
//! Checked encoding rejects interior NULs, oversized counts and fields
//! absent from the layout; [`FrameOfData::to_payload_lossy`] skips checks.

use crate::bytes::{Reader, Writer, vec_for_count};
use crate::error::{Error, Result};
use crate::message::{self, NAT_FRAMEOFDATA};
use crate::version::{Layout, ParsePolicy, Version};

/// One rigid body in a data frame.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RigidBody {
    /// Streaming id (resolve to a name via [`ModelDef`](crate::ModelDef);
    /// never hard-coded).
    pub id: i32,
    /// Position `[x, y, z]` in the server's native (Motive) frame — plain
    /// array, convention-neutral. The Y-up→Z-up conversion is the
    /// consumer's job.
    pub position: [f32; 3],
    /// Orientation quaternion `[x, y, z, w]` in the server's native frame.
    pub orientation: [f32; 4],
    /// Mean marker error (metres) reported by the tracker.
    pub mean_marker_error: f32,
    /// Raw parameter bitfield; bit 0 is tracking-valid.
    pub params: u16,
}

impl RigidBody {
    /// Whether the tracker reported a valid pose this frame (params bit 0).
    #[must_use]
    pub const fn tracking_valid(&self) -> bool {
        self.params & 0x01 != 0
    }
}

/// A named set of reconstructed marker positions.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MarkerSet {
    /// Marker-set name.
    pub name: String,
    /// Marker positions in the server's native frame.
    pub markers: Vec<[f32; 3]>,
}

/// One labeled marker (NatNet >= 3.0), with a spec-derived, unconfirmed layout.
/// The same 26-byte entry carries an [`Asset`] marker (SDK 4.1.0.0).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LabeledMarker {
    /// Marker id (encodes model + member for set markers).
    pub id: i32,
    /// Position in the server's native frame.
    pub position: [f32; 3],
    /// Marker size (diameter, metres).
    pub size: f32,
    /// Parameter bitfield.
    pub params: u16,
    /// Residual (metres).
    pub residual: f32,
}

/// Splits a composite id into `(entity_id, member_id)` using its signed high
/// 16 bits and unsigned low 16 bits. The SDK documents this encoding for
/// decoded skeleton-bone and legacy asset-marker ids, but does not confirm
/// raw wire bone ids; that layout remains spec-derived. [`Skeleton`] retains
/// the wire id unchanged.
#[must_use]
pub const fn decode_id(composite_id: i32) -> (i32, i32) {
    (composite_id >> 16, composite_id & 0xffff)
}

/// Builds the inverse of [`decode_id`], combining signed entity and unsigned
/// member ids into high and low 16-bit fields.
///
/// # Errors
/// Returns [`Error::InvalidValue`] when `entity_id` is outside
/// `-32768..=32767` or `member_id` is outside `0..=65535`, because such a
/// pair cannot be represented without truncation.
pub const fn encode_id(entity_id: i32, member_id: i32) -> Result<i32> {
    if entity_id < i16::MIN as i32 || entity_id > i16::MAX as i32 {
        return Err(Error::InvalidValue {
            field: "composite entity id",
            reason: "entity id must fit in 16 signed bits",
        });
    }
    if member_id < 0 || member_id > 0xffff {
        return Err(Error::InvalidValue {
            field: "composite member id",
            reason: "member id must fit in 16 unsigned bits",
        });
    }
    Ok((entity_id << 16) | member_id)
}

/// One skeleton with rigid-body bones. Its spec-derived, unconfirmed layout
/// is `int32 id`, `int32 nRigidBodies`, then 38-byte rigid bodies. See
/// [`decode_id`] for bone id encoding.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Skeleton {
    /// Skeleton id (matches [`SkeletonDef::id`](crate::SkeletonDef::id)).
    pub id: i32,
    /// Bones, with their wire ids unchanged.
    pub rigid_bodies: Vec<RigidBody>,
}

/// One asset in a NatNet 4.1 frame. Its spec-derived, unconfirmed layout is
/// `int32 id`, rigid-body count and entries, then marker count and entries.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Asset {
    /// Asset id (matches [`AssetDef::id`](crate::AssetDef::id)).
    pub id: i32,
    /// Asset rigid bodies.
    pub rigid_bodies: Vec<RigidBody>,
    /// Asset markers.
    pub markers: Vec<LabeledMarker>,
}

/// One force plate with per-channel analog samples. Its spec-derived,
/// unconfirmed layout is `int32 id`, channel count, then each channel's frame
/// count and `f32` samples; the wire layout has no `params` field.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForcePlate {
    /// Force-plate id (matches [`ForcePlateDef::id`](crate::ForcePlateDef::id)).
    pub id: i32,
    /// Samples per channel, in channel order.
    pub channels: Vec<Vec<f32>>,
}

/// One peripheral device. Its spec-derived, unconfirmed layout matches
/// [`ForcePlate`]; the wire layout has no `params` field.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Device {
    /// Device id (matches [`DeviceDef::id`](crate::DeviceDef::id)).
    pub id: i32,
    /// Samples per channel, in channel order.
    pub channels: Vec<Vec<f32>>,
}

/// A section's wire body bytes and entry count. Typed decode and encode
/// methods are available for skeletons, assets, force plates and devices.
/// Decodable sections re-encode to the same count and payload bytes. In
/// compatible NatNet 4.1 parsing, arbitrary bodies are preserved; strict 4.1
/// and legacy count-only parsing validate their grammar first.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RawSection {
    /// The section's declared entry count.
    pub count: i32,
    /// The section body bytes (exactly the declared section size).
    pub payload: Vec<u8>,
}

impl RawSection {
    /// Encodes skeletons as a section body (layout on [`Skeleton`]).
    ///
    /// # Errors
    /// Returns [`Error::LengthOverflow`] when a count or the body size does
    /// not fit its `int32` wire field.
    pub fn from_skeletons(skeletons: &[Skeleton]) -> Result<Self> {
        encode_raw(
            skeletons,
            "skeleton count",
            "skeleton section size",
            |w, skeleton| {
                w.i32(skeleton.id);
                write_counted(
                    w,
                    &skeleton.rigid_bodies,
                    "skeleton rigid body count",
                    |w, body| {
                        write_rigid_body(w, body);
                        Ok(())
                    },
                )
            },
        )
    }

    /// Encodes assets as a section body (layout on [`Asset`]). Only the
    /// NatNet 4.1 layout carries assets; encoding a frame at an older version
    /// refuses a non-empty asset section.
    ///
    /// # Errors
    /// Returns [`Error::LengthOverflow`] when a count or the body size does
    /// not fit its `int32` wire field.
    pub fn from_assets(assets: &[Asset]) -> Result<Self> {
        encode_raw(assets, "asset count", "asset section size", |w, asset| {
            w.i32(asset.id);
            write_counted(
                w,
                &asset.rigid_bodies,
                "asset rigid body count",
                |w, body| {
                    write_rigid_body(w, body);
                    Ok(())
                },
            )?;
            write_counted(w, &asset.markers, "asset marker count", |w, marker| {
                write_labeled_marker(w, marker);
                Ok(())
            })
        })
    }

    /// Encodes force plates as a section body (layout on [`ForcePlate`]).
    ///
    /// # Errors
    /// Returns [`Error::LengthOverflow`] when a count or the body size does
    /// not fit its `int32` wire field.
    pub fn from_force_plates(force_plates: &[ForcePlate]) -> Result<Self> {
        encode_raw(
            force_plates,
            "force plate count",
            "force plate section size",
            |w, plate| {
                w.i32(plate.id);
                write_channels(
                    w,
                    &plate.channels,
                    "force plate channel count",
                    "force plate sample count",
                )
            },
        )
    }

    /// Encodes devices as a section body (layout on [`Device`]).
    ///
    /// # Errors
    /// Returns [`Error::LengthOverflow`] when a count or the body size does
    /// not fit its `int32` wire field.
    pub fn from_devices(devices: &[Device]) -> Result<Self> {
        encode_raw(
            devices,
            "device count",
            "device section size",
            |w, device| {
                w.i32(device.id);
                write_channels(
                    w,
                    &device.channels,
                    "device channel count",
                    "device sample count",
                )
            },
        )
    }
}

/// Frame timing block (all fields little-endian; camera-clock fields count
/// in the SERVERINFO `high_res_clock_frequency` ticks).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FrameTiming {
    /// SMPTE timecode.
    pub timecode: u32,
    /// SMPTE timecode sub-frame.
    pub timecode_subframe: u32,
    /// Software timestamp in seconds.
    pub timestamp: f64,
    /// Camera mid-exposure timestamp, in clock ticks.
    pub camera_mid_exposure: u64,
    /// Camera data-received timestamp, in clock ticks.
    pub camera_data_received: u64,
    /// Transmit timestamp, in clock ticks.
    pub transmit: u64,
    /// Precision-timestamp seconds (NatNet >= 4.1; spec-derived).
    pub precision_timestamp_secs: u32,
    /// Precision-timestamp fractional seconds (NatNet >= 4.1; spec-derived).
    pub precision_timestamp_fractional_secs: u32,
}

/// A decoded FrameOfData message.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FrameOfData {
    /// Wrapping frame number read as signed `i32`; reinterpret its bits as
    /// `u32` when joining with the imagery side-channel past `2^31`.
    pub frame_number: i32,
    /// Named marker sets.
    pub marker_sets: Vec<MarkerSet>,
    /// Legacy unlabelled ("other") markers.
    pub other_markers: Vec<[f32; 3]>,
    /// Rigid bodies.
    pub rigid_bodies: Vec<RigidBody>,
    /// Skeleton wire bytes; typed view via [`Self::decode_skeletons`] or
    /// construction via [`RawSection::from_skeletons`].
    pub skeletons: RawSection,
    /// Asset wire bytes (NatNet 4.1 only); typed view via [`Self::decode_assets`]
    /// or construction via [`RawSection::from_assets`].
    pub assets: RawSection,
    /// Labeled markers (spec-derived).
    pub labeled_markers: Vec<LabeledMarker>,
    /// Force-plate wire bytes; typed view via [`Self::decode_force_plates`]
    /// or construction via [`RawSection::from_force_plates`].
    pub force_plates: RawSection,
    /// Device wire bytes; typed view via [`Self::decode_devices`] or
    /// construction via [`RawSection::from_devices`].
    pub devices: RawSection,
    /// Timing block.
    pub timing: FrameTiming,
    /// Raw frame parameter bits. Per the SDK 4.1.0.0 header: bit 0
    /// recording, bit 1 model list changed, bit 2 Edit mode, bit 3 bitstream
    /// changed ([`Self::bitstream_changed`]). The SDK's `PacketClient.cpp`
    /// sample treats bit 2 as bitstream changed instead. Only `0x0000` has
    /// been observed from Motive, so test the bit you trust.
    pub params: u16,
}

/// Reads a signed typed count; negatives are [`Error::InvalidCount`].
fn typed_count(r: &mut Reader<'_>) -> Result<i32> {
    let offset = r.position();
    let count = r.i32()?;
    if count < 0 {
        return Err(Error::InvalidCount { count, offset });
    }
    Ok(count)
}

/// Guards a count before allocating or walking entries. The defensive
/// negative arm reports the current position because the count offset is
/// unavailable.
fn count_len(r: &Reader<'_>, count: i32, min_entry: usize) -> Result<usize> {
    let Ok(count) = usize::try_from(count) else {
        return Err(Error::InvalidCount {
            count,
            offset: r.position(),
        });
    };
    if count > r.remaining().checked_div(min_entry).unwrap_or(0) {
        return Err(Error::ImplausibleCount {
            count,
            available: r.remaining(),
        });
    }
    Ok(count)
}

/// Reads a sized header. Compatibility raw sections may retain negative
/// counts; typed counts and negative byte sizes are always rejected.
fn sized_header(r: &mut Reader<'_>, policy: ParsePolicy, raw: bool) -> Result<(i32, usize)> {
    let count_offset = r.position();
    let count = r.i32()?;
    if count < 0 && (!raw || matches!(policy, ParsePolicy::Strict)) {
        return Err(Error::InvalidCount {
            count,
            offset: count_offset,
        });
    }

    let size_offset = r.position();
    let raw_size = r.i32()?;
    let Ok(size) = usize::try_from(raw_size) else {
        return Err(Error::InvalidCount {
            count: raw_size,
            offset: size_offset,
        });
    };
    if size > r.remaining() {
        return Err(Error::UnexpectedEof {
            offset: r.position(),
            needed: size,
            available: r.remaining(),
        });
    }
    Ok((count, size))
}

fn sized_typed_section<T>(
    r: &mut Reader<'_>,
    min_entry: usize,
    policy: ParsePolicy,
    mut entry: impl FnMut(&mut Reader<'_>) -> Result<T>,
) -> Result<Vec<T>> {
    let (count, size) = sized_header(r, policy, false)?;
    let mut body = r.subreader(size)?;
    let count = count_len(&body, count, min_entry)?;
    let mut out = vec_for_count(count);
    for _ in 0..count {
        out.push(entry(&mut body)?);
    }
    let consumed = size - body.remaining();
    if body.remaining() != 0 {
        return Err(Error::SectionSizeMismatch {
            declared: size,
            consumed,
        });
    }
    Ok(out)
}

fn legacy_typed_section<T>(
    r: &mut Reader<'_>,
    min_entry: usize,
    mut entry: impl FnMut(&mut Reader<'_>) -> Result<T>,
) -> Result<Vec<T>> {
    let count = typed_count(r)?;
    let count = count_len(r, count, min_entry)?;
    let mut out = vec_for_count(count);
    for _ in 0..count {
        out.push(entry(r)?);
    }
    Ok(out)
}

#[derive(Clone, Copy)]
enum RawKind {
    Skeleton,
    Asset,
    ForcePlate,
    Device,
}

/// Wire size of one rigid body.
const RIGID_BODY_BYTES: usize = 38;
/// Wire size of one labeled marker.
const LABELED_MARKER_BYTES: usize = 26;
/// Minimum structural entry size: id and count.
const MIN_RAW_ENTRY_BYTES: usize = 8;

fn read_marker_set(r: &mut Reader<'_>) -> Result<MarkerSet> {
    let name = r.cstr()?;
    let n = typed_count(r)?;
    let n = count_len(r, n, 12)?;
    let mut markers = vec_for_count(n);
    for _ in 0..n {
        markers.push(r.vec3()?);
    }
    Ok(MarkerSet { name, markers })
}

fn read_rigid_body(r: &mut Reader<'_>) -> Result<RigidBody> {
    Ok(RigidBody {
        id: r.i32()?,
        position: r.vec3()?,
        orientation: r.quat()?,
        mean_marker_error: r.f32()?,
        params: r.u16()?,
    })
}

fn write_rigid_body(w: &mut Writer, body: &RigidBody) {
    w.i32(body.id);
    w.vec3(body.position);
    w.quat(body.orientation);
    w.f32(body.mean_marker_error);
    w.u16(body.params);
}

fn read_labeled_marker(r: &mut Reader<'_>) -> Result<LabeledMarker> {
    Ok(LabeledMarker {
        id: r.i32()?,
        position: r.vec3()?,
        size: r.f32()?,
        params: r.u16()?,
        residual: r.f32()?,
    })
}

fn write_labeled_marker(w: &mut Writer, marker: &LabeledMarker) {
    w.i32(marker.id);
    w.vec3(marker.position);
    w.f32(marker.size);
    w.u16(marker.params);
    w.f32(marker.residual);
}

fn read_entries<T>(
    r: &mut Reader<'_>,
    count: i32,
    min_entry: usize,
    mut entry: impl FnMut(&mut Reader<'_>) -> Result<T>,
) -> Result<Vec<T>> {
    let count = count_len(r, count, min_entry)?;
    let mut out = vec_for_count(count);
    for _ in 0..count {
        out.push(entry(r)?);
    }
    Ok(out)
}

fn read_counted<T>(
    r: &mut Reader<'_>,
    min_entry: usize,
    entry: impl FnMut(&mut Reader<'_>) -> Result<T>,
) -> Result<Vec<T>> {
    let count = typed_count(r)?;
    read_entries(r, count, min_entry, entry)
}

fn write_counted<T>(
    w: &mut Writer,
    items: &[T],
    count_field: &'static str,
    mut entry: impl FnMut(&mut Writer, &T) -> Result<()>,
) -> Result<()> {
    w.i32(crate::bytes::checked_i32(items.len(), count_field)?);
    for item in items {
        entry(w, item)?;
    }
    Ok(())
}

fn read_skeleton(r: &mut Reader<'_>) -> Result<Skeleton> {
    let id = r.i32()?;
    let rigid_bodies = read_counted(r, RIGID_BODY_BYTES, read_rigid_body)?;
    Ok(Skeleton { id, rigid_bodies })
}

fn read_asset(r: &mut Reader<'_>) -> Result<Asset> {
    let id = r.i32()?;
    let rigid_bodies = read_counted(r, RIGID_BODY_BYTES, read_rigid_body)?;
    let markers = read_counted(r, LABELED_MARKER_BYTES, read_labeled_marker)?;
    Ok(Asset {
        id,
        rigid_bodies,
        markers,
    })
}

/// Reads/writes channel data: channel count, then each frame count and floats.
fn read_channels(r: &mut Reader<'_>) -> Result<Vec<Vec<f32>>> {
    read_counted(r, 4, |r| read_counted(r, 4, |r| r.f32()))
}

fn write_channels(
    w: &mut Writer,
    channels: &[Vec<f32>],
    count_field: &'static str,
    sample_field: &'static str,
) -> Result<()> {
    write_counted(w, channels, count_field, |w, samples| {
        write_counted(w, samples, sample_field, |w, sample| {
            w.f32(*sample);
            Ok(())
        })
    })
}

fn read_force_plate(r: &mut Reader<'_>) -> Result<ForcePlate> {
    Ok(ForcePlate {
        id: r.i32()?,
        channels: read_channels(r)?,
    })
}

fn read_device(r: &mut Reader<'_>) -> Result<Device> {
    Ok(Device {
        id: r.i32()?,
        channels: read_channels(r)?,
    })
}

/// Validates a raw body with the typed decoders' grammar.
fn walk_raw_body(r: &mut Reader<'_>, count: i32, kind: RawKind) -> Result<()> {
    match kind {
        RawKind::Skeleton => {
            read_entries(r, count, MIN_RAW_ENTRY_BYTES, read_skeleton)?;
        }
        RawKind::Asset => {
            read_entries(r, count, MIN_RAW_ENTRY_BYTES, read_asset)?;
        }
        RawKind::ForcePlate => {
            read_entries(r, count, MIN_RAW_ENTRY_BYTES, read_force_plate)?;
        }
        RawKind::Device => {
            read_entries(r, count, MIN_RAW_ENTRY_BYTES, read_device)?;
        }
    }
    Ok(())
}

fn decode_raw<T>(
    section: &RawSection,
    entry: impl FnMut(&mut Reader<'_>) -> Result<T>,
) -> Result<Vec<T>> {
    let mut body = Reader::new(&section.payload);
    let declared = body.remaining();
    let out = read_entries(&mut body, section.count, MIN_RAW_ENTRY_BYTES, entry)?;
    if body.remaining() != 0 {
        let consumed = declared - body.remaining();
        return Err(Error::SectionSizeMismatch { declared, consumed });
    }
    Ok(out)
}

fn encode_raw<T>(
    items: &[T],
    count_field: &'static str,
    size_field: &'static str,
    mut entry: impl FnMut(&mut Writer, &T) -> Result<()>,
) -> Result<RawSection> {
    let count = crate::bytes::checked_i32(items.len(), count_field)?;
    let mut w = Writer::new();
    for item in items {
        entry(&mut w, item)?;
    }
    let payload = w.into_vec();
    crate::bytes::checked_i32(payload.len(), size_field)?;
    Ok(RawSection { count, payload })
}

fn validate_raw_body(body: &mut Reader<'_>, count: i32, kind: RawKind) -> Result<()> {
    let declared = body.remaining();
    walk_raw_body(body, count, kind)?;
    if body.remaining() != 0 {
        let consumed = declared - body.remaining();
        return Err(Error::SectionSizeMismatch { declared, consumed });
    }
    Ok(())
}

/// Reads a count-only section and preserves its body bytes.
fn legacy_raw_section(r: &mut Reader<'_>, kind: RawKind) -> Result<RawSection> {
    let count = typed_count(r)?;
    let body_start = r.position();
    walk_raw_body(r, count, kind)?;
    let body_end = r.position();
    Ok(RawSection {
        count,
        payload: r.span(body_start, body_end)?.to_vec(),
    })
}

/// Reads a sized section; compatibility keeps its body opaque, strict mode
/// validates its grammar.
fn sized_raw_section(r: &mut Reader<'_>, kind: RawKind, policy: ParsePolicy) -> Result<RawSection> {
    let (count, size) = sized_header(r, policy, true)?;
    let body_start = r.position();
    let mut body = r.subreader(size)?;
    if matches!(policy, ParsePolicy::Strict) {
        validate_raw_body(&mut body, count, kind)?;
    }
    let body_end = r.position();
    Ok(RawSection {
        count,
        payload: r.span(body_start, body_end)?.to_vec(),
    })
}

fn validate_legacy_raw(s: &RawSection, kind: RawKind) -> Result<()> {
    if s.count < 0 {
        return Err(Error::InvalidValue {
            field: "raw section count",
            reason: "count must be non-negative",
        });
    }
    let mut body = Reader::new(&s.payload);
    validate_raw_body(&mut body, s.count, kind)
}

/// Writes a typed section with checked count, size and entry encoding.
fn checked_typed_section<T>(
    w: &mut Writer,
    items: &[T],
    sized: bool,
    count_field: &'static str,
    size_field: &'static str,
    mut entry: impl FnMut(&mut Writer, &T) -> Result<()>,
) -> Result<()> {
    let count = crate::bytes::checked_i32(items.len(), count_field)?;
    w.i32(count);
    // Reserve and patch the 4.1 size after writing entries.
    let size_at = w.len();
    if sized {
        w.i32(0);
    }
    let body_start = w.len();
    for item in items {
        entry(w, item)?;
    }
    if sized {
        let size = crate::bytes::checked_i32(w.len() - body_start, size_field)?;
        w.patch_i32(size_at, size);
    }
    Ok(())
}

/// Writes typed entries without validation.
fn lossy_typed_section<T>(
    w: &mut Writer,
    items: &[T],
    sized: bool,
    mut entry: impl FnMut(&mut Writer, &T),
) {
    w.i32(i32::try_from(items.len()).unwrap_or(i32::MAX));
    let size_at = w.len();
    if sized {
        w.i32(0);
    }
    let body_start = w.len();
    for item in items {
        entry(w, item);
    }
    if sized {
        w.patch_i32(
            size_at,
            i32::try_from(w.len() - body_start).unwrap_or(i32::MAX),
        );
    }
}

fn checked_raw_section(
    w: &mut Writer,
    s: &RawSection,
    sized: bool,
    count_field: &'static str,
    size_field: &'static str,
) -> Result<()> {
    // `to_payload` validates the nested grammar before encoding.
    let payload_len = crate::bytes::checked_i32(s.payload.len(), size_field)?;
    if s.count < 0 {
        return Err(Error::InvalidValue {
            field: count_field,
            reason: "count must be non-negative",
        });
    }
    w.i32(s.count);
    if sized {
        w.i32(payload_len);
    }
    w.bytes(&s.payload);
    Ok(())
}

fn lossy_raw_section(w: &mut Writer, s: &RawSection, sized: bool) {
    w.i32(s.count);
    if sized {
        w.i32(i32::try_from(s.payload.len()).unwrap_or(i32::MAX));
    }
    w.bytes(&s.payload);
}

impl FrameOfData {
    /// Whether [`Self::params`] bit 3 (`0x08`), *bitstream version changed*,
    /// is set. This meaning follows the SDK 4.1.0.0 header; see [`Self::params`]
    /// for conflicting SDK documentation and the lack of an observed set bit.
    #[must_use]
    pub const fn bitstream_changed(&self) -> bool {
        self.params & 0x08 != 0
    }

    /// Decodes [`Self::skeletons`] into typed skeletons, requiring exactly
    /// the declared entries and no residue. Layout is the same for all
    /// accepted versions; see [`Skeleton`].
    ///
    /// # Errors
    /// [`Error::InvalidCount`] for negatives, [`Error::ImplausibleCount`] or
    /// [`Error::UnexpectedEof`] for truncation, and
    /// [`Error::SectionSizeMismatch`] for trailing bytes.
    pub fn decode_skeletons(&self) -> Result<Vec<Skeleton>> {
        decode_raw(&self.skeletons, read_skeleton)
    }

    /// Decodes [`Self::assets`] into typed assets. Before 4.1, the section is
    /// empty.
    ///
    /// # Errors
    /// See [`Self::decode_skeletons`].
    pub fn decode_assets(&self) -> Result<Vec<Asset>> {
        decode_raw(&self.assets, read_asset)
    }

    /// Decodes [`Self::force_plates`].
    ///
    /// # Errors
    /// See [`Self::decode_skeletons`].
    pub fn decode_force_plates(&self) -> Result<Vec<ForcePlate>> {
        decode_raw(&self.force_plates, read_force_plate)
    }

    /// Decodes [`Self::devices`].
    ///
    /// # Errors
    /// See [`Self::decode_skeletons`].
    pub fn decode_devices(&self) -> Result<Vec<Device>> {
        decode_raw(&self.devices, read_device)
    }

    /// Parses a FrameOfData **datagram**, including its 4-byte header, using
    /// the negotiated `version`.
    /// # Errors
    ///
    /// [`Error::UnexpectedMessageId`] for non-id-7, [`Error::UnsupportedVersion`]
    /// for unverified layouts, [`Error::UnexpectedEof`],
    /// [`Error::UnterminatedString`], [`Error::SectionSizeMismatch`],
    /// [`Error::ImplausibleCount`] or [`Error::InvalidCount`] for malformed
    /// frames. Compatibility preserves opaque 4.1 bodies and accepts a
    /// nonzero terminator or suffix; use [`Self::parse_with_policy`] with
    /// [`ParsePolicy::Strict`] for full validation.
    pub fn parse(datagram: &[u8], version: Version) -> Result<Self> {
        Self::parse_with_policy(datagram, version, ParsePolicy::Compatible)
    }

    /// Parses a FrameOfData datagram with an explicit policy.
    /// # Errors
    /// Returns [`Error::UnexpectedMessageId`] for non-frame data,
    /// [`Error::UnsupportedVersion`] for unverified layouts, or a field,
    /// count, section, terminator or trailing-byte error under the policy.
    pub fn parse_with_policy(
        datagram: &[u8],
        version: Version,
        policy: ParsePolicy,
    ) -> Result<Self> {
        // Reject unknown versions before interpreting payload bytes.
        version.layout()?;
        let payload = message::expect(datagram, NAT_FRAMEOFDATA)?;
        Self::parse_payload_with_policy(payload, version, policy)
    }

    /// Parses a FrameOfData payload (bytes **after** the 4-byte header).
    /// # Errors
    ///
    /// See [`FrameOfData::parse`].
    pub fn parse_payload(payload: &[u8], version: Version) -> Result<Self> {
        Self::parse_payload_with_policy(payload, version, ParsePolicy::Compatible)
    }

    /// Parses a FrameOfData payload with an explicit policy.
    /// # Errors
    /// Returns [`Error::UnsupportedVersion`] for unverified layouts, or a
    /// field, count, section, terminator or trailing-byte error under the policy.
    #[allow(
        clippy::too_many_lines,
        reason = "wire fields must remain in protocol order"
    )]
    pub fn parse_payload_with_policy(
        payload: &[u8],
        version: Version,
        policy: ParsePolicy,
    ) -> Result<Self> {
        let layout = version.layout()?;
        let mut r = Reader::new(payload);
        let frame_number = r.i32()?;

        let marker_sets = if layout.has_section_sizes() {
            sized_typed_section(&mut r, 5, policy, read_marker_set)?
        } else {
            legacy_typed_section(&mut r, 5, read_marker_set)?
        };

        let other_markers = if layout.has_section_sizes() {
            sized_typed_section(&mut r, 12, policy, |r| r.vec3())?
        } else {
            legacy_typed_section(&mut r, 12, |r| r.vec3())?
        };

        let rigid_bodies = if layout.has_section_sizes() {
            sized_typed_section(&mut r, RIGID_BODY_BYTES, policy, read_rigid_body)?
        } else {
            legacy_typed_section(&mut r, RIGID_BODY_BYTES, read_rigid_body)?
        };

        let skeletons = if layout.has_section_sizes() {
            sized_raw_section(&mut r, RawKind::Skeleton, policy)?
        } else {
            legacy_raw_section(&mut r, RawKind::Skeleton)?
        };
        let assets = if layout.has_assets() {
            sized_raw_section(&mut r, RawKind::Asset, policy)?
        } else {
            RawSection {
                count: 0,
                payload: Vec::new(),
            }
        };

        let labeled_markers = if layout.has_section_sizes() {
            sized_typed_section(&mut r, LABELED_MARKER_BYTES, policy, read_labeled_marker)?
        } else {
            legacy_typed_section(&mut r, LABELED_MARKER_BYTES, read_labeled_marker)?
        };

        let force_plates = if layout.has_section_sizes() {
            sized_raw_section(&mut r, RawKind::ForcePlate, policy)?
        } else {
            legacy_raw_section(&mut r, RawKind::ForcePlate)?
        };
        let devices = if layout.has_section_sizes() {
            sized_raw_section(&mut r, RawKind::Device, policy)?
        } else {
            legacy_raw_section(&mut r, RawKind::Device)?
        };

        let timecode = r.u32()?;
        let timecode_subframe = r.u32()?;
        let timestamp = r.f64()?;
        let camera_mid_exposure = r.u64()?;
        let camera_data_received = r.u64()?;
        let transmit = r.u64()?;
        let (precision_timestamp_secs, precision_timestamp_fractional_secs) =
            if layout.has_precision() {
                (r.u32()?, r.u32()?)
            } else {
                (0, 0)
            };
        let params = r.u16()?;
        let end_offset = r.position();
        let end_of_data = r.i32()?;
        if matches!(policy, ParsePolicy::Strict) && end_of_data != 0 {
            return Err(Error::InvalidTerminator {
                value: end_of_data,
                offset: end_offset,
            });
        }
        if matches!(policy, ParsePolicy::Strict) {
            r.finish()?;
        }

        Ok(Self {
            frame_number,
            marker_sets,
            other_markers,
            rigid_bodies,
            skeletons,
            assets,
            labeled_markers,
            force_plates,
            devices,
            timing: FrameTiming {
                timecode,
                timecode_subframe,
                timestamp,
                camera_mid_exposure,
                camera_data_received,
                transmit,
                precision_timestamp_secs,
                precision_timestamp_fractional_secs,
            },
            params,
        })
    }

    /// Encodes a checked FrameOfData payload (without the four-byte header).
    /// Counts, sizes and strings are checked; unsupported fields are rejected.
    ///
    /// # Errors
    /// Returns an encoding or unsupported-layout error for unrepresentable values.
    pub fn to_payload(&self, version: Version) -> Result<Vec<u8>> {
        let layout = version.layout()?;
        self.validate_for_layout(layout)?;
        self.encode_payload_checked(layout)
    }

    /// Encodes a payload without value validation.
    /// # Errors
    /// Returns [`Error::UnsupportedVersion`] for unsupported layouts.
    pub fn to_payload_lossy(&self, version: Version) -> Result<Vec<u8>> {
        Ok(self.encode_payload_lossy(version.layout()?))
    }

    /// Encodes a checked datagram (header plus payload).
    ///
    /// # Errors
    /// Same as [`Self::to_payload`], plus [`Error::LengthOverflow`] for a
    /// payload above the 65 535-byte `u16` limit.
    pub fn to_datagram(&self, version: Version) -> Result<Vec<u8>> {
        let payload = self.to_payload(version)?;
        message::frame_checked(NAT_FRAMEOFDATA, &payload)
    }

    /// Encodes without value validation. Payloads over 65 535 bytes are emitted
    /// in full with a saturated declared size.
    /// # Errors
    /// Returns [`Error::UnsupportedVersion`] for unsupported layouts.
    pub fn to_datagram_lossy(&self, version: Version) -> Result<Vec<u8>> {
        let payload = self.to_payload_lossy(version)?;
        Ok(message::frame(NAT_FRAMEOFDATA, &payload))
    }

    fn validate_for_layout(&self, layout: Layout) -> Result<()> {
        if !layout.has_assets() && (self.assets.count != 0 || !self.assets.payload.is_empty()) {
            return Err(Error::InvalidValue {
                field: "assets",
                reason: "assets are not present in this protocol layout",
            });
        }
        if !layout.has_precision()
            && (self.timing.precision_timestamp_secs != 0
                || self.timing.precision_timestamp_fractional_secs != 0)
        {
            return Err(Error::InvalidValue {
                field: "precision timestamp",
                reason: "precision timestamp is not present in this protocol layout",
            });
        }
        validate_legacy_raw(&self.skeletons, RawKind::Skeleton)?;
        validate_legacy_raw(&self.force_plates, RawKind::ForcePlate)?;
        validate_legacy_raw(&self.devices, RawKind::Device)?;
        if layout.has_assets() {
            validate_legacy_raw(&self.assets, RawKind::Asset)?;
        }
        Ok(())
    }

    #[allow(
        clippy::too_many_lines,
        reason = "wire fields must remain in protocol order"
    )]
    fn encode_payload_checked(&self, layout: Layout) -> Result<Vec<u8>> {
        let sized = layout.has_section_sizes();
        let mut w = Writer::new();
        w.i32(self.frame_number);

        checked_typed_section(
            &mut w,
            &self.marker_sets,
            sized,
            "marker set count",
            "marker set section size",
            |w, ms| {
                w.try_cstr(&ms.name, "marker set name")?;
                w.i32(crate::bytes::checked_i32(ms.markers.len(), "marker count")?);
                for marker in &ms.markers {
                    w.vec3(*marker);
                }
                Ok(())
            },
        )?;
        checked_typed_section(
            &mut w,
            &self.other_markers,
            sized,
            "other marker count",
            "other marker section size",
            |w, marker| {
                w.vec3(*marker);
                Ok(())
            },
        )?;
        checked_typed_section(
            &mut w,
            &self.rigid_bodies,
            sized,
            "rigid body count",
            "rigid body section size",
            |w, body| {
                write_rigid_body(w, body);
                Ok(())
            },
        )?;
        checked_raw_section(
            &mut w,
            &self.skeletons,
            sized,
            "skeleton count",
            "skeleton section size",
        )?;
        if layout.has_assets() {
            checked_raw_section(
                &mut w,
                &self.assets,
                sized,
                "asset count",
                "asset section size",
            )?;
        }
        checked_typed_section(
            &mut w,
            &self.labeled_markers,
            sized,
            "labeled marker count",
            "labeled marker section size",
            |w, marker| {
                write_labeled_marker(w, marker);
                Ok(())
            },
        )?;
        checked_raw_section(
            &mut w,
            &self.force_plates,
            sized,
            "force plate count",
            "force plate section size",
        )?;
        checked_raw_section(
            &mut w,
            &self.devices,
            sized,
            "device count",
            "device section size",
        )?;
        w.u32(self.timing.timecode);
        w.u32(self.timing.timecode_subframe);
        w.f64(self.timing.timestamp);
        w.u64(self.timing.camera_mid_exposure);
        w.u64(self.timing.camera_data_received);
        w.u64(self.timing.transmit);
        if layout.has_precision() {
            w.u32(self.timing.precision_timestamp_secs);
            w.u32(self.timing.precision_timestamp_fractional_secs);
        }
        w.u16(self.params);
        w.i32(0);
        Ok(w.into_vec())
    }

    fn encode_payload_lossy(&self, layout: Layout) -> Vec<u8> {
        let sized = layout.has_section_sizes();
        let mut w = Writer::new();
        w.i32(self.frame_number);
        lossy_typed_section(&mut w, &self.marker_sets, sized, |w, ms| {
            w.cstr(&ms.name);
            w.i32(i32::try_from(ms.markers.len()).unwrap_or(i32::MAX));
            for marker in &ms.markers {
                w.vec3(*marker);
            }
        });
        lossy_typed_section(&mut w, &self.other_markers, sized, |w, marker| {
            w.vec3(*marker);
        });
        lossy_typed_section(&mut w, &self.rigid_bodies, sized, write_rigid_body);
        lossy_raw_section(&mut w, &self.skeletons, sized);
        if layout.has_assets() {
            lossy_raw_section(&mut w, &self.assets, sized);
        }
        lossy_typed_section(&mut w, &self.labeled_markers, sized, write_labeled_marker);
        lossy_raw_section(&mut w, &self.force_plates, sized);
        lossy_raw_section(&mut w, &self.devices, sized);
        w.u32(self.timing.timecode);
        w.u32(self.timing.timecode_subframe);
        w.f64(self.timing.timestamp);
        w.u64(self.timing.camera_mid_exposure);
        w.u64(self.timing.camera_data_received);
        w.u64(self.timing.transmit);
        if layout.has_precision() {
            w.u32(self.timing.precision_timestamp_secs);
            w.u32(self.timing.precision_timestamp_fractional_secs);
        }
        w.u16(self.params);
        w.i32(0);
        w.into_vec()
    }
}
