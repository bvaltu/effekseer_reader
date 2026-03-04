//! Depth parameter parser.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::depth::ParameterDepthValues;
use crate::types::{ParseConfig, ZSortType};

/// Parse ParameterDepthValues (32 bytes, no size prefix).
pub(crate) fn parse_depth(
    reader: &mut BinaryReader,
    config: &ParseConfig,
) -> Result<ParameterDepthValues, Error> {
    let depth_offset = reader.read_f32()?;
    let is_depth_offset_scaled_with_camera = reader.read_i32_as_bool()?;
    let is_depth_offset_scaled_with_particle_scale = reader.read_i32_as_bool()?;
    let suppression_of_scaling_by_depth = reader.read_f32()?;
    let depth_clipping = reader.read_f32()?;
    let z_sort: ZSortType = reader.read_enum(config, "Depth.z_sort")?;
    let drawing_priority = reader.read_i32()?;
    let soft_particle = reader.read_f32()?;

    Ok(ParameterDepthValues {
        depth_offset,
        is_depth_offset_scaled_with_camera,
        is_depth_offset_scaled_with_particle_scale,
        suppression_of_scaling_by_depth,
        depth_clipping,
        z_sort,
        drawing_priority,
        soft_particle,
    })
}
