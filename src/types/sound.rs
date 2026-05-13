//! Sound parameter types.

use super::enums::{ParameterSoundPanType, ParameterSoundType};
use super::primitives::{RandomFloat, RandomInt};

/// Sound parameters for a node.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterSound {
    /// Sound type (None or Use).
    pub sound_type: ParameterSoundType,
    /// Sound data (present only if sound_type == Use).
    pub data: Option<SoundData>,
}

impl Default for ParameterSound {
    fn default() -> Self {
        Self {
            sound_type: ParameterSoundType::default(),
            data: None,
        }
    }
}

/// Sound data (when sound is enabled).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SoundData {
    /// Wave resource index.
    pub wave_index: i32,
    /// Volume range.
    pub volume: RandomFloat,
    /// Pitch range.
    pub pitch: RandomFloat,
    /// Panning type.
    pub pan_type: ParameterSoundPanType,
    /// Pan value range.
    pub pan: RandomFloat,
    /// Sound distance.
    pub distance: f32,
    /// Delay range.
    pub delay: RandomInt,
}
