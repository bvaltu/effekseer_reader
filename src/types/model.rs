//! Model file types (.efkmodel).

use super::primitives::{Color, Vector2D, Vector3D};

/// A parsed Effekseer model file.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModelFile {
    /// Model file format version (0-6, independent from effect version).
    pub version: i32,
    /// Animation frames (each with its own vertex/face data).
    pub frames: Vec<ModelFrame>,
}

/// A single animation frame of a model.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModelFrame {
    /// Vertices in this frame.
    pub vertices: Vec<ModelVertex>,
    /// Triangle faces in this frame.
    pub faces: Vec<ModelFace>,
}

/// A model vertex with position, normals, UVs, and vertex color.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModelVertex {
    /// Vertex position.
    pub position: Vector3D,
    /// Vertex normal.
    pub normal: Vector3D,
    /// Vertex binormal.
    pub binormal: Vector3D,
    /// Vertex tangent.
    pub tangent: Vector3D,
    /// Primary UV coordinates.
    pub uv1: Vector2D,
    /// Secondary UV coordinates (equals uv1 for version < 6).
    pub uv2: Vector2D,
    /// Vertex color (white for version 0).
    pub vcolor: Color,
}

/// A triangle face defined by three vertex indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModelFace {
    /// Three vertex indices forming the triangle.
    pub indexes: [i32; 3],
}
