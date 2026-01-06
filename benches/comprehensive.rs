use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fontmesh::glyph::Glyph;
use fontmesh::{char_to_mesh_2d, char_to_mesh_3d, Face};

// Comprehensive benchmark covering all important use cases
fn bench_comprehensive(c: &mut Criterion) {
    let font_data = include_bytes!("../assets/test_font.ttf");
    let cursive_data = include_bytes!("../assets/test_font_cursive.ttf");
    let face = Face::parse(font_data, 0).unwrap();
    let cursive_face = Face::parse(cursive_data, 0).unwrap();

    let mut group = c.benchmark_group("fontmesh_comprehensive");

    // === Glyph Complexity ===

    // Simple glyphs
    group.bench_function("simple_glyph_2d", |b| {
        b.iter(|| char_to_mesh_2d(&face, black_box('I'), 20));
    });

    group.bench_function("simple_glyph_3d", |b| {
        b.iter(|| char_to_mesh_3d(&face, black_box('I'), black_box(5.0), 20));
    });

    // Medium complexity
    group.bench_function("medium_glyph_2d", |b| {
        b.iter(|| char_to_mesh_2d(&face, black_box('A'), 20));
    });

    group.bench_function("medium_glyph_3d", |b| {
        b.iter(|| char_to_mesh_3d(&face, black_box('A'), black_box(5.0), 20));
    });

    // Complex glyphs
    group.bench_function("complex_glyph_2d", |b| {
        b.iter(|| char_to_mesh_2d(&face, black_box('@'), 20));
    });

    group.bench_function("complex_glyph_3d", |b| {
        b.iter(|| char_to_mesh_3d(&face, black_box('@'), black_box(5.0), 20));
    });

    // Cursive font (very complex)
    group.bench_function("cursive_glyph_2d", |b| {
        b.iter(|| {
            Glyph::new(&cursive_face, black_box('A'))
                .unwrap()
                .with_subdivisions(black_box(50))
                .to_mesh_2d()
        });
    });

    group.bench_function("cursive_glyph_3d", |b| {
        b.iter(|| {
            Glyph::new(&cursive_face, black_box('A'))
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
                    Glyph::new(&face, black_box('@'))
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
                let _ = char_to_mesh_2d(&face, black_box(ch), 20);
            }
        });
    });

    group.bench_function("batch_word_3d", |b| {
        let word = "HELLO";
        b.iter(|| {
            for ch in word.chars() {
                let _ = char_to_mesh_3d(&face, black_box(ch), black_box(5.0), 20);
            }
        });
    });

    group.bench_function("batch_alphabet_2d", |b| {
        b.iter(|| {
            for ch in 'A'..='Z' {
                let _ = char_to_mesh_2d(&face, black_box(ch), 20);
            }
        });
    });

    // === Pipeline Stages ===

    group.bench_function("stage_outline", |b| {
        b.iter(|| Glyph::new(&face, black_box('@')).unwrap().outline());
    });

    group.bench_function("stage_linearize", |b| {
        b.iter(|| {
            Glyph::new(&face, black_box('@'))
                .unwrap()
                .with_subdivisions(black_box(20))
                .to_outline()
        });
    });

    let outline = Glyph::new(&face, '@')
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
