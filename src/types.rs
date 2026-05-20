//! Core type definitions for fontmesh

use glam::Vec2;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type Point2D = Vec2;

/// Kind of off-curve control point.
///
/// `Quad` is the TrueType quadratic case (one control point between two
/// on-curve points). `Cubic` is the CFF/PostScript cubic case (two control
/// points between two on-curve points); the linearizer relies on the
/// `Cubic` tag to interpret two consecutive off-curve points as a cubic
/// rather than as a TrueType on-the-fly midpoint pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveKind {
    Quad,
    Cubic,
}

/// A point in a contour with on-curve flag
#[derive(Debug, Clone, Copy)]
pub struct ContourPoint {
    pub point: Point2D,
    pub on_curve: bool,
    /// Curve degree this control point belongs to. Meaningful only when
    /// `on_curve == false`; ignored for on-curve points.
    pub curve_kind: CurveKind,
}

impl ContourPoint {
    pub fn new(point: Point2D, on_curve: bool) -> Self {
        Self {
            point,
            on_curve,
            curve_kind: CurveKind::Quad,
        }
    }

    pub fn on_curve(point: Point2D) -> Self {
        Self {
            point,
            on_curve: true,
            curve_kind: CurveKind::Quad,
        }
    }

    pub fn off_curve(point: Point2D) -> Self {
        Self {
            point,
            on_curve: false,
            curve_kind: CurveKind::Quad,
        }
    }

    pub fn off_curve_cubic(point: Point2D) -> Self {
        Self {
            point,
            on_curve: false,
            curve_kind: CurveKind::Cubic,
        }
    }
}

/// A single contour (closed or open path)
#[derive(Debug, Clone)]
pub struct Contour {
    pub points: Vec<ContourPoint>,
    pub closed: bool,
}

impl Contour {
    pub fn new(closed: bool) -> Self {
        Self {
            points: Vec::new(),
            closed,
        }
    }

    pub fn push(&mut self, point: ContourPoint) {
        self.points.push(point);
    }

    pub fn push_on_curve(&mut self, point: Point2D) {
        self.points.push(ContourPoint::on_curve(point));
    }

    pub fn push_off_curve(&mut self, point: Point2D) {
        self.points.push(ContourPoint::off_curve(point));
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

/// A collection of contours representing a glyph outline
#[derive(Debug, Clone)]
pub struct Outline2D {
    pub contours: Vec<Contour>,
}

impl Outline2D {
    pub fn new() -> Self {
        Self {
            contours: Vec::new(),
        }
    }

    pub fn add_contour(&mut self, contour: Contour) {
        self.contours.push(contour);
    }

    pub fn is_empty(&self) -> bool {
        self.contours.is_empty()
    }
}

impl Default for Outline2D {
    fn default() -> Self {
        Self::new()
    }
}

impl Outline2D {
    /// ```ignore
    /// use fontmesh::{parse_font, glyph_id, GlyphMeshBuilder};
    ///
    /// let font_data = include_bytes!("../assets/test_font.ttf");
    /// let font = parse_font(font_data)?;
    /// let gid = glyph_id(&font, 'A').unwrap();
    /// let outline = GlyphMeshBuilder::new(&font, gid).with_subdivisions(20).to_outline()?;
    /// let mesh = outline.triangulate()?;
    /// # Ok::<(), fontmesh::FontMeshError>(())
    /// ```
    #[inline]
    pub fn triangulate(&self) -> crate::error::Result<Mesh2D> {
        crate::triangulate::triangulate(self)
    }

    /// Convert this outline to a 3D mesh by triangulating and extruding.
    ///
    /// ```ignore
    /// use fontmesh::{parse_font, glyph_id, GlyphMeshBuilder};
    ///
    /// let font_data = include_bytes!("../assets/test_font.ttf");
    /// let font = parse_font(font_data)?;
    /// let gid = glyph_id(&font, 'A').unwrap();
    /// let outline = GlyphMeshBuilder::new(&font, gid).with_subdivisions(30).to_outline()?;
    /// let mesh = outline.to_mesh_3d(5.0)?;
    /// # Ok::<(), fontmesh::FontMeshError>(())
    /// ```
    #[inline]
    pub fn to_mesh_3d(&self, depth: f32) -> crate::error::Result<Mesh3D> {
        let mesh_2d = self.triangulate()?;
        crate::extrude::extrude(&mesh_2d, self, depth)
    }
}

/// A 2D triangle mesh
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Mesh2D {
    pub vertices: Vec<Point2D>,
    pub indices: Vec<u32>,
}

impl Mesh2D {
    #[must_use]
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Get the number of triangles in the mesh
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Check if the mesh is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Extrude this 2D mesh into a 3D mesh (fluent API)
    ///
    /// # Arguments
    /// * `outline` - The linearized outline (used for side geometry)
    /// * `depth` - The extrusion depth
    ///
    /// # Returns
    /// A 3D triangle mesh with normals
    ///
    /// ```ignore
    /// use fontmesh::{parse_font, glyph_id, GlyphMeshBuilder};
    ///
    /// let font_data = include_bytes!("../assets/test_font.ttf");
    /// let font = parse_font(font_data)?;
    /// let gid = glyph_id(&font, 'A').unwrap();
    /// let outline = GlyphMeshBuilder::new(&font, gid).with_subdivisions(30).to_outline()?;
    /// let mesh_2d = outline.triangulate()?;
    /// let mesh_3d = mesh_2d.extrude(&outline, 5.0)?;
    /// # Ok::<(), fontmesh::FontMeshError>(())
    /// ```
    #[inline]
    pub fn extrude(&self, outline: &Outline2D, depth: f32) -> crate::error::Result<Mesh3D> {
        crate::extrude::extrude(self, outline, depth)
    }
}

impl Default for Mesh2D {
    fn default() -> Self {
        Self::new()
    }
}

/// A 3D triangle mesh with normals
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Mesh3D {
    pub vertices: Vec<glam::Vec3>,
    pub normals: Vec<glam::Vec3>,
    pub indices: Vec<u32>,
}

impl Mesh3D {
    #[must_use]
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Get the number of triangles in the mesh
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Check if the mesh is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

impl Default for Mesh3D {
    fn default() -> Self {
        Self::new()
    }
}
