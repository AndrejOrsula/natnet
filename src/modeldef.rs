//! MODELDEF data descriptions (message id 5).
//!
//! - **Envelope:** NatNet 3.x and 4.0 carry a count, then type and body per
//!   dataset; 4.1 adds a byte count after each type. The version picks the
//!   envelope; payload bytes are never sniffed.
//! - **Types:** 0–6 are typed; unknown 4.1 types become [`DataSet::Other`].
//!   Layouts for types 2–6 are spec-derived, unconfirmed against Motive.
//! - **Strict parsing** rejects residue, truncation and malformed counts.
//! - **Compatible parsing** drops residue after type 0 fields and after the
//!   payload, keeps type 1 tails verbatim, and (4.1 only) keeps a type 2–6
//!   body that would not re-encode identically, including invalid UTF-8, as
//!   [`DataSet::Other`]. Legacy layouts have no body size, so a malformed
//!   body is an error there.
//! - **Checked encoding** reproduces the parsed bytes, minus anything
//!   compatible parsing discarded.

use crate::bytes::{Reader, Writer, checked_i32, vec_for_count};
use crate::error::{Error, Result};
use crate::message::{self, NAT_MODELDEF};
use crate::version::{Layout, ParsePolicy, Version};

/// Dataset type id: marker set.
pub const TYPE_MARKER_SET: i32 = 0;
/// Dataset type id: rigid body.
pub const TYPE_RIGID_BODY: i32 = 1;
/// Dataset type id: skeleton.
pub const TYPE_SKELETON: i32 = 2;
/// Dataset type id: force plate.
pub const TYPE_FORCE_PLATE: i32 = 3;
/// Dataset type id: device.
pub const TYPE_DEVICE: i32 = 4;
/// Dataset type id: camera.
pub const TYPE_CAMERA: i32 = 5;
/// Dataset type id: asset.
pub const TYPE_ASSET: i32 = 6;

/// A marker-set data description: a name and its member marker names.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MarkerSetDef {
    /// Marker-set name.
    pub name: String,
    /// Member marker names.
    pub marker_names: Vec<String>,
}

/// A rigid-body data description.
///
/// `tail` contains the version-dependent marker description after the common
/// name/id/parent/offset fields. Compatible parsing keeps it byte-for-byte,
/// including unknown extensions. Strict parsing and checked encoding validate
/// the mandatory prefix; 4.1 preserves suffixes, while legacy requires an
/// exact tail because it has no body size. Both reject truncated or malformed
/// prefixes. Use [`ModelDef::to_payload_lossy`] to retain an invalid tail.
/// Skeleton bones and asset rigid bodies require an exact marker-description
/// tail in every layout ([`SkeletonDef::rigid_bodies`], [`AssetDef::rigid_bodies`]).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RigidBodyDef {
    /// Rigid-body name.
    pub name: String,
    /// Streaming id for a top-level rigid body; in a skeleton, the bone index.
    pub id: i32,
    /// Parent id, or −1 for none.
    pub parent_id: i32,
    /// Pivot offset `[x, y, z]` in the server's native frame.
    pub offset: [f32; 3],
    /// Version-dependent marker description bytes.
    pub tail: Vec<u8>,
}

/// A skeleton data description (dataset type 2).
///
/// Wire: `cstr name`, `int32 id`, `int32 nRigidBodies`, then that many
/// rigid-body descriptions. Spec-derived (SDK `UnpackSkeletonDescription`),
/// unconfirmed against real Motive bytes. Bone tails contain
/// exactly the version-dependent marker description, with no opaque suffix.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SkeletonDef {
    /// Skeleton name.
    pub name: String,
    /// Skeleton id, as streamed in FrameOfData skeletons.
    pub id: i32,
    /// Bone descriptions. `NatNetTypes.h` documents a bone's `id` as its
    /// bone index and `parent_id` as the parent bone.
    pub rigid_bodies: Vec<RigidBodyDef>,
}

/// A force-plate data description (dataset type 3).
///
/// Spec-derived (SDK `UnpackForcePlateDescription`, `sForcePlateDescription`),
/// unconfirmed against real Motive bytes.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForcePlateDef {
    /// Force-plate id, as streamed in FrameOfData force plates.
    pub id: i32,
    /// Serial number.
    pub serial_number: String,
    /// Physical width (manufacturer supplied).
    pub width: f32,
    /// Physical length (manufacturer supplied).
    pub length: f32,
    /// Electrical-centre offset `[x, y, z]` (manufacturer supplied).
    pub origin: [f32; 3],
    /// 12x12 calibration matrix, row-major as on the wire.
    pub calibration_matrix: [[f32; 12]; 12],
    /// Four plate corners `[x, y, z]` in world coordinates.
    pub corners: [[f32; 3]; 4],
    /// Plate type (C3D convention).
    pub plate_type: i32,
    /// Channel data type (`NatNetTypes.h`: 0 calibrated force data, 1 raw
    /// analog voltages).
    pub channel_data_type: i32,
    /// Channel names.
    pub channel_names: Vec<String>,
}

/// A peripheral-device data description (dataset type 4).
///
/// Spec-derived (SDK `UnpackDeviceDescription`, `sDeviceDescription`),
/// unconfirmed against real Motive bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeviceDef {
    /// Device id, as streamed in FrameOfData devices.
    pub id: i32,
    /// Device name as shown in Motive.
    pub name: String,
    /// Serial number.
    pub serial_number: String,
    /// Device type code.
    pub device_type: i32,
    /// Channel data type code.
    pub channel_data_type: i32,
    /// Channel names.
    pub channel_names: Vec<String>,
}

/// A camera data description (dataset type 5).
///
/// Wire: `cstr name`, `[f32; 3]` position, `[f32; 4]` orientation.
/// Spec-derived (SDK `UnpackCameraDescription`), unconfirmed against real
/// Motive bytes.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CameraDef {
    /// Camera name.
    pub name: String,
    /// Position `[x, y, z]` in the server's native frame.
    pub position: [f32; 3],
    /// Orientation quaternion `[x, y, z, w]` in the server's native frame.
    pub orientation: [f32; 4],
}

/// A marker description inside an [`AssetDef`].
///
/// Spec-derived layout: name, id, initial position, size, and raw 16-bit
/// params (bit 0 is documented as active).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MarkerDef {
    /// Marker name.
    pub name: String,
    /// Marker id.
    pub id: i32,
    /// Initial position `[x, y, z]`.
    pub position: [f32; 3],
    /// Marker size.
    pub size: f32,
    /// Raw parameter bits.
    pub params: u16,
}

/// An asset data description (dataset type 6), for example a trained marker
/// set.
///
/// Spec-derived and unconfirmed against real Motive bytes. Rigid bodies
/// follow the same exact-tail rule as [`SkeletonDef`] bones.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssetDef {
    /// Asset name.
    pub name: String,
    /// Asset type (`NatNetTypes.h` `AssetTypes`: 1 is a trained marker set).
    pub asset_type: i32,
    /// Asset id, as streamed in FrameOfData assets.
    pub id: i32,
    /// Rigid-body descriptions.
    pub rigid_bodies: Vec<RigidBodyDef>,
    /// Marker descriptions.
    pub markers: Vec<MarkerDef>,
}

/// One dataset in a MODELDEF reply.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum DataSet {
    /// A marker-set description.
    MarkerSet(MarkerSetDef),
    /// A rigid-body description.
    RigidBody(RigidBodyDef),
    /// A skeleton description.
    Skeleton(SkeletonDef),
    /// A force-plate description.
    ForcePlate(Box<ForcePlateDef>),
    /// A peripheral-device description.
    Device(DeviceDef),
    /// A camera description.
    Camera(CameraDef),
    /// An asset description.
    Asset(AssetDef),
    /// A dataset kept verbatim: unknown 4.1 type or unrepresentable type 2–6
    /// body (see the [module docs](self)).
    Other {
        /// The dataset type id.
        type_id: i32,
        /// The dataset body bytes.
        body: Vec<u8>,
    },
}

impl DataSet {
    /// The dataset's wire type id.
    #[must_use]
    pub const fn type_id(&self) -> i32 {
        match self {
            Self::MarkerSet(_) => TYPE_MARKER_SET,
            Self::RigidBody(_) => TYPE_RIGID_BODY,
            Self::Skeleton(_) => TYPE_SKELETON,
            Self::ForcePlate(_) => TYPE_FORCE_PLATE,
            Self::Device(_) => TYPE_DEVICE,
            Self::Camera(_) => TYPE_CAMERA,
            Self::Asset(_) => TYPE_ASSET,
            Self::Other { type_id, .. } => *type_id,
        }
    }
}

/// A decoded MODELDEF reply: an ordered list of data descriptions.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModelDef {
    /// The datasets, in wire order.
    pub datasets: Vec<DataSet>,
}

fn invalid_count(count: i32, offset: usize) -> Error {
    Error::InvalidCount { count, offset }
}

/// Reads a count, checking `count * min_entry_bytes` fits in the remaining
/// bytes before anything is allocated.
fn count(r: &mut Reader<'_>, min_entry_bytes: usize) -> Result<usize> {
    let offset = r.position();
    let raw = r.i32()?;
    if raw < 0 {
        return Err(invalid_count(raw, offset));
    }
    r.guard_count(raw, min_entry_bytes)
}

fn size(r: &mut Reader<'_>) -> Result<usize> {
    let offset = r.position();
    let raw = r.i32()?;
    if raw < 0 {
        return Err(invalid_count(raw, offset));
    }
    let size = usize::try_from(raw).map_err(|_| invalid_count(raw, offset))?;
    if size > r.remaining() {
        return Err(Error::UnexpectedEof {
            offset: r.position(),
            needed: size,
            available: r.remaining(),
        });
    }
    Ok(size)
}

fn read_marker_set_def(r: &mut Reader<'_>) -> Result<MarkerSetDef> {
    let name = r.cstr()?;
    let n = count(r, 1)?;
    let mut marker_names = vec_for_count(n);
    for _ in 0..n {
        marker_names.push(r.cstr()?);
    }
    Ok(MarkerSetDef { name, marker_names })
}

fn rigid_marker_min(layout: Layout) -> usize {
    if layout.rigid_body_marker_names() {
        17
    } else {
        16
    }
}

fn walk_rigid_body_tail(r: &mut Reader<'_>, layout: Layout) -> Result<()> {
    let n = count(r, rigid_marker_min(layout))?;
    for _ in 0..n {
        r.vec3()?;
    }
    for _ in 0..n {
        r.i32()?;
    }
    if layout.rigid_body_marker_names() {
        for _ in 0..n {
            r.cstr()?;
        }
    }
    Ok(())
}

fn read_rigid_body_def(r: &mut Reader<'_>, layout: Layout) -> Result<RigidBodyDef> {
    let name = r.cstr()?;
    let id = r.i32()?;
    let parent_id = r.i32()?;
    let offset = r.vec3()?;
    let tail_start = r.position();
    walk_rigid_body_tail(r, layout)?;
    let tail = r.span(tail_start, r.position())?.to_vec();
    Ok(RigidBodyDef {
        name,
        id,
        parent_id,
        offset,
        tail,
    })
}

const MIN_RIGID_BODY_DEF: usize = 25;
const MIN_MARKER_DEF: usize = 23;

fn read_rigid_body_defs(r: &mut Reader<'_>, layout: Layout) -> Result<Vec<RigidBodyDef>> {
    let n = count(r, MIN_RIGID_BODY_DEF)?;
    let mut out = vec_for_count(n);
    for _ in 0..n {
        out.push(read_rigid_body_def(r, layout)?);
    }
    Ok(out)
}

fn read_names(r: &mut Reader<'_>) -> Result<Vec<String>> {
    let n = count(r, 1)?;
    let mut out = vec_for_count(n);
    for _ in 0..n {
        out.push(r.cstr()?);
    }
    Ok(out)
}

fn read_skeleton_def(r: &mut Reader<'_>, layout: Layout) -> Result<SkeletonDef> {
    Ok(SkeletonDef {
        name: r.cstr()?,
        id: r.i32()?,
        rigid_bodies: read_rigid_body_defs(r, layout)?,
    })
}

fn read_force_plate_def(r: &mut Reader<'_>) -> Result<ForcePlateDef> {
    let id = r.i32()?;
    let serial_number = r.cstr()?;
    let width = r.f32()?;
    let length = r.f32()?;
    let origin = r.vec3()?;
    let mut calibration_matrix = [[0.0; 12]; 12];
    for row in &mut calibration_matrix {
        for value in row {
            *value = r.f32()?;
        }
    }
    let mut corners = [[0.0; 3]; 4];
    for corner in &mut corners {
        *corner = r.vec3()?;
    }
    Ok(ForcePlateDef {
        id,
        serial_number,
        width,
        length,
        origin,
        calibration_matrix,
        corners,
        plate_type: r.i32()?,
        channel_data_type: r.i32()?,
        channel_names: read_names(r)?,
    })
}

fn read_device_def(r: &mut Reader<'_>) -> Result<DeviceDef> {
    Ok(DeviceDef {
        id: r.i32()?,
        name: r.cstr()?,
        serial_number: r.cstr()?,
        device_type: r.i32()?,
        channel_data_type: r.i32()?,
        channel_names: read_names(r)?,
    })
}

fn read_camera_def(r: &mut Reader<'_>) -> Result<CameraDef> {
    Ok(CameraDef {
        name: r.cstr()?,
        position: r.vec3()?,
        orientation: r.quat()?,
    })
}

fn read_marker_def(r: &mut Reader<'_>) -> Result<MarkerDef> {
    Ok(MarkerDef {
        name: r.cstr()?,
        id: r.i32()?,
        position: r.vec3()?,
        size: r.f32()?,
        params: r.u16()?,
    })
}

fn read_asset_def(r: &mut Reader<'_>, layout: Layout) -> Result<AssetDef> {
    let name = r.cstr()?;
    let asset_type = r.i32()?;
    let id = r.i32()?;
    let rigid_bodies = read_rigid_body_defs(r, layout)?;
    let n = count(r, MIN_MARKER_DEF)?;
    let mut markers = vec_for_count(n);
    for _ in 0..n {
        markers.push(read_marker_def(r)?);
    }
    Ok(AssetDef {
        name,
        asset_type,
        id,
        rigid_bodies,
        markers,
    })
}

fn read_described(type_id: i32, r: &mut Reader<'_>, layout: Layout) -> Result<Option<DataSet>> {
    Ok(Some(match type_id {
        TYPE_SKELETON => DataSet::Skeleton(read_skeleton_def(r, layout)?),
        TYPE_FORCE_PLATE => DataSet::ForcePlate(Box::new(read_force_plate_def(r)?)),
        TYPE_DEVICE => DataSet::Device(read_device_def(r)?),
        TYPE_CAMERA => DataSet::Camera(read_camera_def(r)?),
        TYPE_ASSET => DataSet::Asset(read_asset_def(r, layout)?),
        _ => return Ok(None),
    }))
}

fn walk_dataset(r: &mut Reader<'_>, type_id: i32, layout: Layout, version: Version) -> Result<()> {
    match type_id {
        TYPE_MARKER_SET => {
            read_marker_set_def(r)?;
        }
        TYPE_RIGID_BODY => {
            read_rigid_body_def(r, layout)?;
        }
        _ => {
            if read_described(type_id, r, layout)?.is_none() {
                return Err(Error::UnsupportedDataSet {
                    type_id,
                    major: version.major,
                    minor: version.minor,
                });
            }
        }
    }
    Ok(())
}

fn validate_rigid_tail(tail: &[u8], layout: Layout) -> Result<()> {
    let mut r = Reader::new(tail);
    walk_rigid_body_tail(&mut r, layout)?;
    if !layout.has_section_sizes() {
        r.finish()?;
    }
    Ok(())
}

fn validate_nested_rigid_tail(tail: &[u8], layout: Layout) -> Result<()> {
    let mut r = Reader::new(tail);
    walk_rigid_body_tail(&mut r, layout)?;
    r.finish()
}

fn parse_marker_set_reader(r: &mut Reader<'_>, policy: ParsePolicy) -> Result<DataSet> {
    let marker_set = read_marker_set_def(r)?;
    if policy == ParsePolicy::Strict {
        r.finish()?;
    } else {
        r.take(r.remaining())?;
    }
    Ok(DataSet::MarkerSet(marker_set))
}

fn parse_rigid_body_reader(
    r: &mut Reader<'_>,
    layout: Layout,
    policy: ParsePolicy,
) -> Result<DataSet> {
    let name = r.cstr()?;
    let id = r.i32()?;
    let parent_id = r.i32()?;
    let offset = r.vec3()?;
    let tail_start = r.position();
    let tail_len = r.remaining();
    let mut tail_reader = r.subreader(tail_len)?;
    if policy == ParsePolicy::Strict {
        walk_rigid_body_tail(&mut tail_reader, layout)?;
    }
    let tail = r.span(tail_start, tail_start + tail_len)?.to_vec();
    Ok(DataSet::RigidBody(RigidBodyDef {
        name,
        id,
        parent_id,
        offset,
        tail,
    }))
}

fn parse_described_reader(
    type_id: i32,
    r: &mut Reader<'_>,
    layout: Layout,
    version: Version,
    policy: ParsePolicy,
) -> Result<DataSet> {
    let start = r.position();
    let len = r.remaining();
    let mut body = r.subreader(len)?;
    let parsed = read_described(type_id, &mut body, layout).and_then(|dataset| {
        body.finish()?;
        Ok(dataset)
    });
    let bytes = r.span(start, start + len)?;
    match (policy, parsed) {
        (ParsePolicy::Strict, Ok(Some(dataset))) => Ok(dataset),
        (ParsePolicy::Strict, Err(error)) => Err(error),
        (ParsePolicy::Compatible, Ok(Some(dataset)))
            if encode_dataset(&dataset, layout, version).is_ok_and(|encoded| encoded == bytes) =>
        {
            Ok(dataset)
        }
        _ => Ok(DataSet::Other {
            type_id,
            body: bytes.to_vec(),
        }),
    }
}

fn parse_dataset_reader(
    type_id: i32,
    r: &mut Reader<'_>,
    layout: Layout,
    version: Version,
    policy: ParsePolicy,
) -> Result<DataSet> {
    match type_id {
        TYPE_MARKER_SET => parse_marker_set_reader(r, policy),
        TYPE_RIGID_BODY => parse_rigid_body_reader(r, layout, policy),
        TYPE_SKELETON..=TYPE_ASSET => parse_described_reader(type_id, r, layout, version, policy),
        _ => Ok(DataSet::Other {
            type_id,
            body: r.take(r.remaining())?.to_vec(),
        }),
    }
}

fn parse_dataset(
    type_id: i32,
    body: &[u8],
    layout: Layout,
    version: Version,
    policy: ParsePolicy,
) -> Result<DataSet> {
    let mut r = Reader::new(body);
    let dataset = parse_dataset_reader(type_id, &mut r, layout, version, policy)?;
    r.finish()?;
    Ok(dataset)
}

impl ModelDef {
    /// Parses a MODELDEF datagram with compatible validation; use
    /// [`Self::parse_with_policy`] for strict decoding.
    ///
    /// # Errors
    /// Returns a header, unsupported-version, count, or dataset decoding error.
    pub fn parse(datagram: &[u8], version: Version) -> Result<Self> {
        Self::parse_with_policy(datagram, version, ParsePolicy::Compatible)
    }

    /// Parses a MODELDEF payload (bytes after the 4-byte header) with
    /// compatible validation.
    ///
    /// # Errors
    /// Returns an unsupported-version, count, or dataset decoding error.
    pub fn parse_payload(payload: &[u8], version: Version) -> Result<Self> {
        Self::parse_payload_with_policy(payload, version, ParsePolicy::Compatible)
    }

    /// Parses a datagram with an explicit version and validation policy.
    /// # Errors
    /// Returns a header, unsupported-version, count, or policy validation error.
    pub fn parse_with_policy(
        datagram: &[u8],
        version: Version,
        policy: ParsePolicy,
    ) -> Result<Self> {
        Self::parse_payload_with_policy(message::expect(datagram, NAT_MODELDEF)?, version, policy)
    }

    /// Parses a payload with an explicit version and validation policy.
    /// # Errors
    /// Returns an unsupported-version, count, or policy validation error.
    pub fn parse_payload_with_policy(
        payload: &[u8],
        version: Version,
        policy: ParsePolicy,
    ) -> Result<Self> {
        let layout = version.layout()?;
        let mut r = Reader::new(payload);
        let dataset_count = count(&mut r, if layout.has_section_sizes() { 8 } else { 4 })?;
        let mut datasets = vec_for_count(dataset_count);

        for _ in 0..dataset_count {
            let type_id = r.i32()?;
            if layout.has_section_sizes() {
                let body_size = size(&mut r)?;
                let mut body_reader = r.subreader(body_size)?;
                let dataset =
                    parse_dataset_reader(type_id, &mut body_reader, layout, version, policy)?;
                body_reader.finish()?;
                datasets.push(dataset);
            } else {
                let start = r.position();
                walk_dataset(&mut r, type_id, layout, version)?;
                let end = r.position();
                let body = r.span(start, end)?.to_vec();
                datasets.push(parse_dataset(type_id, &body, layout, version, policy)?);
            }
        }

        if policy == ParsePolicy::Strict {
            r.finish()?;
        }
        Ok(Self { datasets })
    }

    /// Encodes a checked MODELDEF payload (without the four-byte header).
    ///
    /// # Errors
    /// Returns an unsupported-version error, or a checked encoding error for an
    /// invalid string, count, dataset, or rigid-body marker prefix.
    pub fn to_payload(&self, version: Version) -> Result<Vec<u8>> {
        self.encode_payload(version.layout()?, version)
    }

    /// Encodes a checked datagram (header plus payload).
    ///
    /// # Errors
    /// Same as [`Self::to_payload`], plus [`Error::LengthOverflow`] for a
    /// payload above the 65 535-byte `u16` limit.
    pub fn to_datagram(&self, version: Version) -> Result<Vec<u8>> {
        let payload = self.to_payload(version)?;
        message::frame_checked(NAT_MODELDEF, &payload)
    }

    /// Encodes a payload without value validation: strings may truncate and
    /// counts may saturate or use type aliases.
    ///
    /// # Errors
    /// Returns [`Error::UnsupportedVersion`] for unsupported layouts.
    pub fn to_payload_lossy(&self, version: Version) -> Result<Vec<u8>> {
        Ok(self.encode_payload_lossy(version.layout()?))
    }

    /// Encodes without value validation. Payloads over 65 535 bytes are emitted
    /// in full with a saturated declared size.
    ///
    /// # Errors
    /// Returns [`Error::UnsupportedVersion`] for unsupported layouts.
    pub fn to_datagram_lossy(&self, version: Version) -> Result<Vec<u8>> {
        let payload = self.to_payload_lossy(version)?;
        Ok(message::frame(NAT_MODELDEF, &payload))
    }

    fn encode_payload(&self, layout: Layout, version: Version) -> Result<Vec<u8>> {
        let mut w = Writer::new();
        w.i32(checked_i32(self.datasets.len(), "dataset count")?);
        for dataset in &self.datasets {
            let type_id = dataset.type_id();
            let body = encode_dataset(dataset, layout, version)?;
            w.i32(type_id);
            if layout.has_section_sizes() {
                w.i32(checked_i32(body.len(), "dataset body")?);
            }
            w.bytes(&body);
        }
        Ok(w.into_vec())
    }

    fn encode_payload_lossy(&self, layout: Layout) -> Vec<u8> {
        let mut w = Writer::new();
        w.i32(i32::try_from(self.datasets.len()).unwrap_or(i32::MAX));
        for dataset in &self.datasets {
            let type_id = dataset.type_id();
            let body = encode_dataset_lossy(dataset, layout);
            w.i32(type_id);
            if layout.has_section_sizes() {
                w.i32(i32::try_from(body.len()).unwrap_or(i32::MAX));
            }
            w.bytes(&body);
        }
        w.into_vec()
    }

    /// Resolves a rigid-body name to its streaming id. Returns the first
    /// match, or `None`.
    #[must_use]
    pub fn resolve_rigid_body(&self, name: &str) -> Option<i32> {
        self.datasets.iter().find_map(|ds| match ds {
            DataSet::RigidBody(rb) if rb.name == name => Some(rb.id),
            _ => None,
        })
    }

    /// All rigid-body (name, id) pairs in the definition.
    pub fn rigid_bodies(&self) -> impl Iterator<Item = (&str, i32)> {
        self.datasets.iter().filter_map(|ds| match ds {
            DataSet::RigidBody(rb) => Some((rb.name.as_str(), rb.id)),
            _ => None,
        })
    }

    /// Counts datasets of a given wire `type_id`.
    #[must_use]
    pub fn count_of_type(&self, type_id: i32) -> usize {
        self.datasets
            .iter()
            .filter(|d| d.type_id() == type_id)
            .count()
    }
}

fn write_rigid_body_head(w: &mut Writer, rigid_body: &RigidBodyDef) -> Result<()> {
    w.try_cstr(&rigid_body.name, "rigid-body name")?;
    w.i32(rigid_body.id);
    w.i32(rigid_body.parent_id);
    w.vec3(rigid_body.offset);
    Ok(())
}

fn write_nested_rigid_bodies(
    w: &mut Writer,
    rigid_bodies: &[RigidBodyDef],
    layout: Layout,
    count_field: &'static str,
) -> Result<()> {
    w.i32(checked_i32(rigid_bodies.len(), count_field)?);
    for rigid_body in rigid_bodies {
        write_rigid_body_head(w, rigid_body)?;
        validate_nested_rigid_tail(&rigid_body.tail, layout)?;
        w.bytes(&rigid_body.tail);
    }
    Ok(())
}

fn write_names(
    w: &mut Writer,
    names: &[String],
    count_field: &'static str,
    name_field: &'static str,
) -> Result<()> {
    w.i32(checked_i32(names.len(), count_field)?);
    for name in names {
        w.try_cstr(name, name_field)?;
    }
    Ok(())
}

fn write_force_plate_fixed(w: &mut Writer, plate: &ForcePlateDef) {
    w.f32(plate.width);
    w.f32(plate.length);
    w.vec3(plate.origin);
    for row in &plate.calibration_matrix {
        for value in row {
            w.f32(*value);
        }
    }
    for corner in &plate.corners {
        w.vec3(*corner);
    }
    w.i32(plate.plate_type);
    w.i32(plate.channel_data_type);
}

fn write_marker_def_fixed(w: &mut Writer, marker: &MarkerDef) {
    w.i32(marker.id);
    w.vec3(marker.position);
    w.f32(marker.size);
    w.u16(marker.params);
}

fn encode_dataset(dataset: &DataSet, layout: Layout, version: Version) -> Result<Vec<u8>> {
    let mut w = Writer::new();
    match dataset {
        DataSet::MarkerSet(marker_set) => {
            w.try_cstr(&marker_set.name, "marker-set name")?;
            write_names(
                &mut w,
                &marker_set.marker_names,
                "marker count",
                "marker name",
            )?;
        }
        DataSet::RigidBody(rigid_body) => {
            write_rigid_body_head(&mut w, rigid_body)?;
            validate_rigid_tail(&rigid_body.tail, layout)?;
            w.bytes(&rigid_body.tail);
        }
        DataSet::Skeleton(skeleton) => {
            w.try_cstr(&skeleton.name, "skeleton name")?;
            w.i32(skeleton.id);
            write_nested_rigid_bodies(
                &mut w,
                &skeleton.rigid_bodies,
                layout,
                "skeleton rigid body count",
            )?;
        }
        DataSet::ForcePlate(plate) => {
            w.i32(plate.id);
            w.try_cstr(&plate.serial_number, "force plate serial number")?;
            write_force_plate_fixed(&mut w, plate);
            write_names(
                &mut w,
                &plate.channel_names,
                "force plate channel count",
                "force plate channel name",
            )?;
        }
        DataSet::Device(device) => {
            w.i32(device.id);
            w.try_cstr(&device.name, "device name")?;
            w.try_cstr(&device.serial_number, "device serial number")?;
            w.i32(device.device_type);
            w.i32(device.channel_data_type);
            write_names(
                &mut w,
                &device.channel_names,
                "device channel count",
                "device channel name",
            )?;
        }
        DataSet::Camera(camera) => {
            w.try_cstr(&camera.name, "camera name")?;
            w.vec3(camera.position);
            w.quat(camera.orientation);
        }
        DataSet::Asset(asset) => {
            w.try_cstr(&asset.name, "asset name")?;
            w.i32(asset.asset_type);
            w.i32(asset.id);
            write_nested_rigid_bodies(
                &mut w,
                &asset.rigid_bodies,
                layout,
                "asset rigid body count",
            )?;
            w.i32(checked_i32(asset.markers.len(), "asset marker count")?);
            for marker in &asset.markers {
                w.try_cstr(&marker.name, "asset marker name")?;
                write_marker_def_fixed(&mut w, marker);
            }
        }
        DataSet::Other { type_id, body } => {
            if *type_id == TYPE_MARKER_SET || *type_id == TYPE_RIGID_BODY {
                return Err(Error::InvalidValue {
                    field: "dataset type",
                    reason: "type 0 and type 1 require their typed representation",
                });
            }
            if !layout.has_section_sizes() {
                let mut r = Reader::new(body);
                walk_dataset(&mut r, *type_id, layout, version)?;
                r.finish()?;
            }
            w.bytes(body);
        }
    }
    Ok(w.into_vec())
}

fn lossy_count(len: usize) -> i32 {
    i32::try_from(len).unwrap_or(i32::MAX)
}

fn lossy_rigid_bodies(w: &mut Writer, rigid_bodies: &[RigidBodyDef]) {
    w.i32(lossy_count(rigid_bodies.len()));
    for rigid_body in rigid_bodies {
        lossy_rigid_body(w, rigid_body);
    }
}

fn lossy_rigid_body(w: &mut Writer, rigid_body: &RigidBodyDef) {
    w.cstr(&rigid_body.name);
    w.i32(rigid_body.id);
    w.i32(rigid_body.parent_id);
    w.vec3(rigid_body.offset);
    w.bytes(&rigid_body.tail);
}

fn lossy_names(w: &mut Writer, names: &[String]) {
    w.i32(lossy_count(names.len()));
    for name in names {
        w.cstr(name);
    }
}

fn encode_dataset_lossy(dataset: &DataSet, layout: Layout) -> Vec<u8> {
    let mut w = Writer::new();
    match dataset {
        DataSet::MarkerSet(marker_set) => {
            w.cstr(&marker_set.name);
            lossy_names(&mut w, &marker_set.marker_names);
        }
        DataSet::RigidBody(rigid_body) => lossy_rigid_body(&mut w, rigid_body),
        DataSet::Skeleton(skeleton) => {
            w.cstr(&skeleton.name);
            w.i32(skeleton.id);
            lossy_rigid_bodies(&mut w, &skeleton.rigid_bodies);
        }
        DataSet::ForcePlate(plate) => {
            w.i32(plate.id);
            w.cstr(&plate.serial_number);
            write_force_plate_fixed(&mut w, plate);
            lossy_names(&mut w, &plate.channel_names);
        }
        DataSet::Device(device) => {
            w.i32(device.id);
            w.cstr(&device.name);
            w.cstr(&device.serial_number);
            w.i32(device.device_type);
            w.i32(device.channel_data_type);
            lossy_names(&mut w, &device.channel_names);
        }
        DataSet::Camera(camera) => {
            w.cstr(&camera.name);
            w.vec3(camera.position);
            w.quat(camera.orientation);
        }
        DataSet::Asset(asset) => {
            w.cstr(&asset.name);
            w.i32(asset.asset_type);
            w.i32(asset.id);
            lossy_rigid_bodies(&mut w, &asset.rigid_bodies);
            w.i32(lossy_count(asset.markers.len()));
            for marker in &asset.markers {
                w.cstr(&marker.name);
                write_marker_def_fixed(&mut w, marker);
            }
        }
        DataSet::Other { body, .. } => w.bytes(body),
    }
    let _ = layout;
    w.into_vec()
}
