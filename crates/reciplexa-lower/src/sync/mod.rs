//! Map GUI nudges back onto CST numeric leaves (Glisp-style).
//!
//! Bindings are collected in the same preorder as [`reciplexa_view`] flattening
//! so hit-test indices line up with editable leaves.

use reciplexa_syntax::{parse_source, SyntaxKind, SyntaxNode};

use crate::cst_walk::{list_atoms, Child};

mod geometry;
mod layers;
mod package;
mod pages;

pub use geometry::{
    collect_drag_targets, collect_drag_targets_page, collect_size_targets_from_root,
    collect_size_targets_page, layer_opacity, layer_rotation_deg, nudge_drag_target,
    nudge_first_translate, nudge_layer_page, scale_box_axes, scale_layer_uniform,
    scale_size_target, scale_size_target_axes, scale_text_box, set_box_xywh, set_layer_opacity,
    set_layer_rotation_deg, set_line_endpoint, set_poly_vertex, set_text_box,
};
pub use layers::{
    collect_layers_from_root, collect_layers_page, delete_layer_page, duplicate_layer_page,
    group_layers_page, insert_layer_page, reorder_layer_page, ungroup_layer_page,
};
pub use package::{
    collect_layers_package, collect_package_pages, find_main_expr, find_main_expr_in_source,
    find_package_page, find_package_page_in_source, is_package_paint_wrapper,
    is_package_shaped_authoring, is_package_transparent_wrapper, nudge_layer_package,
    package_page_content_nodes, paint_wrapper_shape,
};
pub use pages::{count_pages, delete_page, find_page, insert_page_after, page_body_start};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncError {
    pub message: String,
}

impl SyncError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// How a flattened world shape maps back to editable size numbers (scale handles).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeTarget {
    /// `(circle x y r …)` — scale radius about center.
    CircleR(usize),
    /// `(rect x y w h …)` — scale w/h about center (rewrites x/y too).
    RectWh(usize),
    /// `(ellipse x y rx ry …)` — scale radii about center.
    EllipseRxRy(usize),
    /// `(ring x y r width …)` — scale radius about center.
    RingR(usize),
    /// `(frame x y w h …)` — scale w/h about center.
    FrameWh(usize),
    /// `(text x y size [w h] …)` — resize layout box (writes/inserts w/h; font size unchanged).
    TextSize(usize),
    /// `(image path x y w h)` — scale w/h about center.
    ImageWh(usize),
    /// `(line x1 y1 x2 y2 …)` — scale endpoints about midpoint.
    LineSeg(usize),
    /// `(polyline …)` — scale vertices about centroid.
    PolylinePoints(usize),
    /// `(polygon …)` — scale vertices about centroid.
    PolygonPoints(usize),
    /// Unknown / not yet editable size.
    Unsupported,
}

/// How a flattened world shape maps back to editable CST numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragTarget {
    /// N-th `(translate tx ty …)` in preorder (0-based).
    Translate(usize),
    /// N-th `(circle x y …)` center when not under a translate binding.
    CircleXy(usize),
    /// N-th `(rect x y …)` origin when not under a translate binding.
    RectXy(usize),
    /// N-th `(ellipse x y …)` center when not under a translate binding.
    EllipseXy(usize),
    /// N-th `(ring x y …)` center when not under a translate binding.
    RingXy(usize),
    /// N-th `(frame x y …)` origin when not under a translate binding.
    FrameXy(usize),
    /// N-th `(text x y …)` baseline when not under a translate binding.
    TextXy(usize),
    /// N-th `(line …)` — nudges both endpoints by the same delta.
    LineXy(usize),
    /// N-th `(polyline …)` — nudges every vertex by the same delta.
    PolylineXy(usize),
    /// N-th `(polygon …)` — nudges every vertex by the same delta.
    PolygonXy(usize),
    /// N-th `(image …)` origin when not under a translate binding.
    ImageXy(usize),
}

/// One flattened drawable for the GUI layer list (same order as hit-test indices).
///
/// **Z-order is source order:** later siblings under a page (or group) paint on
/// top. There is no separate z field — reordering layers rewrites the `.rpx`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerInfo {
    pub kind: String,
    pub label: String,
    /// Byte range of the leaf shape list in the source (for editor highlight).
    pub byte_start: usize,
    pub byte_end: usize,
    /// Direct child of `(page …)` that owns this leaf (equals byte_* if top-level).
    pub root_start: usize,
    pub root_end: usize,
}

pub fn is_headed(node: &SyntaxNode, name: &str) -> bool {
    let items = list_atoms(node);
    matches!(
        items.first(),
        Some(Child::Token(t)) if t.kind() == SyntaxKind::Ident && t.text() == name
    )
}

pub fn extent_with_leading_ws(src: &str, start: usize, end: usize) -> (usize, usize) {
    let bytes = src.as_bytes();
    let mut s = start;
    while s > 0 && matches!(bytes[s - 1], b' ' | b'\t') {
        s -= 1;
    }
    if s > 0 && bytes[s - 1] == b'\n' {
        s -= 1;
        if s > 0 && bytes[s - 1] == b'\r' {
            s -= 1;
        }
    }
    (s, end)
}

pub fn parse_root(src: &str) -> Result<SyntaxNode, SyncError> {
    parse_source(src)
        .into_result()
        .map_err(|e| SyncError::new(format!("parse error: {}", e[0].message)))
}
