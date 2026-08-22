//! Font-backed text/math layout substrate (Step 7 item 3; Step 8 simple ruby).
//!
//! Shared face/shaping data with two engines: JLReq Profile v1 and Math Profile v1.
//! `reciplexa-std` heuristics remain the explicit stub/reference path.

#![forbid(unsafe_code)]

pub mod bou;
pub mod bidi;
pub mod engine;
pub mod error;
pub mod fallback;
pub mod fixture;
pub mod font;
pub mod ja;
pub mod ja_tables;
pub mod math_layout;
pub mod policy;
pub mod position;
pub mod protocol;
pub mod ruby;
pub mod shape;
pub mod tcy;
pub mod vert;

pub use bou::{
    layout_bou_horizontal, layout_bou_vertical, positioned_bou_horizontal_to_shapes,
    positioned_bou_vertical_to_shapes, PositionedBouHorizontal, PositionedBouVertical,
    BOU_PARENT_GAP_EM,
};
pub use bidi::{analyze_paragraph, visual_glyph_indices, visual_positions_em, BidiSegment};
pub use engine::{host_typeset_engine, TypesetEngine};
pub use error::LayoutError;
pub use fallback::{register_run_fonts, FontFallbackChain};
pub use fixture::{
    build_fallback_only_font_bytes, fixture_font_bytes, FIXTURE_AXIS_HEIGHT, FIXTURE_FRACTION_RULE_THICKNESS,
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
pub use protocol::{infer_script, Script, ShapingAttributes};
pub use position::{
    glyph_run_from_shaped, positioned_line_to_glyph_shapes, positioned_line_to_glyph_texts,
    positioned_line_to_scene_text, positioned_lines_to_shapes, positioned_math_to_shapes,
    productize_document_text, productize_shape_text, Direction, GlyphRun, PositionedGlyph,
    PositionedLine, WritingMode,
};
pub use ruby::{layout_simple_ruby, positioned_ruby_to_shapes, PositionedRuby, RUBY_PARENT_GAP_EM};
pub use shape::{clusters_cover_input, shape_run, ShapedGlyph, ShapedRun};
pub use tcy::{
    layout_tate_chu_yoko, positioned_tcy_to_shapes, PositionedTateChuYoko, TCY_MAX_CHARS,
};
pub use vert::{
    layout_vertical_run, positioned_vertical_to_shapes, vert_substitute_gid, PositionedVertical,
};
