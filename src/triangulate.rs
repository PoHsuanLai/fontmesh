//! 2D triangulation using lyon_tessellation

use crate::error::{FontMeshError, Result};
use crate::types::{Mesh2D, Outline2D};
use glam::Vec2;
use lyon_tessellation::{
    FillOptions, FillTessellator, FillVertex, GeometryBuilder, VertexBuffers, VertexId,
};

/// Triangulate a 2D outline into a triangle mesh
///
/// Uses lyon_tessellation to convert the outline polygons into triangles
/// with proper handling of holes and complex shapes.
///
/// # Arguments
/// * `outline` - The linearized outline to triangulate
///
/// # Returns
/// A 2D triangle mesh
#[inline]
pub fn triangulate(outline: &Outline2D) -> Result<Mesh2D> {
    if outline.is_empty() {
        return Err(FontMeshError::TriangulationFailed(
            "Empty outline".to_string(),
        ));
    }

    // Pre-allocate buffers based on outline size
    // Estimate: roughly 4x the number of outline points for vertices
    // and ~3x vertices for indices (each triangle = 3 indices)
    let point_count: usize = outline.contours.iter()
        .map(|c| c.points.len())
        .sum();
    let estimated_vertices = point_count * 4;
    let estimated_indices = estimated_vertices * 3;

    let mut geometry: VertexBuffers<[f32; 2], u32> = VertexBuffers::with_capacity(
        estimated_vertices,
        estimated_indices
    );
    let mut tessellator = FillTessellator::new();

    // Configure fill options (even-odd rule for font glyphs)
    let options = FillOptions::default().with_fill_rule(lyon_tessellation::FillRule::EvenOdd);

    // Build the path from our outline
    let mut builder = lyon_tessellation::path::Path::builder();

    for contour in &outline.contours {
        if contour.is_empty() {
            continue;
        }

        // Start the contour
        let first = contour.points[0].point;
        builder.begin(lyon_tessellation::math::Point::new(first.x, first.y));

        // Add lines to the rest of the points
        for cp in &contour.points[1..] {
            builder.line_to(lyon_tessellation::math::Point::new(cp.point.x, cp.point.y));
        }

        // Close the contour if needed
        if contour.closed {
            builder.close();
        } else {
            builder.end(false);
        }
    }

    let path = builder.build();

    // Tessellate the path
    tessellator
        .tessellate_path(&path, &options, &mut SimpleBuffersBuilder(&mut geometry))
        .map_err(|e| {
            FontMeshError::TriangulationFailed(format!("Lyon tessellation failed: {:?}", e))
        })?;

    // Convert to our Mesh2D format (pre-allocate for efficiency)
    let vertices: Vec<Vec2> = geometry.vertices.into_iter().map(Vec2::from).collect();
    Ok(Mesh2D {
        vertices,
        indices: geometry.indices,
    })
}

/// Simple geometry builder for lyon tessellation
struct SimpleBuffersBuilder<'a>(&'a mut VertexBuffers<[f32; 2], u32>);

impl<'a> GeometryBuilder for SimpleBuffersBuilder<'a> {
    #[inline]
    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
        self.0.indices.push(a.0);
        self.0.indices.push(b.0);
        self.0.indices.push(c.0);
    }
}

impl<'a> lyon_tessellation::FillGeometryBuilder for SimpleBuffersBuilder<'a> {
    fn add_fill_vertex(
        &mut self,
        vertex: FillVertex,
    ) -> std::result::Result<VertexId, lyon_tessellation::GeometryBuilderError> {
        let index = self.0.vertices.len() as u32;
        self.0
            .vertices
            .push([vertex.position().x, vertex.position().y]);
        Ok(VertexId(index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Contour;
    use glam::Vec2;

    #[test]
    fn test_triangulate_square() {
        // Create a simple square outline
        let mut outline = Outline2D::new();
        let mut contour = Contour::new(true);

        contour.push_on_curve(Vec2::new(0.0, 0.0));
        contour.push_on_curve(Vec2::new(1.0, 0.0));
        contour.push_on_curve(Vec2::new(1.0, 1.0));
        contour.push_on_curve(Vec2::new(0.0, 1.0));

        outline.add_contour(contour);

        let mesh = triangulate(&outline).expect("Triangulation should succeed");

        // A square should produce at least 2 triangles (4 vertices minimum)
        assert!(mesh.vertices.len() >= 4);
        assert!(mesh.triangle_count() >= 2);
    }
}
