//! Procedural model parameter parser.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::primitives::Color;
use crate::types::procedural_model::{
    MeshParams, NoiseParams2, NoiseParams3, PrimitiveParams, ProceduralModelNoise,
    ProceduralModelParameter, RibbonParams,
};
use crate::types::{
    ParseConfig, ProceduralModelAxisType, ProceduralModelCrossSectionType,
    ProceduralModelPrimitiveType, ProceduralModelType,
};
use crate::version::VERSION_16_ALPHA9;

/// Parse a single ProceduralModelParameter from the binary stream.
pub(crate) fn parse_procedural_model(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<ProceduralModelParameter, Error> {
    let model_type: ProceduralModelType = reader.read_enum(config, "ProceduralModel.model_type")?;
    let primitive_type: ProceduralModelPrimitiveType =
        reader.read_enum(config, "ProceduralModel.primitive_type")?;
    let axis_type: ProceduralModelAxisType =
        reader.read_enum(config, "ProceduralModel.axis_type")?;

    // Type-specific data
    let mesh_params = if model_type == ProceduralModelType::Mesh {
        let angle_begin = reader.read_f32()?;
        let angle_end = reader.read_f32()?;
        let divisions_axial = reader.read_i32()?;
        let divisions_radial = reader.read_i32()?;
        let rotate = if version >= VERSION_16_ALPHA9 {
            Some(reader.read_f32()?)
        } else {
            None
        };
        Some(MeshParams {
            angle_begin,
            angle_end,
            divisions_axial,
            divisions_radial,
            rotate,
        })
    } else {
        None
    };

    let ribbon_params = if model_type == ProceduralModelType::Ribbon {
        let cross_section: ProceduralModelCrossSectionType =
            reader.read_enum(config, "ProceduralModel.cross_section")?;
        let rotate = if version >= VERSION_16_ALPHA9 {
            Some(reader.read_f32()?)
        } else {
            None
        };
        let vertices = reader.read_i32()?;
        let ribbon_sizes = [reader.read_f32()?, reader.read_f32()?];
        let ribbon_angles = [reader.read_f32()?, reader.read_f32()?];
        let ribbon_noises = [reader.read_f32()?, reader.read_f32()?];
        let count = reader.read_i32()?;
        Some(RibbonParams {
            cross_section,
            rotate,
            vertices,
            ribbon_sizes,
            ribbon_angles,
            ribbon_noises,
            count,
        })
    } else {
        None
    };

    // Primitive-specific data
    let primitive_params = parse_primitive_params(reader, primitive_type)?;

    // Noise parameters
    let tilt = read_noise_params2(reader)?;
    let wave = read_noise_params3(reader)?;
    let curl = read_noise_params3(reader)?;
    let vertex_color = if version >= VERSION_16_ALPHA9 {
        Some(read_noise_params3(reader)?)
    } else {
        None
    };
    let noise = ProceduralModelNoise {
        tilt,
        wave,
        curl,
        vertex_color,
    };

    // Vertex color grid: 6 always present, 3 more for version >= 1608
    let white = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    let c0 = reader.read_color()?;
    let c1 = reader.read_color()?;
    let c2 = reader.read_color()?;
    let c3 = reader.read_color()?;
    let c4 = reader.read_color()?;
    let c5 = reader.read_color()?;
    let (c6, c7, c8) = if version >= VERSION_16_ALPHA9 {
        (
            reader.read_color()?,
            reader.read_color()?,
            reader.read_color()?,
        )
    } else {
        (white, white, white)
    };
    let vertex_colors = [c0, c1, c2, c3, c4, c5, c6, c7, c8];

    // Color center (version >= 1608)
    let (color_center_position, color_center_area) = if version >= VERSION_16_ALPHA9 {
        let pos = [reader.read_f32()?, reader.read_f32()?];
        let area = [reader.read_f32()?, reader.read_f32()?];
        (Some(pos), Some(area))
    } else {
        (None, None)
    };

    // UV (version >= 1608)
    let (uv_position, uv_size) = if version >= VERSION_16_ALPHA9 {
        let pos = [reader.read_f32()?, reader.read_f32()?];
        let size = [reader.read_f32()?, reader.read_f32()?];
        (Some(pos), Some(size))
    } else {
        (None, None)
    };

    Ok(ProceduralModelParameter {
        model_type,
        primitive_type,
        axis_type,
        mesh_params,
        ribbon_params,
        primitive_params,
        noise,
        vertex_colors,
        color_center_position,
        color_center_area,
        uv_position,
        uv_size,
    })
}

fn parse_primitive_params(
    reader: &mut BinaryReader,
    primitive_type: ProceduralModelPrimitiveType,
) -> Result<PrimitiveParams, Error> {
    match primitive_type {
        ProceduralModelPrimitiveType::Sphere => Ok(PrimitiveParams::Sphere {
            radius: reader.read_f32()?,
            depth_min: reader.read_f32()?,
            depth_max: reader.read_f32()?,
        }),
        ProceduralModelPrimitiveType::Cone => Ok(PrimitiveParams::Cone {
            radius: reader.read_f32()?,
            depth: reader.read_f32()?,
        }),
        ProceduralModelPrimitiveType::Cylinder => Ok(PrimitiveParams::Cylinder {
            radius1: reader.read_f32()?,
            radius2: reader.read_f32()?,
            depth: reader.read_f32()?,
        }),
        ProceduralModelPrimitiveType::Spline4 => Ok(PrimitiveParams::Spline4 {
            point1: [reader.read_f32()?, reader.read_f32()?],
            point2: [reader.read_f32()?, reader.read_f32()?],
            point3: [reader.read_f32()?, reader.read_f32()?],
            point4: [reader.read_f32()?, reader.read_f32()?],
        }),
        _ => {
            // Unknown primitive type — default to Sphere with zeros
            // This can happen if config allows unknown enums
            Ok(PrimitiveParams::Sphere {
                radius: 0.0,
                depth_min: 0.0,
                depth_max: 0.0,
            })
        }
    }
}

fn read_noise_params2(reader: &mut BinaryReader) -> Result<NoiseParams2, Error> {
    Ok(NoiseParams2 {
        frequency: [reader.read_f32()?, reader.read_f32()?],
        offset: [reader.read_f32()?, reader.read_f32()?],
        power: [reader.read_f32()?, reader.read_f32()?],
    })
}

fn read_noise_params3(reader: &mut BinaryReader) -> Result<NoiseParams3, Error> {
    Ok(NoiseParams3 {
        frequency: [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
        offset: [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
        power: [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
    })
}
