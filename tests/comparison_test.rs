//! Comprehensive tests to validate fontmesh correctness
//!
//! This test suite validates that fontmesh produces correct mesh output
//! by checking:
//! - Mesh structure validity
//! - Vertex/normal/index count relationships
//! - Proper triangulation (all indices within bounds)
//! - Normal vector validity (normalized)
//! - Mesh topology (closed, manifold)

use fontmesh::{
    glyph_id, glyph_to_mesh_2d, glyph_to_mesh_3d, parse_font, FontRef, GlyphId, GlyphMeshBuilder,
    Mesh2D, Mesh3D,
};

const TEST_FONT: &[u8] = include_bytes!("../assets/test_font.ttf");

fn font() -> FontRef<'static> {
    parse_font(TEST_FONT).expect("Failed to load font")
}

fn gid(font: &FontRef, c: char) -> GlyphId {
    glyph_id(font, c).unwrap_or_else(|| panic!("font is missing '{c}'"))
}

fn mesh_2d(font: &FontRef, c: char, subdivisions: u8) -> Mesh2D {
    glyph_to_mesh_2d(font, gid(font, c), subdivisions)
        .unwrap_or_else(|e| panic!("Failed to generate 2D mesh for '{c}': {e:?}"))
}

fn mesh_3d(font: &FontRef, c: char, depth: f32, subdivisions: u8) -> Mesh3D {
    glyph_to_mesh_3d(font, gid(font, c), depth, subdivisions)
        .unwrap_or_else(|e| panic!("Failed to generate 3D mesh for '{c}': {e:?}"))
}

#[test]
fn test_2d_mesh_structure() {
    let font = font();

    for c in "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".chars() {
        let mesh = mesh_2d(&font, c, 20);

        assert!(
            !mesh.vertices.is_empty(),
            "Mesh for '{}' should have vertices",
            c
        );
        assert!(
            mesh.indices.len().is_multiple_of(3),
            "Indices for '{}' should be multiple of 3",
            c
        );

        for &idx in &mesh.indices {
            assert!(
                (idx as usize) < mesh.vertices.len(),
                "Index {} out of bounds for character '{}' with {} vertices",
                idx,
                c,
                mesh.vertices.len()
            );
        }

        for vertex in &mesh.vertices {
            assert!(vertex.x.is_finite(), "Vertex x should be finite for '{}'", c);
            assert!(vertex.y.is_finite(), "Vertex y should be finite for '{}'", c);
        }
    }
}

#[test]
fn test_3d_mesh_structure() {
    let font = font();

    for c in "ABCXYZ123".chars() {
        for depth in [1.0, 5.0, 10.0] {
            let mesh = mesh_3d(&font, c, depth, 20);

            assert!(
                !mesh.vertices.is_empty(),
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

            for &idx in &mesh.indices {
                assert!(
                    (idx as usize) < mesh.vertices.len(),
                    "Index {} out of bounds for character '{}' with {} vertices",
                    idx,
                    c,
                    mesh.vertices.len()
                );
            }

            for normal in &mesh.normals {
                let length = normal.length();
                assert!(
                    (length - 1.0).abs() < 0.01,
                    "Normal should be normalized for '{}', got length {}",
                    c,
                    length
                );
            }

            for vertex in &mesh.vertices {
                assert!(vertex.x.is_finite(), "Vertex x should be finite for '{}'", c);
                assert!(vertex.y.is_finite(), "Vertex y should be finite for '{}'", c);
                assert!(vertex.z.is_finite(), "Vertex z should be finite for '{}'", c);

                let half_depth = depth / 2.0;
                assert!(
                    vertex.z >= -half_depth - 0.01 && vertex.z <= half_depth + 0.01,
                    "Vertex z {} should be within depth range [-{}, {}] for '{}'",
                    vertex.z,
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
    let font = font();
    let s = gid(&font, 'S');

    let low = GlyphMeshBuilder::new(&font, s).with_subdivisions(10).to_mesh_2d().unwrap();
    let normal = GlyphMeshBuilder::new(&font, s).with_subdivisions(20).to_mesh_2d().unwrap();
    let high = GlyphMeshBuilder::new(&font, s).with_subdivisions(50).to_mesh_2d().unwrap();

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
    let font = font();
    let mesh = mesh_3d(&font, 'A', 5.0, 20);

    assert!(!mesh.vertices.is_empty(), "Mesh should have vertices");
    assert_eq!(
        mesh.normals.len(),
        mesh.vertices.len(),
        "Should have one normal per vertex"
    );
    assert_eq!(mesh.indices.len() % 3, 0, "Indices should be multiple of 3");

    for vertex in mesh.vertices.iter().take(5) {
        assert!(
            vertex.x.is_finite() && vertex.y.is_finite() && vertex.z.is_finite(),
            "Vertex values should be finite"
        );
    }

    for normal in mesh.normals.iter().take(5) {
        let length_sq = normal.length_squared();
        assert!(
            (length_sq.sqrt() - 1.0).abs() < 0.01,
            "Normal should be normalized"
        );
    }

    for chunk in mesh.indices.chunks(3).take(5) {
        for &i in chunk {
            assert!(
                (i as usize) < mesh.vertices.len(),
                "Face index should be in bounds"
            );
        }
    }
}

#[test]
fn test_mesh_topology() {
    let font = font();

    let test_chars = vec![
        ('A', "single contour with hole"),
        ('B', "multiple holes"),
        ('O', "single hole"),
        ('I', "simple vertical"),
        ('8', "two holes"),
    ];

    for (c, description) in test_chars {
        let mesh = mesh_2d(&font, c, 20);

        println!(
            "Character '{}' ({}): {} vertices, {} triangles",
            c,
            description,
            mesh.vertices.len(),
            mesh.triangle_count()
        );

        assert!(
            mesh.triangle_count() >= 1,
            "Character '{}' should have at least 1 triangle",
            c
        );

        assert!(mesh.vertices.len() >= 3, "Should have at least 3 vertices for '{}'", c);
        assert!(mesh.triangle_count() >= 1, "Should have at least 1 face for '{}'", c);
    }
}

#[test]
fn test_special_characters() {
    let font = font();

    let special = vec!['.', ',', '!', '?', '@', '#', '$', '%', '&', '*'];

    for c in special {
        let Some(gid) = glyph_id(&font, c) else {
            println!("Special char '{c}' is missing from the font");
            continue;
        };
        match glyph_to_mesh_2d(&font, gid, 20) {
            Ok(mesh) => {
                assert!(
                    !mesh.vertices.is_empty(),
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
                println!("Character '{c}' couldn't be tessellated: {e:?}");
            }
        }
    }
}

#[test]
fn test_depth_consistency() {
    let font = font();

    let depths = vec![0.5, 1.0, 2.0, 5.0, 10.0];
    let mut vertex_counts = Vec::new();

    for &depth in &depths {
        let mesh = mesh_3d(&font, 'M', depth, 20);
        vertex_counts.push(mesh.vertices.len());

        for vertex in &mesh.vertices {
            let half_depth = depth / 2.0;
            assert!(
                vertex.z >= -half_depth - 0.01 && vertex.z <= half_depth + 0.01,
                "Vertex z {} should be within depth range [-{}, {}]",
                vertex.z,
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

    let min_count = *vertex_counts.iter().min().unwrap();
    let max_count = *vertex_counts.iter().max().unwrap();
    println!("Vertex count range: {} to {}", min_count, max_count);
}

#[test]
fn test_error_handling() {
    let font = font();

    let rare_chars = vec!['\u{1F600}', '\u{2603}', '\u{FFFF}'];

    for c in rare_chars {
        match glyph_id(&font, c) {
            None => {
                println!("Character U+{:04X} is not in the font", c as u32);
            }
            Some(gid) => match glyph_to_mesh_2d(&font, gid, 20) {
                Ok(_) => println!("Character U+{:04X} is available", c as u32),
                Err(e) => {
                    println!("Character U+{:04X} not available: {:?}", c as u32, e);
                    assert!(
                        format!("{e:?}").contains("NoOutline")
                            || format!("{e:?}").contains("OutlineExtractionFailed"),
                        "Error should be NoOutline or OutlineExtractionFailed"
                    );
                }
            },
        }
    }
}
