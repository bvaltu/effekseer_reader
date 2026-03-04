//! Effect struct — resource tables and node tree root.

/// A parsed Effekseer effect, containing resource paths and the node tree.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Effect {
    /// Binary format version.
    pub version: i32,
    /// Global magnification factor.
    pub magnification: f32,
    /// Random seed for the effect.
    pub random_seed: i32,
    /// Culling information (sphere).
    pub culling: Option<CullingInfo>,
    /// LOD distance thresholds (up to 3).
    pub lod_distances: Option<[f32; 3]>,

    /// Color/diffuse texture paths.
    pub color_images: Vec<String>,
    /// Normal map texture paths.
    pub normal_images: Vec<String>,
    /// Distortion texture paths.
    pub distortion_images: Vec<String>,
    /// Sound file paths.
    pub sounds: Vec<String>,
    /// Model file paths.
    pub models: Vec<String>,
    /// Material file paths.
    pub materials: Vec<String>,
    /// Curve file paths.
    pub curves: Vec<String>,

    /// Inline procedural model definitions.
    pub procedural_models: Vec<super::procedural_model::ProceduralModelParameter>,
    /// Dynamic input values.
    pub dynamic_inputs: Vec<f32>,
    /// Dynamic equation bytecode blobs (opaque).
    pub dynamic_equations: Vec<Vec<u8>>,

    /// Number of rendering nodes (optimization hint).
    pub rendering_node_count: i32,
    /// Rendering threshold (optimization hint).
    pub rendering_threshold: i32,

    /// Root node of the effect tree.
    pub root: super::node::EffectNode,
}

/// Culling sphere information.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CullingInfo {
    /// Shape type.
    pub shape: super::enums::CullingShape,
    /// Location of the culling sphere.
    pub location: super::primitives::Vector3D,
    /// Radius of the culling sphere.
    pub radius: f32,
}
