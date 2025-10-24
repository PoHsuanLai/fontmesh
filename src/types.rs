//! Core types for fontmesh

use glam::{Vec2, Vec3};

/// Quality level for mesh generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    /// Low quality (10 subdivisions)
    Low,
    /// Normal quality (20 subdivisions)
    Normal,
    /// High quality (50 subdivisions)
    High,
    /// Custom quality level
    Custom(u8),
}

impl Quality {
    /// Get the subdivision level as a scalar value
    pub fn value(&self) -> u8 {
        match self {
            Quality::Low => 10,
            Quality::Normal => 20,
            Quality::High => 50,
            Quality::Custom(v) => *v,
        }
    }
}

impl Default for Quality {
    fn default() -> Self {
        Quality::Normal
    }
}

/// A 2D point
pub type Point2D = Vec2;

/// A 3D point
pub type Point3D = Vec3;

/// A contour is a closed or open path
#[derive(Debug, Clone)]
pub struct Contour {
    /// Points in the contour
    pub points: Vec<Point2D>,
    /// Whether the contour is closed
    pub closed: bool,
}

impl Contour {
    /// Create a new contour
    pub fn new(closed: bool) -> Self {
        Self {
            points: Vec::new(),
            closed,
        }
    }

    /// Add a point to the contour
    pub fn push(&mut self, point: Point2D) {
        self.points.push(point);
    }

    /// Get the number of points
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Check if the contour is empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

/// A 2D outline consisting of multiple contours
#[derive(Debug, Clone, Default)]
pub struct Outline2D {
    /// Contours in the outline
    pub contours: Vec<Contour>,
}

impl Outline2D {
    /// Create a new empty outline
    pub fn new() -> Self {
        Self {
            contours: Vec::new(),
        }
    }

    /// Add a contour to the outline
    pub fn add_contour(&mut self, contour: Contour) {
        self.contours.push(contour);
    }

    /// Check if the outline is empty
    pub fn is_empty(&self) -> bool {
        self.contours.is_empty()
    }

    /// Get the total number of points across all contours
    pub fn total_points(&self) -> usize {
        self.contours.iter().map(|c| c.len()).sum()
    }
}

/// A 2D triangle mesh
#[derive(Debug, Clone, Default)]
pub struct Mesh2D {
    /// Vertex positions (x, y)
    pub vertices: Vec<[f32; 2]>,
    /// Triangle indices (3 per triangle)
    pub indices: Vec<u32>,
}

impl Mesh2D {
    /// Create a new empty 2D mesh
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Get the number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get the number of triangles
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Check if the mesh is empty
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() || self.indices.is_empty()
    }

    /// Iterator over vertices
    pub fn iter_vertices(&self) -> impl Iterator<Item = Vertex2D> + '_ {
        self.vertices.iter().map(|v| Vertex2D { val: *v })
    }

    /// Iterator over faces (triangles)
    pub fn iter_faces(&self) -> impl Iterator<Item = Face> + '_ {
        self.indices.chunks_exact(3).map(|chunk| Face {
            indices: (chunk[0], chunk[1], chunk[2]),
        })
    }
}

/// A 3D triangle mesh with normals
#[derive(Debug, Clone, Default)]
pub struct Mesh3D {
    /// Vertex positions (x, y, z)
    pub vertices: Vec<[f32; 3]>,
    /// Vertex normals (normalized)
    pub normals: Vec<[f32; 3]>,
    /// Triangle indices (3 per triangle)
    pub indices: Vec<u32>,
}

impl Mesh3D {
    /// Create a new empty 3D mesh
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Get the number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get the number of triangles
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Check if the mesh is empty
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() || self.indices.is_empty()
    }

    /// Get the total number of vertices (for compatibility with ttf2mesh API)
    pub fn vertices_len(&self) -> usize {
        self.vertices.len()
    }

    /// Iterator over vertices
    pub fn iter_vertices(&self) -> impl Iterator<Item = Vertex3D> + '_ {
        self.vertices.iter().map(|v| Vertex3D { val: *v })
    }

    /// Iterator over normals (returns Some if normals exist)
    pub fn iter_normals(&self) -> Option<impl Iterator<Item = Normal> + '_> {
        if self.normals.is_empty() {
            None
        } else {
            Some(self.normals.iter().map(|n| Normal { val: *n }))
        }
    }

    /// Iterator over faces (triangles)
    pub fn iter_faces(&self) -> impl Iterator<Item = Face> + '_ {
        self.indices.chunks_exact(3).map(|chunk| Face {
            indices: (chunk[0], chunk[1], chunk[2]),
        })
    }
}

/// 2D vertex wrapper (compatible with ttf2mesh API)
#[derive(Debug, Clone, Copy)]
pub struct Vertex2D {
    pub val: [f32; 2],
}

impl Vertex2D {
    /// Get the vertex values as a tuple (x, y)
    pub fn val(&self) -> (f32, f32) {
        (self.val[0], self.val[1])
    }
}

/// 3D vertex wrapper (compatible with ttf2mesh API)
#[derive(Debug, Clone, Copy)]
pub struct Vertex3D {
    pub val: [f32; 3],
}

impl Vertex3D {
    /// Get the vertex values as a tuple (x, y, z)
    pub fn val(&self) -> (f32, f32, f32) {
        (self.val[0], self.val[1], self.val[2])
    }
}

/// Normal vector wrapper (compatible with ttf2mesh API)
#[derive(Debug, Clone, Copy)]
pub struct Normal {
    pub val: [f32; 3],
}

impl Normal {
    /// Get the normal values as a tuple (x, y, z)
    pub fn val(&self) -> (f32, f32, f32) {
        (self.val[0], self.val[1], self.val[2])
    }
}

/// Face (triangle) wrapper (compatible with ttf2mesh API)
#[derive(Debug, Clone, Copy)]
pub struct Face {
    pub indices: (u32, u32, u32),
}

impl Face {
    /// Get the face indices as a tuple (i0, i1, i2)
    pub fn val(&self) -> (u32, u32, u32) {
        self.indices
    }
}
