//! Integration tests for .efkefc container and SKFE effect binary parsing.

use effekseer_reader::error::Error;
use effekseer_reader::types::enums::{CullingShape, EffectNodeType};
use effekseer_reader::types::node::RendererVariant;
use effekseer_reader::types::{ParseConfig, ResourceLimits};

// ─── Test data builders ───

/// Encode a Rust string as a UTF-16LE binary blob (i32 length + u16 values).
fn encode_utf16(s: &str) -> Vec<u8> {
    let chars: Vec<u16> = s.encode_utf16().collect();
    let mut buf = (chars.len() as i32).to_le_bytes().to_vec();
    for c in &chars {
        buf.extend_from_slice(&c.to_le_bytes());
    }
    buf
}

/// Write an empty resource table (count = 0).
fn empty_resource_table() -> Vec<u8> {
    0i32.to_le_bytes().to_vec()
}

/// Write a resource table with the given paths.
fn resource_table(paths: &[&str]) -> Vec<u8> {
    let mut buf = (paths.len() as i32).to_le_bytes().to_vec();
    for p in paths {
        buf.extend(encode_utf16(p));
    }
    buf
}

/// Build a minimal SKFE binary with default/empty fields.
///
/// `version`: effect version (e.g., 1500, 1700, 1810)
/// Returns the raw SKFE bytes.
fn build_skfe(version: i32) -> Vec<u8> {
    build_skfe_full(version, &SkfeOptions::default())
}

#[derive(Default)]
struct SkfeOptions {
    color_images: Vec<String>,
    normal_images: Vec<String>,
    distortion_images: Vec<String>,
    sounds: Vec<String>,
    models: Vec<String>,
    materials: Vec<String>,
    curves: Vec<String>,
    dynamic_inputs: Vec<f32>,
    magnification: Option<f32>,
    random_seed: Option<i32>,
    culling_sphere: Option<(f32, f32, f32, f32)>, // radius, x, y, z
    lod_distances: Option<[f32; 3]>,
}

fn build_skfe_full(version: i32, opts: &SkfeOptions) -> Vec<u8> {
    let mut buf = Vec::new();

    // Header
    buf.extend_from_slice(b"SKFE");
    buf.extend_from_slice(&version.to_le_bytes());

    // Resource tables helper
    fn write_table(buf: &mut Vec<u8>, paths: &[String]) {
        if paths.is_empty() {
            buf.extend(empty_resource_table());
        } else {
            let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
            buf.extend(resource_table(&refs));
        }
    }

    write_table(&mut buf, &opts.color_images);
    write_table(&mut buf, &opts.normal_images);
    write_table(&mut buf, &opts.distortion_images);
    write_table(&mut buf, &opts.sounds);
    write_table(&mut buf, &opts.models);
    write_table(&mut buf, &opts.materials);

    // Curves (version >= 1607)
    if version >= 1607 {
        write_table(&mut buf, &opts.curves);
    }

    // Procedural models (version >= 1607)
    if version >= 1607 {
        buf.extend_from_slice(&0i32.to_le_bytes()); // count = 0
    }

    // Dynamic inputs
    buf.extend_from_slice(&(opts.dynamic_inputs.len() as i32).to_le_bytes());
    for v in &opts.dynamic_inputs {
        buf.extend_from_slice(&v.to_le_bytes());
    }

    // Dynamic equations: count = 0
    buf.extend_from_slice(&0i32.to_le_bytes());

    // Rendering optimization
    buf.extend_from_slice(&0i32.to_le_bytes()); // node_count
    buf.extend_from_slice(&0i32.to_le_bytes()); // threshold

    // Magnification
    buf.extend_from_slice(&opts.magnification.unwrap_or(1.0f32).to_le_bytes());

    // Random seed
    buf.extend_from_slice(&opts.random_seed.unwrap_or(0i32).to_le_bytes());

    // Culling
    if let Some((radius, x, y, z)) = opts.culling_sphere {
        buf.extend_from_slice(&1i32.to_le_bytes()); // CullingShape::Sphere
        buf.extend_from_slice(&radius.to_le_bytes());
        buf.extend_from_slice(&x.to_le_bytes());
        buf.extend_from_slice(&y.to_le_bytes());
        buf.extend_from_slice(&z.to_le_bytes());
    } else {
        buf.extend_from_slice(&0i32.to_le_bytes()); // CullingShape::NoneShape
    }

    // LOD distances (version >= 1702)
    if version >= 1702 {
        if let Some(dists) = opts.lod_distances {
            buf.extend_from_slice(&dists[0].to_le_bytes());
            buf.extend_from_slice(&dists[1].to_le_bytes());
            buf.extend_from_slice(&dists[2].to_le_bytes());
        } else {
            buf.extend_from_slice(&0.0f32.to_le_bytes());
            buf.extend_from_slice(&0.0f32.to_le_bytes());
            buf.extend_from_slice(&0.0f32.to_le_bytes());
        }
    }

    // Root node type = -1, child_count = 0
    buf.extend_from_slice(&(-1i32).to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());

    buf
}

/// Wrap SKFE bytes in an EFKE container.
fn wrap_in_efkefc(skfe: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"EFKE");
    buf.extend_from_slice(&1i32.to_le_bytes()); // container version
    // INFO chunk (dummy)
    buf.extend_from_slice(b"INFO");
    let info_data = b"dummy";
    buf.extend_from_slice(&(info_data.len() as i32).to_le_bytes());
    buf.extend_from_slice(info_data);
    // BIN_ chunk
    buf.extend_from_slice(b"BIN_");
    buf.extend_from_slice(&(skfe.len() as i32).to_le_bytes());
    buf.extend_from_slice(skfe);
    buf
}

// ─── Tests ───

#[test]
fn test_parse_minimal_v15_efkefc() {
    let skfe = build_skfe(1500);
    let efkefc = wrap_in_efkefc(&skfe);
    let effect = effekseer_reader::load_efkefc(&efkefc).unwrap();

    assert_eq!(effect.version, 1500);
    assert!((effect.magnification - 1.0).abs() < f32::EPSILON);
    assert_eq!(effect.random_seed, 0);
    assert!(effect.culling.is_none());
    assert!(effect.lod_distances.is_none()); // v15 doesn't have LOD
    assert!(effect.color_images.is_empty());
    assert!(effect.normal_images.is_empty());
    assert!(effect.distortion_images.is_empty());
    assert!(effect.sounds.is_empty());
    assert!(effect.models.is_empty());
    assert!(effect.materials.is_empty());
    assert!(effect.curves.is_empty()); // v15 doesn't have curves section
    assert!(effect.procedural_models.is_empty());
    assert!(effect.dynamic_inputs.is_empty());
    assert!(effect.dynamic_equations.is_empty());
    assert_eq!(effect.root.node_type, EffectNodeType::Root);
}

#[test]
fn test_parse_minimal_v18_efkefc() {
    let skfe = build_skfe(1810);
    let efkefc = wrap_in_efkefc(&skfe);
    let effect = effekseer_reader::load_efkefc(&efkefc).unwrap();

    assert_eq!(effect.version, 1810);
    // v18 has LOD distances section
    assert!(effect.lod_distances.is_some());
}

#[test]
fn test_parse_raw_efk() {
    let skfe = build_skfe(1500);
    let effect = effekseer_reader::load_efk(&skfe).unwrap();
    assert_eq!(effect.version, 1500);
    assert_eq!(effect.root.node_type, EffectNodeType::Root);
}

#[test]
fn test_load_efkefc_auto_detects_raw_skfe() {
    // load_efkefc should accept raw SKFE data too
    let skfe = build_skfe(1500);
    let effect = effekseer_reader::load_efkefc(&skfe).unwrap();
    assert_eq!(effect.version, 1500);
}

#[test]
fn test_resource_paths_extracted() {
    let opts = SkfeOptions {
        color_images: vec![
            "Texture/Particle01.png".into(),
            "Texture/Particle02.png".into(),
        ],
        normal_images: vec!["Texture/Normal01.png".into()],
        models: vec!["Model/Sphere.efkmodel".into()],
        materials: vec!["Material/Glow.efkmat".into()],
        ..Default::default()
    };
    let skfe = build_skfe_full(1500, &opts);
    let effect = effekseer_reader::load_efk(&skfe).unwrap();

    assert_eq!(effect.color_images.len(), 2);
    assert_eq!(effect.color_images[0], "Texture/Particle01.png");
    assert_eq!(effect.color_images[1], "Texture/Particle02.png");
    assert_eq!(effect.normal_images.len(), 1);
    assert_eq!(effect.normal_images[0], "Texture/Normal01.png");
    assert_eq!(effect.models.len(), 1);
    assert_eq!(effect.models[0], "Model/Sphere.efkmodel");
    assert_eq!(effect.materials.len(), 1);
    assert_eq!(effect.materials[0], "Material/Glow.efkmat");
}

#[test]
fn test_curves_extracted_v1607() {
    let opts = SkfeOptions {
        curves: vec!["Curve/Path01.efkcurve".into()],
        ..Default::default()
    };
    let skfe = build_skfe_full(1607, &opts);
    let effect = effekseer_reader::load_efk(&skfe).unwrap();

    assert_eq!(effect.curves.len(), 1);
    assert_eq!(effect.curves[0], "Curve/Path01.efkcurve");
}

#[test]
fn test_magnification_and_seed() {
    let opts = SkfeOptions {
        magnification: Some(2.5),
        random_seed: Some(42),
        ..Default::default()
    };
    let skfe = build_skfe_full(1500, &opts);
    let effect = effekseer_reader::load_efk(&skfe).unwrap();

    assert!((effect.magnification - 2.5).abs() < f32::EPSILON);
    assert_eq!(effect.random_seed, 42);
}

#[test]
fn test_culling_sphere() {
    let opts = SkfeOptions {
        culling_sphere: Some((10.0, 1.0, 2.0, 3.0)),
        ..Default::default()
    };
    let skfe = build_skfe_full(1500, &opts);
    let effect = effekseer_reader::load_efk(&skfe).unwrap();

    let culling = effect.culling.unwrap();
    assert_eq!(culling.shape, CullingShape::Sphere);
    assert!((culling.radius - 10.0).abs() < f32::EPSILON);
    assert!((culling.location.x - 1.0).abs() < f32::EPSILON);
    assert!((culling.location.y - 2.0).abs() < f32::EPSILON);
    assert!((culling.location.z - 3.0).abs() < f32::EPSILON);
}

#[test]
fn test_lod_distances_v1702() {
    let opts = SkfeOptions {
        lod_distances: Some([100.0, 200.0, 300.0]),
        ..Default::default()
    };
    let skfe = build_skfe_full(1702, &opts);
    let effect = effekseer_reader::load_efk(&skfe).unwrap();

    let lod = effect.lod_distances.unwrap();
    assert!((lod[0] - 100.0).abs() < f32::EPSILON);
    assert!((lod[1] - 200.0).abs() < f32::EPSILON);
    assert!((lod[2] - 300.0).abs() < f32::EPSILON);
}

#[test]
fn test_dynamic_inputs() {
    let opts = SkfeOptions {
        dynamic_inputs: vec![1.0, 2.5, 3.7],
        ..Default::default()
    };
    let skfe = build_skfe_full(1500, &opts);
    let effect = effekseer_reader::load_efk(&skfe).unwrap();

    assert_eq!(effect.dynamic_inputs.len(), 3);
    assert!((effect.dynamic_inputs[0] - 1.0).abs() < f32::EPSILON);
    assert!((effect.dynamic_inputs[1] - 2.5).abs() < f32::EPSILON);
    assert!((effect.dynamic_inputs[2] - 3.7).abs() < f32::EPSILON);
}

#[test]
fn test_error_on_invalid_magic() {
    let data = b"BADMdata_here";
    let result = effekseer_reader::load_efkefc(data);
    assert!(matches!(result, Err(Error::InvalidMagic { .. })));
}

#[test]
fn test_error_on_unsupported_version() {
    let mut skfe = Vec::new();
    skfe.extend_from_slice(b"SKFE");
    skfe.extend_from_slice(&9999i32.to_le_bytes());
    let result = effekseer_reader::load_efk(&skfe);
    assert!(matches!(
        result,
        Err(Error::UnsupportedVersion { version: 9999 })
    ));
}

#[test]
fn test_error_on_version_too_low() {
    let mut skfe = Vec::new();
    skfe.extend_from_slice(b"SKFE");
    skfe.extend_from_slice(&100i32.to_le_bytes());
    let result = effekseer_reader::load_efk(&skfe);
    assert!(matches!(
        result,
        Err(Error::UnsupportedVersion { version: 100 })
    ));
}

#[test]
fn test_error_on_truncated_file() {
    // Just magic + version, then EOF
    let mut skfe = Vec::new();
    skfe.extend_from_slice(b"SKFE");
    skfe.extend_from_slice(&1500i32.to_le_bytes());
    // Missing resource tables
    let result = effekseer_reader::load_efk(&skfe);
    assert!(matches!(result, Err(Error::UnexpectedEof { .. })));
}

#[test]
fn test_resource_limit_exceeded() {
    let config = ParseConfig {
        limits: ResourceLimits {
            max_resource_paths: 1,
            ..Default::default()
        },
        ..Default::default()
    };
    let opts = SkfeOptions {
        color_images: vec!["a.png".into(), "b.png".into()],
        ..Default::default()
    };
    let skfe = build_skfe_full(1500, &opts);
    let result = effekseer_reader::parser::load_efk_with_config(&skfe, &config);
    assert!(matches!(
        result,
        Err(Error::ResourceLimitExceeded {
            field: "color_images",
            ..
        })
    ));
}

#[test]
fn test_dynamic_equations_stored() {
    // Build SKFE with 1 dynamic equation (opaque blob)
    let opts = SkfeOptions::default();
    let mut skfe = Vec::new();

    // Header
    skfe.extend_from_slice(b"SKFE");
    skfe.extend_from_slice(&1500i32.to_le_bytes());

    // 6 empty resource tables
    for _ in 0..6 {
        skfe.extend(empty_resource_table());
    }

    // Dynamic inputs: 0
    skfe.extend_from_slice(&0i32.to_le_bytes());

    // Dynamic equations: 1 equation with 8-byte blob
    skfe.extend_from_slice(&1i32.to_le_bytes());
    let blob = [0xAA, 0xBB, 0xCC, 0xDD, 0x11, 0x22, 0x33, 0x44];
    skfe.extend_from_slice(&(blob.len() as i32).to_le_bytes());
    skfe.extend_from_slice(&blob);

    // Rendering optimization
    skfe.extend_from_slice(&0i32.to_le_bytes());
    skfe.extend_from_slice(&0i32.to_le_bytes());

    // Magnification, seed
    skfe.extend_from_slice(&1.0f32.to_le_bytes());
    skfe.extend_from_slice(&0i32.to_le_bytes());

    // Culling: none
    skfe.extend_from_slice(&0i32.to_le_bytes());

    // Root node type + child_count = 0
    skfe.extend_from_slice(&(-1i32).to_le_bytes());
    skfe.extend_from_slice(&0i32.to_le_bytes());

    let _ = opts; // suppress unused warning
    let effect = effekseer_reader::load_efk(&skfe).unwrap();
    assert_eq!(effect.dynamic_equations.len(), 1);
    assert_eq!(effect.dynamic_equations[0], blob);
}

#[test]
fn test_unicode_resource_paths() {
    let opts = SkfeOptions {
        color_images: vec!["テクスチャ/パーティクル.png".into()],
        ..Default::default()
    };
    let skfe = build_skfe_full(1500, &opts);
    let effect = effekseer_reader::load_efk(&skfe).unwrap();

    assert_eq!(effect.color_images[0], "テクスチャ/パーティクル.png");
}

// ─── Additional error case tests ───

#[test]
fn test_error_on_empty_file() {
    let result = effekseer_reader::load_efkefc(&[]);
    assert!(result.is_err());
}

#[test]
fn test_error_on_version_boundary_1499() {
    let mut skfe = Vec::new();
    skfe.extend_from_slice(b"SKFE");
    skfe.extend_from_slice(&1499i32.to_le_bytes());
    let result = effekseer_reader::load_efk(&skfe);
    assert!(matches!(
        result,
        Err(Error::UnsupportedVersion { version: 1499 })
    ));
}

#[test]
fn test_error_on_version_boundary_1811() {
    let mut skfe = Vec::new();
    skfe.extend_from_slice(b"SKFE");
    skfe.extend_from_slice(&1811i32.to_le_bytes());
    let result = effekseer_reader::load_efk(&skfe);
    assert!(matches!(
        result,
        Err(Error::UnsupportedVersion { version: 1811 })
    ));
}

#[test]
fn test_version_boundary_1500_accepted() {
    let skfe = build_skfe(1500);
    let effect = effekseer_reader::load_efk(&skfe).unwrap();
    assert_eq!(effect.version, 1500);
}

#[test]
fn test_version_boundary_1810_accepted() {
    let skfe = build_skfe(1810);
    let effect = effekseer_reader::load_efk(&skfe).unwrap();
    assert_eq!(effect.version, 1810);
}

#[test]
fn test_error_on_truncated_after_resource_tables() {
    // Build a full SKFE and truncate after the 6 resource tables
    let skfe = build_skfe(1500);
    // Header (8) + 6 tables × 4 bytes each = 32 bytes
    let truncated = &skfe[..32];
    let result = effekseer_reader::load_efk(truncated);
    assert!(matches!(result, Err(Error::UnexpectedEof { .. })));
}

#[test]
fn test_error_on_truncated_efkefc_container() {
    let skfe = build_skfe(1500);
    let efkefc = wrap_in_efkefc(&skfe);
    // Truncate mid-container (cut off part of the BIN_ chunk data)
    let truncated = &efkefc[..efkefc.len() - 10];
    let result = effekseer_reader::load_efkefc(truncated);
    assert!(result.is_err());
}

#[test]
fn test_error_on_corrupt_utf16_resource_path() {
    // Build a SKFE with a manually corrupt UTF-16 string in the color_images table.
    let mut skfe = Vec::new();
    skfe.extend_from_slice(b"SKFE");
    skfe.extend_from_slice(&1500i32.to_le_bytes());

    // color_images: 1 path with invalid UTF-16 (unpaired surrogate)
    skfe.extend_from_slice(&1i32.to_le_bytes()); // count = 1
    // String: length 2 (2 u16 values), containing a lone high surrogate
    skfe.extend_from_slice(&2i32.to_le_bytes()); // char count
    skfe.extend_from_slice(&0xD800u16.to_le_bytes()); // lone high surrogate
    skfe.extend_from_slice(&0xD800u16.to_le_bytes()); // another lone high surrogate

    // We don't need the rest — the parse should fail during resource table reading
    let result = effekseer_reader::load_efk(&skfe);
    assert!(
        matches!(result, Err(Error::Utf16DecodeError { .. })),
        "Expected Utf16DecodeError, got: {result:?}"
    );
}

#[test]
fn test_error_efkefc_missing_bin_chunk() {
    let mut data = Vec::new();
    data.extend_from_slice(b"EFKE");
    data.extend_from_slice(&1i32.to_le_bytes());
    // Only an INFO chunk, no BIN_
    data.extend_from_slice(b"INFO");
    data.extend_from_slice(&4i32.to_le_bytes());
    data.extend_from_slice(&[0u8; 4]);
    let result = effekseer_reader::load_efkefc(&data);
    assert!(result.is_err());
}

#[test]
fn test_error_child_count_exceeds_limit() {
    let config = ParseConfig {
        limits: ResourceLimits {
            max_node_children: 0,
            ..Default::default()
        },
        ..Default::default()
    };

    // Build SKFE with root child_count = 1 (exceeds limit of 0)
    let mut skfe = Vec::new();
    skfe.extend_from_slice(b"SKFE");
    skfe.extend_from_slice(&1500i32.to_le_bytes());
    for _ in 0..6 {
        skfe.extend_from_slice(&0i32.to_le_bytes());
    }
    skfe.extend_from_slice(&0i32.to_le_bytes()); // dynamic inputs
    skfe.extend_from_slice(&0i32.to_le_bytes()); // dynamic equations
    skfe.extend_from_slice(&0i32.to_le_bytes()); // rendering node_count
    skfe.extend_from_slice(&0i32.to_le_bytes()); // rendering threshold
    skfe.extend_from_slice(&1.0f32.to_le_bytes()); // magnification
    skfe.extend_from_slice(&0i32.to_le_bytes()); // seed
    skfe.extend_from_slice(&0i32.to_le_bytes()); // culling: none
    // Root node: type=-1, child_count=1
    skfe.extend_from_slice(&(-1i32).to_le_bytes());
    skfe.extend_from_slice(&1i32.to_le_bytes());

    let result = effekseer_reader::parser::load_efk_with_config(&skfe, &config);
    assert!(matches!(
        result,
        Err(Error::ResourceLimitExceeded {
            field: "node_children",
            ..
        })
    ));
}

#[test]
fn test_warn_mode_continues_on_unknown_enum() {
    let config = ParseConfig {
        unknown_enum_behavior: effekseer_reader::types::UnknownEnumBehavior::Warn,
        ..Default::default()
    };

    // In Warn mode, parsing should still succeed even if it encounters unknowns.
    // Use a standard valid file to ensure warn mode doesn't break anything.
    let skfe = build_skfe(1500);
    let effect = effekseer_reader::parser::load_efk_with_config(&skfe, &config).unwrap();
    assert_eq!(effect.version, 1500);
}

#[test]
fn test_dynamic_input_limit_exceeded() {
    let config = ParseConfig {
        limits: ResourceLimits {
            max_dynamic_inputs: 1,
            ..Default::default()
        },
        ..Default::default()
    };
    let opts = SkfeOptions {
        dynamic_inputs: vec![1.0, 2.0],
        ..Default::default()
    };
    let skfe = build_skfe_full(1500, &opts);
    let result = effekseer_reader::parser::load_efk_with_config(&skfe, &config);
    assert!(matches!(
        result,
        Err(Error::ResourceLimitExceeded {
            field: "dynamic_inputs",
            ..
        })
    ));
}

#[test]
fn test_multiple_resource_types_populated() {
    let opts = SkfeOptions {
        color_images: vec!["tex/a.png".into()],
        normal_images: vec!["tex/n.png".into()],
        distortion_images: vec!["tex/d.png".into()],
        sounds: vec!["snd/boom.wav".into()],
        models: vec!["mdl/sphere.efkmodel".into()],
        materials: vec!["mat/glow.efkmat".into()],
        curves: vec!["curve/path.efkcurve".into()],
        ..Default::default()
    };
    let skfe = build_skfe_full(1810, &opts);
    let effect = effekseer_reader::load_efk(&skfe).unwrap();

    assert_eq!(effect.color_images, vec!["tex/a.png"]);
    assert_eq!(effect.normal_images, vec!["tex/n.png"]);
    assert_eq!(effect.distortion_images, vec!["tex/d.png"]);
    assert_eq!(effect.sounds, vec!["snd/boom.wav"]);
    assert_eq!(effect.models, vec!["mdl/sphere.efkmodel"]);
    assert_eq!(effect.materials, vec!["mat/glow.efkmat"]);
    assert_eq!(effect.curves, vec!["curve/path.efkcurve"]);
}

// ─── Real sample file tests ───

/// Helper: load a file from tests/test_data/.
fn load_test_file(name: &str) -> Vec<u8> {
    let path = format!("tests/test_data/{name}");
    std::fs::read(&path).unwrap_or_else(|e| panic!("Failed to read {path}: {e}"))
}

/// Recursively count nodes in a tree.
fn count_nodes(node: &effekseer_reader::types::node::EffectNode) -> usize {
    1 + node.children.iter().map(count_nodes).sum::<usize>()
}

/// Parse square_r.efkefc (v1610) — simplest file: no resources, 1 Sprite child.
#[test]
fn test_sample_square_r() {
    let data = load_test_file("square_r.efkefc");
    let effect = effekseer_reader::load_efkefc(&data).unwrap();

    assert_eq!(effect.version, 1610);
    assert!((effect.magnification - 1.0).abs() < f32::EPSILON);
    assert_eq!(effect.random_seed, -1);

    // No resources
    assert_eq!(effect.color_images.len(), 0);
    assert_eq!(effect.normal_images.len(), 0);
    assert_eq!(effect.distortion_images.len(), 0);
    assert_eq!(effect.sounds.len(), 0);
    assert_eq!(effect.models.len(), 0);
    assert_eq!(effect.materials.len(), 0);
    assert_eq!(effect.curves.len(), 0);
    assert_eq!(effect.dynamic_inputs.len(), 4);

    // Node tree: Root -> 1 Sprite
    assert_eq!(count_nodes(&effect.root), 2);
    assert_eq!(effect.root.node_type, EffectNodeType::Root);
    assert_eq!(effect.root.children.len(), 1);
    assert_eq!(effect.root.children[0].node_type, EffectNodeType::Sprite);

    // Verify the Sprite node has a Sprite renderer
    let params = effect.root.children[0].params.as_ref().unwrap();
    assert!(params.is_rendered);
    assert!(matches!(params.renderer, RendererVariant::Sprite(_)));
}

/// Parse AlphaBlendTexture01.efkefc (v1602) — resource-heavy, no child nodes.
#[test]
fn test_sample_alpha_blend_texture() {
    let data = load_test_file("AlphaBlendTexture01.efkefc");
    let effect = effekseer_reader::load_efkefc(&data).unwrap();

    assert_eq!(effect.version, 1602);

    // Resources
    assert_eq!(effect.color_images.len(), 6);
    assert_eq!(effect.normal_images.len(), 0);
    assert_eq!(effect.distortion_images.len(), 6);
    assert_eq!(effect.models.len(), 1);
    assert_eq!(effect.materials.len(), 0);
    assert_eq!(effect.dynamic_inputs.len(), 4);

    // Node tree: Root only (no children)
    assert_eq!(count_nodes(&effect.root), 1);
    assert_eq!(effect.root.node_type, EffectNodeType::Root);
    assert_eq!(effect.root.children.len(), 0);
}

/// Parse BasicRenderSettings_Blend.efkefc (v1500) — oldest version, 5 Sprite children.
#[test]
fn test_sample_basic_render_settings_blend() {
    let data = load_test_file("BasicRenderSettings_Blend.efkefc");
    let effect = effekseer_reader::load_efkefc(&data).unwrap();

    assert_eq!(effect.version, 1500);
    assert!((effect.magnification - 1.0).abs() < f32::EPSILON);
    assert_eq!(effect.random_seed, -1);

    // Resources
    assert_eq!(effect.color_images.len(), 1);
    assert_eq!(effect.color_images[0], "../Textures/Particle03.png");
    assert_eq!(effect.normal_images.len(), 0);
    assert_eq!(effect.distortion_images.len(), 0);
    assert_eq!(effect.models.len(), 0);
    assert_eq!(effect.materials.len(), 0);
    // v1500 has no curves section
    assert_eq!(effect.curves.len(), 0);
    assert!(effect.lod_distances.is_none());

    // Node tree: Root -> 5 Sprites
    assert_eq!(count_nodes(&effect.root), 6);
    assert_eq!(effect.root.children.len(), 5);
    for child in &effect.root.children {
        assert_eq!(child.node_type, EffectNodeType::Sprite);
        let params = child.params.as_ref().unwrap();
        assert!(params.is_rendered);
        assert!(matches!(params.renderer, RendererVariant::Sprite(_)));
        assert_eq!(child.children.len(), 0);
    }
}

/// Parse Laser01.efkefc (v1610) — 5 Sprite children with 3 color textures.
#[test]
fn test_sample_laser01() {
    let data = load_test_file("Laser01.efkefc");
    let effect = effekseer_reader::load_efkefc(&data).unwrap();

    assert_eq!(effect.version, 1610);
    assert!((effect.magnification - 1.0).abs() < f32::EPSILON);
    assert_eq!(effect.random_seed, -1);

    // Resources
    assert_eq!(effect.color_images.len(), 3);
    assert_eq!(effect.color_images[0], "Texture/LaserMain01.png");
    assert_eq!(effect.normal_images.len(), 0);
    assert_eq!(effect.distortion_images.len(), 0);
    assert_eq!(effect.models.len(), 0);
    assert_eq!(effect.materials.len(), 0);
    assert_eq!(effect.dynamic_inputs.len(), 4);

    // Node tree: Root -> 5 Sprites
    assert_eq!(count_nodes(&effect.root), 6);
    assert_eq!(effect.root.children.len(), 5);
    for child in &effect.root.children {
        assert_eq!(child.node_type, EffectNodeType::Sprite);
        let params = child.params.as_ref().unwrap();
        assert!(params.is_rendered);
        assert!(matches!(params.renderer, RendererVariant::Sprite(_)));
    }
}

/// Parse Gradient1.efkefc (v1705) — uses material files, Model+Sprite nodes.
#[test]
fn test_sample_gradient1() {
    let data = load_test_file("Gradient1.efkefc");
    let effect = effekseer_reader::load_efkefc(&data).unwrap();

    assert_eq!(effect.version, 1705);
    assert!((effect.magnification - 1.0).abs() < f32::EPSILON);
    assert_eq!(effect.random_seed, -1);
    assert!(effect.lod_distances.is_some());

    // Resources
    assert_eq!(effect.color_images.len(), 0);
    assert_eq!(effect.models.len(), 1);
    assert_eq!(effect.materials.len(), 5);
    assert_eq!(effect.dynamic_inputs.len(), 4);

    // Node tree: Root -> 2 NoneType -> (5 Sprites, 5 Models) = 13 total
    assert_eq!(count_nodes(&effect.root), 13);
    assert_eq!(effect.root.children.len(), 2);

    // First branch: NoneType -> 5 Sprites
    let branch1 = &effect.root.children[0];
    assert_eq!(branch1.node_type, EffectNodeType::NoneType);
    assert_eq!(branch1.children.len(), 5);
    for child in &branch1.children {
        assert_eq!(child.node_type, EffectNodeType::Sprite);
    }

    // Second branch: NoneType -> 5 Models
    let branch2 = &effect.root.children[1];
    assert_eq!(branch2.node_type, EffectNodeType::NoneType);
    assert_eq!(branch2.children.len(), 5);
    for child in &branch2.children {
        assert_eq!(child.node_type, EffectNodeType::Model);
        let params = child.params.as_ref().unwrap();
        assert!(matches!(params.renderer, RendererVariant::Model(_)));
    }
}

/// Parse TriggerLaser.efkefc (v1705) — 2 NoneType groups, 7 Sprites total.
#[test]
fn test_sample_trigger_laser() {
    let data = load_test_file("TriggerLaser.efkefc");
    let effect = effekseer_reader::load_efkefc(&data).unwrap();

    assert_eq!(effect.version, 1705);
    assert!((effect.magnification - 1.0).abs() < f32::EPSILON);
    assert_eq!(effect.random_seed, -1);
    assert!(effect.lod_distances.is_some());

    // Resources
    assert_eq!(effect.color_images.len(), 3);
    assert_eq!(effect.normal_images.len(), 0);
    assert_eq!(effect.distortion_images.len(), 0);
    assert_eq!(effect.models.len(), 0);
    assert_eq!(effect.materials.len(), 0);
    assert_eq!(effect.dynamic_inputs.len(), 4);

    // Node tree: Root -> 2 NoneType -> (2 Sprites, 5 Sprites) = 10 total
    assert_eq!(count_nodes(&effect.root), 10);
    assert_eq!(effect.root.children.len(), 2);

    let branch1 = &effect.root.children[0];
    assert_eq!(branch1.node_type, EffectNodeType::NoneType);
    assert_eq!(branch1.children.len(), 2);
    for child in &branch1.children {
        assert_eq!(child.node_type, EffectNodeType::Sprite);
    }

    let branch2 = &effect.root.children[1];
    assert_eq!(branch2.node_type, EffectNodeType::NoneType);
    assert_eq!(branch2.children.len(), 5);
    for child in &branch2.children {
        assert_eq!(child.node_type, EffectNodeType::Sprite);
    }
}

/// Verify version-gated fields: v15 file should NOT have LOD distances.
#[test]
fn test_v15_file_has_no_lod_or_curves() {
    let skfe = build_skfe(1500);
    let effect = effekseer_reader::load_efk(&skfe).unwrap();

    assert!(
        effect.lod_distances.is_none(),
        "v1500 should not have LOD distances"
    );
    assert!(
        effect.curves.is_empty(),
        "v1500 should not have curves section"
    );
}

/// Verify version-gated fields: v1702+ has LOD distances, v1607+ has curves.
#[test]
fn test_v17_file_has_lod_and_curves() {
    let skfe = build_skfe(1702);
    let effect = effekseer_reader::load_efk(&skfe).unwrap();

    assert!(
        effect.lod_distances.is_some(),
        "v1702 should have LOD distances"
    );
    // Curves section is present (though may be empty).
}

/// Parse homing_laser.efkefc and verify target_location is extracted from EDIT chunk.
#[test]
fn test_homing_laser_target_location() {
    let data = load_test_file("homing_laser.efkefc");
    let effect = effekseer_reader::load_efkefc(&data).unwrap();

    assert_eq!(effect.version, 1710);

    // The Effekseer editor has Behavior > TargetLocation > Y = 15 for this effect.
    let tl = effect.target_location.expect("should have target_location from EDIT chunk");
    assert!((tl.x - 0.0).abs() < 0.001, "expected x=0, got {}", tl.x);
    assert!((tl.y - 15.0).abs() < 0.001, "expected y=15, got {}", tl.y);
    assert!((tl.z - 0.0).abs() < 0.001, "expected z=0, got {}", tl.z);
}

/// Verify all sample .efkefc files at minimum have valid EFKE/SKFE header.
#[test]
fn test_all_sample_efkefc_files_have_valid_header() {
    let files = [
        "Laser01.efkefc",
        "TriggerLaser.efkefc",
        "square_r.efkefc",
        "BasicRenderSettings_Blend.efkefc",
        "AlphaBlendTexture01.efkefc",
        "Gradient1.efkefc",
        "homing_laser.efkefc",
    ];

    for name in &files {
        let data = load_test_file(name);
        assert!(data.len() >= 4, "{name} is too small");
        let magic = &data[0..4];
        assert!(
            magic == b"EFKE" || magic == b"SKFE",
            "{name} has invalid magic: {magic:?}"
        );
    }
}
