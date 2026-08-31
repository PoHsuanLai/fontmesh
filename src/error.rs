//! Error types returned by fontmesh operations.
//!
//! Most fallible entry points return [`crate::Result`], which is an alias for
//! `Result<T, FontMeshError>`. Character lookups via [`crate::glyph_id`] return
//! `Option` instead of an error — a missing mapping is not exceptional.
//!
//! ```
//! use fontmesh::{parse_font, FontMeshError};
//!
//! assert!(matches!(parse_font(&[]), Err(FontMeshError::ParseError(_))));
//! ```

use std::fmt;

/// Errors that can occur during font parsing and mesh generation.
///
/// ```
/// use fontmesh::{parse_font, FontMeshError};
///
/// match parse_font(b"not a font") {
///     Err(FontMeshError::ParseError(msg)) => assert!(!msg.is_empty()),
///     Err(other) => panic!("expected ParseError, got {other:?}"),
///     Ok(_) => panic!("expected an error"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum FontMeshError {
    /// Failed to parse the font file.
    ///
    /// Returned by [`crate::parse_font`] when `skrifa` cannot read the bytes as
    /// a TrueType or OpenType font.
    ParseError(String),

    /// Failed to extract a glyph outline from the font.
    ///
    /// Typically means the glyph id has no `glyf`/`CFF` outline entry.
    OutlineExtractionFailed(String),

    /// Failed to triangulate a linearized outline.
    ///
    /// Returned for an empty outline, or if `lyon_tessellation` rejects the
    /// path (self-intersections that the fill tessellator cannot handle).
    TriangulationFailed(String),

    /// Failed to extrude a 2D mesh into 3D.
    ///
    /// Currently raised when `depth` is not a finite `f32` (`NaN` or ±∞).
    ExtrusionFailed(String),

    /// `subdivisions` was 0.
    ///
    /// Curve linearization needs at least one sample; zero would produce a
    /// degenerate mesh, so the call is rejected instead.
    InvalidQuality(u8),

    /// The glyph has no outline (whitespace, control characters, `.notdef`
    /// without contours, and similar).
    NoOutline,
}

impl fmt::Display for FontMeshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError(msg) => write!(f, "Font parse error: {}", msg),
            Self::OutlineExtractionFailed(msg) => write!(f, "Outline extraction failed: {}", msg),
            Self::TriangulationFailed(msg) => write!(f, "Triangulation failed: {}", msg),
            Self::ExtrusionFailed(msg) => write!(f, "Extrusion failed: {}", msg),
            Self::InvalidQuality(q) => write!(f, "Invalid quality parameter: {}", q),
            Self::NoOutline => write!(f, "Glyph has no outline"),
        }
    }
}

impl std::error::Error for FontMeshError {}

/// Result type for fontmesh operations.
///
/// Equivalent to `std::result::Result<T, FontMeshError>`.
///
/// ```
/// fn load(bytes: &[u8]) -> fontmesh::Result<fontmesh::FontRef<'_>> {
///     fontmesh::parse_font(bytes)
/// }
///
/// assert!(load(&[]).is_err());
/// ```
pub type Result<T> = std::result::Result<T, FontMeshError>;
