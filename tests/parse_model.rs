//! Integration tests for .efkmodel model file parsing.

use effekseer_reader::types::ParseConfig;
use effekseer_reader::types::primitives::{Color, Vector2D};
use effekseer_reader::{load_model, load_model_with_config};

/// Build a synthetic .efkmodel binary.
fn build_model(
    version: i32,
    frame_count: usize,
    vertices_per_frame: usize,
    faces_per_frame: usize,
) -> Vec<u8> {
    let mut data = Vec::new();

    // version
    data.extend_from_slice(&version.to_le_bytes());

    // scale (version == 2 or version >= 5)
    if version == 2 || version >= 5 {
        data.extend_from_slice(&1i32.to_le_bytes());
    }

    // model_count (backward compat)
    data.extend_from_slice(&1i32.to_le_bytes());

    // frame_count (version >= 5)
    if version >= 5 {
        data.extend_from_slice(&(frame_count as i32).to_le_bytes());
    }

    for _ in 0..frame_count {
        data.extend_from_slice(&(vertices_per_frame as i32).to_le_bytes());

        for i in 0..vertices_per_frame {
            let x = i as f32;
            // position
            data.extend_from_slice(&x.to_le_bytes());
            data.extend_from_slice(&0.0f32.to_le_bytes());
            data.extend_from_slice(&0.0f32.to_le_bytes());
            // normal
            data.extend_from_slice(&0.0f32.to_le_bytes());
            data.extend_from_slice(&1.0f32.to_le_bytes());
            data.extend_from_slice(&0.0f32.to_le_bytes());
            // binormal
            data.extend_from_slice(&0.0f32.to_le_bytes());
            data.extend_from_slice(&0.0f32.to_le_bytes());
            data.extend_from_slice(&1.0f32.to_le_bytes());
            // tangent
            data.extend_from_slice(&1.0f32.to_le_bytes());
            data.extend_from_slice(&0.0f32.to_le_bytes());
            data.extend_from_slice(&0.0f32.to_le_bytes());
            // uv1
            data.extend_from_slice(&0.5f32.to_le_bytes());
            data.extend_from_slice(&0.5f32.to_le_bytes());

            if version >= 6 {
                // uv2
                data.extend_from_slice(&0.25f32.to_le_bytes());
                data.extend_from_slice(&0.75f32.to_le_bytes());
            }

            if version >= 1 {
                // vcolor
                data.push(255);
                data.push(128);
                data.push(64);
                data.push(255);
            }
        }

        // faces
        data.extend_from_slice(&(faces_per_frame as i32).to_le_bytes());
        for _ in 0..faces_per_frame {
            data.extend_from_slice(&0i32.to_le_bytes());
            data.extend_from_slice(&1i32.to_le_bytes());
            data.extend_from_slice(&2i32.to_le_bytes());
        }
    }

    data
}

#[test]
fn test_load_model_v6_single_frame() {
    let data = build_model(6, 1, 4, 2);
    let model = load_model(&data).unwrap();

    assert_eq!(model.version, 6);
    assert_eq!(model.frames.len(), 1);
    assert_eq!(model.frames[0].vertices.len(), 4);
    assert_eq!(model.frames[0].faces.len(), 2);
}

#[test]
fn test_load_model_v6_multiple_frames() {
    let data = build_model(6, 3, 3, 1);
    let model = load_model(&data).unwrap();

    assert_eq!(model.frames.len(), 3);
    for frame in &model.frames {
        assert_eq!(frame.vertices.len(), 3);
        assert_eq!(frame.faces.len(), 1);
    }
}

#[test]
fn test_load_model_v5_uv2_equals_uv1() {
    let data = build_model(5, 1, 3, 1);
    let model = load_model(&data).unwrap();

    for v in &model.frames[0].vertices {
        assert_eq!(v.uv1, v.uv2, "version 5: uv2 should equal uv1");
    }
}

#[test]
fn test_load_model_v6_separate_uv2() {
    let data = build_model(6, 1, 3, 1);
    let model = load_model(&data).unwrap();

    let v = &model.frames[0].vertices[0];
    assert_eq!(v.uv1, Vector2D { x: 0.5, y: 0.5 });
    assert_eq!(v.uv2, Vector2D { x: 0.25, y: 0.75 });
}

#[test]
fn test_load_model_v0_default_white_vcolor() {
    let data = build_model(0, 1, 2, 1);
    let model = load_model(&data).unwrap();

    for v in &model.frames[0].vertices {
        assert_eq!(
            v.vcolor,
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255
            }
        );
        assert_eq!(v.uv1, v.uv2);
    }
}

#[test]
fn test_load_model_v1_reads_vcolor() {
    let data = build_model(1, 1, 2, 1);
    let model = load_model(&data).unwrap();

    assert_eq!(
        model.frames[0].vertices[0].vcolor,
        Color {
            r: 255,
            g: 128,
            b: 64,
            a: 255
        }
    );
}

#[test]
fn test_reject_version_7() {
    let mut data = Vec::new();
    data.extend_from_slice(&7i32.to_le_bytes());
    let result = load_model(&data);
    assert!(result.is_err());
}

#[test]
fn test_vertex_resource_limit() {
    let mut config = ParseConfig::default();
    config.limits.max_vertices_per_frame = 2;

    let data = build_model(6, 1, 3, 1);
    let result = load_model_with_config(&data, &config);
    assert!(result.is_err());
}

#[test]
fn test_face_resource_limit() {
    let mut config = ParseConfig::default();
    config.limits.max_faces_per_frame = 0;

    let data = build_model(6, 1, 3, 1);
    let result = load_model_with_config(&data, &config);
    assert!(result.is_err());
}

#[test]
fn test_frame_count_resource_limit() {
    let mut config = ParseConfig::default();
    config.limits.max_frame_count = 1;

    let data = build_model(6, 3, 3, 1);
    let result = load_model_with_config(&data, &config);
    assert!(result.is_err());
}

#[test]
fn test_vertex_positions_sequential() {
    let data = build_model(6, 1, 5, 1);
    let model = load_model(&data).unwrap();

    for (i, v) in model.frames[0].vertices.iter().enumerate() {
        assert_eq!(v.position.x, i as f32);
        assert_eq!(v.position.y, 0.0);
        assert_eq!(v.position.z, 0.0);
    }
}

#[test]
fn test_face_indices() {
    let data = build_model(6, 1, 3, 2);
    let model = load_model(&data).unwrap();

    for face in &model.frames[0].faces {
        assert_eq!(face.indexes, [0, 1, 2]);
    }
}

#[test]
fn test_model_v2_has_scale_field() {
    // Version 2 should read scale field without error
    let data = build_model(2, 1, 2, 1);
    let model = load_model(&data).unwrap();
    assert_eq!(model.version, 2);
}

#[test]
fn test_model_v3_no_scale_field() {
    // Version 3 does NOT have scale or frame_count fields
    let data = build_model(3, 1, 2, 1);
    let model = load_model(&data).unwrap();
    assert_eq!(model.version, 3);
    // frame_count defaults to 1 for version < 5
    assert_eq!(model.frames.len(), 1);
}

// ─── Real sample file tests ───

fn load_test_file(name: &str) -> Vec<u8> {
    let path = format!("tests/test_data/{name}");
    std::fs::read(&path).unwrap_or_else(|e| panic!("Failed to read {path}: {e}"))
}

#[test]
fn test_sample_block_efkmodel() {
    let data = load_test_file("Block.efkmodel");
    let model = load_model(&data).unwrap();

    assert_eq!(model.version, 0);
    assert_eq!(model.frames.len(), 1);
    assert!(
        !model.frames[0].vertices.is_empty(),
        "model should have vertices"
    );
    assert!(!model.frames[0].faces.is_empty(), "model should have faces");

    // Verify vertex data is reasonable (finite positions and normals)
    for v in &model.frames[0].vertices {
        assert!(v.position.x.is_finite(), "position.x should be finite");
        assert!(v.position.y.is_finite(), "position.y should be finite");
        assert!(v.position.z.is_finite(), "position.z should be finite");
        assert!(v.normal.x.is_finite(), "normal.x should be finite");
        assert!(v.normal.y.is_finite(), "normal.y should be finite");
        assert!(v.normal.z.is_finite(), "normal.z should be finite");
    }

    // v0 defaults to white vcolor
    for v in &model.frames[0].vertices {
        assert_eq!(
            v.vcolor,
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255
            }
        );
    }
}

#[test]
fn test_sample_sample1_efkmodel() {
    let data = load_test_file("Sample1.efkmodel");
    let model = load_model(&data).unwrap();

    assert_eq!(model.version, 5);
    assert_eq!(model.frames.len(), 1);

    let frame = &model.frames[0];
    assert!(!frame.vertices.is_empty(), "Sample1 should have vertices");
    assert!(!frame.faces.is_empty(), "Sample1 should have faces");

    // Verify all vertex data is finite
    for v in &frame.vertices {
        assert!(v.position.x.is_finite());
        assert!(v.position.y.is_finite());
        assert!(v.position.z.is_finite());
        assert!(v.normal.x.is_finite());
        assert!(v.normal.y.is_finite());
        assert!(v.normal.z.is_finite());
        assert!(v.uv1.x.is_finite());
        assert!(v.uv1.y.is_finite());
        // v5: uv2 should equal uv1
        assert_eq!(v.uv1, v.uv2, "v5: uv2 should equal uv1");
    }

    // Verify face indices are within bounds
    let vert_count = frame.vertices.len() as i32;
    for face in &frame.faces {
        for &idx in &face.indexes {
            assert!(
                idx >= 0 && idx < vert_count,
                "face index {idx} out of bounds (0..{vert_count})"
            );
        }
    }
}
