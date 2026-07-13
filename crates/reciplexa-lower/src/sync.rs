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
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
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
            for item in items.iter().skip(3) {
                if let Child::Node(n) = item {
                    collect_from_shape(n, Some(idx), counters, out);
                }
            }
        }
        "rotate" | "scale" | "group" | "opacity" => {
            let skip = if head.text() == "scale" {
                // (scale s …) or (scale sx sy …)
                if items.len() >= 4
                    && matches!(&items[1], Child::Token(t) if t.kind() == SyntaxKind::Number)
                    && matches!(&items[2], Child::Token(t) if t.kind() == SyntaxKind::Number)
                {
                    3
                } else {
                    2
                }
            } else if head.text() == "group" {
                1
            } else {
                // rotate / opacity: (head num shape…)
                2
            };
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
}
