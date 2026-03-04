//! NURBS curve parser.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::ParseConfig;
use crate::types::curve::{DVector4, NurbsCurve};

/// Parse a NURBS curve file.
///
/// No magic bytes — the file starts directly with `converter_version`.
pub(crate) fn parse_curve(data: &[u8], config: &ParseConfig) -> Result<NurbsCurve, Error> {
    let mut reader = BinaryReader::new(data);

    let converter_version = reader.read_i32()?;

    // Control points
    let control_point_count = reader.read_i32()? as usize;
    if control_point_count > config.limits.max_vertices_per_frame {
        return Err(Error::ResourceLimitExceeded {
            field: "curve.control_points",
            count: control_point_count,
            max: config.limits.max_vertices_per_frame,
        });
    }

    let mut control_points = Vec::with_capacity(control_point_count);
    for _ in 0..control_point_count {
        let x = reader.read_f64()?;
        let y = reader.read_f64()?;
        let z = reader.read_f64()?;
        let w = reader.read_f64()?;
        control_points.push(DVector4 { x, y, z, w });
    }

    // Knots
    let knot_count = reader.read_i32()? as usize;
    if knot_count > config.limits.max_vertices_per_frame {
        return Err(Error::ResourceLimitExceeded {
            field: "curve.knots",
            count: knot_count,
            max: config.limits.max_vertices_per_frame,
        });
    }

    let mut knots = Vec::with_capacity(knot_count);
    for _ in 0..knot_count {
        knots.push(reader.read_f64()?);
    }

    let order = reader.read_i32()?;
    let step = reader.read_f64()?;
    let curve_type = reader.read_i32()?;
    let dimension = reader.read_i32()?;

    // Compute length: sum of Euclidean distances between consecutive control points (xyz only)
    let length = compute_length(&control_points);

    Ok(NurbsCurve {
        converter_version,
        control_points,
        knots,
        order,
        step,
        curve_type,
        dimension,
        length,
    })
}

/// Sum of Euclidean distances between consecutive control points (x, y, z only, ignoring w).
fn compute_length(points: &[DVector4]) -> f64 {
    if points.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0;
    for i in 0..points.len() - 1 {
        let dx = points[i + 1].x - points[i].x;
        let dy = points[i + 1].y - points[i].y;
        let dz = points[i + 1].z - points[i].z;
        total += (dx * dx + dy * dy + dz * dz).sqrt();
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_curve_binary(num_points: usize, num_knots: usize) -> Vec<u8> {
        let mut data = Vec::new();

        // converter_version
        data.extend_from_slice(&1i32.to_le_bytes());

        // control_point_count
        data.extend_from_slice(&(num_points as i32).to_le_bytes());

        // control points
        for i in 0..num_points {
            let x = i as f64;
            data.extend_from_slice(&x.to_le_bytes()); // x
            data.extend_from_slice(&0.0f64.to_le_bytes()); // y
            data.extend_from_slice(&0.0f64.to_le_bytes()); // z
            data.extend_from_slice(&1.0f64.to_le_bytes()); // w
        }

        // knot_count
        data.extend_from_slice(&(num_knots as i32).to_le_bytes());

        // knots
        for i in 0..num_knots {
            let k = i as f64;
            data.extend_from_slice(&k.to_le_bytes());
        }

        // order
        data.extend_from_slice(&4i32.to_le_bytes());
        // step
        data.extend_from_slice(&0.01f64.to_le_bytes());
        // type
        data.extend_from_slice(&0i32.to_le_bytes());
        // dimension
        data.extend_from_slice(&3i32.to_le_bytes());

        data
    }

    #[test]
    fn test_parse_curve() {
        let data = build_curve_binary(4, 8);
        let config = ParseConfig::default();
        let curve = parse_curve(&data, &config).unwrap();

        assert_eq!(curve.converter_version, 1);
        assert_eq!(curve.control_points.len(), 4);
        assert_eq!(curve.knots.len(), 8);
        assert_eq!(curve.order, 4);
        assert!((curve.step - 0.01).abs() < f64::EPSILON);
        assert_eq!(curve.dimension, 3);

        // Length: 3 segments of length 1.0 along x axis = 3.0
        assert!((curve.length - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_single_point_length() {
        let data = build_curve_binary(1, 2);
        let config = ParseConfig::default();
        let curve = parse_curve(&data, &config).unwrap();
        assert_eq!(curve.control_points.len(), 1);
        assert!((curve.length - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_empty_curve_length() {
        let data = build_curve_binary(0, 0);
        let config = ParseConfig::default();
        let curve = parse_curve(&data, &config).unwrap();
        assert_eq!(curve.control_points.len(), 0);
        assert!((curve.length - 0.0).abs() < f64::EPSILON);
    }
}
