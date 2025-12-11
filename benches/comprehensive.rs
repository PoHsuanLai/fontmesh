use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fontmesh::Font;

// Comprehensive benchmark covering all important use cases
fn bench_comprehensive(c: &mut Criterion) {
    let font_data = include_bytes!("../examples/test_font.ttf");
    let cursive_data = include_bytes!("../examples/test_font_cursive.ttf");
    let font = Font::from_bytes(font_data).unwrap();
    let cursive_font = Font::from_bytes(cursive_data).unwrap();

    let mut group = c.benchmark_group("fontmesh_comprehensive");

    // === Glyph Complexity ===

    // Simple glyphs
    group.bench_function("simple_glyph_2d", |b| {
        b.iter(|| font.glyph_to_mesh_2d(black_box('I')));
    });

    group.bench_function("simple_glyph_3d", |b| {
        b.iter(|| font.glyph_to_mesh_3d(black_box('I'), black_box(5.0)));
    });

    // Medium complexity
    group.bench_function("medium_glyph_2d", |b| {
        b.iter(|| font.glyph_to_mesh_2d(black_box('A')));
    });

    group.bench_function("medium_glyph_3d", |b| {
        b.iter(|| font.glyph_to_mesh_3d(black_box('A'), black_box(5.0)));
    });

    // Complex glyphs
    group.bench_function("complex_glyph_2d", |b| {
        b.iter(|| font.glyph_to_mesh_2d(black_box('@')));
    });

    group.bench_function("complex_glyph_3d", |b| {
        b.iter(|| font.glyph_to_mesh_3d(black_box('@'), black_box(5.0)));
    });

    // Cursive font (very complex)
    group.bench_function("cursive_glyph_2d", |b| {
        b.iter(|| {
            cursive_font
                .glyph_by_char(black_box('A'))
                .unwrap()
                .with_subdivisions(black_box(50))
                .to_mesh_2d()
        });
    });

    group.bench_function("cursive_glyph_3d", |b| {
        b.iter(|| {
            cursive_font
                .glyph_by_char(black_box('A'))
                .unwrap()
                .with_subdivisions(black_box(50))
                .to_mesh_3d(black_box(5.0))
        });
    });

    // === Quality Levels ===

    for subdivisions in [5, 20, 50] {
        group.bench_with_input(
            BenchmarkId::new("quality", subdivisions),
            &subdivisions,
            |b, &subdivisions| {
                b.iter(|| {
                    font.glyph_by_char(black_box('@'))
                        .unwrap()
                        .with_subdivisions(black_box(subdivisions))
                        .to_mesh_3d(black_box(5.0))
                });
            },
        );
    }

    // === Batch Processing (Real-world) ===

    group.bench_function("batch_word_2d", |b| {
        let word = "HELLO";
        b.iter(|| {
            for ch in word.chars() {
                let _ = font.glyph_to_mesh_2d(black_box(ch));
            }
        });
    });

    group.bench_function("batch_word_3d", |b| {
        let word = "HELLO";
        b.iter(|| {
            for ch in word.chars() {
                let _ = font.glyph_to_mesh_3d(black_box(ch), black_box(5.0));
            }
        });
    });

    group.bench_function("batch_alphabet_2d", |b| {
        b.iter(|| {
            for ch in 'A'..='Z' {
                let _ = font.glyph_to_mesh_2d(black_box(ch));
            }
        });
    });

    // === Pipeline Stages ===

    group.bench_function("stage_outline", |b| {
        b.iter(|| {
            font.glyph_by_char(black_box('@'))
                .unwrap()
                .outline()
        });
    });

    group.bench_function("stage_linearize", |b| {
        b.iter(|| {
            font.glyph_by_char(black_box('@'))
                .unwrap()
                .with_subdivisions(black_box(20))
                .to_outline()
        });
    });

    let outline = font
        .glyph_by_char('@')
        .unwrap()
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
