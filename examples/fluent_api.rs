//! Example demonstrating the fluent API with method chaining

use fontmesh::Font;

fn main() {
    // Load the font
    let font = Font::from_bytes(include_bytes!("test_font.ttf")).expect("Failed to load font");

    println!("=== Fluent API Examples ===\n");

    // Example 1: Fluent 2D mesh generation with default quality
    println!("1. Fluent 2D mesh generation (default quality):");
    let mesh_2d = font
        .glyph_by_char('A')
        .expect("Failed to get glyph")
        .to_mesh_2d()
        .expect("Failed to generate mesh");
    println!("   'A': {} vertices, {} triangles", mesh_2d.vertices.len(), mesh_2d.triangle_count());

    // Example 2: Fluent 3D mesh generation with custom quality
    println!("\n2. Fluent 3D mesh generation (custom quality: 50):");
    let mesh_3d = font
        .glyph_by_char('B')
        .expect("Failed to get glyph")
        .with_subdivisions(50)
        .to_mesh_3d(5.0)
        .expect("Failed to generate mesh");
    println!("   'B': {} vertices, {} triangles", mesh_3d.vertices.len(), mesh_3d.triangle_count());

    // Example 3: Access intermediate pipeline stages
    println!("\n3. Pipeline with intermediate stages:");
    let glyph = font.glyph_by_char('C').expect("Failed to get glyph");
    println!("   Glyph advance: {}", glyph.advance());

    let outline = glyph.linearize().expect("Failed to linearize");
    println!("   Contours: {}", outline.contours.len());

    let mesh_2d = outline.triangulate().expect("Failed to triangulate");
    println!("   2D mesh: {} vertices", mesh_2d.vertices.len());

    let mesh_3d = mesh_2d.extrude(&outline, 5.0).expect("Failed to extrude");
    println!("   3D mesh: {} vertices", mesh_3d.vertices.len());

    // Example 4: Reusing 2D mesh for multiple extrusions
    println!("\n4. Multiple extrusions from same 2D mesh:");
    let glyph = font.glyph_by_char('D').expect("Failed to get glyph");
    let outline = glyph.linearize().expect("Failed to linearize");
    let mesh_2d = outline.triangulate().expect("Failed to triangulate");

    for depth in &[1.0, 5.0, 10.0] {
        let mesh_3d = mesh_2d.extrude(&outline, *depth).expect("Failed to extrude");
        println!("   Depth {}: {} vertices, {} triangles",
                 depth, mesh_3d.vertices.len(), mesh_3d.triangle_count());
    }

    // Example 5: Direct outline to 3D (skip 2D mesh variable)
    println!("\n5. Direct outline to 3D:");
    let mesh_3d = font
        .glyph_by_char('E')
        .expect("Failed to get glyph")
        .linearize()
        .expect("Failed to linearize")
        .to_mesh_3d(5.0)
        .expect("Failed to create 3D mesh");
    println!("   'E': {} vertices, {} triangles", mesh_3d.vertices.len(), mesh_3d.triangle_count());

    // Example 6: Compare with traditional API
    println!("\n6. Traditional vs Fluent API:");

    // Traditional (uses default quality)
    let traditional = font.glyph_to_mesh_3d('F', 5.0)
        .expect("Failed to generate mesh");
    println!("   Traditional: {} vertices", traditional.vertices.len());

    // Fluent (default quality)
    let fluent = font
        .glyph_by_char('F')
        .expect("Failed to get glyph")
        .to_mesh_3d(5.0)
        .expect("Failed to generate mesh");
    println!("   Fluent: {} vertices", fluent.vertices.len());

    // Fluent with custom quality
    let custom = font
        .glyph_by_char('F')
        .expect("Failed to get glyph")
        .with_subdivisions(30)
        .to_mesh_3d(5.0)
        .expect("Failed to generate mesh");
    println!("   Fluent (subdivisions=30): {} vertices", custom.vertices.len());

    println!("\n✓ All fluent API examples completed!");
}
