//! Data model types for Effekseer binary formats.
//!
//! This module contains all struct and enum definitions. No parsing logic lives here.

pub mod alpha_cutoff;
pub mod collision;
pub mod color;
pub mod common_values;
pub mod curve;
pub mod custom_data;
pub mod depth;
pub mod effect;
pub mod enums;
pub mod fcurve;
pub mod force_field;
pub mod gpu_particles;
pub mod gradient;
pub mod kill_rules;
pub mod material;
pub mod model;
pub mod node;
pub mod params;
pub mod primitives;
pub mod procedural_model;
pub mod renderer;
pub mod renderer_common;
pub mod sound;
pub mod uv;

// Re-export commonly used types
pub use enums::*;
pub use primitives::*;

/// Configuration for parsing behavior.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParseConfig {
    /// How to handle unknown enum discriminant values.
    pub unknown_enum_behavior: UnknownEnumBehavior,
    /// Resource count limits to prevent OOM from malicious/corrupt files.
    pub limits: ResourceLimits,
}

impl Default for ParseConfig {
    fn default() -> Self {
        Self {
            unknown_enum_behavior: UnknownEnumBehavior::Error,
            limits: ResourceLimits::default(),
        }
    }
}

/// Behavior when an unknown enum discriminant is encountered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnknownEnumBehavior {
    /// Return an error immediately (default — fail-fast).
    Error,
    /// Log a warning and continue with the `Unknown` variant.
    Warn,
}

/// Resource count limits to prevent out-of-memory conditions.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResourceLimits {
    /// Maximum number of resource paths (textures, models, etc.).
    pub max_resource_paths: usize,
    /// Maximum number of procedural models.
    pub max_procedural_models: usize,
    /// Maximum number of dynamic inputs.
    pub max_dynamic_inputs: usize,
    /// Maximum number of dynamic equations.
    pub max_dynamic_equations: usize,
    /// Maximum depth of the node tree.
    pub max_node_depth: usize,
    /// Maximum number of children per node.
    pub max_node_children: usize,
    /// Maximum vertices per model frame.
    pub max_vertices_per_frame: usize,
    /// Maximum faces per model frame.
    pub max_faces_per_frame: usize,
    /// Maximum F-Curve sample count.
    pub max_fcurve_samples: usize,
    /// Maximum model frame count.
    pub max_frame_count: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_resource_paths: 1_024,
            max_procedural_models: 256,
            max_dynamic_inputs: 256,
            max_dynamic_equations: 256,
            max_node_depth: 64,
            max_node_children: 4_096,
            max_vertices_per_frame: 1_000_000,
            max_faces_per_frame: 1_000_000,
            max_fcurve_samples: 1_000_000,
            max_frame_count: 1_024,
        }
    }
}
