//! Translation parameter parser.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::params::TranslationParameter;
use crate::types::{NurbsCurveLoopType, ParameterTranslationType, ParseConfig};

use super::easing::parse_easing_vector3d;
use super::fcurve::parse_fcurve_vector3d;

/// Parse a TranslationParameter from the binary stream.
pub(crate) fn parse_translation(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<TranslationParameter, Error> {
    let type_: ParameterTranslationType = reader.read_enum(config, "Translation.type")?;
    let size = reader.read_i32()? as usize;
    let start_pos = reader.position();

    let result = match type_ {
        ParameterTranslationType::None => TranslationParameter::None,
        ParameterTranslationType::Fixed => {
            let ref_eq = reader.read_i32()?;
            let position = reader.read_vector3d()?;
            TranslationParameter::Fixed { ref_eq, position }
        }
        ParameterTranslationType::Pva => {
            let ref_eq_p = reader.read_ref_min_max()?;
            let ref_eq_v = reader.read_ref_min_max()?;
            let ref_eq_a = reader.read_ref_min_max()?;
            let position = reader.read_random_vector3d()?;
            let velocity = reader.read_random_vector3d()?;
            let acceleration = reader.read_random_vector3d()?;
            TranslationParameter::Pva {
                ref_eq_p,
                ref_eq_v,
                ref_eq_a,
                position,
                velocity,
                acceleration,
            }
        }
        ParameterTranslationType::Easing => {
            // minDynamicParameterVersion = 14 (always true), minAppendParameterVersion = 1600
            let easing = parse_easing_vector3d(reader, version, config, 14, 1600)?;
            TranslationParameter::Easing(Box::new(easing))
        }
        ParameterTranslationType::FCurve => {
            let fcurve = parse_fcurve_vector3d(reader, version, config)?;
            TranslationParameter::FCurve(Box::new(fcurve))
        }
        ParameterTranslationType::NurbsCurve => {
            // Read as 16-byte memcpy (4x i32/f32)
            let index = reader.read_i32()?;
            let scale = reader.read_f32()?;
            let move_speed = reader.read_f32()?;
            let loop_type: NurbsCurveLoopType =
                reader.read_enum(config, "Translation.NurbsCurve.loop_type")?;
            TranslationParameter::NurbsCurve {
                index,
                scale,
                move_speed,
                loop_type,
            }
        }
        ParameterTranslationType::ViewOffset => {
            let distance = reader.read_random_float()?;
            TranslationParameter::ViewOffset { distance }
        }
        _ => {
            // Unknown type — skip the data
            TranslationParameter::None
        }
    };

    // Advance cursor to start_pos + size for forward compatibility
    let consumed = reader.position() - start_pos;
    if size > consumed {
        reader.skip(size - consumed)?;
    }

    Ok(result)
}
