//! Alpha cutoff parser.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::alpha_cutoff::{AlphaCutoffParameter, AlphaCutoffVariant};
use crate::types::{AlphaCutoffType, ParseConfig};

use super::easing::parse_easing_float;
use super::fcurve::parse_fcurve_scalar;

/// Parse the alpha cutoff section (version >= 1605).
pub(crate) fn parse_alpha_cutoff(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<AlphaCutoffParameter, Error> {
    if version >= 1605 {
        let flag = reader.read_i32()?;
        if flag == 1 && version >= 1600 {
            // Type
            let cutoff_type: AlphaCutoffType = reader.read_enum(config, "AlphaCutoff.type")?;
            // BufferSize
            let buffer_size = reader.read_i32()? as usize;
            let start = reader.position();

            let cutoff = match cutoff_type {
                AlphaCutoffType::Fixed => {
                    let ref_eq = reader.read_i32()?;
                    let threshold = reader.read_f32()?;
                    Some(AlphaCutoffVariant::Fixed { ref_eq, threshold })
                }
                AlphaCutoffType::FourPointInterpolation => {
                    let begin_threshold = reader.read_random_float()?;
                    let transition_frame_num = reader.read_random_int()?;
                    let no2_threshold = reader.read_random_float()?;
                    let no3_threshold = reader.read_random_float()?;
                    let transition_frame_num2 = reader.read_random_int()?;
                    let end_threshold = reader.read_random_float()?;
                    Some(AlphaCutoffVariant::FourPointInterpolation {
                        begin_threshold,
                        transition_frame_num,
                        no2_threshold,
                        no3_threshold,
                        transition_frame_num2,
                        end_threshold,
                    })
                }
                AlphaCutoffType::Easing => {
                    // ParameterEasingFloat: minDynamic=14 (always true), minAppend=1600
                    let easing = parse_easing_float(reader, version, config, 14, 1600)?;
                    Some(AlphaCutoffVariant::Easing(Box::new(easing)))
                }
                AlphaCutoffType::FCurve => {
                    let fcurve = parse_fcurve_scalar(reader, version, config)?;
                    Some(AlphaCutoffVariant::FCurve(Box::new(fcurve)))
                }
                _ => {
                    // Unknown type — skip buffer
                    reader.skip(buffer_size)?;
                    None
                }
            };

            // Ensure we've consumed exactly buffer_size bytes for the type data
            let consumed = reader.position() - start;
            if consumed < buffer_size {
                reader.skip(buffer_size - consumed)?;
            }

            // Edge fields (after buffer)
            let edge_threshold = reader.read_f32()?;
            let edge_color = reader.read_color()?;
            // Version < 1606: read as i32 then cast to f32 (integer-to-float conversion).
            // Version >= 1606: read directly as f32.
            let edge_color_scaling = if version < 1606 {
                reader.read_i32()? as f32
            } else {
                reader.read_f32()?
            };

            Ok(AlphaCutoffParameter {
                cutoff,
                edge_threshold,
                edge_color,
                edge_color_scaling,
            })
        } else {
            Ok(AlphaCutoffParameter::default())
        }
    } else {
        Ok(AlphaCutoffParameter::default())
    }
}
