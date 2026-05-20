//! Serialize/deserialize meshes (requires --features serde)

use fontmesh::{glyph_id, glyph_to_mesh_3d, parse_font};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let font_data = include_bytes!("../assets/test_font.ttf");
    let font = parse_font(font_data)?;
    let gid = glyph_id(&font, 'A').expect("font is missing 'A'");
    let mesh = glyph_to_mesh_3d(&font, gid, 5.0, 20)?;

    let json = serde_json::to_string(&mesh)?;
    let loaded: fontmesh::Mesh3D = serde_json::from_str(&json)?;

    assert_eq!(mesh.vertices.len(), loaded.vertices.len());
    println!(
        "Serialized {} vertices to {} bytes",
        mesh.vertices.len(),
        json.len()
    );
    Ok(())
}
