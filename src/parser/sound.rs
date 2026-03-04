//! Sound parameter parser.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::sound::{ParameterSound, SoundData};
use crate::types::{ParameterSoundPanType, ParameterSoundType, ParseConfig};

/// Parse ParameterSound.
pub(crate) fn parse_sound(
    reader: &mut BinaryReader,
    config: &ParseConfig,
) -> Result<ParameterSound, Error> {
    let sound_type: ParameterSoundType = reader.read_enum(config, "Sound.type")?;

    let data = if sound_type == ParameterSoundType::Use {
        let wave_index = reader.read_i32()?;
        let volume = reader.read_random_float()?;
        let pitch = reader.read_random_float()?;
        let pan_type: ParameterSoundPanType = reader.read_enum(config, "Sound.pan_type")?;
        let pan = reader.read_random_float()?;
        let distance = reader.read_f32()?;
        let delay = reader.read_random_int()?;
        Some(SoundData {
            wave_index,
            volume,
            pitch,
            pan_type,
            pan,
            distance,
            delay,
        })
    } else {
        None
    };

    Ok(ParameterSound { sound_type, data })
}
