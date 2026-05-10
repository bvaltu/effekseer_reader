//! ParameterEasing template parsers — version-dependent easing for Float and Vector3D.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::params::{
    MiddlePointFloat, MiddlePointVector3D, ParameterEasingFloat, ParameterEasingVector3D,
};
use crate::types::primitives::RefMinMax;
use crate::types::{Easing3Type, ParseConfig};

/// Parse a ParameterEasingFloat (ElemNum=1).
///
/// `min_dynamic_version` and `min_append_version` control which fields are read.
pub(crate) fn parse_easing_float(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
    min_dynamic_version: i32,
    min_append_version: i32,
) -> Result<ParameterEasingFloat, Error> {
    // [Step 1] Dynamic equation refs
    let (ref_eq_s, ref_eq_e) = if version >= min_dynamic_version {
        (reader.read_ref_min_max()?, reader.read_ref_min_max()?)
    } else {
        (RefMinMax::default(), RefMinMax::default())
    };

    // [Step 2] Core values
    let start = reader.read_random_float()?;
    let end = reader.read_random_float()?;

    // [Step 3] Middle point
    let middle = if version >= min_append_version {
        let is_middle_enabled = reader.read_i32()?;
        if is_middle_enabled != 0 {
            let ref_eq_m = reader.read_ref_min_max()?;
            let middle = reader.read_random_float()?;
            Some(MiddlePointFloat { ref_eq_m, middle })
        } else {
            None
        }
    } else {
        None
    };

    // [Step 4] Easing type
    let (easing_type, easing_params) = if version >= min_append_version {
        let type_: Easing3Type = reader.read_enum(config, "Easing.type")?;
        let params = if type_ == Easing3Type::StartEndSpeed {
            let a = reader.read_f32()?;
            let b = reader.read_f32()?;
            let c = reader.read_f32()?;
            Some([a, b, c])
        } else {
            None
        };
        (type_, params)
    } else {
        // Fallback: always read a/b/c, type = StartEndSpeed
        let a = reader.read_f32()?;
        let b = reader.read_f32()?;
        let c = reader.read_f32()?;
        (Easing3Type::StartEndSpeed, Some([a, b, c]))
    };

    // [Step 5] Channel configuration
    let channel = if version >= min_append_version {
        reader.read_i32()?
    } else {
        0 // identity: channel[0] = 0
    };

    // [Step 6] Individual per-component easing
    let individual_types = if version >= min_append_version {
        let is_individual = reader.read_i32()?;
        if is_individual != 0 {
            // ElemNum = 1 for float
            let mut types = Vec::with_capacity(1);
            for _ in 0..1 {
                types.push(reader.read_enum(config, "Easing.individual_type")?);
            }
            Some(types)
        } else {
            None
        }
    } else {
        None
    };

    Ok(ParameterEasingFloat {
        ref_eq_s,
        ref_eq_e,
        start,
        end,
        middle,
        easing_type,
        easing_params,
        channel,
        individual_types,
    })
}

/// Parse a ParameterEasingVector3D (ElemNum=3).
///
/// `min_dynamic_version` and `min_append_version` control which fields are read.
pub(crate) fn parse_easing_vector3d(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
    min_dynamic_version: i32,
    min_append_version: i32,
) -> Result<ParameterEasingVector3D, Error> {
    // [Step 1] Dynamic equation refs
    let (ref_eq_s, ref_eq_e) = if version >= min_dynamic_version {
        (reader.read_ref_min_max()?, reader.read_ref_min_max()?)
    } else {
        (RefMinMax::default(), RefMinMax::default())
    };

    // [Step 2] Core values
    let start = reader.read_random_vector3d()?;
    let end = reader.read_random_vector3d()?;

    // [Step 3] Middle point
    let middle = if version >= min_append_version {
        let is_middle_enabled = reader.read_i32()?;
        if is_middle_enabled != 0 {
            let ref_eq_m = reader.read_ref_min_max()?;
            let middle = reader.read_random_vector3d()?;
            Some(MiddlePointVector3D { ref_eq_m, middle })
        } else {
            None
        }
    } else {
        None
    };

    // [Step 4] Easing type
    let (easing_type, easing_params) = if version >= min_append_version {
        let type_: Easing3Type = reader.read_enum(config, "Easing.type")?;
        let params = if type_ == Easing3Type::StartEndSpeed {
            let a = reader.read_f32()?;
            let b = reader.read_f32()?;
            let c = reader.read_f32()?;
            Some([a, b, c])
        } else {
            None
        };
        (type_, params)
    } else {
        // Fallback: always read a/b/c, type = StartEndSpeed
        let a = reader.read_f32()?;
        let b = reader.read_f32()?;
        let c = reader.read_f32()?;
        (Easing3Type::StartEndSpeed, Some([a, b, c]))
    };

    // [Step 5] Channel configuration
    let channel = if version >= min_append_version {
        reader.read_i32()?
    } else {
        // Identity mapping: channelIDs[i] = i → packed as 0x00_02_01_00
        0x00_02_01_00
    };

    // [Step 6] Individual per-component easing
    let individual_types = if version >= min_append_version {
        let is_individual = reader.read_i32()?;
        if is_individual != 0 {
            // ElemNum = 3 for Vector3D
            let mut types = Vec::with_capacity(3);
            for _ in 0..3 {
                types.push(reader.read_enum(config, "Easing.individual_type")?);
            }
            Some(types)
        } else {
            None
        }
    } else {
        None
    };

    Ok(ParameterEasingVector3D {
        ref_eq_s,
        ref_eq_e,
        start,
        end,
        middle,
        easing_type,
        easing_params,
        channel,
        individual_types,
    })
}

/// `sizeof(easing_float)` in C++: 2 random_float (16 B) + 3 float (12 B) = 28.
/// Used by `parse_float_easing_with_size` for `version < 1608` when no inner
/// size prefix exists on disk.
const SIZEOF_EASING_FLOAT: usize = 28;

/// Mirror of C++ `LoadFloatEasing`
/// (`Effekseer/Dev/Cpp/Effekseer/Effekseer/Utils/Effekseer.Compatiblity.h:10`).
///
/// For `version >= 1608` (Version16Alpha9), reads an `int32 size` and bounds the
/// inner load to that size. For older versions, bounds to `sizeof(easing_float)`
/// = 28 bytes. In both cases the outer reader advances by exactly the bounded
/// size after the inner load — this absorbs any forward-compat fields the writer
/// emitted that `parse_easing_float` doesn't know about.
pub(crate) fn parse_float_easing_with_size(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
    min_dynamic_version: i32,
    min_append_version: i32,
) -> Result<ParameterEasingFloat, Error> {
    let size = if version >= 1608 {
        reader.read_i32()? as usize
    } else {
        SIZEOF_EASING_FLOAT
    };
    reader.read_sized_block(size, |sub| {
        parse_easing_float(sub, version, config, min_dynamic_version, min_append_version)
    })
}

/// Vector3D analogue of `parse_float_easing_with_size`. C++ has no
/// `LoadVector3DEasing` symbol — instead, callers
/// (`Effekseer.Rotation.cpp`, `Effekseer.Translation.h`, `Effekseer.Scaling.h`)
/// read the outer-variant `size` and call `param.Load(pos, size, version)`
/// directly with that size. The effect is identical: bounded inner load, then
/// `pos += size`.
///
/// Caller passes the outer-variant `size` it already read; the inner parse runs
/// against a sub-reader bounded to `size` bytes.
pub(crate) fn parse_vector3d_easing_with_size(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
    min_dynamic_version: i32,
    min_append_version: i32,
    size: usize,
) -> Result<ParameterEasingVector3D, Error> {
    reader.read_sized_block(size, |sub| {
        parse_easing_vector3d(sub, version, config, min_dynamic_version, min_append_version)
    })
}
