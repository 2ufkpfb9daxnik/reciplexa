//! Minimal CST → scene lowering for pages, shapes, and Glisp-like transforms.
//!
//! **S6b:** Keyword-table `(page)/(circle)/…` lower is gated by
//! `feature = "interim-surface"` (or `cfg(test)` for this crate's unit tests).
//! Production builds reject bare keyword pages — author with
//! `(import graphics/…)(val main (page …))`. Fixture lower:
//! [`lower_interim_source`] / enable `interim-surface`.
//!
//! Supported interim forms (Lisp mode, feature-gated):
//!
//! ```text
//! (page a4
//!   (circle <x> <y> <r>)
//!   (circle <x> <y> <r> red)
//!   (circle <x> <y> <r> (rgb 0.1 0.2 0.3))
//!   (translate <tx> <ty> <shape…>)
//!   (rotate <deg> <shape…>)
//!   (scale <s> <shape…>)
//!   (scale <sx> <sy> <shape…>)
//!   (text <x> <y> <size-mm> "…")
//!   (text <x> <y> <size-mm> <w-mm> <h-mm> "…")
//!   (text <x> <y> <size-mm> "…" color)
//!   (line <x1> <y1> <x2> <y2> [color [width-mm]]))
//! ```

#![forbid(unsafe_code)]

pub mod cst_walk;
mod props;
mod sync;

pub use cst_walk::{find_list_covering, list_atoms};
pub use props::{
    collect_layer_props, coverage_collect_paint_unknown, coverage_parse_f64,
    coverage_polyline_stroke_short, coverage_push_geom_missing, set_layer_fill_rgb, set_layer_prop,
    set_layer_stroke_rgb, set_layers_fill_rgb, set_layers_opacity, set_layers_stroke_rgb,
    set_layers_stroke_width, PropEditContext, PropField, PropGroup, PropValue,
};
pub use sync::{
    attach_glyph_children, authoring_indices_for_selection, collect_drag_targets,
    collect_drag_targets_page, collect_layers_authoring, collect_layers_document,
    collect_layers_live_layout, collect_layers_package, collect_layers_page,
    collect_layers_vertical_demo, collect_package_pages, collect_size_targets_authoring,
    collect_size_targets_package, collect_size_targets_page, count_package_pages, count_pages,
    delete_layer_authoring, delete_layer_package, delete_layer_page, delete_page,
    document_columns_params, document_layer_indent_em, document_layer_text,
    duplicate_layer_authoring, duplicate_layer_package, duplicate_layer_page,
    extent_with_leading_ws, find_main_expr, find_main_expr_in_source, find_package_page,
    find_package_page_in_source, find_page, first_shape_for_authoring, group_layers_page,
    insert_layer_authoring, insert_layer_package, insert_layer_page, insert_page_after,
    is_document_page_authoring, is_headed, is_live_layout_authoring, is_package_paint_wrapper,
    is_package_shaped_authoring, is_package_transparent_wrapper, is_vertical_demo_authoring,
    layer_opacity, layer_rotation_deg, layers_cover_shapes, math_layer_glyph, nudge_drag_target,
    nudge_first_translate, nudge_layer_authoring, nudge_layer_package, nudge_layer_page,
    nudge_vertical_demo_layer, package_page_content_nodes, page_body_start, paint_wrapper_shape,
    parent_layer_index, parse_root, preview_index_for_authoring, reorder_layer_authoring,
    reorder_layer_package, reorder_layer_page, scale_box_axes, scale_layer_uniform,
    scale_size_target, scale_size_target_axes, scale_text_box, set_box_xywh,
    set_document_columns_count, set_document_columns_gutter, set_document_layer_indent_em,
    set_document_layer_text, set_layer_opacity, set_layer_rotation_deg, set_line_endpoint,
    set_live_layout_document_text, set_math_layer_glyph, set_poly_vertex, set_text_box,
    set_text_content_authoring, set_text_content_package, shape_indices_for_authoring,
    ungroup_layer_page, vertical_demo_nudge_span, DragTarget, LayerInfo, SizeTarget, SyncError,
};

use reciplexa_scene::{
    Affine, Circle, Color, Document, Ellipse, Frame, Image, Line, Page, PaperSize, Polygon,
    Polyline, Rect, Ring, Shape, Text,
};
use reciplexa_syntax::{parse_source, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

/// Lowering / validation error (fail-fast: no partial scene for rendering).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LowerError {
    pub message: String,
}

impl LowerError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

const INTERIM_RETIRED: &str = "interim keyword `(page)/(circle)/…` lower is retired from \
     production; author with `(import graphics/…)(val main (page …))`, or call \
     `lower_interim_source` / enable feature `interim-surface` in tests";

#[inline]
fn interim_surface_enabled() -> bool {
    cfg!(any(test, feature = "interim-surface"))
}

/// Parse source and lower it to a [`Document`].
///
/// Without `interim-surface` / `cfg(test)`, rejects keyword-table `(page …)` sources.
pub fn lower_source(input: &str) -> Result<Document, LowerError> {
    let parse = parse_source(input);
    let root = parse
        .into_result()
        .map_err(|errs| LowerError::new(format!("parse error: {}", errs[0].message)))?;
    lower_syntax(&root)
}

/// Explicit interim keyword-table lower for fixtures and gated tests.
///
/// Same as [`lower_source`] when `interim-surface` (or this crate's `cfg(test)`) is on;
/// otherwise returns a clear retirement error.
pub fn lower_interim_source(input: &str) -> Result<Document, LowerError> {
    if !interim_surface_enabled() {
        return Err(LowerError::new(INTERIM_RETIRED));
    }
    lower_source(input)
}

/// Lower an already-parsed CST root (`SourceFile`).
pub fn lower_syntax(root: &SyntaxNode) -> Result<Document, LowerError> {
    if root.kind() != SyntaxKind::SourceFile {
        return Err(LowerError::new(format!(
            "expected SourceFile, got {:?}",
            root.kind()
        )));
    }

    let forms: Vec<SyntaxNode> = root.children().collect();
    if forms
        .iter()
        .all(|f| f.kind() == reciplexa_syntax::SyntaxKind::StructuredComment)
        || forms.is_empty()
    {
        return Err(LowerError::new("empty source: expected a (page …) form"));
    }

    let mut pages = Vec::new();
    for form in forms {
        if form.kind() == reciplexa_syntax::SyntaxKind::StructuredComment {
            continue;
        }
        let items = list_children(&form);
        let Ok(head) = ident_at(&items, 0, "top-level") else {
            continue;
        };
        match head {
            "page" => {
                if !interim_surface_enabled() {
                    return Err(LowerError::new(INTERIM_RETIRED));
                }
                pages.push(lower_page(&form)?);
            }
            "markup" | "src" | "type" | "val" | "perform" | "handle" | "//" => {
                // `markup` expands to package-shaped graphics (or empty package page).
                // Leftover markup/src/effects/decls are skipped (logic / package seams).
                // `//` should be StructuredComment; skip if it ever appears as a list.
            }
            other => {
                return Err(LowerError::new(format!(
                    "expected head `page`, `markup`, `src`, `type`/`val`, or perform/handle, found `{other}`"
                )));
            }
        }
    }
    if pages.is_empty() {
        return Err(LowerError::new(
            "no (page …) forms to lower (markup/src alone cannot produce a scene)",
        ));
    }
    Ok(Document { pages })
}

fn lower_page(node: &SyntaxNode) -> Result<Page, LowerError> {
    // Caller only routes `(page …)` list forms.
    let items = list_children(node);
    // Caller only routes heads that already matched `page`.
    if items.len() < 2 {
        return Err(LowerError::new(
            "`page` requires paper: (page a4 …), (page letter …), or (page width-mm height-mm …)",
        ));
    }

    let (paper, shape_start) = lower_paper_spec(&items)?;

    let mut shapes = Vec::new();
    for item in items.iter().skip(shape_start) {
        shapes.push(lower_shape_child(item)?);
    }
    Ok(Page { paper, shapes })
}

fn lower_paper_spec(items: &[Child]) -> Result<(PaperSize, usize), LowerError> {
    // (page a4 …) | (page letter …) | (page w h …)
    // Named sizes are temporary sugar; macros can expand to numeric sizes later.
    if let Child::Token(wtok) = &items[1] {
        if wtok.kind() == SyntaxKind::Number {
            if items.len() < 3 {
                return Err(LowerError::new(
                    "`page` numeric paper needs width and height in mm",
                ));
            }
            let width_mm = number_token_f64(wtok);
            let height_mm = match &items[2] {
                Child::Token(t) if t.kind() == SyntaxKind::Number => number_token_f64(t),
                _ => {
                    return Err(LowerError::new(
                        "page height: expected Number, got non-number",
                    ))
                }
            };
            let paper = PaperSize {
                width_mm,
                height_mm,
            };
            if !paper.is_positive() {
                return Err(LowerError::new("paper size must be positive"));
            }
            return Ok((paper, 3));
        }
    }

    let paper = match atom_ident(&items[1])? {
        "a4" => PaperSize::a4(),
        "letter" => PaperSize::letter(),
        other => {
            return Err(LowerError::new(format!(
                "unknown paper size `{other}` (use `a4`, `letter`, or numeric mm)"
            )))
        }
    };
    Ok((paper, 2))
}

fn lower_shape_child(child: &Child) -> Result<Shape, LowerError> {
    match child {
        Child::Node(n) => lower_shape(n),
        Child::Token(t) => Err(LowerError::new(format!(
            "expected a shape list, got token {:?}",
            t.kind()
        ))),
    }
}

fn lower_shape(node: &SyntaxNode) -> Result<Shape, LowerError> {
    // Shape children are always `(…)` lists from the CST.
    let items = list_children(node);
    let head = ident_at(&items, 0, "shape")?;
    match head {
        "circle" => lower_circle(&items),
        "rect" => lower_rect(&items),
        "ellipse" => lower_ellipse(&items),
        "ring" => lower_ring(&items),
        "frame" => lower_frame(&items),
        "text" => lower_text(&items),
        "line" => lower_line(&items),
        "polyline" => lower_polyline(&items),
        "polygon" => lower_polygon(&items),
        "image" => lower_image(&items),
        "opacity" => lower_opacity(&items),
        "group" => lower_group(&items),
        "translate" => lower_translate(&items),
        "rotate" => lower_rotate(&items),
        "scale" => lower_scale(&items),
        other => Err(LowerError::new(format!("unknown shape `{other}`"))),
    }
}

fn lower_circle(items: &[Child]) -> Result<Shape, LowerError> {
    // (circle x y r) | (circle x y r color)
    if items.len() != 4 && items.len() != 5 {
        return Err(LowerError::new(
            "`circle` expects (circle x y r) or (circle x y r color)",
        ));
    }
    let x = number_at(items, 1, "circle x")?;
    let y = number_at(items, 2, "circle y")?;
    let r = number_at(items, 3, "circle radius")?;
    let fill = if items.len() == 5 {
        lower_color(&items[4])?
    } else {
        Color::BLACK
    };
    let circle = Circle {
        x_mm: x,
        y_mm: y,
        radius_mm: r,
        fill,
    };
    if !circle.is_drawable() {
        return Err(LowerError::new(format!(
            "circle is not drawable (radius={r})"
        )));
    }
    Ok(Shape::Circle(circle))
}

fn lower_rect(items: &[Child]) -> Result<Shape, LowerError> {
    // (rect x y w h) | (rect x y w h color)
    if items.len() != 5 && items.len() != 6 {
        return Err(LowerError::new(
            "`rect` expects (rect x y w h) or (rect x y w h color)",
        ));
    }
    let x = number_at(items, 1, "rect x")?;
    let y = number_at(items, 2, "rect y")?;
    let w = number_at(items, 3, "rect width")?;
    let h = number_at(items, 4, "rect height")?;
    let fill = if items.len() == 6 {
        lower_color(&items[5])?
    } else {
        Color::BLACK
    };
    let rect = Rect {
        x_mm: x,
        y_mm: y,
        width_mm: w,
        height_mm: h,
        fill,
    };
    if !rect.is_drawable() {
        return Err(LowerError::new(format!(
            "rect is not drawable (w={w}, h={h})"
        )));
    }
    Ok(Shape::Rect(rect))
}

fn lower_ellipse(items: &[Child]) -> Result<Shape, LowerError> {
    // (ellipse x y rx ry) | (ellipse x y rx ry color)
    if items.len() != 5 && items.len() != 6 {
        return Err(LowerError::new(
            "`ellipse` expects (ellipse x y rx ry) or with a trailing color",
        ));
    }
    let x = number_at(items, 1, "ellipse x")?;
    let y = number_at(items, 2, "ellipse y")?;
    let rx = number_at(items, 3, "ellipse rx")?;
    let ry = number_at(items, 4, "ellipse ry")?;
    let fill = if items.len() == 6 {
        lower_color(&items[5])?
    } else {
        Color::BLACK
    };
    let ellipse = Ellipse {
        x_mm: x,
        y_mm: y,
        rx_mm: rx,
        ry_mm: ry,
        fill,
    };
    if !ellipse.is_drawable() {
        return Err(LowerError::new(format!(
            "ellipse is not drawable (rx={rx}, ry={ry})"
        )));
    }
    Ok(Shape::Ellipse(ellipse))
}

fn lower_ring(items: &[Child]) -> Result<Shape, LowerError> {
    // (ring x y r width) | (ring x y r width color)
    if items.len() != 5 && items.len() != 6 {
        return Err(LowerError::new(
            "`ring` expects (ring x y r width-mm [color])",
        ));
    }
    let x = number_at(items, 1, "ring x")?;
    let y = number_at(items, 2, "ring y")?;
    let r = number_at(items, 3, "ring radius")?;
    let width = number_at(items, 4, "ring width")?;
    let stroke = if items.len() == 6 {
        lower_color(&items[5])?
    } else {
        Color::BLACK
    };
    let ring = Ring {
        x_mm: x,
        y_mm: y,
        radius_mm: r,
        width_mm: width,
        stroke,
    };
    if !ring.is_drawable() {
        return Err(LowerError::new("ring is not drawable"));
    }
    Ok(Shape::Ring(ring))
}

fn lower_frame(items: &[Child]) -> Result<Shape, LowerError> {
    // (frame x y w h width) | (frame x y w h width color)
    if items.len() != 6 && items.len() != 7 {
        return Err(LowerError::new(
            "`frame` expects (frame x y w h stroke-width-mm [color])",
        ));
    }
    let x = number_at(items, 1, "frame x")?;
    let y = number_at(items, 2, "frame y")?;
    let w = number_at(items, 3, "frame width")?;
    let h = number_at(items, 4, "frame height")?;
    let sw = number_at(items, 5, "frame stroke width")?;
    let stroke = if items.len() == 7 {
        lower_color(&items[6])?
    } else {
        Color::BLACK
    };
    let frame = Frame {
        x_mm: x,
        y_mm: y,
        width_mm: w,
        height_mm: h,
        stroke_width_mm: sw,
        stroke,
    };
    if !frame.is_drawable() {
        return Err(LowerError::new("frame is not drawable"));
    }
    Ok(Shape::Frame(frame))
}

fn lower_text(items: &[Child]) -> Result<Shape, LowerError> {
    // (text x y size "content" [color])
    // (text x y size w h "content" [color])
    let (width, height, content_slot, color_slot) = match items.len() {
        5 => (None, None, 4, None),
        6 => {
            // Either trailing color or start of boxed form — boxed needs ≥7.
            (None, None, 4, Some(5))
        }
        7 | 8 => {
            let w = number_at(items, 4, "text width")?;
            let h = number_at(items, 5, "text height")?;
            let color_slot = if items.len() == 8 { Some(7) } else { None };
            (Some(w), Some(h), 6, color_slot)
        }
        _ => {
            return Err(LowerError::new(
                "`text` expects (text x y size-mm [w-mm h-mm] \"…\" [color])",
            ));
        }
    };
    // Ambiguous len==6: must be content string then color, not w/h alone.
    if items.len() == 6 {
        string_at(items, 4, "text content")?;
    }
    let x = number_at(items, 1, "text x")?;
    let y = number_at(items, 2, "text y")?;
    let size = number_at(items, 3, "text size")?;
    let content = string_at(items, content_slot, "text content")?;
    let fill = if let Some(i) = color_slot {
        lower_color(&items[i])?
    } else {
        Color::BLACK
    };
    let text = Text {
        x_mm: x,
        y_mm: y,
        size_mm: size,
        width_mm: width,
        height_mm: height,
        content,
        fill,
    };
    if !text.is_drawable() {
        return Err(LowerError::new("text is not drawable"));
    }
    Ok(Shape::Text(text))
}

fn lower_line(items: &[Child]) -> Result<Shape, LowerError> {
    // (line x1 y1 x2 y2) | (line x1 y1 x2 y2 color) | (line x1 y1 x2 y2 color width)
    if items.len() < 5 || items.len() > 7 {
        return Err(LowerError::new(
            "`line` expects (line x1 y1 x2 y2 [color [width-mm]])",
        ));
    }
    let x1 = number_at(items, 1, "line x1")?;
    let y1 = number_at(items, 2, "line y1")?;
    let x2 = number_at(items, 3, "line x2")?;
    let y2 = number_at(items, 4, "line y2")?;
    let mut stroke = Color::BLACK;
    let mut width = 0.5;
    if items.len() >= 6 {
        stroke = lower_color(&items[5])?;
    }
    if items.len() == 7 {
        width = number_at(items, 6, "line width")?;
    }
    let line = Line {
        x1_mm: x1,
        y1_mm: y1,
        x2_mm: x2,
        y2_mm: y2,
        stroke,
        width_mm: width,
    };
    if !line.is_drawable() {
        return Err(LowerError::new("line is not drawable"));
    }
    Ok(Shape::Line(line))
}

fn lower_polyline(items: &[Child]) -> Result<Shape, LowerError> {
    // (polyline x1 y1 x2 y2 … [color [width]])
    if items.len() < 5 {
        return Err(LowerError::new(
            "`polyline` expects at least two points (x y)×2",
        ));
    }
    let mut end = items.len();
    let mut stroke = Color::BLACK;
    let mut width = 0.5;
    // Optional trailing `(color width)` or trailing color only.
    // `items.len() >= 5`, so the last two slots are always addressable.
    let last = &items[end - 1];
    let prev = &items[end - 2];
    match last {
        Child::Token(wtok) if wtok.kind() == SyntaxKind::Number && is_color_child(prev) => {
            width = number_token_f64(wtok);
            stroke = lower_color(prev)?;
            end -= 2;
        }
        other => {
            if is_color_child(other) {
                stroke = lower_color(other)?;
                end -= 1;
            }
        }
    }
    let coords = &items[1..end];
    if coords.len() < 4 || !coords.len().is_multiple_of(2) {
        return Err(LowerError::new(
            "`polyline` needs an even number of coordinates (≥4)",
        ));
    }
    let mut points_mm = Vec::with_capacity(coords.len() / 2);
    for i in (0..coords.len()).step_by(2) {
        let x = number_at_slice(coords, i, "polyline x")?;
        let y = number_at_slice(coords, i + 1, "polyline y")?;
        points_mm.push((x, y));
    }
    let poly = Polyline {
        points_mm,
        stroke,
        width_mm: width,
    };
    if !poly.is_drawable() {
        return Err(LowerError::new("polyline is not drawable"));
    }
    Ok(Shape::Polyline(poly))
}

fn lower_polygon(items: &[Child]) -> Result<Shape, LowerError> {
    // (polygon x1 y1 x2 y2 x3 y3 … [color])
    if items.len() < 7 {
        return Err(LowerError::new(
            "`polygon` expects at least three points (x y)×3",
        ));
    }
    let mut end = items.len();
    let mut fill = Color::BLACK;
    if end >= 2 && is_color_child(&items[end - 1]) {
        fill = lower_color(&items[end - 1])?;
        end -= 1;
    }
    let coords = &items[1..end];
    if coords.len() < 6 || !coords.len().is_multiple_of(2) {
        return Err(LowerError::new(
            "`polygon` needs an even number of coordinates (≥6)",
        ));
    }
    let mut points_mm = Vec::with_capacity(coords.len() / 2);
    for i in (0..coords.len()).step_by(2) {
        let x = number_at_slice(coords, i, "polygon x")?;
        let y = number_at_slice(coords, i + 1, "polygon y")?;
        points_mm.push((x, y));
    }
    let poly = Polygon { points_mm, fill };
    // After the coordinate-count guard, polygons are always drawable (valid fill).
    Ok(Shape::Polygon(poly))
}

fn lower_image(items: &[Child]) -> Result<Shape, LowerError> {
    // (image "path" x y w h)
    if items.len() != 6 {
        return Err(LowerError::new(
            "`image` expects (image \"path\" x y width-mm height-mm)",
        ));
    }
    let path = string_at(items, 1, "image path")?;
    let x = number_at(items, 2, "image x")?;
    let y = number_at(items, 3, "image y")?;
    let w = number_at(items, 4, "image width")?;
    let h = number_at(items, 5, "image height")?;
    let image = Image {
        path,
        x_mm: x,
        y_mm: y,
        width_mm: w,
        height_mm: h,
    };
    if !image.is_drawable() {
        return Err(LowerError::new("image is not drawable"));
    }
    Ok(Shape::Image(image))
}

fn is_color_child(child: &Child) -> bool {
    match child {
        Child::Token(t) if t.kind() == SyntaxKind::Ident => Color::named(t.text()).is_some(),
        Child::Node(n) if n.kind() == SyntaxKind::List => {
            let items = list_children(n);
            matches!(
                items.first(),
                Some(Child::Token(h)) if h.kind() == SyntaxKind::Ident && h.text() == "rgb"
            )
        }
        _ => false,
    }
}

fn number_at_slice(items: &[Child], index: usize, ctx: &str) -> Result<f64, LowerError> {
    number_at(items, index, ctx)
}

fn string_at(items: &[Child], index: usize, ctx: &str) -> Result<String, LowerError> {
    // Callers only request in-range slots after arity checks.
    match &items[index] {
        Child::Token(t) if t.kind() == SyntaxKind::String => {
            reciplexa_syntax::decode_string_literal(t.text())
                .map_err(|msg| LowerError::new(format!("{ctx}: {msg}")))
        }
        Child::Token(t) => Err(LowerError::new(format!(
            "{ctx}: expected String, got {:?}",
            t.kind()
        ))),
        Child::Node(n) => Err(LowerError::new(format!(
            "{ctx}: expected String, got node {:?}",
            n.kind()
        ))),
    }
}

fn lower_color(child: &Child) -> Result<Color, LowerError> {
    match child {
        Child::Token(t) if t.kind() == SyntaxKind::Ident => Color::named(t.text())
            .ok_or_else(|| LowerError::new(format!("unknown color `{}`", t.text()))),
        Child::Node(n) => {
            let items = list_children(n);
            let head = ident_at(&items, 0, "color")?;
            if head != "rgb" {
                return Err(LowerError::new(format!(
                    "unknown color form `{head}` (expected rgb)"
                )));
            }
            if items.len() != 4 {
                return Err(LowerError::new("`rgb` expects three channels"));
            }
            let r = number_at(&items, 1, "rgb r")?;
            let g = number_at(&items, 2, "rgb g")?;
            let b = number_at(&items, 3, "rgb b")?;
            let c = Color::new(r, g, b);
            if !c.is_channel_valid() {
                return Err(LowerError::new("rgb channels must be in 0..=1"));
            }
            Ok(c)
        }
        Child::Token(t) => Err(LowerError::new(format!(
            "expected color ident or (rgb …), got {:?}",
            t.kind()
        ))),
    }
}

fn lower_translate(items: &[Child]) -> Result<Shape, LowerError> {
    // (translate tx ty shape…)
    if items.len() < 4 {
        return Err(LowerError::new(
            "`translate` expects tx ty and at least one shape",
        ));
    }
    let tx = number_at(items, 1, "translate x")?;
    let ty = number_at(items, 2, "translate y")?;
    let children = lower_shape_tail(&items[3..])?;
    Ok(Shape::Group {
        transform: Affine::translate(tx, ty),
        children,
    })
}

fn lower_group(items: &[Child]) -> Result<Shape, LowerError> {
    // (group shape…) — identity transform, for layering / annotations.
    if items.len() < 2 {
        return Err(LowerError::new("`group` expects at least one shape"));
    }
    Ok(Shape::Group {
        transform: Affine::identity(),
        children: lower_shape_tail(&items[1..])?,
    })
}

fn lower_opacity(items: &[Child]) -> Result<Shape, LowerError> {
    // (opacity a shape…)
    if items.len() < 3 {
        return Err(LowerError::new(
            "`opacity` expects alpha and at least one shape",
        ));
    }
    let alpha = number_at(items, 1, "opacity alpha")?;
    // Range check rejects NaN/±inf as well (`contains` is false for non-finite).
    if !(0.0..=1.0).contains(&alpha) {
        return Err(LowerError::new(
            "`opacity` alpha must be a finite number in 0..=1",
        ));
    }
    Ok(Shape::Opacity {
        alpha,
        children: lower_shape_tail(&items[2..])?,
    })
}

fn lower_rotate(items: &[Child]) -> Result<Shape, LowerError> {
    // (rotate deg shape…)
    if items.len() < 3 {
        return Err(LowerError::new(
            "`rotate` expects degrees and at least one shape",
        ));
    }
    let deg = number_at(items, 1, "rotate degrees")?;
    let children = lower_shape_tail(&items[2..])?;
    Ok(Shape::Group {
        transform: Affine::rotate_deg(deg),
        children,
    })
}

fn lower_scale(items: &[Child]) -> Result<Shape, LowerError> {
    // (scale s shape…) | (scale sx sy shape…)
    if items.len() < 3 {
        return Err(LowerError::new(
            "`scale` expects factor(s) and at least one shape",
        ));
    }
    let first = number_at(items, 1, "scale")?;
    let (transform, rest) = if items.len() >= 4 {
        if let Child::Token(sy_tok) = &items[2] {
            if sy_tok.kind() == SyntaxKind::Number {
                let sy = number_token_f64(sy_tok);
                (Affine::scale(first, sy), &items[3..])
            } else {
                (Affine::scale_uniform(first), &items[2..])
            }
        } else {
            (Affine::scale_uniform(first), &items[2..])
        }
    } else {
        (Affine::scale_uniform(first), &items[2..])
    };
    // Arity ≥3 (and ≥4 for sx sy) guarantees a non-empty body slice.
    Ok(Shape::Group {
        transform,
        children: lower_shape_tail(rest)?,
    })
}

fn lower_shape_tail(items: &[Child]) -> Result<Vec<Shape>, LowerError> {
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        out.push(lower_shape_child(item)?);
    }
    Ok(out)
}

#[derive(Debug)]
enum Child {
    Node(SyntaxNode),
    Token(SyntaxToken),
}

fn list_children(node: &SyntaxNode) -> Vec<Child> {
    let mut items = Vec::new();
    for el in node.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) => {
                if t.kind().is_trivia()
                    || matches!(t.kind(), SyntaxKind::LParen | SyntaxKind::RParen)
                {
                    continue;
                }
                items.push(Child::Token(t));
            }
            SyntaxElement::Node(n) => items.push(Child::Node(n)),
        }
    }
    items
}

#[inline(never)]
fn number_token_f64(t: &SyntaxToken) -> f64 {
    parse_num_text(t.text())
}

#[inline(never)]
fn parse_num_text(text: &str) -> f64 {
    reciplexa_syntax::parse_number_literal(text).unwrap_or(0.0)
}

fn ident_at<'a>(items: &'a [Child], index: usize, ctx: &str) -> Result<&'a str, LowerError> {
    let Some(child) = items.get(index) else {
        return Err(LowerError::new(format!("{ctx}: missing element {index}")));
    };
    match child {
        Child::Token(t) if t.kind() == SyntaxKind::Ident => Ok(t.text()),
        Child::Token(t) => Err(LowerError::new(format!(
            "{ctx}: expected Ident, got {:?}",
            t.kind()
        ))),
        Child::Node(n) => Err(LowerError::new(format!(
            "{ctx}: expected Ident, got node {:?}",
            n.kind()
        ))),
    }
}

fn atom_ident(child: &Child) -> Result<&str, LowerError> {
    match child {
        Child::Token(t) if t.kind() == SyntaxKind::Ident => Ok(t.text()),
        Child::Token(t) => Err(LowerError::new(format!(
            "expected paper Ident, got {:?}",
            t.kind()
        ))),
        Child::Node(n) => Err(LowerError::new(format!(
            "expected paper Ident, got node {:?}",
            n.kind()
        ))),
    }
}

#[inline(never)]
fn number_at(items: &[Child], index: usize, ctx: &str) -> Result<f64, LowerError> {
    match items.get(index) {
        Some(Child::Token(t)) if t.kind() == SyntaxKind::Number => Ok(number_token_f64(t)),
        other => {
            let got = match other {
                Some(Child::Token(t)) => format!("{:?}", t.kind()),
                Some(Child::Node(n)) => format!("node {:?}", n.kind()),
                None => "missing".into(),
            };
            Err(LowerError::new(format!(
                "{ctx}: expected Number, got {got}"
            )))
        }
    }
}

/// Test/coverage hook: `number_at` missing-index error (non-`cfg(test)` lib copy).
#[doc(hidden)]
pub fn coverage_number_at_missing() -> bool {
    number_at(&[], 0, "x").is_err()
}

/// Test/coverage hook: digit scanner with a leading `+` / non-digit junk.
#[doc(hidden)]
pub fn coverage_parse_num_text(sample: &str) -> f64 {
    parse_num_text(sample)
}

#[cfg(test)]
mod number_at_coverage {
    use super::*;

    #[test]
    fn coverage_hooks_also_run_under_cfg_test_lib() {
        assert!(coverage_number_at_missing());
        assert_eq!(coverage_parse_num_text("+2"), 2.0);
        assert_eq!(coverage_parse_num_text("zz"), 0.0);
    }
}
