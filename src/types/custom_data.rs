//! Custom data parameter types.

use super::fcurve::{FCurveVector2D, FCurveVectorColor};
use super::primitives::{EasingVector2D, RandomVector2D, Vector2D};

/// Custom data attached to particles.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParameterCustomData {
    /// No custom data.
    None,
    /// Fixed 2D value.
    Fixed2D(Vector2D),
    /// Random 2D value.
    Random2D(RandomVector2D),
    /// Easing 2D value (old-style, 44 bytes).
    Easing2D(EasingVector2D),
    /// F-Curve 2D value.
    FCurve2D(Box<FCurveVector2D>),
    /// Fixed 4D value.
    Fixed4D([f32; 4]),
    /// F-Curve color value.
    FCurveColor(Box<FCurveVectorColor>),
    /// Dynamic input (reads zero bytes from binary; values sourced at runtime).
    DynamicInput,
}

impl Default for ParameterCustomData {
    fn default() -> Self {
        Self::None
    }
}
