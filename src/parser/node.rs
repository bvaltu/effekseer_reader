//! Node tree parser — reads nodes recursively in strict binary order.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::node::{EffectNode, FalloffParameter, NodeParams, SoftParticleParam};
use crate::types::params::{LODsParam, SteeringBehaviorParam};
use crate::types::{
    EffectNodeType, NonMatchingLODBehaviour, ParseConfig, TranslationParentBindType,
};

use super::alpha_cutoff::parse_alpha_cutoff;
use super::collision::parse_collisions;
use super::common_values::{parse_common_values, parse_trigger_param};
use super::depth::parse_depth;
use super::force_field::parse_force_fields;
use super::gpu_particles::parse_gpu_particles;
use super::kill_rules::parse_kill_rules;
use super::renderer::parse_renderer;
use super::renderer_common::parse_renderer_common;
use super::rotation::parse_rotation;
use super::scaling::parse_scaling;
use super::sound::parse_sound;
use super::spawn::parse_spawn_location;
use super::translation::parse_translation;

/// Parse a node and its children recursively.
pub(crate) fn parse_node(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
    depth: usize,
) -> Result<EffectNode, Error> {
    if depth > config.limits.max_node_depth {
        return Err(Error::ResourceLimitExceeded {
            field: "node_depth",
            count: depth,
            max: config.limits.max_node_depth,
        });
    }

    let p_nt = reader.position();
    let node_type: EffectNodeType = reader.read_enum(config, "node_type")?;
    log::debug!("Node type={node_type:?} at pos {p_nt} depth={depth}");

    let params = if node_type == EffectNodeType::Root {
        // Root node: no parameter data, skip directly to children
        None
    } else {
        // Non-root: read all parameter sections in strict binary order
        Some(parse_node_params(reader, version, config, node_type)?)
    };

    // [Children]
    let child_count = reader.read_i32()? as usize;
    if child_count > config.limits.max_node_children {
        return Err(Error::ResourceLimitExceeded {
            field: "node_children",
            count: child_count,
            max: config.limits.max_node_children,
        });
    }

    let mut children = Vec::with_capacity(child_count);
    for _ in 0..child_count {
        children.push(parse_node(reader, version, config, depth + 1)?);
    }

    Ok(EffectNode {
        node_type,
        children,
        params,
    })
}

/// Parse all parameter sections for a non-root node.
fn parse_node_params(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
    node_type: EffectNodeType,
) -> Result<NodeParams, Error> {
    // [1] IsRendered
    let is_rendered = reader.read_i32_as_bool()?;

    // [2] RenderingPriority
    let rendering_priority = reader.read_i32()?;

    // [3] CommonValues
    let p3 = reader.position();
    let mut common_values = parse_common_values(reader, version, config)?;
    log::debug!("[3] CommonValues: pos {}..{}", p3, reader.position());

    // [4] SteeringBehaviorParam (version >= 1600 AND FollowParent bind types)
    let p4 = reader.position();
    let steering_behavior = if version >= 1600
        && (common_values.translation_bind_type == TranslationParentBindType::NotBindFollowParent
            || common_values.translation_bind_type
                == TranslationParentBindType::WhenCreatingFollowParent)
    {
        let max_follow_speed = reader.read_random_float()?;
        let steering_speed = reader.read_random_float()?;
        Some(SteeringBehaviorParam {
            max_follow_speed,
            steering_speed,
        })
    } else {
        None
    };
    log::debug!("[4] SteeringBehavior: pos {}..{}", p4, reader.position());

    // [5] TriggerParam (version 1700..1801)
    let p5 = reader.position();
    parse_trigger_param(reader, version, &mut common_values)?;
    log::debug!("[5] TriggerParam: pos {}..{}", p5, reader.position());

    // [6] LODsParam (version >= 1702)
    let p6 = reader.position();
    let lods = if version >= 1702 {
        let matching_lods = reader.read_i32()?;
        let lod_behaviour: NonMatchingLODBehaviour = reader.read_enum(config, "LODs.behaviour")?;
        Some(LODsParam {
            matching_lods,
            lod_behaviour,
        })
    } else {
        None
    };
    log::debug!("[6] LODs: pos {}..{}", p6, reader.position());

    // [7] TranslationParam
    let p7 = reader.position();
    let translation = parse_translation(reader, version, config)?;
    log::debug!("[7] Translation: pos {}..{}", p7, reader.position());

    // [8] LocalForceField
    let p8 = reader.position();
    let force_fields = parse_force_fields(reader, version, config)?;
    log::debug!("[8] ForceFields: pos {}..{}", p8, reader.position());

    // [9] RotationParam
    let p9 = reader.position();
    let rotation = parse_rotation(reader, version, config)?;
    log::debug!("[9] Rotation: pos {}..{}", p9, reader.position());

    // [10] ScalingParam
    let p10 = reader.position();
    let scaling = parse_scaling(reader, version, config)?;
    log::debug!("[10] Scaling: pos {}..{}", p10, reader.position());

    // [11] GenerationLocation
    let p11 = reader.position();
    let (spawn_effects_rotation, spawn_location) = parse_spawn_location(reader, version, config)?;
    log::debug!("[11] SpawnLocation: pos {}..{}", p11, reader.position());

    // [12] DepthValues
    let p12 = reader.position();
    let depth = parse_depth(reader, config)?;
    log::debug!("[12] Depth: pos {}..{}", p12, reader.position());

    // [13] KillParam
    let p13 = reader.position();
    let kill_rules = parse_kill_rules(reader, version, config)?;
    log::debug!("[13] KillRules: pos {}..{}", p13, reader.position());

    // [14] Collisions
    let p14 = reader.position();
    let collisions = parse_collisions(reader, version, config)?;
    log::debug!("[14] Collisions: pos {}..{}", p14, reader.position());

    // [15] RendererCommon
    let p15 = reader.position();
    let renderer_common = parse_renderer_common(reader, version, config)?;
    log::debug!("[15] RendererCommon: pos {}..{}", p15, reader.position());

    // [16] AlphaCutoff
    let alpha_cutoff = parse_alpha_cutoff(reader, version, config)?;

    // [17] Falloff (version >= 1602)
    let falloff = if version >= 1602 {
        let flag = reader.read_i32()?;
        if flag == 1 {
            let color_blend_type = reader.read_enum(config, "Falloff.color_blend_type")?;
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

    // [18] Soft Particle
    let soft_particle = {
        let mut sp = SoftParticleParam::default();
        if version >= 1603 {
            sp.distance_far = reader.read_f32()?;
        }
        if version >= 1604 {
            sp.distance_near = reader.read_f32()?;
            sp.distance_near_offset = reader.read_f32()?;
        }
        sp
    };

    // [19] Renderer
    let renderer = parse_renderer(reader, node_type, version, config)?;

    // [20] Sound
    let sound = parse_sound(reader, config)?;

    // [21] GPU Particles
    let gpu_particles = parse_gpu_particles(reader, version, config)?;

    // CustomData1, CustomData2 — consumed by RendererCommon
    let custom_data1 = renderer_common.custom_data1.clone();
    let custom_data2 = renderer_common.custom_data2.clone();

    Ok(NodeParams {
        is_rendered,
        rendering_priority,
        common_values,
        steering_behavior,
        lods,
        translation,
        force_fields,
        rotation,
        scaling,
        spawn_location,
        spawn_effects_rotation,
        depth,
        kill_rules,
        collisions,
        renderer_common,
        alpha_cutoff,
        falloff,
        soft_particle,
        renderer,
        sound,
        gpu_particles,
        custom_data1,
        custom_data2,
    })
}
