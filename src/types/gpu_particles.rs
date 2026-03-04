//! GPU particle parameter types.

use super::enums::BindType;
use super::enums::{
    AlphaBlendType, GpuColorParamType, GpuColorSpaceType, GpuEmitShape, GpuMaterialType,
    GpuRenderShape, GpuScaleType, TextureFilterType, TextureWrapType,
};
use super::fcurve::FCurveVectorColor;
use super::gradient::Gradient;
use super::primitives::Color;

/// GPU particle system parameters (version >= 1800).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GpuParticlesParameter {
    /// Basic emission parameters.
    pub basic: GpuBasicParams,
    /// Emission shape configuration.
    pub emit_shape: GpuEmitShapeParams,
    /// Velocity parameters.
    pub velocity: GpuVelocityParams,
    /// Rotation parameters (stored as degrees, converted to radians by consumer).
    pub rotation: GpuRotationParams,
    /// Scale parameters.
    pub scale: GpuScaleParams,
    /// Force parameters (gravity, vortex, turbulence).
    pub force: GpuForceParams,
    /// Render basic settings (blend, z-write, z-test).
    pub render_basic: GpuRenderBasicParams,
    /// Render shape settings.
    pub render_shape: GpuRenderShapeParams,
    /// Render color settings.
    pub render_color: GpuRenderColorParams,
    /// Render material settings.
    pub render_material: GpuRenderMaterialParams,
}

/// Basic GPU particle emission parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GpuBasicParams {
    /// Total emit count.
    pub emit_count: i32,
    /// Particles emitted per frame.
    pub emit_per_frame: i32,
    /// Emission time offset.
    pub emit_offset: f32,
    /// Lifetime range [min, max].
    pub life_time: [f32; 2],
}

/// GPU particle emission shape parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GpuEmitShapeParams {
    /// Emission shape type.
    pub shape_type: GpuEmitShape,
    /// Whether rotation is applied to emission.
    pub rotation_applied: bool,
    /// Shape-specific data.
    pub data: GpuEmitShapeData,
}

/// Shape-specific emission data.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GpuEmitShapeData {
    /// Point emission (no data).
    Point,
    /// Line emission.
    Line {
        /// Start position.
        start: [f32; 3],
        /// End position.
        end: [f32; 3],
        /// Line width.
        width: f32,
    },
    /// Circle emission.
    Circle {
        /// Circle axis direction.
        axis: [f32; 3],
        /// Inner radius.
        inner: f32,
        /// Outer radius.
        outer: f32,
    },
    /// Sphere emission.
    Sphere {
        /// Sphere radius.
        radius: f32,
    },
    /// Model surface emission.
    Model {
        /// Model resource index.
        index: i32,
        /// Model size scale.
        size: f32,
    },
}

/// GPU particle velocity parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GpuVelocityParams {
    /// Emission direction.
    pub direction: [f32; 3],
    /// Spread angle.
    pub spread: f32,
    /// Initial speed range [min, max].
    pub initial_speed: [f32; 2],
    /// Damping range [min, max].
    pub damping: [f32; 2],
}

/// GPU particle rotation parameters (degrees in binary).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GpuRotationParams {
    /// Rotation offset [min, max] (each xyz).
    pub offset: [[f32; 3]; 2],
    /// Rotation velocity [min, max] (each xyz).
    pub velocity: [[f32; 3]; 2],
}

/// GPU particle scale parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GpuScaleParams {
    /// Scale type.
    pub scale_type: GpuScaleType,
    /// Scale data (varies by type).
    pub data: GpuScaleData,
}

/// Scale-type-specific data.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GpuScaleData {
    /// Fixed scale.
    Fixed {
        /// Scale values (ReadScale4 format).
        scale: GpuScale4,
    },
    /// Easing scale.
    Easing {
        /// Start scale.
        start: GpuScale4,
        /// End scale.
        end: GpuScale4,
        /// Easing speed [x, y, z].
        speed: [f32; 3],
    },
}

/// GPU scale data in ReadScale4 format (32 bytes).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GpuScale4 {
    /// Uniform scale min.
    pub uniform_min: f32,
    /// Uniform scale max.
    pub uniform_max: f32,
    /// Per-axis scale min [x, y, z].
    pub axis_min: [f32; 3],
    /// Per-axis scale max [x, y, z].
    pub axis_max: [f32; 3],
}

/// GPU particle force parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GpuForceParams {
    /// Gravity vector.
    pub gravity: [f32; 3],
    /// Vortex rotation speed.
    pub vortex_rotation: f32,
    /// Vortex attraction strength.
    pub vortex_attraction: f32,
    /// Vortex center position.
    pub vortex_center: [f32; 3],
    /// Vortex axis direction.
    pub vortex_axis: [f32; 3],
    /// Turbulence power.
    pub turbulence_power: f32,
    /// Turbulence random seed.
    pub turbulence_seed: i32,
    /// Turbulence scale.
    pub turbulence_scale: f32,
    /// Turbulence octave count.
    pub turbulence_octave: i32,
}

/// GPU particle render basic settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GpuRenderBasicParams {
    /// Alpha blending mode.
    pub blend_type: AlphaBlendType,
    /// Z-write enabled.
    pub z_write: bool,
    /// Z-test enabled.
    pub z_test: bool,
}

/// GPU particle render shape parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GpuRenderShapeParams {
    /// Render shape type.
    pub shape_type: GpuRenderShape,
    /// Shape-specific data (model index, trail data, or unused for sprite).
    pub data: u32,
    /// Render size.
    pub size: f32,
}

/// GPU particle render color parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GpuRenderColorParams {
    /// Color inheritance binding type.
    pub color_inherit: BindType,
    /// Color parameter type.
    pub color_all_type: GpuColorParamType,
    /// Color space.
    pub color_space: GpuColorSpaceType,
    /// Color data (varies by type).
    pub data: GpuColorData,
    /// Emissive multiplier.
    pub emissive: f32,
    /// Fade-in time.
    pub fade_in: f32,
    /// Fade-out time.
    pub fade_out: f32,
}

/// Color-type-specific data.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GpuColorData {
    /// Fixed color.
    Fixed(Color),
    /// Random color range [min, max].
    Random(Color, Color),
    /// Easing between start and end colors.
    Easing {
        /// Start color range [min, max].
        start: [Color; 2],
        /// End color range [min, max].
        end: [Color; 2],
        /// Easing speed [r, g, b].
        speed: [f32; 3],
    },
    /// F-Curve color animation.
    FCurve(Box<FCurveVectorColor>),
    /// Gradient color.
    Gradient(Gradient),
}

/// GPU particle render material parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GpuRenderMaterialParams {
    /// Material type.
    pub material: GpuMaterialType,
    /// Texture resource indices.
    pub texture_indexes: [u32; 4],
    /// Texture filter modes.
    pub texture_filters: [TextureFilterType; 4],
    /// Texture wrap modes.
    pub texture_wraps: [TextureWrapType; 4],
}
