//! Basic fontmesh usage

use fontmesh::Font;

fn main() {
    let font = Font::from_bytes(include_bytes!("../assets/test_font.ttf")).unwrap();

    // 2D mesh (default quality)
    let mesh_2d = font.glyph_to_mesh_2d('A').unwrap();
    println!(
        "2D 'A': {} vertices, {} triangles",
        mesh_2d.vertices.len(),
        mesh_2d.triangle_count()
    );

    // 3D mesh with custom quality
    let mesh_3d = font
        .glyph_by_char('B')
        .unwrap()
        .with_subdivisions(50)
        .to_mesh_3d(5.0)
        .unwrap();
    println!(
        "3D 'B': {} vertices, {} triangles",
        mesh_3d.vertices.len(),
        mesh_3d.triangle_count()
    );
}
