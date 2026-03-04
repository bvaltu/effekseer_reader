//! Rotation parameter parser.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::params::RotationParameter;
use crate::types::{ParameterRotationType, ParseConfig};

use super::easing::{parse_easing_float, parse_easing_vector3d};
use super::fcurve::parse_fcurve_vector3d;

/// Parse a RotationParameter from the binary stream.
pub(crate) fn parse_rotation(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<RotationParameter, Error> {
    let type_: ParameterRotationType = reader.read_enum(config, "Rotation.type")?;
    let size = reader.read_i32()? as usize;
    let start_pos = reader.position();

    let result = match type_ {
        ParameterRotationType::None => RotationParameter::None,
        ParameterRotationType::Fixed => {
            let ref_eq = reader.read_i32()?;
            let rotation = reader.read_vector3d()?;
            RotationParameter::Fixed { ref_eq, rotation }
        }
        ParameterRotationType::Pva => {
            let ref_eq_p = reader.read_ref_min_max()?;
            let ref_eq_v = reader.read_ref_min_max()?;
            let ref_eq_a = reader.read_ref_min_max()?;
            let rotation = reader.read_random_vector3d()?;
            let velocity = reader.read_random_vector3d()?;
            let acceleration = reader.read_random_vector3d()?;
            RotationParameter::Pva {
                ref_eq_p,
                ref_eq_v,
                ref_eq_a,
                rotation,
                velocity,
                acceleration,
            }
        }
        ParameterRotationType::Easing => {
            // minDynamicParameterVersion = 14 (always true), minAppendParameterVersion = 1600
            let easing = parse_easing_vector3d(reader, version, config, 14, 1600)?;
            RotationParameter::Easing(Box::new(easing))
        }
        ParameterRotationType::AxisPva => {
            // 48 bytes POD
            let axis = reader.read_random_vector3d()?;
            let rotation = reader.read_random_float()?;
            let velocity = reader.read_random_float()?;
            let acceleration = reader.read_random_float()?;
            RotationParameter::AxisPva {
                axis,
                rotation,
                velocity,
                acceleration,
            }
        }
        ParameterRotationType::AxisEasing => {
            let axis = reader.read_random_vector3d()?;
            // AxisEasing uses 1608 for both thresholds
            let easing = parse_easing_float(reader, version, config, 1608, 1608)?;
            RotationParameter::AxisEasing { axis, easing }
        }
        ParameterRotationType::FCurve => {
            let fcurve = parse_fcurve_vector3d(reader, version, config)?;
            RotationParameter::FCurve(Box::new(fcurve))
        }
        ParameterRotationType::RotateToViewpoint => RotationParameter::RotateToViewpoint,
        ParameterRotationType::Velocity => RotationParameter::Velocity,
        _ => {
            // Unknown type — skip
            RotationParameter::None
        }
    };

    // Advance cursor to start_pos + size
    let consumed = reader.position() - start_pos;
    if size > consumed {
        reader.skip(size - consumed)?;
    }

    Ok(result)
}
