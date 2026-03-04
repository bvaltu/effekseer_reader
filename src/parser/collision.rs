//! Collision parameter parser.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::collision::CollisionsParameter;
use crate::types::primitives::RandomFloat;
use crate::types::{ParseConfig, WorldCoordinateSystemType};

/// Parse CollisionsParameter.
///
/// Binary layout changed across versions:
/// - v1800 (Version18Alpha1): isEnabled(i32) + Bounce(f32) + Height(f32) + WorldCoord(i32) = 16 bytes
/// - v1801+ (Version18Alpha2+): full format with random_float bounce/friction, scene collision, etc.
/// - v < 1800: no collision data
pub(crate) fn parse_collisions(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<CollisionsParameter, Error> {
    if version >= 1801 {
        // Version18Alpha2+ format
        let is_ground_collision_enabled = reader.read_i32_as_bool()?;
        let is_scene_collision_with_external = reader.read_i32_as_bool()?;
        let bounce = reader.read_random_float()?;
        let height = reader.read_f32()?;
        let friction = reader.read_random_float()?;
        let lifetime_reduction_per_collision = reader.read_random_float()?;
        let world_coordinate_system: WorldCoordinateSystemType =
            reader.read_enum(config, "Collisions.world_coordinate_system")?;

        Ok(CollisionsParameter {
            is_ground_collision_enabled,
            is_scene_collision_with_external,
            bounce,
            height,
            friction,
            lifetime_reduction_per_collision,
            world_coordinate_system,
        })
    } else if version >= 1800 {
        // Version18Alpha1 format (original 16-byte layout)
        let is_enabled = reader.read_i32_as_bool()?;
        let bounce_val = reader.read_f32()?;
        let height = reader.read_f32()?;
        let world_coordinate_system: WorldCoordinateSystemType =
            reader.read_enum(config, "Collisions.world_coordinate_system")?;

        Ok(CollisionsParameter {
            is_ground_collision_enabled: is_enabled,
            is_scene_collision_with_external: false,
            bounce: RandomFloat {
                max: bounce_val,
                min: bounce_val,
            },
            height,
            friction: RandomFloat {
                max: 0.0,
                min: 0.0,
            },
            lifetime_reduction_per_collision: RandomFloat {
                max: 0.0,
                min: 0.0,
            },
            world_coordinate_system,
        })
    } else {
        Ok(CollisionsParameter::default())
    }
}
