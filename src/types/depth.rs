//! Depth parameter types.

use super::enums::ZSortType;

/// Depth-related rendering parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterDepthValues {
    /// Depth offset value.
    pub depth_offset: f32,
    /// Whether depth offset is scaled with camera distance.
    pub is_depth_offset_scaled_with_camera: bool,
    /// Whether depth offset is scaled with particle scale.
    pub is_depth_offset_scaled_with_particle_scale: bool,
    /// Suppression of scaling by depth.
    pub suppression_of_scaling_by_depth: f32,
    /// Depth clipping distance. `f32::MAX` disables clipping; finite values
    /// cause the sprite vertex shader to discard particles farther than this
    /// distance from the camera.
    pub depth_clipping: f32,
    /// Z-sort mode.
    pub z_sort: ZSortType,
    /// Drawing priority.
    pub drawing_priority: i32,
    /// Soft particle distance (legacy field in depth struct).
    pub soft_particle: f32,
}

impl Default for ParameterDepthValues {
    fn default() -> Self {
        Self {
            depth_offset: 0.0,
            is_depth_offset_scaled_with_camera: false,
            is_depth_offset_scaled_with_particle_scale: false,
            suppression_of_scaling_by_depth: 0.0,
            depth_clipping: f32::MAX,
            z_sort: ZSortType::None,
            drawing_priority: 0,
            soft_particle: 0.0,
        }
    }
}
