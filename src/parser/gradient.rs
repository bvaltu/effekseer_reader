//! Gradient binary loading (variable-length).

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::gradient::{Gradient, GradientAlphaKey, GradientColorKey};

/// Read a Gradient from the binary stream (variable-length).
///
/// Binary format (matches C++ `LoadGradient`):
/// - `ColorCount` (i32)
/// - `ColorKey[ColorCount]` (20 bytes each: Position, Color[3], Intensity)
/// - `AlphaCount` (i32)
/// - `AlphaKey[AlphaCount]` (8 bytes each: Position, Alpha)
pub(crate) fn parse_gradient(reader: &mut BinaryReader) -> Result<Gradient, Error> {
    let color_count = reader.read_i32()?.clamp(0, 8) as usize;
    let mut colors = Vec::with_capacity(color_count);
    for _ in 0..color_count {
        let position = reader.read_f32()?;
        let r = reader.read_f32()?;
        let g = reader.read_f32()?;
        let b = reader.read_f32()?;
        let intensity = reader.read_f32()?;
        colors.push(GradientColorKey {
            position,
            r,
            g,
            b,
            intensity,
        });
    }

    let alpha_count = reader.read_i32()?.clamp(0, 8) as usize;
    let mut alphas = Vec::with_capacity(alpha_count);
    for _ in 0..alpha_count {
        let position = reader.read_f32()?;
        let alpha = reader.read_f32()?;
        alphas.push(GradientAlphaKey { position, alpha });
    }

    Ok(Gradient { colors, alphas })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gradient() {
        let mut data = Vec::new();
        // color_count: 2
        data.extend_from_slice(&2i32.to_le_bytes());
        // 2 color keys (20 bytes each)
        for i in 0..2 {
            let pos = i as f32;
            data.extend_from_slice(&pos.to_le_bytes()); // position
            data.extend_from_slice(&1.0f32.to_le_bytes()); // r
            data.extend_from_slice(&0.5f32.to_le_bytes()); // g
            data.extend_from_slice(&0.0f32.to_le_bytes()); // b
            data.extend_from_slice(&1.0f32.to_le_bytes()); // intensity
        }

        // alpha_count: 1
        data.extend_from_slice(&1i32.to_le_bytes());
        // 1 alpha key (8 bytes)
        data.extend_from_slice(&0.0f32.to_le_bytes()); // position
        data.extend_from_slice(&1.0f32.to_le_bytes()); // alpha

        // Expected size: 4 + 2*20 + 4 + 1*8 = 56 bytes
        assert_eq!(data.len(), 56);

        let mut reader = BinaryReader::new(&data);
        let g = parse_gradient(&mut reader).unwrap();
        assert_eq!(g.colors.len(), 2);
        assert_eq!(g.alphas.len(), 1);
        assert!((g.colors[0].r - 1.0).abs() < f32::EPSILON);
    }
}
