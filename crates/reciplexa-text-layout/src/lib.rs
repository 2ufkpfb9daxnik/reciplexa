//! Font-backed text/math layout substrate (Step 7 item 3).
//!
//! Shared face/shaping data with two engines: JLReq Profile v1 and Math Profile v1.
//! `reciplexa-std` heuristics remain the explicit stub/reference path.

#![forbid(unsafe_code)]

pub mod engine;
pub mod error;
pub mod fixture;
pub mod font;
pub mod ja;
pub mod ja_tables;
pub mod math_layout;
pub mod policy;
pub mod position;
pub mod shape;

pub use engine::{host_typeset_engine, TypesetEngine};
pub use error::LayoutError;
pub use fixture::{
    fixture_font_bytes, FIXTURE_AXIS_HEIGHT, FIXTURE_FRACTION_RULE_THICKNESS,
    FIXTURE_PAREN_VARIANT_ADVANCE, FIXTURE_SCRIPT_PERCENT_SCALE_DOWN,
    FIXTURE_SCRIPT_SCRIPT_PERCENT_SCALE_DOWN, FIXTURE_TALL_PAREN_VARIANT_ADVANCE,
};
pub use font::{content_digest, host_math_font, host_product_font, FontId, LoadedFont};
pub use ja::{
    break_line_font, glyph_advance_em, justify_line_font, layout_column_paragraph_product,
    layout_wrapped_paragraph_product, layout_wrapped_paragraph_product_lines, measure_run_em,
    BreakReason, LineSegment, JLREQ_PROFILE_V1, JLREQ_PROFILE_V1_TABLES,
};
pub use ja_tables::pair_matrix_fingerprint;
pub use math_layout::{
    layout_math_atom, MathConstantsEm, MathRule, PositionedMath, PositionedMathGlyph,
    MATH_PROFILE_V1,
};
pub use policy::{select_font, FontRegistry};
pub use position::{
    glyph_run_from_shaped, positioned_line_to_glyph_shapes, positioned_line_to_glyph_texts,
    positioned_line_to_scene_text, positioned_lines_to_shapes, positioned_math_to_shapes,
    Direction, GlyphRun, PositionedGlyph, PositionedLine, WritingMode,
};
pub use shape::{clusters_cover_input, shape_run, ShapedGlyph, ShapedRun};
