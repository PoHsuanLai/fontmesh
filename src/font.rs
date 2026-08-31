//! Font parsing and em-normalised metrics.
//!
//! In 0.5 the canonical parsed-font handle is [`skrifa::FontRef`]. This module
//! exposes a small set of helpers around it so that callers don't need to
//! reach into skrifa for routine work (charmap lookups, em-normalized metrics,
//! advance widths).
//!
//! All metric helpers return values in **em units** (1 em = 1.0), independent
//! of the font's `unitsPerEm`.
//!
//! ```
//! use fontmesh::{parse_font, glyph_id, glyph_advance};
//!
//! # let data = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"));
//! let font = parse_font(data)?;
//! let gid = glyph_id(&font, 'A').unwrap();
//! let width = glyph_advance(&font, 'A').unwrap();
//! assert!(width > 0.0);
//! let _ = gid;
//! # Ok::<(), fontmesh::FontMeshError>(())
//! ```

use crate::error::{FontMeshError, Result};
use skrifa::{
    instance::{LocationRef, Size},
    metrics::Metrics,
    FontRef, GlyphId, MetadataProvider,
};

/// Parse font data into a skrifa [`FontRef`].
///
/// Convenience wrapper around [`FontRef::from_index`]`(data, 0)` — the first
/// font in a collection (`.ttc`) is used.
///
/// # Errors
///
/// Returns [`FontMeshError::ParseError`] if the bytes are not a valid
/// TrueType or OpenType font.
///
/// ```
/// use fontmesh::parse_font;
///
/// # let data = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"));
/// let font = parse_font(data)?;
/// assert!(fontmesh::glyph_id(&font, 'A').is_some());
/// # Ok::<(), fontmesh::FontMeshError>(())
/// ```
pub fn parse_font(data: &[u8]) -> Result<FontRef<'_>> {
    FontRef::from_index(data, 0)
        .map_err(|e| FontMeshError::ParseError(format!("Failed to parse font: {e:?}")))
}

#[inline]
fn em_scale(font: &FontRef) -> f32 {
    let upem = font
        .metrics(Size::unscaled(), LocationRef::default())
        .units_per_em
        .max(1);
    1.0 / upem as f32
}

#[inline]
fn metrics(font: &FontRef) -> Metrics {
    font.metrics(Size::unscaled(), LocationRef::default())
}

/// Map a character to its glyph id in this font, if any.
///
/// Returns `None` when the font has no cmap entry for `character`. Callers
/// that already have a [`GlyphId`] from a shaper can skip this lookup.
///
/// ```
/// # let font = fontmesh::parse_font(include_bytes!(concat!(
/// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
/// assert!(fontmesh::glyph_id(&font, 'A').is_some());
/// assert!(fontmesh::glyph_id(&font, '\u{FFFF}').is_none());
/// ```
#[inline]
pub fn glyph_id(font: &FontRef, character: char) -> Option<GlyphId> {
    font.charmap().map(character)
}

/// Font ascender, normalised to 1.0 em.
///
/// Typically a positive value around 0.8–1.0.
///
/// ```
/// # let font = fontmesh::parse_font(include_bytes!(concat!(
/// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
/// assert!(fontmesh::ascender(&font) > 0.0);
/// ```
#[inline]
pub fn ascender(font: &FontRef) -> f32 {
    metrics(font).ascent * em_scale(font)
}

/// Font descender, normalised to 1.0 em (typically negative).
///
/// ```
/// # let font = fontmesh::parse_font(include_bytes!(concat!(
/// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
/// assert!(fontmesh::descender(&font) < 0.0);
/// ```
#[inline]
pub fn descender(font: &FontRef) -> f32 {
    metrics(font).descent * em_scale(font)
}

/// Recommended line gap, normalised to 1.0 em.
///
/// This is the extra spacing between the previous row's descender and the
/// next row's ascender (`hhea.lineGap` / `OS/2.sTypoLineGap`).
///
/// ```
/// # let font = fontmesh::parse_font(include_bytes!(concat!(
/// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
/// let _gap = fontmesh::line_gap(&font);
/// ```
#[inline]
pub fn line_gap(font: &FontRef) -> f32 {
    metrics(font).leading * em_scale(font)
}

/// Horizontal advance width for a glyph id, normalised to 1.0 em.
///
/// Returns `None` if the glyph id has no advance metric.
///
/// ```
/// # let font = fontmesh::parse_font(include_bytes!(concat!(
/// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
/// let gid = fontmesh::glyph_id(&font, 'A').unwrap();
/// assert!(fontmesh::advance(&font, gid).unwrap() > 0.0);
/// ```
#[inline]
pub fn advance(font: &FontRef, glyph_id: GlyphId) -> Option<f32> {
    let scale = em_scale(font);
    font.glyph_metrics(Size::unscaled(), LocationRef::default())
        .advance_width(glyph_id)
        .map(|adv| adv * scale)
}

/// Advance width for a character, normalised to 1.0 em.
///
/// Resolves the character to a glyph id internally. Returns `None` if either
/// the lookup or the metric is missing.
///
/// ```
/// # let font = fontmesh::parse_font(include_bytes!(concat!(
/// #     env!("CARGO_MANIFEST_DIR"), "/assets/test_font.ttf"))).unwrap();
/// assert!(fontmesh::glyph_advance(&font, 'A').unwrap() > 0.0);
/// assert!(fontmesh::glyph_advance(&font, '\u{FFFF}').is_none());
/// ```
#[inline]
pub fn glyph_advance(font: &FontRef, character: char) -> Option<f32> {
    advance(font, glyph_id(font, character)?)
}
