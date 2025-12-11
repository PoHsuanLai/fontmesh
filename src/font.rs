//! Font loading and management

use crate::error::{FontMeshError, Result};
use crate::glyph::Glyph;
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
    ///
    /// # Example
    /// ```ignore
    /// let glyph = font.glyph_by_char('A')?;
    /// println!("Advance: {}", glyph.advance);
    /// ```
    pub fn glyph_by_char(&self, c: char) -> Result<Glyph<'_>> {
        let glyph_id = self
            .face
            .glyph_index(c)
            .ok_or(FontMeshError::GlyphNotFound(c))?;

        Ok(self.glyph_by_id(glyph_id, c))
    }

    /// Get a glyph by its glyph ID
    ///
    /// This is useful when working with text shaping libraries or when you
    /// already have glyph IDs from other sources (e.g., `rustybuzz`, `cosmic-text`).
    ///
    /// # Arguments
    /// * `glyph_id` - The glyph ID to look up
    /// * `character` - The character this glyph represents (for display purposes)
    ///
    /// # Returns
    /// A glyph with metrics and outline information
    ///
    /// # Example
    /// ```ignore
    /// use ttf_parser::GlyphId;
    ///
    /// let glyph_id = GlyphId(42);
    /// let glyph = font.glyph_by_id(glyph_id, 'A');
    /// ```
    pub fn glyph_by_id(&self, glyph_id: GlyphId, character: char) -> Glyph<'_> {
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

        Glyph {
            character,
            glyph_id,
            face: &self.face,
            advance,
            bounds,
        }
    }

    /// Access the underlying ttf-parser Face for advanced operations
    ///
    /// This allows you to use any ttf-parser functionality directly,
    /// such as accessing font tables, kerning information, or other
    /// font metadata that fontmesh doesn't expose.
    ///
    /// # Example
    /// ```ignore
    /// // Get kerning between two glyphs
    /// let face = font.face();
    /// let kern = face.kerning_subtables();
    ///
    /// // Access font name
    /// for name in face.names() {
    ///     println!("{:?}: {}", name.name_id, name.to_string());
    /// }
    /// ```
    pub fn face(&self) -> &Face<'a> {
        &self.face
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
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_font_loading() {
        // This test requires a font file - will be added when we add test fonts
        // For now, just verify the API compiles
    }
}
