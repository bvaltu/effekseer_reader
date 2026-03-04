//! UV parameter parser.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::uv::UVParameter;
use crate::types::{
    ParseConfig, UVAnimationInterpolationType, UVAnimationLoopType, UVAnimationType,
};

use super::fcurve::parse_fcurve_vector2d;

/// Parse a UVParameter from the binary stream.
///
/// `uv_index` is the UV slot index (0-4). Interpolation type is only read
/// for version >= 1600 AND uv_index == 0.
pub(crate) fn parse_uv(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
    uv_index: usize,
) -> Result<UVParameter, Error> {
    let type_: UVAnimationType = reader.read_enum(config, "UV.type")?;

    match type_ {
        UVAnimationType::Default => Ok(UVParameter::Default),
        UVAnimationType::Fixed => {
            let position = reader.read_rectf()?;
            Ok(UVParameter::Fixed { position })
        }
        UVAnimationType::Animation => {
            let position = reader.read_rectf()?;
            let frame_length = reader.read_i32()?;
            let frame_count_x = reader.read_i32()?;
            let frame_count_y = reader.read_i32()?;
            let loop_type: UVAnimationLoopType =
                reader.read_enum(config, "UV.Animation.loop_type")?;
            let start_frame = reader.read_random_int()?;
            let interpolation_type = if version >= 1600 && uv_index == 0 {
                Some(reader.read_enum::<UVAnimationInterpolationType>(
                    config,
                    "UV.Animation.interpolation_type",
                )?)
            } else {
                None
            };
            Ok(UVParameter::Animation {
                position,
                frame_length,
                frame_count_x,
                frame_count_y,
                loop_type,
                start_frame,
                interpolation_type,
            })
        }
        UVAnimationType::Scroll => {
            let position = reader.read_random_vector2d()?;
            let size = reader.read_random_vector2d()?;
            let speed = reader.read_random_vector2d()?;
            Ok(UVParameter::Scroll {
                position,
                size,
                speed,
            })
        }
        UVAnimationType::FCurve => {
            let position = parse_fcurve_vector2d(reader, version, config)?;
            let size = parse_fcurve_vector2d(reader, version, config)?;
            Ok(UVParameter::FCurve {
                position: Box::new(position),
                size: Box::new(size),
            })
        }
        _ => Ok(UVParameter::Default),
    }
}
