//! `nalgebra` conversions (`nalgebra` feature) — offered as a convenience for
//! consumers that already use `nalgebra`. The core API stays plain-array
//! regardless; these are additive `From` impls, not a change to the
//! wire types.
//!
//! Quaternion order note: NatNet stores quaternions as `[x, y, z, w]`;
//! [`nalgebra::Quaternion::new`] takes `(w, i, j, k)`. The conversions below
//! respect that ordering so a round trip preserves the rotation.

use nalgebra::{Point3, Quaternion, Vector3};

use crate::frame::RigidBody;

/// The position of a rigid body as an `nalgebra` point (metres, native frame).
impl From<&RigidBody> for Point3<f32> {
    fn from(rb: &RigidBody) -> Self {
        Point3::new(rb.position[0], rb.position[1], rb.position[2])
    }
}

/// The orientation of a rigid body as an `nalgebra` quaternion.
///
/// NatNet `[x, y, z, w]` → `Quaternion::new(w, x, y, z)`.
impl From<&RigidBody> for Quaternion<f32> {
    fn from(rb: &RigidBody) -> Self {
        let [x, y, z, w] = rb.orientation;
        Quaternion::new(w, x, y, z)
    }
}

/// A plain `[f32; 3]` marker position as an `nalgebra` vector.
#[must_use]
pub fn marker_vector(m: [f32; 3]) -> Vector3<f32> {
    Vector3::new(m[0], m[1], m[2])
}
