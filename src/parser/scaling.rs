//! Scaling parameter parser.
//!
//! Mirrors `Effekseer/Dev/Cpp/Effekseer/Effekseer/Parameter/Effekseer.Scaling.h:114-204`
//! arm-for-arm. Every variant in C++ reads an outer `int32 size` prefix; all
//! except FCurve and SingleFCurve do `pos += size`. FCurve variants advance by
//! their inner `Load()` return value.
//!
//! `None` is a sentinel value with no size prefix and no payload — matching
//! C++'s if/else-if chain that simply doesn't match it.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::params::ScalingParameter;
use crate::types::{ParameterScalingType, ParseConfig};

use super::easing::{parse_easing_float, parse_vector3d_easing_with_size};
use super::fcurve::{parse_fcurve_scalar, parse_fcurve_vector3d};

/// Parse a ScalingParameter from the binary stream.
pub(crate) fn parse_scaling(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<ScalingParameter, Error> {
    let type_: ParameterScalingType = reader.read_enum(config, "Scaling.type")?;

    match type_ {
        ParameterScalingType::None => Ok(ScalingParameter::None),

        // C++ Effekseer.Scaling.h:121-145
        ParameterScalingType::Fixed => {
            let size = reader.read_i32()? as usize;
            reader.read_sized_block(size, |sub| {
                let ref_eq = sub.read_i32()?;
                let scale = sub.read_vector3d()?;
                Ok(ScalingParameter::Fixed { ref_eq, scale })
            })
        }

        // C++ Effekseer.Scaling.h:146-160
        ParameterScalingType::Pva => {
            let size = reader.read_i32()? as usize;
            reader.read_sized_block(size, |sub| {
                let ref_eq_p = sub.read_ref_min_max()?;
                let ref_eq_v = sub.read_ref_min_max()?;
                let ref_eq_a = sub.read_ref_min_max()?;
                let scale = sub.read_random_vector3d()?;
                let velocity = sub.read_random_vector3d()?;
                let acceleration = sub.read_random_vector3d()?;
                Ok(ScalingParameter::Pva {
                    ref_eq_p,
                    ref_eq_v,
                    ref_eq_a,
                    scale,
                    velocity,
                    acceleration,
                })
            })
        }

        // C++ Effekseer.Scaling.h:161-167 — `ScalingEasing.Load(pos, size, version); pos += size;`
        ParameterScalingType::Easing => {
            let size = reader.read_i32()? as usize;
            // minDynamicParameterVersion = 14, minAppendParameterVersion = 1600
            let easing = parse_vector3d_easing_with_size(reader, version, config, 14, 1600, size)?;
            Ok(ScalingParameter::Easing(Box::new(easing)))
        }

        // C++ Effekseer.Scaling.h:168-175
        ParameterScalingType::SinglePva => {
            let size = reader.read_i32()? as usize;
            reader.read_sized_block(size, |sub| {
                let scale = sub.read_random_float()?;
                let velocity = sub.read_random_float()?;
                let acceleration = sub.read_random_float()?;
                Ok(ScalingParameter::SinglePva {
                    scale,
                    velocity,
                    acceleration,
                })
            })
        }

        // C++ Effekseer.Scaling.h:176-183 — `ScalingSingleEasing.Load(pos, size, version); pos += size;`
        // Structurally identical to the inner body of `LoadFloatEasing`'s `param.Load(pos, size, version)`,
        // but the size comes from the outer-variant header (not a separate inner-size prefix), so we
        // bind the inner parse to the outer size directly rather than going through
        // `parse_float_easing_with_size` (which would read another size from disk).
        ParameterScalingType::SingleEasing => {
            let size = reader.read_i32()? as usize;
            let easing = reader.read_sized_block(size, |sub| {
                parse_easing_float(sub, version, config, 14, 1600)
            })?;
            Ok(ScalingParameter::SingleEasing(Box::new(easing)))
        }

        // C++ Effekseer.Scaling.h:184-194 — reads size, then advances by FCurve->Load return.
        ParameterScalingType::FCurve => {
            let _size = reader.read_i32()?;
            let fcurve = parse_fcurve_vector3d(reader, version, config)?;
            Ok(ScalingParameter::FCurve(Box::new(fcurve)))
        }

        // C++ Effekseer.Scaling.h:195-203 — reads size, then advances by FCurve->Load return.
        ParameterScalingType::SingleFCurve => {
            let _size = reader.read_i32()?;
            let fcurve = parse_fcurve_scalar(reader, version, config)?;
            Ok(ScalingParameter::SingleFCurve(Box::new(fcurve)))
        }

        _ => Ok(ScalingParameter::None),
    }
}
