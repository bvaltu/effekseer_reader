//! Kill rules parser.

use crate::error::Error;
use crate::reader::BinaryReader;
use crate::types::kill_rules::{KillRulesParameter, KillTypeParams};
use crate::types::{KillType, ParseConfig};

/// Parse KillRulesParameter (version >= 1704).
pub(crate) fn parse_kill_rules(
    reader: &mut BinaryReader,
    version: i32,
    config: &ParseConfig,
) -> Result<KillRulesParameter, Error> {
    if version >= 1704 {
        let kill_type: KillType = reader.read_enum(config, "KillRules.type")?;
        let is_scale_and_rotation_applied = reader.read_i32_as_bool()?;

        let params = match kill_type {
            KillType::Box => {
                let center = reader.read_vector3d()?;
                let size = reader.read_vector3d()?;
                let is_kill_inside = reader.read_i32_as_bool()?;
                KillTypeParams::Box {
                    center,
                    size,
                    is_kill_inside,
                }
            }
            KillType::Plane => {
                let plane_axis = reader.read_vector3d()?;
                let plane_offset = reader.read_f32()?;
                KillTypeParams::Plane {
                    plane_axis,
                    plane_offset,
                }
            }
            KillType::Sphere => {
                let center = reader.read_vector3d()?;
                let radius = reader.read_f32()?;
                let is_kill_inside = reader.read_i32_as_bool()?;
                KillTypeParams::Sphere {
                    center,
                    radius,
                    is_kill_inside,
                }
            }
            _ => KillTypeParams::None,
        };

        Ok(KillRulesParameter {
            kill_type,
            is_scale_and_rotation_applied,
            params,
        })
    } else {
        Ok(KillRulesParameter {
            kill_type: KillType::None,
            is_scale_and_rotation_applied: false,
            params: KillTypeParams::None,
        })
    }
}
