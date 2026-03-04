//! Spawn location (GenerationLocation) parser.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::params::SpawnLocationParameter;
use crate::types::{
    CircleDistributionType, GenerationLocationType, LineDistributionType, ModelCoordinateSpace,
    ModelReferenceType, ModelSpawnType, ParseConfig, SpawnAxisType,
};

/// Parse a SpawnLocationParameter from the binary stream.
/// Returns `(effects_rotation, parameter)`.
pub(crate) fn parse_spawn_location(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<(bool, SpawnLocationParameter), Error> {
    let effects_rotation = reader.read_i32_as_bool()?;
    let type_: GenerationLocationType = reader.read_enum(config, "SpawnLocation.type")?;

    let param = match type_ {
        GenerationLocationType::Point => {
            let location = reader.read_random_vector3d()?;
            SpawnLocationParameter::Point { location }
        }
        GenerationLocationType::Sphere => {
            let radius = reader.read_random_float()?;
            let rotation_x = reader.read_random_float()?;
            let rotation_y = reader.read_random_float()?;
            SpawnLocationParameter::Sphere {
                radius,
                rotation_x,
                rotation_y,
            }
        }
        GenerationLocationType::Model => {
            let reference = if version >= 1602 {
                reader.read_enum::<ModelReferenceType>(config, "SpawnLocation.Model.reference")?
            } else {
                ModelReferenceType::File
            };
            let index = reader.read_i32()?;
            let spawn_type: ModelSpawnType =
                reader.read_enum(config, "SpawnLocation.Model.spawn_type")?;
            let coordinate =
                if version >= 1802 {
                    Some(reader.read_enum::<ModelCoordinateSpace>(
                        config,
                        "SpawnLocation.Model.coordinate",
                    )?)
                } else {
                    None
                };
            SpawnLocationParameter::Model {
                reference,
                index,
                spawn_type,
                coordinate,
            }
        }
        GenerationLocationType::Circle => {
            let division = reader.read_i32()?;
            let radius = reader.read_random_float()?;
            let angle_start = reader.read_random_float()?;
            let angle_end = reader.read_random_float()?;
            let distribution_type: CircleDistributionType =
                reader.read_enum(config, "SpawnLocation.Circle.distribution_type")?;
            let axis_direction: SpawnAxisType =
                reader.read_enum(config, "SpawnLocation.Circle.axis_direction")?;
            let angle_noise = reader.read_random_float()?;
            SpawnLocationParameter::Circle {
                division,
                radius,
                angle_start,
                angle_end,
                distribution_type,
                axis_direction,
                angle_noise,
            }
        }
        GenerationLocationType::Line => {
            let division = reader.read_i32()?;
            let position_start = reader.read_random_vector3d()?;
            let position_end = reader.read_random_vector3d()?;
            let position_noise = reader.read_random_float()?;
            let distribution_type: LineDistributionType =
                reader.read_enum(config, "SpawnLocation.Line.distribution_type")?;
            SpawnLocationParameter::Line {
                division,
                position_start,
                position_end,
                position_noise,
                distribution_type,
            }
        }
        _ => {
            // Unknown type — treat as Point with zero offset
            let location = reader.read_random_vector3d()?;
            SpawnLocationParameter::Point { location }
        }
    };

    Ok((effects_rotation, param))
}
