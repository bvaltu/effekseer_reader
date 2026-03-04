# effekseer_reader

Pure Rust parser for [Effekseer](https://effekseer.github.io/en/) particle effect files.

Supports `.efkefc`, `.efkmat`, `.efkmodel`, and NURBS curve formats.

> **Note:** This crate is under active development as part of an effort to build a Bevy integration for Effekseer. The API may change before 1.0.

## Supported Versions

| effekseer_reader | Effekseer | Binary versions |
|------------------|-----------|-----------------|
| 0.1.x            | 1.5 - 1.8 | 1500 - 1810    |

Files outside the supported binary version range are rejected with `Error::UnsupportedVersion`.

## Features

- Zero `unsafe` code (`#![forbid(unsafe_code)]`)
- Parses all major Effekseer binary formats
- Full node tree with renderer parameters, force fields, and GPU particles
- Configurable resource limits for untrusted input
- Optional serde serialization
- Optional runtime evaluation (F-Curves, easing, gradients, NURBS)

## Installation

```toml
[dependencies]
effekseer_reader = "0.1"
```

### Feature Flags

| Feature | Description |
|---------|-------------|
| `serde` | Derive `Serialize`/`Deserialize` on all public types |
| `eval`  | Runtime evaluation helpers for F-Curves, easing functions, gradient sampling, and NURBS curves |

## Usage

```rust,no_run
use effekseer_reader::load_efkefc;

let data = std::fs::read("effect.efkefc").unwrap();
let effect = load_efkefc(&data).unwrap();

println!("Version: {}", effect.version);
println!("Textures: {}", effect.color_images.len());
println!("Root children: {}", effect.root.children.len());
```

### Parsing other formats

```rust,no_run
// Material files
let mat = effekseer_reader::load_material(&std::fs::read("mat.efkmat").unwrap()).unwrap();

// Model files
let model = effekseer_reader::load_model(&std::fs::read("mesh.efkmodel").unwrap()).unwrap();

// NURBS curves
let curve = effekseer_reader::load_curve(&std::fs::read("path.efkcurve").unwrap()).unwrap();
```

### Configurable parsing

```rust
use effekseer_reader::types::{ParseConfig, UnknownEnumBehavior};

let config = ParseConfig {
    unknown_enum_behavior: UnknownEnumBehavior::Warn,
    ..Default::default()
};
// let effect = effekseer_reader::load_efkefc_with_config(&data, &config).unwrap();
```

## MSRV

The minimum supported Rust version is **1.85** (edition 2024).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
