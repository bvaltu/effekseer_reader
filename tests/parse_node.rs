//! Integration tests for Phase 3 — node parameters parsing.

use effekseer_reader::types::ParseConfig;
use effekseer_reader::types::enums::EffectNodeType;

// ─── Test data builders ───

/// Build a minimal V17 (version 1500) SKFE binary with a root node that has
/// one NoneType child containing minimal parameter data.
fn build_skfe_with_node(version: i32, child_node_data: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();

    // Header
    buf.extend_from_slice(b"SKFE");
    buf.extend_from_slice(&version.to_le_bytes());

    // 6 empty resource tables
    for _ in 0..6 {
        buf.extend_from_slice(&0i32.to_le_bytes());
    }

    // Curves (version >= 1607)
    if version >= 1607 {
        buf.extend_from_slice(&0i32.to_le_bytes());
    }
    // Procedural models (version >= 1607)
    if version >= 1607 {
        buf.extend_from_slice(&0i32.to_le_bytes());
    }

    // Dynamic inputs: 0
    buf.extend_from_slice(&0i32.to_le_bytes());
    // Dynamic equations: 0
    buf.extend_from_slice(&0i32.to_le_bytes());

    // Rendering optimization
    buf.extend_from_slice(&0i32.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());

    // Magnification, seed
    buf.extend_from_slice(&1.0f32.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());

    // Culling: none
    buf.extend_from_slice(&0i32.to_le_bytes());

    // LOD distances (version >= 1702)
    if version >= 1702 {
        buf.extend_from_slice(&0.0f32.to_le_bytes());
        buf.extend_from_slice(&0.0f32.to_le_bytes());
        buf.extend_from_slice(&0.0f32.to_le_bytes());
    }

    // Root node: type=-1, child_count=1
    buf.extend_from_slice(&(-1i32).to_le_bytes());
    buf.extend_from_slice(&1i32.to_le_bytes());

    // Child node data
    buf.extend_from_slice(child_node_data);

    buf
}

/// Build a minimal NoneType (0) node binary for version 1500.
/// NoneType has all parameter sections but no renderer data.
fn build_none_node_v1500() -> Vec<u8> {
    let mut buf = Vec::new();

    // node_type: NoneType (0)
    buf.extend_from_slice(&0i32.to_le_bytes());

    // [1] IsRendered: 0
    buf.extend_from_slice(&0i32.to_le_bytes());
    // [2] RenderingPriority: 0
    buf.extend_from_slice(&0i32.to_le_bytes());

    // [3] CommonValues (V17 layout, 80 bytes)
    append_v17_common_values(&mut buf);

    // [4] SteeringBehaviorParam: NOT present (translation_bind_type is Always=2, not FollowParent)
    // [5] TriggerParam: NOT present (version < 1700)
    // [6] LODsParam: NOT present (version < 1702)

    // [7] Translation: None (0x7ffffffe)
    buf.extend_from_slice(&0x7ffffffei32.to_le_bytes()); // type
    buf.extend_from_slice(&0i32.to_le_bytes()); // size: 0

    // [8] LocalForceField: count=4, all None, plus LocationAbs legacy block
    buf.extend_from_slice(&4i32.to_le_bytes()); // count
    for _ in 0..4 {
        // type: None (0) -- only field for version < 1600
        buf.extend_from_slice(&0i32.to_le_bytes());
    }
    // LocationAbs legacy block (version <= 1600): None
    buf.extend_from_slice(&0i32.to_le_bytes()); // type: None
    buf.extend_from_slice(&0i32.to_le_bytes()); // size: 0

    // [9] Rotation: None (0x7ffffffe)
    buf.extend_from_slice(&0x7ffffffei32.to_le_bytes()); // type
    buf.extend_from_slice(&0i32.to_le_bytes()); // size: 0

    // [10] Scaling: None (0x7ffffffe)
    buf.extend_from_slice(&0x7ffffffei32.to_le_bytes()); // type
    buf.extend_from_slice(&0i32.to_le_bytes()); // size: 0

    // [11] GenerationLocation: EffectsRotation=0, type=Point(0), location=zero RandomVector3D
    buf.extend_from_slice(&0i32.to_le_bytes()); // effects_rotation
    buf.extend_from_slice(&0i32.to_le_bytes()); // type: Point
    // RandomVector3D: max(0,0,0) min(0,0,0)
    for _ in 0..6 {
        buf.extend_from_slice(&0.0f32.to_le_bytes());
    }

    // [12] DepthValues: 32 bytes
    for _ in 0..8 {
        buf.extend_from_slice(&0i32.to_le_bytes());
    }

    // [13] KillParam: NOT present (version < 1704)
    // [14] Collisions: NOT present (version < 1801)

    // [15] RendererCommon
    append_renderer_common_v1500(&mut buf);

    // [16] AlphaCutoff: NOT present (version < 1605)
    // [17] Falloff: NOT present (version < 1602)

    // [18] Soft Particle: NOT present (version < 1603)

    // [19] Renderer: NoneType reads just a type check i32
    buf.extend_from_slice(&0i32.to_le_bytes());

    // [20] Sound: type=None(0)
    buf.extend_from_slice(&0i32.to_le_bytes());

    // [21] GPU Particles: NOT present (version < 1800)

    // child_count: 0
    buf.extend_from_slice(&0i32.to_le_bytes());

    buf
}

/// Append V17 CommonValues (80 bytes) with size prefix.
fn append_v17_common_values(buf: &mut Vec<u8>) {
    // size: 80
    buf.extend_from_slice(&80i32.to_le_bytes());
    // ref_eq_max_generation: -1
    buf.extend_from_slice(&(-1i32).to_le_bytes());
    // ref_eq_life: { -1, -1 }
    buf.extend_from_slice(&(-1i32).to_le_bytes());
    buf.extend_from_slice(&(-1i32).to_le_bytes());
    // ref_eq_generation_time: { -1, -1 }
    buf.extend_from_slice(&(-1i32).to_le_bytes());
    buf.extend_from_slice(&(-1i32).to_le_bytes());
    // ref_eq_generation_time_offset: { -1, -1 }
    buf.extend_from_slice(&(-1i32).to_le_bytes());
    buf.extend_from_slice(&(-1i32).to_le_bytes());
    // max_generation: 10
    buf.extend_from_slice(&10i32.to_le_bytes());
    // translation_bind_type: Always (2)
    buf.extend_from_slice(&2i32.to_le_bytes());
    // rotation_bind_type: Always (2)
    buf.extend_from_slice(&2i32.to_le_bytes());
    // scaling_bind_type: Always (2)
    buf.extend_from_slice(&2i32.to_le_bytes());
    // remove_when_life_is_extinct: 1
    buf.extend_from_slice(&1i32.to_le_bytes());
    // remove_when_parent_is_removed: 0
    buf.extend_from_slice(&0i32.to_le_bytes());
    // remove_when_children_is_extinct: 0
    buf.extend_from_slice(&0i32.to_le_bytes());
    // life: { max: 60, min: 60 }
    buf.extend_from_slice(&60i32.to_le_bytes());
    buf.extend_from_slice(&60i32.to_le_bytes());
    // generation_time: { max: 1.0, min: 1.0 }
    buf.extend_from_slice(&1.0f32.to_le_bytes());
    buf.extend_from_slice(&1.0f32.to_le_bytes());
    // generation_time_offset: { max: 0.0, min: 0.0 }
    buf.extend_from_slice(&0.0f32.to_le_bytes());
    buf.extend_from_slice(&0.0f32.to_le_bytes());
}

/// Append a minimal RendererCommon for version 1500.
fn append_renderer_common_v1500(buf: &mut Vec<u8>) {
    // [1] MaterialType: Default (0)
    buf.extend_from_slice(&0i32.to_le_bytes());
    // No EmissiveScaling (version < 1600)

    // [3] ColorTextureIndex: -1
    buf.extend_from_slice(&(-1i32).to_le_bytes());
    // NormalTextureIndex: -1
    buf.extend_from_slice(&(-1i32).to_le_bytes());
    // No extra textures (version < 1600)

    // [5] AlphaBlend: Opacity (0)
    buf.extend_from_slice(&0i32.to_le_bytes());

    // [6] TextureFilter[0], TextureWrap[0], TextureFilter[1], TextureWrap[1]
    buf.extend_from_slice(&0i32.to_le_bytes()); // Filter[0]: Nearest
    buf.extend_from_slice(&0i32.to_le_bytes()); // Wrap[0]: Repeat
    buf.extend_from_slice(&0i32.to_le_bytes()); // Filter[1]
    buf.extend_from_slice(&0i32.to_le_bytes()); // Wrap[1]
    // No extra filter/wrap (version < 1600)

    // [7] ZTest: 1 (true)
    buf.extend_from_slice(&1i32.to_le_bytes());
    // [8] ZWrite: 0 (false)
    buf.extend_from_slice(&0i32.to_le_bytes());

    // [9] FadeIn: None (0)
    buf.extend_from_slice(&0i32.to_le_bytes());
    // [10] FadeOut: None (0)
    buf.extend_from_slice(&0i32.to_le_bytes());

    // [11] UV[0]: Default (0)
    buf.extend_from_slice(&0i32.to_le_bytes());
    // No UV[1..4] (version < 1600)

    // No UVHorizontalFlipProbability (version < 1801)
    // [13] ColorBindType: Always (2)
    buf.extend_from_slice(&2i32.to_le_bytes());
    // [14] DistortionIntensity: 0.0
    buf.extend_from_slice(&0.0f32.to_le_bytes());

    // [15] CustomData1: None (0)
    buf.extend_from_slice(&0i32.to_le_bytes());
    // [16] CustomData2: None (0)
    buf.extend_from_slice(&0i32.to_le_bytes());
}

// ─── Tests ───

#[test]
fn test_parse_root_with_no_children() {
    let _skfe = build_skfe_with_node(1500, &[]);
    // build_skfe_with_node always sets child_count=1.
    // For no children, use a simple builder.
    let mut buf = Vec::new();
    buf.extend_from_slice(b"SKFE");
    buf.extend_from_slice(&1500i32.to_le_bytes());
    for _ in 0..6 {
        buf.extend_from_slice(&0i32.to_le_bytes());
    }
    buf.extend_from_slice(&0i32.to_le_bytes()); // dynamic inputs
    buf.extend_from_slice(&0i32.to_le_bytes()); // dynamic equations
    buf.extend_from_slice(&0i32.to_le_bytes()); // rendering node_count
    buf.extend_from_slice(&0i32.to_le_bytes()); // rendering threshold
    buf.extend_from_slice(&1.0f32.to_le_bytes()); // magnification
    buf.extend_from_slice(&0i32.to_le_bytes()); // seed
    buf.extend_from_slice(&0i32.to_le_bytes()); // culling: none
    // Root: type=-1, child_count=0
    buf.extend_from_slice(&(-1i32).to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());

    let effect = effekseer_reader::load_efk(&buf).unwrap();
    assert_eq!(effect.root.node_type, EffectNodeType::Root);
    assert!(effect.root.children.is_empty());
    assert!(effect.root.params.is_none());
}

#[test]
fn test_parse_root_with_one_nonetype_child_v1500() {
    let child = build_none_node_v1500();
    let skfe = build_skfe_with_node(1500, &child);
    let effect = effekseer_reader::load_efk(&skfe).unwrap();

    assert_eq!(effect.root.node_type, EffectNodeType::Root);
    assert_eq!(effect.root.children.len(), 1);

    let child = &effect.root.children[0];
    assert_eq!(child.node_type, EffectNodeType::NoneType);
    assert!(child.params.is_some());

    let params = child.params.as_ref().unwrap();
    assert!(!params.is_rendered);
    assert_eq!(params.rendering_priority, 0);
    assert_eq!(params.common_values.max_generation, 10);
    assert_eq!(params.common_values.life.max, 60);
    assert_eq!(params.common_values.life.min, 60);
    assert!(params.steering_behavior.is_none());
    assert!(params.lods.is_none());
    assert!(params.falloff.is_none());
    assert!(params.gpu_particles.is_none());
    assert!(!params.spawn_effects_rotation);
}

#[test]
fn test_node_depth_limit() {
    let config = ParseConfig {
        limits: effekseer_reader::types::ResourceLimits {
            max_node_depth: 2,
            ..Default::default()
        },
        ..Default::default()
    };

    // Build a chain: Root -> NoneType -> NoneType -> NoneType (depth 3)
    let grandchild = build_none_node_v1500();
    // Build a NoneType that has 1 child (the grandchild)
    let child = build_none_node_intermediate_v1500(&grandchild);

    // Build a NoneType that has 1 child (the child above)
    let parent = build_none_node_intermediate_v1500(&child);

    let skfe = build_skfe_with_node(1500, &parent);
    let result = effekseer_reader::parser::load_efk_with_config(&skfe, &config);

    assert!(result.is_err());
    match result.unwrap_err() {
        effekseer_reader::error::Error::ResourceLimitExceeded { field, .. } => {
            assert_eq!(field, "node_depth");
        }
        e => panic!("Expected ResourceLimitExceeded, got: {e:?}"),
    }
}

/// Build a NoneType node for v1500 that contains the given child data.
fn build_none_node_intermediate_v1500(child_data: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();

    // node_type: NoneType (0)
    buf.extend_from_slice(&0i32.to_le_bytes());

    // [1] IsRendered: 0
    buf.extend_from_slice(&0i32.to_le_bytes());
    // [2] RenderingPriority: 0
    buf.extend_from_slice(&0i32.to_le_bytes());

    // [3] CommonValues
    append_v17_common_values(&mut buf);

    // [7] Translation: None
    buf.extend_from_slice(&0x7ffffffei32.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());

    // [8] LocalForceField: count=4, all None + legacy block
    buf.extend_from_slice(&4i32.to_le_bytes());
    for _ in 0..4 {
        buf.extend_from_slice(&0i32.to_le_bytes()); // type: None (only field for v < 1600)
    }
    buf.extend_from_slice(&0i32.to_le_bytes()); // LocationAbs: None
    buf.extend_from_slice(&0i32.to_le_bytes()); // size: 0

    // [9] Rotation: None
    buf.extend_from_slice(&0x7ffffffei32.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());

    // [10] Scaling: None
    buf.extend_from_slice(&0x7ffffffei32.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());

    // [11] GenerationLocation: Point
    buf.extend_from_slice(&0i32.to_le_bytes()); // effects_rotation
    buf.extend_from_slice(&0i32.to_le_bytes()); // type: Point
    for _ in 0..6 {
        buf.extend_from_slice(&0.0f32.to_le_bytes());
    }

    // [12] DepthValues
    for _ in 0..8 {
        buf.extend_from_slice(&0i32.to_le_bytes());
    }

    // [15] RendererCommon
    append_renderer_common_v1500(&mut buf);

    // [19] Renderer: NoneType
    buf.extend_from_slice(&0i32.to_le_bytes());

    // [20] Sound: None
    buf.extend_from_slice(&0i32.to_le_bytes());

    // child_count: 1
    buf.extend_from_slice(&1i32.to_le_bytes());
    // child data
    buf.extend_from_slice(child_data);

    buf
}

#[test]
fn test_parse_fixed_translation() {
    // Build a node with Fixed translation
    let mut child = Vec::new();
    child.extend_from_slice(&0i32.to_le_bytes()); // NoneType
    child.extend_from_slice(&1i32.to_le_bytes()); // IsRendered
    child.extend_from_slice(&0i32.to_le_bytes()); // RenderingPriority
    append_v17_common_values(&mut child);

    // Translation: Fixed (0)
    child.extend_from_slice(&0i32.to_le_bytes()); // type: Fixed
    child.extend_from_slice(&16i32.to_le_bytes()); // size: 16 bytes (refEq + Vector3D)
    child.extend_from_slice(&(-1i32).to_le_bytes()); // ref_eq: -1
    child.extend_from_slice(&1.0f32.to_le_bytes()); // x
    child.extend_from_slice(&2.0f32.to_le_bytes()); // y
    child.extend_from_slice(&3.0f32.to_le_bytes()); // z

    // Rest of node: force fields, rotation, scaling, spawn, etc.
    append_rest_of_node_v1500(&mut child);

    let skfe = build_skfe_with_node(1500, &child);
    let effect = effekseer_reader::load_efk(&skfe).unwrap();
    let params = effect.root.children[0].params.as_ref().unwrap();

    match &params.translation {
        effekseer_reader::types::params::TranslationParameter::Fixed { ref_eq, position } => {
            assert_eq!(*ref_eq, -1);
            assert!((position.x - 1.0).abs() < f32::EPSILON);
            assert!((position.y - 2.0).abs() < f32::EPSILON);
            assert!((position.z - 3.0).abs() < f32::EPSILON);
        }
        other => panic!("Expected Fixed, got: {other:?}"),
    }
}

/// Append the remaining node sections after translation for v1500.
fn append_rest_of_node_v1500(buf: &mut Vec<u8>) {
    // [8] LocalForceField (v1500: only type per element, no power/pos/rot)
    buf.extend_from_slice(&4i32.to_le_bytes());
    for _ in 0..4 {
        buf.extend_from_slice(&0i32.to_le_bytes()); // type: None
    }
    buf.extend_from_slice(&0i32.to_le_bytes()); // LocationAbs: None
    buf.extend_from_slice(&0i32.to_le_bytes()); // size

    // [9] Rotation: None
    buf.extend_from_slice(&0x7ffffffei32.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());

    // [10] Scaling: None
    buf.extend_from_slice(&0x7ffffffei32.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());

    // [11] GenerationLocation
    buf.extend_from_slice(&0i32.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());
    for _ in 0..6 {
        buf.extend_from_slice(&0.0f32.to_le_bytes());
    }

    // [12] Depth
    for _ in 0..8 {
        buf.extend_from_slice(&0i32.to_le_bytes());
    }

    // [15] RendererCommon
    append_renderer_common_v1500(buf);

    // [19] Renderer: NoneType
    buf.extend_from_slice(&0i32.to_le_bytes());

    // [20] Sound: None
    buf.extend_from_slice(&0i32.to_le_bytes());

    // child_count: 0
    buf.extend_from_slice(&0i32.to_le_bytes());
}
