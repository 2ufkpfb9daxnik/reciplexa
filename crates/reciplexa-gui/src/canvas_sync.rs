//! Authoring-source canvas sync helpers.
//!
//! Preview layers may come from macro-expanded buffers (`expanded_for_sync`), but
//! CST mutations must always target the authoring `.rpx`. When those layer lists
//! diverge (e.g. `(markup …)` → synthetic page shapes), refuse the edit softly.
//!
//! Package-shaped authoring (`(import graphics|document` + `val main`) is writable
//! via package CST sync (GUI CST sync v2). Markup authoring that expands to package
//! still soft-refuses.

use reciplexa::wants_package_graphics_path;
use reciplexa_lower::{
    attach_glyph_children, collect_layers_document, collect_layers_live_layout,
    collect_layers_package, collect_layers_page, collect_layers_vertical_demo,
    collect_size_targets_package, collect_size_targets_page, is_document_page_authoring,
    is_live_layout_authoring, is_package_shaped_authoring, is_vertical_demo_authoring,
    layers_cover_shapes, nudge_layer_package, nudge_layer_page, nudge_vertical_demo_layer,
    parent_layer_index, shape_indices_for_authoring, LayerInfo, SizeTarget, SyncError,
};
use reciplexa_view::{hit_test_shapes, PaperLayout, WorldShape};

/// Why a canvas edit cannot be written back to authoring source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncRefuse {
    pub message: String,
}

impl SyncRefuse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Build read-only layer rows from flattened scene shapes (bridge / mismatch path).
///
/// Byte spans are zeroed — there is no CST leaf to highlight or mutate.
/// Text shapes include a short content prefix (same spirit as package CST labels).
pub fn layers_from_world_shapes(shapes: &[WorldShape]) -> Vec<LayerInfo> {
    shapes
        .iter()
        .enumerate()
        .map(|(i, shape)| {
            let kind = world_shape_kind(shape).to_string();
            let label = match shape {
                WorldShape::Text(t) => {
                    let short: String = t.content.chars().take(24).collect();
                    format!("text \"{short}\" (read-only #{})", i + 1)
                }
                _ => format!("{kind} (read-only #{})", i + 1),
            };
            let mut layer = LayerInfo::new(kind, label, 0, 0, 0, 0);
            layer.authoring_index = i;
            layer.shape_index = Some(i);
            layer
        })
        .collect()
}

fn world_shape_kind(shape: &WorldShape) -> &'static str {
    match shape {
        WorldShape::Circle(_) => "circle",
        WorldShape::Polygon(_) => "polygon",
        WorldShape::Text(_) => "text",
        WorldShape::Path(_) => "path",
        WorldShape::Image(_) => "image",
    }
}

/// Unsupported size bindings matching `n` flattened shapes (scale soft-fails).
pub fn readonly_size_targets(n: usize) -> Vec<SizeTarget> {
    vec![SizeTarget::Unsupported; n]
}

fn attach_or_world(
    layers: Vec<LayerInfo>,
    shapes: &[WorldShape],
    keep_uncovered: bool,
) -> Vec<LayerInfo> {
    let tree = attach_glyph_children(layers, shapes);
    if keep_uncovered || layers_cover_shapes(&tree, shapes.len()) {
        tree
    } else {
        layers_from_world_shapes(shapes)
    }
}

/// Union AABB of flattened shapes in page mm.
pub fn union_bounds_mm(shapes: &[WorldShape], indices: &[usize]) -> Option<(f64, f64, f64, f64)> {
    let mut acc: Option<(f64, f64, f64, f64)> = None;
    for &i in indices {
        let b = shapes.get(i).and_then(PaperLayout::shape_bounds_mm)?;
        acc = Some(match acc {
            None => b,
            Some((x0, y0, x1, y1)) => (x0.min(b.0), y0.min(b.1), x1.max(b.2), y1.max(b.3)),
        });
    }
    acc
}

fn pad_aabb(b: (f64, f64, f64, f64), pad: f64) -> (f64, f64, f64, f64) {
    (b.0 - pad, b.1 - pad, b.2 + pad, b.3 + pad)
}

pub fn point_in_aabb(x: f64, y: f64, b: (f64, f64, f64, f64)) -> bool {
    let x0 = b.0.min(b.2);
    let x1 = b.0.max(b.2);
    let y0 = b.1.min(b.3);
    let y1 = b.1.max(b.3);
    x >= x0 && x <= x1 && y >= y0 && y <= y1
}

/// Authoring parent under a paper point: glyph ink first, then the text-box union.
pub fn hit_test_authoring_parent(
    layers: &[LayerInfo],
    shapes: &[WorldShape],
    x_mm: f64,
    y_mm: f64,
) -> Option<usize> {
    if let Some(i) = hit_test_shapes(shapes, x_mm, y_mm) {
        if let Some(p) = parent_layer_index(layers, i) {
            return Some(p);
        }
    }
    let mut best: Option<(usize, f64)> = None;
    for (pi, layer) in layers.iter().enumerate() {
        if layer.glyph_child {
            continue;
        }
        let idxs = shape_indices_for_authoring(layers, &[layer.authoring_index]);
        if idxs.is_empty() {
            continue;
        }
        if let Some(b) = union_bounds_mm(shapes, &idxs) {
            let padded = pad_aabb(b, 1.5);
            if point_in_aabb(x_mm, y_mm, padded) {
                let area = (padded.2 - padded.0).abs() * (padded.3 - padded.1).abs();
                if best.is_none_or(|(_, a)| area <= a) {
                    best = Some((pi, area));
                }
            }
        }
    }
    best.map(|(pi, _)| pi)
}

/// Prefer CST layers; package authoring uses package collectors; else scene rows.
pub fn resolve_preview_layers(
    expanded: &str,
    page_index: usize,
    shapes: &[WorldShape],
) -> Vec<LayerInfo> {
    if is_live_layout_authoring(expanded) {
        let layers = collect_layers_live_layout(expanded, page_index)
            .unwrap_or_else(|_| layers_from_world_shapes(shapes));
        return attach_or_world(layers, shapes, true);
    }
    if is_vertical_demo_authoring(expanded) {
        let layers = collect_layers_vertical_demo(expanded, page_index)
            .unwrap_or_else(|_| layers_from_world_shapes(shapes));
        return attach_or_world(layers, shapes, true);
    }
    if is_document_page_authoring(expanded) {
        let layers = collect_layers_document(expanded, page_index)
            .unwrap_or_else(|_| layers_from_world_shapes(shapes));
        return attach_or_world(layers, shapes, true);
    }
    if is_package_shaped_authoring(expanded) {
        return match collect_layers_package(expanded, page_index) {
            Ok(layers) => attach_or_world(layers, shapes, false),
            Err(_) => layers_from_world_shapes(shapes),
        };
    }
    match collect_layers_page(expanded, page_index) {
        Ok(layers) => attach_or_world(layers, shapes, false),
        Err(_) => layers_from_world_shapes(shapes),
    }
}

/// Prefer CST size targets; fall back to unsupported stubs on mismatch.
pub fn resolve_preview_size_targets(
    expanded: &str,
    page_index: usize,
    shape_count: usize,
) -> Vec<SizeTarget> {
    if is_package_shaped_authoring(expanded) {
        return match collect_size_targets_package(expanded, page_index) {
            Ok(targets) if targets.len() == shape_count => targets,
            Ok(_) | Err(_) => readonly_size_targets(shape_count),
        };
    }
    if wants_package_graphics_path(expanded) {
        return readonly_size_targets(shape_count);
    }
    match collect_size_targets_page(expanded, page_index) {
        Ok(targets) if targets.len() == shape_count => targets,
        Ok(_) | Err(_) => readonly_size_targets(shape_count),
    }
}

/// True when flattened layers in `authoring` line up with `expanded` for `page_index`.
///
/// Spans may differ (e.g. `color-byte` expansion) as long as count and kinds match;
/// mutations re-resolve spans from the authoring buffer.
pub fn authoring_layers_align(
    authoring: &str,
    expanded: &str,
    page_index: usize,
) -> Result<bool, SyncError> {
    if is_live_layout_authoring(authoring) {
        let auth = collect_layers_live_layout(authoring, page_index)?;
        let exp = if is_live_layout_authoring(expanded) {
            collect_layers_live_layout(expanded, page_index)?
        } else {
            return Ok(false);
        };
        return Ok(layer_kinds_match(&auth, &exp));
    }
    if is_vertical_demo_authoring(authoring) {
        let auth = collect_layers_vertical_demo(authoring, page_index)?;
        let exp = if is_vertical_demo_authoring(expanded) {
            collect_layers_vertical_demo(expanded, page_index)?
        } else {
            return Ok(false);
        };
        return Ok(layer_kinds_match(&auth, &exp));
    }
    if is_document_page_authoring(authoring) {
        let auth = collect_layers_document(authoring, page_index)?;
        let exp = if is_document_page_authoring(expanded) {
            collect_layers_document(expanded, page_index)?
        } else {
            return Ok(false);
        };
        return Ok(layer_kinds_match(&auth, &exp));
    }
    if is_package_shaped_authoring(authoring) {
        let auth = collect_layers_package(authoring, page_index)?;
        let exp = if is_package_shaped_authoring(expanded) {
            collect_layers_package(expanded, page_index)?
        } else {
            return Ok(false);
        };
        return Ok(layer_kinds_match(&auth, &exp));
    }
    // Markup / synthetic package expand: not editable in authoring.
    if wants_package_graphics_path(expanded) || wants_package_graphics_path(authoring) {
        return Ok(false);
    }
    let auth = match collect_layers_page(authoring, page_index) {
        Ok(l) => l,
        Err(_) if authoring != expanded => return Ok(false),
        Err(e) => return Err(e),
    };
    let exp = collect_layers_page(expanded, page_index)?;
    Ok(layer_kinds_match(&auth, &exp))
}

fn layer_kinds_match(a: &[LayerInfo], b: &[LayerInfo]) -> bool {
    a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.kind == y.kind)
}

/// Nudge flattened layers in **authoring** source when they align with `expanded`.
///
/// Returns [`SyncRefuse`] (soft) when expanded layers are synthetic / not editable
/// in authoring — callers must not treat this as a hard document error.
pub fn nudge_authoring_layers(
    authoring: &str,
    expanded: &str,
    page_index: usize,
    flat_indices: &[usize],
    dx: f64,
    dy: f64,
) -> Result<String, SyncRefuse> {
    if dx == 0.0 && dy == 0.0 {
        return Ok(authoring.to_string());
    }
    if is_live_layout_authoring(authoring) {
        return Err(SyncRefuse::new(
            "canvas move skipped: math tree structure is edited via the layer properties panel, not glyph positions",
        ));
    }
    if is_vertical_demo_authoring(authoring) {
        match authoring_layers_align(authoring, expanded, page_index) {
            Ok(true) => {}
            Ok(false) => {
                return Err(SyncRefuse::new(
                    "canvas move skipped: vertical-demo authoring layers do not align with expanded",
                ));
            }
            Err(e) => return Err(SyncRefuse::new(e.message)),
        }
        let mut src = authoring.to_string();
        let mut indices = flat_indices.to_vec();
        indices.sort_unstable();
        indices.dedup();
        let mut last_err = None;
        let mut any = false;
        for &idx in &indices {
            match nudge_vertical_demo_layer(&src, page_index, idx, dx, dy) {
                Ok(next) => {
                    src = next;
                    any = true;
                }
                Err(e) => last_err = Some(e),
            }
        }
        if any {
            return Ok(src);
        }
        let message = last_err
            .map(|e| {
                format!(
                    "canvas move skipped: {} (authoring source unchanged)",
                    e.message
                )
            })
            .unwrap_or_else(|| "canvas move skipped: no movable vertical column".into());
        return Err(SyncRefuse::new(message));
    }
    if is_document_page_authoring(authoring) {
        return Err(SyncRefuse::new(
            "canvas move skipped: document/page structure is edited via the layer properties panel, not glyph positions",
        ));
    }
    if is_package_shaped_authoring(authoring) {
        match authoring_layers_align(authoring, expanded, page_index) {
            Ok(true) => {}
            Ok(false) => {
                return Err(SyncRefuse::new(
                    "canvas move skipped: package authoring layers do not align with expanded",
                ));
            }
            Err(e) => return Err(SyncRefuse::new(e.message)),
        }
        let mut src = authoring.to_string();
        let mut indices = flat_indices.to_vec();
        indices.sort_unstable();
        indices.dedup();
        for &idx in &indices {
            match nudge_layer_package(&src, page_index, idx, dx, dy) {
                Ok(next) => src = next,
                Err(e) => {
                    return Err(SyncRefuse::new(format!(
                        "canvas move skipped: {} (authoring source unchanged)",
                        e.message
                    )));
                }
            }
        }
        return Ok(src);
    }
    if wants_package_graphics_path(expanded) || wants_package_graphics_path(authoring) {
        return Err(SyncRefuse::new(
            "canvas move skipped: expanded layers are not editable in authoring source \
             (package sync v2 still does not cover markup authoring — \
              markup-generated pages stay read-only; edit the markup block instead)",
        ));
    }
    match authoring_layers_align(authoring, expanded, page_index) {
        Ok(true) => {}
        Ok(false) => {
            return Err(SyncRefuse::new(
                "canvas move skipped: expanded layers are not editable in authoring source \
                 (package sync v2 still does not cover markup authoring — \
                  markup-generated text stays read-only; edit the markup block instead)",
            ));
        }
        Err(e) => return Err(SyncRefuse::new(e.message)),
    }

    let mut src = authoring.to_string();
    let mut indices = flat_indices.to_vec();
    indices.sort_unstable();
    indices.dedup();
    for &idx in &indices {
        match nudge_layer_page(&src, page_index, idx, dx, dy) {
            Ok(next) => src = next,
            Err(e) => {
                return Err(SyncRefuse::new(format!(
                    "canvas move skipped: {} (authoring source unchanged)",
                    e.message
                )));
            }
        }
    }
    Ok(src)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_page_layers_align_after_color_byte_expand() {
        let authoring = r#"(page a4
  (text 30 260 8 "Reciplexa" black)
  (line 30 250 180 250 (color-byte 200 40 40) 1)
  (translate 105 120
    (circle 0 0 25 (color-byte 30 90 180))))
"#;
        let expanded = reciplexa_macro::expand_source(authoring).unwrap();
        assert_ne!(authoring, expanded);
        assert!(authoring_layers_align(authoring, &expanded, 0).unwrap());
    }

    #[test]
    fn markup_only_does_not_align() {
        let authoring = "(markup @heading(Hi)\n\nbody\n)";
        let expanded = reciplexa_macro::expand_source(authoring).unwrap();
        assert!(!authoring_layers_align(authoring, &expanded, 0).unwrap());
    }

    #[test]
    fn nudge_text_layer_updates_authoring() {
        let src = r#"(page a4 (text 30 260 8 "Hi" black))"#;
        let expanded = src.to_string();
        let out = nudge_authoring_layers(src, &expanded, 0, &[0], 5.0, -3.0).unwrap();
        assert!(
            out.contains("(translate 5 -3 (text 30 260 8 \"Hi\" black))"),
            "{out}"
        );
    }

    #[test]
    fn nudge_zero_delta_is_identity() {
        let src = r#"(page a4 (text 30 260 8 "Hi" black))"#;
        let out = nudge_authoring_layers(src, src, 0, &[0], 0.0, 0.0).unwrap();
        assert_eq!(out, src);
    }

    #[test]
    fn authoring_parse_error_with_identical_buffers_propagates() {
        let bad = "(page";
        let err = authoring_layers_align(bad, bad, 0).unwrap_err();
        assert!(!err.message.is_empty());
    }

    #[test]
    fn nudge_propagates_align_hard_error() {
        let bad = "(page";
        let err = nudge_authoring_layers(bad, bad, 0, &[0], 1.0, 0.0).unwrap_err();
        assert!(!err.message.is_empty());
    }

    #[test]
    fn nudge_bad_flat_index_soft_refuses() {
        let src = r#"(page a4 (text 30 260 8 "Hi" black))"#;
        let err = nudge_authoring_layers(src, src, 0, &[99], 1.0, 0.0).unwrap_err();
        assert!(err.message.contains("skipped"), "{}", err.message);
    }

    #[test]
    fn package_shaped_layers_writable_and_nudge_updates_circle() {
        use reciplexa_scene::{Circle, Color, Document, Page, PaperSize, Shape};
        use reciplexa_view::flatten_page;

        let pkg = include_str!("../../../examples/pkg_black_circle.rpx");
        assert!(is_package_shaped_authoring(pkg));
        assert!(wants_package_graphics_path(pkg));
        assert!(authoring_layers_align(pkg, pkg, 0).unwrap());
        let out = nudge_authoring_layers(pkg, pkg, 0, &[0], 2.0, -1.0).unwrap();
        assert!(
            out.contains("(circle 107 147.5 40)"),
            "package circle should nudge in place: {out}"
        );

        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![Shape::Circle(Circle {
                x_mm: 105.0,
                y_mm: 148.5,
                radius_mm: 40.0,
                fill: Color::BLACK,
            })],
        });
        let (_, shapes) = flatten_page(&doc, 0).expect("page");
        let layers = resolve_preview_layers(pkg, 0, &shapes);
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].kind, "circle");
        assert!(layers[0].byte_end > layers[0].byte_start);
        assert!(!layers[0].label.contains("read-only"));
        assert_eq!(
            resolve_preview_size_targets(pkg, 0, shapes.len()),
            vec![SizeTarget::CircleR(0)]
        );
    }

    /// LL24: live-layout demos are package-path sources (GUI open → wants_package).
    #[test]
    fn live_layout_demos_want_package_graphics_path() {
        let layout = include_str!("../../../examples/pkg_live_layout.rpx");
        let math = include_str!("../../../examples/pkg_live_math.rpx");
        let profile = include_str!("../../../examples/pkg_math_profile_v2.rpx");
        for (name, src) in [
            ("pkg_live_layout", layout),
            ("pkg_live_math", math),
            ("pkg_math_profile_v2", profile),
        ] {
            assert!(
                wants_package_graphics_path(src),
                "{name}: authoring should match package import heuristics"
            );
            let expanded = reciplexa_macro::expand_source(src)
                .unwrap_or_else(|e| panic!("{name} expand: {e:?}"));
            assert!(
                wants_package_graphics_path(&expanded),
                "{name}: GUI should open via package path (wants_package)"
            );
        }
    }

    #[test]
    fn interim_page_fixture_nudge_still_works() {
        let src = include_str!("../../reciplexa-lower/tests/fixtures/interim_page.rpx");
        assert!(!is_package_shaped_authoring(src));
        let out = nudge_authoring_layers(src, src, 0, &[0], 2.0, -1.0).unwrap();
        assert!(
            out.contains("(translate 2 -1 (circle 105 148.5 40))")
                || out.contains("(circle 107 147.5 40)"),
            "{out}"
        );
    }

    #[test]
    fn n5_3_resolve_preview_layers_interim_and_world_kinds() {
        use reciplexa_scene::{
            Color, Document, Image, Line, Page, PaperSize, Polygon, Shape, Text,
        };
        use reciplexa_view::flatten_page;

        let interim = "(page a4 (circle 1 2 3))";
        let doc = Document::single_page(Page {
            paper: PaperSize::a4(),
            shapes: vec![
                Shape::Circle(reciplexa_scene::Circle {
                    x_mm: 1.0,
                    y_mm: 2.0,
                    radius_mm: 3.0,
                    fill: Color::BLACK,
                }),
                Shape::Text(Text {
                    x_mm: 10.0,
                    y_mm: 20.0,
                    size_mm: 8.0,
                    width_mm: None,
                    height_mm: None,
                    content: "hi".into(),
                    fill: Color::BLACK,
                }),
                Shape::Line(Line {
                    x1_mm: 0.0,
                    y1_mm: 0.0,
                    x2_mm: 10.0,
                    y2_mm: 0.0,
                    stroke: Color::BLACK,
                    width_mm: 0.5,
                }),
                Shape::Polygon(Polygon {
                    points_mm: vec![(0.0, 0.0), (1.0, 0.0), (0.5, 1.0)],
                    fill: Color::BLACK,
                }),
                Shape::Image(Image {
                    path: "figures/demo.png".into(),
                    x_mm: 0.0,
                    y_mm: 0.0,
                    width_mm: 10.0,
                    height_mm: 10.0,
                }),
            ],
        });
        let (_, shapes) = flatten_page(&doc, 0).unwrap();
        // Interim CST: prefer collect_layers when counts match; mismatch → scene fallback.
        let layers = resolve_preview_layers(interim, 0, &shapes);
        assert_eq!(layers.len(), shapes.len());
        let kinds: Vec<_> = layers_from_world_shapes(&shapes)
            .into_iter()
            .map(|l| l.kind)
            .collect();
        assert!(kinds.contains(&"circle".into()));
        assert!(kinds.contains(&"text".into()));
        let text_label = layers_from_world_shapes(&shapes)
            .into_iter()
            .find(|l| l.kind == "text")
            .expect("text layer")
            .label;
        assert!(
            text_label.contains("text \"hi\""),
            "read-only text label should include content prefix: {text_label}"
        );
        assert!(kinds.contains(&"path".into()) || kinds.contains(&"polygon".into()));
        assert!(kinds.contains(&"image".into()));
        assert_eq!(readonly_size_targets(3).len(), 3);
        let sizes = resolve_preview_size_targets(interim, 0, shapes.len());
        assert_eq!(sizes.len(), shapes.len());
    }

    #[test]
    fn live_math_preview_layers_are_math_tree() {
        let src = include_str!("../../../examples/pkg_live_math.rpx");
        assert!(is_live_layout_authoring(src));
        let layers = resolve_preview_layers(src, 0, &[]);
        assert!(layers.iter().any(|l| l.kind == "math-delimiter"));
        assert!(layers.iter().any(|l| l.kind == "math-fraction"));
        assert!(layers.iter().any(|l| l.kind == "math-scripts"));
        assert!(layers.iter().filter(|l| l.kind == "math-symbol").count() >= 3);
        assert!(layers.iter().all(|l| !l.label.contains("read-only")));
        let err = nudge_authoring_layers(src, src, 0, &[0], 1.0, 0.0).unwrap_err();
        assert!(
            err.message.contains("math tree") || err.message.contains("properties"),
            "{}",
            err.message
        );
        assert!(layers
            .iter()
            .any(|l| l.kind.starts_with("math-") && l.depth > 0));
    }

    #[test]
    fn text_line_preview_nests_glyph_under_text_parent() {
        use reciplexa::pipeline::document_from_source;
        use reciplexa_view::flatten_page;

        let src = include_str!("../../../examples/text_line.rpx");
        let doc = document_from_source(src).expect("text_line ingest");
        let (_, shapes) = flatten_page(&doc, 0).expect("page");
        let layers = resolve_preview_layers(src, 0, &shapes);
        let text = layers
            .iter()
            .find(|l| l.kind == "text" && !l.glyph_child)
            .expect("text parent");
        assert!(layers
            .iter()
            .any(|l| l.glyph_child && l.authoring_index == text.authoring_index));
        assert!(
            reciplexa_lower::layers_cover_shapes(&layers, shapes.len()),
            "text_line preview must map every scene shape"
        );
        let out = nudge_authoring_layers(src, src, 0, &[text.authoring_index], 5.0, -3.0).unwrap();
        assert!(
            out.contains("(text 35 257 8") || out.contains("(text 35 257.0 8"),
            "{out}"
        );
        assert!(out.contains("\"Reciplexa\""));
        assert!(layers.iter().any(|l| l.kind == "circle" && !l.glyph_child));
    }

    #[test]
    fn vertical_demo_preview_groups_per_glyph_under_samples() {
        use reciplexa::pipeline::document_from_source;
        use reciplexa_lower::parent_layer_index;
        use reciplexa_view::flatten_page;

        let src = include_str!("../../../examples/pkg_vert.rpx");
        let doc = document_from_source(src).expect("pkg_vert ingest");
        let (_, shapes) = flatten_page(&doc, 0).expect("page");
        let layers = resolve_preview_layers(src, 0, &shapes);
        assert!(layers.iter().any(|l| l.kind == "vert-sample"));
        let sample = layers
            .iter()
            .find(|l| l.kind == "vert-sample" && l.text_content.as_deref() == Some("縦書き"))
            .expect("縦書き parent");
        let kids: Vec<_> = layers
            .iter()
            .filter(|l| l.glyph_child && l.authoring_index == sample.authoring_index)
            .collect();
        assert_eq!(kids.len(), 3, "縦書き should have 3 glyph children");
        let heading = layers
            .iter()
            .find(|l| l.kind == "doc-heading")
            .expect("heading parent");
        assert!(
            layers
                .iter()
                .any(|l| l.glyph_child && l.authoring_index == heading.authoring_index),
            "heading should nest per-letter GlyphRuns"
        );
        let ruby = layers
            .iter()
            .find(|l| l.kind == "vert-ruby")
            .expect("ruby parent");
        assert!(
            layers
                .iter()
                .filter(|l| l.glyph_child && l.authoring_index == ruby.authoring_index)
                .count()
                >= 2,
            "ruby annotation+base should be children"
        );
        assert!(layers.iter().any(|l| l.shape_index.is_some()));
        if let Some(si) = kids[0].shape_index {
            assert_eq!(
                parent_layer_index(&layers, si)
                    .and_then(|p| layers.get(p).map(|l| l.kind.as_str())),
                Some("vert-sample")
            );
        }
        let out = nudge_authoring_layers(src, src, 0, &[sample.authoring_index], 2.0, 0.0).unwrap();
        assert!(out.contains("(nudge (list"), "{out}");
    }

    #[test]
    fn preview_union_hit_selects_authoring_parent_between_glyphs() {
        use reciplexa::pipeline::document_from_source;
        use reciplexa_view::{flatten_page, hit_test_shapes};

        let src = include_str!("../../../examples/pkg_vert.rpx");
        let doc = document_from_source(src).expect("pkg_vert ingest");
        let (_, shapes) = flatten_page(&doc, 0).expect("page");
        let layers = resolve_preview_layers(src, 0, &shapes);
        let sample = layers
            .iter()
            .find(|l| l.kind == "vert-sample" && l.text_content.as_deref() == Some("ABC"))
            .expect("ABC parent");
        let idxs = shape_indices_for_authoring(&layers, &[sample.authoring_index]);
        let bounds = union_bounds_mm(&shapes, &idxs).expect("ABC union");
        let parent_row = layers
            .iter()
            .position(|l| !l.glyph_child && l.authoring_index == sample.authoring_index)
            .expect("ABC row");
        let (x0, y0, x1, y1) = bounds;
        let mut gap: Option<(f64, f64)> = None;
        let mut i = 0i32;
        while i <= 8 && gap.is_none() {
            let t = f64::from(i) / 8.0;
            let x = x0 + (x1 - x0) * 0.5;
            let y = y0 + (y1 - y0) * t;
            if point_in_aabb(x, y, bounds) && hit_test_shapes(&shapes, x, y).is_none() {
                gap = Some((x, y));
            }
            i += 1;
        }
        let (hx, hy) = gap.unwrap_or(((x0 + x1) * 0.5, (y0 + y1) * 0.5));
        assert_eq!(
            hit_test_authoring_parent(&layers, &shapes, hx, hy),
            Some(parent_row),
            "column union must be grabbable even when the pointer misses glyph ink"
        );

        let text_src = include_str!("../../../examples/text_line.rpx");
        let text_doc = document_from_source(text_src).expect("text_line ingest");
        let (_, text_shapes) = flatten_page(&text_doc, 0).expect("page");
        let text_layers = resolve_preview_layers(text_src, 0, &text_shapes);
        let text = text_layers
            .iter()
            .find(|l| l.kind == "text" && !l.glyph_child)
            .expect("text parent");
        let text_idxs = shape_indices_for_authoring(&text_layers, &[text.authoring_index]);
        let tb = union_bounds_mm(&text_shapes, &text_idxs).expect("text union");
        let text_row = text_layers
            .iter()
            .position(|l| !l.glyph_child && l.authoring_index == text.authoring_index)
            .expect("text row");
        let (tx, ty) = ((tb.0 + tb.2) * 0.5, (tb.1 + tb.3) * 0.5);
        assert_eq!(
            hit_test_authoring_parent(&text_layers, &text_shapes, tx, ty),
            Some(text_row)
        );
    }
}
