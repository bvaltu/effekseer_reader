//! GPU particles parser (version >= 1800).

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::gpu_particles::{
    GpuBasicParams, GpuColorData, GpuEmitShapeData, GpuEmitShapeParams, GpuForceParams,
    GpuParticlesParameter, GpuRenderBasicParams, GpuRenderColorParams, GpuRenderMaterialParams,
    GpuRenderShapeParams, GpuRotationParams, GpuScale4, GpuScaleData, GpuScaleParams,
    GpuVelocityParams,
};
use crate::types::{
    AlphaBlendType, BindType, GpuColorParamType, GpuColorSpaceType, GpuEmitShape, GpuMaterialType,
    GpuRenderShape, GpuScaleType, ParseConfig, TextureFilterType, TextureWrapType,
};

use super::fcurve::parse_fcurve_vector_color;
use super::gradient::parse_gradient;

/// Parse the GPU particles section (version >= 1800).
/// Returns `Some(param)` if enabled, `None` if not present or disabled.
pub(crate) fn parse_gpu_particles(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<Option<GpuParticlesParameter>, Error> {
    if version < 1800 {
        return Ok(None);
    }

    let enabled = reader.read_i32()?;
    if enabled == 0 {
        return Ok(None);
    }

    let params = parse_gpu_particles_data(reader, version, config)?;
    Ok(Some(params))
}

fn parse_gpu_particles_data(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<GpuParticlesParameter, Error> {
    // [Basic]
    let basic = GpuBasicParams {
        emit_count: reader.read_i32()?,
        emit_per_frame: reader.read_i32()?,
        emit_offset: reader.read_f32()?,
        life_time: [reader.read_f32()?, reader.read_f32()?],
    };

    // [Emit Shape]
    let emit_shape = parse_emit_shape(reader, config)?;

    // [Velocity]
    let velocity = GpuVelocityParams {
        direction: [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
        spread: reader.read_f32()?,
        initial_speed: [reader.read_f32()?, reader.read_f32()?],
        damping: [reader.read_f32()?, reader.read_f32()?],
    };

    // [Rotation]
    let rotation = GpuRotationParams {
        offset: [
            [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
            [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
        ],
        velocity: [
            [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
            [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
        ],
    };

    // [Scale]
    let scale = parse_scale(reader, config)?;

    // [Force]
    let force = GpuForceParams {
        gravity: [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
        vortex_rotation: reader.read_f32()?,
        vortex_attraction: reader.read_f32()?,
        vortex_center: [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
        vortex_axis: [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
        turbulence_power: reader.read_f32()?,
        turbulence_seed: reader.read_i32()?,
        turbulence_scale: reader.read_f32()?,
        turbulence_octave: reader.read_i32()?,
    };

    // [Render Basic]
    let blend_raw = reader.read_u8()?;
    let blend_type = AlphaBlendType::from(blend_raw as i32);
    let z_write = reader.read_u8()? != 0;
    let z_test = reader.read_u8()? != 0;
    let render_basic = GpuRenderBasicParams {
        blend_type,
        z_write,
        z_test,
    };

    // [Render Shape]
    let shape_type: GpuRenderShape = reader.read_enum_u8(config, "GpuParticles.render_shape")?;
    let data = reader.read_u32()?;
    let size = reader.read_f32()?;
    let render_shape = GpuRenderShapeParams {
        shape_type,
        data,
        size,
    };

    // [Render Color]
    let render_color = parse_render_color(reader, version, config)?;

    // [Render Material]
    let render_material = parse_render_material(reader, config)?;

    Ok(GpuParticlesParameter {
        basic,
        emit_shape,
        velocity,
        rotation,
        scale,
        force,
        render_basic,
        render_shape,
        render_color,
        render_material,
    })
}

fn parse_emit_shape(
    reader: &mut BinaryReader,
    config: &ParseConfig,
) -> Result<GpuEmitShapeParams, Error> {
    let shape_type: GpuEmitShape = reader.read_enum_u8(config, "GpuParticles.emit_shape")?;
    let rotation_applied = reader.read_u8()? != 0;

    let data = match shape_type {
        GpuEmitShape::Point => GpuEmitShapeData::Point,
        GpuEmitShape::Line => GpuEmitShapeData::Line {
            start: [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
            end: [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
            width: reader.read_f32()?,
        },
        GpuEmitShape::Circle => GpuEmitShapeData::Circle {
            axis: [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
            inner: reader.read_f32()?,
            outer: reader.read_f32()?,
        },
        GpuEmitShape::Sphere => GpuEmitShapeData::Sphere {
            radius: reader.read_f32()?,
        },
        GpuEmitShape::Model => GpuEmitShapeData::Model {
            index: reader.read_i32()?,
            size: reader.read_f32()?,
        },
        _ => GpuEmitShapeData::Point,
    };

    Ok(GpuEmitShapeParams {
        shape_type,
        rotation_applied,
        data,
    })
}

fn read_scale4(reader: &mut BinaryReader) -> Result<GpuScale4, Error> {
    Ok(GpuScale4 {
        uniform_min: reader.read_f32()?,
        uniform_max: reader.read_f32()?,
        axis_min: [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
        axis_max: [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
    })
}

fn parse_scale(reader: &mut BinaryReader, config: &ParseConfig) -> Result<GpuScaleParams, Error> {
    let scale_type: GpuScaleType = reader.read_enum_u8(config, "GpuParticles.scale_type")?;

    let data = match scale_type {
        GpuScaleType::Fixed => GpuScaleData::Fixed {
            scale: read_scale4(reader)?,
        },
        GpuScaleType::Easing => GpuScaleData::Easing {
            start: read_scale4(reader)?,
            end: read_scale4(reader)?,
            speed: [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?],
        },
        _ => {
            // PVA is unimplemented in C++ — error
            return Err(Error::InvalidEnumValue {
                field: "GpuParticles.scale_type",
                value: 1, // PVA
            });
        }
    };

    Ok(GpuScaleParams { scale_type, data })
}

fn parse_render_color(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<GpuRenderColorParams, Error> {
    let color_inherit_raw = reader.read_u8()?;
    let color_inherit = BindType::from(color_inherit_raw as i32);

    let color_all_type: GpuColorParamType =
        reader.read_enum_u8(config, "GpuParticles.color_type")?;
    let color_space: GpuColorSpaceType = reader.read_enum_u8(config, "GpuParticles.color_space")?;

    let data = match color_all_type {
        GpuColorParamType::Fixed => {
            let c = reader.read_color()?;
            GpuColorData::Fixed(c)
        }
        GpuColorParamType::Random => {
            let min = reader.read_color()?;
            let max = reader.read_color()?;
            GpuColorData::Random(min, max)
        }
        GpuColorParamType::Easing => {
            let start_min = reader.read_color()?;
            let start_max = reader.read_color()?;
            let end_min = reader.read_color()?;
            let end_max = reader.read_color()?;
            let speed = [reader.read_f32()?, reader.read_f32()?, reader.read_f32()?];
            GpuColorData::Easing {
                start: [start_min, start_max],
                end: [end_min, end_max],
                speed,
            }
        }
        GpuColorParamType::FCurve => {
            let fcurve = parse_fcurve_vector_color(reader, version, config)?;
            GpuColorData::FCurve(Box::new(fcurve))
        }
        GpuColorParamType::Gradient => {
            let gradient = parse_gradient(reader)?;
            GpuColorData::Gradient(gradient)
        }
        _ => GpuColorData::Fixed(crate::types::primitives::Color {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        }),
    };

    let emissive = reader.read_f32()?;
    let fade_in = reader.read_f32()?;
    let fade_out = reader.read_f32()?;

    Ok(GpuRenderColorParams {
        color_inherit,
        color_all_type,
        color_space,
        data,
        emissive,
        fade_in,
        fade_out,
    })
}

fn parse_render_material(
    reader: &mut BinaryReader,
    config: &ParseConfig,
) -> Result<GpuRenderMaterialParams, Error> {
    let material: GpuMaterialType = reader.read_enum_u8(config, "GpuParticles.material")?;

    let texture_indexes = [
        reader.read_u32()?,
        reader.read_u32()?,
        reader.read_u32()?,
        reader.read_u32()?,
    ];

    let texture_filters = [
        TextureFilterType::from(reader.read_u8()? as i32),
        TextureFilterType::from(reader.read_u8()? as i32),
        TextureFilterType::from(reader.read_u8()? as i32),
        TextureFilterType::from(reader.read_u8()? as i32),
    ];

    let texture_wraps = [
        TextureWrapType::from(reader.read_u8()? as i32),
        TextureWrapType::from(reader.read_u8()? as i32),
        TextureWrapType::from(reader.read_u8()? as i32),
        TextureWrapType::from(reader.read_u8()? as i32),
    ];

    Ok(GpuRenderMaterialParams {
        material,
        texture_indexes,
        texture_filters,
        texture_wraps,
    })
}
