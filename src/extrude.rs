//! 3D extrusion — converts 2D meshes to 3D with depth.
//!
//! Front faces sit at `z = +depth/2`, back faces at `z = -depth/2`, with
//! side faces connecting outline edges. Side normals are outward for both
//! TrueType (outer CW) and CFF (outer CCW) winding.
//!
//! ```
//! use fontmesh::{extrude, triangulate};
//! use fontmesh::types::{Contour, Outline2D, Point2D};
//!
//! let mut c = Contour::new(true);
//! c.push_on_curve(Point2D::new(0.0, 0.0));
//! c.push_on_curve(Point2D::new(1.0, 0.0));
//! c.push_on_curve(Point2D::new(1.0, 1.0));
//! c.push_on_curve(Point2D::new(0.0, 1.0));
//! let mut outline = Outline2D::new();
//! outline.add_contour(c);
//! let mesh_2d = triangulate(&outline).unwrap();
//! let mesh_3d = extrude(&mesh_2d, &outline, 0.5).unwrap();
//! assert_eq!(mesh_3d.vertices.len(), mesh_3d.normals.len());
//! ```

use crate::error::Result;
use crate::types::{Mesh2D, Mesh3D, Outline2D};
use glam::Vec3;
use rustc_hash::FxHashMap;

/// Extrude a 2D mesh into 3D with the given depth.
///
/// Creates a 3D mesh by:
/// 1. Front face at z = +depth/2
/// 2. Back face at z = -depth/2
/// 3. Side faces connecting front and back edges with outward-facing normals
///
/// # Arguments
/// * `mesh_2d` - The 2D triangle mesh to extrude
/// * `outline` - The linearized outline (used for side-face generation)
/// * `depth` - Extrusion thickness in em units, centred on z = 0
///
/// ```
/// use fontmesh::{extrude, triangulate};
/// use fontmesh::types::{Contour, Outline2D, Point2D};
///
/// let mut c = Contour::new(true);
/// c.push_on_curve(Point2D::new(0.0, 0.0));
/// c.push_on_curve(Point2D::new(1.0, 0.0));
/// c.push_on_curve(Point2D::new(1.0, 1.0));
/// c.push_on_curve(Point2D::new(0.0, 1.0));
/// let mut outline = Outline2D::new();
/// outline.add_contour(c);
/// let mesh_2d = triangulate(&outline).unwrap();
/// let mesh_3d = extrude(&mesh_2d, &outline, 1.0).unwrap();
/// assert!(mesh_3d.triangle_count() > mesh_2d.triangle_count());
/// ```
#[inline]
pub fn extrude(mesh_2d: &Mesh2D, outline: &Outline2D, depth: f32) -> Result<Mesh3D> {
    let half_depth = depth / 2.0;

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

    // Front face: reverse the input winding so the front normal (+z) is CCW
    // from the viewer and back-face culling keeps it visible.
    let normal_front = Vec3::new(0.0, 0.0, 1.0);
    mesh_2d.vertices.iter().for_each(|vertex| {
        mesh_3d
            .vertices
            .push(Vec3::new(vertex.x, vertex.y, half_depth));
        mesh_3d.normals.push(normal_front);
    });
    mesh_2d.indices.chunks_exact(3).for_each(|chunk| {
        mesh_3d.indices.push(chunk[0]);
        mesh_3d.indices.push(chunk[2]);
        mesh_3d.indices.push(chunk[1]);
    });

    // Back face: keep the input winding (CW from the viewer = CCW from -z).
    let back_offset = mesh_3d.vertices.len() as u32;
    let normal_back = Vec3::new(0.0, 0.0, -1.0);
    mesh_2d.vertices.iter().for_each(|vertex| {
        mesh_3d
            .vertices
            .push(Vec3::new(vertex.x, vertex.y, -half_depth));
        mesh_3d.normals.push(normal_back);
    });
    mesh_2d.indices.chunks_exact(3).for_each(|chunk| {
        mesh_3d.indices.push(back_offset + chunk[0]);
        mesh_3d.indices.push(back_offset + chunk[1]);
        mesh_3d.indices.push(back_offset + chunk[2]);
    });

    create_side_faces(&mut mesh_3d, outline, half_depth);

    Ok(mesh_3d)
}

/// Create side faces by connecting outline edges with outward-facing normals.
///
/// The "outward" direction depends on each contour's winding. Fonts don't
/// pin winding: TrueType ships outer contours CW and holes CCW (in math
/// Y-up coordinates), while CFF/PostScript ships outer contours CCW and
/// holes CW. We detect per contour by signed area and flip the perpendicular
/// and triangle winding accordingly so the produced sides face outward in
/// both cases.
#[inline]
fn create_side_faces(mesh_3d: &mut Mesh3D, outline: &Outline2D, half_depth: f32) {
    for contour in &outline.contours {
        let num_points = contour.points.len();
        if num_points < 2 {
            continue;
        }

        let points = &contour.points;

        // Signed area in math (Y-up) coordinates: positive => CCW, negative => CW.
        // When walking the contour in its native direction the interior sits
        // to one side; the *opposite* perpendicular of each edge points
        // outward. For a CCW outer contour the interior is on the left, so
        // the right perpendicular points outward. For a CW outer contour
        // (TrueType convention) the interior is on the right, so the left
        // perpendicular points outward.
        let signed_area_x2: f32 = (0..num_points)
            .map(|i| {
                let p0 = points[i].point;
                let p1 = points[(i + 1) % num_points].point;
                p0.x * p1.y - p1.x * p0.y
            })
            .sum();
        let ccw = signed_area_x2 > 0.0;

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

            let edge_len_sq = edge_vec.length_squared();
            if edge_len_sq < 1e-10 {
                continue;
            }

            let edge_dir = edge_vec * (1.0 / edge_len_sq.sqrt());

            let face_normal = if ccw {
                Vec3::new(edge_dir.y, -edge_dir.x, 0.0) // right perp = outward for CCW
            } else {
                Vec3::new(-edge_dir.y, edge_dir.x, 0.0) // left perp = outward for CW
            };

            let base_idx = mesh_3d.vertices.len() as u32;

            mesh_3d.vertices.push(Vec3::new(p0.x, p0.y, half_depth)); // 0: p0 front
            mesh_3d.normals.push(face_normal);
            mesh_3d.vertices.push(Vec3::new(p1.x, p1.y, half_depth)); // 1: p1 front
            mesh_3d.normals.push(face_normal);
            mesh_3d.vertices.push(Vec3::new(p1.x, p1.y, -half_depth)); // 2: p1 back
            mesh_3d.normals.push(face_normal);
            mesh_3d.vertices.push(Vec3::new(p0.x, p0.y, -half_depth)); // 3: p0 back
            mesh_3d.normals.push(face_normal);

            // Triangle winding must be CCW when viewed from the outward
            // direction so the geometric normal matches `face_normal` and
            // back-face culling keeps the triangle visible from outside.
            // For CCW contours (right perp outward), reversed winding
            // [0,2,1],[0,3,2] orients the quad CCW from outside. For CW
            // contours (left perp outward), the natural winding [0,1,2],
            // [0,2,3] is CCW from outside.
            if ccw {
                mesh_3d.indices.extend_from_slice(&[
                    base_idx,
                    base_idx + 2,
                    base_idx + 1,
                    base_idx,
                    base_idx + 3,
                    base_idx + 2,
                ]);
            } else {
                mesh_3d.indices.extend_from_slice(&[
                    base_idx,
                    base_idx + 1,
                    base_idx + 2,
                    base_idx,
                    base_idx + 2,
                    base_idx + 3,
                ]);
            }
        }
    }
}

/// Recompute per-vertex normals by averaging adjacent face normals.
///
/// Positions are quantized so floating-point duplicates (front/back/side
/// corners that occupy the same point) share a normal. Extrusion already
/// writes usable normals; call this only if you want smoother shading on a
/// custom mesh.
///
/// # Arguments
/// * `mesh` - The mesh to recompute normals for (modified in-place)
///
/// ```
/// use fontmesh::{parse_font, glyph_id, glyph_to_mesh_3d, compute_smooth_normals};
///
/// # let font_data = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"));
/// let font = parse_font(font_data)?;
/// let gid = glyph_id(&font, 'A').unwrap();
/// let mut mesh = glyph_to_mesh_3d(&font, gid, 0.2, 20)?;
/// compute_smooth_normals(&mut mesh);
/// assert_eq!(mesh.vertices.len(), mesh.normals.len());
/// # Ok::<(), fontmesh::FontMeshError>(())
/// ```
pub fn compute_smooth_normals(mesh: &mut Mesh3D) {
    // Quantize positions so floating-point imprecision doesn't split
    // logically-shared vertices when we look up neighbours.
    const QUANTIZE: f32 = 10000.0;
    let mut position_map: FxHashMap<[i32; 3], Vec<usize>> = FxHashMap::default();
    for (i, vertex) in mesh.vertices.iter().enumerate() {
        let key = [
            (vertex[0] * QUANTIZE) as i32,
            (vertex[1] * QUANTIZE) as i32,
            (vertex[2] * QUANTIZE) as i32,
        ];
        position_map.entry(key).or_default().push(i);
    }

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

    let mut processed = vec![false; mesh.vertices.len()];

    for indices in position_map.values() {
        if indices.len() <= 1 {
            let idx = indices[0];
            if accumulated_normals[idx] != Vec3::ZERO {
                mesh.normals[idx] = accumulated_normals[idx].normalize();
                processed[idx] = true;
            }
        } else {
            let mut sum = Vec3::ZERO;
            for &idx in indices {
                sum += accumulated_normals[idx];
            }
            let averaged = sum.normalize();
            for &idx in indices {
                mesh.normals[idx] = averaged;
                processed[idx] = true;
            }
        }
    }

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

        assert!(!mesh_3d.vertices.is_empty());
        assert!(mesh_3d.triangle_count() > 0);
        assert_eq!(mesh_3d.vertices.len(), mesh_3d.normals.len());
    }
}
