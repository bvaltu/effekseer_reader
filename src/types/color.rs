//! All-type color parameter.

use super::fcurve::FCurveVectorColor;
use super::gradient::Gradient;
use super::primitives::{Color, EasingColor, RandomColor};

/// Color parameter (variant based on [`AllTypeColorType`](super::enums::AllTypeColorType)).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AllTypeColorParameter {
    /// Fixed color.
    Fixed {
        /// The color value.
        all: Color,
    },
    /// Random color range.
    Random {
        /// The random color range.
        all: RandomColor,
    },
    /// Easing between colors (old 3-param cubic).
    Easing(EasingColor),
    /// F-Curve RGBA animation.
    FCurveRgba(Box<FCurveVectorColor>),
    /// Gradient over lifetime.
    Gradient(Box<Gradient>),
}

impl AllTypeColorParameter {
    /// Constant color applied to every particle.
    pub fn fixed(color: Color) -> Self {
        Self::Fixed { all: color }
    }

    /// Per-particle random color drawn from the given range.
    pub fn random(range: RandomColor) -> Self {
        Self::Random { all: range }
    }
}
