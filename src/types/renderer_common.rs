//! Common renderer parameters shared by all renderer types.

use super::custom_data::ParameterCustomData;
use super::enums::{
    AlphaBlendType, BindType, RendererMaterialType, TextureFilterType, TextureWrapType,
};
use super::gradient::Gradient;
use super::primitives::EasingFloatWithoutRandom;
use super::uv::UVParameter;

/// Common parameters for all renderers (textures, blending, UV, etc.).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterRendererCommon {
    /// Material type.
    pub material_type: RendererMaterialType,
    /// Emissive scaling (version >= 1600 and Default/Lighting material).
    pub emissive_scaling: Option<f32>,
    /// Color texture index.
    pub color_texture_index: i32,
    /// Normal texture index.
    pub normal_texture_index: i32,
    /// Alpha texture index (version >= 1600).
    pub alpha_texture_index: Option<i32>,
    /// UV distortion texture index (version >= 1600).
    pub uv_distortion_texture_index: Option<i32>,
    /// Blend texture index (version >= 1600).
    pub blend_texture_index: Option<i32>,
    /// Blend alpha texture index (version >= 1600).
    pub blend_alpha_texture_index: Option<i32>,
    /// Blend UV distortion texture index (version >= 1600).
    pub blend_uv_distortion_texture_index: Option<i32>,
    /// Material file data (if material_type == File).
    pub material_data: Option<MaterialFileData>,
    /// Alpha blend mode.
    pub alpha_blend: AlphaBlendType,
    /// Texture filter modes (indices 0-6).
    pub texture_filters: Vec<TextureFilterType>,
    /// Texture wrap modes (indices 0-6).
    pub texture_wraps: Vec<TextureWrapType>,
    /// Z-test enabled.
    pub z_test: bool,
    /// Z-write enabled.
    pub z_write: bool,
    /// Fade-in parameters.
    pub fade_in: Option<FadeParam>,
    /// Fade-out type (None, WithinLifetime, AfterRemoved).
    pub fade_out_type: super::enums::FadeOutType,
    /// Fade-out parameters.
    pub fade_out: Option<FadeParam>,
    /// UV parameters (up to 6 slots, indices 0-5).
    pub uv_params: Vec<UVParameter>,
    /// UV distortion intensity (version >= 1600).
    pub uv_distortion_intensity: Option<f32>,
    /// Texture blend type (version >= 1600).
    pub texture_blend_type: Option<AlphaBlendType>,
    /// Blend UV distortion intensity (version >= 1600).
    pub blend_uv_distortion_intensity: Option<f32>,
    /// UV horizontal flip probability (version >= 1801).
    pub uv_horizontal_flip_probability: Option<i32>,
    /// Color bind type.
    pub color_bind_type: BindType,
    /// Distortion intensity.
    pub distortion_intensity: f32,
    /// Custom data slot 1.
    pub custom_data1: ParameterCustomData,
    /// Custom data slot 2.
    pub custom_data2: ParameterCustomData,
}

/// Fade-in/out parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FadeParam {
    /// Frame duration.
    pub frame: f32,
    /// Easing parameters.
    pub value: EasingFloatWithoutRandom,
}

/// Data from a material file reference.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MaterialFileData {
    /// Material index in effect's material table.
    pub material_index: i32,
    /// Texture parameters (type, index) per texture slot.
    pub texture_indexes: Vec<MaterialTextureParam>,
    /// Uniform values (vec4 per uniform).
    pub uniforms: Vec<[f32; 4]>,
    /// Gradient data (version >= 1703).
    pub gradients: Vec<Gradient>,
}

/// Material texture parameter (type + index).
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MaterialTextureParam {
    /// Texture type (0 = color, 1 = value).
    pub texture_type: i32,
    /// Texture index.
    pub index: i32,
}

// ============================================================
// Constructors — ParameterRendererCommon
// ============================================================

impl ParameterRendererCommon {
    /// Sprite emitter binding `color_images[color_texture_index]` with the
    /// given `alpha_blend` mode. Fills all version-gated `Option` slots with
    /// `Some(-1)` / `Some(0.0)` so the v1810 wire format is well-formed, and
    /// sizes the texture-filter / wrap / uv-param vecs to the engine's
    /// expected slot counts (7, 7, 6).
    ///
    /// `color_bind_type` is `NotBind` so per-spawn
    /// `EffekseerEmitter.color_scale` overrides take effect at runtime.
    fn sprite(color_texture_index: i32, alpha_blend: AlphaBlendType) -> Self {
        Self {
            material_type: RendererMaterialType::Default,
            emissive_scaling: Some(1.0),
            color_texture_index,
            normal_texture_index: -1,
            alpha_texture_index: Some(-1),
            uv_distortion_texture_index: Some(-1),
            blend_texture_index: Some(-1),
            blend_alpha_texture_index: Some(-1),
            blend_uv_distortion_texture_index: Some(-1),
            material_data: None,
            alpha_blend,
            texture_filters: vec![TextureFilterType::Linear; 7],
            texture_wraps: vec![TextureWrapType::Clamp; 7],
            z_test: true,
            z_write: false,
            fade_in: None,
            fade_out_type: super::enums::FadeOutType::default(),
            fade_out: None,
            uv_params: vec![UVParameter::Default; 6],
            uv_distortion_intensity: Some(0.0),
            texture_blend_type: Some(AlphaBlendType::Blend),
            blend_uv_distortion_intensity: Some(0.0),
            uv_horizontal_flip_probability: Some(0),
            color_bind_type: BindType::default(),
            distortion_intensity: 0.0,
            custom_data1: ParameterCustomData::default(),
            custom_data2: ParameterCustomData::default(),
        }
    }

    /// Sprite emitter with additive blending — sparks, glows, fire,
    /// magic-flash effects.
    pub fn sprite_additive(color_texture_index: i32) -> Self {
        Self::sprite(color_texture_index, AlphaBlendType::Add)
    }

    /// Sprite emitter with standard alpha blending — smoke, dust,
    /// fading-debris effects.
    pub fn sprite_blend(color_texture_index: i32) -> Self {
        Self::sprite(color_texture_index, AlphaBlendType::Blend)
    }
}
