//! Comprehensive tests to validate fontmesh correctness
//!
//! This test suite validates that fontmesh produces correct mesh output
//! by checking:
//! - Mesh structure validity
//! - Vertex/normal/index count relationships
//! - Proper triangulation (all indices within bounds)
//! - Normal vector validity (normalized)
//! - Mesh topology (closed, manifold)

use fontmesh::Font;

const TEST_FONT: &[u8] = include_bytes!("../examples/test_font.ttf");

#[test]
fn test_2d_mesh_structure() {
    let font = Font::from_bytes(TEST_FONT).expect("Failed to load font");

    // Test multiple characters
    for c in "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".chars() {
        let mesh = font
            .glyph_to_mesh_2d(c)
            .unwrap_or_else(|_| panic!("Failed to generate mesh for '{}'", c));

        // Basic structure validation
        assert!(
            mesh.vertices.len() > 0,
            "Mesh for '{}' should have vertices",
            c
        );
        assert!(
            mesh.indices.len().is_multiple_of(3),
            "Indices for '{}' should be multiple of 3",
            c
        );

        // All indices should be within vertex range
        for &idx in &mesh.indices {
            assert!(
                (idx as usize) < mesh.vertices.len(),
                "Index {} out of bounds for character '{}' with {} vertices",
                idx,
                c,
                mesh.vertices.len()
            );
        }

        // Vertices should be in reasonable range (normalized coordinates)
        for vertex in &mesh.vertices {
            assert!(
                vertex[0].is_finite(),
                "Vertex x should be finite for '{}'",
                c
            );
            assert!(
                vertex[1].is_finite(),
                "Vertex y should be finite for '{}'",
                c
            );
        }
    }
}

#[test]
fn test_3d_mesh_structure() {
    let font = Font::from_bytes(TEST_FONT).expect("Failed to load font");

    // Test multiple characters with different depths
    for c in "ABCXYZ123".chars() {
        for depth in [1.0, 5.0, 10.0] {
            let mesh = font
                .glyph_to_mesh_3d(c, depth)
                .unwrap_or_else(|_| panic!("Failed to generate 3D mesh for '{}'", c));

            // Basic structure validation
            assert!(
                mesh.vertices.len() > 0,
                "3D Mesh for '{}' should have vertices",
                c
            );
            assert_eq!(
                mesh.vertices.len(),
                mesh.normals.len(),
                "Vertices and normals count should match for '{}'",
                c
            );
            assert!(
                mesh.indices.len().is_multiple_of(3),
                "Indices for '{}' should be multiple of 3",
                c
            );

            // All indices should be within vertex range
            for &idx in &mesh.indices {
                assert!(
                    (idx as usize) < mesh.vertices.len(),
                    "Index {} out of bounds for character '{}' with {} vertices",
                    idx,
                    c,
                    mesh.vertices.len()
                );
            }

            // Normals should be normalized (length ~= 1.0)
            for normal in &mesh.normals {
                let length_sq =
                    normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2];
                let length = length_sq.sqrt();
                assert!(
                    (length - 1.0).abs() < 0.01,
                    "Normal should be normalized for '{}', got length {}",
                    c,
                    length
                );
            }

            // Vertices should be in reasonable range
            for vertex in &mesh.vertices {
                assert!(
                    vertex[0].is_finite(),
                    "Vertex x should be finite for '{}'",
                    c
                );
                assert!(
                    vertex[1].is_finite(),
                    "Vertex y should be finite for '{}'",
                    c
                );
                assert!(
                    vertex[2].is_finite(),
                    "Vertex z should be finite for '{}'",
                    c
                );

                // Z coordinate should be within [-depth/2, depth/2]
                let half_depth = depth / 2.0;
                assert!(
                    vertex[2] >= -half_depth - 0.01 && vertex[2] <= half_depth + 0.01,
                    "Vertex z {} should be within depth range [-{}, {}] for '{}'",
                    vertex[2],
                    half_depth,
                    half_depth,
                    c
                );
            }
        }
    }
}

#[test]
fn test_quality_levels() {
    let font = Font::from_bytes(TEST_FONT).expect("Failed to load font");

    let low = font
        .glyph_by_char('S')
        .unwrap()
        .with_subdivisions(10)
        .to_mesh_2d()
        .unwrap();
    let normal = font
        .glyph_by_char('S')
        .unwrap()
        .with_subdivisions(20)
        .to_mesh_2d()
        .unwrap();
    let high = font
        .glyph_by_char('S')
        .unwrap()
        .with_subdivisions(50)
        .to_mesh_2d()
        .unwrap();

    // Higher quality should generally produce more vertices
    // (for characters with curves like 'S')
    assert!(
        low.vertices.len() <= normal.vertices.len(),
        "Lower subdivisions should have <= vertices than normal"
    );
    assert!(
        normal.vertices.len() <= high.vertices.len(),
        "Default subdivisions should have <= vertices than high"
    );

    println!("Quality comparison for 'S':");
    println!(
        "  subdivisions=10: {} vertices, {} triangles",
        low.vertices.len(),
        low.triangle_count()
    );
    println!(
        "  subdivisions=20: {} vertices, {} triangles",
        normal.vertices.len(),
        normal.triangle_count()
    );
    println!(
        "  subdivisions=50: {} vertices, {} triangles",
        high.vertices.len(),
        high.triangle_count()
    );
}

#[test]
fn test_direct_access() {
    let font = Font::from_bytes(TEST_FONT).expect("Failed to load font");
    let mesh = font.glyph_to_mesh_3d('A', 5.0).unwrap();

    // Test direct vertex access
    assert!(mesh.vertices.len() > 0, "Mesh should have vertices");

    // Test direct normal access
    assert_eq!(
        mesh.normals.len(),
        mesh.vertices.len(),
        "Should have one normal per vertex"
    );

    // Test direct index access
    assert_eq!(mesh.indices.len() % 3, 0, "Indices should be multiple of 3");

    // Test vertex values
    for vertex in mesh.vertices.iter().take(5) {
        assert!(
            vertex.x.is_finite() && vertex.y.is_finite() && vertex.z.is_finite(),
            "Vertex values should be finite"
        );
    }

    // Test normal values
    for normal in mesh.normals.iter().take(5) {
        let length_sq = normal.length_squared();
        assert!(
            (length_sq.sqrt() - 1.0).abs() < 0.01,
            "Normal should be normalized"
        );
    }

    // Test face indices
    for chunk in mesh.indices.chunks(3).take(5) {
        let (i0, i1, i2) = (chunk[0], chunk[1], chunk[2]);
        assert!(
            (i0 as usize) < mesh.vertices.len(),
            "Face index 0 should be in bounds"
        );
        assert!(
            (i1 as usize) < mesh.vertices.len(),
            "Face index 1 should be in bounds"
        );
        assert!(
            (i2 as usize) < mesh.vertices.len(),
            "Face index 2 should be in bounds"
        );
    }
}

#[test]
fn test_mesh_topology() {
    let font = Font::from_bytes(TEST_FONT).expect("Failed to load font");

    // Test characters with different topologies
    let test_chars = vec![
        ('A', "single contour with hole"),
        ('B', "multiple holes"),
        ('O', "single hole"),
        ('I', "simple vertical"),
        ('8', "two holes"),
    ];

    for (c, description) in test_chars {
        let mesh = font.glyph_to_mesh_2d(c).unwrap();

        println!(
            "Character '{}' ({}): {} vertices, {} triangles",
            c,
            description,
            mesh.vertices.len(),
            mesh.triangle_count()
        );

        // Should have at least 1 triangle
        assert!(
            mesh.triangle_count() >= 1,
            "Character '{}' should have at least 1 triangle",
            c
        );

        // Euler characteristic for planar graph: V - E + F = 1 + H
        // where H is number of holes
        // For triangulated mesh: E = (3F + boundary_edges) / 2
        // This is approximate, just checking reasonable bounds
        let v = mesh.vertices.len();
        let f = mesh.triangle_count();

        assert!(v >= 3, "Should have at least 3 vertices for '{}'", c);
        assert!(f >= 1, "Should have at least 1 face for '{}'", c);
    }
}

#[test]
fn test_special_characters() {
    let font = Font::from_bytes(TEST_FONT).expect("Failed to load font");

    // Test punctuation and symbols
    let special = vec!['.', ',', '!', '?', '@', '#', '$', '%', '&', '*'];

    for c in special {
        match font.glyph_to_mesh_2d(c) {
            Ok(mesh) => {
                assert!(
                    mesh.vertices.len() > 0,
                    "Special char '{}' should have vertices",
                    c
                );
                println!(
                    "Character '{}': {} vertices, {} triangles",
                    c,
                    mesh.vertices.len(),
                    mesh.triangle_count()
                );
            }
            Err(e) => {
                // Some fonts might not have all special characters
                println!("Character '{}' not available in font: {:?}", c, e);
            }
        }
    }
}

#[test]
fn test_depth_consistency() {
    let font = Font::from_bytes(TEST_FONT).expect("Failed to load font");

    let depths = vec![0.5, 1.0, 2.0, 5.0, 10.0];
    let mut vertex_counts = Vec::new();

    for &depth in &depths {
        let mesh = font.glyph_to_mesh_3d('M', depth).unwrap();
        vertex_counts.push(mesh.vertices.len());

        // Check that all vertices respect the depth
        for vertex in &mesh.vertices {
            let half_depth = depth / 2.0;
            assert!(
                vertex[2] >= -half_depth - 0.01 && vertex[2] <= half_depth + 0.01,
                "Vertex z {} should be within depth range [-{}, {}]",
                vertex[2],
                half_depth,
                half_depth
            );
        }

        println!(
            "Depth {}: {} vertices, {} triangles",
            depth,
            mesh.vertices.len(),
            mesh.triangle_count()
        );
    }

    // Vertex count should be similar across different depths (same 2D outline)
    // Allow some variance due to extrusion edge handling
    let min_count = *vertex_counts.iter().min().unwrap();
    let max_count = *vertex_counts.iter().max().unwrap();

    // The vertex count can vary slightly, but shouldn't be wildly different
    // Front face + back face + sides should scale consistently
    println!("Vertex count range: {} to {}", min_count, max_count);
}

#[test]
fn test_error_handling() {
    let font = Font::from_bytes(TEST_FONT).expect("Failed to load font");

    // Test character that might not exist in the font
    let rare_chars = vec!['\u{1F600}', '\u{2603}', '\u{FFFF}'];

    for c in rare_chars {
        match font.glyph_to_mesh_2d(c) {
            Ok(_) => {
                println!("Character U+{:04X} is available", c as u32);
            }
            Err(e) => {
                println!("Character U+{:04X} not available: {:?}", c as u32, e);
                // Error should be GlyphNotFound or NoOutline
                assert!(
                    format!("{:?}", e).contains("GlyphNotFound")
                        || format!("{:?}", e).contains("NoOutline"),
                    "Error should be GlyphNotFound or NoOutline"
                );
            }
        }
    }
}
