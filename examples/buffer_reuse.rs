//! Example demonstrating efficient buffer reuse for batch processing

use fontmesh::{Font, Mesh2D, Mesh3D, Quality};

fn main() {
    let font_data = include_bytes!("test_font.ttf");
    let font = Font::from_bytes(font_data).expect("Failed to load font");

    println!("=== Buffer Reuse for 2D Meshes ===\n");

    // Create a single mesh buffer and reuse it
    let mut mesh_2d = Mesh2D::new();
    let text = "Hello, World!";

    for c in text.chars() {
        // Reuse the same buffer - much more efficient than allocating each time
        font.glyph_to_mesh_2d_reuse(c, Quality::Normal, &mut mesh_2d)
            .unwrap_or_else(|_| panic!("Failed to generate mesh for '{}'", c));

        println!(
            "  '{}': {} vertices, {} triangles",
            c,
            mesh_2d.vertex_count(),
            mesh_2d.triangle_count()
        );
    }

    println!("\n=== Buffer Reuse for 3D Meshes ===\n");

    // Same approach for 3D meshes
    let mut mesh_3d = Mesh3D::new();
    let depth = 5.0;

    for c in "FONTMESH".chars() {
        font.glyph_to_mesh_3d_reuse(c, Quality::High, depth, &mut mesh_3d)
            .unwrap_or_else(|_| panic!("Failed to generate 3D mesh for '{}'", c));

        println!(
            "  '{}': {} vertices, {} triangles",
            c,
            mesh_3d.vertex_count(),
            mesh_3d.triangle_count()
        );
    }

    println!("\n✓ Buffer reuse demo complete!");
    println!("This approach avoids repeated allocations and is much faster");
    println!("when processing multiple glyphs in a loop.");
}
