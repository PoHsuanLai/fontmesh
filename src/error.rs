//! Error types for fontmesh

use std::fmt;

/// Errors that can occur during font mesh generation
#[derive(Debug, Clone, PartialEq)]
pub enum FontMeshError {
    /// Failed to parse the font file
    ParseError(String),

    /// Glyph not found for the given character
    GlyphNotFound(char),

    /// Failed to extract glyph outline
    OutlineExtractionFailed(String),

    /// Failed to linearize curves
    LinearizationFailed(String),

    /// Failed to triangulate the outline
    TriangulationFailed(String),

    /// Failed to extrude the mesh
    ExtrusionFailed(String),

    /// Invalid quality parameter
    InvalidQuality(u8),

    /// The glyph has no outline (e.g., space character)
    NoOutline,
}

impl fmt::Display for FontMeshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError(msg) => write!(f, "Font parse error: {}", msg),
            Self::GlyphNotFound(c) => write!(f, "Glyph not found for character: '{}'", c),
            Self::OutlineExtractionFailed(msg) => write!(f, "Outline extraction failed: {}", msg),
            Self::LinearizationFailed(msg) => write!(f, "Linearization failed: {}", msg),
            Self::TriangulationFailed(msg) => write!(f, "Triangulation failed: {}", msg),
            Self::ExtrusionFailed(msg) => write!(f, "Extrusion failed: {}", msg),
            Self::InvalidQuality(q) => write!(f, "Invalid quality parameter: {}", q),
            Self::NoOutline => write!(f, "Glyph has no outline"),
        }
    }
}

impl std::error::Error for FontMeshError {}

impl From<ttf_parser::FaceParsingError> for FontMeshError {
    fn from(err: ttf_parser::FaceParsingError) -> Self {
        Self::ParseError(format!("ttf_parser error: {}", err))
    }
}

/// Result type for fontmesh operations
pub type Result<T> = std::result::Result<T, FontMeshError>;
