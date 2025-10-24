//! 3D extrusion - converts 2D meshes to 3D with depth

use crate::error::Result;
use crate::types::{Mesh2D, Mesh3D, Outline2D};
use glam::Vec3;
use std::collections::HashMap;

/// Extrude a 2D mesh into 3D with the given depth
///
/// Creates a 3D mesh by:
/// 1. Front face at z = 0
/// 2. Back face at z = depth
/// 3. Side faces connecting the edges
/// 4. Smooth normals for curved surfaces
///
/// # Arguments
/// * `mesh_2d` - The 2D triangle mesh to extrude
/// * `outline` - The original outline (used for edge detection)
/// * `depth` - The extrusion depth
///
/// # Returns
/// A 3D triangle mesh with normals
pub fn extrude(mesh_2d: &Mesh2D, outline: &Outline2D, depth: f32) -> Result<Mesh3D> {
    let mut mesh_3d = Mesh3D::new();

    let half_depth = depth / 2.0;

    // 1. Create front face (z = half_depth)
    let front_offset = mesh_3d.vertices.len() as u32;
    for vertex in &mesh_2d.vertices {
        mesh_3d.vertices.push([vertex[0], vertex[1], half_depth]);
        mesh_3d.normals.push([0.0, 0.0, 1.0]); // Front face points +Z
    }

    // Add front face triangles
    for &index in &mesh_2d.indices {
        mesh_3d.indices.push(front_offset + index);
    }

    // 2. Create back face (z = -half_depth) with reversed winding
    let back_offset = mesh_3d.vertices.len() as u32;
    for vertex in &mesh_2d.vertices {
        mesh_3d.vertices.push([vertex[0], vertex[1], -half_depth]);
        mesh_3d.normals.push([0.0, 0.0, -1.0]); // Back face points -Z
    }

    // Add back face triangles (reversed winding for correct normals)
    for chunk in mesh_2d.indices.chunks(3) {
        mesh_3d.indices.push(back_offset + chunk[0]);
        mesh_3d.indices.push(back_offset + chunk[2]); // Reversed
        mesh_3d.indices.push(back_offset + chunk[1]); // Reversed
    }

    // 3. Create side faces
    create_side_faces(&mut mesh_3d, outline, half_depth);

    Ok(mesh_3d)
}

/// Create side faces by connecting outline edges
fn create_side_faces(mesh_3d: &mut Mesh3D, outline: &Outline2D, half_depth: f32) {
    for contour in &outline.contours {
        if contour.points.len() < 2 {
            continue;
        }

        let points = &contour.points;
        let num_points = points.len();

        for i in 0..num_points {
            let next = if contour.closed {
                (i + 1) % num_points
            } else if i == num_points - 1 {
                break; // Don't create edge for last point if not closed
            } else {
                i + 1
            };

            let p0 = points[i];
            let p1 = points[next];

            // Calculate edge direction
            let edge_vec = p1 - p0;

            // Skip degenerate edges (same point)
            if edge_vec.length_squared() < 1e-10 {
                continue;
            }

            let edge_dir = edge_vec.normalize();
            let normal = Vec3::new(-edge_dir.y, edge_dir.x, 0.0); // Perpendicular in XY plane

            // Create 4 vertices for the quad (2 triangles)
            let base_idx = mesh_3d.vertices.len() as u32;

            // Front edge vertices
            mesh_3d.vertices.push([p0.x, p0.y, half_depth]);
            mesh_3d.normals.push(normal.to_array());

            mesh_3d.vertices.push([p1.x, p1.y, half_depth]);
            mesh_3d.normals.push(normal.to_array());

            // Back edge vertices
            mesh_3d.vertices.push([p1.x, p1.y, -half_depth]);
            mesh_3d.normals.push(normal.to_array());

            mesh_3d.vertices.push([p0.x, p0.y, -half_depth]);
            mesh_3d.normals.push(normal.to_array());

            // Two triangles for the quad
            mesh_3d.indices.push(base_idx);
            mesh_3d.indices.push(base_idx + 1);
            mesh_3d.indices.push(base_idx + 2);

            mesh_3d.indices.push(base_idx);
            mesh_3d.indices.push(base_idx + 2);
            mesh_3d.indices.push(base_idx + 3);
        }
    }
}

/// Compute smooth normals for a mesh (optional post-processing)
#[allow(dead_code)]
pub fn compute_smooth_normals(mesh: &mut Mesh3D) {
    // Group vertices by position to find shared vertices
    let mut position_map: HashMap<[i32; 3], Vec<usize>> = HashMap::new();

    // Quantize positions for matching (to handle floating point imprecision)
    const QUANTIZE: f32 = 10000.0;
    for (i, vertex) in mesh.vertices.iter().enumerate() {
        let key = [
            (vertex[0] * QUANTIZE) as i32,
            (vertex[1] * QUANTIZE) as i32,
            (vertex[2] * QUANTIZE) as i32,
        ];
        position_map.entry(key).or_default().push(i);
    }

    // Accumulate normals from all faces using each vertex
    let mut accumulated_normals = vec![Vec3::ZERO; mesh.vertices.len()];

    for triangle in mesh.indices.chunks(3) {
        let i0 = triangle[0] as usize;
        let i1 = triangle[1] as usize;
        let i2 = triangle[2] as usize;

        let v0 = Vec3::from_array(mesh.vertices[i0]);
        let v1 = Vec3::from_array(mesh.vertices[i1]);
        let v2 = Vec3::from_array(mesh.vertices[i2]);

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let face_normal = edge1.cross(edge2).normalize();

        accumulated_normals[i0] += face_normal;
        accumulated_normals[i1] += face_normal;
        accumulated_normals[i2] += face_normal;
    }

    // Average normals for vertices at the same position
    for indices in position_map.values() {
        if indices.len() <= 1 {
            continue;
        }

        // Sum all normals for this position
        let mut sum = Vec3::ZERO;
        for &idx in indices {
            sum += accumulated_normals[idx];
        }
        let averaged = sum.normalize();

        // Apply to all vertices at this position
        for &idx in indices {
            mesh.normals[idx] = averaged.to_array();
        }
    }

    // Normalize all normals
    for (i, normal) in mesh.normals.iter_mut().enumerate() {
        if accumulated_normals[i] != Vec3::ZERO {
            *normal = accumulated_normals[i].normalize().to_array();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Contour, Outline2D};
    use glam::Vec2;

    #[test]
    fn test_extrude_square() {
        // Create a simple square mesh
        let mesh_2d = Mesh2D {
            vertices: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
        };

        let mut outline = Outline2D::new();
        let mut contour = Contour::new(true);
        contour.push(Vec2::new(0.0, 0.0));
        contour.push(Vec2::new(1.0, 0.0));
        contour.push(Vec2::new(1.0, 1.0));
        contour.push(Vec2::new(0.0, 1.0));
        outline.add_contour(contour);

        let mesh_3d = extrude(&mesh_2d, &outline, 1.0).expect("Extrusion should succeed");

        // Should have front face, back face, and 4 side faces
        assert!(mesh_3d.vertex_count() > 0);
        assert!(mesh_3d.triangle_count() > 0);
        assert_eq!(mesh_3d.vertices.len(), mesh_3d.normals.len());
    }
}
