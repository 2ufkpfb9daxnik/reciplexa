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
    collect_layers_package, collect_layers_page, collect_size_targets_package,
    collect_size_targets_page, is_package_shaped_authoring, nudge_layer_package, nudge_layer_page,
    LayerInfo, SizeTarget, SyncError,
};
use reciplexa_view::WorldShape;

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
pub fn layers_from_world_shapes(shapes: &[WorldShape]) -> Vec<LayerInfo> {
    shapes
        .iter()
        .enumerate()
        .map(|(i, shape)| {
            let kind = world_shape_kind(shape).to_string();
            let label = format!("{kind} (read-only #{})", i + 1);
            LayerInfo {
                kind,
                label,
                byte_start: 0,
                byte_end: 0,
                root_start: 0,
                root_end: 0,
            }
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

/// Prefer CST layers; package authoring uses package collectors; else scene rows.
pub fn resolve_preview_layers(
    expanded: &str,
    page_index: usize,
    shapes: &[WorldShape],
) -> Vec<LayerInfo> {
    if is_package_shaped_authoring(expanded) {
        match collect_layers_package(expanded, page_index) {
            Ok(layers) if layers.len() == shapes.len() => return layers,
            Ok(_) | Err(_) => return layers_from_world_shapes(shapes),
        }
    }
    match collect_layers_page(expanded, page_index) {
        Ok(layers) if layers.len() == shapes.len() => layers,
        Ok(_) | Err(_) => layers_from_world_shapes(shapes),
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
             (markup-generated pages are read-only — edit the markup block instead)",
        ));
    }
    match authoring_layers_align(authoring, expanded, page_index) {
        Ok(true) => {}
        Ok(false) => {
            return Err(SyncRefuse::new(
                "canvas move skipped: expanded layers are not editable in authoring source \
                 (markup-generated text is read-only — edit the markup block instead)",
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
        assert!(kinds.contains(&"path".into()) || kinds.contains(&"polygon".into()));
        assert!(kinds.contains(&"image".into()));
        assert_eq!(readonly_size_targets(3).len(), 3);
        let sizes = resolve_preview_size_targets(interim, 0, shapes.len());
        assert_eq!(sizes.len(), shapes.len());
    }
}
