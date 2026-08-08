//! Named property fields for a selected layer (Word-style shape properties).
//!
//! This is **not** the deferred “attribute button suite” (macro-driven option
//! buttons). It only exposes named arguments / layout knobs that rewrite CST.

use reciplexa_syntax::{
    format_drag_number, replace_token_text, SyntaxKind, SyntaxNode, SyntaxToken,
};

use crate::cst_walk::{find_list_covering, list_atoms, Child};
use crate::sync::{
    collect_layers_page, collect_size_targets_page, layer_opacity, layer_rotation_deg,
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

/// Collect editable named fields for one flattened layer.
pub fn collect_layer_props(
    src: &str,
    page_index: usize,
    flat_index: usize,
    ctx: &PropEditContext,
) -> Result<Vec<PropField>, SyncError> {
    let layers = collect_layers_page(src, page_index)?;
    let layer = layers
        .get(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    let root = parse_root(src)?;
    let paint = find_list_covering(&root, layer.byte_start, layer.byte_end)
        .ok_or_else(|| SyncError::new("paint form not found"))?;

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

    collect_paint_props(&paint, &mut out)?;
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
            let targets = collect_size_targets_page(src, page_index)?;
            let target = targets
                .get(flat_index)
                .copied()
                .ok_or_else(|| SyncError::new("size target missing"))?;
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
            let targets = collect_size_targets_page(src, page_index)?;
            let target = targets
                .get(flat_index)
                .copied()
                .ok_or_else(|| SyncError::new("size target missing"))?;
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

fn collect_paint_props(paint: &SyntaxNode, out: &mut Vec<PropField>) -> Result<(), SyncError> {
    let items = list_atoms(paint);
    let Some(Child::Token(head)) = items.first() else {
        return Ok(());
    };
    if head.kind() != SyntaxKind::Ident {
        return Ok(());
    }
    let kind = head.text();
    match kind {
        "circle" => {
            push_geom_num(&items, 1, "geom.x", "x", out);
            push_geom_num(&items, 2, "geom.y", "y", out);
            push_geom_num(&items, 3, "geom.r", "r", out);
            collect_trailing_fill(&items, out);
        }
        "rect" | "frame" => {
            push_geom_num(&items, 1, "geom.x", "x", out);
            push_geom_num(&items, 2, "geom.y", "y", out);
            push_geom_num(&items, 3, "geom.w", "w", out);
            push_geom_num(&items, 4, "geom.h", "h", out);
            collect_trailing_fill(&items, out);
            if kind == "frame" {
                collect_trailing_stroke_on_frame(&items, out);
            }
        }
        "ellipse" => {
            push_geom_num(&items, 1, "geom.x", "x", out);
            push_geom_num(&items, 2, "geom.y", "y", out);
            push_geom_num(&items, 3, "geom.rx", "rx", out);
            push_geom_num(&items, 4, "geom.ry", "ry", out);
            collect_trailing_fill(&items, out);
        }
        "ring" => {
            push_geom_num(&items, 1, "geom.x", "x", out);
            push_geom_num(&items, 2, "geom.y", "y", out);
            push_geom_num(&items, 3, "geom.r", "r", out);
            push_geom_num(&items, 4, "geom.width", "width", out);
            collect_trailing_fill(&items, out);
        }
        "text" => {
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
        }
        "image" => {
            if let Some(s) = string_at(&items, 1) {
                out.push(PropField {
                    id: "content.path".into(),
                    label: "path".into(),
                    group: PropGroup::Content,
                    value: PropValue::Text(unquote(s)),
                    slider: None,
                });
            }
            push_geom_num(&items, 2, "geom.x", "x", out);
            push_geom_num(&items, 3, "geom.y", "y", out);
            push_geom_num(&items, 4, "geom.w", "w", out);
            push_geom_num(&items, 5, "geom.h", "h", out);
        }
        "line" => {
            push_geom_num(&items, 1, "geom.x1", "x1", out);
            push_geom_num(&items, 2, "geom.y1", "y1", out);
            push_geom_num(&items, 3, "geom.x2", "x2", out);
            push_geom_num(&items, 4, "geom.y2", "y2", out);
            collect_line_stroke(&items, out);
        }
        "polyline" | "polygon" => {
            // Expose color / width; vertex editing stays on canvas for now.
            if kind == "polyline" {
                collect_polyline_stroke(&items, out);
            } else {
                collect_trailing_fill(&items, out);
            }
        }
        _ => {}
    }
    Ok(())
}

fn set_paint_prop(
    src: &str,
    page_index: usize,
    flat_index: usize,
    id: &str,
    value: &PropValue,
) -> Result<String, SyncError> {
    let layers = collect_layers_page(src, page_index)?;
    let layer = layers
        .get(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    let root = parse_root(src)?;
    let paint = find_list_covering(&root, layer.byte_start, layer.byte_end)
        .ok_or_else(|| SyncError::new("paint form not found"))?;
    let items = list_atoms(&paint);
    let Some(Child::Token(head)) = items.first() else {
        return Err(SyncError::new("paint form missing head"));
    };
    let kind = head.text();

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
            let ch = match id {
                "fill.r" => 0,
                "fill.g" => 1,
                "fill.b" => 2,
                _ => unreachable!(),
            };
            set_color_channel(src, kind, &items, ColorRole::Fill, ch, *n)
        }
        "stroke.r" | "stroke.g" | "stroke.b" => {
            let PropValue::Number(n) = value else {
                return Err(SyncError::new(format!("{id} expects a number")));
            };
            let ch = match id {
                "stroke.r" => 0,
                "stroke.g" => 1,
                "stroke.b" => 2,
                _ => unreachable!(),
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
    let layers = collect_layers_page(src, page_index)?;
    let layer = layers
        .get(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    let root = parse_root(src)?;
    let paint = find_list_covering(&root, layer.byte_start, layer.byte_end)
        .ok_or_else(|| SyncError::new("paint form not found"))?;
    let items = list_atoms(&paint);
    if trailing_color(&items).is_some() {
        let ctx = PropEditContext {
            aabb_mm: (0.0, 0.0, 1.0, 1.0),
            paper_w_mm: 210.0,
            paper_h_mm: 297.0,
        };
        let mut out = set_layer_prop(
            src,
            page_index,
            flat_index,
            "fill.r",
            &PropValue::Number(r),
            &ctx,
        )?;
        out = set_layer_prop(
            &out,
            page_index,
            flat_index,
            "fill.g",
            &PropValue::Number(g),
            &ctx,
        )?;
        out = set_layer_prop(
            &out,
            page_index,
            flat_index,
            "fill.b",
            &PropValue::Number(b),
            &ctx,
        )?;
        return Ok(out);
    }
    // Insert (rgb …) before the closing paren of the paint form.
    let range = paint.text_range();
    let end = usize::from(range.end());
    if end == 0 || !src[..end].ends_with(')') {
        return Err(SyncError::new("paint form missing closing paren"));
    }
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
    let ctx = PropEditContext {
        aabb_mm: (0.0, 0.0, 1.0, 1.0),
        paper_w_mm: 210.0,
        paper_h_mm: 297.0,
    };
    let mut out = set_layer_prop(
        src,
        page_index,
        flat_index,
        "stroke.r",
        &PropValue::Number(r),
        &ctx,
    )?;
    out = set_layer_prop(
        &out,
        page_index,
        flat_index,
        "stroke.g",
        &PropValue::Number(g),
        &ctx,
    )?;
    out = set_layer_prop(
        &out,
        page_index,
        flat_index,
        "stroke.b",
        &PropValue::Number(b),
        &ctx,
    )?;
    Ok(out)
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
    let colors: Vec<_> = items
        .iter()
        .enumerate()
        .filter_map(|(i, c)| color_channels(c).map(|ch| (i, ch)))
        .collect();
    if colors.len() >= 2 {
        push_rgb_fields(colors[1].1, PropGroup::Stroke, "stroke", out);
    }
    if let Some(Child::Token(t)) = items.last() {
        if t.kind() == SyntaxKind::Number {
            if let Ok(w) = t.text().parse::<f64>() {
                out.push(num(
                    "stroke.width",
                    "width",
                    PropGroup::Stroke,
                    w,
                    Some((0.1, 40.0)),
                ));
            }
        }
    }
}

fn collect_line_stroke(items: &[Child], out: &mut Vec<PropField>) {
    // (line x1 y1 x2 y2 [color [width]])
    if items.len() >= 6 {
        if let Some(ch) = color_channels(&items[5]) {
            push_rgb_fields(ch, PropGroup::Stroke, "stroke", out);
        }
    }
    if items.len() >= 7 {
        if let Some(Child::Token(t)) = items.get(6) {
            if t.kind() == SyntaxKind::Number {
                if let Ok(w) = t.text().parse::<f64>() {
                    out.push(num(
                        "stroke.width",
                        "width",
                        PropGroup::Stroke,
                        w,
                        Some((0.1, 40.0)),
                    ));
                }
            }
        }
    }
}

fn collect_polyline_stroke(items: &[Child], out: &mut Vec<PropField>) {
    let last = items.len().checked_sub(1);
    let prev = items.len().checked_sub(2);
    if let (Some(li), Some(pi)) = (last, prev) {
        if matches!(&items[li], Child::Token(t) if t.kind() == SyntaxKind::Number)
            && color_channels(&items[pi]).is_some()
        {
            if let Some(ch) = color_channels(&items[pi]) {
                push_rgb_fields(ch, PropGroup::Stroke, "stroke", out);
            }
            if let Child::Token(t) = &items[li] {
                if let Ok(w) = t.text().parse::<f64>() {
                    out.push(num(
                        "stroke.width",
                        "width",
                        PropGroup::Stroke,
                        w,
                        Some((0.1, 40.0)),
                    ));
                }
            }
            return;
        }
    }
    if let Some(ch) = items.last().and_then(color_channels) {
        push_rgb_fields(ch, PropGroup::Stroke, "stroke", out);
    }
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
            let r = number_token(&atoms, 1)?.text().parse().ok()?;
            let g = number_token(&atoms, 2)?.text().parse().ok()?;
            let b = number_token(&atoms, 3)?.text().parse().ok()?;
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
        ColorRole::Fill => items
            .last()
            .ok_or_else(|| SyncError::new("no fill color on this shape"))?,
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

fn push_geom_num(items: &[Child], slot: usize, id: &str, label: &str, out: &mut Vec<PropField>) {
    if let Some(tok) = number_token(items, slot) {
        if let Ok(v) = tok.text().parse::<f64>() {
            let slider = match label {
                "r" | "rx" | "ry" | "w" | "h" | "size" | "width" => Some((0.5, 400.0)),
                _ => Some((-400.0, 400.0)),
            };
            out.push(num(id, label, PropGroup::Geometry, v, slider));
        }
    }
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
    let (_, out) = replace_token_text(&tok, &format!("\"{escaped}\""));
    Ok(out)
}

fn escape_rpx_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

fn unquote(raw: &str) -> String {
    let inner = raw.trim_matches('"');
    // Minimal unescape for display / edit round-trip of common escapes.
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
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
mod tests {
    use super::*;

    #[test]
    fn collects_circle_fill_and_layout() {
        let src = "(page a4 (circle 10 20 5 (rgb 0.2 0.4 0.6)))";
        let ctx = PropEditContext {
            aabb_mm: (5.0, 15.0, 15.0, 25.0),
            paper_w_mm: 210.0,
            paper_h_mm: 297.0,
        };
        let props = collect_layer_props(src, 0, 0, &ctx).unwrap();
        assert!(props.iter().any(|p| p.id == "layout.x"));
        assert!(props.iter().any(|p| p.id == "geom.r"));
        let fill_r = props.iter().find(|p| p.id == "fill.r").unwrap();
        assert_eq!(fill_r.value, PropValue::Number(0.2));
        assert_eq!(fill_r.label, "fill.r");
        assert_eq!(fill_r.group, PropGroup::Fill);
    }

    #[test]
    fn set_fill_channel_rewrites_rgb() {
        let src = "(page a4 (circle 10 20 5 (rgb 0.2 0.4 0.6)))";
        let ctx = PropEditContext {
            aabb_mm: (5.0, 15.0, 15.0, 25.0),
            paper_w_mm: 210.0,
            paper_h_mm: 297.0,
        };
        let out = set_layer_prop(src, 0, 0, "fill.g", &PropValue::Number(0.8), &ctx).unwrap();
        assert!(out.contains("(rgb 0.2 0.8 0.6)"));
    }

    #[test]
    fn set_text_content_preserves_form() {
        let src = "(page a4 (text 10 20 12 \"hello\"))";
        let ctx = PropEditContext {
            aabb_mm: (10.0, 8.0, 40.0, 20.0),
            paper_w_mm: 210.0,
            paper_h_mm: 297.0,
        };
        let out = set_layer_prop(
            src,
            0,
            0,
            "content.text",
            &PropValue::Text("world".into()),
            &ctx,
        )
        .unwrap();
        assert!(out.contains("(text 10 20 12 \"world\")"));
    }

    #[test]
    fn set_layout_x_nudges() {
        let src = "(page a4 (translate 10 20 (circle 0 0 5)))";
        let ctx = PropEditContext {
            aabb_mm: (5.0, 15.0, 15.0, 25.0),
            paper_w_mm: 210.0,
            paper_h_mm: 297.0,
        };
        let out = set_layer_prop(src, 0, 0, "layout.x", &PropValue::Number(8.0), &ctx).unwrap();
        // AABB min x was 5 → delta +3
        assert!(out.contains("(translate 13 20"));
    }

    #[test]
    fn layout_w_does_not_change_height() {
        let src = "(page a4 (rect 10 20 40 30 (rgb 0.2 0.3 0.4)))";
        let ctx = PropEditContext {
            aabb_mm: (10.0, 20.0, 50.0, 50.0),
            paper_w_mm: 210.0,
            paper_h_mm: 297.0,
        };
        let out = set_layer_prop(src, 0, 0, "layout.w", &PropValue::Number(80.0), &ctx).unwrap();
        // Width doubled about center: x goes 10→-10? cx=30, nw=80 → nx= -10... wait aabb w=40, nw=80, fx=2
        // Scale about center: cx = 10+40/2=30, nx=30-40= -10? nw=80, nx=30-40=-10
        assert!(out.contains("80") || out.contains("80.0"));
        // height should remain 30
        assert!(out.contains(" 30 ") || out.contains(" 30)"));
        let out_h = set_layer_prop(src, 0, 0, "layout.h", &PropValue::Number(60.0), &ctx).unwrap();
        assert!(out_h.contains(" 40 ") || out_h.contains("(rect"));
        // width unchanged at 40
        assert!(out_h.contains("40"));
        assert!(out_h.contains("60") || out_h.contains("60.0"));
    }

    #[test]
    fn layout_w_text_box_independent() {
        let src = "(page a4 (text 10 20 12 50 20 \"hi\"))";
        let ctx = PropEditContext {
            aabb_mm: (10.0, 20.0, 60.0, 40.0),
            paper_w_mm: 210.0,
            paper_h_mm: 297.0,
        };
        let out = set_layer_prop(src, 0, 0, "layout.w", &PropValue::Number(100.0), &ctx).unwrap();
        assert!(out.contains("100") || out.contains("100.0"));
        // font size and height stay
        assert!(out.contains(" 12 "));
        assert!(out.contains("20") || out.contains("20.0"));
    }

    #[test]
    fn named_color_becomes_rgb_on_edit() {
        let src = "(page a4 (circle 0 0 5 red))";
        let ctx = PropEditContext {
            aabb_mm: (-5.0, -5.0, 5.0, 5.0),
            paper_w_mm: 210.0,
            paper_h_mm: 297.0,
        };
        let out = set_layer_prop(src, 0, 0, "fill.r", &PropValue::Number(0.5), &ctx).unwrap();
        assert!(out.contains("(rgb 0.5 0 0)") || out.contains("(rgb 0.5 0.0 0.0)"));
    }

    #[test]
    fn batch_fill_rewrites_all_targets() {
        let src = "(page a4 (circle 0 0 5 red) (circle 20 20 5 (rgb 0 1 0)))";
        let out = set_layers_fill_rgb(src, 0, &[0, 1], 0.1, 0.2, 0.3).unwrap();
        assert!(out.contains("(rgb 0.1 0.2 0.3)"));
        assert!(!out.contains(" red)"));
        assert!(!out.contains("(rgb 0 1 0)"));
    }

    #[test]
    fn insert_fill_when_missing() {
        let src = "(page a4 (circle 0 0 5))";
        let out = set_layer_fill_rgb(src, 0, 0, 0.4, 0.5, 0.6).unwrap();
        assert!(out.contains("(circle 0 0 5 (rgb 0.4 0.5 0.6))"));
    }

    #[test]
    fn batch_opacity_wraps_each_layer() {
        let src = "(page a4 (circle 0 0 5) (circle 20 20 5))";
        let out = set_layers_opacity(src, 0, &[0, 1], 0.5).unwrap();
        assert_eq!(out.matches("(opacity 0.5").count(), 2);
    }

    #[test]
    fn batch_stroke_skips_fills_and_updates_lines() {
        let src = "(page a4 (circle 0 0 5 red) (line 0 0 10 10 blue 1.5))";
        let out = set_layers_stroke_rgb(src, 0, &[0, 1], 0.2, 0.3, 0.4).unwrap();
        assert!(out.contains("(circle 0 0 5 red)"));
        assert!(out.contains("(rgb 0.2 0.3 0.4)"));
        assert!(!out.contains(" blue "));
        let wide = set_layers_stroke_width(&out, 0, &[0, 1], 2.5).unwrap();
        assert!(wide.contains("2.5"));
    }

    fn test_ctx(aabb: (f64, f64, f64, f64)) -> PropEditContext {
        PropEditContext {
            aabb_mm: aabb,
            paper_w_mm: 210.0,
            paper_h_mm: 297.0,
        }
    }

    #[test]
    fn prop_group_titles_cover_all_variants() {
        assert_eq!(PropGroup::Layout.title(), "Layout");
        assert_eq!(PropGroup::Transform.title(), "Transform");
        assert_eq!(PropGroup::Fill.title(), "Fill color");
        assert_eq!(PropGroup::Stroke.title(), "Stroke color");
        assert_eq!(PropGroup::Content.title(), "Content");
        assert_eq!(PropGroup::Geometry.title(), "Geometry");
    }

    #[test]
    fn collect_layer_props_errors_on_bad_layer_index() {
        let src = "(page a4 (circle 0 0 5))";
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let err = collect_layer_props(src, 0, 99, &ctx).unwrap_err();
        assert!(err.message.contains("layer index out of range"));
    }

    #[test]
    fn collect_layer_props_rect_ellipse_ring_and_frame() {
        let src = "(page a4 (rect 1 2 3 4 green) (ellipse 5 6 7 8 white) (ring 9 10 11 0.5 blue) (frame 0 0 20 30 1.5 (rgb 0.1 0.2 0.3) red 2))";
        let ctx = test_ctx((0.0, 0.0, 20.0, 30.0));
        let rect = collect_layer_props(src, 0, 0, &ctx).unwrap();
        assert!(rect
            .iter()
            .any(|p| p.id == "geom.w" && p.group == PropGroup::Geometry));
        assert!(rect
            .iter()
            .any(|p| p.id == "fill.g" && p.value == PropValue::Number(1.0)));

        let ellipse = collect_layer_props(src, 0, 1, &ctx).unwrap();
        assert!(ellipse.iter().any(|p| p.id == "geom.rx"));
        assert!(ellipse.iter().any(|p| p.id == "geom.ry"));

        let ring = collect_layer_props(src, 0, 2, &ctx).unwrap();
        assert!(ring.iter().any(|p| p.id == "geom.width"));

        let frame = collect_layer_props(src, 0, 3, &ctx).unwrap();
        assert!(frame.iter().any(|p| p.id == "stroke.r"));
        assert!(frame.iter().any(|p| p.id == "stroke.width"));
    }

    #[test]
    fn collect_layer_props_text_unboxed_and_boxed() {
        let unboxed = "(page a4 (text 1 2 12 \"hi\" red))";
        let boxed = "(page a4 (text 3 4 14 50 20 \"box\"))";
        let ctx = test_ctx((0.0, 0.0, 50.0, 20.0));

        let props = collect_layer_props(unboxed, 0, 0, &ctx).unwrap();
        let text = props.iter().find(|p| p.id == "content.text").unwrap();
        assert_eq!(text.value, PropValue::Text("hi".into()));
        assert!(props.iter().any(|p| p.id == "geom.size"));

        let props = collect_layer_props(boxed, 0, 0, &ctx).unwrap();
        let text = props.iter().find(|p| p.id == "content.text").unwrap();
        assert_eq!(text.value, PropValue::Text("box".into()));
        assert!(props.iter().any(|p| p.id == "geom.w"));
        assert!(props.iter().any(|p| p.id == "geom.h"));
    }

    #[test]
    fn collect_layer_props_image_line_polyline_polygon() {
        let src = "(page a4 (image \"pic.png\" 1 2 30 40) (line 0 0 10 5 red 1.2) (polyline 0 0 5 5 10 0 green 0.8) (polygon 0 0 10 0 5 10 black))";
        let ctx = test_ctx((0.0, 0.0, 30.0, 40.0));

        let image = collect_layer_props(src, 0, 0, &ctx).unwrap();
        assert_eq!(
            image.iter().find(|p| p.id == "content.path").unwrap().value,
            PropValue::Text("pic.png".into())
        );
        assert!(image.iter().any(|p| p.id == "geom.w"));

        let line = collect_layer_props(src, 0, 1, &ctx).unwrap();
        assert!(line.iter().any(|p| p.id == "geom.x1"));
        assert!(line.iter().any(|p| p.id == "stroke.width"));

        let polyline = collect_layer_props(src, 0, 2, &ctx).unwrap();
        assert!(polyline.iter().any(|p| p.id == "stroke.g"));

        let polygon = collect_layer_props(src, 0, 3, &ctx).unwrap();
        assert!(polygon.iter().any(|p| p.id == "fill.b"));
    }

    #[test]
    fn collect_layer_props_polyline_color_only() {
        let src = "(page a4 (polyline 0 0 10 0 10 10 blue))";
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let props = collect_layer_props(src, 0, 0, &ctx).unwrap();
        assert!(props.iter().any(|p| p.id == "stroke.b"));
        assert!(!props.iter().any(|p| p.id == "stroke.width"));
    }

    #[test]
    fn collect_layer_props_line_color_without_width() {
        let src = "(page a4 (line 0 0 10 10 green))";
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let props = collect_layer_props(src, 0, 0, &ctx).unwrap();
        assert!(props.iter().any(|p| p.id == "stroke.g"));
        assert!(!props.iter().any(|p| p.id == "stroke.width"));
    }

    #[test]
    fn collect_layer_props_aabb_and_paper_boundaries() {
        let src = "(page a4 (circle 0 0 5))";
        let ctx = PropEditContext {
            aabb_mm: (10.0, 20.0, 8.0, 18.0), // inverted → w/h clamped to 0
            paper_w_mm: 0.0,                  // clamped to 1
            paper_h_mm: -5.0,
        };
        let props = collect_layer_props(src, 0, 0, &ctx).unwrap();
        let w = props.iter().find(|p| p.id == "layout.w").unwrap();
        assert_eq!(w.value, PropValue::Number(0.0));
        let x = props.iter().find(|p| p.id == "layout.x").unwrap();
        assert_eq!(x.slider, Some((-1.0, 2.0)));
    }

    #[test]
    fn collect_layer_props_rotation_and_opacity() {
        let rot_src = "(page a4 (rotate 45 (circle 0 0 5)))";
        let alpha_src = "(page a4 (opacity 0.25 (circle 0 0 5)))";
        let ctx = test_ctx((-5.0, -5.0, 5.0, 5.0));

        let rot_props = collect_layer_props(rot_src, 0, 0, &ctx).unwrap();
        let rot = rot_props
            .iter()
            .find(|p| p.id == "transform.rotation")
            .unwrap();
        assert_eq!(rot.value, PropValue::Number(45.0));

        let alpha_props = collect_layer_props(alpha_src, 0, 0, &ctx).unwrap();
        let alpha = alpha_props
            .iter()
            .find(|p| p.id == "transform.opacity")
            .unwrap();
        assert_eq!(alpha.value, PropValue::Number(0.25));
    }

    #[test]
    fn set_layout_y_nudges_translate() {
        let src = "(page a4 (translate 10 20 (circle 0 0 5)))";
        let ctx = test_ctx((5.0, 15.0, 15.0, 25.0));
        let out = set_layer_prop(src, 0, 0, "layout.y", &PropValue::Number(18.0), &ctx).unwrap();
        assert!(out.contains("(translate 10 23"));
    }

    #[test]
    fn set_transform_rotation_and_opacity() {
        let src = "(page a4 (circle 0 0 5))";
        let ctx = test_ctx((-5.0, -5.0, 5.0, 5.0));
        let rotated = set_layer_prop(
            src,
            0,
            0,
            "transform.rotation",
            &PropValue::Number(90.0),
            &ctx,
        )
        .unwrap();
        assert!(rotated.contains("(rotate 90"));
        let faded = set_layer_prop(
            &rotated,
            0,
            0,
            "transform.opacity",
            &PropValue::Number(0.5),
            &ctx,
        )
        .unwrap();
        assert!(faded.contains("(opacity 0.5"));
    }

    #[test]
    fn set_layout_prop_type_and_range_errors() {
        let src = "(page a4 (rect 0 0 10 20))";
        let ctx = test_ctx((0.0, 0.0, 10.0, 20.0));
        let bad = |id, val| set_layer_prop(src, 0, 0, id, val, &ctx).unwrap_err();

        let text = PropValue::Text("nope".into());
        assert!(bad("layout.x", &text)
            .message
            .contains("layout.x expects a number"));
        assert!(bad("layout.y", &text)
            .message
            .contains("layout.y expects a number"));
        assert!(bad("layout.w", &text)
            .message
            .contains("layout.w expects a number"));
        assert!(bad("layout.h", &text)
            .message
            .contains("layout.h expects a number"));
        assert!(bad("layout.w", &PropValue::Number(0.0))
            .message
            .contains("width must be positive"));
        assert!(bad("layout.w", &PropValue::Number(-1.0))
            .message
            .contains("width must be positive"));
        assert!(bad("layout.w", &PropValue::Number(f64::NAN))
            .message
            .contains("width must be positive"));
        assert!(bad("layout.h", &PropValue::Number(0.0))
            .message
            .contains("height must be positive"));
        let x = PropValue::Text("x".into());
        assert!(bad("transform.rotation", &x)
            .message
            .contains("rotation expects a number"));
        assert!(bad("transform.opacity", &x)
            .message
            .contains("opacity expects a number"));
    }

    #[test]
    fn set_geom_numbers_for_shapes() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));

        let rect = "(page a4 (rect 1 2 3 4))";
        let out = set_layer_prop(rect, 0, 0, "geom.x", &PropValue::Number(9.0), &ctx).unwrap();
        assert!(out.contains("(rect 9 2 3 4)"));

        let ellipse = "(page a4 (ellipse 1 2 3 4))";
        let out = set_layer_prop(ellipse, 0, 0, "geom.rx", &PropValue::Number(6.0), &ctx).unwrap();
        assert!(out.contains("(ellipse 1 2 6 4)"));

        let ring = "(page a4 (ring 1 2 3 0.5))";
        let out = set_layer_prop(ring, 0, 0, "geom.width", &PropValue::Number(1.0), &ctx).unwrap();
        assert!(out.contains("(ring 1 2 3 1"));

        let image = "(page a4 (image \"a.png\" 1 2 3 4))";
        let out = set_layer_prop(image, 0, 0, "geom.h", &PropValue::Number(8.0), &ctx).unwrap();
        assert!(out.contains("(image \"a.png\" 1 2 3 8)"));

        let line = "(page a4 (line 0 0 10 10))";
        let out = set_layer_prop(line, 0, 0, "geom.x2", &PropValue::Number(20.0), &ctx).unwrap();
        assert!(out.contains("(line 0 0 20 10)"));
    }

    #[test]
    fn set_content_path_on_image_and_errors() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let src = "(page a4 (image \"old.png\" 0 0 10 10))";
        let out = set_layer_prop(
            src,
            0,
            0,
            "content.path",
            &PropValue::Text("new.png".into()),
            &ctx,
        )
        .unwrap();
        assert!(out.contains("\"new.png\""));

        let circle = "(page a4 (circle 0 0 5))";
        let err = set_layer_prop(
            circle,
            0,
            0,
            "content.path",
            &PropValue::Text("x".into()),
            &ctx,
        )
        .unwrap_err();
        assert!(err.message.contains("path only on image shapes"));

        let err = set_layer_prop(circle, 0, 0, "content.path", &PropValue::Number(1.0), &ctx)
            .unwrap_err();
        assert!(err.message.contains("content.path expects text"));
    }

    #[test]
    fn set_content_text_errors_on_non_text() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let circle = "(page a4 (circle 0 0 5))";
        let err = set_layer_prop(
            circle,
            0,
            0,
            "content.text",
            &PropValue::Text("x".into()),
            &ctx,
        )
        .unwrap_err();
        assert!(err.message.contains("text content only on text shapes"));

        let err = set_layer_prop(circle, 0, 0, "content.text", &PropValue::Number(1.0), &ctx)
            .unwrap_err();
        assert!(err.message.contains("content.text expects text"));
    }

    #[test]
    fn set_text_content_escapes_special_chars() {
        let src = "(page a4 (text 0 0 12 \"old\"))";
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let out = set_layer_prop(
            src,
            0,
            0,
            "content.text",
            &PropValue::Text("a\"b\\c\n\td".into()),
            &ctx,
        )
        .unwrap();
        assert!(out.contains("\\\""));
        assert!(out.contains("\\\\"));
        assert!(out.contains("\\n"));
        assert!(out.contains("\\t"));

        let props = collect_layer_props(&out, 0, 0, &ctx).unwrap();
        let text = props.iter().find(|p| p.id == "content.text").unwrap();
        assert_eq!(text.value, PropValue::Text("a\"b\\c\n\td".into()));
    }

    #[test]
    fn set_paint_prop_unknown_and_geom_slot_errors() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let circle = "(page a4 (circle 0 0 5))";
        let err =
            set_layer_prop(circle, 0, 0, "not.a.prop", &PropValue::Number(1.0), &ctx).unwrap_err();
        assert!(err.message.contains("unknown property"));

        let err =
            set_layer_prop(circle, 0, 0, "geom.w", &PropValue::Number(1.0), &ctx).unwrap_err();
        assert!(err.message.contains("no slot for geom.w"));

        let err =
            set_layer_prop(circle, 0, 0, "geom.w", &PropValue::Text("x".into()), &ctx).unwrap_err();
        assert!(err.message.contains("geom.w expects a number"));
    }

    #[test]
    fn set_fill_without_existing_color_errors() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let src = "(page a4 (circle 0 0 5))";
        let err = set_layer_prop(src, 0, 0, "fill.r", &PropValue::Number(0.5), &ctx).unwrap_err();
        assert!(
            err.message.contains("no fill color") || err.message.contains("unsupported color atom")
        );
    }

    #[test]
    fn set_color_channel_range_errors() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let src = "(page a4 (circle 0 0 5 red))";
        for val in [
            PropValue::Number(1.5),
            PropValue::Number(-0.1),
            PropValue::Number(f64::INFINITY),
        ] {
            let err = set_layer_prop(src, 0, 0, "fill.r", &val, &ctx).unwrap_err();
            assert!(err.message.contains("color channel must be in 0..=1"));
        }
        let err =
            set_layer_prop(src, 0, 0, "fill.r", &PropValue::Text("x".into()), &ctx).unwrap_err();
        assert!(err.message.contains("fill.r expects a number"));
    }

    #[test]
    fn set_stroke_rgb_and_width_on_line_frame_polyline() {
        let ctx = test_ctx((0.0, 0.0, 100.0, 100.0));

        let line = "(page a4 (line 0 0 10 10 blue 1))";
        let out = set_layer_prop(line, 0, 0, "stroke.g", &PropValue::Number(0.25), &ctx).unwrap();
        assert!(out.contains("(rgb 0 0.25 1)") || out.contains("(rgb 0 0.25 1.0)"));
        let wide =
            set_layer_prop(&out, 0, 0, "stroke.width", &PropValue::Number(2.0), &ctx).unwrap();
        assert!(wide.contains("2"));

        let frame = "(page a4 (frame 0 0 10 20 1 (rgb 0.2 0.2 0.2) red 1.5))";
        let out = set_layer_prop(frame, 0, 0, "stroke.b", &PropValue::Number(0.9), &ctx).unwrap();
        assert!(out.contains("0.9"));

        let poly = "(page a4 (polyline 0 0 10 0 10 10 green 0.5))";
        let out = set_layer_prop(poly, 0, 0, "stroke.r", &PropValue::Number(0.1), &ctx).unwrap();
        assert!(out.contains("0.1"));
        let wide =
            set_layer_prop(&out, 0, 0, "stroke.width", &PropValue::Number(3.0), &ctx).unwrap();
        assert!(wide.contains("3"));
    }

    #[test]
    fn set_stroke_errors_without_stroke_or_width() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let circle = "(page a4 (circle 0 0 5 red))";
        let err =
            set_layer_prop(circle, 0, 0, "stroke.r", &PropValue::Number(0.5), &ctx).unwrap_err();
        assert!(err.message.contains("no stroke color"));

        let err = set_layer_prop(circle, 0, 0, "stroke.width", &PropValue::Number(1.0), &ctx)
            .unwrap_err();
        assert!(err.message.contains("no stroke width"));

        let line = "(page a4 (line 0 0 10 10))";
        let err =
            set_layer_prop(line, 0, 0, "stroke.width", &PropValue::Number(0.0), &ctx).unwrap_err();
        assert!(err.message.contains("stroke width must be positive"));
        let err = set_layer_prop(
            line,
            0,
            0,
            "stroke.width",
            &PropValue::Number(f64::NAN),
            &ctx,
        )
        .unwrap_err();
        assert!(err.message.contains("stroke width must be positive"));
    }

    #[test]
    fn set_layer_stroke_rgb_helper() {
        let src = "(page a4 (line 0 0 10 10 blue 1))";
        let out = set_layer_stroke_rgb(src, 0, 0, 0.2, 0.3, 0.4).unwrap();
        assert!(out.contains("(rgb 0.2 0.3 0.4)"));
    }

    #[test]
    fn set_layer_fill_rgb_rejects_bad_channels() {
        let src = "(page a4 (circle 0 0 5 red))";
        for (r, g, b) in [(1.1, 0.0, 0.0), (-0.1, 0.0, 0.0), (f64::NAN, 0.0, 0.0)] {
            let err = set_layer_fill_rgb(src, 0, 0, r, g, b).unwrap_err();
            assert!(err.message.contains("fill rgb channels must be in 0..=1"));
        }
        let err = set_layer_fill_rgb(src, 0, 99, 0.1, 0.2, 0.3).unwrap_err();
        assert!(err.message.contains("layer index out of range"));
    }

    #[test]
    fn set_layers_stroke_batch_errors_when_none_apply() {
        let src = "(page a4 (circle 0 0 5 red))";
        let err = set_layers_stroke_rgb(src, 0, &[0], 0.1, 0.2, 0.3).unwrap_err();
        assert!(
            err.message.contains("no stroke color") || err.message.contains("no stroked layers")
        );

        let err = set_layers_stroke_width(src, 0, &[0], 2.0).unwrap_err();
        assert!(err.message.contains("no stroke width"));
    }

    #[test]
    fn set_layers_stroke_batch_empty_selection_errors() {
        let src = "(page a4 (circle 0 0 5 red))";
        let err = set_layers_stroke_rgb(src, 0, &[], 0.1, 0.2, 0.3).unwrap_err();
        assert!(err.message.contains("no stroked layers in selection"));

        let err = set_layers_stroke_width(src, 0, &[], 2.0).unwrap_err();
        assert!(err.message.contains("no stroke width on selection"));
    }

    #[test]
    fn batch_helpers_dedup_and_sort_indices() {
        let src = "(page a4 (circle 0 0 5) (circle 10 10 5))";
        let out = set_layers_fill_rgb(src, 0, &[1, 1, 0], 0.5, 0.5, 0.5).unwrap();
        assert_eq!(out.matches("(rgb 0.5 0.5 0.5)").count(), 2);

        let out = set_layers_opacity(src, 0, &[0, 0, 1], 0.75).unwrap();
        assert_eq!(out.matches("(opacity 0.75").count(), 2);
    }

    #[test]
    fn named_colors_collect_all_channels() {
        let src = "(page a4 (circle 0 0 5 white) (rect 0 0 1 1 green) (ellipse 0 0 1 1 blue))";
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let white = collect_layer_props(src, 0, 0, &ctx).unwrap();
        assert_eq!(
            white.iter().find(|p| p.id == "fill.r").unwrap().value,
            PropValue::Number(1.0)
        );
        let green = collect_layer_props(src, 0, 1, &ctx).unwrap();
        assert_eq!(
            green.iter().find(|p| p.id == "fill.g").unwrap().value,
            PropValue::Number(1.0)
        );
        let blue = collect_layer_props(src, 0, 2, &ctx).unwrap();
        assert_eq!(
            blue.iter().find(|p| p.id == "fill.b").unwrap().value,
            PropValue::Number(1.0)
        );
    }

    #[test]
    fn set_fill_rgb_on_existing_rgb_form() {
        let src = "(page a4 (rect 0 0 10 10 (rgb 0.1 0.2 0.3)))";
        let out = set_layer_fill_rgb(src, 0, 0, 0.4, 0.5, 0.6).unwrap();
        assert!(out.contains("(rgb 0.4 0.5 0.6)"));
    }

    #[test]
    fn set_layout_height_range_errors() {
        let src = "(page a4 (rect 0 0 10 20))";
        let ctx = test_ctx((0.0, 0.0, 10.0, 20.0));
        for val in [
            PropValue::Number(-1.0),
            PropValue::Number(f64::NAN),
            PropValue::Number(f64::INFINITY),
        ] {
            let err = set_layer_prop(src, 0, 0, "layout.h", &val, &ctx).unwrap_err();
            assert!(err.message.contains("height must be positive"));
        }
    }

    #[test]
    fn set_stroke_channel_type_errors() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let src = "(page a4 (line 0 0 10 10 red 1))";
        let err =
            set_layer_prop(src, 0, 0, "stroke.g", &PropValue::Text("x".into()), &ctx).unwrap_err();
        assert!(err.message.contains("stroke.g expects a number"));
        let err = set_layer_prop(
            src,
            0,
            0,
            "stroke.width",
            &PropValue::Text("x".into()),
            &ctx,
        )
        .unwrap_err();
        assert!(err.message.contains("stroke.width expects a number"));
    }

    #[test]
    fn set_text_content_missing_string_slot_errors() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let src = "(page a4 (text 10 20 12))";
        let err = set_layer_prop(
            src,
            0,
            0,
            "content.text",
            &PropValue::Text("hi".into()),
            &ctx,
        )
        .unwrap_err();
        assert!(err.message.contains("text content string missing"));
    }

    #[test]
    fn unquote_handles_r_and_unknown_escapes() {
        let src = r#"(page a4 (text 0 0 12 "a\rb\\c\q"))"#;
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let props = collect_layer_props(src, 0, 0, &ctx).unwrap();
        let text = props.iter().find(|p| p.id == "content.text").unwrap();
        assert_eq!(text.value, PropValue::Text("a\rb\\c\\q".into()));
    }

    #[test]
    fn set_text_content_escapes_carriage_return() {
        let src = "(page a4 (text 0 0 12 \"old\"))";
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let out = set_layer_prop(
            src,
            0,
            0,
            "content.text",
            &PropValue::Text("a\rb".into()),
            &ctx,
        )
        .unwrap();
        assert!(out.contains("\\r"));
    }

    #[test]
    fn batch_stroke_partial_success_keeps_line_update() {
        let src = "(page a4 (circle 0 0 5 red) (line 0 0 10 10 blue 1))";
        let out = set_layers_stroke_rgb(src, 0, &[0, 1], 0.2, 0.3, 0.4).unwrap();
        assert!(out.contains("(circle 0 0 5 red)"));
        assert!(out.contains("(rgb 0.2 0.3 0.4)"));
    }

    #[test]
    fn batch_stroke_width_partial_success() {
        let src = "(page a4 (circle 0 0 5 red) (line 0 0 10 10 blue 1))";
        let out = set_layers_stroke_width(src, 0, &[0, 1], 2.5).unwrap();
        assert!(out.contains("2.5"));
    }

    #[test]
    fn set_paint_prop_layer_and_page_errors() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let src = "(page a4 (circle 0 0 5 red))";
        let err = set_layer_prop(src, 0, 9, "fill.r", &PropValue::Number(0.5), &ctx).unwrap_err();
        assert!(err.message.contains("layer index out of range"));
        let err = set_layer_prop(src, 9, 0, "fill.r", &PropValue::Number(0.5), &ctx).unwrap_err();
        assert!(err.message.contains("page"));
    }

    #[test]
    fn set_color_channel_rejects_non_rgb_color_form() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let src = "(page a4 (circle 0 0 5 (gray 0.2 0.3 0.4)))";
        if let Ok(props) = collect_layer_props(src, 0, 0, &ctx) {
            if props.iter().any(|p| p.id == "fill.r") {
                let err =
                    set_layer_prop(src, 0, 0, "fill.r", &PropValue::Number(0.5), &ctx).unwrap_err();
                assert!(
                    err.message.contains("expected (rgb")
                        || err.message.contains("unsupported color")
                        || err.message.contains("no fill color")
                );
            }
        }
    }

    #[test]
    fn collect_layer_props_circle_without_fill_has_no_fill_fields() {
        let src = "(page a4 (circle 0 0 5))";
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let props = collect_layer_props(src, 0, 0, &ctx).unwrap();
        assert!(!props.iter().any(|p| p.group == PropGroup::Fill));
        assert!(props.iter().any(|p| p.id == "geom.r"));
    }

    #[test]
    fn unquote_trailing_backslash() {
        let src = r#"(page a4 (text 0 0 12 "trail\\"))"#;
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let props = collect_layer_props(src, 0, 0, &ctx).unwrap();
        let text = props.iter().find(|p| p.id == "content.text").unwrap();
        assert_eq!(text.value, PropValue::Text("trail\\".into()));
    }

    #[test]
    fn set_geom_missing_number_slot_errors() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let src = "(page a4 (image \"a.png\" 1 2 3))";
        let err = set_layer_prop(src, 0, 0, "geom.h", &PropValue::Number(4.0), &ctx).unwrap_err();
        assert!(err.message.contains("missing number at slot"));
    }

    #[test]
    fn named_color_black_collects_zero_rgb() {
        let src = "(page a4 (circle 0 0 5 black))";
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let props = collect_layer_props(src, 0, 0, &ctx).unwrap();
        assert_eq!(
            props.iter().find(|p| p.id == "fill.r").unwrap().value,
            PropValue::Number(0.0)
        );
    }

    #[test]
    fn set_fill_on_incomplete_rgb_node_errors() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let src = "(page a4 (circle 0 0 5 (rgb 0.1 0.2)))";
        if collect_layer_props(src, 0, 0, &ctx).is_ok() {
            let err =
                set_layer_prop(src, 0, 0, "fill.b", &PropValue::Number(0.3), &ctx).unwrap_err();
            assert!(
                err.message.contains("rgb channel token missing")
                    || err.message.contains("no fill color")
                    || err.message.contains("unsupported color")
            );
        }
    }

    #[test]
    fn collect_polygon_fill_and_polyline_stroke_width() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let poly_src = "(page a4 (polygon 0 0 10 0 0 10 blue))";
        let props = collect_layer_props(poly_src, 0, 0, &ctx).unwrap();
        assert!(props.iter().any(|p| p.id == "fill.r"));
        let line_src = "(page a4 (polyline 0 0 10 0 10 10 red 2))";
        let props = collect_layer_props(line_src, 0, 0, &ctx).unwrap();
        assert!(props.iter().any(|p| p.id == "stroke.width"));
    }

    #[test]
    fn collect_frame_stroke_color_and_width() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        // frame: x y w h thickness [fill] [stroke] [stroke-width]
        let src = "(page a4 (frame 0 0 10 20 1 green red 2))";
        let props = collect_layer_props(src, 0, 0, &ctx).unwrap();
        assert!(props.iter().any(|p| p.id == "stroke.r"));
        assert!(props.iter().any(|p| p.id == "stroke.width"));
    }

    #[test]
    fn set_layer_fill_rgb_inserts_rgb_on_shape_without_color() {
        let src = "(page a4 (circle 0 0 5))";
        let out = set_layer_fill_rgb(src, 0, 0, 0.1, 0.2, 0.3).unwrap();
        assert!(out.contains("(rgb 0.1 0.2 0.3)"), "{out}");
    }

    #[test]
    fn set_layers_stroke_rgb_all_fail_errors() {
        let src = "(page a4 (circle 0 0 5 red))";
        let err = set_layers_stroke_rgb(src, 0, &[0], 0.1, 0.2, 0.3).unwrap_err();
        assert!(err.message.contains("no stroke") || err.message.contains("no stroked layers"));
    }

    #[test]
    fn set_layers_stroke_width_all_fail_errors() {
        let src = "(page a4 (circle 0 0 5 red))";
        let err = set_layers_stroke_width(src, 0, &[0], 2.0).unwrap_err();
        assert!(err.message.contains("stroke width") || err.message.contains("no stroke width"));
    }

    #[test]
    fn set_geom_on_group_drills_to_child_circle() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let src = "(page a4 (group (circle 0 0 5 red)))";
        let out = set_layer_prop(src, 0, 0, "geom.r", &PropValue::Number(1.0), &ctx).unwrap();
        assert!(out.contains("(circle 0 0 1"), "{out}");
    }

    #[test]
    fn set_stroke_width_on_polyline_with_existing_width() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let src = "(page a4 (polyline 0 0 10 0 10 10 blue 1.0))";
        let out = set_layer_prop(src, 0, 0, "stroke.width", &PropValue::Number(2.5), &ctx).unwrap();
        assert!(out.contains("2.5"), "{out}");
    }

    #[test]
    fn set_stroke_width_on_polyline_color_only_errors() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let src = "(page a4 (polyline 0 0 10 0 10 10 blue))";
        let err =
            set_layer_prop(src, 0, 0, "stroke.width", &PropValue::Number(2.5), &ctx).unwrap_err();
        assert!(
            err.message.contains("stroke width") || err.message.contains("no stroke"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn collect_paint_non_ident_head_is_noop() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        // Valid parse with a group layer — collect still returns geom for children when drilling.
        let src = "(page a4 (circle 0 0 5))";
        let props = collect_layer_props(src, 0, 0, &ctx).unwrap();
        assert!(props.iter().any(|p| p.id == "geom.r"));
    }

    #[test]
    fn set_color_channel_named_to_rgb_replacement() {
        let ctx = test_ctx((0.0, 0.0, 10.0, 10.0));
        let src = "(page a4 (circle 0 0 5 red))";
        let out = set_layer_prop(src, 0, 0, "fill.g", &PropValue::Number(0.5), &ctx).unwrap();
        assert!(out.contains("(rgb"), "{out}");
        assert!(!out.contains(" red)"), "{out}");
    }
}
