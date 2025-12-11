//! Export complex characters to OBJ format
//! Demonstrates mesh generation for intricate glyphs

use fontmesh::Font;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CJK Characters ===\n");

    // Try to load a font with CJK support, fallback to test font
    let font_data = std::fs::read("/System/Library/Fonts/PingFang.ttc")
        .or_else(|_| std::fs::read("/System/Library/Fonts/STHeiti Medium.ttc"))
        .unwrap_or_else(|_| include_bytes!("test_font.ttf").to_vec());

    let font = Font::from_bytes(&font_data)?;

    // CJK and complex Latin characters
    let characters = vec![
        ('愛', "love_chinese"),   // Chinese: love
        ('龍', "dragon_chinese"), // Chinese: dragon
        ('@', "at_sign"),         // Complex Latin
        ('&', "ampersand"),       // Complex Latin
    ];

    for (ch, name) in characters {
        if let Ok(glyph) = font.glyph_by_char(ch) {
            println!("Exporting '{}' ({})...", ch, name);

            let mesh = glyph.with_subdivisions(30).to_mesh_3d(5.0)?;

            let filename = format!("{}.obj", name);
            let mut file = File::create(&filename)?;

            writeln!(file, "# Character: '{}' ({})", ch, name)?;
            writeln!(file, "# Vertices: {}", mesh.vertices.len())?;
            writeln!(file, "# Triangles: {}", mesh.triangle_count())?;
            writeln!(file)?;

            // Write vertices
            for vertex in &mesh.vertices {
                writeln!(file, "v {} {} {}", vertex.x, vertex.y, vertex.z)?;
            }

            // Write normals
            for normal in &mesh.normals {
                writeln!(file, "vn {} {} {}", normal.x, normal.y, normal.z)?;
            }

            // Write faces
            for chunk in mesh.indices.chunks(3) {
                writeln!(
                    file,
                    "f {0}//{0} {1}//{1} {2}//{2}",
                    chunk[0] + 1,
                    chunk[1] + 1,
                    chunk[2] + 1
                )?;
            }

            println!(
                "  ✓ Exported to {}: {} vertices, {} triangles\n",
                filename,
                mesh.vertices.len(),
                mesh.triangle_count()
            );
        } else {
            println!("  ✗ Character '{}' not found in font\n", ch);
        }
    }

    println!("\n=== Cursive Font ===\n");

    // Test cursive font
    let cursive_font = Font::from_bytes(include_bytes!("test_font_cursive.ttf"))?;

    let cursive_chars = vec![
        ('A', "cursive_a"),
        ('B', "cursive_b"),
        ('Q', "cursive_q"),
        ('&', "cursive_ampersand"),
    ];

    for (ch, name) in cursive_chars {
        if let Ok(glyph) = cursive_font.glyph_by_char(ch) {
            println!("Exporting '{}' ({})...", ch, name);

            let mesh = glyph.with_subdivisions(50).to_mesh_3d(5.0)?; // Higher quality for cursive

            let filename = format!("{}.obj", name);
            let mut file = File::create(&filename)?;

            writeln!(file, "# CJK character: '{}' ({})", ch, name)?;
            writeln!(file, "# Vertices: {}", mesh.vertices.len())?;
            writeln!(file, "# Triangles: {}", mesh.triangle_count())?;
            writeln!(file)?;

            // Write vertices
            for vertex in &mesh.vertices {
                writeln!(file, "v {} {} {}", vertex.x, vertex.y, vertex.z)?;
            }

            // Write normals
            for normal in &mesh.normals {
                writeln!(file, "vn {} {} {}", normal.x, normal.y, normal.z)?;
            }

            // Write faces
            for chunk in mesh.indices.chunks(3) {
                writeln!(
                    file,
                    "f {0}//{0} {1}//{1} {2}//{2}",
                    chunk[0] + 1,
                    chunk[1] + 1,
                    chunk[2] + 1
                )?;
            }

            println!(
                "  ✓ Exported to {}: {} vertices, {} triangles\n",
                filename,
                mesh.vertices.len(),
                mesh.triangle_count()
            );
        } else {
            println!("  ✗ Character '{}' not found in font\n", ch);
        }
    }

    Ok(())
}
