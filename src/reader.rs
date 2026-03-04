//! Binary reader utility for parsing Effekseer files.
//!
//! All reads are little-endian. The reader wraps a `&[u8]` with position tracking.

use crate::error::Error;
use crate::types::{
    Color, ColorMode, EasingColor, EasingFloat, EasingVector2D, EasingVector3D, ParseConfig,
    RandomColor, RandomFloat, RandomInt, RandomVector2D, RandomVector3D, Rectf, RefMinMax,
    TriggerType, TriggerValues, UnknownEnumBehavior, Vector2D, Vector3D,
};

/// Trait implemented by all enums to check if they hold an unknown discriminant.
pub trait IsUnknown {
    /// Returns `true` if this value is an `Unknown` variant.
    fn is_unknown(&self) -> bool;
}

/// A cursor-based binary reader for little-endian data.
pub struct BinaryReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> BinaryReader<'a> {
    /// Create a new reader over the given byte slice.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Current read position in the buffer.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Number of bytes remaining after the current position.
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    /// Total length of the underlying buffer.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the underlying buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Skip `n` bytes, advancing the position.
    pub fn skip(&mut self, n: usize) -> Result<(), Error> {
        if self.pos + n > self.data.len() {
            return Err(Error::UnexpectedEof {
                position: self.pos,
                expected_bytes: n,
            });
        }
        self.pos += n;
        Ok(())
    }

    /// Read `n` raw bytes from the current position.
    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], Error> {
        if self.pos + n > self.data.len() {
            return Err(Error::UnexpectedEof {
                position: self.pos,
                expected_bytes: n,
            });
        }
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    /// Read a single byte.
    pub fn read_u8(&mut self) -> Result<u8, Error> {
        let bytes = self.read_bytes(1)?;
        Ok(bytes[0])
    }

    /// Read a little-endian `u16`.
    pub fn read_u16(&mut self) -> Result<u16, Error> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    /// Read a little-endian `i32`.
    pub fn read_i32(&mut self) -> Result<i32, Error> {
        let bytes = self.read_bytes(4)?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read a little-endian `u32`.
    pub fn read_u32(&mut self) -> Result<u32, Error> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read a little-endian `f32`.
    pub fn read_f32(&mut self) -> Result<f32, Error> {
        let bytes = self.read_bytes(4)?;
        Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read a little-endian `f64`.
    pub fn read_f64(&mut self) -> Result<f64, Error> {
        let bytes = self.read_bytes(8)?;
        Ok(f64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// Read an `i32` and interpret as `bool` (non-zero = `true`).
    pub fn read_i32_as_bool(&mut self) -> Result<bool, Error> {
        Ok(self.read_i32()? != 0)
    }

    /// Read a UTF-16LE encoded string.
    ///
    /// Format: `length: i32` (character count), then `length` u16 values.
    pub fn read_utf16_string(&mut self) -> Result<String, Error> {
        let length = self.read_i32()? as usize;
        let start_pos = self.pos;
        let mut chars = Vec::with_capacity(length);
        for _ in 0..length {
            chars.push(self.read_u16()?);
        }
        let s = String::from_utf16(&chars).map_err(|_| Error::Utf16DecodeError {
            position: start_pos,
        })?;
        Ok(s.trim_end_matches('\0').to_string())
    }

    /// Read an ASCII/UTF-8 encoded string.
    ///
    /// Format: `length: i32` (byte count), then `length` bytes.
    pub fn read_ascii_string(&mut self) -> Result<String, Error> {
        let length = self.read_i32()? as usize;
        let start_pos = self.pos;
        let bytes = self.read_bytes(length)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| Error::Utf8DecodeError {
            position: start_pos,
        })
    }

    /// Read an `i32` enum value, converting via `From<i32>` and checking the unknown policy.
    pub fn read_enum<T>(&mut self, config: &ParseConfig, field: &'static str) -> Result<T, Error>
    where
        T: From<i32> + IsUnknown,
    {
        let raw = self.read_i32()?;
        let value = T::from(raw);
        if value.is_unknown() {
            match config.unknown_enum_behavior {
                UnknownEnumBehavior::Error => {
                    return Err(Error::InvalidEnumValue { field, value: raw });
                }
                UnknownEnumBehavior::Warn => {
                    log::warn!("Unknown enum value {raw} for field {field}");
                }
            }
        }
        Ok(value)
    }

    /// Read a `u8` enum value, converting via `From<u8>` and checking the unknown policy.
    pub fn read_enum_u8<T>(&mut self, config: &ParseConfig, field: &'static str) -> Result<T, Error>
    where
        T: From<u8> + IsUnknown,
    {
        let raw = self.read_u8()?;
        let value = T::from(raw);
        if value.is_unknown() {
            match config.unknown_enum_behavior {
                UnknownEnumBehavior::Error => {
                    return Err(Error::InvalidEnumValue {
                        field,
                        value: raw as i32,
                    });
                }
                UnknownEnumBehavior::Warn => {
                    log::warn!("Unknown enum value {raw} for field {field}");
                }
            }
        }
        Ok(value)
    }

    // --- Primitive composite type readers ---

    /// Read a [`Vector2D`] (x, y as f32).
    pub fn read_vector2d(&mut self) -> Result<Vector2D, Error> {
        Ok(Vector2D {
            x: self.read_f32()?,
            y: self.read_f32()?,
        })
    }

    /// Read a [`Vector3D`] (x, y, z as f32).
    pub fn read_vector3d(&mut self) -> Result<Vector3D, Error> {
        Ok(Vector3D {
            x: self.read_f32()?,
            y: self.read_f32()?,
            z: self.read_f32()?,
        })
    }

    /// Read a [`Rectf`] (x, y, w, h as f32).
    pub fn read_rectf(&mut self) -> Result<Rectf, Error> {
        Ok(Rectf {
            x: self.read_f32()?,
            y: self.read_f32()?,
            w: self.read_f32()?,
            h: self.read_f32()?,
        })
    }

    /// Read a [`Color`] (r, g, b, a as u8).
    pub fn read_color(&mut self) -> Result<Color, Error> {
        Ok(Color {
            r: self.read_u8()?,
            g: self.read_u8()?,
            b: self.read_u8()?,
            a: self.read_u8()?,
        })
    }

    /// Read a [`RandomFloat`] (max, min — note max before min in binary).
    pub fn read_random_float(&mut self) -> Result<RandomFloat, Error> {
        Ok(RandomFloat {
            max: self.read_f32()?,
            min: self.read_f32()?,
        })
    }

    /// Read a [`RandomInt`] (max, min).
    pub fn read_random_int(&mut self) -> Result<RandomInt, Error> {
        Ok(RandomInt {
            max: self.read_i32()?,
            min: self.read_i32()?,
        })
    }

    /// Read a [`RandomVector2D`] (max, min).
    pub fn read_random_vector2d(&mut self) -> Result<RandomVector2D, Error> {
        Ok(RandomVector2D {
            max: self.read_vector2d()?,
            min: self.read_vector2d()?,
        })
    }

    /// Read a [`RandomVector3D`] (max, min).
    pub fn read_random_vector3d(&mut self) -> Result<RandomVector3D, Error> {
        Ok(RandomVector3D {
            max: self.read_vector3d()?,
            min: self.read_vector3d()?,
        })
    }

    /// Read a [`RandomColor`] (10 bytes on disk: mode u8, pad u8, max Color, min Color).
    pub fn read_random_color(&mut self, config: &ParseConfig) -> Result<RandomColor, Error> {
        let mode: ColorMode = self.read_enum_u8(config, "RandomColor.mode")?;
        let _pad = self.read_u8()?;
        let max = self.read_color()?;
        let min = self.read_color()?;
        Ok(RandomColor { mode, max, min })
    }

    /// Read a [`RefMinMax`] (max, min as i32).
    pub fn read_ref_min_max(&mut self) -> Result<RefMinMax, Error> {
        Ok(RefMinMax {
            max: self.read_i32()?,
            min: self.read_i32()?,
        })
    }

    /// Read a [`TriggerValues`] (type_ as u8, index as u8).
    pub fn read_trigger_values(&mut self, config: &ParseConfig) -> Result<TriggerValues, Error> {
        let type_: TriggerType = self.read_enum_u8(config, "TriggerValues.type")?;
        let index = self.read_u8()?;
        Ok(TriggerValues { type_, index })
    }

    /// Read an [`EasingFloat`] (start, end as RandomFloat, then a, b, c as f32).
    pub fn read_easing_float(&mut self) -> Result<EasingFloat, Error> {
        Ok(EasingFloat {
            start: self.read_random_float()?,
            end: self.read_random_float()?,
            a: self.read_f32()?,
            b: self.read_f32()?,
            c: self.read_f32()?,
        })
    }

    /// Read an [`EasingVector2D`].
    pub fn read_easing_vector2d(&mut self) -> Result<EasingVector2D, Error> {
        Ok(EasingVector2D {
            start: self.read_random_vector2d()?,
            end: self.read_random_vector2d()?,
            a: self.read_f32()?,
            b: self.read_f32()?,
            c: self.read_f32()?,
        })
    }

    /// Read an [`EasingVector3D`].
    pub fn read_easing_vector3d(&mut self) -> Result<EasingVector3D, Error> {
        Ok(EasingVector3D {
            start: self.read_random_vector3d()?,
            end: self.read_random_vector3d()?,
            a: self.read_f32()?,
            b: self.read_f32()?,
            c: self.read_f32()?,
        })
    }

    /// Read an [`EasingColor`] (start and end are RandomColor at 10 bytes each, then a, b, c).
    pub fn read_easing_color(&mut self, config: &ParseConfig) -> Result<EasingColor, Error> {
        Ok(EasingColor {
            start: self.read_random_color(config)?,
            end: self.read_random_color(config)?,
            a: self.read_f32()?,
            b: self.read_f32()?,
            c: self.read_f32()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::enums::AlphaBlendType;

    #[test]
    fn test_read_i32() {
        let data = 42i32.to_le_bytes();
        let mut r = BinaryReader::new(&data);
        assert_eq!(r.read_i32().unwrap(), 42);
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn test_read_f32() {
        let data = 1.25f32.to_le_bytes();
        let mut r = BinaryReader::new(&data);
        let val = r.read_f32().unwrap();
        assert!((val - 1.25).abs() < f32::EPSILON);
    }

    #[test]
    fn test_read_utf16_string() {
        let text = "Hello";
        let chars: Vec<u16> = text.encode_utf16().collect();
        let mut data = (chars.len() as i32).to_le_bytes().to_vec();
        for c in &chars {
            data.extend_from_slice(&c.to_le_bytes());
        }
        let mut r = BinaryReader::new(&data);
        assert_eq!(r.read_utf16_string().unwrap(), "Hello");
    }

    #[test]
    fn test_read_utf16_string_trims_null() {
        // "Hi\0" — trailing null should be stripped
        let chars: Vec<u16> = vec![0x48, 0x69, 0x00]; // 'H', 'i', '\0'
        let mut data = (chars.len() as i32).to_le_bytes().to_vec();
        for c in &chars {
            data.extend_from_slice(&c.to_le_bytes());
        }
        let mut r = BinaryReader::new(&data);
        assert_eq!(r.read_utf16_string().unwrap(), "Hi");
    }

    #[test]
    fn test_read_ascii_string() {
        let text = b"BaseColor";
        let mut data = (text.len() as i32).to_le_bytes().to_vec();
        data.extend_from_slice(text);
        let mut r = BinaryReader::new(&data);
        assert_eq!(r.read_ascii_string().unwrap(), "BaseColor");
    }

    #[test]
    fn test_read_enum_known() {
        let data = 2i32.to_le_bytes(); // Add
        let config = ParseConfig::default();
        let mut r = BinaryReader::new(&data);
        let val: AlphaBlendType = r.read_enum(&config, "test").unwrap();
        assert_eq!(val, AlphaBlendType::Add);
    }

    #[test]
    fn test_read_enum_unknown_error() {
        let data = 99i32.to_le_bytes();
        let config = ParseConfig::default(); // Error mode
        let mut r = BinaryReader::new(&data);
        let result: Result<AlphaBlendType, _> = r.read_enum(&config, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_enum_unknown_warn() {
        let data = 99i32.to_le_bytes();
        let config = ParseConfig {
            unknown_enum_behavior: UnknownEnumBehavior::Warn,
            ..Default::default()
        };
        let mut r = BinaryReader::new(&data);
        let val: AlphaBlendType = r.read_enum(&config, "test").unwrap();
        assert_eq!(val, AlphaBlendType::Unknown(99));
    }

    #[test]
    fn test_unexpected_eof() {
        let data = [0u8; 2];
        let mut r = BinaryReader::new(&data);
        assert!(r.read_i32().is_err());
    }

    #[test]
    fn test_read_vector3d() {
        let mut data = Vec::new();
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&3.0f32.to_le_bytes());
        let mut r = BinaryReader::new(&data);
        let v = r.read_vector3d().unwrap();
        assert_eq!(
            v,
            Vector3D {
                x: 1.0,
                y: 2.0,
                z: 3.0
            }
        );
    }

    #[test]
    fn test_read_color() {
        let data = [255, 128, 64, 32];
        let mut r = BinaryReader::new(&data);
        let c = r.read_color().unwrap();
        assert_eq!(
            c,
            Color {
                r: 255,
                g: 128,
                b: 64,
                a: 32
            }
        );
    }

    #[test]
    fn test_read_random_color() {
        // mode=0 (RGBA), pad=0, max=Color(255,0,0,255), min=Color(0,0,0,255)
        let data = [
            0, 0, // mode + pad
            255, 0, 0, 255, // max color
            0, 0, 0, 255, // min color
        ];
        let config = ParseConfig::default();
        let mut r = BinaryReader::new(&data);
        let rc = r.read_random_color(&config).unwrap();
        assert_eq!(rc.mode, ColorMode::Rgba);
        assert_eq!(
            rc.max,
            Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255
            }
        );
        assert_eq!(
            rc.min,
            Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255
            }
        );
    }

    #[test]
    fn test_skip() {
        let data = [0u8; 10];
        let mut r = BinaryReader::new(&data);
        r.skip(5).unwrap();
        assert_eq!(r.position(), 5);
        assert_eq!(r.remaining(), 5);
        assert!(r.skip(6).is_err());
    }
}
