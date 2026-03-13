//! Renderer parser — reads type-specific renderer data for all node types.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::node::{FalloffParameter, RendererVariant};
use crate::types::renderer::{
    ModelParams, NodeRendererTextureUVTypeParameter, RibbonColorParameter, RibbonParams,
    RibbonPositionParameter, RingLocationParameter, RingParams, RingShapeParameter,
    RingSingleParameter, SpriteColorParameter, SpriteParams, SpritePositionParameter, TrackParams,
    TrackSizeParameter,
};
use crate::types::{
    EffectNodeType, ModelReferenceType, ParseConfig, RingLocationType, RingShapeType,
    RingSingleType, TextureUVType, TrackSizeType, TrailSmoothingType, TrailTimeType,
};

use super::color::parse_all_type_color;
use super::easing::parse_easing_float;

/// Parse the renderer section for a given node type.
pub(crate) fn parse_renderer(
    reader: &mut BinaryReader,
    node_type: EffectNodeType,
    version: i32,
    config: &ParseConfig,
) -> Result<RendererVariant, Error> {
    match node_type {
        EffectNodeType::NoneType => {
            let _type_check = reader.read_i32()?;
            Ok(RendererVariant::None)
        }
        EffectNodeType::Sprite => {
            let params = parse_sprite_renderer(reader, version, config)?;
            Ok(RendererVariant::Sprite(params))
        }
        EffectNodeType::Ribbon => {
            let params = parse_ribbon_renderer(reader, version, config)?;
            Ok(RendererVariant::Ribbon(params))
        }
        EffectNodeType::Ring => {
            let params = parse_ring_renderer(reader, version, config)?;
            Ok(RendererVariant::Ring(params))
        }
        EffectNodeType::Model => {
            let params = parse_model_renderer(reader, version, config)?;
            Ok(RendererVariant::Model(params))
        }
        EffectNodeType::Track => {
            let params = parse_track_renderer(reader, version, config)?;
            Ok(RendererVariant::Track(params))
        }
        _ => {
            let _type_check = reader.read_i32()?;
            Ok(RendererVariant::None)
        }
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Parse a NodeRendererTextureUVTypeParameter.
/// The binary format is conditional on the UV type:
///   Stretch (0): no additional data
///   TilePerParticle (1): edge_head, edge_tail, loop_area_begin, loop_area_end
///   Tile (2): tile_length only
fn parse_texture_uv_type(
    reader: &mut BinaryReader,
    config: &ParseConfig,
) -> Result<NodeRendererTextureUVTypeParameter, Error> {
    let uv_type: TextureUVType = reader.read_enum(config, "TextureUVType.type")?;
    let mut param = NodeRendererTextureUVTypeParameter {
        uv_type,
        tile_length: 0.0,
        tile_edge_head: 0,
        tile_edge_tail: 0,
        tile_loop_area_begin: 0.0,
        tile_loop_area_end: 0.0,
    };
    match uv_type {
        TextureUVType::TilePerParticle => {
            param.tile_edge_head = reader.read_i32()?;
            param.tile_edge_tail = reader.read_i32()?;
            param.tile_loop_area_begin = reader.read_f32()?;
            param.tile_loop_area_end = reader.read_f32()?;
        }
        TextureUVType::Tile => {
            param.tile_length = reader.read_f32()?;
        }
        _ => {
            // Stretch: no additional data
        }
    }
    Ok(param)
}

/// Parse a RingSingleParameter.
fn parse_ring_single_parameter(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<RingSingleParameter, Error> {
    let type_: RingSingleType = reader.read_enum(config, "RingSingle.type")?;
    match type_ {
        RingSingleType::Fixed => {
            let value = reader.read_f32()?;
            Ok(RingSingleParameter::Fixed { value })
        }
        RingSingleType::Random => {
            let max = reader.read_f32()?;
            let min = reader.read_f32()?;
            Ok(RingSingleParameter::Random { max, min })
        }
        RingSingleType::Easing => {
            let easing = parse_easing_float(reader, version, config, 1608, 1608)?;
            Ok(RingSingleParameter::Easing(Box::new(easing)))
        }
        _ => Ok(RingSingleParameter::Fixed { value: 0.0 }),
    }
}

/// Parse a RingLocationParameter.
fn parse_ring_location_parameter(
    reader: &mut BinaryReader,
    config: &ParseConfig,
) -> Result<RingLocationParameter, Error> {
    let type_: RingLocationType = reader.read_enum(config, "RingLocation.type")?;
    match type_ {
        RingLocationType::Fixed => {
            let location = reader.read_vector2d()?;
            Ok(RingLocationParameter::Fixed { location })
        }
        RingLocationType::Pva => {
            let location = reader.read_random_vector2d()?;
            let velocity = reader.read_random_vector2d()?;
            let acceleration = reader.read_random_vector2d()?;
            Ok(RingLocationParameter::Pva {
                location,
                velocity,
                acceleration,
            })
        }
        RingLocationType::Easing => {
            let easing = reader.read_easing_vector2d()?;
            Ok(RingLocationParameter::Easing(easing))
        }
        _ => Ok(RingLocationParameter::Fixed {
            location: crate::types::Vector2D { x: 0.0, y: 0.0 },
        }),
    }
}

/// Parse a TrackSizeParameter.
fn parse_track_size(
    reader: &mut BinaryReader,
    config: &ParseConfig,
) -> Result<TrackSizeParameter, Error> {
    let type_: TrackSizeType = reader.read_enum(config, "TrackSize.type")?;
    match type_ {
        TrackSizeType::Fixed => {
            let size = reader.read_f32()?;
            Ok(TrackSizeParameter::Fixed { size })
        }
        _ => Ok(TrackSizeParameter::Fixed { size: 1.0 }),
    }
}

// ---------------------------------------------------------------------------
// Per-renderer parse functions
// ---------------------------------------------------------------------------

/// Parse Sprite renderer data.
fn parse_sprite_renderer(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<SpriteParams, Error> {
    // [1] node_type_check
    let _type_check = reader.read_i32()?;
    // [2] RenderingOrder
    let rendering_order = reader.read_enum(config, "Sprite.rendering_order")?;
    // [4] Billboard
    let billboard = reader.read_enum(config, "Sprite.billboard")?;
    // [5] SpriteAllColor
    let all_color = parse_all_type_color(reader, version, config)?;
    // [6] SpriteColor
    let sprite_color_type = reader.read_i32()?;
    let sprite_color = if sprite_color_type == 1 {
        let ll = reader.read_color()?;
        let lr = reader.read_color()?;
        let ul = reader.read_color()?;
        let ur = reader.read_color()?;
        SpriteColorParameter::Fixed { ll, lr, ul, ur }
    } else {
        SpriteColorParameter::Default
    };
    // [7] SpritePosition — type field + always 32 bytes
    let _position_type = reader.read_i32()?;
    let ll = reader.read_vector2d()?;
    let lr = reader.read_vector2d()?;
    let ul = reader.read_vector2d()?;
    let ur = reader.read_vector2d()?;
    let sprite_position = SpritePositionParameter::Fixed { ll, lr, ul, ur };
    Ok(SpriteParams {
        rendering_order,
        billboard,
        all_color,
        sprite_color,
        sprite_position,
    })
}

/// Parse Ribbon renderer data.
fn parse_ribbon_renderer(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<RibbonParams, Error> {
    // [1] node_type_check
    let _type_check = reader.read_i32()?;
    // [2] TextureUVType
    let texture_uv_type = parse_texture_uv_type(reader, config)?;
    // [3] TimeType (version >= 1700)
    let trail_time_type = if version >= 1700 {
        Some(reader.read_enum::<TrailTimeType>(config, "Ribbon.trail_time_type")?)
    } else {
        None
    };
    // [5] ViewpointDependent
    let viewpoint_dependent = reader.read_i32_as_bool()?;
    // [6] RibbonAllColor
    let all_color = parse_all_type_color(reader, version, config)?;
    // [7] RibbonColor
    let ribbon_color_type = reader.read_i32()?;
    let ribbon_color = if ribbon_color_type == 1 {
        let l = reader.read_color()?;
        let r = reader.read_color()?;
        RibbonColorParameter::Fixed { l, r }
    } else {
        RibbonColorParameter::Default
    };
    // [8] RibbonPosition — type + always read positions
    let _position_type = reader.read_i32()?;
    let l = reader.read_f32()?;
    let r = reader.read_f32()?;
    let ribbon_position = RibbonPositionParameter::Fixed { l, r };
    // [9] SplineDivision
    let spline_division = reader.read_i32()?;
    Ok(RibbonParams {
        texture_uv_type,
        trail_time_type,
        viewpoint_dependent,
        all_color,
        ribbon_color,
        ribbon_position,
        spline_division,
    })
}

/// Parse Ring renderer data.
fn parse_ring_renderer(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<RingParams, Error> {
    // [1] node_type_check
    let _type_check = reader.read_i32()?;
    // [2] RenderingOrder
    let rendering_order = reader.read_enum(config, "Ring.rendering_order")?;
    // [4] Billboard
    let billboard = reader.read_enum(config, "Ring.billboard")?;
    // [5] Ring shape (always version >= 1500)
    let shape_type: RingShapeType = reader.read_enum(config, "Ring.shape_type")?;
    let shape = match shape_type {
        RingShapeType::Crescent => {
            let starting_fade = reader.read_f32()?;
            let ending_fade = reader.read_f32()?;
            let starting_angle = parse_ring_single_parameter(reader, version, config)?;
            let ending_angle = parse_ring_single_parameter(reader, version, config)?;
            RingShapeParameter::Crescent {
                starting_fade,
                ending_fade,
                starting_angle,
                ending_angle,
            }
        }
        _ => RingShapeParameter::Donut,
    };
    // [6] VertexCount
    let vertex_count = reader.read_i32()?;
    // [7] ViewingAngle (legacy — always read, stored but superseded)
    let viewing_angle = parse_ring_single_parameter(reader, version, config)?;
    // [8] OuterLocation
    let outer_location = parse_ring_location_parameter(reader, config)?;
    // [9] InnerLocation
    let inner_location = parse_ring_location_parameter(reader, config)?;
    // [10] CenterRatio
    let center_ratio = parse_ring_single_parameter(reader, version, config)?;
    // [11] OuterColor
    let outer_color = parse_all_type_color(reader, version, config)?;
    // [12] CenterColor
    let center_color = parse_all_type_color(reader, version, config)?;
    // [13] InnerColor
    let inner_color = parse_all_type_color(reader, version, config)?;
    Ok(RingParams {
        rendering_order,
        billboard,
        shape,
        vertex_count,
        viewing_angle,
        outer_location,
        inner_location,
        center_ratio,
        outer_color,
        center_color,
        inner_color,
    })
}

/// Parse Model renderer data.
fn parse_model_renderer(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<ModelParams, Error> {
    // [1] node_type_check
    let _type_check = reader.read_i32()?;
    // [2] Mode (version >= 1602)
    let model_reference_type = if version >= 1602 {
        reader.read_enum::<ModelReferenceType>(config, "Model.mode")?
    } else {
        ModelReferenceType::File
    };
    // [3] Model reference
    let model_index = match model_reference_type {
        ModelReferenceType::File => {
            let _magnification = reader.read_f32()?; // discarded
            reader.read_i32()?
        }
        ModelReferenceType::Procedural => reader.read_i32()?,
        ModelReferenceType::External => {
            if version >= 1802 {
                reader.read_i32()?
            } else {
                -1
            }
        }
        _ => -1,
    };
    // [4] Billboard
    let billboard = reader.read_enum(config, "Model.billboard")?;
    // [6] Culling
    let culling = reader.read_enum(config, "Model.culling")?;
    // [7] AllColor
    let all_color = parse_all_type_color(reader, version, config)?;
    // [8] Falloff (1600 <= version < 1602)
    let falloff = if (1600..1602).contains(&version) {
        let flag = reader.read_i32()?;
        if flag == 1 {
            let color_blend_type = reader.read_enum(config, "ModelFalloff.color_blend_type")?;
            let begin_color = reader.read_color()?;
            let end_color = reader.read_color()?;
            let pow = reader.read_f32()?;
            Some(FalloffParameter {
                color_blend_type,
                begin_color,
                end_color,
                pow,
            })
        } else {
            None
        }
    } else {
        None
    };
    Ok(ModelParams {
        model_reference_type,
        model_index,
        billboard,
        culling,
        all_color,
        falloff,
    })
}

/// Parse Track renderer data.
fn parse_track_renderer(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<TrackParams, Error> {
    // [1] node_type_check
    let _type_check = reader.read_i32()?;
    // [2] TextureUVType
    let texture_uv_type = parse_texture_uv_type(reader, config)?;
    // [3-5] Track sizes
    let track_size_for = parse_track_size(reader, config)?;
    let track_size_middle = parse_track_size(reader, config)?;
    let track_size_back = parse_track_size(reader, config)?;
    // [6] SplineDivision
    let spline_division = reader.read_i32()?;
    // [7] SmoothingType (version >= 1700)
    let smoothing = if version >= 1700 {
        Some(reader.read_enum::<TrailSmoothingType>(config, "Track.smoothing")?)
    } else {
        None
    };
    // [8] TimeType (version >= 1700)
    let trail_time_type = if version >= 1700 {
        Some(reader.read_enum::<TrailTimeType>(config, "Track.trail_time_type")?)
    } else {
        None
    };
    // [9-14] 6 AllTypeColorParameter instances
    let color_left = parse_all_type_color(reader, version, config)?;
    let color_left_middle = parse_all_type_color(reader, version, config)?;
    let color_center = parse_all_type_color(reader, version, config)?;
    let color_center_middle = parse_all_type_color(reader, version, config)?;
    let color_right = parse_all_type_color(reader, version, config)?;
    let color_right_middle = parse_all_type_color(reader, version, config)?;
    Ok(TrackParams {
        texture_uv_type,
        track_size_for,
        track_size_middle,
        track_size_back,
        spline_division,
        smoothing,
        trail_time_type,
        color_left,
        color_left_middle,
        color_center,
        color_center_middle,
        color_right,
        color_right_middle,
    })
}
