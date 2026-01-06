//! Export glyph to OBJ format

use fontmesh::{char_to_mesh_3d, Face};
use std::fs::File;
use std::io::Write;

fn main() {
    let font_data = include_bytes!("../assets/test_font.ttf");
    let face = Face::parse(font_data, 0).unwrap();
    let mesh = char_to_mesh_3d(&face, 'A', 5.0, 20).unwrap();

    let mut file = File::create("glyph_A.obj").unwrap();
    writeln!(
        file,
        "# Glyph 'A': {} vertices, {} triangles",
        mesh.vertices.len(),
        mesh.triangle_count()
    )
    .unwrap();

    for v in &mesh.vertices {
        writeln!(file, "v {} {} {}", v.x, v.y, v.z).unwrap();
    }
    for n in &mesh.normals {
        writeln!(file, "vn {} {} {}", n.x, n.y, n.z).unwrap();
    }
    for tri in mesh.indices.chunks(3) {
        writeln!(
            file,
            "f {0}//{0} {1}//{1} {2}//{2}",
            tri[0] + 1,
            tri[1] + 1,
            tri[2] + 1
        )
        .unwrap();
    }

    println!("Exported glyph_A.obj");
}
