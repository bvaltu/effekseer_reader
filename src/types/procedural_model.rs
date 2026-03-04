//! Procedural model parameter types.

use super::enums::{
    ProceduralModelAxisType, ProceduralModelCrossSectionType, ProceduralModelPrimitiveType,
    ProceduralModelType,
};
use super::primitives::Color;

/// Parameters for procedurally generated models.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProceduralModelParameter {
    /// Generation type (Mesh or Ribbon).
    pub model_type: ProceduralModelType,
    /// Primitive shape type.
    pub primitive_type: ProceduralModelPrimitiveType,
    /// Generation axis.
    pub axis_type: ProceduralModelAxisType,
    /// Mesh-specific parameters (if model_type == Mesh).
    pub mesh_params: Option<MeshParams>,
    /// Ribbon-specific parameters (if model_type == Ribbon).
    pub ribbon_params: Option<RibbonParams>,
    /// Primitive shape parameters.
    pub primitive_params: PrimitiveParams,
    /// Noise parameters.
    pub noise: ProceduralModelNoise,
    /// 3x3 vertex color grid (row-major: upper-left to lower-right).
    pub vertex_colors: [Color; 9],
    /// Color center position (version >= 1608).
    pub color_center_position: Option<[f32; 2]>,
    /// Color center area (version >= 1608).
    pub color_center_area: Option<[f32; 2]>,
    /// UV position (version >= 1608).
    pub uv_position: Option<[f32; 2]>,
    /// UV size (version >= 1608).
    pub uv_size: Option<[f32; 2]>,
}

/// Mesh-specific procedural model parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MeshParams {
    /// Start angle in radians.
    pub angle_begin: f32,
    /// End angle in radians.
    pub angle_end: f32,
    /// Axial divisions.
    pub divisions_axial: i32,
    /// Radial divisions.
    pub divisions_radial: i32,
    /// Rotation amount (version >= 1608).
    pub rotate: Option<f32>,
}

/// Ribbon-specific procedural model parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RibbonParams {
    /// Cross-section shape type.
    pub cross_section: ProceduralModelCrossSectionType,
    /// Rotation amount (version >= 1608).
    pub rotate: Option<f32>,
    /// Number of vertices.
    pub vertices: i32,
    /// Ribbon sizes [min, max].
    pub ribbon_sizes: [f32; 2],
    /// Ribbon angles [min, max].
    pub ribbon_angles: [f32; 2],
    /// Ribbon noise values [min, max].
    pub ribbon_noises: [f32; 2],
    /// Ribbon segment count.
    pub count: i32,
}

/// Primitive shape parameters (variant per shape type).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PrimitiveParams {
    /// Sphere primitive.
    Sphere {
        /// Sphere radius.
        radius: f32,
        /// Minimum depth.
        depth_min: f32,
        /// Maximum depth.
        depth_max: f32,
    },
    /// Cone primitive.
    Cone {
        /// Cone radius.
        radius: f32,
        /// Cone depth.
        depth: f32,
    },
    /// Cylinder primitive.
    Cylinder {
        /// Top radius.
        radius1: f32,
        /// Bottom radius.
        radius2: f32,
        /// Cylinder depth.
        depth: f32,
    },
    /// Spline4 primitive (4 control points).
    Spline4 {
        /// Control point 1 (x, y).
        point1: [f32; 2],
        /// Control point 2 (x, y).
        point2: [f32; 2],
        /// Control point 3 (x, y).
        point3: [f32; 2],
        /// Control point 4 (x, y).
        point4: [f32; 2],
    },
}

/// Noise parameters for procedural models.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProceduralModelNoise {
    /// Tilt noise parameters.
    pub tilt: NoiseParams2,
    /// Wave noise parameters.
    pub wave: NoiseParams3,
    /// Curl noise parameters.
    pub curl: NoiseParams3,
    /// Vertex color noise parameters (version >= 1608).
    pub vertex_color: Option<NoiseParams3>,
}

/// 2-component noise parameters (frequency, offset, power).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NoiseParams2 {
    /// Noise frequency [x, y].
    pub frequency: [f32; 2],
    /// Noise offset [x, y].
    pub offset: [f32; 2],
    /// Noise power [x, y].
    pub power: [f32; 2],
}

/// 3-component noise parameters (frequency, offset, power).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NoiseParams3 {
    /// Noise frequency [x, y, z].
    pub frequency: [f32; 3],
    /// Noise offset [x, y, z].
    pub offset: [f32; 3],
    /// Noise power [x, y, z].
    pub power: [f32; 3],
}
