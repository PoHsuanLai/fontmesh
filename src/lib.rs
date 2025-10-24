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
//!
//! ## Example
//!
//! ```ignore
//! use fontmesh::{Font, Quality};
//!
//! // Load a font
//! let font_data = include_bytes!("path/to/font.ttf");
//! let font = Font::from_bytes(font_data)?;
//!
//! // Generate a 2D mesh
//! let mesh_2d = font.glyph_to_mesh_2d('A', Quality::Normal)?;
//!
//! // Generate a 3D mesh
//! let mesh_3d = font.glyph_to_mesh_3d('A', Quality::High, 5.0)?;
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
pub use glyph::Glyph;
pub use types::{Mesh2D, Mesh3D, Quality};

impl<'a> Font<'a> {
    /// Convert a character glyph to a 2D triangle mesh
    ///
    /// # Arguments
    /// * `character` - The character to convert
    /// * `quality` - The tessellation quality
    ///
    /// # Returns
    /// A 2D triangle mesh
    ///
    /// # Example
    /// ```ignore
    /// let mesh = font.glyph_to_mesh_2d('A', Quality::Normal)?;
    /// println!("Generated {} vertices", mesh.vertex_count());
    /// ```
    pub fn glyph_to_mesh_2d(&self, character: char, quality: Quality) -> Result<Mesh2D> {
        let glyph = self.glyph_by_char(character)?;
        let outline = glyph.linearize(quality)?;
        triangulate::triangulate(&outline)
    }

    /// Convert a character glyph to a 3D triangle mesh with extrusion
    ///
    /// # Arguments
    /// * `character` - The character to convert
    /// * `quality` - The tessellation quality
    /// * `depth` - The extrusion depth
    ///
    /// # Returns
    /// A 3D triangle mesh with normals
    ///
    /// # Example
    /// ```ignore
    /// let mesh = font.glyph_to_mesh_3d('A', Quality::High, 5.0)?;
    /// println!("Generated {} triangles", mesh.triangle_count());
    /// ```
    pub fn glyph_to_mesh_3d(
        &self,
        character: char,
        quality: Quality,
        depth: f32,
    ) -> Result<Mesh3D> {
        let glyph = self.glyph_by_char(character)?;
        let outline = glyph.linearize(quality)?;
        let mesh_2d = triangulate::triangulate(&outline)?;
        extrude::extrude(&mesh_2d, &outline, depth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests will be added when we have test fonts
    #[test]
    fn test_api_compiles() {
        // This test just verifies the API compiles
    }
}
