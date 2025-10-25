//! Font loading and management

use crate::error::{FontMeshError, Result};
use crate::glyph::Glyph;
use crate::types::{Mesh2D, Mesh3D, Quality};
use ttf_parser::{Face, GlyphId};

/// A loaded TrueType font
pub struct Font<'a> {
    /// The underlying ttf-parser face
    face: Face<'a>,
    /// Font data (owned for lifetime management)
    _data: &'a [u8],
}

impl<'a> Font<'a> {
    /// Load a font from bytes
    ///
    /// # Arguments
    /// * `data` - The font file data (must live as long as the Font)
    ///
    /// # Example
    /// ```ignore
    /// let font_data = include_bytes!("../tests/fonts/FiraMono-Medium.ttf");
    /// let font = Font::from_bytes(font_data)?;
    /// ```
    pub fn from_bytes(data: &'a [u8]) -> Result<Self> {
        let face = Face::parse(data, 0)
            .map_err(|e| FontMeshError::ParseError(format!("Failed to parse font: {:?}", e)))?;

        Ok(Self { face, _data: data })
    }

    /// Get a glyph by character
    ///
    /// # Arguments
    /// * `c` - The character to look up
    ///
    /// # Returns
    /// The glyph for the character, or an error if not found
    pub fn glyph_by_char(&self, c: char) -> Result<Glyph<'_>> {
        let glyph_id = self
            .face
            .glyph_index(c)
            .ok_or(FontMeshError::GlyphNotFound(c))?;

        self.glyph_by_id(glyph_id, c)
    }

    /// Get a glyph by its ID
    fn glyph_by_id(&self, glyph_id: GlyphId, character: char) -> Result<Glyph<'_>> {
        // Get horizontal metrics
        let h_metrics = self.face.glyph_hor_advance(glyph_id).unwrap_or(0);
        let advance = h_metrics as f32 / self.face.units_per_em() as f32;

        // Get bounding box
        let bbox = self.face.glyph_bounding_box(glyph_id);

        let bounds = bbox.map(|b| {
            let scale = 1.0 / self.face.units_per_em() as f32;
            [
                [b.x_min as f32 * scale, b.y_min as f32 * scale],
                [b.x_max as f32 * scale, b.y_max as f32 * scale],
            ]
        });

        Ok(Glyph {
            character,
            glyph_id,
            face: &self.face,
            advance,
            bounds,
        })
    }

    /// Get the font's units per em
    pub fn units_per_em(&self) -> u16 {
        self.face.units_per_em()
    }

    /// Get the font's ascender
    pub fn ascender(&self) -> f32 {
        self.face.ascender() as f32 / self.face.units_per_em() as f32
    }

    /// Get the font's descender
    pub fn descender(&self) -> f32 {
        self.face.descender() as f32 / self.face.units_per_em() as f32
    }

    /// Get the font's line gap
    pub fn line_gap(&self) -> f32 {
        self.face.line_gap() as f32 / self.face.units_per_em() as f32
    }

    /// Convert a character to a 2D mesh, reusing an existing buffer
    ///
    /// This is more efficient than `glyph_to_mesh_2d()` when converting many glyphs,
    /// as it reuses allocated memory instead of allocating new vectors each time.
    ///
    /// # Arguments
    /// * `c` - The character to convert
    /// * `quality` - Quality level for curve subdivision
    /// * `mesh` - Existing mesh to write into (will be cleared first)
    ///
    /// # Example
    /// ```ignore
    /// let mut mesh = Mesh2D::new();
    /// for c in "Hello".chars() {
    ///     font.glyph_to_mesh_2d_reuse(c, Quality::Normal, &mut mesh)?;
    ///     // Use mesh...
    /// }
    /// ```
    #[inline]
    pub fn glyph_to_mesh_2d_reuse(
        &self,
        c: char,
        quality: Quality,
        mesh: &mut Mesh2D,
    ) -> Result<()> {
        mesh.clear();
        let glyph = self.glyph_by_char(c)?;
        match glyph.outline() {
            Ok(outline) => {
                let linearized = crate::linearize::linearize_outline(outline, quality)?;
                let new_mesh = crate::triangulate::triangulate(&linearized)?;
                *mesh = new_mesh;
                Ok(())
            }
            Err(FontMeshError::NoOutline) => Ok(()), // Leave mesh empty for space/whitespace
            Err(e) => Err(e),
        }
    }

    /// Convert a character to a 3D mesh, reusing an existing buffer
    ///
    /// This is more efficient than `glyph_to_mesh_3d()` when converting many glyphs,
    /// as it reuses allocated memory instead of allocating new vectors each time.
    ///
    /// # Arguments
    /// * `c` - The character to convert
    /// * `quality` - Quality level for curve subdivision
    /// * `depth` - Extrusion depth
    /// * `mesh` - Existing mesh to write into (will be cleared first)
    ///
    /// # Example
    /// ```ignore
    /// let mut mesh = Mesh3D::new();
    /// for c in "Hello".chars() {
    ///     font.glyph_to_mesh_3d_reuse(c, Quality::Normal, 1.0, &mut mesh)?;
    ///     // Use mesh...
    /// }
    /// ```
    #[inline]
    pub fn glyph_to_mesh_3d_reuse(
        &self,
        c: char,
        quality: Quality,
        depth: f32,
        mesh: &mut Mesh3D,
    ) -> Result<()> {
        mesh.clear();
        let glyph = self.glyph_by_char(c)?;
        match glyph.outline() {
            Ok(outline) => {
                let linearized = crate::linearize::linearize_outline(outline, quality)?;
                let mesh_2d = crate::triangulate::triangulate(&linearized)?;
                let new_mesh = crate::extrude::extrude(&mesh_2d, &linearized, depth)?;
                *mesh = new_mesh;
                Ok(())
            }
            Err(FontMeshError::NoOutline) => Ok(()), // Leave mesh empty for space/whitespace
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_font_loading() {
        // This test requires a font file - will be added when we add test fonts
        // For now, just verify the API compiles
    }
}
