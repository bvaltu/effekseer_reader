//! Version constants for Effekseer binary formats.

use crate::error::Error;

// Effect binary versions
/// Effekseer v15.
pub const VERSION_15: i32 = 1500;
/// Effekseer v16 alpha 1.
pub const VERSION_16_ALPHA1: i32 = 1600;
/// Effekseer v16 alpha 2.
pub const VERSION_16_ALPHA2: i32 = 1601;
/// Effekseer v16 alpha 3.
pub const VERSION_16_ALPHA3: i32 = 1602;
/// Effekseer v16 alpha 4.
pub const VERSION_16_ALPHA4: i32 = 1603;
/// Effekseer v16 alpha 5.
pub const VERSION_16_ALPHA5: i32 = 1604;
/// Effekseer v16 alpha 6.
pub const VERSION_16_ALPHA6: i32 = 1605;
/// Effekseer v16 alpha 7.
pub const VERSION_16_ALPHA7: i32 = 1606;
/// Effekseer v16 alpha 8.
pub const VERSION_16_ALPHA8: i32 = 1607;
/// Effekseer v16 alpha 9.
pub const VERSION_16_ALPHA9: i32 = 1608;
/// Effekseer v16 stable.
pub const VERSION_16: i32 = 1610;
/// Effekseer v17 alpha 1.
pub const VERSION_17_ALPHA1: i32 = 1700;
/// Effekseer v17 alpha 2.
pub const VERSION_17_ALPHA2: i32 = 1701;
/// Effekseer v17 alpha 3.
pub const VERSION_17_ALPHA3: i32 = 1702;
/// Effekseer v17 alpha 4.
pub const VERSION_17_ALPHA4: i32 = 1703;
/// Effekseer v17 alpha 5.
pub const VERSION_17_ALPHA5: i32 = 1704;
/// Effekseer v17 alpha 6.
pub const VERSION_17_ALPHA6: i32 = 1705;
/// Effekseer v17 stable.
pub const VERSION_17: i32 = 1710;
/// Effekseer v18 alpha 1.
pub const VERSION_18_ALPHA1: i32 = 1800;
/// Effekseer v18 alpha 2.
pub const VERSION_18_ALPHA2: i32 = 1801;
/// Effekseer v18 alpha 3.
pub const VERSION_18_ALPHA3: i32 = 1802;
/// Effekseer v18 stable.
pub const VERSION_18: i32 = 1810;

/// Minimum effect binary version supported by this parser.
pub const MIN_SUPPORTED_VERSION: i32 = 1500;
/// Maximum effect binary version supported by this parser.
pub const MAX_SUPPORTED_VERSION: i32 = 1810;

// Material file versions
/// Material format version for Effekseer v15.
pub const MATERIAL_VERSION_15: i32 = 3;
/// Material format version for Effekseer v16.
pub const MATERIAL_VERSION_16: i32 = 1610;
/// Material format version for Effekseer v17 alpha 2.
pub const MATERIAL_VERSION_17_ALPHA2: i32 = 1700;
/// Material format version for Effekseer v17 alpha 4.
pub const MATERIAL_VERSION_17_ALPHA4: i32 = 1703;
/// Material format version for Effekseer v17.
pub const MATERIAL_VERSION_17: i32 = 1710;
/// Material format version for Effekseer v18.
pub const MATERIAL_VERSION_18: i32 = 1800;

// Compiled material versions
/// Compiled material format version for Effekseer v15.
pub const COMPILED_MATERIAL_VERSION_15: i32 = 1;
/// Compiled material format version for Effekseer v16.
pub const COMPILED_MATERIAL_VERSION_16: i32 = 1610;
/// Compiled material format version for Effekseer v16.2.
pub const COMPILED_MATERIAL_VERSION_162: i32 = 1612;

/// Validate that a version number is within the supported range.
pub fn validate_version(version: i32) -> Result<(), Error> {
    if !(MIN_SUPPORTED_VERSION..=MAX_SUPPORTED_VERSION).contains(&version) {
        return Err(Error::UnsupportedVersion { version });
    }
    Ok(())
}
