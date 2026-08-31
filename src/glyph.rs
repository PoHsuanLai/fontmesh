//! Glyph outline extraction and tessellation.
//!
//! The API is glyph-id based. Use [`crate::glyph_id`] to resolve a `char`
//! to a [`GlyphId`], then call [`glyph_to_mesh_2d`] or [`glyph_to_mesh_3d`].
//! [`GlyphMeshBuilder`] is the same pipeline with a fluent interface.
//!
//! Coordinates are scaled to 1.0 em and keep the font's Y-up convention
//! (origin at the glyph baseline).
//!
//! ```
//! use fontmesh::{parse_font, glyph_id, glyph_to_mesh_2d};
//!
//! # let data = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"));
//! let font = parse_font(data)?;
//! let gid = glyph_id(&font, 'A').unwrap();
//! let mesh = glyph_to_mesh_2d(&font, gid, 20)?;
//! assert!(mesh.triangle_count() > 0);
//! # Ok::<(), fontmesh::FontMeshError>(())
//! ```

use crate::error::{FontMeshError, Result};
use crate::types::{Contour, ContourPoint, Mesh2D, Mesh3D, Outline2D, Point2D};
use glam::Vec2;
use skrifa::{
    instance::{LocationRef, Size},
    outline::{DrawSettings, OutlinePen},
    FontRef, GlyphId, MetadataProvider,
};

/// Default quality for curve linearization (20 subdivisions per curve)
const DEFAULT_QUALITY: u8 = 20;

/// Convert a glyph id to a 2D triangle mesh using a parsed font.
///
/// `subdivisions` is the curve-sampling quality (see the crate-level docs).
/// Must be at least 1.
///
/// # Errors
///
/// - [`FontMeshError::InvalidQuality`] if `subdivisions == 0`
/// - [`FontMeshError::OutlineExtractionFailed`] if the glyph has no outline table entry
/// - [`FontMeshError::NoOutline`] if the glyph exists but has no contours (e.g. space)
/// - [`FontMeshError::TriangulationFailed`] if tessellation fails
///
/// ```
/// # let font = fontmesh::parse_font(include_bytes!(concat!(
/// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
/// let gid = fontmesh::glyph_id(&font, 'B').unwrap();
/// let mesh = fontmesh::glyph_to_mesh_2d(&font, gid, 20).unwrap();
/// assert!(!mesh.vertices.is_empty());
/// ```
pub fn glyph_to_mesh_2d(font: &FontRef, glyph_id: GlyphId, subdivisions: u8) -> Result<Mesh2D> {
    if subdivisions == 0 {
        return Err(FontMeshError::InvalidQuality(subdivisions));
    }
    let outline = extract_and_linearize_outline(font, glyph_id, subdivisions)?;
    crate::triangulate::triangulate(&outline)
}

/// Convert a glyph id to a 3D triangle mesh with extrusion.
///
/// `depth` is the extrusion thickness in em units, centred on z = 0
/// (front at `+depth/2`, back at `-depth/2`). `subdivisions` is the
/// curve-sampling quality.
///
/// # Errors
///
/// Same as [`glyph_to_mesh_2d`], plus [`FontMeshError::ExtrusionFailed`]
/// if `depth` is not finite.
///
/// ```
/// # let font = fontmesh::parse_font(include_bytes!(concat!(
/// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
/// let gid = fontmesh::glyph_id(&font, 'A').unwrap();
/// let mesh = fontmesh::glyph_to_mesh_3d(&font, gid, 0.1, 20).unwrap();
/// assert_eq!(mesh.vertices.len(), mesh.normals.len());
/// ```
pub fn glyph_to_mesh_3d(
    font: &FontRef,
    glyph_id: GlyphId,
    depth: f32,
    subdivisions: u8,
) -> Result<Mesh3D> {
    if subdivisions == 0 {
        return Err(FontMeshError::InvalidQuality(subdivisions));
    }
    if !depth.is_finite() {
        return Err(FontMeshError::ExtrusionFailed(
            "depth must be a finite value".to_string(),
        ));
    }
    let outline = extract_and_linearize_outline(font, glyph_id, subdivisions)?;
    let mesh_2d = crate::triangulate::triangulate(&outline)?;
    crate::extrude::extrude(&mesh_2d, &outline, depth)
}

fn extract_and_linearize_outline(
    font: &FontRef,
    glyph_id: GlyphId,
    subdivisions: u8,
) -> Result<Outline2D> {
    let outline = extract_outline(font, glyph_id)?;
    crate::linearize::linearize_outline(outline, subdivisions)
}

fn extract_outline(font: &FontRef, glyph_id: GlyphId) -> Result<Outline2D> {
    let outlines = font.outline_glyphs();
    let glyph = outlines
        .get(glyph_id)
        .ok_or(FontMeshError::OutlineExtractionFailed(format!(
            "glyph {} has no outline in this font",
            glyph_id.to_u32()
        )))?;

    let units_per_em = font
        .metrics(Size::unscaled(), LocationRef::default())
        .units_per_em;
    let mut pen = OutlineExtractor::new(units_per_em);
    glyph
        .draw(
            DrawSettings::unhinted(Size::unscaled(), LocationRef::default()),
            &mut pen,
        )
        .map_err(|e| {
            FontMeshError::OutlineExtractionFailed(format!("skrifa draw failed: {e:?}"))
        })?;

    pen.finish_contour();

    if pen.outline.is_empty() {
        return Err(FontMeshError::NoOutline);
    }
    Ok(pen.outline)
}

/// Builder-style mesh generation for a glyph with configurable subdivisions.
///
/// Defaults to 20 subdivisions per curve. Chain [`Self::with_subdivisions`]
/// then finish with [`Self::to_mesh_2d`], [`Self::to_mesh_3d`], or
/// [`Self::to_outline`].
///
/// ```
/// # let font = fontmesh::parse_font(include_bytes!(concat!(
/// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
/// let gid = fontmesh::glyph_id(&font, 'A').unwrap();
/// let mesh = fontmesh::GlyphMeshBuilder::new(&font, gid)
///     .with_subdivisions(30)
///     .to_mesh_2d()
///     .unwrap();
/// assert!(!mesh.is_empty());
/// ```
pub struct GlyphMeshBuilder<'a> {
    font: &'a FontRef<'a>,
    glyph_id: GlyphId,
    subdivisions: u8,
}

impl<'a> GlyphMeshBuilder<'a> {
    /// Start a builder for `glyph_id` in `font`, with 20 subdivisions.
    ///
    /// ```
    /// # let font = fontmesh::parse_font(include_bytes!(concat!(
    /// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
    /// let gid = fontmesh::glyph_id(&font, 'A').unwrap();
    /// let builder = fontmesh::GlyphMeshBuilder::new(&font, gid);
    /// let mesh = builder.to_mesh_2d().unwrap();
    /// assert!(mesh.triangle_count() > 0);
    /// ```
    pub fn new(font: &'a FontRef<'a>, glyph_id: GlyphId) -> Self {
        Self {
            font,
            glyph_id,
            subdivisions: DEFAULT_QUALITY,
        }
    }

    /// Set the curve-sampling quality used by subsequent `to_*` calls.
    ///
    /// ```
    /// # let font = fontmesh::parse_font(include_bytes!(concat!(
    /// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
    /// let gid = fontmesh::glyph_id(&font, 'S').unwrap();
    /// let low = fontmesh::GlyphMeshBuilder::new(&font, gid)
    ///     .with_subdivisions(8)
    ///     .to_mesh_2d()
    ///     .unwrap();
    /// let high = fontmesh::GlyphMeshBuilder::new(&font, gid)
    ///     .with_subdivisions(40)
    ///     .to_mesh_2d()
    ///     .unwrap();
    /// assert!(low.vertices.len() <= high.vertices.len());
    /// ```
    #[must_use = "builder methods are intended to be chained"]
    pub fn with_subdivisions(mut self, subdivisions: u8) -> Self {
        self.subdivisions = subdivisions;
        self
    }

    /// Extract and linearize the glyph outline without triangulating.
    ///
    /// Useful when you want to inspect contours or feed a custom triangulator.
    ///
    /// ```
    /// # let font = fontmesh::parse_font(include_bytes!(concat!(
    /// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
    /// let gid = fontmesh::glyph_id(&font, 'A').unwrap();
    /// let outline = fontmesh::GlyphMeshBuilder::new(&font, gid)
    ///     .to_outline()
    ///     .unwrap();
    /// assert!(!outline.is_empty());
    /// ```
    pub fn to_outline(self) -> Result<Outline2D> {
        extract_and_linearize_outline(self.font, self.glyph_id, self.subdivisions)
    }

    /// Tessellate the glyph into a 2D triangle mesh.
    ///
    /// ```
    /// # let font = fontmesh::parse_font(include_bytes!(concat!(
    /// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
    /// let gid = fontmesh::glyph_id(&font, 'A').unwrap();
    /// let mesh = fontmesh::GlyphMeshBuilder::new(&font, gid)
    ///     .to_mesh_2d()
    ///     .unwrap();
    /// assert!(mesh.indices.len().is_multiple_of(3));
    /// ```
    pub fn to_mesh_2d(self) -> Result<Mesh2D> {
        glyph_to_mesh_2d(self.font, self.glyph_id, self.subdivisions)
    }

    /// Tessellate and extrude the glyph into a 3D mesh of thickness `depth`.
    ///
    /// ```
    /// # let font = fontmesh::parse_font(include_bytes!(concat!(
    /// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
    /// let gid = fontmesh::glyph_id(&font, 'A').unwrap();
    /// let mesh = fontmesh::GlyphMeshBuilder::new(&font, gid)
    ///     .to_mesh_3d(0.2)
    ///     .unwrap();
    /// assert_eq!(mesh.vertices.len(), mesh.normals.len());
    /// ```
    pub fn to_mesh_3d(self, depth: f32) -> Result<Mesh3D> {
        glyph_to_mesh_3d(self.font, self.glyph_id, depth, self.subdivisions)
    }
}

/// Outline builder that translates skrifa pen events into our [`Outline2D`]
/// representation.
///
/// Font space is Y-up with origin at the baseline; the output mesh keeps that
/// convention and only scales coordinates to 1.0 em.
///
/// Skrifa emits both quadratic (TrueType) and cubic (CFF/PostScript) curves.
/// We represent both in [`Contour`] as on-curve / off-curve points; the
/// quadratic path uses one off-curve control, the cubic path uses two
/// adjacent off-curves and is then handled by [`crate::linearize`].
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
            scale: 1.0 / units_per_em.max(1) as f32,
        }
    }

    #[inline(always)]
    fn point(&self, x: f32, y: f32) -> Point2D {
        Vec2::new(x * self.scale, y * self.scale)
    }

    #[inline(always)]
    fn push(&mut self, point: ContourPoint) {
        if let Some(c) = self.current_contour.as_mut() {
            c.push(point);
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

impl OutlinePen for OutlineExtractor {
    fn move_to(&mut self, x: f32, y: f32) {
        self.finish_contour();
        let mut contour = Contour::new(true);
        contour.push(ContourPoint::on_curve(self.point(x, y)));
        self.current_contour = Some(contour);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.push(ContourPoint::on_curve(self.point(x, y)));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.push(ContourPoint::off_curve(self.point(cx0, cy0)));
        self.push(ContourPoint::on_curve(self.point(x, y)));
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.push(ContourPoint::off_curve_cubic(self.point(cx0, cy0)));
        self.push(ContourPoint::off_curve_cubic(self.point(cx1, cy1)));
        self.push(ContourPoint::on_curve(self.point(x, y)));
    }

    fn close(&mut self) {
        self.finish_contour();
    }
}
