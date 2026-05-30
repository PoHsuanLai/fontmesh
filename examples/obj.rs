//! Export a glyph mesh to a Wavefront OBJ file.

use fontmesh::{glyph_id, glyph_to_mesh_3d, parse_font};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let font_data = include_bytes!("../assets/test_font.ttf");
    let font = parse_font(font_data)?;
    let gid = glyph_id(&font, 'A').expect("font is missing 'A'");
    let mesh = glyph_to_mesh_3d(&font, gid, 5.0, 20)?;

    let mut file = std::fs::File::create("A.obj")?;
    mesh.write_obj(&mut file)?;

    println!(
        "Wrote A.obj: {} vertices, {} triangles",
        mesh.vertices.len(),
        mesh.triangle_count()
    );
    Ok(())
}
