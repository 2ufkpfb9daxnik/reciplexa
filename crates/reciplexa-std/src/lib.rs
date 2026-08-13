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
    break_line, break_line_to_text_shapes, break_line_vertical, break_opportunity,
    break_opportunity_chars, break_pair_matrix_cell, char_em_width, classify_char, hang_width_em,
    hang_width_em_char, is_hangable, justify_line, justify_line_to_text_shapes,
    lines_to_text_shapes, ruby_estimate_box, tate_chu_yoko_estimate_box, vertical_advance_em,
    wrap_text_shape_content, BreakOpportunity, CharClass, KihonHanmen, Ruby, RubyBox, RubyKind,
    TateChuYoko, TateChuYokoBox, TategakiParagraph, WritingMode, BREAK_PAIR_MATRIX,
    BREAK_PAIR_MATRIX_DIM, HANG_WIDTH_EM,
};
pub use layout::{Align, Axis, Column, LayoutChild, Padding, Row, Stack};
pub use math::{
    aligned_column_x, bigop_limit_offsets, cases_column_align, fraction_rule_metrics,
    matrix_cell_x_in_column, matrix_column_widths, radical_vinculum_index_offsets,
    scripts_attachment_offsets, stackrel_spacing_offsets, underbrace_spacing, MathAccentKind,
    MathAtom, MathBox, MathClass, MathMatrixKind, MathStackKind, MatrixColumnAlign,
    ACCENT_CLEARANCE_EM, ACCENT_UNDER_CLEARANCE_EM, ALIGNED_COLUMN_GUTTER_EM, FRAC_DEN_CLEARANCE_EM,
    FRAC_NUM_CLEARANCE_EM, FRAC_RULE_THICKNESS_EM, RADICAL_SURD_PAD_EM, RADICAL_VINCULUM_CLEARANCE_EM,
    RADICAL_VINCULUM_THICKNESS_EM, SCRIPT_SCALE, STACKREL_GAP_EM, UNDERBRACE_CLEARANCE_EM,
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
