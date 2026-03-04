//! Force field types for local force field parameters.

use super::enums::{
    ForceFieldTurbulenceType, ForceFieldVortexType, LocalForceFieldFalloffType, LocalForceFieldType,
};
use super::primitives::Vector3D;

/// A single local force field element.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LocalForceFieldElement {
    /// Force field type.
    pub field_type: LocalForceFieldType,
    /// Power/strength of the force.
    pub power: f32,
    /// Position offset.
    pub position: Vector3D,
    /// Rotation (euler angles).
    pub rotation: Vector3D,
    /// Type-specific parameters.
    pub type_params: ForceFieldTypeParams,
    /// Falloff parameters (version >= 1600).
    pub falloff: Option<ForceFieldFalloff>,
}

/// Type-specific force field parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ForceFieldTypeParams {
    /// No force field.
    None,
    /// Directional force.
    Force {
        /// If true, force acts toward center.
        gravitation: bool,
    },
    /// Wind force (no extra data).
    Wind,
    /// Vortex force.
    Vortex {
        /// Vortex sub-type (version >= 1601).
        vortex_type: Option<ForceFieldVortexType>,
    },
    /// Turbulence (noise-based) force.
    Turbulence {
        /// Turbulence sub-type.
        turbulence_type: ForceFieldTurbulenceType,
        /// Random seed.
        seed: i32,
        /// Noise scale.
        scale: f32,
        /// Noise strength.
        strength: f32,
        /// Noise octave count.
        octave: i32,
    },
    /// Drag force (no extra data).
    Drag,
    /// Gravity force.
    Gravity {
        /// Gravity vector.
        gravity: Vector3D,
    },
    /// Attractive force toward a point.
    AttractiveForce {
        /// Force magnitude.
        force: f32,
        /// Control parameter.
        control: f32,
        /// Minimum range.
        min_range: f32,
        /// Maximum range.
        max_range: f32,
    },
}

/// Force field falloff parameters (version >= 1600).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForceFieldFalloff {
    /// Falloff shape type.
    pub falloff_type: LocalForceFieldFalloffType,
    /// Power exponent.
    pub power: f32,
    /// Maximum distance.
    pub max_distance: f32,
    /// Minimum distance.
    pub min_distance: f32,
    /// Tube-specific parameters.
    pub tube: Option<TubeFalloff>,
    /// Cone-specific parameters.
    pub cone: Option<ConeFalloff>,
}

/// Tube falloff parameters.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TubeFalloff {
    /// Radius power.
    pub radius_power: f32,
    /// Maximum radius.
    pub max_radius: f32,
    /// Minimum radius.
    pub min_radius: f32,
}

/// Cone falloff parameters.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConeFalloff {
    /// Angle power.
    pub angle_power: f32,
    /// Maximum angle.
    pub max_angle: f32,
    /// Minimum angle.
    pub min_angle: f32,
}
