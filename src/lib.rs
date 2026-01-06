//! # fontmesh
//!
//! A pure Rust library for converting TrueType font glyphs to 2D and 3D triangle meshes.
//!
//! This library provides a **stateless, functional API** for generating triangle meshes from
//! TrueType fonts. It uses pure functions that work directly with `ttf_parser::Face`,
//! giving you full control over parsing and caching strategies.
//!
//! ## Features
//!
//! - **Pure Rust**: No C dependencies, fully cross-platform including WASM
//! - **Stateless API**: Pure functions with no hidden state
//! - **2D & 3D**: Generate both flat 2D meshes and extruded 3D meshes
//! - **Quality Control**: Adjustable tessellation quality
//! - **Efficient**: Uses lyon_tessellation for robust triangulation
//! - **Flexible**: You control when to parse and cache fonts
//!
//! ## Basic Usage
//!
//! ```ignore
//! use fontmesh::{Face, char_to_mesh_2d, char_to_mesh_3d};
//!
//! // Parse the font (this is fast - just table header parsing)
//! let font_data = include_bytes!("path/to/font.ttf");
//! let face = Face::parse(font_data, 0)?;
//!
//! // Generate a 2D mesh with 20 subdivisions per curve
//! let mesh_2d = char_to_mesh_2d(&face, 'A', 20)?;
//!
//! // Generate a 3D mesh with depth 5.0 and 20 subdivisions
//! let mesh_3d = char_to_mesh_3d(&face, 'A', 5.0, 20)?;
//! ```
//!
//! ## Caching Strategy
//!
//! Because `Face::parse()` is extremely fast (it only reads the table directory), you should **not**
//! attempt to cache the `Face` struct itself. Doing so is difficult because `Face` borrows the font data.
//!
//! Instead, simply store your font data (e.g., in a `Vec<u8>` or `Arc<Vec<u8>>`) and parse it on-demand
//! whenever you need to generate a mesh.
//!
//! ```ignore
//! use std::collections::HashMap;
//! use std::sync::Arc;
//! use fontmesh::Face;
//!
//! // Simple cache: just store the font data
//! let mut font_cache: HashMap<String, Arc<Vec<u8>>> = HashMap::new();
//! font_cache.insert("myfont".into(), Arc::new(font_data.to_vec()));
//!
//! // Parse Face on-demand (fast!)
//! let data = font_cache.get("myfont").unwrap();
//! let face = Face::parse(data, 0)?;
//! let mesh = fontmesh::char_to_mesh_3d(&face, 'A', 5.0, 20)?;
//! ```
//!
//! ## Font Metrics
//!
//! Helper functions for common font metrics (normalized to 1.0 em):
//!
//! ```ignore
//! use fontmesh::{Face, ascender, descender, line_gap, glyph_advance};
//!
//! let face = Face::parse(font_data, 0)?;
//!
//! let asc = ascender(&face);      // Font ascender
//! let desc = descender(&face);    // Font descender
//! let gap = line_gap(&face);      // Line gap
//! let line_height = asc - desc + gap;
//!
//! // Get advance width for a character
//! if let Some(width) = glyph_advance(&face, 'A') {
//!     println!("'A' advance width: {}", width);
//! }
//! ```
//!
//! ## Advanced: Pipeline Stages
//!
//! The mesh generation pipeline has discrete stages that you can access directly:
//!
//! 1. **Parse Font**: `Face::parse()` → Font tables
//! 2. **Extract Outline**: (internal) → Raw Bezier curves
//! 3. **Linearization**: (internal) → Straight line segments
//! 4. **Triangulation**: `triangulate()` → 2D triangle mesh
//! 5. **Extrusion**: `extrude()` → 3D mesh with depth
//!
//! ```ignore
//! use fontmesh::{Face, triangulate, extrude, Outline2D};
//!
//! let face = Face::parse(font_data, 0)?;
//!
//! // Lower-level pipeline access (if you need it)
//! // Most users should just use char_to_mesh_2d/3d
//! ```
//!
//! ## Integration with Text Shaping
//!
//! Works seamlessly with text shaping libraries like `rustybuzz` or `cosmic-text`:
//!
//! ```ignore
//! use fontmesh::{Face, GlyphId, char_to_mesh_3d};
//!
//! let face = Face::parse(font_data, 0)?;
//!
//! // If you have glyph IDs from a shaping library, you can still use
//! // the Face directly with ttf-parser APIs
//! let glyph_id = GlyphId(42);
//! // ... then generate meshes per character as needed
//! ```

pub mod error;
pub mod extrude;
pub mod font;
pub mod glyph;
pub mod linearize;
pub mod triangulate;
pub mod types;

// Re-export main types
pub use error::{FontMeshError, Result};
pub use types::{Mesh2D, Mesh3D, Outline2D};

// Re-export ttf-parser types for direct usage
pub use ttf_parser::{Face, GlyphId};

// Re-export core pure functions (stateless API)
pub use glyph::{char_to_mesh_2d, char_to_mesh_3d, Glyph};

// Re-export font utilities
pub use font::{ascender, descender, glyph_advance, line_gap, parse_font};

// Re-export pipeline functions for advanced usage
pub use extrude::{compute_smooth_normals, extrude};
pub use linearize::linearize_outline;
pub use triangulate::triangulate;

#[cfg(test)]
mod tests {

    // Tests will be added when we have test fonts
    #[test]
    fn test_api_compiles() {
        // This test just verifies the API compiles
    }
}
