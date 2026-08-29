//! Map GUI nudges back onto CST numeric leaves (Glisp-style).
//!
//! Bindings are collected in the same preorder as [`reciplexa_view`] flattening
//! so hit-test indices line up with editable leaves.

use reciplexa_syntax::{parse_source, SyntaxKind, SyntaxNode};

use crate::cst_walk::{list_atoms, Child};

pub(crate) mod document;
mod geometry;
mod layers;
pub(crate) mod math;
mod package;
mod pages;

pub use document::{
    collect_layers_document, document_columns_params, document_layer_indent_em,
    document_layer_text, is_document_page_authoring, set_document_columns_count,
    set_document_columns_gutter, set_document_layer_indent_em, set_document_layer_text,
};
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
pub use math::{
    collect_layers_live_layout, is_live_layout_authoring, math_layer_glyph,
    set_live_layout_document_text, set_math_layer_glyph,
};
pub use package::{
    collect_layers_package, collect_package_pages, collect_size_targets_package,
    count_package_pages, delete_layer_package, duplicate_layer_package, find_main_expr,
    find_main_expr_in_source, find_package_page, find_package_page_in_source, insert_layer_package,
    is_package_paint_wrapper, is_package_shaped_authoring, is_package_transparent_wrapper,
    nudge_layer_package, package_page_content_nodes, paint_wrapper_shape, reorder_layer_package,
    set_text_content_package,
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

/// Collect layers for live-layout math trees, document-page authoring, package graphics, else interim `(page …)`.
pub fn collect_layers_authoring(src: &str, page_index: usize) -> Result<Vec<LayerInfo>, SyncError> {
    if is_live_layout_authoring(src) {
        collect_layers_live_layout(src, page_index)
    } else if is_document_page_authoring(src) {
        collect_layers_document(src, page_index)
    } else if is_package_shaped_authoring(src) {
        collect_layers_package(src, page_index)
    } else {
        collect_layers_page(src, page_index)
    }
}

/// Size targets for package-shaped authoring, else interim `(page …)`.
pub fn collect_size_targets_authoring(
    src: &str,
    page_index: usize,
) -> Result<Vec<SizeTarget>, SyncError> {
    if is_package_shaped_authoring(src) {
        collect_size_targets_package(src, page_index)
    } else {
        collect_size_targets_page(src, page_index)
    }
}

pub fn insert_layer_authoring(
    src: &str,
    page_index: usize,
    form: &str,
) -> Result<(String, usize), SyncError> {
    if is_live_layout_authoring(src) {
        return Err(math::live_layout_stack_refuse());
    }
    if is_package_shaped_authoring(src) {
        insert_layer_package(src, page_index, form)
    } else {
        insert_layer_page(src, page_index, form)
    }
}

pub fn delete_layer_authoring(
    src: &str,
    page_index: usize,
    flat_index: usize,
) -> Result<String, SyncError> {
    if is_live_layout_authoring(src) {
        return Err(math::live_layout_stack_refuse());
    }
    if is_package_shaped_authoring(src) {
        delete_layer_package(src, page_index, flat_index)
    } else {
        delete_layer_page(src, page_index, flat_index)
    }
}

pub fn reorder_layer_authoring(
    src: &str,
    page_index: usize,
    from: usize,
    to: usize,
) -> Result<String, SyncError> {
    if is_live_layout_authoring(src) {
        return Err(math::live_layout_stack_refuse());
    }
    if is_package_shaped_authoring(src) {
        reorder_layer_package(src, page_index, from, to)
    } else {
        reorder_layer_page(src, page_index, from, to)
    }
}

pub fn nudge_layer_authoring(
    src: &str,
    page_index: usize,
    flat_index: usize,
    dx: f64,
    dy: f64,
) -> Result<String, SyncError> {
    if is_live_layout_authoring(src) {
        return Err(SyncError::new(
            "live-layout math tree is edited via the layer properties panel, not glyph positions",
        ));
    }
    if is_package_shaped_authoring(src) {
        nudge_layer_package(src, page_index, flat_index, dx, dy)
    } else {
        nudge_layer_page(src, page_index, flat_index, dx, dy)
    }
}

pub fn set_text_content_authoring(
    src: &str,
    page_index: usize,
    flat_index: usize,
    text: &str,
) -> Result<String, SyncError> {
    if is_live_layout_authoring(src) {
        let layers = collect_layers_live_layout(src, page_index)?;
        let layer = layers
            .get(flat_index)
            .ok_or_else(|| SyncError::new("layer index out of range"))?;
        if layer.kind.starts_with("doc-") {
            return set_live_layout_document_text(src, page_index, flat_index, text);
        }
        if layer.kind == "math-symbol" {
            return set_math_layer_glyph(src, page_index, flat_index, text);
        }
        return Err(SyncError::new(format!(
            "`{}` is not a text/glyph layer; no ASCII substitution",
            layer.kind
        )));
    }
    if is_document_page_authoring(src) {
        set_document_layer_text(src, page_index, flat_index, text)
    } else if is_package_shaped_authoring(src) {
        set_text_content_package(src, page_index, flat_index, text)
    } else {
        Err(SyncError::new(
            "text content sync only on package or document pages",
        ))
    }
}

pub fn duplicate_layer_authoring(
    src: &str,
    page_index: usize,
    flat_index: usize,
) -> Result<String, SyncError> {
    if is_document_page_authoring(src) {
        return Err(SyncError::new(
            "duplicate is not supported on document/page structure layers",
        ));
    }
    if is_live_layout_authoring(src) {
        return Err(math::live_layout_stack_refuse());
    }
    if is_package_shaped_authoring(src) {
        duplicate_layer_package(src, page_index, flat_index)
    } else {
        duplicate_layer_page(src, page_index, flat_index)
    }
}
