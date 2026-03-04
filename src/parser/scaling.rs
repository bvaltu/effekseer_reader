//! Scaling parameter parser.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::params::ScalingParameter;
use crate::types::{ParameterScalingType, ParseConfig};

use super::easing::{parse_easing_float, parse_easing_vector3d};
use super::fcurve::{parse_fcurve_scalar, parse_fcurve_vector3d};

/// Parse a ScalingParameter from the binary stream.
pub(crate) fn parse_scaling(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<ScalingParameter, Error> {
    let type_: ParameterScalingType = reader.read_enum(config, "Scaling.type")?;
    let size = reader.read_i32()? as usize;
    let start_pos = reader.position();

    let result = match type_ {
        ParameterScalingType::None => ScalingParameter::None,
        ParameterScalingType::Fixed => {
            let ref_eq = reader.read_i32()?;
            let scale = reader.read_vector3d()?;
            ScalingParameter::Fixed { ref_eq, scale }
        }
        ParameterScalingType::Pva => {
            let ref_eq_p = reader.read_ref_min_max()?;
            let ref_eq_v = reader.read_ref_min_max()?;
            let ref_eq_a = reader.read_ref_min_max()?;
            let scale = reader.read_random_vector3d()?;
            let velocity = reader.read_random_vector3d()?;
            let acceleration = reader.read_random_vector3d()?;
            ScalingParameter::Pva {
                ref_eq_p,
                ref_eq_v,
                ref_eq_a,
                scale,
                velocity,
                acceleration,
            }
        }
        ParameterScalingType::Easing => {
            // minDynamicParameterVersion = 14, minAppendParameterVersion = 1600
            let easing = parse_easing_vector3d(reader, version, config, 14, 1600)?;
            ScalingParameter::Easing(Box::new(easing))
        }
        ParameterScalingType::SinglePva => {
            let ref_eq_p = reader.read_ref_min_max()?;
            let ref_eq_v = reader.read_ref_min_max()?;
            let ref_eq_a = reader.read_ref_min_max()?;
            let scale = reader.read_random_float()?;
            let velocity = reader.read_random_float()?;
            let acceleration = reader.read_random_float()?;
            ScalingParameter::SinglePva {
                ref_eq_p,
                ref_eq_v,
                ref_eq_a,
                scale,
                velocity,
                acceleration,
            }
        }
        ParameterScalingType::SingleEasing => {
            // minDynamicParameterVersion = 14, minAppendParameterVersion = 1600
            let easing = parse_easing_float(reader, version, config, 14, 1600)?;
            ScalingParameter::SingleEasing(Box::new(easing))
        }
        ParameterScalingType::FCurve => {
            let fcurve = parse_fcurve_vector3d(reader, version, config)?;
            ScalingParameter::FCurve(Box::new(fcurve))
        }
        ParameterScalingType::SingleFCurve => {
            let fcurve = parse_fcurve_scalar(reader, version, config)?;
            ScalingParameter::SingleFCurve(Box::new(fcurve))
        }
        _ => {
            // Unknown type — skip
            ScalingParameter::None
        }
    };

    // Advance cursor to start_pos + size
    let consumed = reader.position() - start_pos;
    if size > consumed {
        reader.skip(size - consumed)?;
    }

    Ok(result)
}
