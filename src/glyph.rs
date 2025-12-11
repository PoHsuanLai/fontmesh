//! Glyph representation and outline extraction

use crate::error::{FontMeshError, Result};
use crate::types::{Contour, ContourPoint, Outline2D, Point2D};
use glam::Vec2;
use ttf_parser::{Face, GlyphId, OutlineBuilder};

/// Default quality for curve linearization (20 subdivisions per curve)
const DEFAULT_QUALITY: u8 = 20;

/// A glyph from a font
pub struct Glyph<'a> {
    /// The character this glyph represents
    pub(crate) character: char,
    /// The glyph ID in the font
    pub(crate) glyph_id: GlyphId,
    /// Reference to the font face
    pub(crate) face: &'a Face<'a>,
    /// Horizontal advance width (normalized to 1.0 em)
    pub(crate) advance: f32,
    /// Glyph bounds [[x_min, y_min], [x_max, y_max]] (normalized)
    pub(crate) bounds: Option<[[f32; 2]; 2]>,
}

/// Builder for creating meshes from a glyph with configurable subdivisions
pub struct GlyphMeshBuilder<'a> {
    glyph: &'a Glyph<'a>,
    subdivisions: u8,
}

impl<'a> GlyphMeshBuilder<'a> {
    /// Set the number of subdivisions per curve
    ///
    /// Higher values produce smoother curves but more vertices.
    /// Default is 20 subdivisions per curve.
    ///
    /// # Example
    /// ```ignore
    /// let mesh = font.glyph_by_char('A')?
    ///     .with_subdivisions(50)
    ///     .to_mesh_2d()?;
    /// ```
    #[must_use = "builder methods are intended to be chained"]
    pub fn with_subdivisions(mut self, subdivisions: u8) -> Self {
        self.subdivisions = subdivisions;
        self
    }

    /// Convert to a 2D triangle mesh
    #[must_use]
    pub fn to_mesh_2d(self) -> Result<crate::types::Mesh2D> {
        let outline = self.glyph.linearize_with(self.subdivisions)?;
        crate::triangulate::triangulate(&outline)
    }

    /// Convert to a 3D triangle mesh with extrusion
    #[must_use]
    pub fn to_mesh_3d(self, depth: f32) -> Result<crate::types::Mesh3D> {
        let outline = self.glyph.linearize_with(self.subdivisions)?;
        let mesh_2d = crate::triangulate::triangulate(&outline)?;
        crate::extrude::extrude(&mesh_2d, &outline, depth)
    }
}

impl<'a> Glyph<'a> {
    /// Get the character this glyph represents
    ///
    /// # Example
    /// ```ignore
    /// let glyph = font.glyph_by_char('A')?;
    /// assert_eq!(glyph.character(), 'A');
    /// ```
    #[inline]
    pub fn character(&self) -> char {
        self.character
    }

    /// Get the glyph ID
    ///
    /// This is useful for caching, building lookup tables, or integrating
    /// with text shaping libraries that work with glyph IDs.
    ///
    /// # Example
    /// ```ignore
    /// let glyph = font.glyph_by_char('A')?;
    /// let id = glyph.glyph_id();
    /// // Store in cache: cache.insert(id, mesh);
    /// ```
    #[inline]
    pub fn glyph_id(&self) -> GlyphId {
        self.glyph_id
    }

    /// Get the horizontal advance width (normalized to 1.0 em)
    ///
    /// This value represents how far to advance horizontally after rendering
    /// this glyph. It's normalized to 1.0 = 1 em (typically the font size).
    ///
    /// # Example
    /// ```ignore
    /// let glyph = font.glyph_by_char('A')?;
    /// let width = glyph.advance();
    /// ```
    #[inline]
    pub fn advance(&self) -> f32 {
        self.advance
    }

    /// Get the glyph bounds (normalized to 1.0 em)
    ///
    /// Returns `[[x_min, y_min], [x_max, y_max]]` if the glyph has an outline,
    /// or `None` for whitespace characters.
    ///
    /// # Example
    /// ```ignore
    /// let glyph = font.glyph_by_char('A')?;
    /// if let Some([[x_min, y_min], [x_max, y_max]]) = glyph.bounds() {
    ///     println!("Glyph size: {}x{}", x_max - x_min, y_max - y_min);
    /// }
    /// ```
    #[inline]
    pub fn bounds(&self) -> Option<[[f32; 2]; 2]> {
        self.bounds
    }

    /// Set the number of subdivisions per curve for mesh generation (builder pattern)
    ///
    /// Higher values produce smoother curves but more vertices.
    /// Default is 20 subdivisions per curve.
    ///
    /// # Example
    /// ```ignore
    /// let mesh = font.glyph_by_char('A')?
    ///     .with_subdivisions(50)
    ///     .to_mesh_2d()?;
    /// ```
    #[must_use = "builder methods are intended to be chained"]
    pub fn with_subdivisions(&self, subdivisions: u8) -> GlyphMeshBuilder<'_> {
        GlyphMeshBuilder {
            glyph: self,
            subdivisions,
        }
    }

    /// Extract the glyph's outline
    ///
    /// # Returns
    /// The 2D outline of the glyph, or an error if extraction fails
    #[inline]
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
    /// Uses default quality (20 subdivisions per curve).
    ///
    /// # Returns
    /// A linearized outline ready for triangulation
    #[inline]
    pub fn linearize(&self) -> Result<Outline2D> {
        self.linearize_with(DEFAULT_QUALITY)
    }

    /// Linearize the glyph's outline with custom number of subdivisions
    ///
    /// # Arguments
    /// * `subdivisions` - Number of subdivisions per curve
    ///
    /// # Returns
    /// A linearized outline ready for triangulation
    #[inline]
    pub fn linearize_with(&self, subdivisions: u8) -> Result<Outline2D> {
        let outline = self.outline()?;
        crate::linearize::linearize_outline(outline, subdivisions)
    }

    /// Convert this glyph to a 2D triangle mesh
    ///
    /// Uses default quality (20 subdivisions per curve).
    ///
    /// # Example
    /// ```ignore
    /// let mesh = font.glyph_by_char('A')?.to_mesh_2d()?;
    /// ```
    #[inline]
    pub fn to_mesh_2d(&self) -> Result<crate::types::Mesh2D> {
        let outline = self.linearize()?;
        crate::triangulate::triangulate(&outline)
    }

    /// Convert this glyph to a 3D triangle mesh with extrusion
    ///
    /// Uses default quality (20 subdivisions per curve).
    ///
    /// # Arguments
    /// * `depth` - The extrusion depth
    ///
    /// # Example
    /// ```ignore
    /// let mesh = font.glyph_by_char('A')?.to_mesh_3d(5.0)?;
    /// ```
    #[inline]
    pub fn to_mesh_3d(&self, depth: f32) -> Result<crate::types::Mesh3D> {
        let outline = self.linearize()?;
        let mesh_2d = crate::triangulate::triangulate(&outline)?;
        crate::extrude::extrude(&mesh_2d, &outline, depth)
    }
}

/// Outline builder that extracts glyph contours
struct OutlineExtractor {
    outline: Outline2D,
    current_contour: Option<Contour>,
    scale: f32,
    last_point: Option<Point2D>,
}

impl OutlineExtractor {
    #[inline]
    fn new(units_per_em: u16) -> Self {
        Self {
            outline: Outline2D::new(),
            current_contour: None,
            scale: 1.0 / units_per_em as f32,
            last_point: None,
        }
    }

    #[inline(always)]
    fn point(&self, x: f32, y: f32) -> Point2D {
        Vec2::new(x * self.scale, y * self.scale)
    }

    #[inline(always)]
    fn push_point(&mut self, point: ContourPoint) {
        if let Some(ref mut contour) = self.current_contour {
            contour.push(point);
            self.last_point = Some(point.point);
        }
    }

    #[inline]
    fn finish_contour(&mut self) {
        if let Some(contour) = self.current_contour.take() {
            if !contour.is_empty() {
                self.outline.add_contour(contour);
            }
        }
        self.last_point = None;
    }
}

impl OutlineBuilder for OutlineExtractor {
    #[inline]
    fn move_to(&mut self, x: f32, y: f32) {
        // Finish previous contour if any
        self.finish_contour();

        // Start new contour
        let mut contour = Contour::new(true);
        let pt = self.point(x, y);
        contour.push(ContourPoint::on_curve(pt));
        self.last_point = Some(pt);
        self.current_contour = Some(contour);
    }

    #[inline]
    fn line_to(&mut self, x: f32, y: f32) {
        let pt = self.point(x, y);
        self.push_point(ContourPoint::on_curve(pt));
    }

    #[inline]
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        // Quadratic Bezier: control point (off-curve) + end point (on-curve)
        self.push_point(ContourPoint::off_curve(self.point(x1, y1)));
        self.push_point(ContourPoint::on_curve(self.point(x, y)));
    }

    #[inline]
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        // Cubic Bezier: two control points (off-curve) + end point (on-curve)
        self.push_point(ContourPoint::off_curve(self.point(x1, y1)));
        self.push_point(ContourPoint::off_curve(self.point(x2, y2)));
        self.push_point(ContourPoint::on_curve(self.point(x, y)));
    }

    #[inline]
    fn close(&mut self) {
        self.finish_contour();
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_outline_extraction() {
        // This test requires a font file - will be added when we add test fonts
    }
}
