//! Binary deserialization for Effekseer file formats.

mod alpha_cutoff;
mod collision;
mod color;
mod common_values;
mod curve;
mod depth;
mod easing;
mod effect;
mod efkefc;
mod fcurve;
mod force_field;
mod gpu_particles;
mod gradient;
mod kill_rules;
mod material;
mod model;
mod node;
mod procedural_model;
mod renderer;
mod renderer_common;
mod rotation;
mod scaling;
mod sound;
mod spawn;
mod translation;
mod uv;

use crate::error::Error;
use crate::types::ParseConfig;
use crate::types::curve::NurbsCurve;
use crate::types::effect::Effect;
use crate::types::material::MaterialFile;
use crate::types::model::ModelFile;

/// Parse an `.efkefc` container file (or raw `.efk`) using default strict config.
///
/// Auto-detects the format by inspecting the first 4 bytes:
/// - `"EFKE"` → extracts `BIN_` chunk, then parses the SKFE binary
/// - `"SKFE"` → parses directly as a raw `.efk` file
///
/// # Example
///
/// ```no_run
/// let data = std::fs::read("effect.efkefc").unwrap();
/// let effect = effekseer_reader::load_efkefc(&data).unwrap();
/// println!("Version: {}, Nodes: {}", effect.version, effect.root.children.len());
/// ```
pub fn load_efkefc(data: &[u8]) -> Result<Effect, Error> {
    load_efkefc_with_config(data, &ParseConfig::default())
}

/// Parse an `.efkefc` container file (or raw `.efk`) with the given config.
pub fn load_efkefc_with_config(data: &[u8], config: &ParseConfig) -> Result<Effect, Error> {
    let bin_data = efkefc::extract_bin_chunk(data)?;
    effect::parse_effect(bin_data, config)
}

/// Parse a raw `.efk` (SKFE) binary using default strict config.
pub fn load_efk(data: &[u8]) -> Result<Effect, Error> {
    load_efk_with_config(data, &ParseConfig::default())
}

/// Parse a raw `.efk` (SKFE) binary with the given config.
pub fn load_efk_with_config(data: &[u8], config: &ParseConfig) -> Result<Effect, Error> {
    effect::parse_effect(data, config)
}

/// Parse an `.efkmat` material file using default strict config.
///
/// # Example
///
/// ```no_run
/// let data = std::fs::read("material.efkmat").unwrap();
/// let mat = effekseer_reader::load_material(&data).unwrap();
/// println!("Textures: {}, Uniforms: {}", mat.textures.len(), mat.uniforms.len());
/// ```
pub fn load_material(data: &[u8]) -> Result<MaterialFile, Error> {
    load_material_with_config(data, &ParseConfig::default())
}

/// Parse an `.efkmat` material file with the given config.
pub fn load_material_with_config(data: &[u8], config: &ParseConfig) -> Result<MaterialFile, Error> {
    material::parse_material(data, config)
}

/// Parse an `.efkmodel` model file using default strict config.
///
/// # Example
///
/// ```no_run
/// let data = std::fs::read("model.efkmodel").unwrap();
/// let model = effekseer_reader::load_model(&data).unwrap();
/// println!("Frames: {}, Vertices: {}", model.frames.len(), model.frames[0].vertices.len());
/// ```
pub fn load_model(data: &[u8]) -> Result<ModelFile, Error> {
    load_model_with_config(data, &ParseConfig::default())
}

/// Parse an `.efkmodel` model file with the given config.
pub fn load_model_with_config(data: &[u8], config: &ParseConfig) -> Result<ModelFile, Error> {
    model::parse_model(data, config)
}

/// Parse a NURBS curve file using default strict config.
pub fn load_curve(data: &[u8]) -> Result<NurbsCurve, Error> {
    load_curve_with_config(data, &ParseConfig::default())
}

/// Parse a NURBS curve file with the given config.
pub fn load_curve_with_config(data: &[u8], config: &ParseConfig) -> Result<NurbsCurve, Error> {
    curve::parse_curve(data, config)
}
