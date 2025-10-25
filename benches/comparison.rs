use criterion::{black_box, criterion_group, criterion_main, Criterion};
use meshtext::TextSection;

// Average glyph: 'e' (common letter with curves)
// Complex glyph: '@' (multiple holes/islands)

fn benchmark_fontmesh_simple_2d(c: &mut Criterion) {
    let font_data = include_bytes!("../examples/test_font.ttf");
    let font = fontmesh::Font::from_bytes(font_data).unwrap();

    c.bench_function("fontmesh: average 2d (e)", |b| {
        b.iter(|| font.glyph_to_mesh_2d(black_box('e'), fontmesh::Quality::Normal).unwrap())
    });
}

fn benchmark_fontmesh_complex_2d(c: &mut Criterion) {
    let font_data = include_bytes!("../examples/test_font.ttf");
    let font = fontmesh::Font::from_bytes(font_data).unwrap();

    c.bench_function("fontmesh: complex 2d (@)", |b| {
        b.iter(|| font.glyph_to_mesh_2d(black_box('@'), fontmesh::Quality::Normal).unwrap())
    });
}

fn benchmark_fontmesh_simple_3d(c: &mut Criterion) {
    let font_data = include_bytes!("../examples/test_font.ttf");
    let font = fontmesh::Font::from_bytes(font_data).unwrap();

    c.bench_function("fontmesh: average 3d (e)", |b| {
        b.iter(|| font.glyph_to_mesh_3d(black_box('e'), fontmesh::Quality::Normal, 0.1).unwrap())
    });
}

fn benchmark_fontmesh_complex_3d(c: &mut Criterion) {
    let font_data = include_bytes!("../examples/test_font.ttf");
    let font = fontmesh::Font::from_bytes(font_data).unwrap();

    c.bench_function("fontmesh: complex 3d (@)", |b| {
        b.iter(|| font.glyph_to_mesh_3d(black_box('@'), fontmesh::Quality::Normal, 0.1).unwrap())
    });
}

fn benchmark_ttf2mesh_simple_2d(c: &mut Criterion) {
    let font_data = include_bytes!("../examples/test_font.ttf");
    let mut font = ttf2mesh::TTFFile::from_buffer_vec(font_data.to_vec()).unwrap();

    c.bench_function("ttf2mesh: average 2d (e)", |b| {
        b.iter(|| {
            let mut glyph = font.glyph_from_char(black_box('e')).unwrap();
            glyph.to_2d_mesh(ttf2mesh::Quality::Medium)
        })
    });
}

fn benchmark_ttf2mesh_complex_2d(c: &mut Criterion) {
    let font_data = include_bytes!("../examples/test_font.ttf");
    let mut font = ttf2mesh::TTFFile::from_buffer_vec(font_data.to_vec()).unwrap();

    c.bench_function("ttf2mesh: complex 2d (@)", |b| {
        b.iter(|| {
            let mut glyph = font.glyph_from_char(black_box('@')).unwrap();
            glyph.to_2d_mesh(ttf2mesh::Quality::Medium)
        })
    });
}

fn benchmark_ttf2mesh_simple_3d(c: &mut Criterion) {
    let font_data = include_bytes!("../examples/test_font.ttf");
    let mut font = ttf2mesh::TTFFile::from_buffer_vec(font_data.to_vec()).unwrap();

    c.bench_function("ttf2mesh: average 3d (e)", |b| {
        b.iter(|| {
            let mut glyph = font.glyph_from_char(black_box('e')).unwrap();
            glyph.to_3d_mesh(ttf2mesh::Quality::Medium, 0.1)
        })
    });
}

fn benchmark_ttf2mesh_complex_3d(c: &mut Criterion) {
    let font_data = include_bytes!("../examples/test_font.ttf");
    let mut font = ttf2mesh::TTFFile::from_buffer_vec(font_data.to_vec()).unwrap();

    c.bench_function("ttf2mesh: complex 3d (@)", |b| {
        b.iter(|| {
            let mut glyph = font.glyph_from_char(black_box('@')).unwrap();
            glyph.to_3d_mesh(ttf2mesh::Quality::Medium, 0.1)
        })
    });
}

fn benchmark_meshtext_simple_2d(c: &mut Criterion) {
    let font_data = include_bytes!("../examples/test_font.ttf");

    c.bench_function("meshtext: average 2d (e)", |b| {
        b.iter(|| {
            let mut generator = meshtext::MeshGenerator::new(font_data);
            let text: meshtext::MeshText = generator
                .generate_section(black_box("e"), true, None)
                .unwrap();
            text
        })
    });
}

fn benchmark_meshtext_complex_2d(c: &mut Criterion) {
    let font_data = include_bytes!("../examples/test_font.ttf");

    c.bench_function("meshtext: complex 2d (@)", |b| {
        b.iter(|| {
            let mut generator = meshtext::MeshGenerator::new(font_data);
            let text: meshtext::MeshText = generator
                .generate_section(black_box("@"), true, None)
                .unwrap();
            text
        })
    });
}

fn benchmark_meshtext_simple_3d(c: &mut Criterion) {
    let font_data = include_bytes!("../examples/test_font.ttf");

    c.bench_function("meshtext: average 3d (e)", |b| {
        b.iter(|| {
            let mut generator = meshtext::MeshGenerator::new(font_data);
            let text: meshtext::MeshText = generator
                .generate_section(black_box("e"), false, None)
                .unwrap();
            text
        })
    });
}

fn benchmark_meshtext_complex_3d(c: &mut Criterion) {
    let font_data = include_bytes!("../examples/test_font.ttf");

    c.bench_function("meshtext: complex 3d (@)", |b| {
        b.iter(|| {
            let mut generator = meshtext::MeshGenerator::new(font_data);
            let text: meshtext::MeshText = generator
                .generate_section(black_box("@"), false, None)
                .unwrap();
            text
        })
    });
}

criterion_group!(
    benches,
    benchmark_fontmesh_simple_2d,
    benchmark_fontmesh_complex_2d,
    benchmark_fontmesh_simple_3d,
    benchmark_fontmesh_complex_3d,
    benchmark_ttf2mesh_simple_2d,
    benchmark_ttf2mesh_complex_2d,
    benchmark_ttf2mesh_simple_3d,
    benchmark_ttf2mesh_complex_3d,
    benchmark_meshtext_simple_2d,
    benchmark_meshtext_complex_2d,
    benchmark_meshtext_simple_3d,
    benchmark_meshtext_complex_3d
);
criterion_main!(benches);
