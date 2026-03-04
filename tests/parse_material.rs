//! Integration tests for .efkmat material file parsing.

use effekseer_reader::load_material;

/// Build a synthetic .efkmat binary with configurable parameters.
fn build_material(
    version: i32,
    texture_count: usize,
    uniform_count: usize,
    gradient_count: usize,
    include_gene: bool,
) -> Vec<u8> {
    let mut data = Vec::new();

    // Header: EFKM + version + guid
    data.extend_from_slice(b"EFKM");
    data.extend_from_slice(&version.to_le_bytes());
    data.extend_from_slice(&1u32.to_le_bytes()); // guid lo
    data.extend_from_slice(&0u32.to_le_bytes()); // guid hi

    // PRM_ chunk
    let prm = build_prm_data(version, texture_count, uniform_count, gradient_count);
    data.extend_from_slice(b"PRM_");
    data.extend_from_slice(&(prm.len() as i32).to_le_bytes());
    data.extend_from_slice(&prm);

    // GENE chunk
    if include_gene {
        let code = b"$F4$(1,0,0,1)\0";
        data.extend_from_slice(b"GENE");
        let gene_size = 4 + code.len();
        data.extend_from_slice(&(gene_size as i32).to_le_bytes());
        data.extend_from_slice(&(code.len() as i32).to_le_bytes());
        data.extend_from_slice(code);
    }

    data
}

fn build_prm_data(
    version: i32,
    texture_count: usize,
    uniform_count: usize,
    gradient_count: usize,
) -> Vec<u8> {
    let mut prm = Vec::new();

    // shading_model: Lit (0)
    prm.extend_from_slice(&0i32.to_le_bytes());
    // has_normal
    prm.extend_from_slice(&1i32.to_le_bytes());
    // has_refraction: true (1)
    prm.extend_from_slice(&1i32.to_le_bytes());
    // custom_data_1_count: 2
    prm.extend_from_slice(&2i32.to_le_bytes());
    // custom_data_2_count: 4
    prm.extend_from_slice(&4i32.to_le_bytes());

    // Required methods (version >= 1703)
    if version >= 1703 {
        prm.extend_from_slice(&2i32.to_le_bytes()); // count: 2
        prm.extend_from_slice(&0i32.to_le_bytes()); // Gradient
        prm.extend_from_slice(&1i32.to_le_bytes()); // Noise
    }

    // Textures
    prm.extend_from_slice(&(texture_count as i32).to_le_bytes());
    for i in 0..texture_count {
        let name = format!("Texture{}", i);
        prm.extend_from_slice(&(name.len() as i32).to_le_bytes());
        prm.extend_from_slice(name.as_bytes());

        if version >= 3 {
            let uname = format!("efk_tex_{}", i);
            prm.extend_from_slice(&(uname.len() as i32).to_le_bytes());
            prm.extend_from_slice(uname.as_bytes());
        }

        // default_path (empty)
        prm.extend_from_slice(&0i32.to_le_bytes());
        // index
        prm.extend_from_slice(&(i as i32).to_le_bytes());
        // priority
        prm.extend_from_slice(&0i32.to_le_bytes());
        // param
        prm.extend_from_slice(&0i32.to_le_bytes());
        // color_type: Color (0)
        prm.extend_from_slice(&0i32.to_le_bytes());
        // sampler: Clamp (1)
        prm.extend_from_slice(&1i32.to_le_bytes());
    }

    // Uniforms
    prm.extend_from_slice(&(uniform_count as i32).to_le_bytes());
    for i in 0..uniform_count {
        let name = format!("Uniform{}", i);
        prm.extend_from_slice(&(name.len() as i32).to_le_bytes());
        prm.extend_from_slice(name.as_bytes());

        if version >= 3 {
            let uname = format!("efk_u_{}", i);
            prm.extend_from_slice(&(uname.len() as i32).to_le_bytes());
            prm.extend_from_slice(uname.as_bytes());
        }

        // offset, priority
        prm.extend_from_slice(&0i32.to_le_bytes());
        prm.extend_from_slice(&0i32.to_le_bytes());
        // value_type: Float4 (3)
        prm.extend_from_slice(&3i32.to_le_bytes());
        // default_values
        prm.extend_from_slice(&1.0f32.to_le_bytes());
        prm.extend_from_slice(&2.0f32.to_le_bytes());
        prm.extend_from_slice(&3.0f32.to_le_bytes());
        prm.extend_from_slice(&4.0f32.to_le_bytes());
    }

    // Gradients (version >= 1703)
    if version >= 1703 {
        prm.extend_from_slice(&(gradient_count as i32).to_le_bytes());
        for i in 0..gradient_count {
            let name = format!("Grad{}", i);
            prm.extend_from_slice(&(name.len() as i32).to_le_bytes());
            prm.extend_from_slice(name.as_bytes());
            let uname = format!("efk_g_{}", i);
            prm.extend_from_slice(&(uname.len() as i32).to_le_bytes());
            prm.extend_from_slice(uname.as_bytes());
            // offset, priority (C++ reads these before gradient data)
            prm.extend_from_slice(&0i32.to_le_bytes());
            prm.extend_from_slice(&0i32.to_le_bytes());
            // Gradient data (variable-length): 1 color key, 1 alpha key
            prm.extend_from_slice(&1i32.to_le_bytes()); // color_count
            // 1 color key: position=0.0, r=1.0, g=0.0, b=0.0, intensity=1.0
            prm.extend_from_slice(&0.0f32.to_le_bytes());
            prm.extend_from_slice(&1.0f32.to_le_bytes());
            prm.extend_from_slice(&0.0f32.to_le_bytes());
            prm.extend_from_slice(&0.0f32.to_le_bytes());
            prm.extend_from_slice(&1.0f32.to_le_bytes());
            prm.extend_from_slice(&1i32.to_le_bytes()); // alpha_count
            // 1 alpha key: position=0.0, alpha=1.0
            prm.extend_from_slice(&0.0f32.to_le_bytes());
            prm.extend_from_slice(&1.0f32.to_le_bytes());
        }

        // Fixed gradients: 0
        prm.extend_from_slice(&0i32.to_le_bytes());
    }

    prm
}

#[test]
fn test_load_material_default_config() {
    let data = build_material(1800, 2, 1, 1, true);
    let mat = load_material(&data).unwrap();

    assert_eq!(mat.version, 1800);
    assert_eq!(mat.guid, 1);
    assert!(mat.has_refraction);
    assert_eq!(mat.custom_data_1_count, 2);
    assert_eq!(mat.custom_data_2_count, 4);
    assert_eq!(mat.textures.len(), 2);
    assert_eq!(mat.uniforms.len(), 1);
    assert_eq!(mat.gradients.len(), 1);
    assert_eq!(mat.required_methods.len(), 2);
    assert!(mat.code.is_some());
}

#[test]
fn test_texture_and_uniform_extraction() {
    let data = build_material(1800, 3, 2, 0, true);
    let mat = load_material(&data).unwrap();

    assert_eq!(mat.textures.len(), 3);
    assert_eq!(mat.textures[0].name, "Texture0");
    assert_eq!(mat.textures[1].name, "Texture1");
    assert_eq!(mat.textures[2].name, "Texture2");
    assert_eq!(mat.textures[0].uniform_name.as_deref(), Some("efk_tex_0"));
    assert_eq!(mat.textures[2].index, 2);

    assert_eq!(mat.uniforms.len(), 2);
    assert_eq!(mat.uniforms[0].name, "Uniform0");
    assert_eq!(mat.uniforms[0].default_values, [1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_gene_code_extraction() {
    let data = build_material(1800, 0, 0, 0, true);
    let mat = load_material(&data).unwrap();
    let code = mat.code.unwrap();
    assert!(code.contains("$F4$"));
    assert!(!code.contains('\0'), "null terminator should be stripped");
}

#[test]
fn test_material_without_gene_chunk() {
    let data = build_material(1800, 0, 0, 0, false);
    let mat = load_material(&data).unwrap();
    assert!(mat.code.is_none());
}

#[test]
fn test_material_v3_no_gradients_no_required_methods() {
    let data = build_material(3, 1, 1, 0, true);
    let mat = load_material(&data).unwrap();

    assert_eq!(mat.version, 3);
    assert!(mat.required_methods.is_empty());
    assert!(mat.gradients.is_empty());
    assert!(mat.fixed_gradients.is_empty());
}

#[test]
fn test_material_gradient_extraction() {
    let data = build_material(1800, 0, 0, 2, true);
    let mat = load_material(&data).unwrap();

    assert_eq!(mat.gradients.len(), 2);
    assert_eq!(mat.gradients[0].name, "Grad0");
    assert_eq!(mat.gradients[0].uniform_name.as_deref(), Some("efk_g_0"));
    assert_eq!(mat.gradients[0].gradient.colors.len(), 1);
    assert_eq!(mat.gradients[0].gradient.alphas.len(), 1);
}

#[test]
fn test_invalid_magic_bytes() {
    let mut data = build_material(1800, 0, 0, 0, true);
    data[0] = b'X';
    let result = load_material(&data);
    assert!(result.is_err());
}

#[test]
fn test_unsupported_material_version() {
    let mut data = Vec::new();
    data.extend_from_slice(b"EFKM");
    data.extend_from_slice(&999i32.to_le_bytes());
    data.extend_from_slice(&0u64.to_le_bytes());
    let result = load_material(&data);
    assert!(result.is_err());
}

#[test]
fn test_unknown_chunks_skipped() {
    let mut data = Vec::new();
    // Header
    data.extend_from_slice(b"EFKM");
    data.extend_from_slice(&1800i32.to_le_bytes());
    data.extend_from_slice(&1u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());

    // Unknown chunk: "XUNK"
    data.extend_from_slice(b"XUNK");
    data.extend_from_slice(&8i32.to_le_bytes());
    data.extend_from_slice(&[0u8; 8]);

    // PRM_ chunk (minimal)
    let prm = build_prm_data(1800, 0, 0, 0);
    data.extend_from_slice(b"PRM_");
    data.extend_from_slice(&(prm.len() as i32).to_le_bytes());
    data.extend_from_slice(&prm);

    let mat = load_material(&data).unwrap();
    assert_eq!(mat.textures.len(), 0);
}

// ─── Real sample file tests ───

fn load_test_file(name: &str) -> Vec<u8> {
    let path = format!("tests/test_data/{name}");
    std::fs::read(&path).unwrap_or_else(|e| panic!("Failed to read {path}: {e}"))
}

#[test]
fn test_sample_fresnel_efkmat() {
    let data = load_test_file("Fresnel.efkmat");
    let mat = load_material(&data).unwrap();

    assert_eq!(mat.version, 3);
    assert!(mat.code.is_some(), "GENE code should be present");
    let code = mat.code.as_ref().unwrap();
    assert!(!code.is_empty(), "GENE code should be non-empty");
    assert!(!code.contains('\0'), "null terminator should be stripped");
}
