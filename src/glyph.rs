//! Glyph representation and outline extraction

use crate::error::{FontMeshError, Result};
use crate::types::{Contour, Outline2D, Point2D, Quality};
use glam::Vec2;
use ttf_parser::{Face, GlyphId, OutlineBuilder};

/// A glyph from a font
pub struct Glyph<'a> {
    /// The character this glyph represents
    pub character: char,
    /// The glyph ID in the font
    pub(crate) glyph_id: GlyphId,
    /// Reference to the font face
    pub(crate) face: &'a Face<'a>,
    /// Horizontal advance width (normalized to 1.0 em)
    pub advance: f32,
    /// Glyph bounds [[x_min, y_min], [x_max, y_max]] (normalized)
    pub bounds: Option<[[f32; 2]; 2]>,
}

impl<'a> Glyph<'a> {
    /// Extract the glyph's outline
    ///
    /// # Returns
    /// The 2D outline of the glyph, or an error if extraction fails
    pub fn outline(&self) -> Result<Outline2D> {
        let mut builder = OutlineExtractor::new(self.face.units_per_em());

        self.face
            .outline_glyph(self.glyph_id, &mut builder)
            .ok_or(FontMeshError::NoOutline)?;

        if builder.outline.is_empty() {
            return Err(FontMeshError::NoOutline);
        }

        Ok(builder.outline)
    }

    /// Linearize the glyph's outline by converting curves to line segments
    ///
    /// # Arguments
    /// * `quality` - The quality level for curve subdivision
    ///
    /// # Returns
    /// A linearized outline ready for triangulation
    pub fn linearize(&self, quality: Quality) -> Result<Outline2D> {
        let outline = self.outline()?;
        crate::linearize::linearize_outline(outline, quality)
    }
}

/// Outline builder that extracts glyph contours
struct OutlineExtractor {
    outline: Outline2D,
    current_contour: Option<Contour>,
    scale: f32,
}

impl OutlineExtractor {
    fn new(units_per_em: u16) -> Self {
        Self {
            outline: Outline2D::new(),
            current_contour: None,
            scale: 1.0 / units_per_em as f32,
        }
    }

    fn point(&self, x: f32, y: f32) -> Point2D {
        Vec2::new(x * self.scale, y * self.scale)
    }

    fn push_point(&mut self, point: Point2D) {
        if let Some(ref mut contour) = self.current_contour {
            contour.push(point);
        }
    }

    fn finish_contour(&mut self) {
        if let Some(contour) = self.current_contour.take() {
            if !contour.is_empty() {
                self.outline.add_contour(contour);
            }
        }
    }
}

impl OutlineBuilder for OutlineExtractor {
    fn move_to(&mut self, x: f32, y: f32) {
        // Finish previous contour if any
        self.finish_contour();

        // Start new contour
        let mut contour = Contour::new(true);
        contour.push(self.point(x, y));
        self.current_contour = Some(contour);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.push_point(self.point(x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        // Quadratic Bezier curve - we'll store the control point for now
        // Linearization will happen later
        self.push_point(self.point(x1, y1));
        self.push_point(self.point(x, y));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        // Cubic Bezier curve - store control points
        // Linearization will happen later
        self.push_point(self.point(x1, y1));
        self.push_point(self.point(x2, y2));
        self.push_point(self.point(x, y));
    }

    fn close(&mut self) {
        self.finish_contour();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outline_extraction() {
        // This test requires a font file - will be added when we add test fonts
    }
}
