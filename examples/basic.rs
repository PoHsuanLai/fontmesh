//! Basic example demonstrating fontmesh usage

use fontmesh::{Font, Quality};

fn main() {
    // Load the font
    let font_data = include_bytes!("test_font.ttf");
    let font = Font::from_bytes(font_data).expect("Failed to load font");

    println!("Font loaded successfully!");

    // Test 2D mesh generation
    println!("\n=== Testing 2D Mesh Generation ===");
    let mesh_2d = font
        .glyph_to_mesh_2d('A', Quality::Normal)
        .expect("Failed to generate 2D mesh");

    println!("Character 'A' 2D mesh:");
    println!("  Vertices: {}", mesh_2d.vertex_count());
    println!("  Triangles: {}", mesh_2d.triangle_count());

    // Test 3D mesh generation
    println!("\n=== Testing 3D Mesh Generation ===");
    let mesh_3d = font
        .glyph_to_mesh_3d('B', Quality::High, 10.0)
        .expect("Failed to generate 3D mesh");

    println!("Character 'B' 3D mesh:");
    println!("  Vertices: {}", mesh_3d.vertex_count());
    println!("  Triangles: {}", mesh_3d.triangle_count());

    // Test multiple characters
    println!("\n=== Testing Multiple Characters ===");
    for c in "Hello".chars() {
        let mesh = font.glyph_to_mesh_2d(c, Quality::Low).unwrap_or_else(|_| panic!("Failed to generate mesh for '{}'",
        c));
        println!(
            "  '{}': {} vertices, {} triangles",
            c,
            mesh.vertex_count(),
            mesh.triangle_count()
        );
    }

    // Test different quality levels
    println!("\n=== Testing Quality Levels ===");
    for quality in &[Quality::Low, Quality::Normal, Quality::High] {
        let mesh = font
            .glyph_to_mesh_2d('S', *quality)
            .expect("Failed to generate mesh");
        println!(
            "  {:?}: {} vertices, {} triangles",
            quality,
            mesh.vertex_count(),
            mesh.triangle_count()
        );
    }

    println!("\n✓ All tests passed!");
}
