//! # fontmesh
//!
//! Convert font glyphs to 2D and 3D triangle meshes.
//!
//! This crate provides a small, stateless API for tessellating glyph outlines
//! into triangle meshes. Font parsing is handled by [`skrifa`], so both
//! TrueType (`glyf`) and CFF/PostScript (`CFF`/`CFF2`) outlines are supported,
//! and the same parsed-font handle plugs straight into shaping libraries like
//! `cosmic-text`.
//!
//! ## Quick start
//!
//! ```ignore
//! use fontmesh::{parse_font, glyph_id, glyph_to_mesh_3d};
//!
//! let font_data = include_bytes!("path/to/font.ttf");
//! let font = parse_font(font_data)?;
//! let gid = glyph_id(&font, 'A').expect("font has 'A'");
//! let mesh = glyph_to_mesh_3d(&font, gid, 0.1, 20)?;
//! ```
//!
//! ## API surface
//!
//! - Parsing: [`parse_font`] returns a [`skrifa::FontRef`]. It's also exposed
//!   directly as [`FontRef`] if you want to skip the wrapper.
//! - Glyph IDs: [`glyph_id`] resolves a `char` → [`GlyphId`]. If you're
//!   driving fontmesh from a text shaper you already have these IDs and can
//!   skip the lookup.
//! - Meshes: [`glyph_to_mesh_2d`] / [`glyph_to_mesh_3d`] or the fluent
//!   [`GlyphMeshBuilder`].
//! - Metrics: [`ascender`], [`descender`], [`line_gap`], [`advance`],
//!   [`glyph_advance`] — all normalised to 1.0 em.
//!
//! ## Pipeline
//!
//! 1. Parse the font (`FontRef::from_index`)
//! 2. Extract the glyph outline (skrifa `OutlinePen`)
//! 3. Linearise curves ([`linearize_outline`])
//! 4. Triangulate ([`triangulate`])
//! 5. Optionally extrude to 3D ([`extrude`])
//!
//! Each stage is exposed publicly so advanced callers can plug their own
//! glyph source in at step 3.

pub mod error;
pub mod extrude;
pub mod font;
pub mod glyph;
pub mod linearize;
pub mod triangulate;
pub mod types;

pub use error::{FontMeshError, Result};
pub use extrude::{compute_smooth_normals, extrude};
pub use font::{advance, ascender, descender, glyph_advance, glyph_id, line_gap, parse_font};
pub use glyph::{glyph_to_mesh_2d, glyph_to_mesh_3d, GlyphMeshBuilder};
pub use linearize::linearize_outline;
pub use skrifa::{FontRef, GlyphId, MetadataProvider};
pub use triangulate::triangulate;
pub use types::{Mesh2D, Mesh3D, Outline2D};
