//! Core type definitions for fontmesh

use glam::Vec2;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type Point2D = Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Quality {
    Low,
    Normal,
    High,
    Custom(u8),
}

impl Quality {
    pub fn value(&self) -> u8 {
        match self {
            Quality::Low => 10,
            Quality::Normal => 20,
            Quality::High => 50,
            Quality::Custom(v) => *v,
        }
    }
}

/// A point in a contour with on-curve flag
#[derive(Debug, Clone, Copy)]
pub struct ContourPoint {
    pub point: Point2D,
    pub on_curve: bool,
}

impl ContourPoint {
    pub fn new(point: Point2D, on_curve: bool) -> Self {
        Self { point, on_curve }
    }

    pub fn on_curve(point: Point2D) -> Self {
        Self {
            point,
            on_curve: true,
        }
    }

    pub fn off_curve(point: Point2D) -> Self {
        Self {
            point,
            on_curve: false,
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

/// A 2D triangle mesh
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Mesh2D {
    pub vertices: Vec<Point2D>,
    pub indices: Vec<u32>,
}

impl Mesh2D {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Clear the mesh data for reuse (retains capacity)
    ///
    /// This allows the mesh to be reused for multiple glyphs without
    /// reallocating memory, improving performance when processing many glyphs.
    #[inline]
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    /// Check if the mesh is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
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
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Clear the mesh data for reuse (retains capacity)
    ///
    /// This allows the mesh to be reused for multiple glyphs without
    /// reallocating memory, improving performance when processing many glyphs.
    #[inline]
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.normals.clear();
        self.indices.clear();
    }

    /// Check if the mesh is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Iterate over vertices
    pub fn iter_vertices(&self) -> VertexIterator<'_> {
        VertexIterator {
            vertices: &self.vertices,
            index: 0,
        }
    }

    /// Iterate over normals
    pub fn iter_normals(&self) -> Option<NormalIterator<'_>> {
        if self.normals.is_empty() {
            None
        } else {
            Some(NormalIterator {
                normals: &self.normals,
                index: 0,
            })
        }
    }

    /// Iterate over faces (triangles)
    pub fn iter_faces(&self) -> FaceIterator<'_> {
        FaceIterator {
            indices: &self.indices,
            index: 0,
        }
    }
}

impl Default for Mesh3D {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over mesh vertices
pub struct VertexIterator<'a> {
    vertices: &'a [glam::Vec3],
    index: usize,
}

impl<'a> Iterator for VertexIterator<'a> {
    type Item = Vertex<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vertices.len() {
            let vertex = &self.vertices[self.index];
            self.index += 1;
            Some(Vertex { vertex })
        } else {
            None
        }
    }
}

/// Vertex accessor
pub struct Vertex<'a> {
    vertex: &'a glam::Vec3,
}

impl<'a> Vertex<'a> {
    /// Get vertex coordinates as (x, y, z)
    pub fn val(&self) -> (f32, f32, f32) {
        (self.vertex.x, self.vertex.y, self.vertex.z)
    }
}

/// Iterator over mesh normals
pub struct NormalIterator<'a> {
    normals: &'a [glam::Vec3],
    index: usize,
}

impl<'a> Iterator for NormalIterator<'a> {
    type Item = Normal<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.normals.len() {
            let normal = &self.normals[self.index];
            self.index += 1;
            Some(Normal { normal })
        } else {
            None
        }
    }
}

/// Normal accessor
pub struct Normal<'a> {
    normal: &'a glam::Vec3,
}

impl<'a> Normal<'a> {
    /// Get normal coordinates as (nx, ny, nz)
    pub fn val(&self) -> (f32, f32, f32) {
        (self.normal.x, self.normal.y, self.normal.z)
    }
}

/// Iterator over mesh faces
pub struct FaceIterator<'a> {
    indices: &'a [u32],
    index: usize,
}

impl<'a> Iterator for FaceIterator<'a> {
    type Item = Face;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index + 2 < self.indices.len() {
            let i0 = self.indices[self.index];
            let i1 = self.indices[self.index + 1];
            let i2 = self.indices[self.index + 2];
            self.index += 3;
            Some(Face { i0, i1, i2 })
        } else {
            None
        }
    }
}

/// Face (triangle) accessor
pub struct Face {
    i0: u32,
    i1: u32,
    i2: u32,
}

impl Face {
    /// Get face indices as (i0, i1, i2)
    pub fn val(&self) -> (u32, u32, u32) {
        (self.i0, self.i1, self.i2)
    }
}
