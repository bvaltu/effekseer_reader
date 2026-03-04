//! Collision parameter types.

use super::enums::WorldCoordinateSystemType;
use super::primitives::RandomFloat;

/// Collision detection parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CollisionsParameter {
    /// Whether ground collision is enabled.
    pub is_ground_collision_enabled: bool,
    /// Whether scene collision with external is enabled.
    pub is_scene_collision_with_external: bool,
    /// Bounce factor.
    pub bounce: RandomFloat,
    /// Ground height.
    pub height: f32,
    /// Friction factor.
    pub friction: RandomFloat,
    /// Lifetime reduction per collision.
    pub lifetime_reduction_per_collision: RandomFloat,
    /// Coordinate system for collision detection.
    pub world_coordinate_system: WorldCoordinateSystemType,
}

impl Default for CollisionsParameter {
    fn default() -> Self {
        Self {
            is_ground_collision_enabled: false,
            is_scene_collision_with_external: false,
            bounce: RandomFloat { max: 0.0, min: 0.0 },
            height: 0.0,
            friction: RandomFloat { max: 0.0, min: 0.0 },
            lifetime_reduction_per_collision: RandomFloat { max: 0.0, min: 0.0 },
            world_coordinate_system: WorldCoordinateSystemType::Local,
        }
    }
}
