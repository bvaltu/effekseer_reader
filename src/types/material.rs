//! Material file types (.efkmat).

use super::enums::{
    MaterialValueType, RequiredPredefinedMethodType, ShadingModelType, TextureColorType,
    TextureWrapType,
};
use super::gradient::Gradient;

/// A parsed Effekseer material file.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MaterialFile {
    /// Material file format version (separate from effect version).
    pub version: i32,
    /// Unique material identifier.
    pub guid: u64,
    /// Shading model (Lit or Unlit).
    pub shading_model: ShadingModelType,
    /// Whether refraction rendering is enabled.
    pub has_refraction: bool,
    /// Number of custom data 1 channels (0-4).
    pub custom_data_1_count: i32,
    /// Number of custom data 2 channels (0-4).
    pub custom_data_2_count: i32,
    /// Required predefined methods (version >= 1703).
    pub required_methods: Vec<RequiredPredefinedMethodType>,
    /// Texture parameter declarations.
    pub textures: Vec<MaterialTexture>,
    /// Uniform parameter declarations.
    pub uniforms: Vec<MaterialUniform>,
    /// Gradient parameters (version >= 1703).
    pub gradients: Vec<MaterialGradient>,
    /// Fixed (baked) gradient parameters (version >= 1703).
    pub fixed_gradients: Vec<MaterialGradient>,
    /// Tokenized shader code from the GENE chunk (null-terminated, tokens not replaced).
    pub code: Option<String>,
}

/// A texture parameter declaration in a material file.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MaterialTexture {
    /// ASCII texture parameter name.
    pub name: String,
    /// Shader uniform name (version >= 3).
    pub uniform_name: Option<String>,
    /// Texture slot index.
    pub index: i32,
    /// Priority (stored but unused at runtime).
    pub priority: i32,
    /// Parameter value (stored but unused at runtime).
    pub param: i32,
    /// How texture color is interpreted.
    pub color_type: TextureColorType,
    /// Texture wrapping mode.
    pub sampler: TextureWrapType,
}

/// A uniform parameter declaration in a material file.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MaterialUniform {
    /// ASCII uniform name.
    pub name: String,
    /// Shader uniform name (version >= 3).
    pub uniform_name: Option<String>,
    /// Byte offset (stored but unused at runtime).
    pub offset: i32,
    /// Priority (stored but unused at runtime).
    pub priority: i32,
    /// Value type (Float1-Float4).
    pub value_type: MaterialValueType,
    /// Default parameter values (4 floats).
    pub default_values: [f32; 4],
}

/// A gradient parameter declaration in a material file.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MaterialGradient {
    /// ASCII gradient parameter name.
    pub name: String,
    /// Shader uniform name.
    pub uniform_name: Option<String>,
    /// Gradient data.
    pub gradient: Gradient,
}
