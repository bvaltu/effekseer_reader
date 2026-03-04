//! F-Curve binary loading.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::fcurve::{
    FCurve, FCurveScalar, FCurveVector2D, FCurveVector3D, FCurveVectorColor,
};
use crate::types::{FCurveEdge, ParseConfig};

/// Read a single FCurve from the binary stream.
pub(crate) fn parse_fcurve(
    reader: &mut BinaryReader,
    config: &ParseConfig,
) -> Result<FCurve, Error> {
    let start_edge: FCurveEdge = reader.read_enum(config, "FCurve.start_edge")?;
    let end_edge: FCurveEdge = reader.read_enum(config, "FCurve.end_edge")?;
    let offset_max = reader.read_f32()?;
    let offset_min = reader.read_f32()?;
    let offset = reader.read_i32()?;
    let len = reader.read_i32()?;
    let freq = reader.read_i32()?;
    let count = reader.read_i32()? as usize;

    if count > config.limits.max_fcurve_samples {
        return Err(Error::ResourceLimitExceeded {
            field: "FCurve.keys",
            count,
            max: config.limits.max_fcurve_samples,
        });
    }

    let mut keys = Vec::with_capacity(count);
    for _ in 0..count {
        keys.push(reader.read_f32()?);
    }

    Ok(FCurve {
        start_edge,
        end_edge,
        offset_max,
        offset_min,
        offset,
        len,
        freq,
        keys,
    })
}

/// Read an FCurveScalar (1 channel).
pub(crate) fn parse_fcurve_scalar(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<FCurveScalar, Error> {
    // Timeline field: genuinely version >= 1600 for scalar
    let timeline = if version >= 1600 {
        reader.read_i32()?
    } else {
        0 // FCurveTimelineType::Time
    };
    let s = parse_fcurve(reader, config)?;
    Ok(FCurveScalar { timeline, s })
}

/// Read an FCurveVector2D (2 channels).
pub(crate) fn parse_fcurve_vector2d(
    reader: &mut BinaryReader,
    _version: i32,
    config: &ParseConfig,
) -> Result<FCurveVector2D, Error> {
    // Timeline: gate is version >= 15, always true for supported versions
    let timeline = reader.read_i32()?;
    let x = parse_fcurve(reader, config)?;
    let y = parse_fcurve(reader, config)?;
    Ok(FCurveVector2D { timeline, x, y })
}

/// Read an FCurveVector3D (3 channels).
pub(crate) fn parse_fcurve_vector3d(
    reader: &mut BinaryReader,
    _version: i32,
    config: &ParseConfig,
) -> Result<FCurveVector3D, Error> {
    // Timeline: gate is version >= 15, always true for supported versions
    let timeline = reader.read_i32()?;
    let x = parse_fcurve(reader, config)?;
    let y = parse_fcurve(reader, config)?;
    let z = parse_fcurve(reader, config)?;
    Ok(FCurveVector3D { timeline, x, y, z })
}

/// Read an FCurveVectorColor (4 channels).
pub(crate) fn parse_fcurve_vector_color(
    reader: &mut BinaryReader,
    _version: i32,
    config: &ParseConfig,
) -> Result<FCurveVectorColor, Error> {
    // Timeline: gate is version >= 15, always true for supported versions
    let timeline = reader.read_i32()?;
    let r = parse_fcurve(reader, config)?;
    let g = parse_fcurve(reader, config)?;
    let b = parse_fcurve(reader, config)?;
    let a = parse_fcurve(reader, config)?;
    Ok(FCurveVectorColor {
        timeline,
        r,
        g,
        b,
        a,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ParseConfig;

    fn build_fcurve_bytes(count: usize) -> Vec<u8> {
        let mut data = Vec::new();
        // start_edge: Constant (0)
        data.extend_from_slice(&0i32.to_le_bytes());
        // end_edge: Loop (1)
        data.extend_from_slice(&1i32.to_le_bytes());
        // offset_max: 1.0
        data.extend_from_slice(&1.0f32.to_le_bytes());
        // offset_min: -1.0
        data.extend_from_slice(&(-1.0f32).to_le_bytes());
        // offset: 0
        data.extend_from_slice(&0i32.to_le_bytes());
        // len: 100
        data.extend_from_slice(&100i32.to_le_bytes());
        // freq: 1
        data.extend_from_slice(&1i32.to_le_bytes());
        // count
        data.extend_from_slice(&(count as i32).to_le_bytes());
        // keys
        for i in 0..count {
            data.extend_from_slice(&(i as f32).to_le_bytes());
        }
        data
    }

    #[test]
    fn test_parse_fcurve() {
        let data = build_fcurve_bytes(5);
        let config = ParseConfig::default();
        let mut reader = BinaryReader::new(&data);
        let curve = parse_fcurve(&mut reader, &config).unwrap();
        assert_eq!(curve.start_edge, FCurveEdge::Constant);
        assert_eq!(curve.end_edge, FCurveEdge::Loop);
        assert_eq!(curve.keys.len(), 5);
        assert!((curve.keys[2] - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_fcurve_vector3d() {
        let mut data = Vec::new();
        // timeline
        data.extend_from_slice(&0i32.to_le_bytes());
        // 3 curves
        for _ in 0..3 {
            data.extend_from_slice(&build_fcurve_bytes(3));
        }
        let config = ParseConfig::default();
        let mut reader = BinaryReader::new(&data);
        let v = parse_fcurve_vector3d(&mut reader, 1600, &config).unwrap();
        assert_eq!(v.x.keys.len(), 3);
        assert_eq!(v.y.keys.len(), 3);
        assert_eq!(v.z.keys.len(), 3);
    }
}
