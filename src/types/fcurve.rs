//! F-Curve types for animation curves.

use super::enums::FCurveEdge;

/// A single pre-sampled animation curve.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FCurve {
    /// Behavior before curve start.
    pub start_edge: FCurveEdge,
    /// Behavior after curve end.
    pub end_edge: FCurveEdge,
    /// Random offset maximum.
    pub offset_max: f32,
    /// Random offset minimum.
    pub offset_min: f32,
    /// Frame offset for curve start.
    pub offset: i32,
    /// Curve length in frames.
    pub len: i32,
    /// Sampling frequency (frames between samples).
    pub freq: i32,
    /// Uniformly-sampled key values.
    pub keys: Vec<f32>,
}

/// F-Curve for a single float value.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FCurveScalar {
    /// Timeline type (Time or Percent). Only present for version >= 1600.
    pub timeline: i32,
    /// The curve data.
    pub s: FCurve,
}

/// F-Curve for a 2D vector.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FCurveVector2D {
    /// Timeline type (Time or Percent).
    pub timeline: i32,
    /// X component curve.
    pub x: FCurve,
    /// Y component curve.
    pub y: FCurve,
}

/// F-Curve for a 3D vector.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FCurveVector3D {
    /// Timeline type (Time or Percent).
    pub timeline: i32,
    /// X component curve.
    pub x: FCurve,
    /// Y component curve.
    pub y: FCurve,
    /// Z component curve.
    pub z: FCurve,
}

/// F-Curve for a color value (RGBA).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FCurveVectorColor {
    /// Timeline type (Time or Percent).
    pub timeline: i32,
    /// Red component curve.
    pub r: FCurve,
    /// Green component curve.
    pub g: FCurve,
    /// Blue component curve.
    pub b: FCurve,
    /// Alpha component curve.
    pub a: FCurve,
}
