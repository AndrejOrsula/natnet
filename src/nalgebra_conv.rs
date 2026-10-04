//! One-way `nalgebra` conversions enabled by the `nalgebra` feature; core wire
//! types remain plain arrays. NatNet quaternions are `[x, y, z, w]`, while
//! [`nalgebra::Quaternion::new`] takes `(w, i, j, k)`.

use nalgebra::{Point3, Quaternion, Vector3};

use crate::frame::RigidBody;

/// Rigid-body position as an `nalgebra` point (metres, native frame).
impl From<&RigidBody> for Point3<f32> {
    fn from(rb: &RigidBody) -> Self {
        Point3::new(rb.position[0], rb.position[1], rb.position[2])
    }
}

/// Rigid-body quaternion; NatNet `[x, y, z, w]` maps to `Quaternion::new(w, x, y, z)`.
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
