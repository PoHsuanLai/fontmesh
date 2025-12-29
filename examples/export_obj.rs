//! Export glyph to OBJ format

use fontmesh::Font;
use std::fs::File;
use std::io::Write;

fn main() {
    let font = Font::from_bytes(include_bytes!("../assets/test_font.ttf")).unwrap();
    let mesh = font.glyph_to_mesh_3d('A', 5.0).unwrap();

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
