//! Translation parameter parser.
//!
//! Mirrors `Effekseer/Dev/Cpp/Effekseer/Effekseer/Parameter/Effekseer.Translation.h:170-243`
//! arm-for-arm. Each variant decides whether C++ reads an outer `int32 size`
//! prefix and whether it advances by that size (`pos += size`) or by the inner
//! decoder's return length. NurbsCurve and ViewOffset notably do NOT read an
//! outer size — they read fixed-layout payloads directly.
//!
//! The forward-compat protocol: variants with a size prefix run their inner
//! parse against a sub-reader bounded to `size` bytes via `read_sized_block`,
//! which absorbs any unread tail (newer writers may append fields older readers
//! don't know about).

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::params::TranslationParameter;
use crate::types::{NurbsCurveLoopType, ParameterTranslationType, ParseConfig};

use super::easing::parse_vector3d_easing_with_size;
use super::fcurve::parse_fcurve_vector3d;

/// Parse a TranslationParameter from the binary stream.
pub(crate) fn parse_translation(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<TranslationParameter, Error> {
    let type_: ParameterTranslationType = reader.read_enum(config, "Translation.type")?;

    match type_ {
        ParameterTranslationType::None => Ok(TranslationParameter::None),

        // C++ Effekseer.Translation.h:176-199
        ParameterTranslationType::Fixed => {
            let size = reader.read_i32()? as usize;
            reader.read_sized_block(size, |sub| {
                let ref_eq = sub.read_i32()?;
                let position = sub.read_vector3d()?;
                Ok(TranslationParameter::Fixed { ref_eq, position })
            })
        }

        // C++ Effekseer.Translation.h:200-217
        ParameterTranslationType::Pva => {
            let size = reader.read_i32()? as usize;
            reader.read_sized_block(size, |sub| {
                let ref_eq_p = sub.read_ref_min_max()?;
                let ref_eq_v = sub.read_ref_min_max()?;
                let ref_eq_a = sub.read_ref_min_max()?;
                let position = sub.read_random_vector3d()?;
                let velocity = sub.read_random_vector3d()?;
                let acceleration = sub.read_random_vector3d()?;
                Ok(TranslationParameter::Pva {
                    ref_eq_p,
                    ref_eq_v,
                    ref_eq_a,
                    position,
                    velocity,
                    acceleration,
                })
            })
        }

        // C++ Effekseer.Translation.h:218-224 — `Easing.Load(pos, size, version); pos += size;`
        ParameterTranslationType::Easing => {
            let size = reader.read_i32()? as usize;
            // minDynamicParameterVersion = 14, minAppendParameterVersion = 1600
            let easing = parse_vector3d_easing_with_size(reader, version, config, 14, 1600, size)?;
            Ok(TranslationParameter::Easing(Box::new(easing)))
        }

        // C++ Effekseer.Translation.h:225-232 — reads outer size but advances by FCurve->Load
        // return value. We trust FCurve's inner length (matches existing Rust behavior; see
        // plan note on FCurve divergence being out of scope).
        ParameterTranslationType::FCurve => {
            let _size = reader.read_i32()?;
            let fcurve = parse_fcurve_vector3d(reader, version, config)?;
            Ok(TranslationParameter::FCurve(Box::new(fcurve)))
        }

        // C++ Effekseer.Translation.h:233-237 — NO outer size, memcpy 16 bytes
        // (sizeof(ParameterTranslationNurbsCurve)).
        ParameterTranslationType::NurbsCurve => {
            let index = reader.read_i32()?;
            let scale = reader.read_f32()?;
            let move_speed = reader.read_f32()?;
            let loop_type: NurbsCurveLoopType =
                reader.read_enum(config, "Translation.NurbsCurve.loop_type")?;
            Ok(TranslationParameter::NurbsCurve {
                index,
                scale,
                move_speed,
                loop_type,
            })
        }

        // C++ Effekseer.Translation.h:238-242 — NO outer size, memcpy 8 bytes (random_float).
        ParameterTranslationType::ViewOffset => {
            let distance = reader.read_random_float()?;
            Ok(TranslationParameter::ViewOffset { distance })
        }

        _ => Ok(TranslationParameter::None),
    }
}
