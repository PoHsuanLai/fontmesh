//! Basic example demonstrating fontmesh usage

use fontmesh::Font;

fn main() {
    // Load the font
    let font = Font::from_bytes(include_bytes!("test_font.ttf")).expect("Failed to load font");

    println!("Font loaded successfully!");

    // Test 2D mesh generation (default quality: 20 subdivisions)
    println!("\n=== Testing 2D Mesh Generation ===");
    let mesh_2d = font
        .glyph_to_mesh_2d('A')
        .expect("Failed to generate 2D mesh");

    println!("Character 'A' 2D mesh:");
    println!("  Vertices: {}", mesh_2d.vertices.len());
    println!("  Triangles: {}", mesh_2d.triangle_count());

    // Test 3D mesh generation with custom quality
    println!("\n=== Testing 3D Mesh Generation ===");
    let mesh_3d = font
        .glyph_by_char('B')
        .expect("Failed to get glyph")
        .with_subdivisions(50)
        .to_mesh_3d(10.0)
        .expect("Failed to generate 3D mesh");

    println!("Character 'B' 3D mesh (subdivisions=50):");
    println!("  Vertices: {}", mesh_3d.vertices.len());
    println!("  Triangles: {}", mesh_3d.triangle_count());

    // Test multiple characters with default quality
    println!("\n=== Testing Multiple Characters ===");
    for c in "Hello".chars() {
        let mesh = font
            .glyph_to_mesh_2d(c)
            .unwrap_or_else(|_| panic!("Failed to generate mesh for '{}'", c));
        println!(
            "  '{}': {} vertices, {} triangles",
            c,
            mesh.vertices.len(),
            mesh.triangle_count()
        );
    }

    // Test different quality levels
    println!("\n=== Testing Quality Levels ===");
    for quality in &[10u8, 20, 50] {
        let mesh = font
            .glyph_by_char('S')
            .expect("Failed to get glyph")
            .with_subdivisions(*quality)
            .to_mesh_2d()
            .expect("Failed to generate mesh");
        println!(
            "  subdivisions={}: {} vertices, {} triangles",
            quality,
            mesh.vertices.len(),
            mesh.triangle_count()
        );
    }

    println!("\n✓ All tests passed!");
}
