//! FrameOfData (message id 7) — one streamed data frame.
//!
//! # Layout (checked against 25 real Motive NatNet 4.1.0.0 multicast
//! datagrams, constant 260-byte datagram / 256-byte payload; the captures are
//! not distributed with the crate)
//!
//! NatNet 3.0, 3.1 and 4.0 use count-only sections. NatNet 4.1 adds an
//! `int32 section_byte_size` after every count; the decoder bounds each sized
//! body before parsing it and checks exact consumption for typed sections.
//! Skeleton, asset, force-plate and device bodies are kept as [`RawSection`]
//! bytes so every body byte survives re-encoding. The legacy layouts and
//! strict 4.1 decoding validate their nested grammar; compatibility 4.1
//! decoding keeps them unvalidated. Typed views are explicit:
//! [`FrameOfData::decode_skeletons`], [`FrameOfData::decode_assets`],
//! [`FrameOfData::decode_force_plates`] and [`FrameOfData::decode_devices`]
//! decode a section, and [`RawSection::from_skeletons`],
//! [`RawSection::from_assets`], [`RawSection::from_force_plates`] and
//! [`RawSection::from_devices`] encode one. The validator and the typed
//! decoders share one entry grammar.
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
//! **Grounding.** `frame_number`, the two marker sets (`CW-500 wand`, `all`,
//! 3 markers each), the single rigid body (`id 3`, `mean_marker_error 0`,
//! `params & 1 == 0` ⇒ tracking-invalid) and the `timestamp` (= frame_number
//! / 120, matching the 120 f/s capture) are **capture-grounded**. The
//! `other_markers`, `skeletons`, `assets`, `labeled_markers`, `force_plates`
//! and `devices` sections are **empty** in those captures, so their entry
//! layouts — including the typed [`Skeleton`], [`Asset`], [`ForcePlate`] and
//! [`Device`] views — are **spec-derived** from the NatNet SDK 4.1.0.0
//! `PacketClient.cpp` unpackers and `NatNetTypes.h`, and not yet confirmed
//! against real Motive bytes; the same holds for the precision-timestamp
//! split (8 zero bytes).
//!
//! # Round-trip, and the NaN a frame can carry
//!
//! [`FrameOfData::to_payload`] and [`FrameOfData::parse_payload`] are inverses
//! **modulo IEEE-754 float equality**. For an `x` that `parse_payload`
//! produced at some `version`, checked re-serializing at that same `version`
//! parses back to a value equal to `x` — **unless one of `x`'s float-typed slots
//! holds a NaN**, and a frame off the wire can hold one in any of them.
//!
//! **That `unless` governs the value comparison and nothing else.** Whether
//! the payload *bytes* reproduce is a separate question with a separate
//! answer, and a NaN is not what decides it: a NaN costs value equality and
//! never byte equality (below the table), while a marker-set name whose wire
//! bytes are not UTF-8 costs byte equality with no NaN anywhere (the last
//! paragraph of this section).
//!
//! `parse_payload` builds every float with `f32::from_le_bytes` /
//! `f64::from_le_bytes` and validates none of them, so any wire bit pattern
//! with a saturated exponent and a non-zero mantissa decodes to a NaN. The
//! float-typed slots — the ones that enter the derived [`PartialEq`] — are:
//!
//! | slot | floats per entry | entries per frame |
//! |---|---|---|
//! | [`FrameTiming::timestamp`] | 1 × `f64` | **exactly 1, always** |
//! | [`MarkerSet::markers`] | 3 × `f32` per marker | 0..n markers, 0..n sets |
//! | [`FrameOfData::other_markers`] | 3 × `f32` | 0..n |
//! | [`RigidBody::position`], [`RigidBody::orientation`], [`RigidBody::mean_marker_error`] | 3 + 4 + 1 × `f32` | 0..n |
//! | [`LabeledMarker::position`], [`LabeledMarker::size`], [`LabeledMarker::residual`] | 3 + 1 + 1 × `f32` | 0..n |
//!
//! Unlike the imagery side-channel's closed list of thirteen slots (the
//! `imagery` module, *Encoding refuses what it cannot round-trip*), this count
//! is **not fixed** — it scales with the section counts. It is also never
//! zero: every accepted version is at least 3.0, and so `timestamp` is read
//! as an `f64` for every frame this crate parses. A frame with every section empty still has that
//! one NaN-reachable slot, and a [`FrameOfData`] holding a NaN anywhere is
//! not equal even to itself.
//!
//! This is IEEE-754 semantics inherited through the derive, not a framing
//! defect: the NaN's own bits survive the wire unchanged, so a NaN costs
//! value equality and never byte equality. Closing it would
//! mean comparing floats by their bits instead of with `==` — a behaviour
//! change rather than documentation.
//!
//! Two neighbours are **not** affected. `-0.0` keeps its bit pattern across
//! the wire *and* `-0.0 == 0.0`, so it round-trips. And NaN bits inside a
//! [`RawSection`] are compared as the `Vec<u8>` that field is, byte for byte,
//! so the four raw sections round-trip whatever they carry — which is
//! why [`RawSection`] is the only type in this module that derives [`Eq`].
//! Only the float-*typed* slots above are affected. The typed views decoded
//! from a raw section ([`Skeleton`], [`Asset`], [`ForcePlate`], [`Device`])
//! are float-typed too: a decoded NaN costs their value equality, while
//! [`RawSection::from_skeletons`] and its siblings still reproduce its bytes.
//!
//! All of the above is about an `x` that `parse_payload` produced and that the
//! checked encoder accepts. [`FrameOfData::to_payload_lossy`] remains an
//! explicit compatibility escape hatch for intentionally invalid test values;
//! checked encoding rejects interior NULs, unrepresentable counts and values
//! absent from the selected layout instead of silently dropping them.
//!
//! One thing the rule does **not** claim is byte preservation in general: a
//! marker-set name whose wire bytes are not UTF-8 is decoded lossily, so
//! re-serializing it emits U+FFFD where the original bytes were. The value
//! round-trips; the datagram does not.

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
    ///
    /// A `false` here is honest occlusion or absence: a `CW-500 wand` in the
    /// captures above reports `false` (present but untracked).
    #[must_use]
    pub const fn tracking_valid(&self) -> bool {
        self.params & 0x01 != 0
    }
}

/// A named set of reconstructed marker positions.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MarkerSet {
    /// Marker-set name (e.g. `CW-500 wand`, or the catch-all `all`).
    pub name: String,
    /// Marker positions in the server's native frame.
    pub markers: Vec<[f32; 3]>,
}

/// One labelled marker (NatNet >= 3.0). Spec-derived: the real Motive
/// captures carry no labelled markers.
///
/// The same 26-byte entry carries an [`Asset`] marker (NatNet SDK 4.1.0.0
/// `PacketClient.cpp` `UnpackAssetMarkerData`).
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

/// Splits a NatNet composite id into `(entity_id, member_id)`: the high 16
/// bits (arithmetic shift, so the entity keeps its sign) and the low 16 bits.
///
/// NatNet SDK 4.1.0.0 `NatNetTypes.h` documents `sRigidBodyData::ID` of a
/// skeleton bone as combining the skeleton id (high bits) and the bone id
/// (low bits), and `sMarker::ID` of a legacy asset marker the same way; the
/// SDK's `NatNet_DecodeID` (`NatNetCAPI.h`) and `PacketClient.cpp`
/// `DecodeMarkerID` split with `id >> 16` and `id & 0xffff`. That header
/// describes the NatNet library's decoded frame; `PacketClient.cpp`
/// `UnpackSkeletonData` prints the wire id without splitting it, so this
/// encoding of raw bitstream bone ids is spec-derived and has not been
/// confirmed against real Motive bytes. [`Skeleton`] keeps the wire id as is.
#[must_use]
pub const fn decode_id(composite_id: i32) -> (i32, i32) {
    (composite_id >> 16, composite_id & 0xffff)
}

/// Builds a composite id from an entity id (high 16 bits) and a member id
/// (low 16 bits), the inverse of [`decode_id`].
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

/// One skeleton in a data frame: its id and its bones as rigid bodies.
///
/// Layout from NatNet SDK 4.1.0.0 `PacketClient.cpp` `UnpackSkeletonData`
/// (identical for NatNet 3.0 through 4.1): `int32 id`, `int32 nRigidBodies`,
/// then `nRigidBodies` 38-byte rigid bodies. Spec-derived: the real Motive
/// captures carry no skeletons. See [`decode_id`] for the documented bone id
/// encoding.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Skeleton {
    /// Skeleton id (matches [`SkeletonDef::id`](crate::SkeletonDef::id)).
    pub id: i32,
    /// Bones, with their wire ids unchanged.
    pub rigid_bodies: Vec<RigidBody>,
}

/// One asset (for example a trained marker set) in a NatNet 4.1 data frame.
///
/// Layout from NatNet SDK 4.1.0.0 `PacketClient.cpp` `UnpackAssetData`,
/// `UnpackAssetRigidBodyData` and `UnpackAssetMarkerData`: `int32 id`,
/// `int32 nRigidBodies`, 38-byte rigid bodies, `int32 nMarkers`, 26-byte
/// markers. Spec-derived: the real Motive captures carry no assets.
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

/// One force plate in a data frame: per-channel analog samples.
///
/// Layout from NatNet SDK 4.1.0.0 `PacketClient.cpp` `UnpackForcePlateData`
/// (identical for NatNet 3.0 through 4.1): `int32 id`, `int32 nChannels`,
/// then per channel `int32 nFrames` and `nFrames` `f32` samples.
/// `NatNetTypes.h` `sForcePlateData` also declares an `int16 params`, but
/// neither `PacketClient.cpp` nor `NatNetClient.py` reads one from the wire,
/// so it is not part of this layout. Spec-derived: the real Motive captures
/// carry no force plates.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForcePlate {
    /// Force-plate id (matches [`ForcePlateDef::id`](crate::ForcePlateDef::id)).
    pub id: i32,
    /// Samples per channel, in channel order.
    pub channels: Vec<Vec<f32>>,
}

/// One peripheral device (for example an analog DAQ) in a data frame.
///
/// Layout from NatNet SDK 4.1.0.0 `PacketClient.cpp` `UnpackDeviceData`
/// (identical for NatNet 3.0 through 4.1), the same shape as [`ForcePlate`].
/// `NatNetTypes.h` `sDeviceData` also declares an `int16 params` that neither
/// SDK sample reads from the wire. Spec-derived: the real Motive captures
/// carry no devices.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Device {
    /// Device id (matches [`DeviceDef::id`](crate::DeviceDef::id)).
    pub id: i32,
    /// Samples per channel, in channel order.
    pub channels: Vec<Vec<f32>>,
}

/// A frame section kept as its wire body bytes and entry count.
///
/// The four sections kept this way (skeletons, assets, force plates and
/// devices) have typed views: [`FrameOfData::decode_skeletons`] and its
/// siblings decode a section, and [`RawSection::from_skeletons`] and its
/// siblings encode typed entries back. For every section that decodes,
/// re-encoding reproduces the same count and payload bytes, because every
/// field is fixed-width binary (no strings).
///
/// Compatibility decoding of a NatNet 4.1 frame preserves an arbitrary body
/// verbatim; strict decoding and the legacy count-only layouts validate the
/// entry grammar the typed decoders use before retaining the bytes.
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
    /// Software timestamp in seconds (in the 120 f/s captures,
    /// `frame_number / 120`).
    pub timestamp: f64,
    /// Camera mid-exposure timestamp, in clock ticks.
    pub camera_mid_exposure: u64,
    /// Camera data-received timestamp, in clock ticks.
    pub camera_data_received: u64,
    /// Transmit timestamp, in clock ticks.
    pub transmit: u64,
    /// Precision-timestamp seconds (NatNet >= 4.1; spec-derived, zero
    /// in the captures).
    pub precision_timestamp_secs: u32,
    /// Precision-timestamp fractional seconds (NatNet >= 4.1;
    /// spec-derived, zero in the captures).
    pub precision_timestamp_fractional_secs: u32,
}

/// A decoded FrameOfData message.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FrameOfData {
    /// Monotonic frame number.
    ///
    /// Motive's counter is a wrapping 32-bit value and NatNet writes it as
    /// `i32`, so it reads negative once it passes 2^31. The AUGI/1 imagery
    /// side-channel (`imagery` feature) writes the same bits as `u32`: join
    /// the two by reinterpreting the bits, never with `u32::try_from`, which
    /// looks safe and silently discards every frame past the wrap.
    pub frame_number: i32,
    /// Named marker sets.
    pub marker_sets: Vec<MarkerSet>,
    /// Legacy unlabelled ("other") markers.
    pub other_markers: Vec<[f32; 3]>,
    /// Rigid bodies.
    pub rigid_bodies: Vec<RigidBody>,
    /// Skeleton section as wire bytes; typed view via
    /// [`Self::decode_skeletons`], typed construction via
    /// [`RawSection::from_skeletons`].
    pub skeletons: RawSection,
    /// Asset section as wire bytes (NatNet 4.1 only); typed view via
    /// [`Self::decode_assets`], typed construction via
    /// [`RawSection::from_assets`].
    pub assets: RawSection,
    /// Labelled markers (spec-derived; empty in the fixture).
    pub labeled_markers: Vec<LabeledMarker>,
    /// Force-plate section as wire bytes; typed view via
    /// [`Self::decode_force_plates`], typed construction via
    /// [`RawSection::from_force_plates`].
    pub force_plates: RawSection,
    /// Device section as wire bytes; typed view via [`Self::decode_devices`],
    /// typed construction via [`RawSection::from_devices`].
    pub devices: RawSection,
    /// Timing block.
    pub timing: FrameTiming,
    /// Frame parameter bitfield. The NatNet SDK 4.1.0.0 `NatNetTypes.h`
    /// (`sFrameOfMocapData::params`) documents bit 0 (`0x01`) as
    /// *recording*, bit 1 (`0x02`) as *model list changed*, bit 2 (`0x04`)
    /// as *Live/Edit mode* (`0` Live, `1` Edit) and bit 3 (`0x08`) as
    /// *bitstream version changed*; [`Self::bitstream_changed`] reads bit 3.
    /// That is the documented meaning, not an observed behaviour.
    ///
    /// The same SDK's `PacketClient.cpp` sample disagrees with itself about
    /// bit 2: its frame unpacker treats `0x04` as *bitstream version changed*
    /// (and tests Live mode with the two-bit mask `0x03`), while its receive
    /// loop treats `0x04` as Live/Edit and `0x08` as bitstream changed,
    /// matching the header. This crate follows the header and exposes the
    /// raw field, so a caller who observes otherwise can test the bit it
    /// trusts.
    ///
    /// Observed with Motive 3.1.2.1 / NatNet 4.1.0.0 on 2026-08-28: across
    /// 144 356
    /// consecutive frames — a 138 946-frame capture plus two census runs of
    /// 3 062 and 2 348 frames — this field read `0x0000` on every frame, zero
    /// transitions, including an interval the operator described as a
    /// recording in progress. Neither bit was observed set in any of those
    /// frames, and no other observation in this crate's evidence has seen
    /// either bit set. That packet leaves two readings open and this comment
    /// chooses neither: either Motive 3.1.2.1 does not reflect Take state in
    /// this bit, or no Take was running. The recording bit is therefore
    /// unverified against a known-true case, and a consumer should not rely
    /// on it until one is observed. For bit 1, no tracked-model change is
    /// known to have occurred during those frames, so its observation is
    /// *never seen set*, nothing more.
    pub params: u16,
}

/// Reads a signed count used by a typed, count-only section. A negative
/// count is [`Error::InvalidCount`] under every policy.
fn typed_count(r: &mut Reader<'_>) -> Result<i32> {
    let offset = r.position();
    let count = r.i32()?;
    if count < 0 {
        return Err(Error::InvalidCount { count, offset });
    }
    Ok(count)
}

/// Guards a non-negative count before allocating or walking its entries.
///
/// Callers read counts through [`typed_count`] or [`sized_header`], which
/// already refuse negatives; the negative arm here is defensive and reports
/// the current position because the count's own offset is not known.
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

/// Reads a sized section header. A raw section may retain a negative count in
/// compatibility mode because its body is explicitly opaque; typed sections
/// always reject negative counts. A negative byte size is
/// [`Error::InvalidCount`] under every policy.
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

/// Parses a typed section whose body is bounded by its wire size.
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

/// Parses a typed count-only section. The entry grammar supplies the extent;
/// arbitrary bytes are never scanned to guess a legacy section boundary.
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

/// Wire size of one frame rigid body: id, position, orientation, mean marker
/// error and params.
const RIGID_BODY_BYTES: usize = 38;
/// Wire size of one labelled (or asset) marker: id, position, size, params
/// and residual.
const LABELED_MARKER_BYTES: usize = 26;
/// Smallest skeleton, asset, force-plate or device entry: an id and a count.
const MIN_RAW_ENTRY_BYTES: usize = 8;

/// Reads one frame rigid body. NatNet SDK 4.1.0.0 `PacketClient.cpp`
/// `UnpackRigidBodyData`, `UnpackSkeletonData` and `UnpackAssetRigidBodyData`
/// share this 38-byte layout for NatNet >= 3.0.
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

/// Reads one labelled marker. NatNet SDK 4.1.0.0 `PacketClient.cpp`
/// `UnpackLabeledMarkerData` and `UnpackAssetMarkerData` share this 26-byte
/// layout for NatNet >= 3.0.
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

/// Reads `count` entries after proving that their minimum size fits.
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

/// Reads a non-negative nested count followed by its entries.
fn read_counted<T>(
    r: &mut Reader<'_>,
    min_entry: usize,
    entry: impl FnMut(&mut Reader<'_>) -> Result<T>,
) -> Result<Vec<T>> {
    let count = typed_count(r)?;
    read_entries(r, count, min_entry, entry)
}

/// Writes a checked nested count followed by its entries.
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

/// NatNet SDK 4.1.0.0 `PacketClient.cpp` `UnpackSkeletonData`.
fn read_skeleton(r: &mut Reader<'_>) -> Result<Skeleton> {
    let id = r.i32()?;
    let rigid_bodies = read_counted(r, RIGID_BODY_BYTES, read_rigid_body)?;
    Ok(Skeleton { id, rigid_bodies })
}

/// NatNet SDK 4.1.0.0 `PacketClient.cpp` `UnpackAssetData`.
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

/// Channel data shared by `UnpackForcePlateData` and `UnpackDeviceData`:
/// `int32 nChannels`, then per channel `int32 nFrames` and `nFrames` floats.
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

/// NatNet SDK 4.1.0.0 `PacketClient.cpp` `UnpackForcePlateData`.
fn read_force_plate(r: &mut Reader<'_>) -> Result<ForcePlate> {
    Ok(ForcePlate {
        id: r.i32()?,
        channels: read_channels(r)?,
    })
}

/// NatNet SDK 4.1.0.0 `PacketClient.cpp` `UnpackDeviceData`.
fn read_device(r: &mut Reader<'_>) -> Result<Device> {
    Ok(Device {
        id: r.i32()?,
        channels: read_channels(r)?,
    })
}

/// Walks a raw section body with the same grammar the typed decoders use,
/// discarding the decoded entries.
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

/// Decodes a complete raw section body: `count` entries and no residue.
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

/// Encodes typed entries as a raw section with a checked count and size.
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

/// Reads a count-only structural section and preserves its exact body bytes.
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

/// Reads a sized structural section. Compatibility mode intentionally keeps
/// its body opaque; strict mode validates the known nested grammar.
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
    // Entries are written in place; a 4.1 size field is reserved and patched
    // once the body length is known. On error the partial output is dropped
    // with the writer.
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

/// Historical infallible typed-section writer. It is used only by the
/// explicitly named lossy compatibility serializer.
fn lossy_typed_section<T>(
    w: &mut Writer,
    items: &[T],
    sized: bool,
    mut entry: impl FnMut(&mut Writer, &T),
) {
    let mut body = Writer::new();
    for item in items {
        entry(&mut body, item);
    }
    let body = body.into_vec();
    w.i32(i32::try_from(items.len()).unwrap_or(i32::MAX));
    if sized {
        w.i32(i32::try_from(body.len()).unwrap_or(i32::MAX));
    }
    w.bytes(&body);
}

fn checked_raw_section(
    w: &mut Writer,
    s: &RawSection,
    sized: bool,
    count_field: &'static str,
    size_field: &'static str,
) -> Result<()> {
    // The nested raw grammar was already checked by `validate_for_layout`,
    // which `to_payload` runs first for every raw section this writes.
    crate::bytes::checked_i32(s.payload.len(), size_field)?;
    if s.count < 0 {
        return Err(Error::InvalidValue {
            field: count_field,
            reason: "count must be non-negative",
        });
    }
    w.i32(s.count);
    if sized {
        w.i32(crate::bytes::checked_i32(s.payload.len(), size_field)?);
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
    /// Whether [`Self::params`] bit 3 (`0x08`), *bitstream version
    /// changed*, is set.
    ///
    /// The bit meaning is the NatNet SDK 4.1.0.0 header's; see
    /// [`Self::params`] for the conflicting SDK sample and the lack of any
    /// observation of the bit set.
    #[must_use]
    pub const fn bitstream_changed(&self) -> bool {
        self.params & 0x08 != 0
    }

    /// Decodes [`Self::skeletons`] into typed skeletons.
    ///
    /// The section must hold exactly its declared count of entries with no
    /// residue; the layout (on [`Skeleton`]) is the same for every accepted
    /// version.
    ///
    /// # Errors
    /// [`Error::InvalidCount`] for a negative count, [`Error::ImplausibleCount`]
    /// or [`Error::UnexpectedEof`] for a truncated body, and
    /// [`Error::SectionSizeMismatch`] for bytes left after the last entry.
    pub fn decode_skeletons(&self) -> Result<Vec<Skeleton>> {
        decode_raw(&self.skeletons, read_skeleton)
    }

    /// Decodes [`Self::assets`] into typed assets. A frame parsed at a
    /// version before 4.1 has an empty asset section and decodes to an empty
    /// vector.
    ///
    /// # Errors
    /// The same exact-consumption errors as [`Self::decode_skeletons`].
    pub fn decode_assets(&self) -> Result<Vec<Asset>> {
        decode_raw(&self.assets, read_asset)
    }

    /// Decodes [`Self::force_plates`] into typed force plates.
    ///
    /// # Errors
    /// The same exact-consumption errors as [`Self::decode_skeletons`].
    pub fn decode_force_plates(&self) -> Result<Vec<ForcePlate>> {
        decode_raw(&self.force_plates, read_force_plate)
    }

    /// Decodes [`Self::devices`] into typed devices.
    ///
    /// # Errors
    /// The same exact-consumption errors as [`Self::decode_skeletons`].
    pub fn decode_devices(&self) -> Result<Vec<Device>> {
        decode_raw(&self.devices, read_device)
    }

    /// Parses a FrameOfData **datagram** (including the 4-byte header) with
    /// the negotiated protocol `version`.
    ///
    /// # Errors
    ///
    /// [`Error::UnexpectedMessageId`] if the header is not id 7;
    /// [`Error::UnsupportedVersion`] for an unverified layout; or
    /// [`Error::UnexpectedEof`], [`Error::SectionSizeMismatch`] or
    /// [`Error::ImplausibleCount`] for a truncated or inconsistent frame;
    /// [`Error::InvalidCount`] for a negative typed count or section size.
    /// This compatibility wrapper preserves opaque 4.1 raw bodies and accepts
    /// a nonzero terminator or payload suffix; use [`Self::parse_with_policy`]
    /// with [`ParsePolicy::Strict`] for full validation.
    pub fn parse(datagram: &[u8], version: Version) -> Result<Self> {
        Self::parse_with_policy(datagram, version, ParsePolicy::Compatible)
    }

    /// Parses a FrameOfData datagram with an explicit validation policy.
    ///
    /// # Errors
    /// Returns [`Error::UnexpectedMessageId`] for a non-frame datagram,
    /// [`Error::UnsupportedVersion`] for an unverified layout, or a bounded
    /// field, count, section, terminator or trailing-byte error from the
    /// selected policy.
    pub fn parse_with_policy(
        datagram: &[u8],
        version: Version,
        policy: ParsePolicy,
    ) -> Result<Self> {
        // Validate before interpreting any payload bytes. In particular, an
        // unknown future version must not be reduced to a known layout by a
        // threshold comparison.
        version.layout()?;
        let payload = message::expect(datagram, NAT_FRAMEOFDATA)?;
        Self::parse_payload_with_policy(payload, version, policy)
    }

    /// Parses a FrameOfData payload (bytes **after** the 4-byte header).
    ///
    /// # Errors
    ///
    /// See [`FrameOfData::parse`].
    pub fn parse_payload(payload: &[u8], version: Version) -> Result<Self> {
        Self::parse_payload_with_policy(payload, version, ParsePolicy::Compatible)
    }

    /// Parses a FrameOfData payload with an explicit validation policy.
    ///
    /// # Errors
    /// Returns [`Error::UnsupportedVersion`] for an unverified layout, or a
    /// bounded field, count, section, terminator or trailing-byte error from
    /// the selected policy.
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

        // Marker sets: name, nMarkers, nMarkers*[f32;3].
        let marker_sets = if layout.has_section_sizes() {
            sized_typed_section(&mut r, 5, policy, |r| {
                let name = r.cstr()?;
                let n = typed_count(r)?;
                let n = count_len(r, n, 12)?;
                let mut markers = vec_for_count(n);
                for _ in 0..n {
                    markers.push(r.vec3()?);
                }
                Ok(MarkerSet { name, markers })
            })?
        } else {
            legacy_typed_section(&mut r, 5, |r| {
                let name = r.cstr()?;
                let n = typed_count(r)?;
                let n = count_len(r, n, 12)?;
                let mut markers = vec_for_count(n);
                for _ in 0..n {
                    markers.push(r.vec3()?);
                }
                Ok(MarkerSet { name, markers })
            })?
        };

        // Legacy unlabelled markers.
        let other_markers = if layout.has_section_sizes() {
            sized_typed_section(&mut r, 12, policy, |r| r.vec3())?
        } else {
            legacy_typed_section(&mut r, 12, |r| r.vec3())?
        };

        // Rigid bodies.
        let rigid_bodies = if layout.has_section_sizes() {
            sized_typed_section(&mut r, RIGID_BODY_BYTES, policy, read_rigid_body)?
        } else {
            legacy_typed_section(&mut r, RIGID_BODY_BYTES, read_rigid_body)?
        };

        // Skeletons and assets have nested, known structural extents. Their
        // public representation remains raw so every body byte is preserved.
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

        // Labelled markers: id, position, size, params, residual.
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

        // NatNet 3.x and later use an f64 timestamp and three high-resolution
        // camera timestamps. Precision timestamp fields are 4.1-only.
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
    ///
    /// Counts, section sizes and terminated strings are checked before bytes
    /// are emitted. A layout that cannot carry a value is rejected instead of
    /// silently dropping it.
    ///
    /// # Errors
    /// Returns an encoding or unsupported-layout error when a value cannot be
    /// represented faithfully by `version`.
    pub fn to_payload(&self, version: Version) -> Result<Vec<u8>> {
        let layout = version.layout()?;
        self.validate_for_layout(layout)?;
        self.encode_payload_checked(layout)
    }

    /// Historical lossy encoder for deliberately invalid fixture values.
    /// Prefer [`FrameOfData::to_payload`] for application data.
    ///
    /// # Errors
    /// Returns [`Error::UnsupportedVersion`] when `version` does not name a
    /// supported wire layout. Value validation remains intentionally bypassed.
    pub fn to_payload_lossy(&self, version: Version) -> Result<Vec<u8>> {
        Ok(self.encode_payload_lossy(version.layout()?))
    }

    /// Encodes a checked FrameOfData datagram (header plus payload).
    ///
    /// # Errors
    /// Returns the same checked encoding errors as [`Self::to_payload`], and
    /// [`Error::LengthOverflow`] (`field: "message payload size"`) when the
    /// payload is longer than the 65 535 bytes its `u16` header can declare.
    /// Only [`Self::to_datagram_lossy`] stamps a saturated size instead.
    pub fn to_datagram(&self, version: Version) -> Result<Vec<u8>> {
        let payload = self.to_payload(version)?;
        message::frame_checked(NAT_FRAMEOFDATA, &payload)
    }

    /// Historical lossy datagram encoder for deliberately invalid fixture
    /// values. A payload longer than 65 535 bytes is emitted in full behind a
    /// saturated (`0xFFFF`) declared size.
    ///
    /// # Errors
    /// Returns [`Error::UnsupportedVersion`] when `version` does not name a
    /// supported wire layout. Value validation remains intentionally bypassed.
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
