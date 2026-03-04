//! RendererCommon parser.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::custom_data::ParameterCustomData;
use crate::types::renderer_common::{
    FadeParam, MaterialFileData, MaterialTextureParam, ParameterRendererCommon,
};
use crate::types::{
    AlphaBlendType, BindType, EasingFloatWithoutRandom, FadeInType, FadeOutType,
    ParameterCustomDataType, ParseConfig, RendererMaterialType, TextureFilterType, TextureWrapType,
};

use super::fcurve::{parse_fcurve_vector_color, parse_fcurve_vector2d};
use super::gradient::parse_gradient;
use super::uv::parse_uv;

/// Parse ParameterRendererCommon.
pub(crate) fn parse_renderer_common(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<ParameterRendererCommon, Error> {
    // [1] MaterialType
    let p_mat = reader.position();
    let material_type: RendererMaterialType =
        reader.read_enum(config, "RendererCommon.material_type")?;
    log::debug!("  RC[1] MaterialType={material_type:?} at pos {p_mat}");

    // [2-4] Builtin materials (Default/BackDistortion/Lighting) read texture indices;
    // File materials read material file data instead.
    let is_builtin = material_type == RendererMaterialType::Default
        || material_type == RendererMaterialType::BackDistortion
        || material_type == RendererMaterialType::Lighting;

    // [2] EmissiveScaling (version >= 1600 AND default/lighting only)
    let emissive_scaling = if version >= 1600
        && (material_type == RendererMaterialType::Default
            || material_type == RendererMaterialType::Lighting)
    {
        let v = reader.read_f32()?;
        log::debug!("  RC[2] EmissiveScaling={} at pos {}", v, reader.position() - 4);
        Some(v)
    } else {
        None
    };

    // [3] Texture indices (only for builtin materials: Default, BackDistortion, Lighting)
    let (
        color_texture_index,
        normal_texture_index,
        alpha_texture_index,
        uv_distortion_texture_index,
        blend_texture_index,
        blend_alpha_texture_index,
        blend_uv_distortion_texture_index,
    );
    let material_data;

    if is_builtin {
        let p_tex = reader.position();
        color_texture_index = reader.read_i32()?;
        normal_texture_index = reader.read_i32()?;
        let (a, b, c, d, e) = if version >= 1600 {
            (
                Some(reader.read_i32()?),
                Some(reader.read_i32()?),
                Some(reader.read_i32()?),
                Some(reader.read_i32()?),
                Some(reader.read_i32()?),
            )
        } else {
            (None, None, None, None, None)
        };
        alpha_texture_index = a;
        uv_distortion_texture_index = b;
        blend_texture_index = c;
        blend_alpha_texture_index = d;
        blend_uv_distortion_texture_index = e;
        material_data = None;
        log::debug!("  RC[3] Textures: color={}, normal={}, alpha={:?}, uv_dist={:?}, blend={:?}, blend_alpha={:?}, blend_uv_dist={:?} at pos {}..{}", color_texture_index, normal_texture_index, alpha_texture_index, uv_distortion_texture_index, blend_texture_index, blend_alpha_texture_index, blend_uv_distortion_texture_index, p_tex, reader.position());
    } else {
        // [4] Material file data (MaterialType == File)
        color_texture_index = -1;
        normal_texture_index = -1;
        alpha_texture_index = None;
        uv_distortion_texture_index = None;
        blend_texture_index = None;
        blend_alpha_texture_index = None;
        blend_uv_distortion_texture_index = None;

        let material_index = reader.read_i32()?;
        let tex_count = reader.read_i32()? as usize;
        let mut texture_indexes = Vec::with_capacity(tex_count);
        for _ in 0..tex_count {
            let texture_type = reader.read_i32()?;
            let index = reader.read_i32()?;
            texture_indexes.push(MaterialTextureParam {
                texture_type,
                index,
            });
        }
        let uniform_count = reader.read_i32()? as usize;
        let mut uniforms = Vec::with_capacity(uniform_count);
        for _ in 0..uniform_count {
            let a = reader.read_f32()?;
            let b = reader.read_f32()?;
            let c = reader.read_f32()?;
            let d = reader.read_f32()?;
            uniforms.push([a, b, c, d]);
        }
        let gradients = if version >= 1703 {
            let gradient_count = reader.read_i32()? as usize;
            let mut grads = Vec::with_capacity(gradient_count);
            for _ in 0..gradient_count {
                grads.push(parse_gradient(reader)?);
            }
            grads
        } else {
            Vec::new()
        };
        material_data = Some(MaterialFileData {
            material_index,
            texture_indexes,
            uniforms,
            gradients,
        });
    }

    // [5] AlphaBlend
    let p_ab = reader.position();
    log::debug!("  RC[5] about to read AlphaBlend at pos {p_ab}");
    let alpha_blend: AlphaBlendType = reader.read_enum(config, "RendererCommon.alpha_blend")?;

    // [6] Texture filters and wraps
    let (mut texture_filters, mut texture_wraps) = {
        // Indices 0-1: always (2 filters + 2 wraps)
        let f0: TextureFilterType = reader.read_enum(config, "RendererCommon.filter0")?;
        let w0: TextureWrapType = reader.read_enum(config, "RendererCommon.wrap0")?;
        let f1: TextureFilterType = reader.read_enum(config, "RendererCommon.filter1")?;
        let w1: TextureWrapType = reader.read_enum(config, "RendererCommon.wrap1")?;
        (vec![f0, f1], vec![w0, w1])
    };
    if version >= 1600 {
        // Indices 2-6: 5 more pairs
        for i in 2..7 {
            let label_f = match i {
                2 => "RendererCommon.filter2",
                3 => "RendererCommon.filter3",
                4 => "RendererCommon.filter4",
                5 => "RendererCommon.filter5",
                _ => "RendererCommon.filter6",
            };
            let label_w = match i {
                2 => "RendererCommon.wrap2",
                3 => "RendererCommon.wrap3",
                4 => "RendererCommon.wrap4",
                5 => "RendererCommon.wrap5",
                _ => "RendererCommon.wrap6",
            };
            texture_filters.push(reader.read_enum(config, label_f)?);
            texture_wraps.push(reader.read_enum(config, label_w)?);
        }
    }

    // [7] ZTest, [8] ZWrite
    let z_test = reader.read_i32_as_bool()?;
    let z_write = reader.read_i32_as_bool()?;

    // [9] FadeIn
    let fade_in_type: FadeInType = reader.read_enum(config, "RendererCommon.fade_in_type")?;
    let fade_in = if fade_in_type != FadeInType::None {
        let frame = reader.read_f32()?;
        let a = reader.read_f32()?;
        let b = reader.read_f32()?;
        let c = reader.read_f32()?;
        Some(FadeParam {
            frame,
            value: EasingFloatWithoutRandom { a, b, c },
        })
    } else {
        None
    };

    // [10] FadeOut
    let fade_out_type: FadeOutType = reader.read_enum(config, "RendererCommon.fade_out_type")?;
    let fade_out = if fade_out_type != FadeOutType::None {
        let frame = reader.read_f32()?;
        let a = reader.read_f32()?;
        let b = reader.read_f32()?;
        let c = reader.read_f32()?;
        Some(FadeParam {
            frame,
            value: EasingFloatWithoutRandom { a, b, c },
        })
    } else {
        None
    };

    // [11] UV parameters
    // C++ reads 6 UV slots (indices 0-5) in a specific order with interleaved fields:
    //   UVs[0], then (v>=1600): UVs[1], UVs[2], UVDistortionIntensity,
    //   UVs[3], TextureBlendType, UVs[4], UVs[5], BlendUVDistortionIntensity
    let mut uv_params = Vec::new();
    uv_params.push(parse_uv(reader, version, config, 0)?);
    let (uv_distortion_intensity, texture_blend_type, blend_uv_distortion_intensity) =
        if version >= 1600 {
            uv_params.push(parse_uv(reader, version, config, 1)?);
            uv_params.push(parse_uv(reader, version, config, 2)?);
            let uv_dist_intensity = reader.read_f32()?;
            uv_params.push(parse_uv(reader, version, config, 3)?);
            let tex_blend_raw = reader.read_i32()?;
            let tex_blend_type = if tex_blend_raw >= 0 {
                AlphaBlendType::from(tex_blend_raw)
            } else {
                AlphaBlendType::Blend // default
            };
            uv_params.push(parse_uv(reader, version, config, 4)?);
            uv_params.push(parse_uv(reader, version, config, 5)?);
            let blend_uv_dist_intensity = reader.read_f32()?;
            (
                Some(uv_dist_intensity),
                Some(tex_blend_type),
                Some(blend_uv_dist_intensity),
            )
        } else {
            (None, None, None)
        };

    // [12] UVHorizontalFlipProbability (version >= 1801)
    let uv_horizontal_flip_probability = if version >= 1801 {
        Some(reader.read_i32()?)
    } else {
        None
    };

    // [13] ColorBindType
    let color_bind_type: BindType = reader.read_enum(config, "RendererCommon.color_bind_type")?;
    // [14] DistortionIntensity
    let distortion_intensity = reader.read_f32()?;

    // [15] CustomData1, [16] CustomData2
    let custom_data1 = parse_custom_data(reader, version, config)?;
    let custom_data2 = parse_custom_data(reader, version, config)?;

    Ok(ParameterRendererCommon {
        material_type,
        emissive_scaling,
        color_texture_index,
        normal_texture_index,
        alpha_texture_index,
        uv_distortion_texture_index,
        blend_texture_index,
        blend_alpha_texture_index,
        blend_uv_distortion_texture_index,
        material_data,
        alpha_blend,
        texture_filters,
        texture_wraps,
        z_test,
        z_write,
        fade_in,
        fade_out,
        uv_params,
        uv_distortion_intensity,
        texture_blend_type,
        blend_uv_distortion_intensity,
        uv_horizontal_flip_probability,
        color_bind_type,
        distortion_intensity,
        custom_data1,
        custom_data2,
    })
}

/// Parse a CustomData parameter.
fn parse_custom_data(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<ParameterCustomData, Error> {
    let type_: ParameterCustomDataType = reader.read_enum(config, "CustomData.type")?;

    match type_ {
        ParameterCustomDataType::None => Ok(ParameterCustomData::None),
        ParameterCustomDataType::Fixed2D => {
            let v = reader.read_vector2d()?;
            Ok(ParameterCustomData::Fixed2D(v))
        }
        ParameterCustomDataType::Random2D => {
            let v = reader.read_random_vector2d()?;
            Ok(ParameterCustomData::Random2D(v))
        }
        ParameterCustomDataType::Easing2D => {
            let v = reader.read_easing_vector2d()?;
            Ok(ParameterCustomData::Easing2D(v))
        }
        ParameterCustomDataType::FCurve2D => {
            let v = parse_fcurve_vector2d(reader, version, config)?;
            Ok(ParameterCustomData::FCurve2D(Box::new(v)))
        }
        ParameterCustomDataType::Fixed4D => {
            let a = reader.read_f32()?;
            let b = reader.read_f32()?;
            let c = reader.read_f32()?;
            let d = reader.read_f32()?;
            Ok(ParameterCustomData::Fixed4D([a, b, c, d]))
        }
        ParameterCustomDataType::FCurveColor => {
            let v = parse_fcurve_vector_color(reader, version, config)?;
            Ok(ParameterCustomData::FCurveColor(Box::new(v)))
        }
        ParameterCustomDataType::DynamicInput => Ok(ParameterCustomData::DynamicInput),
        _ => Ok(ParameterCustomData::None),
    }
}
