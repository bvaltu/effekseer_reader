//! Alpha cutoff parameter types.

use super::fcurve::FCurveScalar;
use super::params::ParameterEasingFloat;
use super::primitives::{Color, RandomFloat, RandomInt};

/// Alpha cutoff (alpha test) parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AlphaCutoffParameter {
    /// The alpha cutoff variant (None if flag was 0 or version < 1605).
    pub cutoff: Option<AlphaCutoffVariant>,
    /// Edge threshold (after buffer data).
    pub edge_threshold: f32,
    /// Edge color.
    pub edge_color: Color,
    /// Edge color scaling.
    pub edge_color_scaling: f32,
}

/// Alpha cutoff variant based on type.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AlphaCutoffVariant {
    /// Fixed threshold.
    Fixed {
        /// Dynamic equation reference.
        ref_eq: i32,
        /// Threshold value.
        threshold: f32,
    },
    /// Four-point interpolation.
    FourPointInterpolation {
        /// Begin threshold range.
        begin_threshold: RandomFloat,
        /// Transition frame count range.
        transition_frame_num: RandomInt,
        /// Second threshold range.
        no2_threshold: RandomFloat,
        /// Third threshold range.
        no3_threshold: RandomFloat,
        /// Second transition frame count range.
        transition_frame_num2: RandomInt,
        /// End threshold range.
        end_threshold: RandomFloat,
    },
    /// Easing.
    Easing(Box<ParameterEasingFloat>),
    /// F-Curve.
    FCurve(Box<FCurveScalar>),
}

impl Default for AlphaCutoffParameter {
    fn default() -> Self {
        Self {
            cutoff: None,
            edge_threshold: 0.0,
            edge_color: Color {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            edge_color_scaling: 0.0,
        }
    }
}
