//! Serialize/deserialize meshes (requires --features serde)

use fontmesh::Font;
use serde_json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let font = Font::from_bytes(include_bytes!("../assets/test_font.ttf"))?;
    let mesh = font.glyph_to_mesh_3d('A', 5.0)?;

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
