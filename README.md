# fontmesh

A pure Rust library for converting TrueType font glyphs to 2D and 3D triangle meshes.

## Features

- **Pure Rust**: No C dependencies, fully cross-platform including WASM
- **2D & 3D**: Generate both flat 2D meshes and extruded 3D meshes
- **Quality Control**: Adjustable tessellation quality (Low, Normal, High, Custom)
- **Efficient**: Uses lyon_tessellation for robust triangulation
- **Cross-platform**: Works on macOS, Linux, Windows, and WASM

## Why fontmesh?

This library was created as a modern, pure Rust replacement for ttf2mesh, addressing several key limitations:

- **Cross-platform**: No C dependencies means it works everywhere Rust works
- **WASM Support**: Can be compiled to WebAssembly
- **Modern**: Built with modern Rust practices and actively maintained
- **Safe**: Memory-safe with no unsafe code in the core API

## Example

```rust
use fontmesh::{Font, Quality};

// Load a font
let font_data = include_bytes!("path/to/font.ttf");
let font = Font::from_bytes(font_data)?;

// Generate a 2D mesh
let mesh_2d = font.glyph_to_mesh_2d('A', Quality::Normal)?;
println!("Vertices: {}, Triangles: {}",
    mesh_2d.vertex_count(),
    mesh_2d.triangle_count());

// Generate a 3D mesh with extrusion
let mesh_3d = font.glyph_to_mesh_3d('A', Quality::High, 5.0)?;
println!("Vertices: {}, Triangles: {}",
    mesh_3d.vertex_count(),
    mesh_3d.triangle_count());
```

## Quality Levels

fontmesh provides three preset quality levels:

- **Quality::Low** - 10 subdivisions (fast, lower quality)
- **Quality::Normal** - 20 subdivisions (balanced)
- **Quality::High** - 50 subdivisions (slow, high quality)
- **Quality::Custom(n)** - Custom subdivision count

Higher quality levels produce smoother curves but with more vertices.

## Running Examples

```bash
cargo run --example basic
```

## Architecture

fontmesh is built on top of:

- **ttf-parser**: TrueType font parsing (pure Rust, no_std compatible)
- **lyon_tessellation**: Robust 2D triangulation
- **glam**: Efficient vector math

The mesh generation pipeline:
1. Load font with ttf-parser
2. Extract glyph outline
3. Linearize Bezier curves (quadratic/cubic)
4. Triangulate with lyon (2D mesh)
5. Optional: Extrude to 3D with normals

## License

Licensed under MIT license

## Contributing

Contributions are welcome! This library is part of the dawAI project and was created to enable cross-platform 3D text rendering in Bevy.
