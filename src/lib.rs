//! # fontmesh
//!
//! A pure Rust library for converting TrueType font glyphs to 2D and 3D triangle meshes.
//!
//! This library provides a simple API for loading TrueType fonts and generating triangle
//! meshes from individual glyphs. It supports both 2D (flat) and 3D (extruded) meshes.
//!
//! ## Features
//!
//! - **Pure Rust**: No C dependencies, fully cross-platform including WASM
//! - **2D & 3D**: Generate both flat 2D meshes and extruded 3D meshes
//! - **Quality Control**: Adjustable tessellation quality
//! - **Efficient**: Uses lyon_tessellation for robust triangulation
//! - **Fluent API**: Chain operations for ergonomic usage
//!
//! ## Basic Usage
//!
//! ```ignore
//! use fontmesh::Font;
//!
//! // Load font
//! let font = Font::from_bytes(include_bytes!("path/to/font.ttf"))?;
//!
//! // Generate a 2D mesh (uses default quality: 20 subdivisions)
//! let mesh_2d = font.glyph_to_mesh_2d('A')?;
//!
//! // Generate a 3D mesh (uses default quality: 20 subdivisions)
//! let mesh_3d = font.glyph_to_mesh_3d('A', 5.0)?;
//! ```
//!
//! ## Fluent API
//!
//! For more control, use the fluent API to chain operations:
//!
//! ```ignore
//! use fontmesh::Font;
//!
//! let font = Font::from_bytes(include_bytes!("font.ttf"))?;
//!
//! // Fluent 2D mesh generation with default quality
//! let mesh_2d = font.glyph_by_char('A')?.to_mesh_2d()?;
//!
//! // Fluent 3D mesh generation with custom quality (50 subdivisions)
//! let mesh_3d = font.glyph_by_char('A')?.with_subdivisions(50).to_mesh_3d(5.0)?;
//!
//! // Access intermediate pipeline stages
//! let outline = font.glyph_by_char('A')?.linearize()?;
//! let mesh_2d = outline.triangulate()?;
//! let mesh_3d = mesh_2d.extrude(&outline, 5.0)?;
//! ```
//!
//! ## Advanced Usage
//!
//! The mesh generation pipeline has discrete stages that you can access:
//!
//! 1. **Outline Extraction**: `Glyph::outline()` → Raw Bezier curves
//! 2. **Linearization**: `linearize_outline()` → Straight line segments
//! 3. **Triangulation**: `triangulate()` → 2D triangle mesh
//! 4. **Extrusion**: `extrude()` → 3D mesh with depth
//!
//! ```ignore
//! use fontmesh::{Font, linearize_outline, triangulate, extrude};
//!
//! let font = Font::from_bytes(include_bytes!("font.ttf"))?;
//! let glyph = font.glyph_by_char('A')?;
//!
//! // Get raw Bezier curves
//! let outline = glyph.outline()?;
//!
//! // Convert curves to line segments (50 subdivisions per curve)
//! let linearized = linearize_outline(outline, 50)?;
//!
//! // Triangulate into 2D mesh
//! let mesh_2d = triangulate(&linearized)?;
//!
//! // Extrude to create multiple 3D variations
//! let shallow = extrude(&mesh_2d, &linearized, 1.0)?;
//! let deep = extrude(&mesh_2d, &linearized, 10.0)?;
//! ```
//!
//! ## Integration with Text Shaping
//!
//! For integration with text shaping libraries like `rustybuzz` or `cosmic-text`,
//! use glyph IDs directly:
//!
//! ```ignore
//! use fontmesh::{Font, GlyphId};
//!
//! let font = Font::from_bytes(include_bytes!("font.ttf"))?;
//!
//! // From text shaping (e.g., rustybuzz)
//! let glyph_id = GlyphId(42);
//! let glyph = font.glyph_by_id(glyph_id, 'A');
//! let mesh = glyph.with_subdivisions(50).to_mesh_3d(5.0)?;
//!
//! // Access glyph ID for caching
//! let id = glyph.glyph_id();
//!
//! // Access ttf-parser for advanced features
//! let face = font.face();
//! let kerning = face.kerning_subtables();
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
pub use font::Font;
pub use glyph::{Glyph, GlyphMeshBuilder};
pub use types::{Mesh2D, Mesh3D, Outline2D};

// Re-export ttf-parser types for advanced usage
pub use ttf_parser::GlyphId;

// Re-export pipeline functions for advanced usage
pub use extrude::{compute_smooth_normals, extrude};
pub use linearize::linearize_outline;
pub use triangulate::triangulate;

impl<'a> Font<'a> {
    /// Convert a character glyph to a 2D triangle mesh
    ///
    /// Uses default quality (20 subdivisions per curve).
    ///
    /// # Arguments
    /// * `character` - The character to convert
    ///
    /// # Returns
    /// A 2D triangle mesh
    ///
    /// # Example
    /// ```ignore
    /// let mesh = font.glyph_to_mesh_2d('A')?;
    /// println!("Generated {} vertices", mesh.vertices.len());
    /// ```
    pub fn glyph_to_mesh_2d(&self, character: char) -> Result<Mesh2D> {
        let glyph = self.glyph_by_char(character)?;
        glyph.to_mesh_2d()
    }

    /// Convert a character glyph to a 3D triangle mesh with extrusion
    ///
    /// Uses default quality (20 subdivisions per curve).
    ///
    /// # Arguments
    /// * `character` - The character to convert
    /// * `depth` - The extrusion depth
    ///
    /// # Returns
    /// A 3D triangle mesh with normals
    ///
    /// # Example
    /// ```ignore
    /// let mesh = font.glyph_to_mesh_3d('A', 5.0)?;
    /// println!("Generated {} triangles", mesh.triangle_count());
    /// ```
    pub fn glyph_to_mesh_3d(&self, character: char, depth: f32) -> Result<Mesh3D> {
        let glyph = self.glyph_by_char(character)?;
        glyph.to_mesh_3d(depth)
    }
}

#[cfg(test)]
mod tests {

    // Tests will be added when we have test fonts
    #[test]
    fn test_api_compiles() {
        // This test just verifies the API compiles
    }
}
