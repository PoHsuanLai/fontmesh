# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
