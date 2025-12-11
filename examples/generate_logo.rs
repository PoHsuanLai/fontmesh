//! Generate the FONTMESH logo for README
//!
//! This example creates a high-quality 3D mesh of "FONTMESH" text
//! and exports it to OBJ format for rendering.

use fontmesh::Font;
use std::fs::File;
use std::io::Write;

fn main() {
    // Use Arial Bold for a stronger, bolder look
    let font_bytes = std::fs::read("/System/Library/Fonts/Supplemental/Arial Bold.ttf")
        .expect("Failed to load Arial Bold");
    let font = Font::from_bytes(&font_bytes).expect("Failed to parse font");

    println!("Generating FONTMESH logo...");

    // Generate with high quality and reasonable depth
    let text = "FONTMESH";
    let depth = 30.0; // Twice the depth for more pronounced 3D effect
    let subdivisions = 50; // High quality curves
    let filename = "fontmesh_logo.obj";

    export_text(&font, text, filename, depth, subdivisions);

    println!("\n✓ Logo exported to {}", filename);
    println!("\nTo render in Blender:");
    println!("  1. File > Import > Wavefront (.obj)");
    println!("  2. Add metallic material (Shader Editor):");
    println!("     - Metallic: 1.0");
    println!("     - Roughness: 0.3");
    println!("     - Base Color: Silver/Chrome");
    println!("  3. Add HDRI lighting for reflections");
    println!("  4. Camera: Front orthographic view");
    println!("  5. Render!");
}

fn export_text(font: &Font, text: &str, filename: &str, depth: f32, subdivisions: u8) {
    let mut file = File::create(filename).expect("Failed to create file");

    writeln!(file, "# Fontmesh logo: '{}'", text).unwrap();
    writeln!(file, "# Depth: {}", depth).unwrap();
    writeln!(file, "# Subdivisions: {}", subdivisions).unwrap();
    writeln!(file, "mtllib fontmesh_logo.mtl").unwrap();
    writeln!(file, "usemtl MetallicMaterial").unwrap();
    writeln!(file).unwrap();

    let mut vertex_offset = 0u32;
    let mut x_position = 0.0f32;

    for c in text.chars() {
        let glyph = font.glyph_by_char(c).expect("Glyph not found");
        let mesh = glyph
            .with_subdivisions(subdivisions)
            .to_mesh_3d(depth)
            .expect("Failed to generate mesh");

        writeln!(file, "o glyph_{}", c).unwrap();

        // Write vertices with x-offset
        for vertex in &mesh.vertices {
            writeln!(
                file,
                "v {} {} {}",
                vertex.x + x_position,
                vertex.y,
                vertex.z
            )
            .unwrap();
        }

        // Write normals
        for normal in &mesh.normals {
            writeln!(file, "vn {} {} {}", normal.x, normal.y, normal.z).unwrap();
        }

        // Write faces with offset indices
        for chunk in mesh.indices.chunks(3) {
            writeln!(
                file,
                "f {0}//{0} {1}//{1} {2}//{2}",
                chunk[0] + 1 + vertex_offset,
                chunk[1] + 1 + vertex_offset,
                chunk[2] + 1 + vertex_offset
            )
            .unwrap();
        }

        writeln!(file).unwrap();

        vertex_offset += mesh.vertices.len() as u32;
        x_position += glyph.advance() * 1.1; // Tighter spacing for logo
    }

    // Also create a basic MTL file for the material
    let mtl_filename = "fontmesh_logo.mtl";
    let mut mtl_file = File::create(mtl_filename).expect("Failed to create MTL file");

    writeln!(mtl_file, "# Fontmesh metallic material").unwrap();
    writeln!(mtl_file, "newmtl MetallicMaterial").unwrap();
    writeln!(mtl_file, "Ka 0.8 0.8 0.8  # Ambient color").unwrap();
    writeln!(mtl_file, "Kd 0.9 0.9 0.9  # Diffuse color (light silver)").unwrap();
    writeln!(
        mtl_file,
        "Ks 1.0 1.0 1.0  # Specular color (bright highlights)"
    )
    .unwrap();
    writeln!(mtl_file, "Ns 300.0        # Specular exponent (shininess)").unwrap();
    writeln!(mtl_file, "Ni 1.5          # Optical density").unwrap();
    writeln!(mtl_file, "d 1.0           # Dissolve (opacity)").unwrap();
    writeln!(
        mtl_file,
        "illum 3         # Illumination model (reflection)"
    )
    .unwrap();

    println!("  Created {} and {}", filename, mtl_filename);
}
