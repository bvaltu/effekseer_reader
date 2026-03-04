//! Renderer parameter types (Sprite, Ribbon, Ring, Model, Track).

use super::color::AllTypeColorParameter;
use super::enums::{
    BillboardType, CullingType, ModelReferenceType, RenderingOrder, TextureUVType,
    TrailSmoothingType, TrailTimeType,
};
use super::node::FalloffParameter;
use super::params::ParameterEasingFloat;
use super::primitives::{Color, EasingVector2D, RandomVector2D, Vector2D};

// ============================================================
// Sprite
// ============================================================

/// Sprite renderer parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpriteParams {
    /// Rendering order.
    pub rendering_order: RenderingOrder,
    /// Billboard orientation.
    pub billboard: BillboardType,
    /// Color for all particles.
    pub all_color: AllTypeColorParameter,
    /// Per-corner color.
    pub sprite_color: SpriteColorParameter,
    /// Per-corner position.
    pub sprite_position: SpritePositionParameter,
}

/// Sprite per-corner color parameter.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SpriteColorParameter {
    /// Default (no per-corner color).
    Default,
    /// Fixed per-corner colors.
    Fixed {
        /// Lower-left corner color.
        ll: Color,
        /// Lower-right corner color.
        lr: Color,
        /// Upper-left corner color.
        ul: Color,
        /// Upper-right corner color.
        ur: Color,
    },
}

/// Sprite per-corner position parameter.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SpritePositionParameter {
    /// Fixed positions (always present in binary).
    Fixed {
        /// Lower-left corner position.
        ll: Vector2D,
        /// Lower-right corner position.
        lr: Vector2D,
        /// Upper-left corner position.
        ul: Vector2D,
        /// Upper-right corner position.
        ur: Vector2D,
    },
}

// ============================================================
// Ribbon
// ============================================================

/// Shared UV type parameters between Ribbon and Track.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeRendererTextureUVTypeParameter {
    /// UV type.
    pub uv_type: TextureUVType,
    /// Tile length.
    pub tile_length: f32,
    /// Head tile edge.
    pub tile_edge_head: i32,
    /// Tail tile edge.
    pub tile_edge_tail: i32,
    /// Loop area begin.
    pub tile_loop_area_begin: f32,
    /// Loop area end.
    pub tile_loop_area_end: f32,
}

/// Ribbon renderer parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RibbonParams {
    /// Texture UV type parameters.
    pub texture_uv_type: NodeRendererTextureUVTypeParameter,
    /// Trail time type (version >= 1700).
    pub trail_time_type: Option<TrailTimeType>,
    /// Whether ribbon faces the camera.
    pub viewpoint_dependent: bool,
    /// Color for all particles.
    pub all_color: AllTypeColorParameter,
    /// Per-edge color.
    pub ribbon_color: RibbonColorParameter,
    /// Per-edge position.
    pub ribbon_position: RibbonPositionParameter,
    /// Spline subdivision count.
    pub spline_division: i32,
}

/// Ribbon per-edge color parameter.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RibbonColorParameter {
    /// Default (no per-edge color).
    Default,
    /// Fixed per-edge colors.
    Fixed {
        /// Left edge color.
        l: Color,
        /// Right edge color.
        r: Color,
    },
}

/// Ribbon per-edge position parameter.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RibbonPositionParameter {
    /// Fixed positions (always present in binary).
    Fixed {
        /// Left edge offset.
        l: f32,
        /// Right edge offset.
        r: f32,
    },
}

// ============================================================
// Ring
// ============================================================

/// Ring renderer parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RingParams {
    /// Rendering order.
    pub rendering_order: RenderingOrder,
    /// Billboard orientation.
    pub billboard: BillboardType,
    /// Shape parameters.
    pub shape: RingShapeParameter,
    /// Number of ring segments.
    pub vertex_count: i32,
    /// Viewing angle (legacy, values discarded but read for cursor).
    pub viewing_angle: RingSingleParameter,
    /// Outer location.
    pub outer_location: RingLocationParameter,
    /// Inner location.
    pub inner_location: RingLocationParameter,
    /// Center ratio.
    pub center_ratio: RingSingleParameter,
    /// Outer color.
    pub outer_color: AllTypeColorParameter,
    /// Center color.
    pub center_color: AllTypeColorParameter,
    /// Inner color.
    pub inner_color: AllTypeColorParameter,
}

/// Ring shape parameter.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RingShapeParameter {
    /// Donut shape.
    Donut,
    /// Crescent shape.
    Crescent {
        /// Starting fade.
        starting_fade: f32,
        /// Ending fade.
        ending_fade: f32,
        /// Starting angle.
        starting_angle: RingSingleParameter,
        /// Ending angle.
        ending_angle: RingSingleParameter,
    },
}

/// Ring single-value parameter (used for angles, ratios, etc.).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RingSingleParameter {
    /// Fixed value.
    Fixed {
        /// The fixed value.
        value: f32,
    },
    /// Random range.
    Random {
        /// Maximum value.
        max: f32,
        /// Minimum value.
        min: f32,
    },
    /// Easing (ParameterEasingFloat).
    Easing(Box<ParameterEasingFloat>),
}

/// Ring location parameter (2D radial position).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RingLocationParameter {
    /// Fixed location.
    Fixed {
        /// Location (radial distance, height).
        location: Vector2D,
    },
    /// Position-velocity-acceleration.
    Pva {
        /// Location range.
        location: RandomVector2D,
        /// Velocity range.
        velocity: RandomVector2D,
        /// Acceleration range.
        acceleration: RandomVector2D,
    },
    /// Easing (old-style EasingVector2D, 44 bytes).
    Easing(EasingVector2D),
}

// ============================================================
// Model
// ============================================================

/// Model renderer parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModelParams {
    /// How the model is referenced.
    pub model_reference_type: ModelReferenceType,
    /// Model index (into model or procedural model table).
    pub model_index: i32,
    /// Billboard orientation.
    pub billboard: BillboardType,
    /// Culling mode.
    pub culling: CullingType,
    /// Color for all particles.
    pub all_color: AllTypeColorParameter,
    /// Model-level falloff (version 1600-1601 only).
    pub falloff: Option<FalloffParameter>,
}

// ============================================================
// Track
// ============================================================

/// Track renderer parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrackParams {
    /// Texture UV type parameters.
    pub texture_uv_type: NodeRendererTextureUVTypeParameter,
    /// Front track size.
    pub track_size_for: TrackSizeParameter,
    /// Middle track size.
    pub track_size_middle: TrackSizeParameter,
    /// Back track size.
    pub track_size_back: TrackSizeParameter,
    /// Spline subdivision count.
    pub spline_division: i32,
    /// Smoothing type (version >= 1700).
    pub smoothing: Option<TrailSmoothingType>,
    /// Trail time type (version >= 1700).
    pub trail_time_type: Option<TrailTimeType>,
    /// Left edge color.
    pub color_left: AllTypeColorParameter,
    /// Left-middle color.
    pub color_left_middle: AllTypeColorParameter,
    /// Center color.
    pub color_center: AllTypeColorParameter,
    /// Center-middle color.
    pub color_center_middle: AllTypeColorParameter,
    /// Right edge color.
    pub color_right: AllTypeColorParameter,
    /// Right-middle color.
    pub color_right_middle: AllTypeColorParameter,
}

/// Track size parameter.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TrackSizeParameter {
    /// Fixed size.
    Fixed {
        /// Size value.
        size: f32,
    },
}
