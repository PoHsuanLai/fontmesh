//! 3D extrusion - converts 2D meshes to 3D with depth

use crate::error::Result;
use crate::types::{Mesh2D, Mesh3D, Outline2D};
use glam::Vec3;
use rustc_hash::FxHashMap;

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
#[inline]
pub fn extrude(mesh_2d: &Mesh2D, outline: &Outline2D, depth: f32) -> Result<Mesh3D> {
    let half_depth = depth / 2.0;

    // Pre-calculate total size to avoid reallocations
    let outline_edge_count: usize = outline
        .contours
        .iter()
        .map(|c| {
            if c.closed {
                c.points.len()
            } else {
                c.points.len().saturating_sub(1)
            }
        })
        .sum();

    let total_vertices = mesh_2d.vertices.len() * 2 + outline_edge_count * 4;
    let total_indices = mesh_2d.indices.len() * 2 + outline_edge_count * 6;

    let mut mesh_3d = Mesh3D {
        vertices: Vec::with_capacity(total_vertices),
        normals: Vec::with_capacity(total_vertices),
        indices: Vec::with_capacity(total_indices),
    };

    // 1. Create front face (z = half_depth)
    let normal_front = Vec3::new(0.0, 0.0, 1.0);
    for vertex in &mesh_2d.vertices {
        mesh_3d
            .vertices
            .push(Vec3::new(vertex.x, vertex.y, half_depth));
        mesh_3d.normals.push(normal_front);
    }

    // Add front face triangles
    mesh_3d.indices.extend_from_slice(&mesh_2d.indices);

    // 2. Create back face (z = -half_depth) with reversed winding
    let back_offset = mesh_3d.vertices.len() as u32;
    let normal_back = Vec3::new(0.0, 0.0, -1.0);
    for vertex in &mesh_2d.vertices {
        mesh_3d
            .vertices
            .push(Vec3::new(vertex.x, vertex.y, -half_depth));
        mesh_3d.normals.push(normal_back);
    }

    // Add back face triangles (reversed winding for correct normals)
    for chunk in mesh_2d.indices.chunks_exact(3) {
        mesh_3d.indices.push(back_offset + chunk[0]);
        mesh_3d.indices.push(back_offset + chunk[2]); // Reversed
        mesh_3d.indices.push(back_offset + chunk[1]); // Reversed
    }

    // 3. Create side faces
    create_side_faces(&mut mesh_3d, outline, half_depth);

    Ok(mesh_3d)
}

/// Create side faces by connecting outline edges with smooth normals
#[inline]
fn create_side_faces(mesh_3d: &mut Mesh3D, outline: &Outline2D, half_depth: f32) {
    for contour in &outline.contours {
        let num_points = contour.points.len();
        if num_points < 2 {
            continue;
        }

        let points = &contour.points;

        // Calculate normals on-the-fly to avoid allocation
        for i in 0..num_points {
            let next = if contour.closed {
                (i + 1) % num_points
            } else if i == num_points - 1 {
                break;
            } else {
                i + 1
            };

            let p0 = points[i].point;
            let p1 = points[next].point;
            let edge_vec = p1 - p0;

            // Skip degenerate edges
            let edge_len_sq = edge_vec.length_squared();
            if edge_len_sq < 1e-10 {
                continue;
            }

            // Calculate current edge normal (fast path - no sqrt needed for direction)
            let edge_dir = edge_vec * (1.0 / edge_len_sq.sqrt());
            let current_normal = Vec3::new(-edge_dir.y, edge_dir.x, 0.0);

            // Calculate smooth normals by averaging with adjacent edges
            let prev_idx = if i == 0 {
                if contour.closed {
                    num_points - 1
                } else {
                    i
                }
            } else {
                i - 1
            };

            let normal_p0 = if contour.closed || i > 0 {
                // Calculate previous edge normal
                let prev_next = i;
                let pp0 = points[prev_idx].point;
                let pp1 = points[prev_next].point;
                let prev_edge = pp1 - pp0;
                let prev_len_sq = prev_edge.length_squared();

                if prev_len_sq > 1e-10 {
                    let prev_dir = prev_edge * (1.0 / prev_len_sq.sqrt());
                    let prev_normal = Vec3::new(-prev_dir.y, prev_dir.x, 0.0);
                    ((prev_normal + current_normal) * 0.5).normalize_or_zero()
                } else {
                    current_normal
                }
            } else {
                current_normal
            };

            let normal_p1 = if contour.closed || next < num_points - 1 {
                let next_next = if contour.closed {
                    (next + 1) % num_points
                } else if next < num_points - 1 {
                    next + 1
                } else {
                    next
                };

                let np0 = points[next].point;
                let np1 = points[next_next].point;
                let next_edge = np1 - np0;
                let next_len_sq = next_edge.length_squared();

                if next_len_sq > 1e-10 {
                    let next_dir = next_edge * (1.0 / next_len_sq.sqrt());
                    let next_normal = Vec3::new(-next_dir.y, next_dir.x, 0.0);
                    ((current_normal + next_normal) * 0.5).normalize_or_zero()
                } else {
                    current_normal
                }
            } else {
                current_normal
            };

            // Create 4 vertices for the quad (2 triangles)
            let base_idx = mesh_3d.vertices.len() as u32;

            // Front edge vertices
            mesh_3d.vertices.push(Vec3::new(p0.x, p0.y, half_depth));
            mesh_3d.normals.push(normal_p0);

            mesh_3d.vertices.push(Vec3::new(p1.x, p1.y, half_depth));
            mesh_3d.normals.push(normal_p1);

            // Back edge vertices
            mesh_3d.vertices.push(Vec3::new(p1.x, p1.y, -half_depth));
            mesh_3d.normals.push(normal_p1);

            mesh_3d.vertices.push(Vec3::new(p0.x, p0.y, -half_depth));
            mesh_3d.normals.push(normal_p0);

            // Two triangles for the quad
            let indices = [
                base_idx,
                base_idx + 1,
                base_idx + 2,
                base_idx,
                base_idx + 2,
                base_idx + 3,
            ];
            mesh_3d.indices.extend_from_slice(&indices);
        }
    }
}

/// Compute smooth normals for a mesh (optional post-processing)
///
/// This function recomputes normals by averaging face normals at shared vertices,
/// resulting in smoother shading. The extrude process already generates smooth normals
/// for side faces, but this can be used if you want to regenerate them or apply to
/// a custom mesh.
///
/// **Note:** In most cases, you don't need to call this manually - the 3D extrusion
/// already produces smooth normals.
///
/// # Arguments
/// * `mesh` - The mesh to recompute normals for (modified in-place)
///
/// # Example
/// ```
/// use fontmesh::{Font, compute_smooth_normals};
///
/// let font = Font::from_bytes(include_bytes!("../examples/test_font.ttf"))?;
/// let mut mesh = font.glyph_to_mesh_3d('A', 5.0)?;
///
/// // Regenerate smooth normals (usually not needed)
/// compute_smooth_normals(&mut mesh);
/// # Ok::<(), fontmesh::FontMeshError>(())
/// ```
pub fn compute_smooth_normals(mesh: &mut Mesh3D) {
    // Group vertices by position to find shared vertices
    let mut position_map: FxHashMap<[i32; 3], Vec<usize>> = FxHashMap::default();

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

        let v0 = mesh.vertices[i0];
        let v1 = mesh.vertices[i1];
        let v2 = mesh.vertices[i2];

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let face_normal = edge1.cross(edge2).normalize();

        accumulated_normals[i0] += face_normal;
        accumulated_normals[i1] += face_normal;
        accumulated_normals[i2] += face_normal;
    }

    // Track which vertices have been processed (for shared positions)
    let mut processed = vec![false; mesh.vertices.len()];

    // Average normals for vertices at the same position
    for indices in position_map.values() {
        if indices.len() <= 1 {
            // Single vertex at this position - just normalize its accumulated normal
            let idx = indices[0];
            if accumulated_normals[idx] != Vec3::ZERO {
                mesh.normals[idx] = accumulated_normals[idx].normalize();
                processed[idx] = true;
            }
        } else {
            // Multiple vertices at same position - average their normals
            let mut sum = Vec3::ZERO;
            for &idx in indices {
                sum += accumulated_normals[idx];
            }
            let averaged = sum.normalize();

            // Apply averaged normal to all vertices at this position
            for &idx in indices {
                mesh.normals[idx] = averaged;
                processed[idx] = true;
            }
        }
    }

    // Normalize any remaining normals (shouldn't happen, but be safe)
    for (i, normal) in mesh.normals.iter_mut().enumerate() {
        if !processed[i] && accumulated_normals[i] != Vec3::ZERO {
            *normal = accumulated_normals[i].normalize();
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
            vertices: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                Vec2::new(0.0, 1.0),
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        };

        let mut outline = Outline2D::new();
        let mut contour = Contour::new(true);
        contour.push_on_curve(Vec2::new(0.0, 0.0));
        contour.push_on_curve(Vec2::new(1.0, 0.0));
        contour.push_on_curve(Vec2::new(1.0, 1.0));
        contour.push_on_curve(Vec2::new(0.0, 1.0));
        outline.add_contour(contour);

        let mesh_3d = extrude(&mesh_2d, &outline, 1.0).expect("Extrusion should succeed");

        // Should have front face, back face, and 4 side faces
        assert!(mesh_3d.vertices.len() > 0);
        assert!(mesh_3d.triangle_count() > 0);
        assert_eq!(mesh_3d.vertices.len(), mesh_3d.normals.len());
    }
}
