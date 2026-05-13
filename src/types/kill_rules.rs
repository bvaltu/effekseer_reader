//! Kill rules parameter types.

use super::enums::KillType;
use super::primitives::Vector3D;

/// Kill rules for particle removal based on position.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KillRulesParameter {
    /// Kill rule type.
    pub kill_type: KillType,
    /// Whether scale and rotation are applied to the kill volume.
    pub is_scale_and_rotation_applied: bool,
    /// Type-specific parameters.
    pub params: KillTypeParams,
}

/// Type-specific kill rule parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KillTypeParams {
    /// No kill rule.
    None,
    /// Kill outside/inside a box.
    Box {
        /// Box center.
        center: Vector3D,
        /// Box size.
        size: Vector3D,
        /// If true, kill particles inside the box.
        is_kill_inside: bool,
    },
    /// Kill on one side of a plane.
    Plane {
        /// Plane normal (axis).
        plane_axis: Vector3D,
        /// Plane offset.
        plane_offset: f32,
    },
    /// Kill outside/inside a sphere.
    Sphere {
        /// Sphere center.
        center: Vector3D,
        /// Sphere radius.
        radius: f32,
        /// If true, kill particles inside the sphere.
        is_kill_inside: bool,
    },
}

impl Default for KillTypeParams {
    fn default() -> Self {
        Self::None
    }
}

impl Default for KillRulesParameter {
    fn default() -> Self {
        Self {
            kill_type: KillType::default(),
            is_scale_and_rotation_applied: false,
            params: KillTypeParams::default(),
        }
    }
}
