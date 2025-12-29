# fontmesh

[![CI](https://github.com/PoHsuanLai/fontmesh/actions/workflows/ci.yml/badge.svg)](https://github.com/PoHsuanLai/fontmesh/actions)
[![Crates.io](https://img.shields.io/crates/v/fontmesh)](https://crates.io/crates/fontmesh)
[![Documentation](https://docs.rs/fontmesh/badge.svg)](https://docs.rs/fontmesh)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A fast Rust library for converting TrueType font glyphs to 2D and 3D triangle meshes. A faster, pure Rust alternative to [ttf2mesh](https://github.com/blaind/ttf2mesh-rs).

<p align="center"><img src="images/fontmesh_logo.png" width="70%" alt="3D Text" /></p>

<p align="center">
  <img src="images/glyph_2d.png" width="45%" alt="2D Mesh" />
  <img src="images/glyph_3d.png" width="45%" alt="3D Mesh" />
</p>

## Quick Start

```rust
use fontmesh::Font;

let font = Font::from_bytes(include_bytes!("font.ttf"))?;

// 2D mesh
let mesh_2d = font.glyph_to_mesh_2d('A')?;

// 3D mesh with custom quality
let mesh_3d = font.glyph_by_char('A')?
    .with_subdivisions(50)
    .to_mesh_3d(5.0)?;
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

MIT
