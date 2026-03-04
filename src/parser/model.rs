//! Model file (.efkmodel) parser.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::ParseConfig;
use crate::types::model::{ModelFace, ModelFile, ModelFrame, ModelVertex};
use crate::types::primitives::Color;

/// Latest supported model file version.
const MODEL_LATEST_VERSION: i32 = 6;

/// Default vertex color for model version 0 (which has no vcolor field).
const DEFAULT_VCOLOR: Color = Color { r: 255, g: 255, b: 255, a: 255 };

/// Parse an `.efkmodel` model file.
pub(crate) fn parse_model(data: &[u8], config: &ParseConfig) -> Result<ModelFile, Error> {
    let mut reader = BinaryReader::new(data);

    let version = reader.read_i32()?;
    if version > MODEL_LATEST_VERSION {
        return Err(Error::UnsupportedVersion { version });
    }

    // Scale field: present for version == 2 OR version >= 5
    if version == 2 || version >= 5 {
        let _scale = reader.read_i32()?;
    }

    // model_count: backward compat, ignored
    let _model_count = reader.read_i32()?;

    // frame_count: version >= 5, else default to 1
    let frame_count = if version >= 5 {
        reader.read_i32()? as usize
    } else {
        1
    };

    if frame_count > config.limits.max_frame_count {
        return Err(Error::ResourceLimitExceeded {
            field: "model.frame_count",
            count: frame_count,
            max: config.limits.max_frame_count,
        });
    }

    let mut frames = Vec::with_capacity(frame_count);
    for _ in 0..frame_count {
        let frame = parse_model_frame(&mut reader, version, config)?;
        frames.push(frame);
    }

    Ok(ModelFile { version, frames })
}

fn parse_model_frame(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<ModelFrame, Error> {
    let vertex_count = reader.read_i32()? as usize;
    if vertex_count > config.limits.max_vertices_per_frame {
        return Err(Error::ResourceLimitExceeded {
            field: "model.vertex_count",
            count: vertex_count,
            max: config.limits.max_vertices_per_frame,
        });
    }

    let mut vertices = Vec::with_capacity(vertex_count);

    if version >= 6 {
        // Version 6: full vertex format (68 bytes each)
        for _ in 0..vertex_count {
            let position = reader.read_vector3d()?;
            let normal = reader.read_vector3d()?;
            let binormal = reader.read_vector3d()?;
            let tangent = reader.read_vector3d()?;
            let uv1 = reader.read_vector2d()?;
            let uv2 = reader.read_vector2d()?;
            let vcolor = reader.read_color()?;

            vertices.push(ModelVertex {
                position,
                normal,
                binormal,
                tangent,
                uv1,
                uv2,
                vcolor,
            });
        }
    } else {
        // Version < 6: field-by-field reading with defaults
        for _ in 0..vertex_count {
            let position = reader.read_vector3d()?;
            let normal = reader.read_vector3d()?;
            let binormal = reader.read_vector3d()?;
            let tangent = reader.read_vector3d()?;
            let uv1 = reader.read_vector2d()?;
            let uv2 = uv1; // uv2 = uv1 for version < 6

            let vcolor = if version >= 1 {
                reader.read_color()?
            } else {
                DEFAULT_VCOLOR
            };

            vertices.push(ModelVertex {
                position,
                normal,
                binormal,
                tangent,
                uv1,
                uv2,
                vcolor,
            });
        }
    }

    let face_count = reader.read_i32()? as usize;
    if face_count > config.limits.max_faces_per_frame {
        return Err(Error::ResourceLimitExceeded {
            field: "model.face_count",
            count: face_count,
            max: config.limits.max_faces_per_frame,
        });
    }

    let mut faces = Vec::with_capacity(face_count);
    for _ in 0..face_count {
        let i0 = reader.read_i32()?;
        let i1 = reader.read_i32()?;
        let i2 = reader.read_i32()?;
        faces.push(ModelFace {
            indexes: [i0, i1, i2],
        });
    }

    Ok(ModelFrame { vertices, faces })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::primitives::{Vector2D, Vector3D};

    fn build_model_binary(
        version: i32,
        frame_count: usize,
        vertex_count: usize,
        face_count: usize,
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
            // vertex_count
            data.extend_from_slice(&(vertex_count as i32).to_le_bytes());

            for i in 0..vertex_count {
                let val = i as f32;
                // position
                data.extend_from_slice(&val.to_le_bytes());
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
                    // uv2 (only written for version >= 6)
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
                // version 0: no vcolor written
            }

            // face_count
            data.extend_from_slice(&(face_count as i32).to_le_bytes());
            for _ in 0..face_count {
                data.extend_from_slice(&0i32.to_le_bytes());
                data.extend_from_slice(&1i32.to_le_bytes());
                data.extend_from_slice(&2i32.to_le_bytes());
            }
        }

        data
    }

    #[test]
    fn test_parse_model_v6() {
        let data = build_model_binary(6, 1, 3, 1);
        let config = ParseConfig::default();
        let model = parse_model(&data, &config).unwrap();

        assert_eq!(model.version, 6);
        assert_eq!(model.frames.len(), 1);
        assert_eq!(model.frames[0].vertices.len(), 3);
        assert_eq!(model.frames[0].faces.len(), 1);

        // Version 6 has separate uv2
        let v = &model.frames[0].vertices[0];
        assert_eq!(v.uv2, Vector2D { x: 0.25, y: 0.75 });
        assert_eq!(
            v.vcolor,
            Color {
                r: 255,
                g: 128,
                b: 64,
                a: 255
            }
        );
    }

    #[test]
    fn test_parse_model_v5() {
        let data = build_model_binary(5, 2, 2, 1);
        let config = ParseConfig::default();
        let model = parse_model(&data, &config).unwrap();

        assert_eq!(model.version, 5);
        assert_eq!(model.frames.len(), 2);

        // Version < 6: uv2 = uv1
        let v = &model.frames[0].vertices[0];
        assert_eq!(v.uv2, v.uv1);
    }

    #[test]
    fn test_parse_model_v0() {
        let data = build_model_binary(0, 1, 2, 1);
        let config = ParseConfig::default();
        let model = parse_model(&data, &config).unwrap();

        assert_eq!(model.version, 0);
        // Version 0: default white vcolor, uv2 = uv1
        let v = &model.frames[0].vertices[0];
        assert_eq!(
            v.vcolor,
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255
            }
        );
        assert_eq!(v.uv2, v.uv1);
    }

    #[test]
    fn test_reject_version_too_high() {
        let mut data = Vec::new();
        data.extend_from_slice(&7i32.to_le_bytes());
        let result = parse_model(&data, &ParseConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_resource_limit_vertices() {
        let mut config = ParseConfig::default();
        config.limits.max_vertices_per_frame = 1;
        let data = build_model_binary(6, 1, 3, 1);
        let result = parse_model(&data, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_vertex_positions() {
        let data = build_model_binary(6, 1, 3, 1);
        let config = ParseConfig::default();
        let model = parse_model(&data, &config).unwrap();

        assert_eq!(
            model.frames[0].vertices[0].position,
            Vector3D {
                x: 0.0,
                y: 0.0,
                z: 0.0
            }
        );
        assert_eq!(
            model.frames[0].vertices[1].position,
            Vector3D {
                x: 1.0,
                y: 0.0,
                z: 0.0
            }
        );
        assert_eq!(
            model.frames[0].vertices[2].position,
            Vector3D {
                x: 2.0,
                y: 0.0,
                z: 0.0
            }
        );
    }
}
