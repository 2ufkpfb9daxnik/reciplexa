//! Reference Client standard packages (`rpx.std.*`) — Phase 14.
//!
//! These types are a Strangler-friendly façade over scene / identity / motion
//! so Source, GUI, and Backend can share a small typed API before full Package
//! Manifest wiring lands.

#![forbid(unsafe_code)]

pub mod core;
pub mod document;
pub mod japanese;
pub mod layout;
pub mod math;
pub mod motion;
pub mod slide;
pub mod style;
pub mod text;
pub mod vector;
pub mod visual;

pub use core::{Angle, Color, Length, Point, Range, Rect, Size};
pub use document::{Block, Figure, Flow, Heading, List, ListItem, Page, Section, Table};
pub use japanese::{
    bou_estimate_box, bou_mark_offsets, break_line, break_line_to_text_shapes, break_line_vertical,
    break_opportunity, break_opportunity_chars, break_pair_matrix_cell, char_em_width,
    classify_char, hang_width_em, hang_width_em_char, indent_first_line, is_hangable,
    is_trimmable_line_end, is_trimmable_line_head, justify_line, justify_line_to_text_shapes,
    layout_column_paragraph_shapes, layout_wrapped_paragraph_shapes, lines_to_text_shapes,
    lines_to_vertical_text_shapes, measure_columns, needs_tate_rotation, place_lines_horizontal,
    place_lines_vertical, reciprocal_punctuation_mirror_em, reciprocal_punctuation_widths_em,
    ruby_estimate_box, tate_chu_yoko_estimate_box, trimming_width_em, trimming_width_em_char,
    vertical_advance_em, vertical_glyph_orientation, vertical_ruby_estimate_box,
    wrap_text_shape_content, BouBox, BreakOpportunity, CharClass, KihonHanmen,
    ParagraphSceneLayout, Ruby, RubyBox, RubyKind, TateChuYoko, TateChuYokoBox, TategakiParagraph,
    VerticalGlyphOrientation, VerticalRubyBox, WritingMode, BOU_MARK_SIZE_EM, BOU_SIDE_OFFSET_EM,
    BREAK_PAIR_MATRIX, BREAK_PAIR_MATRIX_DIM, DOC_TEXT_MAX_EM, HANG_WIDTH_EM, TRIMMING_WIDTH_EM,
    VERTICAL_RUBY_SIDE_EM,
};
pub use layout::{Align, Axis, Column, LayoutChild, Padding, Row, Stack};
pub use math::{
    accent_attachment_offset, accent_clearance_em, accent_clearance_em_named, accent_mark_glyph,
    aligned_column_x, bigop_limit_offsets, cases_brace_total_height_em, cases_column_align,
    class_spacing_em, delimiter_fence_extent_em, fraction_rule_metrics, layout_math_atom_to_shapes,
    layout_math_atom_to_shapes_with_style, matrix_cell_x_in_column, matrix_column_widths,
    phantom_box, radical_vinculum_index_offsets, scripts_attachment_offsets, smash_box,
    stackrel_spacing_offsets, underbrace_spacing, EstimateStyle, MathAccentKind, MathAtom, MathBox,
    MathClass, MathMatrixKind, MathStackKind, MatrixColumnAlign, ACCENT_CLEARANCE_EM,
    ACCENT_UNDER_CLEARANCE_EM, ACCENT_WIDE_EXTRA_EM, ALIGNED_COLUMN_GUTTER_EM, CASES_ROW_HEIGHT_EM,
    FRAC_DEN_CLEARANCE_EM, FRAC_NUM_CLEARANCE_EM, FRAC_RULE_THICKNESS_EM, MATH_LAYOUT_EM_TO_MM,
    MED_MUSKIP_EM, RADICAL_SURD_PAD_EM, RADICAL_VINCULUM_CLEARANCE_EM,
    RADICAL_VINCULUM_THICKNESS_EM, SCRIPT_SCALE, SCRIPT_SCALE_TEXT, STACKREL_GAP_EM,
    THICK_MUSKIP_EM, THIN_MUSKIP_EM, UNDERBRACE_CLEARANCE_EM,
};
pub use motion::{
    keyframe_track, DurationMs, Easing, Keyframe, MotionTimeline, MotionTrack, TemporalPlacement,
    TimeMs, Timeline, TimelineTrack,
};
pub use slide::{Master, Notes, Placeholder, PlaceholderKind, Slide, Theme, Transition};
pub use style::{Fill, Opacity, Stroke, Style};
pub use text::{Font, Paragraph, Span, Text, TextBox, TextStyle};
pub use vector::{
    BooleanOp, BooleanPathOp, Gradient, GradientStop, PathCommand, StrokeCap, StrokeJoin,
    VectorPath, VectorStroke,
};
pub use visual::{
    rectangle_from_scene, scene_text_box, Canvas, Ellipse, Group, Image, Line, Path, Rectangle,
    Transform, VisualNode,
};
