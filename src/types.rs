//! Mesh, outline, and contour types used throughout fontmesh.
//!
//! Most callers only need [`Mesh2D`], [`Mesh3D`], and [`Outline2D`] (re-exported
//! at the crate root). The contour types are useful when you build or inspect
//! outlines yourself before passing them to [`Outline2D::triangulate`].
//!
//! ```
//! use fontmesh::types::{Contour, Outline2D, Point2D};
//!
//! let mut contour = Contour::new(true);
//! contour.push_on_curve(Point2D::new(0.0, 0.0));
//! contour.push_on_curve(Point2D::new(1.0, 0.0));
//! contour.push_on_curve(Point2D::new(0.0, 1.0));
//! let mut outline = Outline2D::new();
//! outline.add_contour(contour);
//! let mesh = outline.triangulate().unwrap();
//! assert!(mesh.triangle_count() >= 1);
//! ```

use glam::Vec2;
use std::fmt::Write as _;
use std::io::{self, Write};

/// A 2D point in em-normalised coordinates.
///
/// Alias of [`glam::Vec2`]. `(0, 0)` is the glyph origin; +X is right, +Y is up.
///
/// ```
/// use fontmesh::types::Point2D;
/// let p = Point2D::new(0.5, 0.25);
/// assert_eq!(p.x, 0.5);
/// ```
pub type Point2D = Vec2;

/// Kind of off-curve control point.
///
/// `Quad` is the TrueType quadratic case (one control point between two
/// on-curve points). `Cubic` is the CFF/PostScript cubic case (two control
/// points between two on-curve points); the linearizer relies on the
/// `Cubic` tag to interpret two consecutive off-curve points as a cubic
/// rather than as a TrueType on-the-fly midpoint pair.
///
/// ```
/// use fontmesh::types::{ContourPoint, CurveKind, Point2D};
/// let ctrl = ContourPoint::off_curve_cubic(Point2D::new(0.5, 1.0));
/// assert_eq!(ctrl.curve_kind, CurveKind::Cubic);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveKind {
    /// Quadratic Bézier control (TrueType `glyf`).
    Quad,
    /// Cubic Bézier control (CFF / PostScript).
    Cubic,
}

/// A point in a contour with on-curve flag.
///
/// On-curve points sit on the path. Off-curve points are Bézier controls;
/// their [`curve_kind`](Self::curve_kind) selects quadratic vs cubic.
///
/// ```
/// use fontmesh::types::{ContourPoint, Point2D};
/// let on = ContourPoint::on_curve(Point2D::new(0.0, 0.0));
/// let off = ContourPoint::off_curve(Point2D::new(0.5, 1.0));
/// assert!(on.on_curve && !off.on_curve);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ContourPoint {
    /// Position in em-normalised coordinates.
    pub point: Point2D,
    /// `true` if this point lies on the path; `false` for a Bézier control.
    pub on_curve: bool,
    /// Curve degree this control point belongs to. Meaningful only when
    /// `on_curve == false`; ignored for on-curve points.
    pub curve_kind: CurveKind,
}

impl ContourPoint {
    /// Construct a point, defaulting off-curve points to [`CurveKind::Quad`].
    ///
    /// ```
    /// use fontmesh::types::{ContourPoint, CurveKind, Point2D};
    /// let p = ContourPoint::new(Point2D::new(1.0, 0.0), false);
    /// assert!(!p.on_curve);
    /// assert_eq!(p.curve_kind, CurveKind::Quad);
    /// ```
    pub fn new(point: Point2D, on_curve: bool) -> Self {
        Self {
            point,
            on_curve,
            curve_kind: CurveKind::Quad,
        }
    }

    /// An on-curve path point.
    ///
    /// ```
    /// use fontmesh::types::{ContourPoint, Point2D};
    /// let p = ContourPoint::on_curve(Point2D::new(0.0, 1.0));
    /// assert!(p.on_curve);
    /// ```
    pub fn on_curve(point: Point2D) -> Self {
        Self {
            point,
            on_curve: true,
            curve_kind: CurveKind::Quad,
        }
    }

    /// A quadratic (TrueType) off-curve control point.
    ///
    /// ```
    /// use fontmesh::types::{ContourPoint, CurveKind, Point2D};
    /// let p = ContourPoint::off_curve(Point2D::new(0.5, 0.5));
    /// assert!(!p.on_curve);
    /// assert_eq!(p.curve_kind, CurveKind::Quad);
    /// ```
    pub fn off_curve(point: Point2D) -> Self {
        Self {
            point,
            on_curve: false,
            curve_kind: CurveKind::Quad,
        }
    }

    /// A cubic (CFF/PostScript) off-curve control point.
    ///
    /// ```
    /// use fontmesh::types::{ContourPoint, CurveKind, Point2D};
    /// let p = ContourPoint::off_curve_cubic(Point2D::new(0.3, 0.8));
    /// assert_eq!(p.curve_kind, CurveKind::Cubic);
    /// ```
    pub fn off_curve_cubic(point: Point2D) -> Self {
        Self {
            point,
            on_curve: false,
            curve_kind: CurveKind::Cubic,
        }
    }
}

/// A single contour (closed or open path).
///
/// Glyph outlines are closed. Open contours are accepted by the triangulator
/// but are uncommon in fonts.
///
/// ```
/// use fontmesh::types::{Contour, Point2D};
/// let mut c = Contour::new(true);
/// c.push_on_curve(Point2D::new(0.0, 0.0));
/// c.push_on_curve(Point2D::new(1.0, 0.0));
/// assert!(!c.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct Contour {
    /// Points in path order (on-curve vertices and off-curve controls).
    pub points: Vec<ContourPoint>,
    /// Whether the last point connects back to the first.
    pub closed: bool,
}

impl Contour {
    /// Create an empty contour.
    ///
    /// Pass `true` for a closed loop (the usual case for glyph outlines).
    ///
    /// ```
    /// use fontmesh::types::Contour;
    /// let c = Contour::new(true);
    /// assert!(c.closed && c.is_empty());
    /// ```
    pub fn new(closed: bool) -> Self {
        Self {
            points: Vec::new(),
            closed,
        }
    }

    /// Append a point.
    ///
    /// ```
    /// use fontmesh::types::{Contour, ContourPoint, Point2D};
    /// let mut c = Contour::new(true);
    /// c.push(ContourPoint::on_curve(Point2D::ZERO));
    /// assert_eq!(c.points.len(), 1);
    /// ```
    pub fn push(&mut self, point: ContourPoint) {
        self.points.push(point);
    }

    /// Append an on-curve point.
    ///
    /// ```
    /// use fontmesh::types::{Contour, Point2D};
    /// let mut c = Contour::new(true);
    /// c.push_on_curve(Point2D::new(0.0, 1.0));
    /// assert!(c.points[0].on_curve);
    /// ```
    pub fn push_on_curve(&mut self, point: Point2D) {
        self.points.push(ContourPoint::on_curve(point));
    }

    /// Append a quadratic off-curve control point.
    ///
    /// ```
    /// use fontmesh::types::{Contour, CurveKind, Point2D};
    /// let mut c = Contour::new(true);
    /// c.push_off_curve(Point2D::new(0.5, 1.0));
    /// assert_eq!(c.points[0].curve_kind, CurveKind::Quad);
    /// ```
    pub fn push_off_curve(&mut self, point: Point2D) {
        self.points.push(ContourPoint::off_curve(point));
    }

    /// `true` if this contour has no points.
    ///
    /// ```
    /// use fontmesh::types::Contour;
    /// assert!(Contour::new(true).is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

/// A collection of contours representing a glyph outline.
///
/// Outer paths and holes live together; triangulation uses the even-odd fill
/// rule, which matches typical font winding.
///
/// ```
/// use fontmesh::types::{Contour, Outline2D, Point2D};
/// let mut outline = Outline2D::new();
/// let mut c = Contour::new(true);
/// c.push_on_curve(Point2D::new(0.0, 0.0));
/// c.push_on_curve(Point2D::new(1.0, 0.0));
/// c.push_on_curve(Point2D::new(0.0, 1.0));
/// outline.add_contour(c);
/// assert_eq!(outline.contours.len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct Outline2D {
    /// Contours that make up the outline (outers and holes).
    pub contours: Vec<Contour>,
}

impl Outline2D {
    /// Create an empty outline.
    ///
    /// ```
    /// use fontmesh::Outline2D;
    /// assert!(Outline2D::new().is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            contours: Vec::new(),
        }
    }

    /// Append a contour.
    ///
    /// ```
    /// use fontmesh::types::{Contour, Outline2D};
    /// let mut outline = Outline2D::new();
    /// outline.add_contour(Contour::new(true));
    /// assert_eq!(outline.contours.len(), 1);
    /// ```
    pub fn add_contour(&mut self, contour: Contour) {
        self.contours.push(contour);
    }

    /// `true` if this outline has no contours.
    ///
    /// ```
    /// use fontmesh::Outline2D;
    /// assert!(Outline2D::new().is_empty());
    /// ```
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
    /// Triangulate this linearized outline into a 2D mesh.
    ///
    /// ```
    /// # let font = fontmesh::parse_font(include_bytes!(concat!(
    /// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
    /// # let gid = fontmesh::glyph_id(&font, 'A').unwrap();
    /// let outline = fontmesh::GlyphMeshBuilder::new(&font, gid)
    ///     .with_subdivisions(20)
    ///     .to_outline()?;
    /// let mesh = outline.triangulate()?;
    /// assert!(mesh.triangle_count() > 0);
    /// # Ok::<(), fontmesh::FontMeshError>(())
    /// ```
    #[inline]
    pub fn triangulate(&self) -> crate::error::Result<Mesh2D> {
        crate::triangulate::triangulate(self)
    }

    /// Convert this outline to a 3D mesh by triangulating and extruding.
    ///
    /// `depth` is the extrusion thickness in em units, centred on z = 0.
    ///
    /// ```
    /// # let font = fontmesh::parse_font(include_bytes!(concat!(
    /// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
    /// # let gid = fontmesh::glyph_id(&font, 'A').unwrap();
    /// let outline = fontmesh::GlyphMeshBuilder::new(&font, gid)
    ///     .with_subdivisions(20)
    ///     .to_outline()?;
    /// let mesh = outline.to_mesh_3d(0.15)?;
    /// assert_eq!(mesh.vertices.len(), mesh.normals.len());
    /// # Ok::<(), fontmesh::FontMeshError>(())
    /// ```
    #[inline]
    pub fn to_mesh_3d(&self, depth: f32) -> crate::error::Result<Mesh3D> {
        let mesh_2d = self.triangulate()?;
        crate::extrude::extrude(&mesh_2d, self, depth)
    }
}

/// A 2D triangle mesh.
///
/// Vertices are em-normalised. `indices` is a flat triangle list (groups of
/// three indices into `vertices`).
///
/// ```
/// use fontmesh::Mesh2D;
/// let mesh = Mesh2D::new();
/// assert!(mesh.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct Mesh2D {
    /// Vertex positions in em-normalised 2D coordinates.
    pub vertices: Vec<Point2D>,
    /// Triangle indices, three per triangle.
    pub indices: Vec<u32>,
}

impl Mesh2D {
    /// Create an empty mesh.
    ///
    /// ```
    /// use fontmesh::Mesh2D;
    /// let mesh = Mesh2D::new();
    /// assert_eq!(mesh.triangle_count(), 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Number of triangles in the mesh (`indices.len() / 3`).
    ///
    /// ```
    /// use fontmesh::types::{Contour, Outline2D, Point2D};
    /// let mut c = Contour::new(true);
    /// c.push_on_curve(Point2D::new(0.0, 0.0));
    /// c.push_on_curve(Point2D::new(1.0, 0.0));
    /// c.push_on_curve(Point2D::new(0.0, 1.0));
    /// let mut outline = Outline2D::new();
    /// outline.add_contour(c);
    /// let mesh = outline.triangulate().unwrap();
    /// assert!(mesh.triangle_count() >= 1);
    /// ```
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// `true` if the mesh has no vertices.
    ///
    /// ```
    /// assert!(fontmesh::Mesh2D::new().is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Write this mesh as a Wavefront OBJ to `w`.
    ///
    /// Vertices are emitted as `v x y 0` (the mesh is flat, so z is zero) and
    /// triangles as `f` lines. OBJ indices are 1-based.
    ///
    /// ```
    /// # let font = fontmesh::parse_font(include_bytes!(concat!(
    /// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
    /// # let gid = fontmesh::glyph_id(&font, 'A').unwrap();
    /// let mesh = fontmesh::glyph_to_mesh_2d(&font, gid, 20)?;
    /// let mut obj = Vec::new();
    /// mesh.write_obj(&mut obj)?;
    /// assert!(obj.starts_with(b"v "));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn write_obj<W: Write>(&self, w: &mut W) -> io::Result<()> {
        for v in &self.vertices {
            writeln!(w, "v {} {} 0", v.x, v.y)?;
        }
        for tri in self.indices.as_chunks::<3>().0 {
            writeln!(w, "f {} {} {}", tri[0] + 1, tri[1] + 1, tri[2] + 1)?;
        }
        Ok(())
    }

    /// Render this mesh to a Wavefront OBJ string. See [`Mesh2D::write_obj`].
    ///
    /// ```
    /// # let font = fontmesh::parse_font(include_bytes!(concat!(
    /// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
    /// # let gid = fontmesh::glyph_id(&font, 'A').unwrap();
    /// let mesh = fontmesh::glyph_to_mesh_2d(&font, gid, 20).unwrap();
    /// let obj = mesh.to_obj_string();
    /// assert!(obj.contains("f "));
    /// ```
    #[must_use]
    pub fn to_obj_string(&self) -> String {
        let mut s = String::new();
        for v in &self.vertices {
            let _ = writeln!(s, "v {} {} 0", v.x, v.y);
        }
        for tri in self.indices.as_chunks::<3>().0 {
            let _ = writeln!(s, "f {} {} {}", tri[0] + 1, tri[1] + 1, tri[2] + 1);
        }
        s
    }

    /// Extrude this 2D mesh into a 3D mesh.
    ///
    /// # Arguments
    /// * `outline` - The linearized outline (used for side-face generation)
    /// * `depth` - Extrusion thickness in em units, centred on z = 0
    ///
    /// ```
    /// # let font = fontmesh::parse_font(include_bytes!(concat!(
    /// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
    /// # let gid = fontmesh::glyph_id(&font, 'A').unwrap();
    /// let outline = fontmesh::GlyphMeshBuilder::new(&font, gid)
    ///     .with_subdivisions(20)
    ///     .to_outline()?;
    /// let mesh_2d = outline.triangulate()?;
    /// let mesh_3d = mesh_2d.extrude(&outline, 0.2)?;
    /// assert!(!mesh_3d.is_empty());
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

/// A 3D triangle mesh with per-vertex normals.
///
/// `vertices`, `normals`, and `indices` are parallel: each vertex has a
/// matching normal, and `indices` is a flat triangle list.
///
/// ```
/// use fontmesh::Mesh3D;
/// let mesh = Mesh3D::new();
/// assert!(mesh.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct Mesh3D {
    /// Vertex positions in em-normalised 3D coordinates.
    pub vertices: Vec<glam::Vec3>,
    /// Per-vertex normals, parallel to [`Self::vertices`].
    pub normals: Vec<glam::Vec3>,
    /// Triangle indices, three per triangle.
    pub indices: Vec<u32>,
}

impl Mesh3D {
    /// Create an empty mesh.
    ///
    /// ```
    /// use fontmesh::Mesh3D;
    /// assert_eq!(Mesh3D::new().triangle_count(), 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Number of triangles in the mesh (`indices.len() / 3`).
    ///
    /// ```
    /// # let font = fontmesh::parse_font(include_bytes!(concat!(
    /// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
    /// # let gid = fontmesh::glyph_id(&font, 'A').unwrap();
    /// let mesh = fontmesh::glyph_to_mesh_3d(&font, gid, 0.1, 20).unwrap();
    /// assert!(mesh.triangle_count() > 0);
    /// ```
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// `true` if the mesh has no vertices.
    ///
    /// ```
    /// assert!(fontmesh::Mesh3D::new().is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Write this mesh as a Wavefront OBJ to `w`, including per-vertex normals.
    ///
    /// Vertices are emitted as `v x y z`, normals as `vn x y z`, and triangles
    /// as `f v//vn` lines. Vertex and normal arrays are parallel, so each face
    /// reuses one index for both. OBJ indices are 1-based.
    ///
    /// ```
    /// # let font = fontmesh::parse_font(include_bytes!(concat!(
    /// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
    /// # let gid = fontmesh::glyph_id(&font, 'A').unwrap();
    /// let mesh = fontmesh::glyph_to_mesh_3d(&font, gid, 0.2, 20)?;
    /// let mut obj = Vec::new();
    /// mesh.write_obj(&mut obj)?;
    /// let text = String::from_utf8(obj)?;
    /// assert!(text.contains("vn "));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn write_obj<W: Write>(&self, w: &mut W) -> io::Result<()> {
        for v in &self.vertices {
            writeln!(w, "v {} {} {}", v.x, v.y, v.z)?;
        }
        for n in &self.normals {
            writeln!(w, "vn {} {} {}", n.x, n.y, n.z)?;
        }
        for tri in self.indices.as_chunks::<3>().0 {
            let (a, b, c) = (tri[0] + 1, tri[1] + 1, tri[2] + 1);
            writeln!(w, "f {a}//{a} {b}//{b} {c}//{c}")?;
        }
        Ok(())
    }

    /// Render this mesh to a Wavefront OBJ string. See [`Mesh3D::write_obj`].
    ///
    /// ```
    /// # let font = fontmesh::parse_font(include_bytes!(concat!(
    /// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
    /// # let gid = fontmesh::glyph_id(&font, 'A').unwrap();
    /// let mesh = fontmesh::glyph_to_mesh_3d(&font, gid, 0.2, 20).unwrap();
    /// let obj = mesh.to_obj_string();
    /// assert!(obj.contains("f "));
    /// ```
    #[must_use]
    pub fn to_obj_string(&self) -> String {
        let mut s = String::new();
        for v in &self.vertices {
            let _ = writeln!(s, "v {} {} {}", v.x, v.y, v.z);
        }
        for n in &self.normals {
            let _ = writeln!(s, "vn {} {} {}", n.x, n.y, n.z);
        }
        for tri in self.indices.as_chunks::<3>().0 {
            let (a, b, c) = (tri[0] + 1, tri[1] + 1, tri[2] + 1);
            let _ = writeln!(s, "f {a}//{a} {b}//{b} {c}//{c}");
        }
        s
    }
}

impl Default for Mesh3D {
    fn default() -> Self {
        Self::new()
    }
}
