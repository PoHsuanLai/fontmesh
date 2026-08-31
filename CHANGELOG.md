# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2026-08-31

### Added

- Rustdoc on every public item, with runnable examples. docs.rs coverage is now 100% documented and 100% with examples (was 61% / 0%).

### Removed - BREAKING

- `FontMeshError::GlyphNotFound` — never constructed; missing characters already return `None` from `glyph_id`.
- `FontMeshError::LinearizationFailed` — `linearize_outline` never returned it.

Exhaustive `match`es on `FontMeshError` need those two arms dropped.

## [0.5.0] - 2026-05-21

### Changed - BREAKING

- **Font parsing now uses [`skrifa`](https://crates.io/crates/skrifa)** instead of `ttf-parser`. `skrifa` is the Google Fonts–maintained outline library that backs `cosmic-text` and other Rust shapers, so the same parsed `FontRef` plugs straight in.
- **Glyph-id based API**. The mesh entry points are now keyed by `skrifa::GlyphId` instead of `char`:
  - `char_to_mesh_2d(face, ch, subdivisions)` → `glyph_to_mesh_2d(font, glyph_id, subdivisions)`
  - `char_to_mesh_3d(face, ch, depth, subdivisions)` → `glyph_to_mesh_3d(font, glyph_id, depth, subdivisions)`
  - `GlyphMeshBuilder::new(face, ch)` → `GlyphMeshBuilder::new(font, glyph_id)`
  Use the new helper `glyph_id(&font, ch)` to resolve a `char` to a `GlyphId`. Callers that drive fontmesh from a shaper already have glyph ids and skip the lookup entirely.
- **`Face` is gone**; the parsed type is `skrifa::FontRef`. Parse via `parse_font(data)` (returns `FontRef<'_>`) or build one directly with `FontRef::from_index`.
- **Metrics helpers** (`ascender`, `descender`, `line_gap`, `advance`, `glyph_advance`) now take a `&FontRef` and return em-normalised `f32` values (1 em = 1.0). Previous versions returned raw font units.

### Added

- **CFF / PostScript outline support.** OpenType `.otf` fonts with CFF or CFF2 outlines now produce meshes. Cubic Béziers are linearised via a new `linearize_cbezier` pass.
- **Per-contour winding detection** for side-face generation. Side normals now compute correctly for both TrueType (outer CW) and CFF (outer CCW) fonts; previously the side-face fix in 0.4.1 only worked for one winding.
- `advance(font, glyph_id) -> Option<f32>` exposes em-normalised advance width by glyph id (useful when driving fontmesh from a shaper).

### Migration Guide

**Before (0.4):**
```rust
use fontmesh::{Face, char_to_mesh_3d};

let face = Face::parse(font_data, 0)?;
let mesh = char_to_mesh_3d(&face, 'A', 0.1, 20)?;
```

**After (0.5):**
```rust
use fontmesh::{parse_font, glyph_id, glyph_to_mesh_3d};

let font = parse_font(font_data)?;
let gid = glyph_id(&font, 'A').expect("font contains 'A'");
let mesh = glyph_to_mesh_3d(&font, gid, 0.1, 20)?;
```

## [0.4.1] - 2026-03-02

### Fixed

- **Side face normals**: 3D extruded glyphs now correctly show side faces when rendered with standard back-face culling. Previously the side face normals pointed inward instead of outward, causing them to be culled by the renderer.

## [0.4.0] - 2026-02-26

### Changed - BREAKING

- **Removed `Font` struct** - The library now uses pure functions instead of a stateful `Font` struct
- **New API**: Use `char_to_mesh_2d(&face, char, subdivisions)` and `char_to_mesh_3d(&face, char, depth, subdivisions)` instead of `font.glyph_to_mesh_*()`
- **Direct `Face` usage**: Work directly with `ttf_parser::Face` - parse with `Face::parse(data, 0)` or use the convenience helper `parse_font(data)`
- **Font metrics**: Use helper functions `ascender(&face)`, `descender(&face)`, `line_gap(&face)`, `glyph_advance(&face, char)` instead of methods

### Migration Guide

**Before (0.3.3):**
```rust
let font = Font::from_bytes(font_data)?;
let mesh = font.glyph_to_mesh_3d('A', 5.0)?;
```

**After (0.4.0):**
```rust
let face = Face::parse(font_data, 0)?;
let mesh = char_to_mesh_3d(&face, 'A', 5.0, 20)?;
```

### Added

- Parameter validation: `subdivisions = 0` now returns `FontMeshError::InvalidQuality` instead of silently producing degenerate meshes
- Parameter validation: non-finite `depth` (NaN, infinity) now returns `FontMeshError::ExtrusionFailed` instead of silently producing invalid vertices

### Benefits

- No hidden state - pure functions only
- User controls parsing and caching strategy
- Simpler API - fewer types to learn
- Better integration with existing `ttf_parser` workflows

## [0.3.3] - Previous

- Previous stable release
