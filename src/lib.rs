#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Pure Rust parser for Effekseer particle effect files.
//!
//! Supports `.efkefc`, `.efkmat`, `.efkmodel`, and NURBS curve formats
//! for Effekseer versions 1500 (v15) through 1810 (v18).

pub mod error;
pub mod reader;
pub mod types;
pub mod version;

pub mod parser;

#[cfg(feature = "eval")]
pub mod eval;

// Re-export top-level API
pub use error::Error;
pub use parser::{
    EfkPkg, EfkPkgFile, EfkPkgFileType, load_curve, load_curve_with_config, load_efk,
    load_efk_with_config, load_efkefc, load_efkefc_with_config, load_efkpkg,
    load_efkpkg_with_config, load_material, load_material_with_config, load_model,
    load_model_with_config,
};
pub use types::*;
