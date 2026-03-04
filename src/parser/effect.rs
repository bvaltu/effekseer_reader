//! SKFE effect binary parser — header, resource tables, and global parameters.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::effect::{CullingInfo, Effect};
use crate::types::{CullingShape, ParseConfig};
use crate::version::{self, VERSION_16_ALPHA8, VERSION_17_ALPHA3};

/// Parse a raw SKFE effect binary into an [`Effect`].
pub(crate) fn parse_effect(data: &[u8], config: &ParseConfig) -> Result<Effect, Error> {
    let mut reader = BinaryReader::new(data);

    // --- Header ---
    let magic = reader.read_bytes(4)?;
    if magic != b"SKFE" {
        return Err(Error::InvalidMagic {
            expected: b"SKFE",
            got: magic.to_vec(),
        });
    }

    let version = reader.read_i32()?;
    version::validate_version(version)?;

    // --- Resource tables ---
    let color_images = read_resource_table(&mut reader, config, "color_images")?;
    let normal_images = read_resource_table(&mut reader, config, "normal_images")?;
    let distortion_images = read_resource_table(&mut reader, config, "distortion_images")?;
    let sounds = read_resource_table(&mut reader, config, "sounds")?;
    let models = read_resource_table(&mut reader, config, "models")?;
    let materials = read_resource_table(&mut reader, config, "materials")?;

    // Curves (version >= 1607)
    let curves = if version >= VERSION_16_ALPHA8 {
        read_resource_table(&mut reader, config, "curves")?
    } else {
        Vec::new()
    };

    // Procedural models (version >= 1607) — store raw bytes as placeholder
    let procedural_models = if version >= VERSION_16_ALPHA8 {
        let count = reader.read_i32()? as usize;
        if count > config.limits.max_procedural_models {
            return Err(Error::ResourceLimitExceeded {
                field: "procedural_models",
                count,
                max: config.limits.max_procedural_models,
            });
        }
        let mut models = Vec::with_capacity(count);
        for _ in 0..count {
            models.push(super::procedural_model::parse_procedural_model(
                &mut reader,
                version,
                config,
            )?);
        }
        models
    } else {
        Vec::new()
    };

    // --- Dynamic inputs ---
    let input_count = reader.read_i32()? as usize;
    if input_count > config.limits.max_dynamic_inputs {
        return Err(Error::ResourceLimitExceeded {
            field: "dynamic_inputs",
            count: input_count,
            max: config.limits.max_dynamic_inputs,
        });
    }
    let mut dynamic_inputs = Vec::with_capacity(input_count);
    for _ in 0..input_count {
        dynamic_inputs.push(reader.read_f32()?);
    }

    // --- Dynamic equations ---
    let equation_count = reader.read_i32()? as usize;
    if equation_count > config.limits.max_dynamic_equations {
        return Err(Error::ResourceLimitExceeded {
            field: "dynamic_equations",
            count: equation_count,
            max: config.limits.max_dynamic_equations,
        });
    }
    let mut dynamic_equations = Vec::with_capacity(equation_count);
    for _ in 0..equation_count {
        let binary_size = reader.read_i32()? as usize;
        let blob = reader.read_bytes(binary_size)?.to_vec();
        dynamic_equations.push(blob);
    }

    // --- Rendering optimization ---
    let rendering_node_count = reader.read_i32()?;
    let rendering_threshold = reader.read_i32()?;

    // --- Global parameters ---
    let magnification = reader.read_f32()?;
    let random_seed = reader.read_i32()?;

    // --- Culling ---
    let culling_shape_raw = reader.read_i32()?;
    let culling_shape = CullingShape::from(culling_shape_raw);
    let culling = if culling_shape == CullingShape::Sphere {
        let radius = reader.read_f32()?;
        let location = reader.read_vector3d()?;
        Some(CullingInfo {
            shape: culling_shape,
            location,
            radius,
        })
    } else {
        None
    };

    // --- LOD distances (version >= 1702) ---
    let lod_distances = if version >= VERSION_17_ALPHA3 {
        let d1 = reader.read_f32()?;
        let d2 = reader.read_f32()?;
        let d3 = reader.read_f32()?;
        Some([d1, d2, d3])
    } else {
        None
    };

    // --- Node tree ---
    let root = super::node::parse_node(&mut reader, version, config, 0)?;

    Ok(Effect {
        version,
        magnification,
        random_seed,
        culling,
        lod_distances,
        color_images,
        normal_images,
        distortion_images,
        sounds,
        models,
        materials,
        curves,
        procedural_models,
        dynamic_inputs,
        dynamic_equations,
        rendering_node_count,
        rendering_threshold,
        root,
    })
}

/// Read a resource path table (count + UTF-16LE strings).
fn read_resource_table(
    reader: &mut BinaryReader,
    config: &ParseConfig,
    field_name: &'static str,
) -> Result<Vec<String>, Error> {
    let count = reader.read_i32()? as usize;
    if count > config.limits.max_resource_paths {
        return Err(Error::ResourceLimitExceeded {
            field: field_name,
            count,
            max: config.limits.max_resource_paths,
        });
    }
    let mut paths = Vec::with_capacity(count);
    for _ in 0..count {
        paths.push(reader.read_utf16_string()?);
    }
    Ok(paths)
}
