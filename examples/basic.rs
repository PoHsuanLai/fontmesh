//! Basic fontmesh usage

use fontmesh::{char_to_mesh_2d, char_to_mesh_3d, Face};

fn main() {
    let font_data = include_bytes!("../assets/test_font.ttf");
    let face = Face::parse(font_data, 0).unwrap();

    // 2D mesh with 20 subdivisions
    let mesh_2d = char_to_mesh_2d(&face, 'A', 20).unwrap();
    println!(
        "2D 'A': {} vertices, {} triangles",
        mesh_2d.vertices.len(),
        mesh_2d.triangle_count()
    );

    // 3D mesh with custom quality (50 subdivisions)
    let mesh_3d = char_to_mesh_3d(&face, 'B', 5.0, 50).unwrap();
    println!(
        "3D 'B': {} vertices, {} triangles",
        mesh_3d.vertices.len(),
        mesh_3d.triangle_count()
    );
}
