//! Synthetic **FrameOfData** generation.
//!
//! Turns a caller-supplied trajectory — positions and orientations sampled
//! over time, in the server's native (Motive) frame — into **real NatNet
//! wire bytes** by reusing the exact same [`FrameOfData::to_datagram`]
//! serialize path the decoder reads back. Nothing here invents a second wire
//! format: the generated datagrams are byte-for-byte what
//! [`FrameOfData::parse`](crate::FrameOfData::parse) consumes.
//!
//! **The round-trip rule this module inherits is the qualified one** — the
//! one in [Round-trip, and the NaN a frame can
//! carry](crate::frame#round-trip-and-the-nan-a-frame-can-carry), not an
//! unconditional `parse(serialize(x)) == x`. Two of that rule's conditions
//! bear on a *generator* in particular. The frames here are **constructed,
//! not parsed**, so they are outside the population the rule quantifies over;
//! and [`PlannedBody::tracked`] takes its `position` and `orientation`
//! straight from the caller, with no `is_nan` or `is_finite` filter anywhere
//! in this module. Plan a NaN and you get a datagram whose bytes reproduce
//! exactly and whose parsed value is **not equal even to itself** — it is the
//! comparison that fails, never the bytes.
//!
//! # Sans-io, deterministic, plain arrays
//!
//! Like the rest of the crate this is pure data → bytes: no socket, no
//! clock, no RNG. Pose/quaternion fields are plain `[f32; N]` arrays in the
//! server's native frame (the Y-up→Z-up opinion is the consumer's, exactly
//! as for parsing). The same inputs always produce the same bytes.
//!
//! # Two things a synthetic stream must reproduce honestly
//!
//! - **Dropouts.** A [`PlannedBody`] with `tracking_valid == false` is
//!   emitted as a present-but-untracked rigid body (params bit 0 clear), as
//!   a real Motive capture carried a `CW-500 wand`. The NatNet decoder
//!   retains it; the consumer maps tracking-invalid poses to gaps.
//! - **Injected clock offset.** The server's *software* timestamp is shifted
//!   by [`SyntheticServer::clock_offset_secs`] relative to the camera
//!   mid-exposure ticks, so a clock estimator under test has a real offset
//!   to recover from the `(software timestamp, camera-clock)` pair rather
//!   than a pre-aligned stream.

use crate::frame::{
    Asset, Device, ForcePlate, FrameOfData, FrameTiming, RawSection, RigidBody, Skeleton,
};
use crate::modeldef::ModelDef;
use crate::version::Version;

/// Typed skeleton, asset, force-plate and device entries a synthetic frame
/// carries next to its planned rigid bodies.
///
/// The entries are encoded with [`RawSection::from_skeletons`] and its
/// siblings, so they reach the wire through the same checked path as any
/// other frame. Their layouts are spec-derived from the NatNet SDK 4.1.0.0
/// (see [`Skeleton`]); a synthetic stream exercises the codec, not Motive.
/// Only NatNet 4.1 carries assets: encoding non-empty `assets` at an older
/// version is refused by [`FrameOfData::to_datagram`].
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlannedSections {
    /// Skeletons, in wire order.
    pub skeletons: Vec<Skeleton>,
    /// Assets, in wire order (NatNet 4.1 only).
    pub assets: Vec<Asset>,
    /// Force plates, in wire order.
    pub force_plates: Vec<ForcePlate>,
    /// Devices, in wire order.
    pub devices: Vec<Device>,
}

/// One rigid body's planned state within a synthetic frame, in the server's
/// native (Motive) frame — plain arrays, convention-neutral.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlannedBody {
    /// Streaming id (as it will appear on the wire).
    pub id: i32,
    /// Position `[x, y, z]` in the server's native frame, metres.
    pub position: [f32; 3],
    /// Orientation quaternion `[x, y, z, w]` in the server's native frame.
    pub orientation: [f32; 4],
    /// Mean marker error the tracker would report, metres (`>= 0`).
    pub mean_marker_error: f32,
    /// Whether the tracker reports a valid pose this frame. `false` emits a
    /// present-but-untracked body (params bit 0 clear) — a scheduled dropout.
    pub tracking_valid: bool,
}

impl PlannedBody {
    /// A tracking-valid body at the given native-frame pose.
    #[must_use]
    pub const fn tracked(id: i32, position: [f32; 3], orientation: [f32; 4]) -> Self {
        Self {
            id,
            position,
            orientation,
            mean_marker_error: 0.0,
            tracking_valid: true,
        }
    }

    /// A scheduled dropout for `id`: present in the frame but untracked. The
    /// pose fields are retained but do not denote a valid tracked pose.
    #[must_use]
    pub const fn dropout(id: i32) -> Self {
        Self {
            id,
            position: [0.0; 3],
            orientation: [0.0, 0.0, 0.0, 1.0],
            mean_marker_error: 0.0,
            tracking_valid: false,
        }
    }
}

/// A synthetic Motive-like server: fixes the protocol [`Version`], the
/// high-resolution camera-clock frequency and the injected software-clock
/// offset, then emits wire frames from planned body states.
///
/// The camera mid-exposure ticks are placed on the *true* timeline
/// (`round(true_time · frequency)`), while the software `timestamp` is
/// shifted by [`clock_offset_secs`](Self::clock_offset_secs). A stream with
/// a non-zero offset is therefore *not* pre-aligned: recovering the offset is
/// the clock estimator's job. Camera ticks saturate at `u64::MAX`; negative
/// and NaN products become zero. The floating-point software timestamp retains
/// non-finite values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyntheticServer {
    /// The protocol bitstream version emitted (e.g. [`Version::V4_1`]).
    pub version: Version,
    /// Camera high-resolution clock frequency (ticks per second), the
    /// SERVERINFO `high_res_clock_frequency`. Used to place the camera
    /// mid-exposure ticks.
    pub high_res_clock_frequency: u64,
    /// Injected offset added to the software `timestamp` (seconds); the
    /// camera-clock ticks are left on the true timeline so the offset is
    /// recoverable.
    pub clock_offset_secs: f64,
}

impl SyntheticServer {
    /// A synthetic server at `version` with a `frequency`-tick camera clock
    /// and a zero software-clock offset (a pre-aligned stream).
    #[must_use]
    pub const fn new(version: Version, high_res_clock_frequency: u64) -> Self {
        Self {
            version,
            high_res_clock_frequency,
            clock_offset_secs: 0.0,
        }
    }

    /// The same server with an injected software-clock offset (seconds).
    #[must_use]
    pub const fn with_clock_offset_secs(mut self, offset: f64) -> Self {
        self.clock_offset_secs = offset;
        self
    }

    /// Builds a decoded-shape [`FrameOfData`] for `frame_number` at
    /// `true_time_secs` on the true timeline, carrying `bodies`.
    ///
    /// The other sections (marker sets, skeletons, assets, labelled
    /// markers, force plates, devices) are emitted **empty**, matching real
    /// captures of an empty volume; only the rigid-body and timing content
    /// varies. Use [`Self::frame_with_sections`] for non-empty skeletons,
    /// assets, force plates and devices.
    #[must_use]
    pub fn frame(
        &self,
        frame_number: i32,
        true_time_secs: f64,
        bodies: &[PlannedBody],
    ) -> FrameOfData {
        let rigid_bodies = bodies
            .iter()
            .map(|b| RigidBody {
                id: b.id,
                position: b.position,
                orientation: b.orientation,
                mean_marker_error: b.mean_marker_error,
                // Bit 0 is tracking-valid; a dropout clears it.
                params: u16::from(b.tracking_valid),
            })
            .collect();

        // Camera mid-exposure ticks live on the TRUE timeline; the software
        // timestamp carries the injected offset. The two disagree by exactly
        // `clock_offset_secs`, which is what a clock estimator recovers.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let camera_mid_exposure = (true_time_secs * self.high_res_clock_frequency as f64)
            .round()
            .max(0.0) as u64;

        FrameOfData {
            frame_number,
            marker_sets: Vec::new(),
            other_markers: Vec::new(),
            rigid_bodies,
            skeletons: empty_section(),
            assets: empty_section(),
            labeled_markers: Vec::new(),
            force_plates: empty_section(),
            devices: empty_section(),
            timing: FrameTiming {
                timecode: 0,
                timecode_subframe: 0,
                timestamp: true_time_secs + self.clock_offset_secs,
                camera_mid_exposure,
                camera_data_received: camera_mid_exposure,
                transmit: camera_mid_exposure,
                precision_timestamp_secs: 0,
                precision_timestamp_fractional_secs: 0,
            },
            params: 0,
        }
    }

    /// Builds a decoded-shape [`FrameOfData`] like [`Self::frame`], with the
    /// skeleton, asset, force-plate and device sections encoded from
    /// `sections`.
    ///
    /// # Errors
    /// Returns [`Error::LengthOverflow`](crate::Error::LengthOverflow) when a
    /// planned count or section size does not fit its `int32` wire field.
    pub fn frame_with_sections(
        &self,
        frame_number: i32,
        true_time_secs: f64,
        bodies: &[PlannedBody],
        sections: &PlannedSections,
    ) -> crate::Result<FrameOfData> {
        let mut frame = self.frame(frame_number, true_time_secs, bodies);
        frame.skeletons = RawSection::from_skeletons(&sections.skeletons)?;
        frame.assets = RawSection::from_assets(&sections.assets)?;
        frame.force_plates = RawSection::from_force_plates(&sections.force_plates)?;
        frame.devices = RawSection::from_devices(&sections.devices)?;
        Ok(frame)
    }

    /// Serializes [`Self::frame_with_sections`] to a full FrameOfData
    /// datagram at this server's protocol version.
    ///
    /// # Errors
    /// The errors of [`Self::frame_with_sections`] and [`Self::datagram`],
    /// including a refusal of non-empty assets before NatNet 4.1.
    pub fn datagram_with_sections(
        &self,
        frame_number: i32,
        true_time_secs: f64,
        bodies: &[PlannedBody],
        sections: &PlannedSections,
    ) -> crate::Result<Vec<u8>> {
        self.frame_with_sections(frame_number, true_time_secs, bodies, sections)?
            .to_datagram(self.version)
    }

    /// Serializes a MODELDEF describing this server's scene to a full
    /// datagram at this server's protocol version, with the checked encoder
    /// ([`ModelDef::to_datagram_version`]). Any dataset type, including
    /// skeletons, force plates, devices, cameras and assets, may be emitted.
    ///
    /// # Errors
    /// Returns the checked MODELDEF encoding errors, for example a rigid-body
    /// tail that is not a valid marker description for this version.
    pub fn model_def_datagram(&self, model: &ModelDef) -> crate::Result<Vec<u8>> {
        model.to_datagram_version(self.version)
    }

    /// Serializes one synthetic frame to a full FrameOfData **datagram**
    /// (4-byte header + payload) at this server's protocol version — real
    /// wire bytes a NatNet decoder parses.
    ///
    /// # Errors
    /// Refuses unsupported versions and values the checked frame encoder
    /// cannot represent. Non-finite pose values remain wire-representable.
    pub fn datagram(
        &self,
        frame_number: i32,
        true_time_secs: f64,
        bodies: &[PlannedBody],
    ) -> crate::Result<Vec<u8>> {
        self.frame(frame_number, true_time_secs, bodies)
            .to_datagram(self.version)
    }
}

/// An empty structural section (`count == 0`, no bytes).
fn empty_section() -> RawSection {
    RawSection {
        count: 0,
        payload: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server() -> SyntheticServer {
        // 120 Hz-style camera clock; arbitrary high-res tick rate.
        SyntheticServer::new(Version::V4_1, 1_000_000)
    }

    #[test]
    fn generated_datagram_round_trips_through_parse() {
        let s = server();
        let bodies = [
            PlannedBody::tracked(3, [1.0, 2.0, 3.0], [0.0, 0.0, 0.0, 1.0]),
            PlannedBody::dropout(4),
        ];
        let dg = s.datagram(42, 0.35, &bodies).expect("valid frame encodes");
        let parsed = FrameOfData::parse(&dg, Version::V4_1).expect("real wire bytes parse");
        // The serialize path is the same one the decoder round-trips against.
        assert_eq!(parsed, s.frame(42, 0.35, &bodies));
        assert_eq!(parsed.frame_number, 42);
        assert_eq!(parsed.rigid_bodies.len(), 2);
    }

    #[test]
    fn dropout_is_present_but_untracked() {
        let s = server();
        let frame = s.frame(1, 0.0, &[PlannedBody::dropout(3)]);
        let rb = frame.rigid_bodies[0];
        assert_eq!(rb.id, 3, "the body is present in the frame");
        assert!(!rb.tracking_valid(), "but reports tracking-invalid (a gap)");
        assert_eq!(rb.params & 0x01, 0);
    }

    #[test]
    fn tracked_body_sets_tracking_valid_bit() {
        let s = server();
        let frame = s.frame(
            1,
            0.0,
            &[PlannedBody::tracked(3, [0.0; 3], [0.0, 0.0, 0.0, 1.0])],
        );
        assert!(frame.rigid_bodies[0].tracking_valid());
    }

    #[test]
    fn injected_offset_shifts_software_timestamp_but_not_camera_clock() {
        let s = server().with_clock_offset_secs(0.030); // 30 ms, the 3.5 example
        let frame = s.frame(
            10,
            1.0,
            &[PlannedBody::tracked(3, [0.0; 3], [0.0, 0.0, 0.0, 1.0])],
        );
        // Software timestamp carries the offset...
        assert!((frame.timing.timestamp - 1.030).abs() < 1e-9);
        // ...the camera mid-exposure ticks stay on the true timeline (1.0 s).
        assert_eq!(frame.timing.camera_mid_exposure, 1_000_000);
    }

    #[test]
    fn generation_is_deterministic() {
        let s = server().with_clock_offset_secs(0.005);
        let bodies = [PlannedBody::tracked(
            7,
            [0.1, -0.2, 0.3],
            [0.1, 0.2, 0.3, 0.9],
        )];
        assert_eq!(
            s.datagram(5, 0.5, &bodies).expect("valid frame encodes"),
            s.datagram(5, 0.5, &bodies).expect("valid frame encodes")
        );
    }
}
