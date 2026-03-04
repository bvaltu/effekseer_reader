//! NURBS curve types.

/// Double-precision 4D vector (x, y, z, w) — 32 bytes.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DVector4 {
    /// X component.
    pub x: f64,
    /// Y component.
    pub y: f64,
    /// Z component.
    pub z: f64,
    /// W component (weight for NURBS).
    pub w: f64,
}

/// A NURBS curve with f64 control points.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NurbsCurve {
    /// Converter version from the file.
    pub converter_version: i32,
    /// Control points (double-precision 4D vectors).
    pub control_points: Vec<DVector4>,
    /// Knot vector (double-precision).
    pub knots: Vec<f64>,
    /// B-spline order (degree + 1).
    pub order: i32,
    /// Evaluation step size.
    pub step: f64,
    /// Curve type (read but unused in C++ runtime).
    pub curve_type: i32,
    /// Dimension (read but unused in C++ runtime).
    pub dimension: i32,
    /// Computed length: sum of Euclidean distances between consecutive control points (xyz only).
    pub length: f64,
}
