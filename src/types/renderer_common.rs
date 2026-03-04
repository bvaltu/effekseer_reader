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
