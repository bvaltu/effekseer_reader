//! Local force field parser.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::force_field::{
    ConeFalloff, ForceFieldFalloff, ForceFieldTypeParams, LocalForceFieldElement, TubeFalloff,
};
use crate::types::{
    ForceFieldTurbulenceType, ForceFieldVortexType, LocalForceFieldFalloffType,
    LocalForceFieldType, ParseConfig, Vector3D,
};

/// Parse the local force field section (up to 4 elements + optional legacy block).
pub(crate) fn parse_force_fields(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<Vec<LocalForceFieldElement>, Error> {
    let count = reader.read_i32()? as usize;
    if count > 4 {
        return Err(Error::ResourceLimitExceeded {
            field: "force_fields",
            count,
            max: 4,
        });
    }

    let mut elements = Vec::with_capacity(count);
    for _ in 0..count {
        elements.push(parse_force_field_element(reader, version, config)?);
    }

    // LocationAbs legacy block (1500 <= version <= 1600)
    if version <= 1600 {
        let abs_type = reader.read_i32()?;
        let size = reader.read_i32()? as usize;
        match abs_type {
            0 => {} // None: 0 bytes
            _ => {
                reader.skip(size)?;
            }
        }
    }

    Ok(elements)
}

/// Parse one force field element.
fn parse_force_field_element(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<LocalForceFieldElement, Error> {
    let field_type: LocalForceFieldType = reader.read_enum(config, "ForceField.type")?;

    // power, position, rotation are only present for version >= 1600
    let (power, position, rotation) = if version >= 1600 {
        let power = reader.read_f32()?;
        let position = reader.read_vector3d()?;
        let rotation = reader.read_vector3d()?;
        (power, position, rotation)
    } else {
        let zero = Vector3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        (1.0, zero, zero)
    };

    // Type-specific data
    let type_params = match field_type {
        LocalForceFieldType::None => ForceFieldTypeParams::None,
        LocalForceFieldType::Force => {
            let gravitation = reader.read_i32_as_bool()?;
            ForceFieldTypeParams::Force { gravitation }
        }
        LocalForceFieldType::Wind => ForceFieldTypeParams::Wind,
        LocalForceFieldType::Vortex => {
            let vortex_type = if version >= 1601 {
                Some(reader.read_enum::<ForceFieldVortexType>(config, "ForceField.vortex_type")?)
            } else {
                None
            };
            ForceFieldTypeParams::Vortex { vortex_type }
        }
        LocalForceFieldType::Turbulence => {
            // For version < 1601, turbulence_type is not read (defaults to Complicated)
            let turbulence_type = if version >= 1601 {
                reader.read_enum(config, "ForceField.turbulence_type")?
            } else {
                ForceFieldTurbulenceType::Complicated
            };
            let seed = reader.read_i32()?;
            let scale = reader.read_f32()?;
            // For version < 1601, strength is read directly from binary
            // For version >= 1601, strength comes from the power field (already read above)
            let strength = if version < 1601 {
                reader.read_f32()?
            } else {
                power
            };
            let octave = reader.read_i32()?;
            ForceFieldTypeParams::Turbulence {
                turbulence_type,
                seed,
                scale,
                strength,
                octave,
            }
        }
        LocalForceFieldType::Drag => ForceFieldTypeParams::Drag,
        LocalForceFieldType::Gravity => {
            let gravity = reader.read_vector3d()?;
            ForceFieldTypeParams::Gravity { gravity }
        }
        LocalForceFieldType::AttractiveForce => {
            // For version < 1600, force = power which is 1.0 (default)
            let force = power;
            let control = reader.read_f32()?;
            let min_range = reader.read_f32()?;
            let max_range = reader.read_f32()?;
            ForceFieldTypeParams::AttractiveForce {
                force,
                control,
                min_range,
                max_range,
            }
        }
        _ => ForceFieldTypeParams::None,
    };

    // Falloff (version >= 1600)
    let falloff = if version >= 1600 {
        let falloff_type: LocalForceFieldFalloffType =
            reader.read_enum(config, "ForceField.falloff_type")?;
        if falloff_type != LocalForceFieldFalloffType::None {
            let power = reader.read_f32()?;
            let max_distance = reader.read_f32()?;
            let min_distance = reader.read_f32()?;

            let (tube, cone) = match falloff_type {
                LocalForceFieldFalloffType::Tube => {
                    let radius_power = reader.read_f32()?;
                    let max_radius = reader.read_f32()?;
                    let min_radius = reader.read_f32()?;
                    (
                        Some(TubeFalloff {
                            radius_power,
                            max_radius,
                            min_radius,
                        }),
                        None,
                    )
                }
                LocalForceFieldFalloffType::Cone => {
                    let angle_power = reader.read_f32()?;
                    let max_angle = reader.read_f32()?;
                    let min_angle = reader.read_f32()?;
                    (
                        None,
                        Some(ConeFalloff {
                            angle_power,
                            max_angle,
                            min_angle,
                        }),
                    )
                }
                _ => (None, None),
            };

            Some(ForceFieldFalloff {
                falloff_type,
                power,
                max_distance,
                min_distance,
                tube,
                cone,
            })
        } else {
            None
        }
    } else {
        None
    };

    Ok(LocalForceFieldElement {
        field_type,
        power,
        position,
        rotation,
        type_params,
        falloff,
    })
}
