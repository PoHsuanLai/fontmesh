//! Example: Export glyphs to Wavefront OBJ format

use fontmesh::{Font, Quality};
use std::fs::File;
use std::io::Write;

fn main() {
    let font_data = include_bytes!("test_font.ttf");
    let font = Font::from_bytes(font_data).expect("Failed to load font");

    println!("Exporting glyphs to OBJ files...\n");

    // Export a single 2D glyph
    export_2d_glyph(&font, 'A', "glyph_A_2d.obj");

    // Export a single 3D glyph
    export_3d_glyph(&font, 'B', "glyph_B_3d.obj", 10.0);

    // Export a string as separate objects
    export_string(&font, "ABC", "text_ABC.obj", 5.0);

    println!("\n✓ Export complete!");
}

fn export_2d_glyph(font: &Font, c: char, filename: &str) {
    let mesh = font
        .glyph_to_mesh_2d(c, Quality::High)
        .expect("Failed to generate 2D mesh");

    let mut file = File::create(filename).expect("Failed to create file");

    writeln!(file, "# Fontmesh 2D glyph: '{}'", c).unwrap();
    writeln!(file, "# Vertices: {}", mesh.vertex_count()).unwrap();
    writeln!(file, "# Triangles: {}", mesh.triangle_count()).unwrap();
    writeln!(file).unwrap();

    // Write vertices
    for vertex in &mesh.vertices {
        writeln!(file, "v {} {} 0.0", vertex.x, vertex.y).unwrap();
    }

    writeln!(file).unwrap();

    // Write faces (OBJ indices are 1-based)
    for chunk in mesh.indices.chunks(3) {
        writeln!(file, "f {} {} {}", chunk[0] + 1, chunk[1] + 1, chunk[2] + 1).unwrap();
    }

    println!("  Exported 2D glyph '{}' -> {}", c, filename);
}

fn export_3d_glyph(font: &Font, c: char, filename: &str, depth: f32) {
    let mesh = font
        .glyph_to_mesh_3d(c, Quality::High, depth)
        .expect("Failed to generate 3D mesh");

    let mut file = File::create(filename).expect("Failed to create file");

    writeln!(file, "# Fontmesh 3D glyph: '{}'", c).unwrap();
    writeln!(file, "# Vertices: {}", mesh.vertex_count()).unwrap();
    writeln!(file, "# Triangles: {}", mesh.triangle_count()).unwrap();
    writeln!(file, "# Depth: {}", depth).unwrap();
    writeln!(file).unwrap();

    // Write vertices
    for vertex in &mesh.vertices {
        writeln!(file, "v {} {} {}", vertex.x, vertex.y, vertex.z).unwrap();
    }

    writeln!(file).unwrap();

    // Write normals
    for normal in &mesh.normals {
        writeln!(file, "vn {} {} {}", normal.x, normal.y, normal.z).unwrap();
    }

    writeln!(file).unwrap();

    // Write faces with normals (OBJ indices are 1-based)
    for chunk in mesh.indices.chunks(3) {
        writeln!(
            file,
            "f {0}//{0} {1}//{1} {2}//{2}",
            chunk[0] + 1,
            chunk[1] + 1,
            chunk[2] + 1
        )
        .unwrap();
    }

    println!("  Exported 3D glyph '{}' -> {}", c, filename);
}

fn export_string(font: &Font, text: &str, filename: &str, depth: f32) {
    let mut file = File::create(filename).expect("Failed to create file");

    writeln!(file, "# Fontmesh string export: '{}'", text).unwrap();
    writeln!(file, "# Depth: {}", depth).unwrap();
    writeln!(file).unwrap();

    let mut vertex_offset = 0u32;
    let mut x_position = 0.0f32;

    for c in text.chars() {
        let glyph = font.glyph_by_char(c).expect("Glyph not found");
        let mesh = font
            .glyph_to_mesh_3d(c, Quality::Normal, depth)
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

        vertex_offset += mesh.vertex_count() as u32;
        x_position += glyph.advance * 1.2; // Add some spacing
    }

    println!("  Exported string '{}' -> {}", text, filename);
}
