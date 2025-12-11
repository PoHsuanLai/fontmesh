# fontmesh

[![CI](https://github.com/PoHsuanLai/fontmesh/actions/workflows/ci.yml/badge.svg)](https://github.com/PoHsuanLai/fontmesh/actions)
[![Crates.io](https://img.shields.io/crates/v/fontmesh)](https://crates.io/crates/fontmesh)
[![Documentation](https://docs.rs/fontmesh/badge.svg)](https://docs.rs/fontmesh)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A fast Rust library for converting TrueType font glyphs to 2D and 3D triangle meshes. Providing a faster, pure rust alternative for [ttf2mesh](https://github.com/blaind/ttf2mesh-rs)

<p align="center">
  <img src="images/fontmesh_logo.png" width="70%" alt="3D Text" />
</p>

<p align="center">
  <img src="images/glyph_2d.png" width="45%" alt="2D Mesh" />
  <img src="images/glyph_3d.png" width="45%" alt="3D Mesh" />
</p>

## Quick Start

```rust
use fontmesh::Font;

// Load font
let font = Font::from_bytes(include_bytes!("font.ttf"))?;

// Generate a 2D mesh (default: 20 subdivisions per curve)
let mesh_2d = font.glyph_to_mesh_2d('A')?;

// Generate a 3D mesh with custom quality (50 subdivisions per curve)
let mesh_3d = font.glyph_by_char('A')?
    .with_subdivisions(50)
    .to_mesh_3d(5.0)?;
```

## API

fontmesh provides a chainable API that works at different levels of abstraction:

```rust
// High-level: One-step mesh generation (uses default: 20 subdivisions)
let mesh = font.glyph_to_mesh_3d('A', 5.0)?;

// Mid-level: Chain operations with custom subdivisions
let mesh = font.glyph_by_char('A')?
    .with_subdivisions(50)
    .to_mesh_3d(5.0)?;

// Low-level: Access intermediate pipeline stages
let glyph = font.glyph_by_char('A')?;
let outline = glyph.linearize()?;  // Uses default subdivisions
let mesh_2d = outline.triangulate()?;
let mesh_3d = mesh_2d.extrude(&outline, 5.0)?;
```


## Performance

fontmesh is **2-3x faster** than comparable libraries, with the incredible Lyon.

<p align="center">
  <img src="images/benchmark.png" width="85%" alt="Benchmark Comparison" />
</p>

Run benchmarks yourself: `cargo bench --all-features`

## Examples

```bash
# Basic usage
cargo run --example basic

# Chainable API examples
cargo run --example fluent_api

# Export glyphs to OBJ format
cargo run --example export_obj
```

## Pipeline

The mesh generation pipeline consists of the following stages:

1. **Font Loading** - Parse TrueType fonts with ttf-parser
2. **Outline Extraction** - Get glyph Bezier curves
3. **Linearization** - Convert curves to line segments using adaptive subdivision
4. **Triangulation** - Generate 2D triangle mesh with lyon_tessellation
5. **Extrusion** - Create 3D mesh with depth and smooth normals

Each stage can be accessed individually, allowing you to:
- Export vector outlines for SVG/PDF rendering
- Reuse 2D meshes for multiple extrusion depths
- Apply custom post-processing (e.g., `compute_smooth_normals`)
- Implement custom rendering pipelines

## License

MIT
