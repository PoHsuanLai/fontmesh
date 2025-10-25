# fontmesh

A fast Rust library for converting TrueType font glyphs to 2D and 3D triangle meshes.

<p align="center">
  <img src="images/glyph_2d.png" width="45%" alt="2D Mesh" />
  <img src="images/glyph_3d.png" width="45%" alt="3D Mesh" />
</p>

<p align="center">
  <img src="images/text_3d.png" width="70%" alt="3D Text" />
</p>

## Quick Start

```rust
use fontmesh::{Font, Quality};

let font_data = include_bytes!("font.ttf");
let font = Font::from_bytes(font_data)?;

// Generate a 2D mesh
let mesh_2d = font.glyph_to_mesh_2d('A', Quality::Normal)?;

// Generate a 3D mesh with depth
let mesh_3d = font.glyph_to_mesh_3d('A', Quality::High, 5.0)?;
```

## Performance

fontmesh is **2-3x faster** than comparable libraries, with robust triangulation via Lyon.

<p align="center">
  <img src="images/benchmark_comparison.png" width="85%" alt="Benchmark Comparison" />
</p>

Run benchmarks yourself: `cargo bench`

## Examples

```bash
# Basic usage
cargo run --example basic

# Efficient buffer reuse for batch processing
cargo run --example buffer_reuse

# Export glyphs to OBJ format
cargo run --example export_obj
```

## Quality Levels

- `Quality::Low` - 10 subdivisions (fastest)
- `Quality::Normal` - 20 subdivisions (balanced)
- `Quality::High` - 50 subdivisions (smoothest)
- `Quality::Custom(n)` - Custom subdivision count

## Buffer Reuse

For processing multiple glyphs, reuse buffers to avoid allocations:

```rust
let mut mesh = Mesh2D::new();

for c in "Hello".chars() {
    font.glyph_to_mesh_2d_reuse(c, Quality::Normal, &mut mesh)?;
    // Use mesh...
}
```

## How It Works

1. Parse font with ttf-parser
2. Extract glyph outline (Bezier curves)
3. Linearize curves using adaptive subdivision
4. Triangulate with lyon_tessellation
5. Optional: Extrude to 3D with smooth normals

## Dependencies

- **ttf-parser** - Font parsing (pure Rust, no_std)
- **lyon_tessellation** - Robust triangulation
- **glam** - Vector math

## License

MIT
