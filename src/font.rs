//! Font parsing utilities
//!
//! In 0.5 the canonical parsed-font handle is [`skrifa::FontRef`]. This module
//! exposes a small set of helpers around it so that callers don't need to
//! reach into skrifa for routine work (charmap lookups, em-normalized metrics,
//! advance widths).

use crate::error::{FontMeshError, Result};
use skrifa::{
    instance::{LocationRef, Size},
    metrics::Metrics,
    FontRef, GlyphId, MetadataProvider,
};

/// Parse font data into a skrifa `FontRef`.
///
/// Convenience wrapper around `FontRef::from_index(data, 0)`.
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
#[inline]
pub fn glyph_id(font: &FontRef, character: char) -> Option<GlyphId> {
    font.charmap().map(character)
}

/// Font ascender, normalised to 1.0 em.
#[inline]
pub fn ascender(font: &FontRef) -> f32 {
    metrics(font).ascent * em_scale(font)
}

/// Font descender, normalised to 1.0 em (typically negative).
#[inline]
pub fn descender(font: &FontRef) -> f32 {
    metrics(font).descent * em_scale(font)
}

/// Recommended line gap, normalised to 1.0 em.
#[inline]
pub fn line_gap(font: &FontRef) -> f32 {
    metrics(font).leading * em_scale(font)
}

/// Horizontal advance width for a glyph id, normalised to 1.0 em.
///
/// Returns `None` if the glyph id has no advance metric.
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
#[inline]
pub fn glyph_advance(font: &FontRef, character: char) -> Option<f32> {
    advance(font, glyph_id(font, character)?)
}
