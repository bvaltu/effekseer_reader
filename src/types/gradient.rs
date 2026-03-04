//! Gradient types for color/alpha interpolation.

/// A color key in a gradient.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GradientColorKey {
    /// Position along the gradient (0.0-1.0).
    pub position: f32,
    /// Red component (linear).
    pub r: f32,
    /// Green component (linear).
    pub g: f32,
    /// Blue component (linear).
    pub b: f32,
    /// HDR intensity multiplier.
    pub intensity: f32,
}

/// An alpha key in a gradient.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GradientAlphaKey {
    /// Position along the gradient (0.0-1.0).
    pub position: f32,
    /// Alpha value.
    pub alpha: f32,
}

/// A gradient with color and alpha keys (up to 8 each).
///
/// Binary format: fixed 232 bytes (all 8 slots always stored).
/// Only the first `color_count`/`alpha_count` keys are meaningful.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Gradient {
    /// Color keys (up to 8).
    pub colors: Vec<GradientColorKey>,
    /// Alpha keys (up to 8).
    pub alphas: Vec<GradientAlphaKey>,
}
