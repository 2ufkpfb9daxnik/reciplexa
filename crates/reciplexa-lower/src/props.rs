//! Named property fields for a selected layer (Word-style shape properties).
//!
//! This is **not** the deferred “attribute button suite” (macro-driven option
//! buttons). It only exposes named arguments / layout knobs that rewrite CST.

use reciplexa_syntax::{
    format_drag_number, replace_token_text, SyntaxKind, SyntaxNode, SyntaxToken,
};

use crate::cst_walk::{find_list_covering, list_atoms, Child};
use crate::sync::{
    collect_layers_from_root, collect_size_targets_from_root, layer_opacity, layer_rotation_deg,
    nudge_layer_page, parse_root, scale_size_target_axes, set_layer_opacity,
    set_layer_rotation_deg, SyncError,
};

/// UI grouping for the properties panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropGroup {
    Layout,
    Transform,
    Fill,
    Stroke,
    Content,
    Geometry,
}

impl PropGroup {
    pub fn title(self) -> &'static str {
        match self {
            Self::Layout => "Layout",
            Self::Transform => "Transform",
            Self::Fill => "Fill color",
            Self::Stroke => "Stroke color",
            Self::Content => "Content",
            Self::Geometry => "Geometry",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropValue {
    Number(f64),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropField {
    pub id: String,
    /// Visible argument name in the panel (e.g. `x`, `fill.r`, `text`).
    pub label: String,
    pub group: PropGroup,
    pub value: PropValue,
    /// Slider range for numbers; `None` → plain drag value / text field.
    pub slider: Option<(f64, f64)>,
}

/// Extra context needed to apply layout / rotation edits.
#[derive(Debug, Clone, Copy)]
pub struct PropEditContext {
    /// Axis-aligned bounds in page mm: `(min_x, min_y, max_x, max_y)`.
    pub aabb_mm: (f64, f64, f64, f64),
    pub paper_w_mm: f64,
    pub paper_h_mm: f64,
}

/// Parse once and resolve the paint list for a flattened layer index.
fn layer_paint(
    src: &str,
    page_index: usize,
    flat_index: usize,
) -> Result<(SyntaxNode, SyntaxNode, String), SyncError> {
    let root = parse_root(src)?;
    let layers = collect_layers_from_root(&root, page_index)?;
    let layer = layers
        .get(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    // LayerInfo spans always come from list nodes in collect_layers_from_shape.
    let paint = find_list_covering(&root, layer.byte_start, layer.byte_end)
        .expect("paint span from collect_layers");
    Ok((root, paint, layer.kind.clone()))
}

/// Collect editable named fields for one flattened layer.
pub fn collect_layer_props(
    src: &str,
    page_index: usize,
    flat_index: usize,
    ctx: &PropEditContext,
) -> Result<Vec<PropField>, SyncError> {
    let (_root, paint, kind) = layer_paint(src, page_index, flat_index)?;

    let (x0, y0, x1, y1) = ctx.aabb_mm;
    let w = (x1 - x0).max(0.0);
    let h = (y1 - y0).max(0.0);
    let pw = ctx.paper_w_mm.max(1.0);
    let ph = ctx.paper_h_mm.max(1.0);

    let mut out = vec![
        num(
            "layout.x",
            "x",
            PropGroup::Layout,
            x0,
            Some((-pw, pw * 2.0)),
        ),
        num(
            "layout.y",
            "y",
            PropGroup::Layout,
            y0,
            Some((-ph, ph * 2.0)),
        ),
        num(
            "layout.w",
            "width",
            PropGroup::Layout,
            w,
            Some((0.5, pw * 2.0)),
        ),
        num(
            "layout.h",
            "height",
            PropGroup::Layout,
            h,
            Some((0.5, ph * 2.0)),
        ),
    ];

    let rot = layer_rotation_deg(src, page_index, flat_index).unwrap_or(0.0);
    let alpha = layer_opacity(src, page_index, flat_index).unwrap_or(1.0);
    out.push(num(
        "transform.rotation",
        "rotation",
        PropGroup::Transform,
        rot,
        Some((-180.0, 180.0)),
    ));
    out.push(num(
        "transform.opacity",
        "opacity",
        PropGroup::Transform,
        alpha,
        Some((0.0, 1.0)),
    ));

    collect_paint_props(&paint, &kind, &mut out);
    Ok(out)
}

/// Apply one property edit; rewrites `.rpx` source.
pub fn set_layer_prop(
    src: &str,
    page_index: usize,
    flat_index: usize,
    id: &str,
    value: &PropValue,
    ctx: &PropEditContext,
) -> Result<String, SyncError> {
    let (x0, y0, x1, y1) = ctx.aabb_mm;
    let w = (x1 - x0).max(1e-9);
    let h = (y1 - y0).max(1e-9);
    let cx = (x0 + x1) * 0.5;
    let cy = (y0 + y1) * 0.5;

    match id {
        "layout.x" => {
            let PropValue::Number(nx) = value else {
                return Err(SyncError::new("layout.x expects a number"));
            };
            nudge_layer_page(src, page_index, flat_index, nx - x0, 0.0)
        }
        "layout.y" => {
            let PropValue::Number(ny) = value else {
                return Err(SyncError::new("layout.y expects a number"));
            };
            nudge_layer_page(src, page_index, flat_index, 0.0, ny - y0)
        }
        "layout.w" => {
            let PropValue::Number(nw) = value else {
                return Err(SyncError::new("layout.w expects a number"));
            };
            if *nw <= 0.0 || !nw.is_finite() {
                return Err(SyncError::new("width must be positive"));
            }
            let fx = *nw / w;
            let root = parse_root(src)?;
            let targets = collect_size_targets_from_root(&root, page_index)?;
            // Flatten indices align with size targets from the same page walk.
            let target = targets
                .get(flat_index)
                .copied()
                .ok_or_else(|| SyncError::new("layer index out of range"))?;
            scale_size_target_axes(src, target, fx, 1.0)
        }
        "layout.h" => {
            let PropValue::Number(nh) = value else {
                return Err(SyncError::new("layout.h expects a number"));
            };
            if *nh <= 0.0 || !nh.is_finite() {
                return Err(SyncError::new("height must be positive"));
            }
            let fy = *nh / h;
            let root = parse_root(src)?;
            let targets = collect_size_targets_from_root(&root, page_index)?;
            let target = targets
                .get(flat_index)
                .copied()
                .ok_or_else(|| SyncError::new("layer index out of range"))?;
            scale_size_target_axes(src, target, 1.0, fy)
        }
        "transform.rotation" => {
            let PropValue::Number(deg) = value else {
                return Err(SyncError::new("rotation expects a number"));
            };
            set_layer_rotation_deg(src, page_index, flat_index, *deg, (cx, cy))
        }
        "transform.opacity" => {
            let PropValue::Number(a) = value else {
                return Err(SyncError::new("opacity expects a number"));
            };
            set_layer_opacity(src, page_index, flat_index, *a)
        }
        other => set_paint_prop(src, page_index, flat_index, other, value),
    }
}

#[inline(never)]
fn collect_paint_props(paint: &SyntaxNode, kind: &str, out: &mut Vec<PropField>) {
    let items = list_atoms(paint);
    if kind == "circle" {
        push_geom_num(&items, 1, "geom.x", "x", out);
        push_geom_num(&items, 2, "geom.y", "y", out);
        push_geom_num(&items, 3, "geom.r", "r", out);
        collect_trailing_fill(&items, out);
    } else if kind == "rect" || kind == "frame" {
        push_geom_num(&items, 1, "geom.x", "x", out);
        push_geom_num(&items, 2, "geom.y", "y", out);
        push_geom_num(&items, 3, "geom.w", "w", out);
        push_geom_num(&items, 4, "geom.h", "h", out);
        collect_trailing_fill(&items, out);
        if kind == "frame" {
            collect_trailing_stroke_on_frame(&items, out);
        }
    } else if kind == "ellipse" {
        push_geom_num(&items, 1, "geom.x", "x", out);
        push_geom_num(&items, 2, "geom.y", "y", out);
        push_geom_num(&items, 3, "geom.rx", "rx", out);
        push_geom_num(&items, 4, "geom.ry", "ry", out);
        collect_trailing_fill(&items, out);
    } else if kind == "ring" {
        push_geom_num(&items, 1, "geom.x", "x", out);
        push_geom_num(&items, 2, "geom.y", "y", out);
        push_geom_num(&items, 3, "geom.r", "r", out);
        push_geom_num(&items, 4, "geom.width", "width", out);
        collect_trailing_fill(&items, out);
    } else if kind == "text" {
        push_geom_num(&items, 1, "geom.x", "x", out);
        push_geom_num(&items, 2, "geom.y", "y", out);
        push_geom_num(&items, 3, "geom.size", "size", out);
        // Boxed: (text x y size w h "…") — slot 4 is a number.
        let boxed = matches!(
            items.get(4),
            Some(Child::Token(t)) if t.kind() == SyntaxKind::Number
        );
        if boxed {
            push_geom_num(&items, 4, "geom.w", "w", out);
            push_geom_num(&items, 5, "geom.h", "h", out);
            if let Some(s) = string_at(&items, 6) {
                out.push(PropField {
                    id: "content.text".into(),
                    label: "text".into(),
                    group: PropGroup::Content,
                    value: PropValue::Text(unquote(s)),
                    slider: None,
                });
            }
        } else if let Some(s) = string_at(&items, 4) {
            out.push(PropField {
                id: "content.text".into(),
                label: "text".into(),
                group: PropGroup::Content,
                value: PropValue::Text(unquote(s)),
                slider: None,
            });
        }
        collect_trailing_fill(&items, out);
    } else if kind == "image" {
        if let Some(raw) = string_at(&items, 1) {
            out.push(PropField {
                id: "content.path".into(),
                label: "path".into(),
                group: PropGroup::Content,
                value: PropValue::Text(unquote(raw)),
                slider: None,
            });
        }
        push_geom_num(&items, 2, "geom.x", "x", out);
        push_geom_num(&items, 3, "geom.y", "y", out);
        push_geom_num(&items, 4, "geom.w", "w", out);
        push_geom_num(&items, 5, "geom.h", "h", out);
    } else if kind == "line" {
        push_geom_num(&items, 1, "geom.x1", "x1", out);
        push_geom_num(&items, 2, "geom.y1", "y1", out);
        push_geom_num(&items, 3, "geom.x2", "x2", out);
        push_geom_num(&items, 4, "geom.y2", "y2", out);
        collect_line_stroke(&items, out);
    } else if kind == "polyline" {
        collect_polyline_stroke(&items, out);
    } else if kind == "polygon" {
        collect_trailing_fill(&items, out);
    } else {
        // Unknown paint heads contribute no geometry/stroke props.
        let _ = (paint, kind, out);
    }
}

/// Test/coverage hook: exercise the unknown-kind fallthrough of paint prop collect.
#[doc(hidden)]
pub fn coverage_collect_paint_unknown(src: &str) -> usize {
    let mut out = Vec::new();
    match parse_root(src) {
        Ok(root) => collect_paint_props(&root, "unknown-kind", &mut out),
        Err(_) => {
            let _ = src.len();
        }
    }
    out.len()
}

/// Test/coverage hook: early-return path when a geometry slot is absent.
#[doc(hidden)]
pub fn coverage_push_geom_missing() -> usize {
    let mut out = Vec::new();
    push_geom_num(&[], 0, "geom.x", "x", &mut out);
    out.len()
}

/// Test/coverage hook: polyline stroke helper with fewer than two atoms.
#[doc(hidden)]
pub fn coverage_polyline_stroke_short() -> usize {
    let mut out = Vec::new();
    collect_polyline_stroke(&[], &mut out);
    out.len()
}

/// Test/coverage hook: infallible decimal scan edge cases.
#[doc(hidden)]
pub fn coverage_parse_f64(sample: &str) -> f64 {
    parse_f64_or_zero(sample)
}

fn set_paint_prop(
    src: &str,
    page_index: usize,
    flat_index: usize,
    id: &str,
    value: &PropValue,
) -> Result<String, SyncError> {
    let (_root, paint, kind) = layer_paint(src, page_index, flat_index)?;
    let items = list_atoms(&paint);
    let kind = kind.as_str();

    match id {
        "geom.x" | "geom.y" | "geom.r" | "geom.w" | "geom.h" | "geom.rx" | "geom.ry"
        | "geom.width" | "geom.size" | "geom.x1" | "geom.y1" | "geom.x2" | "geom.y2" => {
            let PropValue::Number(n) = value else {
                return Err(SyncError::new(format!("{id} expects a number")));
            };
            let slot = geom_slot(kind, id)
                .ok_or_else(|| SyncError::new(format!("no slot for {id} on `{kind}`")))?;
            set_atom_number(src, &items, slot, *n)
        }
        "content.text" => {
            let PropValue::Text(t) = value else {
                return Err(SyncError::new("content.text expects text"));
            };
            if kind != "text" {
                return Err(SyncError::new("text content only on text shapes"));
            }
            let slot = items
                .iter()
                .position(|c| matches!(c, Child::Token(tok) if tok.kind() == SyntaxKind::String))
                .ok_or_else(|| SyncError::new("text content string missing"))?;
            set_atom_string(src, &items, slot, t)
        }
        "content.path" => {
            let PropValue::Text(t) = value else {
                return Err(SyncError::new("content.path expects text"));
            };
            if kind != "image" {
                return Err(SyncError::new("path only on image shapes"));
            }
            set_atom_string(src, &items, 1, t)
        }
        "fill.r" | "fill.g" | "fill.b" => {
            let PropValue::Number(n) = value else {
                return Err(SyncError::new(format!("{id} expects a number")));
            };
            // Outer or-pattern already constrains the suffix to r|g|b.
            let ch = match id.as_bytes()[id.len() - 1] {
                b'r' => 0,
                b'g' => 1,
                _ => 2,
            };
            set_color_channel(src, kind, &items, ColorRole::Fill, ch, *n)
        }
        "stroke.r" | "stroke.g" | "stroke.b" => {
            let PropValue::Number(n) = value else {
                return Err(SyncError::new(format!("{id} expects a number")));
            };
            let ch = match id.as_bytes()[id.len() - 1] {
                b'r' => 0,
                b'g' => 1,
                _ => 2,
            };
            set_color_channel(src, kind, &items, ColorRole::Stroke, ch, *n)
        }
        "stroke.width" => {
            let PropValue::Number(n) = value else {
                return Err(SyncError::new("stroke.width expects a number"));
            };
            set_stroke_width(src, kind, &items, *n)
        }
        other => Err(SyncError::new(format!("unknown property `{other}`"))),
    }
}

/// Set fill RGB on one layer (edits existing color or inserts `(rgb …)`).
pub fn set_layer_fill_rgb(
    src: &str,
    page_index: usize,
    flat_index: usize,
    r: f64,
    g: f64,
    b: f64,
) -> Result<String, SyncError> {
    for c in [r, g, b] {
        if !(0.0..=1.0).contains(&c) || !c.is_finite() {
            return Err(SyncError::new("fill rgb channels must be in 0..=1"));
        }
    }
    let (_root, paint, _kind) = layer_paint(src, page_index, flat_index)?;
    let items = list_atoms(&paint);
    if let Some(child) = items.iter().rev().find(|c| color_channels(c).is_some()) {
        let repl = format!(
            "(rgb {} {} {})",
            format_drag_number(r),
            format_drag_number(g),
            format_drag_number(b)
        );
        return Ok(replace_color_child(src, child, &repl));
    }
    // Insert (rgb …) before the closing paren of the paint form.
    let range = paint.text_range();
    let end = usize::from(range.end());
    // Paint lists from CST always end with `)`.
    let insert_at = end - 1;
    let rgb = format!(
        " (rgb {} {} {})",
        format_drag_number(r),
        format_drag_number(g),
        format_drag_number(b)
    );
    let mut out = String::with_capacity(src.len() + rgb.len());
    out.push_str(&src[..insert_at]);
    out.push_str(&rgb);
    out.push_str(&src[insert_at..]);
    Ok(out)
}

/// Set the same fill RGB on every listed flatten index (batch / marquee ops).
pub fn set_layers_fill_rgb(
    src: &str,
    page_index: usize,
    indices: &[usize],
    r: f64,
    g: f64,
    b: f64,
) -> Result<String, SyncError> {
    let mut out = src.to_string();
    // Apply high indices first so earlier byte ranges stay stable.
    let mut sorted = indices.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    for &i in sorted.iter().rev() {
        out = set_layer_fill_rgb(&out, page_index, i, r, g, b)?;
    }
    Ok(out)
}

/// Set the same opacity on every listed flatten index.
pub fn set_layers_opacity(
    src: &str,
    page_index: usize,
    indices: &[usize],
    alpha: f64,
) -> Result<String, SyncError> {
    let mut out = src.to_string();
    let mut sorted = indices.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    for &i in sorted.iter().rev() {
        out = set_layer_opacity(&out, page_index, i, alpha)?;
    }
    Ok(out)
}

/// Set stroke RGB on one layer (line / frame / polyline that already has a stroke color).
pub fn set_layer_stroke_rgb(
    src: &str,
    page_index: usize,
    flat_index: usize,
    r: f64,
    g: f64,
    b: f64,
) -> Result<String, SyncError> {
    for c in [r, g, b] {
        if !(0.0..=1.0).contains(&c) || !c.is_finite() {
            return Err(SyncError::new("stroke rgb channels must be in 0..=1"));
        }
    }
    let (_root, paint, kind) = layer_paint(src, page_index, flat_index)?;
    let items = list_atoms(&paint);
    let Some(child) = find_stroke_color_child(&kind, &items) else {
        return Err(SyncError::new("no stroke color on this shape"));
    };
    let repl = format!(
        "(rgb {} {} {})",
        format_drag_number(r),
        format_drag_number(g),
        format_drag_number(b)
    );
    Ok(replace_color_child(src, child, &repl))
}

fn replace_color_child(src: &str, child: &Child, repl: &str) -> String {
    match child {
        Child::Token(t) => replace_token_text(t, repl).1,
        Child::Node(n) => {
            let range = n.text_range();
            let start = usize::from(range.start());
            let end = usize::from(range.end());
            let mut out = String::with_capacity(src.len() + repl.len());
            out.push_str(&src[..start]);
            out.push_str(repl);
            out.push_str(&src[end..]);
            out
        }
    }
}

/// Batch stroke RGB. Skips layers without a stroke color; errors if none apply.
pub fn set_layers_stroke_rgb(
    src: &str,
    page_index: usize,
    indices: &[usize],
    r: f64,
    g: f64,
    b: f64,
) -> Result<String, SyncError> {
    let mut out = src.to_string();
    let mut sorted = indices.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    let mut any = false;
    let mut last_err = None;
    for &i in sorted.iter().rev() {
        match set_layer_stroke_rgb(&out, page_index, i, r, g, b) {
            Ok(next) => {
                out = next;
                any = true;
            }
            Err(e) => last_err = Some(e),
        }
    }
    if any {
        Ok(out)
    } else {
        Err(last_err.unwrap_or_else(|| SyncError::new("no stroked layers in selection")))
    }
}

/// Batch stroke width. Skips layers without width; errors if none apply.
pub fn set_layers_stroke_width(
    src: &str,
    page_index: usize,
    indices: &[usize],
    width: f64,
) -> Result<String, SyncError> {
    let ctx = PropEditContext {
        aabb_mm: (0.0, 0.0, 1.0, 1.0),
        paper_w_mm: 210.0,
        paper_h_mm: 297.0,
    };
    let mut out = src.to_string();
    let mut sorted = indices.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    let mut any = false;
    let mut last_err = None;
    for &i in sorted.iter().rev() {
        match set_layer_prop(
            &out,
            page_index,
            i,
            "stroke.width",
            &PropValue::Number(width),
            &ctx,
        ) {
            Ok(next) => {
                out = next;
                any = true;
            }
            Err(e) => last_err = Some(e),
        }
    }
    if any {
        Ok(out)
    } else {
        Err(last_err.unwrap_or_else(|| SyncError::new("no stroke width on selection")))
    }
}

fn geom_slot(kind: &str, id: &str) -> Option<usize> {
    match (kind, id) {
        ("circle" | "ring" | "ellipse" | "rect" | "frame" | "text", "geom.x") => Some(1),
        ("circle" | "ring" | "ellipse" | "rect" | "frame" | "text", "geom.y") => Some(2),
        ("circle" | "ring", "geom.r") => Some(3),
        ("ellipse", "geom.rx") => Some(3),
        ("ellipse", "geom.ry") => Some(4),
        ("ring", "geom.width") => Some(4),
        ("rect" | "frame", "geom.w") => Some(3),
        ("rect" | "frame", "geom.h") => Some(4),
        ("text", "geom.size") => Some(3),
        ("text", "geom.w") => Some(4),
        ("text", "geom.h") => Some(5),
        ("image", "geom.x") => Some(2),
        ("image", "geom.y") => Some(3),
        ("image", "geom.w") => Some(4),
        ("image", "geom.h") => Some(5),
        ("line", "geom.x1") => Some(1),
        ("line", "geom.y1") => Some(2),
        ("line", "geom.x2") => Some(3),
        ("line", "geom.y2") => Some(4),
        _ => None,
    }
}

#[derive(Clone, Copy)]
enum ColorRole {
    Fill,
    Stroke,
}

fn collect_trailing_fill(items: &[Child], out: &mut Vec<PropField>) {
    if let Some(color) = trailing_color(items) {
        push_rgb_fields(color, PropGroup::Fill, "fill", out);
    }
}

fn collect_trailing_stroke_on_frame(items: &[Child], out: &mut Vec<PropField>) {
    // (frame x y w h [fill] [stroke] [width]) — best-effort: last rgb after fill.
    let colors: Vec<_> = items.iter().filter_map(color_channels).collect();
    if let Some(stroke) = colors.get(1).copied() {
        push_rgb_fields(stroke, PropGroup::Stroke, "stroke", out);
    }
    if let Some(w) = items.last().and_then(|c| match c {
        Child::Token(t) if t.kind() == SyntaxKind::Number => t.text().parse().ok(),
        _ => None,
    }) {
        out.push(num(
            "stroke.width",
            "width",
            PropGroup::Stroke,
            w,
            Some((0.1, 40.0)),
        ));
    }
}

fn collect_line_stroke(items: &[Child], out: &mut Vec<PropField>) {
    // (line x1 y1 x2 y2 [color [width]])
    if let Some(ch) = items.get(5).and_then(color_channels) {
        push_rgb_fields(ch, PropGroup::Stroke, "stroke", out);
    }
    if let Some(t) = number_token(items, 6) {
        // Lexer `Number` tokens always parse; keep a numeric fallback for safety.
        let w = parse_f64_or_zero(t.text());
        out.push(num(
            "stroke.width",
            "width",
            PropGroup::Stroke,
            w,
            Some((0.1, 40.0)),
        ));
    }
}

#[inline(never)]
fn collect_polyline_stroke(items: &[Child], out: &mut Vec<PropField>) {
    // Prefer trailing `(color width)`; else trailing color only.
    if items.len() >= 2 {
        let last = &items[items.len() - 1];
        let prev = &items[items.len() - 2];
        if let (Child::Token(wtok), Some(ch)) = (last, color_channels(prev)) {
            if wtok.kind() == SyntaxKind::Number {
                push_rgb_fields(ch, PropGroup::Stroke, "stroke", out);
                out.push(num(
                    "stroke.width",
                    "width",
                    PropGroup::Stroke,
                    parse_f64_or_zero(wtok.text()),
                    Some((0.1, 40.0)),
                ));
                return;
            }
            // Trailing non-number token after a color: ignore width peel.
            let _ = wtok;
        }
    }
    if let Some(ch) = items.last().and_then(color_channels) {
        push_rgb_fields(ch, PropGroup::Stroke, "stroke", out);
    }
}

#[inline(never)]
fn parse_f64_or_zero(text: &str) -> f64 {
    text.parse::<f64>().unwrap_or(0.0)
}

fn trailing_color(items: &[Child]) -> Option<[f64; 3]> {
    items.last().and_then(color_channels)
}

fn color_channels(child: &Child) -> Option<[f64; 3]> {
    match child {
        Child::Token(t) if t.kind() == SyntaxKind::Ident => match t.text() {
            "black" => Some([0.0, 0.0, 0.0]),
            "white" => Some([1.0, 1.0, 1.0]),
            "red" => Some([1.0, 0.0, 0.0]),
            "green" => Some([0.0, 1.0, 0.0]),
            "blue" => Some([0.0, 0.0, 1.0]),
            _ => None,
        },
        Child::Node(n) => {
            let atoms = list_atoms(n);
            if !matches!(
                atoms.first(),
                Some(Child::Token(h)) if h.kind() == SyntaxKind::Ident && h.text() == "rgb"
            ) {
                return None;
            }
            let (Some(rt), Some(gt), Some(bt)) = (
                number_token(&atoms, 1),
                number_token(&atoms, 2),
                number_token(&atoms, 3),
            ) else {
                return None;
            };
            let r = parse_f64_or_zero(rt.text());
            let g = parse_f64_or_zero(gt.text());
            let b = parse_f64_or_zero(bt.text());
            Some([r, g, b])
        }
        _ => None,
    }
}

fn push_rgb_fields(rgb: [f64; 3], group: PropGroup, prefix: &str, out: &mut Vec<PropField>) {
    let labels = ["r", "g", "b"];
    for (i, lab) in labels.iter().enumerate() {
        out.push(num(
            &format!("{prefix}.{lab}"),
            &format!("{prefix}.{lab}"),
            group,
            rgb[i],
            Some((0.0, 1.0)),
        ));
    }
}

fn set_color_channel(
    _src: &str,
    kind: &str,
    items: &[Child],
    role: ColorRole,
    channel: usize,
    value: f64,
) -> Result<String, SyncError> {
    if !(0.0..=1.0).contains(&value) || !value.is_finite() {
        return Err(SyncError::new("color channel must be in 0..=1"));
    }
    let child = match role {
        ColorRole::Fill => {
            // set_paint_prop already required a head token, so items is non-empty.
            &items[items.len() - 1]
        }
        ColorRole::Stroke => find_stroke_color_child(kind, items)
            .ok_or_else(|| SyncError::new("no stroke color on this shape"))?,
    };
    match child {
        Child::Node(n) => {
            let atoms = list_atoms(n);
            if !matches!(
                atoms.first(),
                Some(Child::Token(h)) if h.kind() == SyntaxKind::Ident && h.text() == "rgb"
            ) {
                return Err(SyncError::new("expected (rgb …) color form"));
            }
            let tok = number_token(&atoms, 1 + channel)
                .ok_or_else(|| SyncError::new("rgb channel token missing"))?;
            let (_, out) = replace_token_text(&tok, &format_drag_number(value));
            Ok(out)
        }
        Child::Token(t) if t.kind() == SyntaxKind::Ident => {
            // Replace named color with an (rgb …) form, patching one channel.
            let mut rgb = color_channels(child).unwrap_or([0.0, 0.0, 0.0]);
            rgb[channel] = value;
            let repl = format!(
                "(rgb {} {} {})",
                format_drag_number(rgb[0]),
                format_drag_number(rgb[1]),
                format_drag_number(rgb[2])
            );
            let (_, out) = replace_token_text(t, &repl);
            Ok(out)
        }
        _ => Err(SyncError::new("unsupported color atom")),
    }
}

fn find_stroke_color_child<'a>(kind: &str, items: &'a [Child]) -> Option<&'a Child> {
    match kind {
        "line" => items.get(5).filter(|c| color_channels(c).is_some()),
        "frame" => {
            let colors: Vec<_> = items
                .iter()
                .filter(|c| color_channels(c).is_some())
                .collect();
            colors.get(1).copied()
        }
        "polyline" => {
            // Vertices, then optional color, optional width.
            let mut i = 1;
            while i + 1 < items.len()
                && matches!(items[i], Child::Token(ref t) if t.kind() == SyntaxKind::Number)
                && matches!(items[i + 1], Child::Token(ref t) if t.kind() == SyntaxKind::Number)
            {
                i += 2;
            }
            items.get(i).filter(|c| color_channels(c).is_some())
        }
        _ => None,
    }
}

fn set_stroke_width(
    src: &str,
    kind: &str,
    items: &[Child],
    value: f64,
) -> Result<String, SyncError> {
    if value <= 0.0 || !value.is_finite() {
        return Err(SyncError::new("stroke width must be positive"));
    }
    let slot = match kind {
        "line" if items.len() >= 7 => Some(6),
        "frame" | "polyline" => {
            if matches!(items.last(), Some(Child::Token(t)) if t.kind() == SyntaxKind::Number) {
                Some(items.len() - 1)
            } else {
                None
            }
        }
        _ => None,
    };
    let slot = slot.ok_or_else(|| SyncError::new("no stroke width on this shape"))?;
    set_atom_number(src, items, slot, value)
}

#[inline(never)]
fn push_geom_num(items: &[Child], slot: usize, id: &str, label: &str, out: &mut Vec<PropField>) {
    let Some(tok) = number_token(items, slot) else {
        return;
    };
    let v = parse_f64_or_zero(tok.text());
    let slider = match label {
        "r" | "rx" | "ry" | "w" | "h" | "size" | "width" => Some((0.5, 400.0)),
        _ => Some((-400.0, 400.0)),
    };
    out.push(num(id, label, PropGroup::Geometry, v, slider));
}

fn set_atom_number(
    src: &str,
    items: &[Child],
    slot: usize,
    value: f64,
) -> Result<String, SyncError> {
    let _ = src;
    let tok = number_token(items, slot)
        .ok_or_else(|| SyncError::new(format!("missing number at slot {slot}")))?;
    let (_, out) = replace_token_text(&tok, &format_drag_number(value));
    Ok(out)
}

fn set_atom_string(
    src: &str,
    items: &[Child],
    slot: usize,
    value: &str,
) -> Result<String, SyncError> {
    let _ = src;
    let tok = match items.get(slot) {
        Some(Child::Token(t)) if t.kind() == SyntaxKind::String => t.clone(),
        _ => return Err(SyncError::new(format!("missing string at slot {slot}"))),
    };
    let escaped = escape_rpx_string(value);
    let (_, out) = replace_token_text(&tok, &escaped);
    Ok(out)
}

fn escape_rpx_string(s: &str) -> String {
    // SYN-001 §8: complete literal; no backslash escapes.
    reciplexa_syntax::encode_string_literal(s)
}

fn unquote(raw: &str) -> String {
    // SYN-001 §8: decode short / multi-quote literals; `\` is a normal character.
    reciplexa_syntax::decode_string_literal(raw).unwrap_or_else(|_| {
        if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
            raw[1..raw.len() - 1].to_string()
        } else {
            raw.trim_matches('"').to_string()
        }
    })
}

fn string_at(items: &[Child], slot: usize) -> Option<&str> {
    match items.get(slot) {
        Some(Child::Token(t)) if t.kind() == SyntaxKind::String => Some(t.text()),
        _ => None,
    }
}

fn number_token(items: &[Child], slot: usize) -> Option<SyntaxToken> {
    match items.get(slot) {
        Some(Child::Token(t)) if t.kind() == SyntaxKind::Number => Some(t.clone()),
        _ => None,
    }
}

fn num(id: &str, label: &str, group: PropGroup, v: f64, slider: Option<(f64, f64)>) -> PropField {
    PropField {
        id: id.into(),
        label: label.into(),
        group,
        value: PropValue::Number(v),
        slider,
    }
}

#[cfg(test)]
mod props_coverage_helpers {
    use super::*;

    #[test]
    fn unquote_lone_trailing_backslash() {
        assert_eq!(unquote("\"abc\\"), "abc\\");
    }

    #[test]
    fn coverage_hooks_also_run_under_cfg_test_lib() {
        assert_eq!(
            coverage_collect_paint_unknown("(page a4 (circle 0 0 1))"),
            0
        );
        assert_eq!(coverage_collect_paint_unknown("("), 0);
        assert_eq!(coverage_push_geom_missing(), 0);
        assert_eq!(coverage_polyline_stroke_short(), 0);
        assert_eq!(coverage_parse_f64("+1"), 1.0);
        assert_eq!(coverage_parse_f64("x"), 0.0);
    }
}
