//! Rotation parameter parser.
//!
//! Mirrors `Effekseer/Dev/Cpp/Effekseer/Effekseer/Parameter/Effekseer.Rotation.cpp:9-101`
//! arm-for-arm. Every variant in C++ reads an outer `int32 size` prefix; only
//! Fixed/Pva/Easing/AxisPva use that size to advance (`pos += size`). The other
//! variants either read inside the bound (RotateToViewpoint reads nothing,
//! Velocity reads a 4-byte axis) or use the inner decoder's return length
//! (FCurve, AxisEasing).
//!
//! `None` is a sentinel value with no size prefix and no payload — matching
//! C++'s if/else-if chain that simply doesn't match it.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::params::RotationParameter;
use crate::types::{DirectionalAxisType, ParameterRotationType, ParseConfig};

use super::easing::{parse_float_easing_with_size, parse_vector3d_easing_with_size};
use super::fcurve::parse_fcurve_vector3d;

/// Parse a RotationParameter from the binary stream.
pub(crate) fn parse_rotation(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<RotationParameter, Error> {
    let type_: ParameterRotationType = reader.read_enum(config, "Rotation.type")?;

    match type_ {
        ParameterRotationType::None => Ok(RotationParameter::None),

        // C++ Effekseer.Rotation.cpp:16-39
        ParameterRotationType::Fixed => {
            let size = reader.read_i32()? as usize;
            reader.read_sized_block(size, |sub| {
                let ref_eq = sub.read_i32()?;
                let rotation = sub.read_vector3d()?;
                Ok(RotationParameter::Fixed { ref_eq, rotation })
            })
        }

        // C++ Effekseer.Rotation.cpp:40-54
        ParameterRotationType::Pva => {
            let size = reader.read_i32()? as usize;
            reader.read_sized_block(size, |sub| {
                let ref_eq_p = sub.read_ref_min_max()?;
                let ref_eq_v = sub.read_ref_min_max()?;
                let ref_eq_a = sub.read_ref_min_max()?;
                let rotation = sub.read_random_vector3d()?;
                let velocity = sub.read_random_vector3d()?;
                let acceleration = sub.read_random_vector3d()?;
                Ok(RotationParameter::Pva {
                    ref_eq_p,
                    ref_eq_v,
                    ref_eq_a,
                    rotation,
                    velocity,
                    acceleration,
                })
            })
        }

        // C++ Effekseer.Rotation.cpp:55-61 — `RotationEasing.Load(pos, size, version); pos += size;`
        ParameterRotationType::Easing => {
            let size = reader.read_i32()? as usize;
            // minDynamicParameterVersion = 14, minAppendParameterVersion = 1600
            let easing = parse_vector3d_easing_with_size(reader, version, config, 14, 1600, size)?;
            Ok(RotationParameter::Easing(Box::new(easing)))
        }

        // C++ Effekseer.Rotation.cpp:62-69
        ParameterRotationType::AxisPva => {
            let size = reader.read_i32()? as usize;
            reader.read_sized_block(size, |sub| {
                let axis = sub.read_random_vector3d()?;
                let rotation = sub.read_random_float()?;
                let velocity = sub.read_random_float()?;
                let acceleration = sub.read_random_float()?;
                Ok(RotationParameter::AxisPva {
                    axis,
                    rotation,
                    velocity,
                    acceleration,
                })
            })
        }

        // C++ Effekseer.Rotation.cpp:70-79 — reads outer size but does NOT do `pos += size`;
        // advances by axis (24 B) + LoadFloatEasing's internal size+payload.
        ParameterRotationType::AxisEasing => {
            let _size = reader.read_i32()?;
            let axis = reader.read_random_vector3d()?;
            // LoadFloatEasing uses 1608 for both thresholds.
            let easing = parse_float_easing_with_size(reader, version, config, 1608, 1608)?;
            Ok(RotationParameter::AxisEasing { axis, easing })
        }

        // C++ Effekseer.Rotation.cpp:80-87 — reads size, then advances by FCurve->Load
        // return value (NOT `pos += size`). We trust FCurve's inner length
        // (matches existing Rust behavior; see plan note on FCurve divergence).
        ParameterRotationType::FCurve => {
            let _size = reader.read_i32()?;
            let fcurve = parse_fcurve_vector3d(reader, version, config)?;
            Ok(RotationParameter::FCurve(Box::new(fcurve)))
        }

        // C++ Effekseer.Rotation.cpp:88-92 — reads size only; payload is empty.
        ParameterRotationType::RotateToViewpoint => {
            let _size = reader.read_i32()?;
            Ok(RotationParameter::RotateToViewpoint)
        }

        // C++ Effekseer.Rotation.cpp:93-100 — reads size, then memcpy(&axis,
        // pos, sizeof(DirectionalAxisType)) but `pos += sizeof(int)`. Whether
        // C++ trusts size or sizeof(int) is moot: DirectionalAxisType is i32.
        ParameterRotationType::Velocity => {
            let _size = reader.read_i32()?;
            let axis: DirectionalAxisType =
                reader.read_enum(config, "Rotation.Velocity.axis")?;
            Ok(RotationParameter::Velocity { axis })
        }

        _ => Ok(RotationParameter::None),
    }
}
