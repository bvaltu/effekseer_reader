//! Effect node types — the node tree structure.

use super::alpha_cutoff::AlphaCutoffParameter;
use super::collision::CollisionsParameter;
use super::common_values::ParameterCommonValues;
use super::custom_data::ParameterCustomData;
use super::depth::ParameterDepthValues;
use super::force_field::LocalForceFieldElement;
use super::gpu_particles::GpuParticlesParameter;
use super::kill_rules::KillRulesParameter;
use super::params::{
    LODsParam, RotationParameter, ScalingParameter, SpawnLocationParameter, SteeringBehaviorParam,
    TranslationParameter,
};
use super::renderer_common::ParameterRendererCommon;
use super::sound::ParameterSound;

/// A single node in the effect tree.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EffectNode {
    /// The type of this node (Root, Sprite, Ribbon, etc.).
    pub node_type: super::enums::EffectNodeType,
    /// Child nodes.
    pub children: Vec<EffectNode>,
    /// Node parameters (None for Root nodes).
    pub params: Option<NodeParams>,
}

/// All parameters for a non-root node, loaded in strict binary order.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeParams {
    /// Whether this node renders particles.
    pub is_rendered: bool,
    /// Rendering priority (draw order).
    pub rendering_priority: i32,
    /// Common values (lifetime, spawn rate, binding).
    pub common_values: ParameterCommonValues,
    /// Steering behavior (only for FollowParent translation bind types).
    pub steering_behavior: Option<SteeringBehaviorParam>,
    /// LOD parameters (version >= 1702).
    pub lods: Option<LODsParam>,
    /// Translation parameters.
    pub translation: TranslationParameter,
    /// Local force field elements.
    pub force_fields: Vec<LocalForceFieldElement>,
    /// Rotation parameters.
    pub rotation: RotationParameter,
    /// Scaling parameters.
    pub scaling: ScalingParameter,
    /// Spawn location parameters.
    pub spawn_location: SpawnLocationParameter,
    /// Whether parent rotation affects spawn.
    pub spawn_effects_rotation: bool,
    /// Depth parameters.
    pub depth: ParameterDepthValues,
    /// Kill rules.
    pub kill_rules: KillRulesParameter,
    /// Collision parameters.
    pub collisions: CollisionsParameter,
    /// Renderer common parameters.
    pub renderer_common: ParameterRendererCommon,
    /// Alpha cutoff parameters.
    pub alpha_cutoff: AlphaCutoffParameter,
    /// Falloff parameter (version >= 1602).
    pub falloff: Option<FalloffParameter>,
    /// Soft particle distances.
    pub soft_particle: SoftParticleParam,
    /// The type-specific renderer.
    pub renderer: RendererVariant,
    /// Sound parameters.
    pub sound: ParameterSound,
    /// GPU particle parameters (version >= 1800).
    pub gpu_particles: Option<GpuParticlesParameter>,
    /// Custom data slot 1.
    pub custom_data1: ParameterCustomData,
    /// Custom data slot 2.
    pub custom_data2: ParameterCustomData,
}

/// Falloff parameters (version >= 1602).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FalloffParameter {
    /// Blend type for falloff.
    pub color_blend_type: super::enums::FalloffBlendType,
    /// Begin color.
    pub begin_color: super::primitives::Color,
    /// End color.
    pub end_color: super::primitives::Color,
    /// Power exponent.
    pub pow: f32,
}

/// Soft particle distance parameters.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SoftParticleParam {
    /// Far distance for soft particle blending (version >= 1603).
    pub distance_far: f32,
    /// Near distance for soft particle blending (version >= 1604).
    pub distance_near: f32,
    /// Near offset for soft particle blending (version >= 1604).
    pub distance_near_offset: f32,
}

impl Default for SoftParticleParam {
    fn default() -> Self {
        Self {
            distance_far: 0.0,
            distance_near: 0.0,
            distance_near_offset: 0.0,
        }
    }
}

/// The renderer variant attached to a node.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum RendererVariant {
    /// No renderer.
    None,
    /// Sprite (billboard quad) renderer.
    Sprite(super::renderer::SpriteParams),
    /// Ribbon (connected strip) renderer.
    Ribbon(super::renderer::RibbonParams),
    /// Ring (torus) renderer.
    Ring(super::renderer::RingParams),
    /// 3D model renderer.
    Model(super::renderer::ModelParams),
    /// Track (trail) renderer.
    Track(super::renderer::TrackParams),
}
