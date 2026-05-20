//! Glyph outline extraction and tessellation.
//!
//! The API is glyph-id based. Use [`crate::glyph_id`] to resolve a `char`
//! to a [`GlyphId`], then call [`glyph_to_mesh_2d`] or [`glyph_to_mesh_3d`].

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
pub fn glyph_to_mesh_2d(font: &FontRef, glyph_id: GlyphId, subdivisions: u8) -> Result<Mesh2D> {
    if subdivisions == 0 {
        return Err(FontMeshError::InvalidQuality(subdivisions));
    }
    let outline = extract_and_linearize_outline(font, glyph_id, subdivisions)?;
    crate::triangulate::triangulate(&outline)
}

/// Convert a glyph id to a 3D triangle mesh with extrusion.
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
pub struct GlyphMeshBuilder<'a> {
    font: &'a FontRef<'a>,
    glyph_id: GlyphId,
    subdivisions: u8,
}

impl<'a> GlyphMeshBuilder<'a> {
    pub fn new(font: &'a FontRef<'a>, glyph_id: GlyphId) -> Self {
        Self {
            font,
            glyph_id,
            subdivisions: DEFAULT_QUALITY,
        }
    }

    #[must_use = "builder methods are intended to be chained"]
    pub fn with_subdivisions(mut self, subdivisions: u8) -> Self {
        self.subdivisions = subdivisions;
        self
    }

    pub fn to_outline(self) -> Result<Outline2D> {
        extract_and_linearize_outline(self.font, self.glyph_id, self.subdivisions)
    }

    pub fn to_mesh_2d(self) -> Result<Mesh2D> {
        glyph_to_mesh_2d(self.font, self.glyph_id, self.subdivisions)
    }

    pub fn to_mesh_3d(self, depth: f32) -> Result<Mesh3D> {
        glyph_to_mesh_3d(self.font, self.glyph_id, depth, self.subdivisions)
    }
}

/// Outline builder that translates skrifa pen events into our [`Outline2D`]
/// representation. Y coordinates are flipped because font coordinate space
/// has Y up at the baseline, while we want the same.
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
