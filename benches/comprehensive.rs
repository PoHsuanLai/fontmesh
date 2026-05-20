use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fontmesh::{
    glyph_id, glyph_to_mesh_2d, glyph_to_mesh_3d, parse_font, FontRef, GlyphId, GlyphMeshBuilder,
};

fn id(font: &FontRef, c: char) -> GlyphId {
    glyph_id(font, c).unwrap_or_else(|| panic!("font is missing '{c}'"))
}

fn bench_comprehensive(c: &mut Criterion) {
    let font_data = include_bytes!("../assets/test_font.ttf");
    let cursive_data = include_bytes!("../assets/test_font_cursive.ttf");
    let font = parse_font(font_data).unwrap();
    let cursive_font = parse_font(cursive_data).unwrap();

    // Resolve all the glyph IDs up front so we benchmark tessellation, not the
    // charmap lookup. This matches the previous bench's intent — the old API
    // hid the charmap behind `char_to_mesh_*` but never measured it
    // separately.
    let gid_i = id(&font, 'I');
    let gid_a = id(&font, 'A');
    let gid_at = id(&font, '@');
    let gid_a_cursive = id(&cursive_font, 'A');
    let hello_gids: Vec<GlyphId> = "HELLO".chars().map(|ch| id(&font, ch)).collect();
    let alphabet_gids: Vec<GlyphId> = ('A'..='Z').map(|ch| id(&font, ch)).collect();

    let mut group = c.benchmark_group("fontmesh_comprehensive");

    // === Glyph Complexity ===

    group.bench_function("simple_glyph_2d", |b| {
        b.iter(|| glyph_to_mesh_2d(&font, black_box(gid_i), 20));
    });

    group.bench_function("simple_glyph_3d", |b| {
        b.iter(|| glyph_to_mesh_3d(&font, black_box(gid_i), black_box(5.0), 20));
    });

    group.bench_function("medium_glyph_2d", |b| {
        b.iter(|| glyph_to_mesh_2d(&font, black_box(gid_a), 20));
    });

    group.bench_function("medium_glyph_3d", |b| {
        b.iter(|| glyph_to_mesh_3d(&font, black_box(gid_a), black_box(5.0), 20));
    });

    group.bench_function("complex_glyph_2d", |b| {
        b.iter(|| glyph_to_mesh_2d(&font, black_box(gid_at), 20));
    });

    group.bench_function("complex_glyph_3d", |b| {
        b.iter(|| glyph_to_mesh_3d(&font, black_box(gid_at), black_box(5.0), 20));
    });

    group.bench_function("cursive_glyph_2d", |b| {
        b.iter(|| {
            GlyphMeshBuilder::new(&cursive_font, black_box(gid_a_cursive))
                .with_subdivisions(black_box(50))
                .to_mesh_2d()
        });
    });

    group.bench_function("cursive_glyph_3d", |b| {
        b.iter(|| {
            GlyphMeshBuilder::new(&cursive_font, black_box(gid_a_cursive))
                .with_subdivisions(black_box(50))
                .to_mesh_3d(black_box(5.0))
        });
    });

    // === Quality Levels ===

    for subdivisions in [5u8, 20, 50] {
        group.bench_with_input(
            BenchmarkId::new("quality", subdivisions),
            &subdivisions,
            |b, &subdivisions| {
                b.iter(|| {
                    GlyphMeshBuilder::new(&font, black_box(gid_at))
                        .with_subdivisions(black_box(subdivisions))
                        .to_mesh_3d(black_box(5.0))
                });
            },
        );
    }

    // === Batch Processing (Real-world) ===

    group.bench_function("batch_word_2d", |b| {
        b.iter(|| {
            for &gid in &hello_gids {
                let _ = glyph_to_mesh_2d(&font, black_box(gid), 20);
            }
        });
    });

    group.bench_function("batch_word_3d", |b| {
        b.iter(|| {
            for &gid in &hello_gids {
                let _ = glyph_to_mesh_3d(&font, black_box(gid), black_box(5.0), 20);
            }
        });
    });

    group.bench_function("batch_alphabet_2d", |b| {
        b.iter(|| {
            for &gid in &alphabet_gids {
                let _ = glyph_to_mesh_2d(&font, black_box(gid), 20);
            }
        });
    });

    // === Pipeline Stages ===

    group.bench_function("stage_linearize", |b| {
        b.iter(|| {
            GlyphMeshBuilder::new(&font, black_box(gid_at))
                .with_subdivisions(black_box(20))
                .to_outline()
        });
    });

    let outline = GlyphMeshBuilder::new(&font, gid_at)
        .with_subdivisions(20)
        .to_outline()
        .unwrap();

    group.bench_function("stage_triangulate", |b| {
        b.iter(|| black_box(&outline).triangulate());
    });

    let mesh_2d = outline.triangulate().unwrap();

    group.bench_function("stage_extrude", |b| {
        b.iter(|| black_box(&mesh_2d).extrude(black_box(&outline), black_box(5.0)));
    });

    group.finish();
}

criterion_group!(benches, bench_comprehensive);
criterion_main!(benches);
