# fontmesh

[![CI](https://github.com/PoHsuanLai/fontmesh/actions/workflows/ci.yml/badge.svg)](https://github.com/PoHsuanLai/fontmesh/actions)
[![Crates.io](https://img.shields.io/crates/v/fontmesh)](https://crates.io/crates/fontmesh)
[![Documentation](https://docs.rs/fontmesh/badge.svg)](https://docs.rs/fontmesh)
[![License: MIT or Apache-2.0](https://img.shields.io/badge/License-MIT%20or%20Apache--2.0-blue.svg)](LICENSE-MIT)

Library for converting TrueType font glyphs to 2D and 3D triangle meshes. A faster, pure Rust alternative to [ttf2mesh](https://github.com/blaind/ttf2mesh-rs).

<p align="center"><img src="images/fontmesh_logo.png" width="70%" alt="3D Text" /></p>

<p align="center">
  <img src="images/glyph_2d.png" width="45%" alt="2D Mesh" />
  <img src="images/glyph_3d.png" width="45%" alt="3D Mesh" />
</p>

## Quick Start

```rust
use fontmesh::{Face, char_to_mesh_2d, char_to_mesh_3d};

let font_data = include_bytes!("font.ttf");
let face = Face::parse(font_data, 0)?;

// 2D mesh with 20 subdivisions per curve
let mesh_2d = char_to_mesh_2d(&face, 'A', 20)?;

// 3D mesh with depth 5.0 and 50 subdivisions
let mesh_3d = char_to_mesh_3d(&face, 'A', 5.0, 50)?;
```

## Caching
```rust
use std::collections::HashMap;
use std::sync::Arc;

// Cache just the font data
let mut cache: HashMap<String, Arc<Vec<u8>>> = HashMap::new();
cache.insert("myfont".into(), Arc::new(font_data.to_vec()));

// Parse Face when needed (cheap!)
let data = cache.get("myfont").unwrap();
let face = Face::parse(data, 0)?;
let mesh = char_to_mesh_3d(&face, 'A', 5.0, 20)?;
```

## Examples

```bash
cargo run --example basic
cargo run --example export_obj
cargo run --example serde --features serde
```

## Performance

fontmesh is **2-3x faster** than comparable libraries.

<p align="center">
  <img src="images/benchmark.png" width="85%" alt="Benchmark Comparison" />
</p>

Run benchmarks: `cargo bench`

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
