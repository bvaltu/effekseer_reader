//! Primitive and composite data types used in Effekseer binary formats.

use super::enums::{ColorMode, TriggerType};

/// 2D vector (x, y).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vector2D {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
}

/// 3D vector (x, y, z).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vector3D {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
    /// Z component.
    pub z: f32,
}

/// Rectangle (x, y, width, height).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rectf {
    /// X position.
    pub x: f32,
    /// Y position.
    pub y: f32,
    /// Width.
    pub w: f32,
    /// Height.
    pub h: f32,
}

/// RGBA color (8 bits per channel).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Color {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha channel.
    pub a: u8,
}

/// Random float range. Note: binary layout is max *before* min.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RandomFloat {
    /// Maximum value.
    pub max: f32,
    /// Minimum value.
    pub min: f32,
}

/// Random integer range. Note: binary layout is max *before* min.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RandomInt {
    /// Maximum value.
    pub max: i32,
    /// Minimum value.
    pub min: i32,
}

/// Random 2D vector range.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RandomVector2D {
    /// Maximum value.
    pub max: Vector2D,
    /// Minimum value.
    pub min: Vector2D,
}

/// Random 3D vector range.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RandomVector3D {
    /// Maximum value.
    pub max: Vector3D,
    /// Minimum value.
    pub min: Vector3D,
}

/// Random color range (10 bytes on disk: mode u8, pad u8, max Color, min Color).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RandomColor {
    /// Color space mode (RGBA or HSVA).
    pub mode: ColorMode,
    /// Maximum color value.
    pub max: Color,
    /// Minimum color value.
    pub min: Color,
}

/// Reference min/max pair for dynamic equation indices (both default to -1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RefMinMax {
    /// Maximum reference index.
    pub max: i32,
    /// Minimum reference index.
    pub min: i32,
}

impl Default for RefMinMax {
    fn default() -> Self {
        Self { max: -1, min: -1 }
    }
}

/// Trigger configuration (type and index).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TriggerValues {
    /// Trigger type.
    pub type_: TriggerType,
    /// Trigger index.
    pub index: u8,
}

/// Easing parameters without random start/end (3-parameter cubic: a*t³ + b*t² + c*t).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EasingFloatWithoutRandom {
    /// Cubic coefficient.
    pub a: f32,
    /// Quadratic coefficient.
    pub b: f32,
    /// Linear coefficient.
    pub c: f32,
}

/// Easing float with random start/end ranges and cubic parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EasingFloat {
    /// Start value range.
    pub start: RandomFloat,
    /// End value range.
    pub end: RandomFloat,
    /// Cubic coefficient.
    pub a: f32,
    /// Quadratic coefficient.
    pub b: f32,
    /// Linear coefficient.
    pub c: f32,
}

/// Easing 2D vector with random start/end ranges.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EasingVector2D {
    /// Start value range.
    pub start: RandomVector2D,
    /// End value range.
    pub end: RandomVector2D,
    /// Cubic coefficient.
    pub a: f32,
    /// Quadratic coefficient.
    pub b: f32,
    /// Linear coefficient.
    pub c: f32,
}

/// Easing 3D vector with random start/end ranges.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EasingVector3D {
    /// Start value range.
    pub start: RandomVector3D,
    /// End value range.
    pub end: RandomVector3D,
    /// Cubic coefficient.
    pub a: f32,
    /// Quadratic coefficient.
    pub b: f32,
    /// Linear coefficient.
    pub c: f32,
}

/// Easing color with random start/end ranges.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EasingColor {
    /// Start color range.
    pub start: RandomColor,
    /// End color range.
    pub end: RandomColor,
    /// Cubic coefficient.
    pub a: f32,
    /// Quadratic coefficient.
    pub b: f32,
    /// Linear coefficient.
    pub c: f32,
}

// ============================================================
// Constructors — Color
// ============================================================

impl Color {
    /// Opaque white.
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    /// Opaque black.
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    /// Opaque magenta — the canonical "missing color" sentinel.
    pub const MAGENTA: Self = Self { r: 255, g: 0, b: 255, a: 255 };
    /// Fully transparent.
    pub const TRANSPARENT: Self = Self { r: 0, g: 0, b: 0, a: 0 };

    /// Construct from explicit RGBA channels.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Construct from RGB channels with full alpha.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}

// ============================================================
// Defaults — primitive ranges
// ============================================================

impl Default for RandomFloat {
    fn default() -> Self {
        Self { max: 0.0, min: 0.0 }
    }
}

impl Default for RandomInt {
    fn default() -> Self {
        Self { max: 0, min: 0 }
    }
}

impl Default for TriggerValues {
    fn default() -> Self {
        Self {
            type_: TriggerType::default(),
            index: 0,
        }
    }
}

// ============================================================
// Constructors — RandomVector3D
// ============================================================

impl RandomVector3D {
    /// Both min and max are the zero vector — "no range, value is always zero."
    pub const fn zero() -> Self {
        Self {
            max: Vector3D { x: 0.0, y: 0.0, z: 0.0 },
            min: Vector3D { x: 0.0, y: 0.0, z: 0.0 },
        }
    }

    /// Explicit min and max corners of the random box.
    pub const fn range(min: Vector3D, max: Vector3D) -> Self {
        Self { max, min }
    }

    /// Symmetric box around the origin: `±half_extent` on each axis.
    pub const fn symmetric(half_extent: Vector3D) -> Self {
        Self {
            max: half_extent,
            min: Vector3D {
                x: -half_extent.x,
                y: -half_extent.y,
                z: -half_extent.z,
            },
        }
    }

    /// Symmetric box with the same magnitude on x, y, and z: `±scalar` on each axis.
    pub const fn symmetric_uniform(scalar: f32) -> Self {
        Self::symmetric(Vector3D { x: scalar, y: scalar, z: scalar })
    }
}
