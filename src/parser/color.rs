//! All-type color parameter parser.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::color::AllTypeColorParameter;
use crate::types::{AllTypeColorType, ParseConfig};

use super::fcurve::parse_fcurve_vector_color;
use super::gradient::parse_gradient;

/// Parse an AllTypeColorParameter from the binary stream.
pub(crate) fn parse_all_type_color(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<AllTypeColorParameter, Error> {
    let type_: AllTypeColorType = reader.read_enum(config, "AllTypeColor.type")?;
    eprintln!("DEBUG_COLOR_PARSE: type={:?} at stream_pos={}", type_, reader.position());

    match type_ {
        AllTypeColorType::Fixed => {
            let all = reader.read_color()?;
            eprintln!("DEBUG_COLOR_PARSE: Fixed color=({},{},{},{})", all.r, all.g, all.b, all.a);
            Ok(AllTypeColorParameter::Fixed { all })
        }
        AllTypeColorType::Random => {
            let all = reader.read_random_color(config)?;
            Ok(AllTypeColorParameter::Random { all })
        }
        AllTypeColorType::Easing => {
            let easing = reader.read_easing_color(config)?;
            Ok(AllTypeColorParameter::Easing(easing))
        }
        AllTypeColorType::FCurveRgba => {
            let fcurve = parse_fcurve_vector_color(reader, version, config)?;
            Ok(AllTypeColorParameter::FCurveRgba(Box::new(fcurve)))
        }
        AllTypeColorType::Gradient => {
            let gradient = parse_gradient(reader)?;
            Ok(AllTypeColorParameter::Gradient(Box::new(gradient)))
        }
        _ => {
            // Unknown — default to fixed white
            eprintln!("DEBUG_COLOR_PARSE: UNKNOWN type {:?} — defaulting to white!", type_);
            Ok(AllTypeColorParameter::Fixed {
                all: crate::types::Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255,
                },
            })
        }
    }
}
