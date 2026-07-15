//! Map GUI nudges back onto CST numeric leaves (Glisp-style).
//!
//! Bindings are collected in the same preorder as [`reciplexa_view`] flattening
//! so hit-test indices line up with editable leaves.

use reciplexa_syntax::{
    format_drag_number, parse_source, replace_token_text, SyntaxElement, SyntaxKind, SyntaxNode,
    SyntaxToken,
};

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

/// Collect one [`DragTarget`] per flattened drawable, in flatten order.
pub fn collect_drag_targets(src: &str) -> Result<Vec<DragTarget>, SyncError> {
    collect_drag_targets_page(src, 0)
}

/// Collect drag targets for a single page (0-based), matching [`reciplexa_view::flatten_page`].
pub fn collect_drag_targets_page(
    src: &str,
    page_index: usize,
) -> Result<Vec<DragTarget>, SyncError> {
    let root = parse_root(src)?;
    let mut out = Vec::new();
    let mut counters = Counters::default();
    let page = find_page(&root, page_index)?;
    let items = list_atoms(&page);
    let start = page_body_start(&items);
    for item in items.iter().skip(start) {
        if let Child::Node(n) = item {
            collect_from_shape(n, None, &mut counters, &mut out);
        }
    }
    Ok(out)
}

/// Collect layer labels + source spans for a page (flatten / hit-test order).
pub fn collect_layers_page(src: &str, page_index: usize) -> Result<Vec<LayerInfo>, SyncError> {
    let root = parse_root(src)?;
    let mut out = Vec::new();
    let page = find_page(&root, page_index)?;
    let items = list_atoms(&page);
    let start = page_body_start(&items);
    for item in items.iter().skip(start) {
        if let Child::Node(n) = item {
            let rr = n.text_range();
            let root_span = (usize::from(rr.start()), usize::from(rr.end()));
            collect_layers_from_shape(n, root_span, &mut out);
        }
    }
    Ok(out)
}

/// Move flattened layer `from` to flatten index `to` (0 = bottom / earliest in source).
///
/// Rewrites the `.rpx` so paint order stays entirely in the code (no hidden z).
pub fn reorder_layer_page(
    src: &str,
    page_index: usize,
    from: usize,
    to: usize,
) -> Result<String, SyncError> {
    if from == to {
        return Ok(src.to_string());
    }
    let layers = collect_layers_page(src, page_index)?;
    if from >= layers.len() || to >= layers.len() {
        return Err(SyncError::new("layer index out of range"));
    }
    let from_root = (layers[from].root_start, layers[from].root_end);
    let to_root = (layers[to].root_start, layers[to].root_end);

    if from_root == to_root {
        reorder_among_shared_root(src, &layers, from, to, from_root)
    } else {
        reorder_page_roots(src, page_index, from_root, to_root)
    }
}

/// Remove flattened layer `flat_index` from the page (rewrites `.rpx`).
///
/// If the layer is the sole drawable under its page-level root, the whole root
/// form is removed. Otherwise only that leaf span is cut out of a shared group.
pub fn delete_layer_page(
    src: &str,
    page_index: usize,
    flat_index: usize,
) -> Result<String, SyncError> {
    let layers = collect_layers_page(src, page_index)?;
    let layer = layers
        .get(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    let root = (layer.root_start, layer.root_end);
    let shared = layers
        .iter()
        .filter(|l| (l.root_start, l.root_end) == root)
        .count();
    let (cut_start, cut_end) = if shared <= 1 {
        extent_with_leading_ws(src, root.0, root.1)
    } else {
        extent_with_leading_ws(src, layer.byte_start, layer.byte_end)
    };
    let mut out = String::with_capacity(src.len());
    out.push_str(&src[..cut_start]);
    out.push_str(&src[cut_end..]);
    Ok(out)
}

/// Duplicate flattened layer `flat_index` (inserts a copy after it in source order).
pub fn duplicate_layer_page(
    src: &str,
    page_index: usize,
    flat_index: usize,
) -> Result<String, SyncError> {
    let layers = collect_layers_page(src, page_index)?;
    let layer = layers
        .get(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    let root = (layer.root_start, layer.root_end);
    let shared = layers
        .iter()
        .filter(|l| (l.root_start, l.root_end) == root)
        .count();
    let (span_start, span_end) = if shared <= 1 {
        (root.0, root.1)
    } else {
        (layer.byte_start, layer.byte_end)
    };
    let snippet = &src[span_start..span_end];
    // Insert after the form, preserving a leading newline when the original had one.
    let insert_at = extent_with_leading_ws(src, span_start, span_end).1;
    let pad = if src[..span_start].ends_with('\n') || snippet.contains('\n') {
        "\n  "
    } else {
        " "
    };
    let mut out = String::with_capacity(src.len() + snippet.len() + pad.len());
    out.push_str(&src[..insert_at]);
    out.push_str(pad);
    out.push_str(snippet);
    out.push_str(&src[insert_at..]);
    Ok(out)
}

/// Append a complete shape form as a new top-level child of `(page …)`.
///
/// `form` must be a full list, e.g. `(circle 105 148.5 20)`. Returns the new
/// source and the flatten index of the inserted leaf (usually last).
pub fn insert_layer_page(
    src: &str,
    page_index: usize,
    form: &str,
) -> Result<(String, usize), SyncError> {
    let form = form.trim();
    if form.is_empty() || !form.starts_with('(') || !form.ends_with(')') {
        return Err(SyncError::new("insert form must be a complete (…) list"));
    }
    let root = parse_root(src)?;
    let page = find_page(&root, page_index)?;
    let range = page.text_range();
    let page_start = usize::from(range.start());
    let page_end = usize::from(range.end());
    if page_end == 0 || !src[..page_end].ends_with(')') {
        return Err(SyncError::new("page form missing closing paren"));
    }
    let insert_at = page_end - 1;
    let pad = if src[page_start..insert_at].contains('\n') {
        "\n  "
    } else {
        " "
    };
    let mut out = String::with_capacity(src.len() + form.len() + pad.len());
    out.push_str(&src[..insert_at]);
    out.push_str(pad);
    out.push_str(form);
    out.push_str(&src[insert_at..]);
    let layers = collect_layers_page(&out, page_index)?;
    let idx = layers
        .len()
        .checked_sub(1)
        .ok_or_else(|| SyncError::new("insert produced no layers"))?;
    Ok((out, idx))
}

fn reorder_page_roots(
    src: &str,
    page_index: usize,
    from_root: (usize, usize),
    to_root: (usize, usize),
) -> Result<String, SyncError> {
    let root = parse_root(src)?;
    let page = find_page(&root, page_index)?;
    let items = list_atoms(&page);
    let start = page_body_start(&items);
    let mut forms = Vec::new();
    for item in items.iter().skip(start) {
        if let Child::Node(n) = item {
            let r = n.text_range();
            forms.push((usize::from(r.start()), usize::from(r.end())));
        }
    }
    let fi = forms
        .iter()
        .position(|r| *r == from_root)
        .ok_or_else(|| SyncError::new("from layer root not found under page"))?;
    let ti = forms
        .iter()
        .position(|r| *r == to_root)
        .ok_or_else(|| SyncError::new("to layer root not found under page"))?;
    if fi == ti {
        return Ok(src.to_string());
    }
    let item = forms.remove(fi);
    forms.insert(ti, item);
    Ok(rewrite_form_order(src, &forms))
}

fn reorder_among_shared_root(
    src: &str,
    layers: &[LayerInfo],
    from: usize,
    to: usize,
    root: (usize, usize),
) -> Result<String, SyncError> {
    let sibling_idx: Vec<usize> = layers
        .iter()
        .enumerate()
        .filter(|(_, l)| (l.root_start, l.root_end) == root)
        .map(|(i, _)| i)
        .collect();
    let fi = sibling_idx
        .iter()
        .position(|&i| i == from)
        .ok_or_else(|| SyncError::new("from layer not under shared root"))?;
    let ti = sibling_idx
        .iter()
        .position(|&i| i == to)
        .ok_or_else(|| SyncError::new("to layer not under shared root"))?;
    if fi == ti {
        return Ok(src.to_string());
    }
    let mut forms: Vec<(usize, usize)> = sibling_idx
        .iter()
        .map(|&i| (layers[i].byte_start, layers[i].byte_end))
        .collect();
    let item = forms.remove(fi);
    forms.insert(ti, item);
    Ok(rewrite_form_order(src, &forms))
}

/// `forms` is the desired order of existing `(start,end)` list spans (without trivia).
fn rewrite_form_order(src: &str, forms: &[(usize, usize)]) -> String {
    if forms.is_empty() {
        return src.to_string();
    }
    let extents: Vec<(usize, usize)> = forms
        .iter()
        .map(|&(a, b)| extent_with_leading_ws(src, a, b))
        .collect();
    let body_start = extents.iter().map(|e| e.0).min().unwrap();
    let body_end = extents.iter().map(|e| e.1).max().unwrap();
    let mut new_body = String::new();
    for &(s, e) in &extents {
        new_body.push_str(&src[s..e]);
    }
    let mut out = String::with_capacity(src.len());
    out.push_str(&src[..body_start]);
    out.push_str(&new_body);
    out.push_str(&src[body_end..]);
    out
}

fn extent_with_leading_ws(src: &str, start: usize, end: usize) -> (usize, usize) {
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

fn find_page(root: &SyntaxNode, page_index: usize) -> Result<SyntaxNode, SyncError> {
    let mut page_i = 0usize;
    for form in root.children() {
        if !is_list_headed(&form, "page") {
            continue;
        }
        if page_i == page_index {
            return Ok(form);
        }
        page_i += 1;
    }
    Err(SyncError::new(format!("no page #{page_index}")))
}

/// Index of the first shape child under `(page …)`.
fn page_body_start(items: &[Child]) -> usize {
    if items.len() >= 3
        && matches!(&items[1], Child::Token(t) if t.kind() == SyntaxKind::Number)
        && matches!(&items[2], Child::Token(t) if t.kind() == SyntaxKind::Number)
    {
        3
    } else {
        2
    }
}

/// Nudge the numbers described by `target` by `(dx, dy)` mm.
pub fn nudge_drag_target(
    src: &str,
    target: DragTarget,
    dx: f64,
    dy: f64,
) -> Result<String, SyncError> {
    match target {
        DragTarget::Translate(i) => nudge_nth_pair(src, "translate", i, 1, 2, dx, dy),
        DragTarget::CircleXy(i) => nudge_nth_pair(src, "circle", i, 1, 2, dx, dy),
        DragTarget::RectXy(i) => nudge_nth_pair(src, "rect", i, 1, 2, dx, dy),
        DragTarget::EllipseXy(i) => nudge_nth_pair(src, "ellipse", i, 1, 2, dx, dy),
        DragTarget::RingXy(i) => nudge_nth_pair(src, "ring", i, 1, 2, dx, dy),
        DragTarget::FrameXy(i) => nudge_nth_pair(src, "frame", i, 1, 2, dx, dy),
        DragTarget::TextXy(i) => nudge_nth_pair(src, "text", i, 1, 2, dx, dy),
        DragTarget::LineXy(i) => nudge_line(src, i, dx, dy),
        DragTarget::PolylineXy(i) => nudge_polyline(src, i, dx, dy),
        DragTarget::PolygonXy(i) => nudge_polygon(src, i, dx, dy),
        DragTarget::ImageXy(i) => nudge_nth_pair(src, "image", i, 2, 3, dx, dy),
    }
}

/// Compatibility wrapper: nudge the first translate in the file.
pub fn nudge_first_translate(src: &str, dx: f64, dy: f64) -> Result<String, SyncError> {
    nudge_drag_target(src, DragTarget::Translate(0), dx, dy)
}

/// Translate flattened layer `flat_index` by `(dx, dy)` in **page / world mm**.
///
/// This never edits rotate/scale factors. If the layer root is not already a
/// `(translate …)` (or center-rotate sandwich), it wraps the root so movement
/// stays axis-aligned even when the leaf sits under a bare `(rotate …)`.
pub fn nudge_layer_page(
    src: &str,
    page_index: usize,
    flat_index: usize,
    dx: f64,
    dy: f64,
) -> Result<String, SyncError> {
    if dx == 0.0 && dy == 0.0 {
        return Ok(src.to_string());
    }
    let layers = collect_layers_page(src, page_index)?;
    let layer = layers
        .get(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    let root = parse_root(src)?;
    let node = find_list_covering(&root, layer.root_start, layer.root_end)
        .ok_or_else(|| SyncError::new("layer root not found"))?;

    if is_center_rotate_sandwich(&node) || is_headed(&node, "translate") {
        return nudge_xy_slots_of_list(&node, 1, 2, dx, dy);
    }

    if is_headed(&node, "opacity") {
        if let Some(child) = first_shape_child(&node) {
            if is_center_rotate_sandwich(&child) || is_headed(&child, "translate") {
                return nudge_xy_slots_of_list(&child, 1, 2, dx, dy);
            }
        }
    }

    wrap_span_with_translate(src, layer.root_start, layer.root_end, dx, dy)
}

fn is_headed(node: &SyntaxNode, name: &str) -> bool {
    let items = list_atoms(node);
    matches!(
        items.first(),
        Some(Child::Token(t)) if t.kind() == SyntaxKind::Ident && t.text() == name
    )
}

fn first_shape_child(node: &SyntaxNode) -> Option<SyntaxNode> {
    let items = list_atoms(node);
    let skip = match items.first() {
        Some(Child::Token(t)) if t.kind() == SyntaxKind::Ident => match t.text() {
            "opacity" | "rotate" => 2,
            "translate" => 3,
            "scale" => transform_body_skip("scale", &items),
            "group" => 1,
            _ => return None,
        },
        _ => return None,
    };
    items.into_iter().skip(skip).find_map(|c| match c {
        Child::Node(n) => Some(n),
        _ => None,
    })
}

fn nudge_xy_slots_of_list(
    node: &SyntaxNode,
    x_slot: usize,
    y_slot: usize,
    dx: f64,
    dy: f64,
) -> Result<String, SyncError> {
    let items = list_atoms(node);
    let x_tok = match items.get(x_slot) {
        Some(Child::Token(t)) if t.kind() == SyntaxKind::Number => t.clone(),
        _ => return Err(SyncError::new("translate missing numeric x")),
    };
    let y_tok = match items.get(y_slot) {
        Some(Child::Token(t)) if t.kind() == SyntaxKind::Number => t.clone(),
        _ => return Err(SyncError::new("translate missing numeric y")),
    };
    let x: f64 = x_tok
        .text()
        .parse()
        .map_err(|_| SyncError::new("bad translate x"))?;
    let y: f64 = y_tok
        .text()
        .parse()
        .map_err(|_| SyncError::new("bad translate y"))?;
    let (_, after_x) = replace_token_text(&x_tok, &format_drag_number(x + dx));
    let root2 = parse_root(&after_x)?;
    let start = usize::from(node.text_range().start());
    let end = usize::from(node.text_range().end());
    // Re-find the same list after the x patch (span may shift if digit count changes).
    let node2 = find_list_covering(&root2, start, end)
        .or_else(|| {
            // Span length may have changed; search by approximate start.
            root2.descendants().find(|n| {
                n.kind() == SyntaxKind::List && usize::from(n.text_range().start()) == start
            })
        })
        .ok_or_else(|| SyncError::new("translate form lost after x patch"))?;
    let items2 = list_atoms(&node2);
    let y_tok2 = match items2.get(y_slot) {
        Some(Child::Token(t)) if t.kind() == SyntaxKind::Number => t.clone(),
        _ => return Err(SyncError::new("translate missing numeric y after x patch")),
    };
    let (_, after_y) = replace_token_text(&y_tok2, &format_drag_number(y + dy));
    Ok(after_y)
}

fn wrap_span_with_translate(
    src: &str,
    start: usize,
    end: usize,
    tx: f64,
    ty: f64,
) -> Result<String, SyncError> {
    let inner = &src[start..end];
    let wrapped = format!(
        "(translate {} {} {})",
        format_drag_number(tx),
        format_drag_number(ty),
        inner
    );
    let mut out = String::with_capacity(src.len() + wrapped.len() - inner.len());
    out.push_str(&src[..start]);
    out.push_str(&wrapped);
    out.push_str(&src[end..]);
    Ok(out)
}

/// Collect one [`SizeTarget`] per flattened drawable (same order as hit-test).
pub fn collect_size_targets_page(
    src: &str,
    page_index: usize,
) -> Result<Vec<SizeTarget>, SyncError> {
    let root = parse_root(src)?;
    let mut out = Vec::new();
    let mut counters = Counters::default();
    let page = find_page(&root, page_index)?;
    let items = list_atoms(&page);
    let start = page_body_start(&items);
    for item in items.iter().skip(start) {
        if let Child::Node(n) = item {
            collect_size_from_shape(n, &mut counters, &mut out);
        }
    }
    Ok(out)
}

/// Multiply a leaf shape's size by `factor` (>0), rewriting CST numbers only.
pub fn scale_size_target(src: &str, target: SizeTarget, factor: f64) -> Result<String, SyncError> {
    if !(factor.is_finite() && factor > 0.0) {
        return Err(SyncError::new("scale factor must be finite and > 0"));
    }
    match target {
        SizeTarget::CircleR(i) => multiply_nth_number(src, "circle", i, 3, factor),
        SizeTarget::RingR(i) => multiply_nth_number(src, "ring", i, 3, factor),
        SizeTarget::EllipseRxRy(i) => {
            let after = multiply_nth_number(src, "ellipse", i, 3, factor)?;
            multiply_nth_number(&after, "ellipse", i, 4, factor)
        }
        SizeTarget::TextSize(i) => scale_text_box(src, i, factor, factor),
        SizeTarget::RectWh(i) => scale_box_about_center(src, "rect", i, [1, 2, 3, 4], factor),
        SizeTarget::FrameWh(i) => scale_box_about_center(src, "frame", i, [1, 2, 3, 4], factor),
        SizeTarget::ImageWh(i) => scale_box_about_center(src, "image", i, [2, 3, 4, 5], factor),
        SizeTarget::LineSeg(i) => {
            let after = scale_xy_pairs_about_centroid(src, "line", i, factor)?;
            // Optional trailing width at slot 6: (line x1 y1 x2 y2 color width)
            multiply_nth_number_optional(&after, "line", i, 6, factor)
        }
        SizeTarget::PolylinePoints(i) => {
            let after = scale_xy_pairs_about_centroid(src, "polyline", i, factor)?;
            scale_polyline_trailing_width(&after, i, factor)
        }
        SizeTarget::PolygonPoints(i) => scale_xy_pairs_about_centroid(src, "polygon", i, factor),
        SizeTarget::Unsupported => Ok(src.to_string()),
    }
}

/// Resize a `(text …)` layout box with independent width/height factors.
///
/// Font `size` is unchanged. Missing `w`/`h` slots are inserted after the size atom.
pub fn scale_text_box(src: &str, index: usize, fx: f64, fy: f64) -> Result<String, SyncError> {
    if !(fx.is_finite() && fy.is_finite() && fx > 0.0 && fy > 0.0) {
        return Err(SyncError::new("scale factors must be finite and > 0"));
    }
    let root = parse_root(src)?;
    let x = read_nth_number(&root, "text", index, 1)?;
    let y = read_nth_number(&root, "text", index, 2)?;
    let size = read_nth_number(&root, "text", index, 3)?;
    let content = read_nth_text_content(&root, index)?;
    let (w, h) = match text_box_dims(&root, index)? {
        Some(dims) => dims,
        None => reciplexa_view::text_extent_mm(&content, size),
    };
    let cx = x + w * 0.5;
    let cy = y + h * 0.5;
    let nw = (w * fx).max(0.5);
    let nh = (h * fy).max(0.5);
    let nx = cx - nw * 0.5;
    let ny = cy - nh * 0.5;
    set_text_box(src, index, nx, ny, nw, nh)
}

/// Set one endpoint of `(line x1 y1 x2 y2 …)`; `endpoint` is 0 or 1.
pub fn set_line_endpoint(
    src: &str,
    index: usize,
    endpoint: usize,
    x: f64,
    y: f64,
) -> Result<String, SyncError> {
    if endpoint > 1 {
        return Err(SyncError::new("line endpoint must be 0 or 1"));
    }
    if !x.is_finite() || !y.is_finite() {
        return Err(SyncError::new("line endpoint must be finite"));
    }
    let x_slot = 1 + endpoint * 2;
    let y_slot = x_slot + 1;
    let out = set_nth_number(src, "line", index, x_slot, x)?;
    set_nth_number(&out, "line", index, y_slot, y)
}

/// Set one vertex of `(polyline …)` / `(polygon …)` (`vertex` is 0-based).
pub fn set_poly_vertex(
    src: &str,
    head: &str,
    index: usize,
    vertex: usize,
    x: f64,
    y: f64,
) -> Result<String, SyncError> {
    if head != "polyline" && head != "polygon" {
        return Err(SyncError::new("set_poly_vertex only for polyline/polygon"));
    }
    if !x.is_finite() || !y.is_finite() {
        return Err(SyncError::new("vertex must be finite"));
    }
    let x_slot = 1 + vertex * 2;
    let y_slot = x_slot + 1;
    let out = set_nth_number(src, head, index, x_slot, x)?;
    set_nth_number(&out, head, index, y_slot, y)
}

/// Set absolute origin + size for a box-like leaf (`rect` / `frame` / `image`).
#[allow(clippy::too_many_arguments)]
pub fn set_box_xywh(
    src: &str,
    head: &str,
    index: usize,
    slots: [usize; 4],
    x: f64,
    y: f64,
    w: f64,
    h: f64,
) -> Result<String, SyncError> {
    if ![x, y, w, h].into_iter().all(|v| v.is_finite()) {
        return Err(SyncError::new("box values must be finite"));
    }
    if !(w > 0.0 && h > 0.0) {
        return Err(SyncError::new("box w/h must be > 0"));
    }
    let mut out = set_nth_number(src, head, index, slots[0], x)?;
    out = set_nth_number(&out, head, index, slots[1], y)?;
    out = set_nth_number(&out, head, index, slots[2], w)?;
    set_nth_number(&out, head, index, slots[3], h)
}

/// Set absolute `(text …)` origin and layout box (`w`/`h`), inserting slots if needed.
pub fn set_text_box(
    src: &str,
    index: usize,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
) -> Result<String, SyncError> {
    if ![x, y, w, h].into_iter().all(|v| v.is_finite()) {
        return Err(SyncError::new("text box values must be finite"));
    }
    if !(w > 0.0 && h > 0.0) {
        return Err(SyncError::new("text box w/h must be > 0"));
    }
    let root = parse_root(src)?;
    let has_box = text_box_dims(&root, index)?.is_some();
    let mut out = set_nth_number(src, "text", index, 1, x)?;
    out = set_nth_number(&out, "text", index, 2, y)?;
    if has_box {
        out = set_nth_number(&out, "text", index, 4, w)?;
        set_nth_number(&out, "text", index, 5, h)
    } else {
        insert_text_box_slots(&out, index, w, h)
    }
}

fn text_box_dims(root: &SyntaxNode, index: usize) -> Result<Option<(f64, f64)>, SyncError> {
    let items = nth_text_items(root, index)?;
    // Boxed form: (text x y size w h "…" …)
    if items.len() < 7 {
        return Ok(None);
    }
    match (items.get(4), items.get(5), items.get(6)) {
        (Some(Child::Token(w)), Some(Child::Token(h)), Some(Child::Token(s)))
            if w.kind() == SyntaxKind::Number
                && h.kind() == SyntaxKind::Number
                && s.kind() == SyntaxKind::String =>
        {
            let ww: f64 = w
                .text()
                .parse()
                .map_err(|_| SyncError::new("bad text width"))?;
            let hh: f64 = h
                .text()
                .parse()
                .map_err(|_| SyncError::new("bad text height"))?;
            Ok(Some((ww, hh)))
        }
        _ => Ok(None),
    }
}

fn nth_text_items(root: &SyntaxNode, index: usize) -> Result<Vec<Child>, SyncError> {
    let mut seen = 0usize;
    for node in root.descendants() {
        if node.kind() != SyntaxKind::List {
            continue;
        }
        let items = list_atoms(&node);
        let Some(Child::Token(h)) = items.first() else {
            continue;
        };
        if h.kind() != SyntaxKind::Ident || h.text() != "text" {
            continue;
        }
        if seen == index {
            return Ok(items);
        }
        seen += 1;
    }
    Err(SyncError::new(format!("no `text` #{index}")))
}

fn insert_text_box_slots(src: &str, index: usize, w: f64, h: f64) -> Result<String, SyncError> {
    let root = parse_root(src)?;
    let size_tok = find_nth_number(&root, "text", index, 3)
        .ok_or_else(|| SyncError::new(format!("no `text` #{index} size")))?;
    let insert_at: usize = size_tok.text_range().end().into();
    let snippet = format!(" {} {}", format_drag_number(w), format_drag_number(h));
    let mut out = String::with_capacity(src.len() + snippet.len());
    out.push_str(&src[..insert_at]);
    out.push_str(&snippet);
    out.push_str(&src[insert_at..]);
    Ok(out)
}

/// Degrees on a center-pivoted `(rotate …)` for this flatten index, or 0.
pub fn layer_rotation_deg(
    src: &str,
    page_index: usize,
    flat_index: usize,
) -> Result<f64, SyncError> {
    let layers = collect_layers_page(src, page_index)?;
    let layer = layers
        .get(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    let root = parse_root(src)?;
    let node = find_list_covering(&root, layer.root_start, layer.root_end)
        .ok_or_else(|| SyncError::new("layer root not found"))?;
    match find_rotate_degrees_token(&node) {
        Some(tok) => tok
            .text()
            .parse()
            .map_err(|_| SyncError::new("bad rotate degrees")),
        None => Ok(0.0),
    }
}

/// Set absolute rotation about `center_mm` (object center in page mm).
///
/// Rewrites the layer root to
/// `(translate cx cy (rotate deg (translate -cx -cy …)))` so PDF/`cm` rotation
/// (which is around the origin) pivots on the object instead of the page origin.
pub fn set_layer_rotation_deg(
    src: &str,
    page_index: usize,
    flat_index: usize,
    degrees: f64,
    center_mm: (f64, f64),
) -> Result<String, SyncError> {
    if !degrees.is_finite() {
        return Err(SyncError::new("rotation degrees must be finite"));
    }
    if !center_mm.0.is_finite() || !center_mm.1.is_finite() {
        return Err(SyncError::new("rotation center must be finite"));
    }
    let layers = collect_layers_page(src, page_index)?;
    let layer = layers
        .get(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    let root = parse_root(src)?;
    let node = find_list_covering(&root, layer.root_start, layer.root_end)
        .ok_or_else(|| SyncError::new("layer root not found"))?;

    // Already a center sandwich: patch degrees only (pivot stays put).
    if is_center_rotate_sandwich(&node) {
        if let Some(tok) = find_rotate_degrees_token(&node) {
            let (_, out) = replace_token_text(&tok, &format_drag_number(degrees));
            return Ok(out);
        }
    }

    // Peel bare `(rotate …)` so we don't nest origin-pivoted rotates inside a
    // center sandwich. Also peel `(translate (rotate …))` from world-move wraps.
    let (start, end, inner) = {
        let mut peel = node.clone();
        if is_headed(&peel, "translate") {
            if let Some(child) = first_shape_child(&peel) {
                if is_bare_rotate(&child) {
                    peel = child;
                }
            }
        }
        if is_bare_rotate(&peel) {
            let items = list_atoms(&peel);
            let body = items.iter().skip(2).find_map(|c| match c {
                Child::Node(n) => Some(n),
                _ => None,
            });
            match body {
                Some(n) => {
                    let r = n.text_range();
                    (
                        layer.root_start,
                        layer.root_end,
                        src[usize::from(r.start())..usize::from(r.end())].to_string(),
                    )
                }
                None => {
                    return Err(SyncError::new("rotate form missing body"));
                }
            }
        } else {
            (
                layer.root_start,
                layer.root_end,
                src[layer.root_start..layer.root_end].to_string(),
            )
        }
    };

    let cx = format_drag_number(center_mm.0);
    let cy = format_drag_number(center_mm.1);
    let ncx = format_drag_number(-center_mm.0);
    let ncy = format_drag_number(-center_mm.1);
    let deg = format_drag_number(degrees);
    let wrapped = format!("(translate {cx} {cy} (rotate {deg} (translate {ncx} {ncy} {inner})))");
    let mut out = String::with_capacity(src.len() + wrapped.len());
    out.push_str(&src[..start]);
    out.push_str(&wrapped);
    out.push_str(&src[end..]);
    Ok(out)
}

fn is_bare_rotate(node: &SyntaxNode) -> bool {
    let items = list_atoms(node);
    matches!(
        items.first(),
        Some(Child::Token(t)) if t.kind() == SyntaxKind::Ident && t.text() == "rotate"
    )
}

/// `(translate cx cy (rotate deg (translate …)))`
fn is_center_rotate_sandwich(node: &SyntaxNode) -> bool {
    let items = list_atoms(node);
    if !matches!(
        items.first(),
        Some(Child::Token(t)) if t.kind() == SyntaxKind::Ident && t.text() == "translate"
    ) {
        return false;
    }
    let Some(Child::Node(rot)) = items.get(3) else {
        return false;
    };
    let ritems = list_atoms(rot);
    if !matches!(
        ritems.first(),
        Some(Child::Token(t)) if t.kind() == SyntaxKind::Ident && t.text() == "rotate"
    ) {
        return false;
    }
    matches!(ritems.get(2), Some(Child::Node(inner)) if {
        let iitems = list_atoms(inner);
        matches!(
            iitems.first(),
            Some(Child::Token(t)) if t.kind() == SyntaxKind::Ident && t.text() == "translate"
        )
    })
}

fn find_rotate_degrees_token(node: &SyntaxNode) -> Option<SyntaxToken> {
    if is_center_rotate_sandwich(node) {
        let items = list_atoms(node);
        let Child::Node(rot) = items.get(3)? else {
            return None;
        };
        let ritems = list_atoms(rot);
        match ritems.get(1)? {
            Child::Token(t) if t.kind() == SyntaxKind::Number => Some(t.clone()),
            _ => None,
        }
    } else if is_bare_rotate(node) {
        let items = list_atoms(node);
        match items.get(1)? {
            Child::Token(t) if t.kind() == SyntaxKind::Number => Some(t.clone()),
            _ => None,
        }
    } else if is_headed(node, "translate") {
        // Move may wrap `(translate … (rotate …))` — still report that angle.
        let child = first_shape_child(node)?;
        find_rotate_degrees_token(&child)
    } else {
        None
    }
}

fn find_list_covering(root: &SyntaxNode, start: usize, end: usize) -> Option<SyntaxNode> {
    root.descendants().find(|n| {
        if n.kind() != SyntaxKind::List {
            return false;
        }
        let r = n.text_range();
        usize::from(r.start()) == start && usize::from(r.end()) == end
    })
}

/// Opacity on the layer root `(opacity α …)`, or 1.0 if absent.
pub fn layer_opacity(src: &str, page_index: usize, flat_index: usize) -> Result<f64, SyncError> {
    let layers = collect_layers_page(src, page_index)?;
    let layer = layers
        .get(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    let root = parse_root(src)?;
    let node = find_list_covering(&root, layer.root_start, layer.root_end)
        .ok_or_else(|| SyncError::new("layer root not found"))?;
    let items = list_atoms(&node);
    if !matches!(
        items.first(),
        Some(Child::Token(t)) if t.kind() == SyntaxKind::Ident && t.text() == "opacity"
    ) {
        return Ok(1.0);
    }
    match items.get(1) {
        Some(Child::Token(t)) if t.kind() == SyntaxKind::Number => {
            let a: f64 = t
                .text()
                .parse()
                .map_err(|_| SyncError::new("bad opacity number"))?;
            Ok(a.clamp(0.0, 1.0))
        }
        _ => Err(SyncError::new("opacity form missing alpha")),
    }
}

/// Set absolute opacity on the layer root (wraps with `(opacity …)` if needed).
pub fn set_layer_opacity(
    src: &str,
    page_index: usize,
    flat_index: usize,
    alpha: f64,
) -> Result<String, SyncError> {
    if !alpha.is_finite() {
        return Err(SyncError::new("opacity must be finite"));
    }
    let alpha = alpha.clamp(0.0, 1.0);
    let layers = collect_layers_page(src, page_index)?;
    let layer = layers
        .get(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    let root = parse_root(src)?;
    let node = find_list_covering(&root, layer.root_start, layer.root_end)
        .ok_or_else(|| SyncError::new("layer root not found"))?;
    let items = list_atoms(&node);
    if matches!(
        items.first(),
        Some(Child::Token(t)) if t.kind() == SyntaxKind::Ident && t.text() == "opacity"
    ) {
        let tok = match items.get(1) {
            Some(Child::Token(t)) if t.kind() == SyntaxKind::Number => t.clone(),
            _ => return Err(SyncError::new("opacity form missing alpha")),
        };
        let (_, out) = replace_token_text(&tok, &format_drag_number(alpha));
        return Ok(out);
    }
    let start = layer.root_start;
    let end = layer.root_end;
    let inner = &src[start..end];
    let wrapped = format!("(opacity {} {})", format_drag_number(alpha), inner);
    let mut out = String::with_capacity(src.len() + wrapped.len() - inner.len());
    out.push_str(&src[..start]);
    out.push_str(&wrapped);
    out.push_str(&src[end..]);
    Ok(out)
}

#[derive(Default)]
struct Counters {
    translate: usize,
    circle: usize,
    rect: usize,
    ellipse: usize,
    ring: usize,
    frame: usize,
    text: usize,
    line: usize,
    polyline: usize,
    polygon: usize,
    image: usize,
}

fn collect_from_shape(
    node: &SyntaxNode,
    inherited_translate: Option<usize>,
    counters: &mut Counters,
    out: &mut Vec<DragTarget>,
) {
    let items = list_atoms(node);
    let Some(Child::Token(head)) = items.first() else {
        return;
    };
    if head.kind() != SyntaxKind::Ident {
        return;
    }
    match head.text() {
        "translate" => {
            let idx = counters.translate;
            counters.translate += 1;
            // Prefer the outermost translate so center-pivot sandwiches
            // `(translate c (rotate (translate -c …)))` still move as one unit.
            let bind = inherited_translate.or(Some(idx));
            for item in items.iter().skip(3) {
                if let Child::Node(n) = item {
                    collect_from_shape(n, bind, counters, out);
                }
            }
        }
        "rotate" | "scale" | "group" | "opacity" => {
            let skip = transform_body_skip(head.text(), &items);
            for item in items.iter().skip(skip) {
                if let Child::Node(n) = item {
                    collect_from_shape(n, inherited_translate, counters, out);
                }
            }
        }
        "circle" => {
            let idx = counters.circle;
            counters.circle += 1;
            out.push(match inherited_translate {
                Some(t) => DragTarget::Translate(t),
                None => DragTarget::CircleXy(idx),
            });
        }
        "rect" => {
            let idx = counters.rect;
            counters.rect += 1;
            out.push(match inherited_translate {
                Some(t) => DragTarget::Translate(t),
                None => DragTarget::RectXy(idx),
            });
        }
        "ellipse" => {
            let idx = counters.ellipse;
            counters.ellipse += 1;
            out.push(match inherited_translate {
                Some(t) => DragTarget::Translate(t),
                None => DragTarget::EllipseXy(idx),
            });
        }
        "ring" => {
            let idx = counters.ring;
            counters.ring += 1;
            out.push(match inherited_translate {
                Some(t) => DragTarget::Translate(t),
                None => DragTarget::RingXy(idx),
            });
        }
        "frame" => {
            let idx = counters.frame;
            counters.frame += 1;
            out.push(match inherited_translate {
                Some(t) => DragTarget::Translate(t),
                None => DragTarget::FrameXy(idx),
            });
        }
        "text" => {
            let idx = counters.text;
            counters.text += 1;
            out.push(match inherited_translate {
                Some(t) => DragTarget::Translate(t),
                None => DragTarget::TextXy(idx),
            });
        }
        "line" => {
            let idx = counters.line;
            counters.line += 1;
            out.push(match inherited_translate {
                Some(t) => DragTarget::Translate(t),
                None => DragTarget::LineXy(idx),
            });
        }
        "polyline" => {
            let idx = counters.polyline;
            counters.polyline += 1;
            out.push(match inherited_translate {
                Some(t) => DragTarget::Translate(t),
                None => DragTarget::PolylineXy(idx),
            });
        }
        "polygon" => {
            let idx = counters.polygon;
            counters.polygon += 1;
            out.push(match inherited_translate {
                Some(t) => DragTarget::Translate(t),
                None => DragTarget::PolygonXy(idx),
            });
        }
        "image" => {
            let idx = counters.image;
            counters.image += 1;
            out.push(match inherited_translate {
                Some(t) => DragTarget::Translate(t),
                None => DragTarget::ImageXy(idx),
            });
        }
        _ => {}
    }
}

fn collect_size_from_shape(node: &SyntaxNode, counters: &mut Counters, out: &mut Vec<SizeTarget>) {
    let items = list_atoms(node);
    let Some(Child::Token(head)) = items.first() else {
        return;
    };
    if head.kind() != SyntaxKind::Ident {
        return;
    }
    match head.text() {
        "translate" => {
            counters.translate += 1;
            for item in items.iter().skip(3) {
                if let Child::Node(n) = item {
                    collect_size_from_shape(n, counters, out);
                }
            }
        }
        "rotate" | "scale" | "group" | "opacity" => {
            let skip = transform_body_skip(head.text(), &items);
            for item in items.iter().skip(skip) {
                if let Child::Node(n) = item {
                    collect_size_from_shape(n, counters, out);
                }
            }
        }
        "circle" => {
            let idx = counters.circle;
            counters.circle += 1;
            out.push(SizeTarget::CircleR(idx));
        }
        "rect" => {
            let idx = counters.rect;
            counters.rect += 1;
            out.push(SizeTarget::RectWh(idx));
        }
        "ellipse" => {
            let idx = counters.ellipse;
            counters.ellipse += 1;
            out.push(SizeTarget::EllipseRxRy(idx));
        }
        "ring" => {
            let idx = counters.ring;
            counters.ring += 1;
            out.push(SizeTarget::RingR(idx));
        }
        "frame" => {
            let idx = counters.frame;
            counters.frame += 1;
            out.push(SizeTarget::FrameWh(idx));
        }
        "text" => {
            let idx = counters.text;
            counters.text += 1;
            out.push(SizeTarget::TextSize(idx));
        }
        "line" => {
            let idx = counters.line;
            counters.line += 1;
            out.push(SizeTarget::LineSeg(idx));
        }
        "polyline" => {
            let idx = counters.polyline;
            counters.polyline += 1;
            out.push(SizeTarget::PolylinePoints(idx));
        }
        "polygon" => {
            let idx = counters.polygon;
            counters.polygon += 1;
            out.push(SizeTarget::PolygonPoints(idx));
        }
        "image" => {
            let idx = counters.image;
            counters.image += 1;
            out.push(SizeTarget::ImageWh(idx));
        }
        _ => {}
    }
}

fn transform_body_skip(head: &str, items: &[Child]) -> usize {
    if head == "scale" {
        if items.len() >= 4
            && matches!(&items[1], Child::Token(t) if t.kind() == SyntaxKind::Number)
            && matches!(&items[2], Child::Token(t) if t.kind() == SyntaxKind::Number)
        {
            3
        } else {
            2
        }
    } else if head == "group" {
        1
    } else {
        // rotate / opacity: (head num shape…)
        2
    }
}

fn collect_layers_from_shape(
    node: &SyntaxNode,
    root_span: (usize, usize),
    out: &mut Vec<LayerInfo>,
) {
    let items = list_atoms(node);
    let Some(Child::Token(head)) = items.first() else {
        return;
    };
    if head.kind() != SyntaxKind::Ident {
        return;
    }
    let kind = head.text();
    match kind {
        "translate" => {
            for item in items.iter().skip(3) {
                if let Child::Node(n) = item {
                    collect_layers_from_shape(n, root_span, out);
                }
            }
        }
        "rotate" | "scale" | "group" | "opacity" => {
            let skip = if kind == "scale" {
                if items.len() >= 4
                    && matches!(&items[1], Child::Token(t) if t.kind() == SyntaxKind::Number)
                    && matches!(&items[2], Child::Token(t) if t.kind() == SyntaxKind::Number)
                {
                    3
                } else {
                    2
                }
            } else if kind == "group" {
                1
            } else {
                2
            };
            for item in items.iter().skip(skip) {
                if let Child::Node(n) = item {
                    collect_layers_from_shape(n, root_span, out);
                }
            }
        }
        "circle" | "rect" | "ellipse" | "ring" | "frame" | "text" | "line" | "polyline"
        | "polygon" | "image" => {
            let range = node.text_range();
            out.push(LayerInfo {
                kind: kind.to_string(),
                label: layer_label(kind, &items),
                byte_start: range.start().into(),
                byte_end: range.end().into(),
                root_start: root_span.0,
                root_end: root_span.1,
            });
        }
        _ => {}
    }
}

fn layer_label(kind: &str, items: &[Child]) -> String {
    match kind {
        "text" => {
            let snippet = items.iter().find_map(|c| match c {
                Child::Token(t) if t.kind() == SyntaxKind::String => {
                    let raw = t.text();
                    let inner = raw.trim_matches('"');
                    let short: String = inner.chars().take(24).collect();
                    Some(format!("text \"{short}\""))
                }
                _ => None,
            });
            snippet.unwrap_or_else(|| "text".into())
        }
        "image" => {
            let snippet = items.iter().find_map(|c| match c {
                Child::Token(t) if t.kind() == SyntaxKind::String => {
                    let raw = t.text().trim_matches('"');
                    let name = raw.rsplit(['/', '\\']).next().unwrap_or(raw);
                    Some(format!("image {name}"))
                }
                _ => None,
            });
            snippet.unwrap_or_else(|| "image".into())
        }
        other => other.to_string(),
    }
}

fn nudge_nth_pair(
    src: &str,
    head: &str,
    index: usize,
    x_slot: usize,
    y_slot: usize,
    dx: f64,
    dy: f64,
) -> Result<String, SyncError> {
    let root = parse_root(src)?;
    let (x_tok, y_tok) = find_nth_number_pair(&root, head, index, x_slot, y_slot)
        .ok_or_else(|| SyncError::new(format!("no `{head}` #{index} with numeric x/y")))?;

    let x: f64 = x_tok
        .text()
        .parse()
        .map_err(|_| SyncError::new("bad x number"))?;
    let y: f64 = y_tok
        .text()
        .parse()
        .map_err(|_| SyncError::new("bad y number"))?;

    let (_, after_x) = replace_token_text(&x_tok, &format_drag_number(x + dx));
    let root2 = parse_root(&after_x)?;
    let (_, y_tok2) = find_nth_number_pair(&root2, head, index, x_slot, y_slot)
        .ok_or_else(|| SyncError::new(format!("`{head}` #{index} lost after x patch")))?;
    let (_, after_y) = replace_token_text(&y_tok2, &format_drag_number(y + dy));
    Ok(after_y)
}

fn multiply_nth_number(
    src: &str,
    head: &str,
    index: usize,
    slot: usize,
    factor: f64,
) -> Result<String, SyncError> {
    let root = parse_root(src)?;
    let tok = find_nth_number(&root, head, index, slot)
        .ok_or_else(|| SyncError::new(format!("no `{head}` #{index} slot {slot}")))?;
    let v: f64 = tok
        .text()
        .parse()
        .map_err(|_| SyncError::new("bad size number"))?;
    let (_, out) = replace_token_text(&tok, &format_drag_number(v * factor));
    Ok(out)
}

fn multiply_nth_number_optional(
    src: &str,
    head: &str,
    index: usize,
    slot: usize,
    factor: f64,
) -> Result<String, SyncError> {
    let root = parse_root(src)?;
    if find_nth_number(&root, head, index, slot).is_none() {
        return Ok(src.to_string());
    }
    multiply_nth_number(src, head, index, slot, factor)
}

fn scale_xy_pairs_about_centroid(
    src: &str,
    head: &str,
    index: usize,
    factor: f64,
) -> Result<String, SyncError> {
    let root = parse_root(src)?;
    let n = count_leading_number_coords(&root, head, index)
        .ok_or_else(|| SyncError::new(format!("no `{head}` #{index}")))?;
    if n < 4 || !n.is_multiple_of(2) {
        return Err(SyncError::new(format!(
            "`{head}` #{index} needs an even number of coordinates (≥4)"
        )));
    }
    let pairs = n / 2;
    let mut pts = Vec::with_capacity(pairs);
    for p in 0..pairs {
        let x = read_nth_number(&root, head, index, 1 + p * 2)?;
        let y = read_nth_number(&root, head, index, 2 + p * 2)?;
        pts.push((x, y));
    }
    let cx = pts.iter().map(|p| p.0).sum::<f64>() / pairs as f64;
    let cy = pts.iter().map(|p| p.1).sum::<f64>() / pairs as f64;
    let mut out = src.to_string();
    for (p, (x, y)) in pts.into_iter().enumerate() {
        let nx = cx + (x - cx) * factor;
        let ny = cy + (y - cy) * factor;
        out = set_nth_number(&out, head, index, 1 + p * 2, nx)?;
        out = set_nth_number(&out, head, index, 2 + p * 2, ny)?;
    }
    Ok(out)
}

/// Scale trailing `(polyline … color width)` width when present.
fn scale_polyline_trailing_width(
    src: &str,
    index: usize,
    factor: f64,
) -> Result<String, SyncError> {
    let root = parse_root(src)?;
    let mut seen = 0usize;
    for node in root.descendants() {
        if node.kind() != SyntaxKind::List {
            continue;
        }
        let items = list_atoms(&node);
        let Some(Child::Token(h)) = items.first() else {
            continue;
        };
        if h.kind() != SyntaxKind::Ident || h.text() != "polyline" {
            continue;
        }
        if seen == index {
            // Width is last atom when it is a number and the previous atom is a color.
            let last = items.len().checked_sub(1);
            let prev = items.len().checked_sub(2);
            let (Some(li), Some(pi)) = (last, prev) else {
                return Ok(src.to_string());
            };
            let width_is_num =
                matches!(&items[li], Child::Token(t) if t.kind() == SyntaxKind::Number);
            let prev_is_color = match &items[pi] {
                Child::Token(t) if t.kind() == SyntaxKind::Ident => true,
                Child::Node(n) => {
                    let inner = list_atoms(n);
                    matches!(
                        inner.first(),
                        Some(Child::Token(t)) if t.kind() == SyntaxKind::Ident && t.text() == "rgb"
                    )
                }
                _ => false,
            };
            if width_is_num && prev_is_color {
                return multiply_nth_number(src, "polyline", index, li, factor);
            }
            return Ok(src.to_string());
        }
        seen += 1;
    }
    Ok(src.to_string())
}

fn scale_box_about_center(
    src: &str,
    head: &str,
    index: usize,
    // [x, y, w, h] atom slots inside the list.
    slots: [usize; 4],
    factor: f64,
) -> Result<String, SyncError> {
    scale_box_axes(src, head, index, slots, factor, factor)
}

/// Scale a box leaf's width/height independently about its center.
pub fn scale_box_axes(
    src: &str,
    head: &str,
    index: usize,
    slots: [usize; 4],
    fx: f64,
    fy: f64,
) -> Result<String, SyncError> {
    if !(fx.is_finite() && fy.is_finite() && fx > 0.0 && fy > 0.0) {
        return Err(SyncError::new("scale factors must be finite and > 0"));
    }
    let root = parse_root(src)?;
    let x = read_nth_number(&root, head, index, slots[0])?;
    let y = read_nth_number(&root, head, index, slots[1])?;
    let w = read_nth_number(&root, head, index, slots[2])?;
    let h = read_nth_number(&root, head, index, slots[3])?;
    let cx = x + w * 0.5;
    let cy = y + h * 0.5;
    let nw = (w * fx).max(0.5);
    let nh = (h * fy).max(0.5);
    let nx = cx - nw * 0.5;
    let ny = cy - nh * 0.5;
    let mut out = set_nth_number(src, head, index, slots[0], nx)?;
    out = set_nth_number(&out, head, index, slots[1], ny)?;
    out = set_nth_number(&out, head, index, slots[2], nw)?;
    set_nth_number(&out, head, index, slots[3], nh)
}

/// Resize one axis of a size target (width via `fx`, height via `fy`).
pub fn scale_size_target_axes(
    src: &str,
    target: SizeTarget,
    fx: f64,
    fy: f64,
) -> Result<String, SyncError> {
    if !(fx.is_finite() && fy.is_finite() && fx > 0.0 && fy > 0.0) {
        return Err(SyncError::new("scale factors must be finite and > 0"));
    }
    match target {
        SizeTarget::TextSize(i) => scale_text_box(src, i, fx, fy),
        SizeTarget::RectWh(i) => scale_box_axes(src, "rect", i, [1, 2, 3, 4], fx, fy),
        SizeTarget::FrameWh(i) => scale_box_axes(src, "frame", i, [1, 2, 3, 4], fx, fy),
        SizeTarget::ImageWh(i) => scale_box_axes(src, "image", i, [2, 3, 4, 5], fx, fy),
        SizeTarget::EllipseRxRy(i) => {
            let after = multiply_nth_number(src, "ellipse", i, 3, fx)?;
            multiply_nth_number(&after, "ellipse", i, 4, fy)
        }
        // Circles/rings have one radius; width and height change together.
        SizeTarget::CircleR(_)
        | SizeTarget::RingR(_)
        | SizeTarget::LineSeg(_)
        | SizeTarget::PolylinePoints(_)
        | SizeTarget::PolygonPoints(_)
        | SizeTarget::Unsupported => scale_size_target(src, target, fx.max(fy)),
    }
}

fn read_nth_text_content(root: &SyntaxNode, index: usize) -> Result<String, SyncError> {
    let mut seen = 0usize;
    for node in root.descendants() {
        if node.kind() != SyntaxKind::List {
            continue;
        }
        let items = list_atoms(&node);
        let Some(Child::Token(h)) = items.first() else {
            continue;
        };
        if h.kind() != SyntaxKind::Ident || h.text() != "text" {
            continue;
        }
        if seen == index {
            let content = items.iter().find_map(|c| match c {
                Child::Token(t) if t.kind() == SyntaxKind::String => {
                    Some(t.text().trim_matches('"').to_string())
                }
                _ => None,
            });
            return content.ok_or_else(|| SyncError::new(format!("no `text` #{index} string")));
        }
        seen += 1;
    }
    Err(SyncError::new(format!("no `text` #{index}")))
}

fn read_nth_number(
    root: &SyntaxNode,
    head: &str,
    index: usize,
    slot: usize,
) -> Result<f64, SyncError> {
    let tok = find_nth_number(root, head, index, slot)
        .ok_or_else(|| SyncError::new(format!("no `{head}` #{index} slot {slot}")))?;
    tok.text().parse().map_err(|_| SyncError::new("bad number"))
}

fn set_nth_number(
    src: &str,
    head: &str,
    index: usize,
    slot: usize,
    value: f64,
) -> Result<String, SyncError> {
    let root = parse_root(src)?;
    let tok = find_nth_number(&root, head, index, slot)
        .ok_or_else(|| SyncError::new(format!("no `{head}` #{index} slot {slot}")))?;
    let (_, out) = replace_token_text(&tok, &format_drag_number(value));
    Ok(out)
}

fn find_nth_number(
    root: &SyntaxNode,
    head: &str,
    index: usize,
    slot: usize,
) -> Option<SyntaxToken> {
    let mut seen = 0usize;
    for node in root.descendants() {
        if node.kind() != SyntaxKind::List {
            continue;
        }
        let items = list_atoms(&node);
        let Some(Child::Token(h)) = items.first() else {
            continue;
        };
        if h.kind() != SyntaxKind::Ident || h.text() != head {
            continue;
        }
        if seen == index {
            return match items.get(slot) {
                Some(Child::Token(t)) if t.kind() == SyntaxKind::Number => Some(t.clone()),
                _ => None,
            };
        }
        seen += 1;
    }
    None
}

fn nudge_line(src: &str, index: usize, dx: f64, dy: f64) -> Result<String, SyncError> {
    // Move both endpoints: slots 1,2 then 3,4.
    let after = nudge_nth_pair(src, "line", index, 1, 2, dx, dy)?;
    nudge_nth_pair(&after, "line", index, 3, 4, dx, dy)
}

fn nudge_polyline(src: &str, index: usize, dx: f64, dy: f64) -> Result<String, SyncError> {
    let root = parse_root(src)?;
    let n_coords = count_leading_number_coords(&root, "polyline", index)
        .ok_or_else(|| SyncError::new(format!("no `polyline` #{index}")))?;
    let mut out = src.to_string();
    let pairs = n_coords / 2;
    for p in 0..pairs {
        let x_slot = 1 + p * 2;
        let y_slot = x_slot + 1;
        out = nudge_nth_pair(&out, "polyline", index, x_slot, y_slot, dx, dy)?;
    }
    Ok(out)
}

fn nudge_polygon(src: &str, index: usize, dx: f64, dy: f64) -> Result<String, SyncError> {
    let root = parse_root(src)?;
    let n_coords = count_leading_number_coords(&root, "polygon", index)
        .ok_or_else(|| SyncError::new(format!("no `polygon` #{index}")))?;
    let mut out = src.to_string();
    let pairs = n_coords / 2;
    for p in 0..pairs {
        let x_slot = 1 + p * 2;
        let y_slot = x_slot + 1;
        out = nudge_nth_pair(&out, "polygon", index, x_slot, y_slot, dx, dy)?;
    }
    Ok(out)
}

fn count_leading_number_coords(root: &SyntaxNode, head: &str, index: usize) -> Option<usize> {
    let mut seen = 0usize;
    for node in root.descendants() {
        if node.kind() != SyntaxKind::List {
            continue;
        }
        let items = list_atoms(&node);
        let Some(Child::Token(h)) = items.first() else {
            continue;
        };
        if h.kind() != SyntaxKind::Ident || h.text() != head {
            continue;
        }
        if seen == index {
            let mut n = 0usize;
            for item in items.iter().skip(1) {
                match item {
                    Child::Token(t) if t.kind() == SyntaxKind::Number => n += 1,
                    _ => break,
                }
            }
            return Some(n);
        }
        seen += 1;
    }
    None
}

fn find_nth_number_pair(
    root: &SyntaxNode,
    head: &str,
    index: usize,
    x_slot: usize,
    y_slot: usize,
) -> Option<(SyntaxToken, SyntaxToken)> {
    let mut seen = 0usize;
    for node in root.descendants() {
        if node.kind() != SyntaxKind::List {
            continue;
        }
        let items = list_atoms(&node);
        let Some(Child::Token(h)) = items.first() else {
            continue;
        };
        if h.kind() != SyntaxKind::Ident || h.text() != head {
            continue;
        }
        if seen == index {
            let x = match items.get(x_slot) {
                Some(Child::Token(t)) if t.kind() == SyntaxKind::Number => t.clone(),
                _ => return None,
            };
            let y = match items.get(y_slot) {
                Some(Child::Token(t)) if t.kind() == SyntaxKind::Number => t.clone(),
                _ => return None,
            };
            return Some((x, y));
        }
        seen += 1;
    }
    None
}

fn parse_root(src: &str) -> Result<SyntaxNode, SyncError> {
    parse_source(src)
        .into_result()
        .map_err(|e| SyncError::new(format!("parse error: {}", e[0].message)))
}

fn is_list_headed(node: &SyntaxNode, name: &str) -> bool {
    if node.kind() != SyntaxKind::List {
        return false;
    }
    let items = list_atoms(node);
    matches!(items.first(), Some(Child::Token(t)) if t.kind() == SyntaxKind::Ident && t.text() == name)
}

enum Child {
    Token(SyntaxToken),
    Node(SyntaxNode),
}

fn list_atoms(node: &SyntaxNode) -> Vec<Child> {
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
