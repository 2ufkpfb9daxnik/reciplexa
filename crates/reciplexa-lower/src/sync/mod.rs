//! Map GUI nudges back onto CST numeric leaves (Glisp-style).
//!
//! Bindings are collected in the same preorder as [`reciplexa_view`] flattening
//! so hit-test indices line up with editable leaves.

use reciplexa_syntax::{parse_source, SyntaxKind, SyntaxNode};

use crate::cst_walk::{list_atoms, Child};

mod geometry;
mod layers;
mod pages;

pub use geometry::{
    collect_drag_targets, collect_drag_targets_page, collect_size_targets_page, layer_opacity,
    layer_rotation_deg, nudge_drag_target, nudge_first_translate, nudge_layer_page,
    scale_layer_uniform, scale_size_target, scale_size_target_axes, scale_text_box, set_box_xywh,
    set_layer_opacity, set_layer_rotation_deg, set_line_endpoint, set_poly_vertex, set_text_box,
};
pub use layers::{
    collect_layers_page, delete_layer_page, duplicate_layer_page, group_layers_page,
    insert_layer_page, reorder_layer_page, ungroup_layer_page,
};
pub use pages::{count_pages, delete_page, insert_page_after};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncError {
    pub message: String,
}

impl SyncError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
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

pub(crate) fn is_headed(node: &SyntaxNode, name: &str) -> bool {
    let items = list_atoms(node);
    matches!(
        items.first(),
        Some(Child::Token(t)) if t.kind() == SyntaxKind::Ident && t.text() == name
    )
}

pub(crate) fn extent_with_leading_ws(src: &str, start: usize, end: usize) -> (usize, usize) {
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

pub(crate) fn parse_root(src: &str) -> Result<SyntaxNode, SyncError> {
    parse_source(src)
        .into_result()
        .map_err(|e| SyncError::new(format!("parse error: {}", e[0].message)))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- validity ---

    #[test]
    fn nudging_translate_preserves_layout() {
        let src = "(page a4\n  (translate  105  148.5\n    (circle 0 0 20)))\n";
        let out = nudge_first_translate(src, 10.0, -5.0).unwrap();
        assert!(out.contains("115"));
        assert!(out.contains("143.5"));
        assert!(out.contains("(page a4\n  (translate  "));
        assert!(out.contains("\n    (circle 0 0 20)))\n"));
    }

    #[test]
    fn bindings_prefer_enclosing_translate() {
        let src = r#"
(page a4
  (rect 1 2 3 4)
  (translate 10 20 (circle 0 0 5))
  (circle 30 40 5))
"#;
        let t = collect_drag_targets(src).unwrap();
        assert_eq!(
            t,
            vec![
                DragTarget::RectXy(0),
                DragTarget::Translate(0),
                DragTarget::CircleXy(1), // second circle form in file
            ]
        );
    }

    #[test]
    fn page_scoped_bindings_ignore_other_pages() {
        let src = "(page a4 (circle 1 2 3))\n(page a4 (rect 0 0 1 1))";
        assert_eq!(
            collect_drag_targets_page(src, 0).unwrap(),
            vec![DragTarget::CircleXy(0)]
        );
        assert_eq!(
            collect_drag_targets_page(src, 1).unwrap(),
            vec![DragTarget::RectXy(0)]
        );
    }

    #[test]
    fn nudge_second_circle_center() {
        let src = "(page a4 (circle 1 2 3) (circle 10 20 5))";
        let out = nudge_drag_target(src, DragTarget::CircleXy(1), 1.0, 1.0).unwrap();
        assert!(out.contains("(circle 1 2 3)"));
        assert!(out.contains("(circle 11 21 5)"));
    }

    #[test]
    fn two_translates_nudge_independently() {
        let src = "(page a4 (translate 1 2 (circle 0 0 1)) (translate 8 9 (circle 0 0 1)))";
        let out = nudge_drag_target(src, DragTarget::Translate(1), 1.0, 1.0).unwrap();
        assert!(out.contains("(translate 1 2"));
        assert!(out.contains("(translate 9 10"));
    }

    #[test]
    fn text_and_line_bindings_nudge() {
        let src = r#"(page a4 (text 1 2 3 "Hi") (line 10 20 30 40))"#;
        let t = collect_drag_targets(src).unwrap();
        assert_eq!(t, vec![DragTarget::TextXy(0), DragTarget::LineXy(0)]);
        let out = nudge_drag_target(src, DragTarget::TextXy(0), 1.0, 1.0).unwrap();
        assert!(out.contains(r#"(text 2 3 3 "Hi")"#));
        let out = nudge_drag_target(src, DragTarget::LineXy(0), 1.0, 1.0).unwrap();
        assert!(out.contains("(line 11 21 31 41)"));
    }

    #[test]
    fn layers_match_flatten_order_and_spans() {
        let src = r#"(page 210 297 (circle 1 2 3) (text 4 5 6 "あ"))"#;
        let layers = collect_layers_page(src, 0).unwrap();
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0].kind, "circle");
        assert_eq!(layers[1].label, "text \"あ\"");
        assert!(src[layers[0].byte_start..layers[0].byte_end].contains("circle"));
        assert!(src[layers[1].byte_start..layers[1].byte_end].contains("text"));
        assert_eq!(layers[0].root_start, layers[0].byte_start);
        assert_eq!(
            collect_drag_targets_page(src, 0).unwrap().len(),
            layers.len()
        );
    }

    #[test]
    fn reorder_layer_moves_source_order() {
        let src = "(page a4\n  (circle 1 2 3)\n  (circle 4 5 6)\n  (rect 0 0 1 1))\n";
        // Move bottom (0) to top (2).
        let out = reorder_layer_page(src, 0, 0, 2).unwrap();
        let layers = collect_layers_page(&out, 0).unwrap();
        assert_eq!(
            layers.iter().map(|l| l.kind.as_str()).collect::<Vec<_>>(),
            vec!["circle", "rect", "circle"]
        );
        assert!(out.find("(circle 1 2 3)").unwrap() > out.find("(rect 0 0 1 1)").unwrap());
    }

    #[test]
    fn reorder_within_translate_rewrites_group_body() {
        let src = "(page a4\n  (translate 0 0\n    (circle 1 2 3)\n    (rect 0 0 1 1)))\n";
        let out = reorder_layer_page(src, 0, 0, 1).unwrap();
        let layers = collect_layers_page(&out, 0).unwrap();
        assert_eq!(layers[0].kind, "rect");
        assert_eq!(layers[1].kind, "circle");
        assert!(out.contains("(translate 0 0"));
    }

    // --- defect ---

    #[test]
    fn no_translate_errors() {
        let err = nudge_first_translate("(page a4 (circle 1 2 3))", 1.0, 1.0).unwrap_err();
        assert!(err.message.contains("translate"));
    }

    #[test]
    fn parse_error_surfaces() {
        assert!(nudge_first_translate("(translate 1", 1.0, 1.0).is_err());
    }

    #[test]
    fn out_of_range_target_errors() {
        let src = "(page a4 (circle 1 2 3))";
        assert!(nudge_drag_target(src, DragTarget::CircleXy(3), 1.0, 0.0).is_err());
    }

    #[test]
    fn scale_circle_radius() {
        let src = "(page a4 (circle 10 20 5))";
        let sizes = collect_size_targets_page(src, 0).unwrap();
        assert_eq!(sizes, vec![SizeTarget::CircleR(0)]);
        let out = scale_size_target(src, SizeTarget::CircleR(0), 2.0).unwrap();
        assert!(out.contains("(circle 10 20 10)"));
    }

    #[test]
    fn scale_rect_about_center() {
        let src = "(page a4 (rect 0 0 10 20))";
        let out = scale_size_target(src, SizeTarget::RectWh(0), 2.0).unwrap();
        // center stays at (5, 10); size 20×40 → origin (-5, -10)
        assert!(
            out.contains("(rect -5 -10 20 40)"),
            "unexpected rewrite: {out}"
        );
    }

    #[test]
    fn scale_rect_width_only() {
        let src = "(page a4 (rect 0 0 10 20))";
        let out = scale_size_target_axes(src, SizeTarget::RectWh(0), 2.0, 1.0).unwrap();
        assert!(
            out.contains("(rect -5 0 20 20)"),
            "unexpected rewrite: {out}"
        );
    }

    #[test]
    fn scale_ellipse_axes_independently() {
        let src = "(page a4 (ellipse 50 60 10 20))";
        let out = scale_size_target_axes(src, SizeTarget::EllipseRxRy(0), 2.0, 0.5).unwrap();
        assert!(
            out.contains("(ellipse 50 60 20 10)"),
            "unexpected rewrite: {out}"
        );
    }

    #[test]
    fn scale_text_box_axes() {
        let src = "(page a4 (text 10 20 12 \"hello\"))";
        let out = scale_text_box(src, 0, 2.0, 0.5).unwrap();
        // Font size stays; w/h inserted and scaled about center.
        assert!(
            out.contains("(text ") && out.contains(" 12 ") && out.contains("\"hello\")"),
            "unexpected rewrite: {out}"
        );
        assert!(
            !out.contains("(text 10 20 12 \"hello\")"),
            "should insert box dims: {out}"
        );
        let sizes = collect_size_targets_page(&out, 0).unwrap();
        assert_eq!(sizes, vec![SizeTarget::TextSize(0)]);
        // Second scale edits existing w/h.
        let out2 = scale_text_box(&out, 0, 1.0, 2.0).unwrap();
        assert!(out2.contains(" 12 "), "size must stay: {out2}");
    }

    #[test]
    fn set_text_box_inserts_and_updates() {
        let src = "(page a4 (text 10 20 12 \"hello\" blue))";
        let out = set_text_box(src, 0, 11.0, 21.0, 40.0, 15.0).unwrap();
        assert!(
            out.contains("(text 11 21 12 40 15 \"hello\" blue)"),
            "unexpected: {out}"
        );
        let out2 = set_text_box(&out, 0, 0.0, 0.0, 8.0, 9.0).unwrap();
        assert!(
            out2.contains("(text 0 0 12 8 9 \"hello\" blue)"),
            "unexpected: {out2}"
        );
    }

    #[test]
    fn set_rect_box_xywh() {
        let src = "(page a4 (rect 10 20 30 40 red))";
        let out = set_box_xywh(src, "rect", 0, [1, 2, 3, 4], 5.0, 6.0, 7.0, 8.0).unwrap();
        assert!(out.contains("(rect 5 6 7 8 red)"), "unexpected: {out}");
    }

    #[test]
    fn set_line_endpoint_rewrites_pair() {
        let src = "(page a4 (line 0 0 10 10 red 1))";
        let out = set_line_endpoint(src, 0, 1, 20.0, 30.0).unwrap();
        assert!(out.contains("(line 0 0 20 30 red 1)"), "unexpected: {out}");
        let out0 = set_line_endpoint(src, 0, 0, -1.0, -2.0).unwrap();
        assert!(
            out0.contains("(line -1 -2 10 10 red 1)"),
            "unexpected: {out0}"
        );
    }

    #[test]
    fn set_polyline_vertex_rewrites_pair() {
        let src = "(page a4 (polyline 0 0 10 0 10 10 red 1))";
        let out = set_poly_vertex(src, "polyline", 0, 1, 5.0, 5.0).unwrap();
        assert!(
            out.contains("(polyline 0 0 5 5 10 10 red 1)"),
            "unexpected: {out}"
        );
    }

    #[test]
    fn scale_under_translate_still_edits_leaf() {
        let src = "(page a4 (translate 1 2 (circle 0 0 4)))";
        let sizes = collect_size_targets_page(src, 0).unwrap();
        assert_eq!(sizes, vec![SizeTarget::CircleR(0)]);
        let out = scale_size_target(src, SizeTarget::CircleR(0), 0.5).unwrap();
        assert!(out.contains("(circle 0 0 2)"));
    }

    #[test]
    fn bad_scale_factor_errors() {
        let src = "(page a4 (circle 1 2 3))";
        assert!(scale_size_target(src, SizeTarget::CircleR(0), 0.0).is_err());
        assert!(scale_size_target(src, SizeTarget::CircleR(0), -1.0).is_err());
    }

    #[test]
    fn set_rotation_wraps_about_object_center() {
        let src = "(page a4 (circle 10 20 5))";
        assert_eq!(layer_rotation_deg(src, 0, 0).unwrap(), 0.0);
        let out = set_layer_rotation_deg(src, 0, 0, 30.0, (10.0, 20.0)).unwrap();
        assert!(
            out.contains("(translate 10 20 (rotate 30 (translate -10 -20 (circle 10 20 5))))"),
            "unexpected rewrite: {out}"
        );
        assert_eq!(layer_rotation_deg(&out, 0, 0).unwrap(), 30.0);
        let out2 = set_layer_rotation_deg(&out, 0, 0, -15.0, (10.0, 20.0)).unwrap();
        assert!(out2.contains("(rotate -15 "), "unexpected rewrite: {out2}");
        assert_eq!(layer_rotation_deg(&out2, 0, 0).unwrap(), -15.0);
    }

    #[test]
    fn set_rotation_upgrades_bare_rotate_to_center_pivot() {
        let src = "(page a4 (rotate 10 (circle 10 20 5)))";
        let out = set_layer_rotation_deg(src, 0, 0, 45.0, (10.0, 20.0)).unwrap();
        assert!(
            out.contains("(translate 10 20 (rotate 45 (translate -10 -20 (circle 10 20 5))))"),
            "unexpected rewrite: {out}"
        );
        assert!(!out.contains("(rotate 45 (circle"));
    }

    #[test]
    fn set_rotation_on_existing_rotate_root() {
        let src = "(page a4 (translate 1 2 (rotate 10 (translate -1 -2 (circle 0 0 3)))))";
        let out = set_layer_rotation_deg(src, 0, 0, 45.0, (1.0, 2.0)).unwrap();
        assert!(out.contains("(rotate 45 "));
        assert_eq!(layer_rotation_deg(&out, 0, 0).unwrap(), 45.0);
    }

    #[test]
    fn center_rotate_sandwich_drag_uses_outer_translate() {
        let src = "(page a4 (translate 10 20 (rotate 30 (translate -10 -20 (circle 10 20 5)))))";
        let t = collect_drag_targets_page(src, 0).unwrap();
        assert_eq!(t, vec![DragTarget::Translate(0)]);
        let out = nudge_drag_target(src, DragTarget::Translate(0), 1.0, 2.0).unwrap();
        assert!(out.contains("(translate 11 22 (rotate 30 (translate -10 -20"));
    }

    #[test]
    fn set_opacity_wraps_then_edits() {
        let src = "(page a4 (circle 1 2 3))";
        assert_eq!(layer_opacity(src, 0, 0).unwrap(), 1.0);
        let out = set_layer_opacity(src, 0, 0, 0.4).unwrap();
        assert!(out.contains("(opacity 0.4 (circle 1 2 3))"));
        assert_eq!(layer_opacity(&out, 0, 0).unwrap(), 0.4);
        let out2 = set_layer_opacity(&out, 0, 0, 0.75).unwrap();
        assert!(out2.contains("(opacity 0.75 (circle 1 2 3))"));
    }

    #[test]
    fn opacity_clamps_to_unit_interval() {
        let src = "(page a4 (circle 1 2 3))";
        let out = set_layer_opacity(src, 0, 0, 2.0).unwrap();
        assert!(out.contains("(opacity 1 (circle 1 2 3))"));
        let out = set_layer_opacity(src, 0, 0, -0.5).unwrap();
        assert!(out.contains("(opacity 0 (circle 1 2 3))"));
    }

    #[test]
    fn scale_line_about_midpoint() {
        let src = "(page a4 (line 0 0 10 0 red 2))";
        let sizes = collect_size_targets_page(src, 0).unwrap();
        assert_eq!(sizes, vec![SizeTarget::LineSeg(0)]);
        let out = scale_size_target(src, SizeTarget::LineSeg(0), 2.0).unwrap();
        // midpoint (5,0); endpoints → (-5,0) and (15,0); width 4
        assert!(
            out.contains("(line -5 0 15 0 red 4)"),
            "unexpected rewrite: {out}"
        );
    }

    #[test]
    fn scale_polyline_vertices_and_width() {
        let src = "(page a4 (polyline 0 0 10 0 10 10 red 1.5))";
        let out = scale_size_target(src, SizeTarget::PolylinePoints(0), 2.0).unwrap();
        // centroid (20/3, 10/3) ≈ (6.667, 3.333) — check scaled width and that coords moved
        assert!(out.contains("red 3"), "unexpected rewrite: {out}");
        assert!(!out.contains("(polyline 0 0 10 0 10 10"));
    }

    #[test]
    fn scale_polygon_about_centroid() {
        let src = "(page a4 (polygon 0 0 10 0 0 10))";
        let out = scale_size_target(src, SizeTarget::PolygonPoints(0), 2.0).unwrap();
        // centroid (10/3, 10/3); first vertex 0,0 → -10/3, -10/3
        assert!(
            out.contains("-3.333") || out.contains("-3.333333"),
            "unexpected rewrite: {out}"
        );
    }

    #[test]
    fn delete_top_level_layer() {
        let src = "(page a4 (circle 1 2 3) (rect 0 0 1 1))";
        let out = delete_layer_page(src, 0, 0).unwrap();
        assert!(!out.contains("circle"));
        assert!(out.contains("(rect 0 0 1 1)"));
        let layers = collect_layers_page(&out, 0).unwrap();
        assert_eq!(layers.len(), 1);
    }

    #[test]
    fn delete_leaf_inside_translate_keeps_sibling() {
        let src = "(page a4 (translate 0 0 (circle 1 2 3) (rect 0 0 1 1)))";
        let out = delete_layer_page(src, 0, 0).unwrap();
        assert!(out.contains("(translate 0 0"));
        assert!(!out.contains("circle"));
        assert!(out.contains("(rect 0 0 1 1)"));
    }

    #[test]
    fn duplicate_top_level_layer() {
        let src = "(page a4 (circle 1 2 3) (rect 0 0 1 1))";
        let out = duplicate_layer_page(src, 0, 0).unwrap();
        let layers = collect_layers_page(&out, 0).unwrap();
        assert_eq!(layers.len(), 3);
        assert_eq!(layers[0].kind, "circle");
        assert_eq!(layers[1].kind, "circle");
        assert_eq!(layers[2].kind, "rect");
    }

    #[test]
    fn insert_layer_appends_and_returns_index() {
        let src = "(page a4 (circle 1 2 3))";
        let (out, idx) = insert_layer_page(src, 0, "(rect 10 20 30 40 red)").unwrap();
        assert!(out.contains("(rect 10 20 30 40 red)"));
        let layers = collect_layers_page(&out, 0).unwrap();
        assert_eq!(idx, layers.len() - 1);
        assert_eq!(layers[idx].kind, "rect");
    }

    #[test]
    fn insert_layer_into_empty_page() {
        let src = "(page a4)";
        let (out, idx) = insert_layer_page(src, 0, "(circle 105 148.5 20)").unwrap();
        assert_eq!(out, "(page a4 (circle 105 148.5 20))");
        assert_eq!(idx, 0);
    }

    #[test]
    fn scale_layer_uniform_wraps_any_root() {
        let src = "(page a4 (circle 1 2 3))";
        let out = scale_layer_uniform(src, 0, 0, 2.0).unwrap();
        assert!(out.contains("(scale 2 (circle 1 2 3))"));
    }

    #[test]
    fn group_and_ungroup_contiguous_roots() {
        let src = "(page a4 (circle 1 2 3) (rect 0 0 10 10) (ellipse 50 60 5 4))";
        let (grouped, sel) = group_layers_page(src, 0, &[0, 1]).unwrap();
        assert!(grouped.contains("(group"), "expected group: {grouped}");
        assert!(grouped.contains("(circle 1 2 3)"));
        assert!(grouped.contains("(rect 0 0 10 10)"));
        assert!(grouped.contains("(ellipse 50 60 5 4)"));
        assert_eq!(sel, vec![0, 1]);
        let layers = collect_layers_page(&grouped, 0).unwrap();
        assert_eq!(layers.len(), 3);
        // Shared group root for first two leaves.
        assert_eq!(
            (layers[0].root_start, layers[0].root_end),
            (layers[1].root_start, layers[1].root_end)
        );
        assert_ne!(
            (layers[0].root_start, layers[0].root_end),
            (layers[2].root_start, layers[2].root_end)
        );

        let (ungrouped, sel2) = ungroup_layer_page(&grouped, 0, 0).unwrap();
        assert!(!ungrouped.contains("(group"), "peeled: {ungrouped}");
        assert_eq!(sel2, vec![0, 1]);
        let layers2 = collect_layers_page(&ungrouped, 0).unwrap();
        assert_eq!(layers2.len(), 3);
        assert_ne!(
            (layers2[0].root_start, layers2[0].root_end),
            (layers2[1].root_start, layers2[1].root_end)
        );
    }

    #[test]
    fn group_rejects_noncontiguous_roots() {
        let src = "(page a4 (circle 1 2 3) (rect 0 0 10 10) (ellipse 50 60 5 4))";
        let err = group_layers_page(src, 0, &[0, 2]).unwrap_err();
        assert!(err.message.contains("contiguous"));
    }

    #[test]
    fn insert_and_delete_pages() {
        let src = "(page a4 (circle 1 2 3))\n(page a4)";
        assert_eq!(count_pages(src).unwrap(), 2);
        let (out, idx) = insert_page_after(src, Some(0), "(page letter)").unwrap();
        assert_eq!(idx, 1);
        assert_eq!(count_pages(&out).unwrap(), 3);
        assert!(out.contains("(page letter)"));
        let layers_mid = collect_layers_page(&out, 1).unwrap();
        assert!(layers_mid.is_empty());
        let trimmed = delete_page(&out, 1).unwrap();
        assert_eq!(count_pages(&trimmed).unwrap(), 2);
        assert!(!trimmed.contains("letter"));
        let only = "(page a4 (circle 0 0 1))";
        assert!(delete_page(only, 0).unwrap_err().message.contains("only"));
    }

    #[test]
    fn nudge_layer_wraps_bare_shape_with_translate() {
        let src = "(page a4 (circle 10 20 5))";
        let out = nudge_layer_page(src, 0, 0, 3.0, -1.0).unwrap();
        assert!(
            out.contains("(translate 3 -1 (circle 10 20 5))"),
            "unexpected rewrite: {out}"
        );
        // Second nudge edits the translate only (world axes).
        let out2 = nudge_layer_page(&out, 0, 0, 1.0, 1.0).unwrap();
        assert!(
            out2.contains("(translate 4 0 (circle 10 20 5))"),
            "unexpected rewrite: {out2}"
        );
    }

    #[test]
    fn nudge_layer_under_bare_rotate_stays_world_axis() {
        // Without wrap, nudging circle x/y would move along the rotated frame.
        let src = "(page a4 (rotate 90 (circle 10 0 1)))";
        let out = nudge_layer_page(src, 0, 0, 5.0, 0.0).unwrap();
        assert!(
            out.contains("(translate 5 0 (rotate 90 (circle 10 0 1)))"),
            "unexpected rewrite: {out}"
        );
        // Angle remains visible to the rotate knob after the move wrap.
        assert_eq!(layer_rotation_deg(&out, 0, 0).unwrap(), 90.0);
    }

    #[test]
    fn nudge_layer_center_sandwich_moves_outer_only() {
        let src = "(page a4 (translate 10 20 (rotate 30 (translate -10 -20 (circle 10 20 5)))))";
        let out = nudge_layer_page(src, 0, 0, 2.0, 3.0).unwrap();
        assert!(out.contains("(translate 12 23 (rotate 30 (translate -10 -20"));
        assert!(out.contains("(circle 10 20 5)"));
    }
}
