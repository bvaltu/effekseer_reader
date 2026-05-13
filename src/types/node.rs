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

// ============================================================
// Authoring API — sprite node composition
// ============================================================

/// The artistic / structural inputs required to construct a sprite-emitter
/// [`NodeParams`]. Every field here is a deliberate choice; the rest of
/// `NodeParams` is filled by [`NodeParams::sprite`] with safe defaults
/// (`is_rendered: true`, no kill rules, no collisions, no sound, no GPU
/// particles, no custom data, depth defaults, etc.).
pub struct SpriteNodeSpec {
    /// Lifetime, particle count, and parent binding behavior.
    pub common_values: ParameterCommonValues,
    /// How particles move (stationary, radial burst, PVA, etc.).
    pub translation: TranslationParameter,
    /// Per-particle scale.
    pub scaling: ScalingParameter,
    /// Texture / blend / depth settings shared with non-sprite renderers.
    pub renderer_common: ParameterRendererCommon,
    /// Sprite-specific shape and tinting.
    pub sprite: super::renderer::SpriteParams,
}

impl EffectNode {
    /// Root node carrying the given child subtree. Root nodes have no
    /// `NodeParams`; their job is to host the actual renderer nodes below
    /// them.
    pub fn root(children: Vec<EffectNode>) -> Self {
        Self {
            node_type: super::enums::EffectNodeType::Root,
            children,
            params: None,
        }
    }

    /// Sprite renderer node with the given `NodeParams`. The `params`
    /// argument is typically constructed via [`NodeParams::sprite`].
    pub fn sprite(params: NodeParams) -> Self {
        Self {
            node_type: super::enums::EffectNodeType::Sprite,
            children: Vec::new(),
            params: Some(params),
        }
    }
}

impl NodeParams {
    /// Sprite-emitter node parameters composed from `spec`. All other
    /// fields receive safe defaults: `is_rendered: true`, no kill rules,
    /// no collisions, no sound, no LODs, no GPU particles, no custom data,
    /// no rotation, spawn at the emitter origin.
    ///
    /// Authors override individual safe-default fields after construction
    /// (e.g. `params.rotation = ...; params.spawn_location = ...`) when
    /// they need behavior outside the common case.
    pub fn sprite(spec: SpriteNodeSpec) -> Self {
        Self {
            is_rendered: true,
            rendering_priority: 0,
            common_values: spec.common_values,
            steering_behavior: None,
            lods: None,
            translation: spec.translation,
            force_fields: Vec::new(),
            rotation: RotationParameter::default(),
            scaling: spec.scaling,
            spawn_location: SpawnLocationParameter::point_at_origin(),
            spawn_effects_rotation: false,
            depth: ParameterDepthValues::default(),
            kill_rules: KillRulesParameter::default(),
            collisions: CollisionsParameter::default(),
            renderer_common: spec.renderer_common,
            alpha_cutoff: AlphaCutoffParameter::default(),
            falloff: None,
            soft_particle: SoftParticleParam::default(),
            renderer: RendererVariant::Sprite(spec.sprite),
            sound: ParameterSound::default(),
            gpu_particles: None,
            custom_data1: ParameterCustomData::default(),
            custom_data2: ParameterCustomData::default(),
        }
    }
}
