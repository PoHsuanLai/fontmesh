//! Font parsing utilities
//!
//! This module provides helper functions for parsing font data.
//! The core mesh generation functions are now stateless and accept
//! `ttf_parser::Face` directly - see the `glyph` module for the main API.

use crate::error::{FontMeshError, Result};
use ttf_parser::Face;

/// Parse font data into a ttf-parser Face
///
/// This is a convenience wrapper around `ttf_parser::Face::parse`.
/// You can also use `Face::parse` directly if you prefer.
///
/// # Example
/// ```ignore
/// use fontmesh::parse_font;
///
/// let face = parse_font(font_data)?;
/// let mesh = fontmesh::char_to_mesh_3d(&face, 'A', 5.0, 20)?;
/// ```
pub fn parse_font(data: &[u8]) -> Result<Face<'_>> {
    Face::parse(data, 0)
        .map_err(|e| FontMeshError::ParseError(format!("Failed to parse font: {:?}", e)))
}

/// Get font metrics helpers
/// Get the font's ascender (normalized to 1.0 em)
pub fn ascender(face: &Face) -> f32 {
    face.ascender() as f32 / face.units_per_em() as f32
}

/// Get the font's descender (normalized to 1.0 em)
pub fn descender(face: &Face) -> f32 {
    face.descender() as f32 / face.units_per_em() as f32
}

/// Get the font's line gap (normalized to 1.0 em)
pub fn line_gap(face: &Face) -> f32 {
    face.line_gap() as f32 / face.units_per_em() as f32
}

/// Get glyph advance width for a character (normalized to 1.0 em)
///
/// Returns None if the glyph is not found in the font.
pub fn glyph_advance(face: &Face, character: char) -> Option<f32> {
    let glyph_id = face.glyph_index(character)?;
    let h_metrics = face.glyph_hor_advance(glyph_id)?;
    Some(h_metrics as f32 / face.units_per_em() as f32)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_font_loading() {
        // This test requires a font file - will be added when we add test fonts
        // For now, just verify the API compiles
    }
}
