//! Basic fontmesh usage

use fontmesh::{glyph_id, glyph_to_mesh_2d, glyph_to_mesh_3d, parse_font};

fn main() {
    let font_data = include_bytes!("../assets/test_font.ttf");
    let font = parse_font(font_data).unwrap();

    let a = glyph_id(&font, 'A').expect("font is missing 'A'");
    let b = glyph_id(&font, 'B').expect("font is missing 'B'");

    let mesh_2d = glyph_to_mesh_2d(&font, a, 20).unwrap();
    println!(
        "2D 'A': {} vertices, {} triangles",
        mesh_2d.vertices.len(),
        mesh_2d.triangle_count()
    );

    let mesh_3d = glyph_to_mesh_3d(&font, b, 5.0, 50).unwrap();
    println!(
        "3D 'B': {} vertices, {} triangles",
        mesh_3d.vertices.len(),
        mesh_3d.triangle_count()
    );
}
