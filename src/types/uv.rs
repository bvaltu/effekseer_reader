//! UV parameter types.

use super::enums::{UVAnimationInterpolationType, UVAnimationLoopType};
use super::fcurve::FCurveVector2D;
use super::primitives::{RandomInt, RandomVector2D, Rectf};

/// UV animation/mapping parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UVParameter {
    /// Default UV mapping (no data).
    Default,
    /// Fixed UV coordinates.
    Fixed {
        /// UV rectangle.
        position: Rectf,
    },
    /// Animated UV frames.
    Animation {
        /// Initial UV rectangle.
        position: Rectf,
        /// Length of each frame.
        frame_length: i32,
        /// Number of horizontal frames.
        frame_count_x: i32,
        /// Number of vertical frames.
        frame_count_y: i32,
        /// Loop behavior.
        loop_type: UVAnimationLoopType,
        /// Start frame (random range).
        start_frame: RandomInt,
        /// Interpolation type (version >= 1600, uvIndex == 0 only).
        interpolation_type: Option<UVAnimationInterpolationType>,
    },
    /// Scrolling UV.
    Scroll {
        /// Position range.
        position: RandomVector2D,
        /// Size range.
        size: RandomVector2D,
        /// Scroll speed range.
        speed: RandomVector2D,
    },
    /// F-Curve UV animation.
    FCurve {
        /// Position curve.
        position: Box<FCurveVector2D>,
        /// Size curve.
        size: Box<FCurveVector2D>,
    },
}
