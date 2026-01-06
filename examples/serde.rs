//! Serialize/deserialize meshes (requires --features serde)

use fontmesh::{char_to_mesh_3d, Face};
use serde_json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let font_data = include_bytes!("../assets/test_font.ttf");
    let face = Face::parse(font_data, 0)?;
    let mesh = char_to_mesh_3d(&face, 'A', 5.0, 20)?;

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
