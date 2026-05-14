//! Translation, rotation, scaling, and spawn location parameter types.

use super::enums::{
    CircleDistributionType, LineDistributionType, ModelCoordinateSpace, ModelReferenceType,
    ModelSpawnType, NurbsCurveLoopType, SpawnAxisType,
};
use super::fcurve::{FCurveScalar, FCurveVector3D};
use super::primitives::{RandomFloat, RandomVector3D, RefMinMax, Vector3D};

// ============================================================
// ParameterEasing types (newer system, version >= 1600)
// ============================================================

use super::enums::Easing3Type;

/// Middle point data for easing parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MiddlePointFloat {
    /// Dynamic equation reference for middle.
    pub ref_eq_m: RefMinMax,
    /// Middle value range.
    pub middle: RandomFloat,
}

/// Middle point data for easing vec3 parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MiddlePointVector3D {
    /// Dynamic equation reference for middle.
    pub ref_eq_m: RefMinMax,
    /// Middle value range.
    pub middle: RandomVector3D,
}

/// New-style easing for a single float value (ElemNum=1).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterEasingFloat {
    /// Dynamic equation references for start.
    pub ref_eq_s: RefMinMax,
    /// Dynamic equation references for end.
    pub ref_eq_e: RefMinMax,
    /// Start value range.
    pub start: RandomFloat,
    /// End value range.
    pub end: RandomFloat,
    /// Middle point (version >= minAppendParameterVersion, enabled).
    pub middle: Option<MiddlePointFloat>,
    /// Easing type.
    pub easing_type: Easing3Type,
    /// Easing parameters (only for StartEndSpeed).
    pub easing_params: Option<[f32; 3]>,
    /// Channel configuration (packed).
    pub channel: i32,
    /// Individual per-component easing types (only if isIndividual != 0).
    pub individual_types: Option<Vec<Easing3Type>>,
}

/// New-style easing for a 3D vector (ElemNum=3).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterEasingVector3D {
    /// Dynamic equation references for start.
    pub ref_eq_s: RefMinMax,
    /// Dynamic equation references for end.
    pub ref_eq_e: RefMinMax,
    /// Start value range.
    pub start: RandomVector3D,
    /// End value range.
    pub end: RandomVector3D,
    /// Middle point (version >= minAppendParameterVersion, enabled).
    pub middle: Option<MiddlePointVector3D>,
    /// Easing type.
    pub easing_type: Easing3Type,
    /// Easing parameters (only for StartEndSpeed).
    pub easing_params: Option<[f32; 3]>,
    /// Channel configuration (packed).
    pub channel: i32,
    /// Individual per-component easing types (only if isIndividual != 0).
    pub individual_types: Option<Vec<Easing3Type>>,
}

// ============================================================
// Translation
// ============================================================

/// Translation parameter (variant based on [`ParameterTranslationType`](super::enums::ParameterTranslationType)).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TranslationParameter {
    /// No translation.
    None,
    /// Fixed position.
    Fixed {
        /// Dynamic equation reference.
        ref_eq: i32,
        /// Position value.
        position: Vector3D,
    },
    /// Position-Velocity-Acceleration.
    Pva {
        /// Dynamic equation refs for position.
        ref_eq_p: RefMinMax,
        /// Dynamic equation refs for velocity.
        ref_eq_v: RefMinMax,
        /// Dynamic equation refs for acceleration.
        ref_eq_a: RefMinMax,
        /// Position range.
        position: RandomVector3D,
        /// Velocity range.
        velocity: RandomVector3D,
        /// Acceleration range.
        acceleration: RandomVector3D,
    },
    /// Easing interpolation.
    Easing(Box<ParameterEasingVector3D>),
    /// F-Curve animation.
    FCurve(Box<FCurveVector3D>),
    /// NURBS curve path (version >= 1607).
    NurbsCurve {
        /// Curve resource index.
        index: i32,
        /// Scale factor.
        scale: f32,
        /// Movement speed.
        move_speed: f32,
        /// Loop behavior.
        loop_type: NurbsCurveLoopType,
    },
    /// View-space offset (version >= 1704).
    ViewOffset {
        /// Distance offset.
        distance: RandomFloat,
    },
}

// ============================================================
// Rotation
// ============================================================

/// Rotation parameter (variant based on [`ParameterRotationType`](super::enums::ParameterRotationType)).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RotationParameter {
    /// No rotation.
    None,
    /// Fixed rotation.
    Fixed {
        /// Dynamic equation reference.
        ref_eq: i32,
        /// Rotation value (euler angles).
        rotation: Vector3D,
    },
    /// Position-Velocity-Acceleration.
    Pva {
        /// Dynamic equation refs for rotation.
        ref_eq_p: RefMinMax,
        /// Dynamic equation refs for velocity.
        ref_eq_v: RefMinMax,
        /// Dynamic equation refs for acceleration.
        ref_eq_a: RefMinMax,
        /// Rotation range.
        rotation: RandomVector3D,
        /// Angular velocity range.
        velocity: RandomVector3D,
        /// Angular acceleration range.
        acceleration: RandomVector3D,
    },
    /// Easing interpolation.
    Easing(Box<ParameterEasingVector3D>),
    /// Axis-aligned PVA.
    AxisPva {
        /// Random axis direction.
        axis: RandomVector3D,
        /// Rotation angle range.
        rotation: RandomFloat,
        /// Angular velocity range.
        velocity: RandomFloat,
        /// Angular acceleration range.
        acceleration: RandomFloat,
    },
    /// Axis-aligned easing.
    AxisEasing {
        /// Random axis direction.
        axis: RandomVector3D,
        /// Easing parameters for the angle.
        easing: ParameterEasingFloat,
    },
    /// F-Curve animation.
    FCurve(Box<FCurveVector3D>),
    /// Rotate to face viewpoint (no data).
    RotateToViewpoint,
    /// Velocity-based rotation. Axis selects which local axis aligns with the
    /// instantaneous velocity vector. Mirrors C++ `RotationVelocity.axis`
    /// (`Effekseer.Rotation.cpp:98` reads it from disk;
    /// `Effekseer.Rotation.cpp:381-405` consumes it during rendering).
    Velocity {
        /// Axis to align with velocity.
        axis: super::enums::DirectionalAxisType,
    },
}

// ============================================================
// Scaling
// ============================================================

/// Scaling parameter (variant based on [`ParameterScalingType`](super::enums::ParameterScalingType)).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScalingParameter {
    /// No scaling.
    None,
    /// Fixed scale.
    Fixed {
        /// Dynamic equation reference.
        ref_eq: i32,
        /// Scale value.
        scale: Vector3D,
    },
    /// Position-Velocity-Acceleration.
    Pva {
        /// Dynamic equation refs for scale.
        ref_eq_p: RefMinMax,
        /// Dynamic equation refs for velocity.
        ref_eq_v: RefMinMax,
        /// Dynamic equation refs for acceleration.
        ref_eq_a: RefMinMax,
        /// Scale range.
        scale: RandomVector3D,
        /// Velocity range.
        velocity: RandomVector3D,
        /// Acceleration range.
        acceleration: RandomVector3D,
    },
    /// Easing interpolation (3D).
    Easing(Box<ParameterEasingVector3D>),
    /// Single-axis PVA (uniform scale).
    SinglePva {
        /// Uniform scale range.
        scale: RandomFloat,
        /// Velocity range.
        velocity: RandomFloat,
        /// Acceleration range.
        acceleration: RandomFloat,
    },
    /// Single-axis easing.
    SingleEasing(Box<ParameterEasingFloat>),
    /// F-Curve animation (3D).
    FCurve(Box<FCurveVector3D>),
    /// Single-axis F-Curve (uniform scale).
    SingleFCurve(Box<FCurveScalar>),
}

// ============================================================
// Spawn Location
// ============================================================

/// Spawn location parameter (variant based on [`GenerationLocationType`](super::enums::GenerationLocationType)).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SpawnLocationParameter {
    /// Spawn at a point.
    Point {
        /// Random spawn offset.
        location: RandomVector3D,
    },
    /// Spawn on a sphere surface.
    Sphere {
        /// Radius range.
        radius: RandomFloat,
        /// Rotation around X axis.
        rotation_x: RandomFloat,
        /// Rotation around Y axis.
        rotation_y: RandomFloat,
    },
    /// Spawn on a model surface.
    Model {
        /// Reference type (File, Procedural, External).
        reference: ModelReferenceType,
        /// Model/procedural model index.
        index: i32,
        /// Spawn distribution type.
        spawn_type: ModelSpawnType,
        /// Coordinate space (version >= 1802).
        coordinate: Option<ModelCoordinateSpace>,
    },
    /// Spawn on a circle.
    Circle {
        /// Number of divisions.
        division: i32,
        /// Radius range.
        radius: RandomFloat,
        /// Start angle range.
        angle_start: RandomFloat,
        /// End angle range.
        angle_end: RandomFloat,
        /// Distribution type.
        distribution_type: CircleDistributionType,
        /// Axis direction.
        axis_direction: SpawnAxisType,
        /// Angle noise.
        angle_noise: RandomFloat,
    },
    /// Spawn along a line.
    Line {
        /// Number of divisions.
        division: i32,
        /// Start position range.
        position_start: RandomVector3D,
        /// End position range.
        position_end: RandomVector3D,
        /// Position noise.
        position_noise: RandomFloat,
        /// Distribution type.
        distribution_type: LineDistributionType,
    },
}

/// Steering behavior parameters (for FollowParent translation bind types).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SteeringBehaviorParam {
    /// Maximum follow speed range.
    pub max_follow_speed: RandomFloat,
    /// Steering speed range.
    pub steering_speed: RandomFloat,
}

/// LOD (Level of Detail) parameters.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LODsParam {
    /// Bitmask of which LOD levels this node is enabled for.
    pub matching_lods: i32,
    /// Behavior when node doesn't match active LOD.
    pub lod_behaviour: super::enums::NonMatchingLODBehaviour,
}
