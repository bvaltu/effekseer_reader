//! CommonValues parser — 4 version-dependent binary layouts.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::common_values::{GenerationParameter, ParameterCommonValues, RemovalParameter};
use crate::types::primitives::{RandomInt, RefMinMax, TriggerValues};
use crate::types::{
    BindType, GenerationTiming, ParseConfig, RemovalTiming, TranslationParentBindType, TriggerType,
};

/// Default trigger values (None, index 0).
fn default_trigger() -> TriggerValues {
    TriggerValues {
        type_: TriggerType::None,
        index: 0,
    }
}

/// Parse ParameterCommonValues from the binary stream.
///
/// The binary starts with a size prefix, then version-dependent struct data.
pub(crate) fn parse_common_values(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<ParameterCommonValues, Error> {
    let size = reader.read_i32()? as usize;
    let start_pos = reader.position();

    let result = if version >= 1802 {
        parse_v18(reader, config)?
    } else {
        // Version 14..1801 (ParameterCommonValues_V17)
        parse_v17(reader, config)?
    };

    // Advance to start_pos + size for forward compatibility
    let consumed = reader.position() - start_pos;
    if size > consumed {
        reader.skip(size - consumed)?;
    }

    Ok(result)
}

/// Parse V18 layout (version >= 1802) — up to 100 bytes.
fn parse_v18(
    reader: &mut BinaryReader,
    config: &ParseConfig,
) -> Result<ParameterCommonValues, Error> {
    let ref_eq_max_generation = reader.read_i32()?;
    let ref_eq_life = reader.read_ref_min_max()?;
    let ref_eq_interval = reader.read_ref_min_max()?;
    let ref_eq_offset = reader.read_ref_min_max()?;
    let ref_eq_burst = reader.read_ref_min_max()?;
    let max_generation = reader.read_i32()?;
    let translation_bind_type: TranslationParentBindType =
        reader.read_enum(config, "CommonValues.translation_bind_type")?;
    let rotation_bind_type: BindType =
        reader.read_enum(config, "CommonValues.rotation_bind_type")?;
    let scaling_bind_type: BindType = reader.read_enum(config, "CommonValues.scaling_bind_type")?;
    let generation_type: GenerationTiming =
        reader.read_enum(config, "CommonValues.generation_type")?;
    let removal_flags_raw = reader.read_i32()?;
    let removal_flags = RemovalTiming::from_bits_truncate(removal_flags_raw);
    let life = reader.read_random_int()?;
    let interval = reader.read_random_float()?;
    let offset = reader.read_random_float()?;
    let burst = reader.read_random_int()?;

    // Triggers encoded as u16: low byte = type, high byte = index
    let trigger_to_start = read_trigger_u16(reader)?;
    let trigger_to_stop = read_trigger_u16(reader)?;
    let trigger_to_remove = read_trigger_u16(reader)?;
    let trigger_to_generate = read_trigger_u16(reader)?;

    Ok(ParameterCommonValues {
        ref_eq_max_generation,
        ref_eq_life,
        max_generation,
        translation_bind_type,
        rotation_bind_type,
        scaling_bind_type,
        life,
        generation: GenerationParameter {
            ref_eq_interval,
            ref_eq_offset,
            ref_eq_burst,
            type_: generation_type,
            interval,
            offset,
            burst,
            trigger_to_start,
            trigger_to_stop,
            trigger_to_generate,
        },
        removal: RemovalParameter {
            flags: removal_flags,
            trigger_to_remove,
        },
    })
}

/// Read a trigger encoded as u16 (low byte = TriggerType, high byte = index).
fn read_trigger_u16(reader: &mut BinaryReader) -> Result<TriggerValues, Error> {
    let val = reader.read_u16()?;
    let type_raw = (val & 0xFF) as u8;
    let index = ((val >> 8) & 0xFF) as u8;
    Ok(TriggerValues {
        type_: TriggerType::from(type_raw),
        index,
    })
}

/// Parse V17 layout (version 14..1801) — 80 bytes.
fn parse_v17(
    reader: &mut BinaryReader,
    config: &ParseConfig,
) -> Result<ParameterCommonValues, Error> {
    let ref_eq_max_generation = reader.read_i32()?;
    let ref_eq_life = reader.read_ref_min_max()?;
    let ref_eq_generation_time = reader.read_ref_min_max()?;
    let ref_eq_generation_time_offset = reader.read_ref_min_max()?;
    let max_generation = reader.read_i32()?;
    let translation_bind_type: TranslationParentBindType =
        reader.read_enum(config, "CommonValues.translation_bind_type")?;
    let rotation_bind_type: BindType =
        reader.read_enum(config, "CommonValues.rotation_bind_type")?;
    let scaling_bind_type: BindType = reader.read_enum(config, "CommonValues.scaling_bind_type")?;

    // 3 bool removal fields → convert to bitflags
    let remove_when_life = reader.read_i32()?;
    let remove_when_parent = reader.read_i32()?;
    let remove_when_children = reader.read_i32()?;

    let mut removal_flags = RemovalTiming::empty();
    if remove_when_life != 0 {
        removal_flags |= RemovalTiming::WHEN_LIFE_IS_EXTINCT;
    }
    if remove_when_parent != 0 {
        removal_flags |= RemovalTiming::WHEN_PARENT_IS_REMOVED;
    }
    if remove_when_children != 0 {
        removal_flags |= RemovalTiming::WHEN_CHILDREN_IS_EXTINCT;
    }

    let life = reader.read_random_int()?;
    let generation_time = reader.read_random_float()?;
    let generation_time_offset = reader.read_random_float()?;

    Ok(ParameterCommonValues {
        ref_eq_max_generation,
        ref_eq_life,
        max_generation,
        translation_bind_type,
        rotation_bind_type,
        scaling_bind_type,
        life,
        generation: GenerationParameter {
            ref_eq_interval: ref_eq_generation_time,
            ref_eq_offset: ref_eq_generation_time_offset,
            ref_eq_burst: RefMinMax::default(),
            type_: GenerationTiming::Continuous,
            interval: generation_time,
            offset: generation_time_offset,
            burst: RandomInt { max: 1, min: 1 },
            trigger_to_start: default_trigger(),
            trigger_to_stop: default_trigger(),
            trigger_to_generate: default_trigger(),
        },
        removal: RemovalParameter {
            flags: removal_flags,
            trigger_to_remove: default_trigger(),
        },
    })
}

/// Parse TriggerParam section (version 1701..1801 only).
/// This merges trigger data into the already-parsed CommonValues.
pub(crate) fn parse_trigger_param(
    reader: &mut BinaryReader,
    version: i32,
    common: &mut ParameterCommonValues,
) -> Result<(), Error> {
    if !(1701..1802).contains(&version) {
        // Nothing to read for these versions
        return Ok(());
    }

    let flags = reader.read_u8()?;

    let read_trigger = |reader: &mut BinaryReader| -> Result<TriggerValues, Error> {
        let type_raw = reader.read_u8()?;
        let index = reader.read_u8()?;
        Ok(TriggerValues {
            type_: TriggerType::from(type_raw),
            index,
        })
    };

    if flags & (1 << 0) != 0 {
        common.generation.trigger_to_start = read_trigger(reader)?;
    }
    if flags & (1 << 1) != 0 {
        common.generation.trigger_to_stop = read_trigger(reader)?;
    }
    if flags & (1 << 2) != 0 {
        let trigger = read_trigger(reader)?;
        common.removal.trigger_to_remove = trigger;
        if trigger.type_ != TriggerType::None {
            common.removal.flags |= RemovalTiming::WHEN_TRIGGERED;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a V17 (version 1500-1801) CommonValues binary.
    fn build_v17_common_values() -> Vec<u8> {
        let mut data = Vec::new();
        // size: 80 bytes
        data.extend_from_slice(&80i32.to_le_bytes());
        // ref_eq_max_generation: -1
        data.extend_from_slice(&(-1i32).to_le_bytes());
        // ref_eq_life: { max: -1, min: -1 }
        data.extend_from_slice(&(-1i32).to_le_bytes());
        data.extend_from_slice(&(-1i32).to_le_bytes());
        // ref_eq_generation_time: { max: -1, min: -1 }
        data.extend_from_slice(&(-1i32).to_le_bytes());
        data.extend_from_slice(&(-1i32).to_le_bytes());
        // ref_eq_generation_time_offset: { max: -1, min: -1 }
        data.extend_from_slice(&(-1i32).to_le_bytes());
        data.extend_from_slice(&(-1i32).to_le_bytes());
        // max_generation: 10
        data.extend_from_slice(&10i32.to_le_bytes());
        // translation_bind_type: Always (2)
        data.extend_from_slice(&2i32.to_le_bytes());
        // rotation_bind_type: Always (2)
        data.extend_from_slice(&2i32.to_le_bytes());
        // scaling_bind_type: Always (2)
        data.extend_from_slice(&2i32.to_le_bytes());
        // remove_when_life_is_extinct: 1
        data.extend_from_slice(&1i32.to_le_bytes());
        // remove_when_parent_is_removed: 0
        data.extend_from_slice(&0i32.to_le_bytes());
        // remove_when_children_is_extinct: 0
        data.extend_from_slice(&0i32.to_le_bytes());
        // life: { max: 60, min: 60 }
        data.extend_from_slice(&60i32.to_le_bytes());
        data.extend_from_slice(&60i32.to_le_bytes());
        // generation_time: { max: 1.0, min: 1.0 }
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        // generation_time_offset: { max: 0.0, min: 0.0 }
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data
    }

    #[test]
    fn test_parse_common_values_v17() {
        let data = build_v17_common_values();
        let config = ParseConfig::default();
        let mut reader = BinaryReader::new(&data);
        let cv = parse_common_values(&mut reader, 1500, &config).unwrap();
        assert_eq!(cv.max_generation, 10);
        assert_eq!(cv.life.max, 60);
        assert_eq!(cv.translation_bind_type, TranslationParentBindType::Always);
        assert!(
            cv.removal
                .flags
                .contains(RemovalTiming::WHEN_LIFE_IS_EXTINCT)
        );
        assert!(
            !cv.removal
                .flags
                .contains(RemovalTiming::WHEN_PARENT_IS_REMOVED)
        );
        assert_eq!(cv.generation.type_, GenerationTiming::Continuous);
        assert_eq!(cv.generation.burst.max, 1);
    }

    /// Build a V18 (version >= 1802) CommonValues binary.
    fn build_v18_common_values() -> Vec<u8> {
        let mut data = Vec::new();
        // size: 100 bytes
        data.extend_from_slice(&100i32.to_le_bytes());
        // ref_eq_max_generation: -1
        data.extend_from_slice(&(-1i32).to_le_bytes());
        // ref_eq_life
        data.extend_from_slice(&(-1i32).to_le_bytes());
        data.extend_from_slice(&(-1i32).to_le_bytes());
        // ref_eq_interval
        data.extend_from_slice(&(-1i32).to_le_bytes());
        data.extend_from_slice(&(-1i32).to_le_bytes());
        // ref_eq_offset
        data.extend_from_slice(&(-1i32).to_le_bytes());
        data.extend_from_slice(&(-1i32).to_le_bytes());
        // ref_eq_burst
        data.extend_from_slice(&(-1i32).to_le_bytes());
        data.extend_from_slice(&(-1i32).to_le_bytes());
        // max_generation: 5
        data.extend_from_slice(&5i32.to_le_bytes());
        // translation_bind_type: Always (2)
        data.extend_from_slice(&2i32.to_le_bytes());
        // rotation_bind_type: Always (2)
        data.extend_from_slice(&2i32.to_le_bytes());
        // scaling_bind_type: Always (2)
        data.extend_from_slice(&2i32.to_le_bytes());
        // generation_type: Continuous (0)
        data.extend_from_slice(&0i32.to_le_bytes());
        // removal_flags: WhenLifeIsExtinct (1)
        data.extend_from_slice(&1i32.to_le_bytes());
        // life: { max: 100, min: 50 }
        data.extend_from_slice(&100i32.to_le_bytes());
        data.extend_from_slice(&50i32.to_le_bytes());
        // interval: { max: 2.0, min: 1.0 }
        data.extend_from_slice(&2.0f32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        // offset: { max: 0.0, min: 0.0 }
        data.extend_from_slice(&0.0f32.to_le_bytes());
        data.extend_from_slice(&0.0f32.to_le_bytes());
        // burst: { max: 3, min: 1 }
        data.extend_from_slice(&3i32.to_le_bytes());
        data.extend_from_slice(&1i32.to_le_bytes());
        // trigger_to_start: None (0x0000)
        data.extend_from_slice(&0u16.to_le_bytes());
        // trigger_to_stop: None (0x0000)
        data.extend_from_slice(&0u16.to_le_bytes());
        // trigger_to_remove: None (0x0000)
        data.extend_from_slice(&0u16.to_le_bytes());
        // trigger_to_generate: None (0x0000)
        data.extend_from_slice(&0u16.to_le_bytes());
        data
    }

    #[test]
    fn test_parse_common_values_v18() {
        let data = build_v18_common_values();
        let config = ParseConfig::default();
        let mut reader = BinaryReader::new(&data);
        let cv = parse_common_values(&mut reader, 1802, &config).unwrap();
        assert_eq!(cv.max_generation, 5);
        assert_eq!(cv.life.max, 100);
        assert_eq!(cv.life.min, 50);
        assert_eq!(cv.generation.burst.max, 3);
        assert_eq!(cv.generation.type_, GenerationTiming::Continuous);
    }
}
