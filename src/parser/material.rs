//! Material file (.efkmat) parser.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::ParseConfig;
use crate::types::material::{MaterialFile, MaterialGradient, MaterialTexture, MaterialUniform};
use crate::version;

use super::gradient::parse_gradient;

fn is_valid_material_version(v: i32) -> bool {
    matches!(
        v,
        version::MATERIAL_VERSION_15
            | version::MATERIAL_VERSION_16
            | version::MATERIAL_VERSION_17_ALPHA2
            | version::MATERIAL_VERSION_17_ALPHA4
            | version::MATERIAL_VERSION_17
            | version::MATERIAL_VERSION_18
    )
}

/// Parse an `.efkmat` material file.
pub(crate) fn parse_material(data: &[u8], config: &ParseConfig) -> Result<MaterialFile, Error> {
    let mut reader = BinaryReader::new(data);

    // Header: magic "EFKM" + version + guid
    let magic = reader.read_bytes(4)?;
    if magic != b"EFKM" {
        return Err(Error::InvalidMagic {
            expected: b"EFKM",
            got: magic.to_vec(),
        });
    }

    let version = reader.read_i32()?;
    if !is_valid_material_version(version) {
        return Err(Error::UnsupportedVersion { version });
    }

    let guid = {
        let lo = reader.read_u32()? as u64;
        let hi = reader.read_u32()? as u64;
        lo | (hi << 32)
    };

    let mut material = MaterialFile {
        version,
        guid,
        shading_model: crate::types::ShadingModelType::Unlit,
        has_refraction: false,
        custom_data_1_count: 0,
        custom_data_2_count: 0,
        required_methods: Vec::new(),
        textures: Vec::new(),
        uniforms: Vec::new(),
        gradients: Vec::new(),
        fixed_gradients: Vec::new(),
        code: None,
    };

    // Chunk loop — read until EOF
    while reader.remaining() >= 8 {
        let chunk_id = reader.read_bytes(4)?;
        let chunk_size = reader.read_i32()?;

        if chunk_size < 0 {
            return Err(Error::InvalidChunk {
                message: format!(
                    "negative chunk size {} for chunk {:?}",
                    chunk_size,
                    std::str::from_utf8(chunk_id).unwrap_or("????")
                ),
            });
        }

        let chunk_size = chunk_size as usize;

        if chunk_id == b"PRM_" && material.textures.is_empty() {
            parse_prm_chunk(&mut reader, version, config, &mut material)?;
        } else if chunk_id == b"GENE" && material.code.is_none() {
            parse_gene_chunk(&mut reader, &mut material)?;
        } else {
            // Skip unknown or duplicate chunks
            reader.skip(chunk_size)?;
        }
    }

    Ok(material)
}

/// Parse the PRM_ chunk (parameter declarations).
fn parse_prm_chunk(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
    material: &mut MaterialFile,
) -> Result<(), Error> {
    // Shading model
    material.shading_model = reader.read_enum(config, "Material.shading_model")?;

    // has_normal (ignored)
    let _has_normal = reader.read_i32()?;

    // has_refraction
    material.has_refraction = reader.read_i32_as_bool()?;

    // custom data counts
    material.custom_data_1_count = reader.read_i32()?;
    material.custom_data_2_count = reader.read_i32()?;

    // Required methods (version >= 1703)
    if version >= version::MATERIAL_VERSION_17_ALPHA4 {
        let count = reader.read_i32()? as usize;
        material.required_methods = Vec::with_capacity(count);
        for _ in 0..count {
            let method = reader.read_enum(config, "Material.required_method")?;
            material.required_methods.push(method);
        }
    }

    // Textures
    let texture_count = reader.read_i32()? as usize;
    material.textures = Vec::with_capacity(texture_count);
    for _ in 0..texture_count {
        let name = reader.read_ascii_string()?;

        let uniform_name = if version >= version::MATERIAL_VERSION_15 {
            Some(reader.read_ascii_string()?)
        } else {
            None
        };

        // default_path — read and discard
        let _default_path = reader.read_ascii_string()?;

        let index = reader.read_i32()?;
        let priority = reader.read_i32()?;
        let param = reader.read_i32()?;
        let color_type = reader.read_enum(config, "MaterialTexture.color_type")?;
        let sampler = reader.read_enum(config, "MaterialTexture.sampler")?;

        material.textures.push(MaterialTexture {
            name,
            uniform_name,
            index,
            priority,
            param,
            color_type,
            sampler,
        });
    }

    // Uniforms
    let uniform_count = reader.read_i32()? as usize;
    material.uniforms = Vec::with_capacity(uniform_count);
    for _ in 0..uniform_count {
        let name = reader.read_ascii_string()?;

        let uniform_name = if version >= version::MATERIAL_VERSION_15 {
            Some(reader.read_ascii_string()?)
        } else {
            None
        };

        let offset = reader.read_i32()?;
        let priority = reader.read_i32()?;
        let value_type = reader.read_enum(config, "MaterialUniform.value_type")?;
        let default_values = [
            reader.read_f32()?,
            reader.read_f32()?,
            reader.read_f32()?,
            reader.read_f32()?,
        ];

        material.uniforms.push(MaterialUniform {
            name,
            uniform_name,
            offset,
            priority,
            value_type,
            default_values,
        });
    }

    // Gradients (version >= 1703)
    if version >= version::MATERIAL_VERSION_17_ALPHA4 {
        let gradient_count = reader.read_i32()? as usize;
        material.gradients = Vec::with_capacity(gradient_count);
        for _ in 0..gradient_count {
            let name = reader.read_ascii_string()?;
            let uniform_name = Some(reader.read_ascii_string()?);
            // C++ reads offset (i32) and priority (i32) here but discards them
            let _offset = reader.read_i32()?;
            let _priority = reader.read_i32()?;
            let gradient = parse_gradient(reader)?;
            material.gradients.push(MaterialGradient {
                name,
                uniform_name,
                gradient,
            });
        }

        // Fixed gradients
        let fixed_gradient_count = reader.read_i32()? as usize;
        material.fixed_gradients = Vec::with_capacity(fixed_gradient_count);
        for _ in 0..fixed_gradient_count {
            let name = reader.read_ascii_string()?;
            let uniform_name = Some(reader.read_ascii_string()?);
            // C++ reads offset (i32) and priority (i32) here but discards them
            let _offset = reader.read_i32()?;
            let _priority = reader.read_i32()?;
            let gradient = parse_gradient(reader)?;
            material.fixed_gradients.push(MaterialGradient {
                name,
                uniform_name,
                gradient,
            });
        }
    }

    Ok(())
}

/// Parse the GENE chunk (shader code).
fn parse_gene_chunk(reader: &mut BinaryReader, material: &mut MaterialFile) -> Result<(), Error> {
    let code_length = reader.read_i32()? as usize;
    let code_bytes = reader.read_bytes(code_length)?;

    // Null-terminated string — strip trailing null if present
    let code = if code_bytes.last() == Some(&0) {
        String::from_utf8_lossy(&code_bytes[..code_bytes.len() - 1]).into_owned()
    } else {
        String::from_utf8_lossy(code_bytes).into_owned()
    };

    material.code = Some(code);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ParseConfig;

    /// Build a minimal .efkmat binary for testing.
    fn build_minimal_material(version: i32) -> Vec<u8> {
        let mut data = Vec::new();

        // Header
        data.extend_from_slice(b"EFKM");
        data.extend_from_slice(&version.to_le_bytes());
        // guid (8 bytes as two u32)
        data.extend_from_slice(&42u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());

        // PRM_ chunk
        let prm_data = build_prm_data(version);
        data.extend_from_slice(b"PRM_");
        data.extend_from_slice(&(prm_data.len() as i32).to_le_bytes());
        data.extend_from_slice(&prm_data);

        // GENE chunk
        let gene_code = b"float4 main() { return $F4$(1,1,1,1); }\0";
        data.extend_from_slice(b"GENE");
        let gene_payload_size = 4 + gene_code.len();
        data.extend_from_slice(&(gene_payload_size as i32).to_le_bytes());
        data.extend_from_slice(&(gene_code.len() as i32).to_le_bytes());
        data.extend_from_slice(gene_code);

        data
    }

    fn build_prm_data(version: i32) -> Vec<u8> {
        let mut prm = Vec::new();

        // shading_model: Unlit (1)
        prm.extend_from_slice(&1i32.to_le_bytes());
        // has_normal: 0
        prm.extend_from_slice(&0i32.to_le_bytes());
        // has_refraction: 0
        prm.extend_from_slice(&0i32.to_le_bytes());
        // custom_data_1_count: 0
        prm.extend_from_slice(&0i32.to_le_bytes());
        // custom_data_2_count: 0
        prm.extend_from_slice(&0i32.to_le_bytes());

        // required methods (version >= 1703)
        if version >= 1703 {
            prm.extend_from_slice(&0i32.to_le_bytes()); // count: 0
        }

        // textures: 1
        prm.extend_from_slice(&1i32.to_le_bytes());
        // texture 0:
        // name: "BaseColor"
        let name = b"BaseColor";
        prm.extend_from_slice(&(name.len() as i32).to_le_bytes());
        prm.extend_from_slice(name);
        // uniform_name (version >= 3)
        if version >= 3 {
            let uname = b"efk_texture_0";
            prm.extend_from_slice(&(uname.len() as i32).to_le_bytes());
            prm.extend_from_slice(uname);
        }
        // default_path
        prm.extend_from_slice(&0i32.to_le_bytes()); // empty path
        // index, priority, param
        prm.extend_from_slice(&0i32.to_le_bytes());
        prm.extend_from_slice(&0i32.to_le_bytes());
        prm.extend_from_slice(&0i32.to_le_bytes());
        // color_type: Color (0)
        prm.extend_from_slice(&0i32.to_le_bytes());
        // sampler: Repeat (0)
        prm.extend_from_slice(&0i32.to_le_bytes());

        // uniforms: 0
        prm.extend_from_slice(&0i32.to_le_bytes());

        // gradients (version >= 1703)
        if version >= 1703 {
            prm.extend_from_slice(&0i32.to_le_bytes()); // gradient count: 0
            prm.extend_from_slice(&0i32.to_le_bytes()); // fixed gradient count: 0
        }

        prm
    }

    #[test]
    fn test_parse_material_v1800() {
        let data = build_minimal_material(1800);
        let config = ParseConfig::default();
        let mat = parse_material(&data, &config).unwrap();

        assert_eq!(mat.version, 1800);
        assert_eq!(mat.guid, 42);
        assert_eq!(mat.shading_model, crate::types::ShadingModelType::Unlit);
        assert!(!mat.has_refraction);
        assert_eq!(mat.textures.len(), 1);
        assert_eq!(mat.textures[0].name, "BaseColor");
        assert_eq!(
            mat.textures[0].uniform_name.as_deref(),
            Some("efk_texture_0")
        );
        assert!(mat.code.is_some());
        assert!(mat.code.as_ref().unwrap().contains("$F4$"));
    }

    #[test]
    fn test_parse_material_v3() {
        let data = build_minimal_material(3);
        let config = ParseConfig::default();
        let mat = parse_material(&data, &config).unwrap();

        assert_eq!(mat.version, 3);
        assert_eq!(mat.textures.len(), 1);
        assert!(mat.required_methods.is_empty());
        assert!(mat.gradients.is_empty());
    }

    #[test]
    fn test_invalid_magic() {
        let mut data = build_minimal_material(1800);
        data[0] = b'X';
        let result = parse_material(&data, &ParseConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_version() {
        let mut data = Vec::new();
        data.extend_from_slice(b"EFKM");
        data.extend_from_slice(&999i32.to_le_bytes());
        data.extend_from_slice(&0u64.to_le_bytes());
        let result = parse_material(&data, &ParseConfig::default());
        assert!(result.is_err());
    }
}
