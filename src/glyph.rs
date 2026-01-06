//! Glyph representation and outline extraction

use crate::error::{FontMeshError, Result};
use crate::types::{Contour, ContourPoint, Mesh2D, Mesh3D, Outline2D, Point2D};
use glam::Vec2;
use ttf_parser::{Face, GlyphId, OutlineBuilder};

/// Default quality for curve linearization (20 subdivisions per curve)
const DEFAULT_QUALITY: u8 = 20;

// ============================================================================
// Pure Functions API - Stateless core functions
// ============================================================================

/// Convert a character to a 2D triangle mesh using a parsed font face
///
/// This is a pure function that takes a parsed `Face` and generates a mesh.
/// This is the most flexible API - you control when to parse and cache the Face.
///
/// # Arguments
/// * `face` - A parsed ttf-parser Face
/// * `character` - The character to convert
/// * `subdivisions` - Number of subdivisions per curve (higher = smoother, default 20)
///
/// # Example
/// ```ignore
/// use ttf_parser::Face;
/// use fontmesh::char_to_mesh_2d;
///
/// let face = Face::parse(font_data, 0)?;
/// let mesh = char_to_mesh_2d(&face, 'A', 20)?;
/// ```
pub fn char_to_mesh_2d(face: &Face, character: char, subdivisions: u8) -> Result<Mesh2D> {
    let outline = extract_and_linearize_outline(face, character, subdivisions)?;
    crate::triangulate::triangulate(&outline)
}

/// Convert a character to a 3D triangle mesh with extrusion using a parsed font face
///
/// This is a pure function that takes a parsed `Face` and generates a 3D mesh.
/// This is the most flexible API - you control when to parse and cache the Face.
///
/// # Arguments
/// * `face` - A parsed ttf-parser Face
/// * `character` - The character to convert
/// * `depth` - The extrusion depth
/// * `subdivisions` - Number of subdivisions per curve (higher = smoother, default 20)
///
/// # Example
/// ```ignore
/// use ttf_parser::Face;
/// use fontmesh::char_to_mesh_3d;
///
/// let face = Face::parse(font_data, 0)?;
/// let mesh = char_to_mesh_3d(&face, 'A', 5.0, 20)?;
/// ```
pub fn char_to_mesh_3d(
    face: &Face,
    character: char,
    depth: f32,
    subdivisions: u8,
) -> Result<Mesh3D> {
    let outline = extract_and_linearize_outline(face, character, subdivisions)?;
    let mesh_2d = crate::triangulate::triangulate(&outline)?;
    crate::extrude::extrude(&mesh_2d, &outline, depth)
}

/// Extract and linearize a glyph outline from a parsed face
///
/// This is a helper function used by the other pure functions.
/// You can use this directly if you want to work with outlines.
fn extract_and_linearize_outline(
    face: &Face,
    character: char,
    subdivisions: u8,
) -> Result<Outline2D> {
    let glyph_id = face
        .glyph_index(character)
        .ok_or(FontMeshError::GlyphNotFound(character))?;

    let mut builder = OutlineExtractor::new(face.units_per_em());
    face.outline_glyph(glyph_id, &mut builder)
        .ok_or(FontMeshError::NoOutline)?;

    if builder.outline.is_empty() {
        return Err(FontMeshError::NoOutline);
    }

    crate::linearize::linearize_outline(builder.outline, subdivisions)
}

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

    /// Convert to a linearized outline
    pub fn to_outline(self) -> Result<crate::types::Outline2D> {
        self.glyph.linearize_with(self.subdivisions)
    }

    /// Convert to a 2D triangle mesh
    pub fn to_mesh_2d(self) -> Result<crate::types::Mesh2D> {
        let outline = self.glyph.linearize_with(self.subdivisions)?;
        crate::triangulate::triangulate(&outline)
    }

    /// Convert to a 3D triangle mesh with extrusion
    pub fn to_mesh_3d(self, depth: f32) -> Result<crate::types::Mesh3D> {
        let outline = self.glyph.linearize_with(self.subdivisions)?;
        let mesh_2d = crate::triangulate::triangulate(&outline)?;
        crate::extrude::extrude(&mesh_2d, &outline, depth)
    }
}

impl<'a> Glyph<'a> {
    /// Create a new Glyph wrapper from a Face and a character
    pub fn new(face: &'a Face<'a>, character: char) -> Result<Self> {
        let glyph_id = face
            .glyph_index(character)
            .ok_or(FontMeshError::GlyphNotFound(character))?;

        let advance = face
            .glyph_hor_advance(glyph_id)
            .map(|adv| adv as f32 / face.units_per_em() as f32)
            .unwrap_or(0.0);

        let bounds = face.glyph_bounding_box(glyph_id).map(|bb| {
            let scale = 1.0 / face.units_per_em() as f32;
            [
                [bb.x_min as f32 * scale, bb.y_min as f32 * scale],
                [bb.x_max as f32 * scale, bb.y_max as f32 * scale],
            ]
        });

        Ok(Self {
            character,
            glyph_id,
            face,
            advance,
            bounds,
        })
    }

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
