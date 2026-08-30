//! Package-shaped AST locators and layer sync for GUI CST sync v2.
//!
//! Finds `(val main …)` and nested `(page …)` forms, treating `fill` / `stroke` /
//! `paint` / `list` as transparent wrappers (matching eval bridge flatten).
//! Locate via CST spans; mutate authoring strings (hybrid, same as interim sync).

use reciplexa_syntax::{
    decode_string_literal, format_drag_number, replace_token_text, SyntaxKind, SyntaxNode,
    SyntaxToken,
};

use super::layers::{reorder_among_shared_root, rewrite_form_order};
use super::{extent_with_leading_ws, is_headed, parse_root, LayerInfo, SizeTarget, SyncError};
use crate::cst_walk::{find_list_covering, list_atoms, Child};

/// Heads that wrap a single shape child at slot 1 (`fill`/`stroke`/`paint`).
pub fn is_package_paint_wrapper(head: &str) -> bool {
    matches!(head, "fill" | "stroke" | "paint")
}

/// Paint wrappers plus `list` — transparent for layer flatten under a page.
pub fn is_package_transparent_wrapper(head: &str) -> bool {
    is_package_paint_wrapper(head) || head == "list"
}

/// Cheap authoring-shape check (no env overrides): import + `val main`.
///
/// Prefer this in `reciplexa-lower` / GUI sync to avoid depending on pipeline
/// env flags for editability. Markup authoring stays false.
pub fn is_package_shaped_authoring(src: &str) -> bool {
    (src.contains("(import graphics") || src.contains("(import document"))
        && src.contains("(val main")
}

/// Top-level `(val main EXPR)` → the `EXPR` node.
pub fn find_main_expr(root: &SyntaxNode) -> Result<SyntaxNode, SyncError> {
    for form in root.children() {
        if form.kind() == SyntaxKind::StructuredComment {
            continue;
        }
        let items = list_atoms(&form);
        let Some(Child::Token(head)) = items.first() else {
            continue;
        };
        if head.kind() != SyntaxKind::Ident || head.text() != "val" {
            continue;
        }
        let Some(Child::Token(name)) = items.get(1) else {
            continue;
        };
        if name.kind() != SyntaxKind::Ident || name.text() != "main" {
            continue;
        }
        return match items.get(2) {
            Some(Child::Node(n)) => Ok(n.clone()),
            _ => Err(SyncError::new("val main missing expression")),
        };
    }
    Err(SyncError::new("no top-level (val main …)"))
}

/// Nth `(page …)` under `(val main …)` (direct page, `(list …)` of pages, or `(pages …)`).
pub fn find_package_page(root: &SyntaxNode, page_index: usize) -> Result<SyntaxNode, SyncError> {
    let main = find_main_expr(root)?;
    let pages = collect_package_pages(&main)?;
    pages
        .into_iter()
        .nth(page_index)
        .ok_or_else(|| SyncError::new(format!("no package page #{page_index}")))
}

/// All `(page …)` nodes under a main expression, in document order.
pub fn collect_package_pages(main_expr: &SyntaxNode) -> Result<Vec<SyntaxNode>, SyncError> {
    if is_headed(main_expr, "page") {
        return Ok(vec![main_expr.clone()]);
    }
    if is_headed(main_expr, "list") {
        let mut pages = Vec::new();
        for item in list_atoms(main_expr).into_iter().skip(1) {
            if let Child::Node(n) = item {
                if is_headed(&n, "page") {
                    pages.push(n);
                }
            }
        }
        if pages.is_empty() {
            return Err(SyncError::new("main list has no (page …) children"));
        }
        return Ok(pages);
    }
    if is_headed(main_expr, "pages") {
        // `(pages ITEMS)` — ITEMS is typically a `(list …)` of pages.
        let items = list_atoms(main_expr);
        let Some(Child::Node(items_node)) = items.get(1) else {
            return Err(SyncError::new("pages missing items"));
        };
        return collect_package_pages(items_node);
    }
    Err(SyncError::new(
        "val main must be (page …), (list (page …) …), or (pages …)",
    ))
}

/// Shape / content children of a package `(page SIZE CONTENT)`.
///
/// `CONTENT` may be a single shape, a `(list …)` of shapes, or
/// `(group (list …))` / `(group child …)`. Group/list wrappers are flattened so
/// each drawable root (fill/stroke/translate/…) is independently editable.
pub fn package_page_content_nodes(page: &SyntaxNode) -> Result<Vec<SyntaxNode>, SyncError> {
    if !is_headed(page, "page") {
        return Err(SyncError::new("expected (page …)"));
    }
    let items = list_atoms(page);
    // `(page SIZE CONTENT)` — SIZE is token or node (a4 / page-size …).
    let Some(content) = items.get(2) else {
        return Err(SyncError::new("page missing content"));
    };
    match content {
        Child::Node(n) => Ok(flatten_package_content_nodes(n)),
        _ => Err(SyncError::new("page content must be a list form")),
    }
}

fn flatten_package_content_nodes(n: &SyntaxNode) -> Vec<SyntaxNode> {
    if is_headed(n, "list") {
        let mut out = Vec::new();
        for item in list_atoms(n).into_iter().skip(1) {
            if let Child::Node(c) = item {
                out.push(c);
            }
        }
        return out;
    }
    if is_headed(n, "group") {
        let kids: Vec<SyntaxNode> = list_atoms(n)
            .into_iter()
            .skip(1)
            .filter_map(|c| match c {
                Child::Node(inner) => Some(inner),
                _ => None,
            })
            .collect();
        if kids.len() == 1 && is_headed(&kids[0], "list") {
            return flatten_package_content_nodes(&kids[0]);
        }
        if kids.is_empty() {
            return vec![n.clone()];
        }
        return kids;
    }
    vec![n.clone()]
}

/// Immediate shape child under a paint wrapper (`fill`/`stroke`/`paint`), if any.
pub fn paint_wrapper_shape(node: &SyntaxNode) -> Option<SyntaxNode> {
    let items = list_atoms(node);
    let Some(Child::Token(head)) = items.first() else {
        return None;
    };
    if head.kind() != SyntaxKind::Ident || !is_package_paint_wrapper(head.text()) {
        return None;
    }
    match items.get(1) {
        Some(Child::Node(n)) => Some(n.clone()),
        _ => None,
    }
}

/// Parse + [`find_main_expr`] convenience.
pub fn find_main_expr_in_source(src: &str) -> Result<SyntaxNode, SyncError> {
    let root = parse_root(src)?;
    find_main_expr(&root)
}

/// Number of `(page …)` forms under `(val main …)`.
pub fn count_package_pages(src: &str) -> Result<usize, SyncError> {
    let root = parse_root(src)?;
    let main = find_main_expr(&root)?;
    Ok(collect_package_pages(&main)?.len())
}

/// Parse + [`find_package_page`] convenience.
pub fn find_package_page_in_source(src: &str, page_index: usize) -> Result<SyntaxNode, SyncError> {
    let root = parse_root(src)?;
    find_package_page(&root, page_index)
}

/// Collect flattened drawable layers under package `(val main (page …))`.
///
/// Order matches bridge flatten / hit-test. Paint wrappers are transparent;
/// `root_*` spans point at the page-content child that owns the leaf.
pub fn collect_layers_package(src: &str, page_index: usize) -> Result<Vec<LayerInfo>, SyncError> {
    let root = parse_root(src)?;
    let page = find_package_page(&root, page_index)?;
    let contents = package_page_content_nodes(&page)?;
    let mut out = Vec::new();
    for content in contents {
        let rr = content.text_range();
        let root_span = (usize::from(rr.start()), usize::from(rr.end()));
        collect_layers_from_package_shape(&content, root_span, &mut out);
    }
    Ok(out)
}

/// Nudge flattened package layer `flat_index` by `(dx, dy)` in page mm.
///
/// Mirrors interim `nudge_layer_page` wrap rules on the page-content root:
/// existing `(translate …)` (or opacity→translate) is edited in place; bare
/// `(rotate …)` roots get a translate wrap. Paint-wrapped / bare leaves still
/// patch geometry numbers in place (S1 circle behavior).
pub fn nudge_layer_package(
    src: &str,
    page_index: usize,
    flat_index: usize,
    dx: f64,
    dy: f64,
) -> Result<String, SyncError> {
    if dx == 0.0 && dy == 0.0 {
        return Ok(src.to_string());
    }
    let layers = collect_layers_package(src, page_index)?;
    let layer = layers
        .get(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    let root = parse_root(src).expect("parse ok after collect_layers_package");
    let root_node = find_list_covering(&root, layer.root_start, layer.root_end)
        .ok_or_else(|| SyncError::new("layer root span missing after parse"))?;

    if is_headed(&root_node, "translate") {
        return nudge_xy_slots_of_list(&root_node, 1, 2, dx, dy);
    }
    if is_headed(&root_node, "opacity") {
        if let Some(child) = first_package_shape_child(&root_node) {
            if is_headed(&child, "translate") {
                return nudge_xy_slots_of_list(&child, 1, 2, dx, dy);
            }
        }
    }
    if is_headed(&root_node, "rotate") || is_headed(&root_node, "scale") {
        return wrap_span_with_translate(src, layer.root_start, layer.root_end, dx, dy);
    }

    let leaf = find_list_covering(&root, layer.byte_start, layer.byte_end)
        .ok_or_else(|| SyncError::new("layer leaf span missing after parse"))?;
    let (x_slot, y_slot) = match layer.kind.as_str() {
        "circle" | "rect" | "ellipse" | "ring" | "frame" | "text" => (1usize, 2usize),
        "image" => (2, 3),
        "line" => {
            let after = nudge_xy_slots_of_list(&leaf, 1, 2, dx, dy)?;
            let root2 = parse_root(&after).expect("parse ok after line p1 patch");
            let leaf2 = find_list_covering(&root2, layer.byte_start, layer.byte_end)
                .or_else(|| {
                    root2.descendants().find(|n| {
                        n.kind() == SyntaxKind::List
                            && usize::from(n.text_range().start()) == layer.byte_start
                    })
                })
                .ok_or_else(|| SyncError::new("line form missing after p1 patch"))?;
            return nudge_xy_slots_of_list(&leaf2, 3, 4, dx, dy);
        }
        "polyline" | "polygon" | "path" => {
            return Err(SyncError::new(format!(
                "package nudge unsupported for kind `{}`",
                layer.kind
            )));
        }
        other => {
            return Err(SyncError::new(format!(
                "package nudge unsupported for kind `{other}`"
            )));
        }
    };
    nudge_xy_slots_of_list(&leaf, x_slot, y_slot, dx, dy)
}

fn first_package_shape_child(node: &SyntaxNode) -> Option<SyntaxNode> {
    let items = list_atoms(node);
    let skip = if is_headed(node, "translate") {
        3
    } else if is_headed(node, "scale") {
        if items.len() >= 4
            && matches!(&items[1], Child::Token(t) if t.kind() == SyntaxKind::Number)
            && matches!(&items[2], Child::Token(t) if t.kind() == SyntaxKind::Number)
        {
            3
        } else {
            2
        }
    } else if is_headed(node, "group") || is_headed(node, "list") {
        1
    } else {
        // opacity / rotate / paint wrappers
        2
    };
    items.into_iter().skip(skip).find_map(|c| match c {
        Child::Node(n) => Some(n),
        _ => None,
    })
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

/// Collect one [`SizeTarget`] per flattened drawable under a package page.
pub fn collect_size_targets_package(
    src: &str,
    page_index: usize,
) -> Result<Vec<SizeTarget>, SyncError> {
    let root = parse_root(src)?;
    let page = find_package_page(&root, page_index)?;
    let contents = package_page_content_nodes(&page)?;
    let mut out = Vec::new();
    let mut counters = PackageSizeCounters::default();
    for content in contents {
        collect_size_from_package_shape(&content, &mut counters, &mut out);
    }
    Ok(out)
}

#[derive(Default)]
struct PackageSizeCounters {
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

fn collect_size_from_package_shape(
    node: &SyntaxNode,
    counters: &mut PackageSizeCounters,
    out: &mut Vec<SizeTarget>,
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
        "fill" | "stroke" | "paint" => {
            if let Some(Child::Node(n)) = items.get(1) {
                collect_size_from_package_shape(n, counters, out);
            }
        }
        "list" | "group" => {
            for item in items.iter().skip(1) {
                if let Child::Node(n) = item {
                    collect_size_from_package_shape(n, counters, out);
                }
            }
        }
        "translate" => {
            for item in items.iter().skip(3) {
                if let Child::Node(n) = item {
                    collect_size_from_package_shape(n, counters, out);
                }
            }
        }
        "rotate" | "opacity" => {
            for item in items.iter().skip(2) {
                if let Child::Node(n) = item {
                    collect_size_from_package_shape(n, counters, out);
                }
            }
        }
        "scale" => {
            let skip = if items.len() >= 4
                && matches!(&items[1], Child::Token(t) if t.kind() == SyntaxKind::Number)
                && matches!(&items[2], Child::Token(t) if t.kind() == SyntaxKind::Number)
            {
                3
            } else {
                2
            };
            for item in items.iter().skip(skip) {
                if let Child::Node(n) = item {
                    collect_size_from_package_shape(n, counters, out);
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
        "path" => out.push(SizeTarget::Unsupported),
        _ => {}
    }
}

fn collect_layers_from_package_shape(
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
        "fill" | "stroke" | "paint" => {
            if let Some(Child::Node(n)) = items.get(1) {
                collect_layers_from_package_shape(n, root_span, out);
            }
        }
        "list" | "group" => {
            for item in items.iter().skip(1) {
                if let Child::Node(n) = item {
                    collect_layers_from_package_shape(n, root_span, out);
                }
            }
        }
        "translate" => {
            for item in items.iter().skip(3) {
                if let Child::Node(n) = item {
                    collect_layers_from_package_shape(n, root_span, out);
                }
            }
        }
        "rotate" | "opacity" => {
            for item in items.iter().skip(2) {
                if let Child::Node(n) = item {
                    collect_layers_from_package_shape(n, root_span, out);
                }
            }
        }
        "scale" => {
            let skip = if items.len() >= 4
                && matches!(&items[1], Child::Token(t) if t.kind() == SyntaxKind::Number)
                && matches!(&items[2], Child::Token(t) if t.kind() == SyntaxKind::Number)
            {
                3
            } else {
                2
            };
            for item in items.iter().skip(skip) {
                if let Child::Node(n) = item {
                    collect_layers_from_package_shape(n, root_span, out);
                }
            }
        }
        "circle" | "rect" | "ellipse" | "ring" | "frame" | "text" | "line" | "polyline"
        | "polygon" | "image" | "path" => {
            let range = node.text_range();
            let mut layer = LayerInfo::new(
                kind,
                package_layer_label(kind, &items),
                range.start().into(),
                range.end().into(),
                root_span.0,
                root_span.1,
            );
            if kind == "text" {
                if let Some(text) = package_first_decoded_string(&items) {
                    layer = layer.with_text_content(text);
                }
            }
            out.push(layer);
        }
        _ => {}
    }
}

fn package_first_decoded_string(items: &[Child]) -> Option<String> {
    items.iter().find_map(|c| match c {
        Child::Token(t) if t.kind() == SyntaxKind::String => decode_string_literal(t.text()).ok(),
        _ => None,
    })
}

fn package_layer_label(kind: &str, items: &[Child]) -> String {
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

fn nudge_xy_slots_of_list(
    node: &SyntaxNode,
    x_slot: usize,
    y_slot: usize,
    dx: f64,
    dy: f64,
) -> Result<String, SyncError> {
    let items = list_atoms(node);
    let x_tok =
        number_token_at(&items, x_slot).ok_or_else(|| SyncError::new("shape missing numeric x"))?;
    let y_tok =
        number_token_at(&items, y_slot).ok_or_else(|| SyncError::new("shape missing numeric y"))?;
    let x: f64 =
        reciplexa_syntax::parse_number_literal(x_tok.text()).expect("lexer Number parses as f64");
    let y: f64 =
        reciplexa_syntax::parse_number_literal(y_tok.text()).expect("lexer Number parses as f64");
    let (_, after_x) = replace_token_text(&x_tok, &format_drag_number(x + dx));
    let root2 = parse_root(&after_x).expect("parse ok after x patch");
    let start = usize::from(node.text_range().start());
    let end = usize::from(node.text_range().end());
    let node2 = find_list_covering(&root2, start, end)
        .or_else(|| {
            root2.descendants().find(|n| {
                n.kind() == SyntaxKind::List && usize::from(n.text_range().start()) == start
            })
        })
        .expect("shape form still present after x patch");
    let items2 = list_atoms(&node2);
    let y_tok2 = number_token_at(&items2, y_slot).expect("y still numeric after x patch");
    let (_, after_y) = replace_token_text(&y_tok2, &format_drag_number(y + dy));
    Ok(after_y)
}

fn number_token_at(items: &[Child], slot: usize) -> Option<SyntaxToken> {
    match items.get(slot) {
        Some(Child::Token(t)) if t.kind() == SyntaxKind::Number => Some(t.clone()),
        _ => None,
    }
}

/// Insert `form` as a new page-content child of package `(val main (page …))`.
///
/// `form` must be a complete list such as `(fill (circle 105 148.5 20) black)`.
pub fn insert_layer_package(
    src: &str,
    page_index: usize,
    form: &str,
) -> Result<(String, usize), SyncError> {
    let form = form.trim();
    if form.is_empty() || !form.starts_with('(') || !form.ends_with(')') {
        return Err(SyncError::new("insert form must be a complete (…) list"));
    }
    let root = parse_root(src)?;
    let page = find_package_page(&root, page_index)?;
    let wrap_span: Option<(usize, usize)> = {
        let items = list_atoms(&page);
        match items.get(2) {
            Some(Child::Node(n)) if is_headed(n, "list") => None,
            Some(Child::Node(n)) if is_headed(n, "group") => {
                let has_list = list_atoms(n)
                    .iter()
                    .skip(1)
                    .any(|c| matches!(c, Child::Node(inner) if is_headed(inner, "list")));
                if has_list {
                    None
                } else {
                    let r = n.text_range();
                    Some((usize::from(r.start()), usize::from(r.end())))
                }
            }
            Some(Child::Node(n)) => {
                let r = n.text_range();
                Some((usize::from(r.start()), usize::from(r.end())))
            }
            _ => None,
        }
    };
    let out = if let Some((start, end)) = wrap_span {
        let old = &src[start..end];
        let wrapped = format!("(list {old} {form})");
        let mut next = String::with_capacity(src.len() + wrapped.len());
        next.push_str(&src[..start]);
        next.push_str(&wrapped);
        next.push_str(&src[end..]);
        next
    } else {
        let insert_at = package_content_insert_at(src, &page)?;
        let pad = if src[..insert_at].contains('\n') {
            "\n        "
        } else {
            " "
        };
        let mut next = String::with_capacity(src.len() + form.len() + pad.len());
        next.push_str(&src[..insert_at]);
        next.push_str(pad);
        next.push_str(form);
        next.push_str(&src[insert_at..]);
        next
    };
    let layers = collect_layers_package(&out, page_index)?;
    let idx = layers
        .len()
        .checked_sub(1)
        .ok_or_else(|| SyncError::new("insert produced no layers"))?;
    Ok((out, idx))
}

fn package_content_insert_at(src: &str, page: &SyntaxNode) -> Result<usize, SyncError> {
    let _ = src;
    let items = list_atoms(page);
    let Some(content) = items.get(2) else {
        let page_end = usize::from(page.text_range().end());
        return Ok(page_end.saturating_sub(1));
    };
    match content {
        Child::Node(n) if is_headed(n, "list") => {
            Ok(usize::from(n.text_range().end()).saturating_sub(1))
        }
        Child::Node(n) if is_headed(n, "group") => {
            for item in list_atoms(n).into_iter().skip(1) {
                if let Child::Node(inner) = item {
                    if is_headed(&inner, "list") {
                        return Ok(usize::from(inner.text_range().end()).saturating_sub(1));
                    }
                }
            }
            Ok(usize::from(n.text_range().end()).saturating_sub(1))
        }
        Child::Node(n) => Err(SyncError::new(format!(
            "page content is a single form @{}..{}",
            usize::from(n.text_range().start()),
            usize::from(n.text_range().end())
        ))),
        _ => Err(SyncError::new("page content must be a list form")),
    }
}

/// Remove flattened package layer `flat_index` (rewrites `.rpx`).
pub fn delete_layer_package(
    src: &str,
    page_index: usize,
    flat_index: usize,
) -> Result<String, SyncError> {
    let layers = collect_layers_package(src, page_index)?;
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

/// Move flattened package layer `from` to flatten index `to`.
pub fn reorder_layer_package(
    src: &str,
    page_index: usize,
    from: usize,
    to: usize,
) -> Result<String, SyncError> {
    if from == to {
        return Ok(src.to_string());
    }
    let layers = collect_layers_package(src, page_index)?;
    if from >= layers.len() || to >= layers.len() {
        return Err(SyncError::new("layer index out of range"));
    }
    let from_root = (layers[from].root_start, layers[from].root_end);
    let to_root = (layers[to].root_start, layers[to].root_end);
    if from_root == to_root {
        reorder_among_shared_root(src, &layers, from, to, from_root)
    } else {
        reorder_package_roots(src, page_index, from_root, to_root)
    }
}

fn reorder_package_roots(
    src: &str,
    page_index: usize,
    from_root: (usize, usize),
    to_root: (usize, usize),
) -> Result<String, SyncError> {
    let root = parse_root(src).expect("parse ok after collect_layers_package");
    let page = find_package_page(&root, page_index).expect("page exists after collect");
    let contents = package_page_content_nodes(&page)?;
    let mut forms: Vec<(usize, usize)> = Vec::new();
    for n in &contents {
        if is_headed(n, "list") {
            for item in list_atoms(n).into_iter().skip(1) {
                if let Child::Node(c) = item {
                    let r = c.text_range();
                    forms.push((usize::from(r.start()), usize::from(r.end())));
                }
            }
        } else {
            let r = n.text_range();
            forms.push((usize::from(r.start()), usize::from(r.end())));
        }
    }
    let mut fi = None;
    let mut ti = None;
    for (i, r) in forms.iter().enumerate() {
        if *r == from_root {
            fi = Some(i);
        }
        if *r == to_root {
            ti = Some(i);
        }
    }
    let fi = fi.ok_or_else(|| SyncError::new("from root missing on package page"))?;
    let ti = ti.ok_or_else(|| SyncError::new("to root missing on package page"))?;
    let item = forms.remove(fi);
    forms.insert(ti, item);
    Ok(rewrite_form_order(src, &forms))
}

/// Replace the string literal of a package `(text …)` leaf.
pub fn set_text_content_package(
    src: &str,
    page_index: usize,
    flat_index: usize,
    text: &str,
) -> Result<String, SyncError> {
    let layers = collect_layers_package(src, page_index)?;
    let layer = layers
        .get(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    if layer.kind != "text" {
        return Err(SyncError::new("text content only on text shapes"));
    }
    let root = parse_root(src)?;
    let leaf = find_list_covering(&root, layer.byte_start, layer.byte_end)
        .ok_or_else(|| SyncError::new("text leaf span missing"))?;
    let items = list_atoms(&leaf);
    let tok = items
        .iter()
        .find_map(|c| match c {
            Child::Token(t) if t.kind() == SyntaxKind::String => Some(t.clone()),
            _ => None,
        })
        .ok_or_else(|| SyncError::new("text content string missing"))?;
    let escaped = reciplexa_syntax::encode_string_literal(text);
    let (_, out) = replace_token_text(&tok, &escaped);
    Ok(out)
}

/// Duplicate flattened package layer `flat_index` (inserts a copy after it).
pub fn duplicate_layer_package(
    src: &str,
    page_index: usize,
    flat_index: usize,
) -> Result<String, SyncError> {
    let layers = collect_layers_package(src, page_index)?;
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
    let insert_at = extent_with_leading_ws(src, span_start, span_end).1;
    let pad = if src[..span_start].ends_with('\n') || snippet.contains('\n') {
        "\n        "
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

#[cfg(test)]
mod tests {
    use super::*;

    const PKG_CIRCLE: &str = r#"(import graphics/shapes only circle fill)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main (page a4 (fill (circle 105 148.5 40) black)))
"#;

    #[test]
    fn find_main_and_page_on_pkg_black_circle() {
        let root = parse_root(PKG_CIRCLE).unwrap();
        let main = find_main_expr(&root).unwrap();
        assert!(is_headed(&main, "page"));
        let page = find_package_page(&root, 0).unwrap();
        assert!(is_headed(&page, "page"));
        let contents = package_page_content_nodes(&page).unwrap();
        assert_eq!(contents.len(), 1);
        assert!(is_headed(&contents[0], "fill"));
        let shape = paint_wrapper_shape(&contents[0]).unwrap();
        assert!(is_headed(&shape, "circle"));
        let range = shape.text_range();
        let slice = &PKG_CIRCLE[usize::from(range.start())..usize::from(range.end())];
        assert!(slice.starts_with("(circle 105"));
    }

    #[test]
    fn paint_and_list_wrappers_are_transparent_flags() {
        assert!(is_package_paint_wrapper("fill"));
        assert!(is_package_paint_wrapper("stroke"));
        assert!(is_package_paint_wrapper("paint"));
        assert!(!is_package_paint_wrapper("circle"));
        assert!(is_package_transparent_wrapper("list"));
        assert!(is_package_transparent_wrapper("fill"));
        assert!(!is_package_transparent_wrapper("translate"));
    }

    #[test]
    fn package_shaped_authoring_detects_imports() {
        assert!(is_package_shaped_authoring(PKG_CIRCLE));
        assert!(!is_package_shaped_authoring("(page a4 (circle 1 2 3))"));
        assert!(!is_package_shaped_authoring("(markup @heading(Hi)\n)"));
    }

    #[test]
    fn list_content_and_multipage_list() {
        let src = r#"(import graphics/page only a4 page)
(import graphics/shapes only circle fill rect stroke)
(import graphics/color only black red)
(val main
  (list
    (page a4 (list (fill (circle 10 20 5) black) (stroke (rect 1 2 3 4) 1 red)))
    (page a4 (fill (circle 0 0 1) black))))
"#;
        let root = parse_root(src).unwrap();
        let p0 = find_package_page(&root, 0).unwrap();
        let kids = package_page_content_nodes(&p0).unwrap();
        assert_eq!(kids.len(), 2);
        assert!(is_headed(&kids[0], "fill"));
        assert!(is_headed(&kids[1], "stroke"));
        let p1 = find_package_page(&root, 1).unwrap();
        assert!(is_headed(&p1, "page"));
        assert!(find_package_page(&root, 2).is_err());
    }

    #[test]
    fn pages_wrapper_locates_inner_list() {
        let src = r#"(import graphics/page only a4 page pages)
(import graphics/shapes only circle fill)
(import graphics/color only black)
(val main (pages (list (page a4 (fill (circle 1 2 3) black)))))
"#;
        let page = find_package_page_in_source(src, 0).unwrap();
        assert!(is_headed(&page, "page"));
        let main = find_main_expr_in_source(src).unwrap();
        assert!(is_headed(&main, "pages"));
    }

    #[test]
    fn missing_main_errors() {
        let root = parse_root("(import graphics/shapes)\n(val other 1)\n").unwrap();
        assert!(find_main_expr(&root).is_err());
        assert!(find_main_expr_in_source("(page a4)").is_err());
    }

    #[test]
    fn collect_and_nudge_pkg_black_circle() {
        let layers = collect_layers_package(PKG_CIRCLE, 0).unwrap();
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].kind, "circle");
        assert!(layers[0].byte_end > layers[0].byte_start);
        let out = nudge_layer_package(PKG_CIRCLE, 0, 0, 2.0, -1.0).unwrap();
        assert!(
            out.contains("(circle 107 147.5 40)"),
            "expected in-place xy nudge, got:\n{out}"
        );
        assert!(out.contains("(fill (circle 107 147.5 40) black)"));
        assert!(out.contains("(import graphics/shapes"));
    }

    #[test]
    fn nudge_zero_delta_is_identity() {
        assert_eq!(
            nudge_layer_package(PKG_CIRCLE, 0, 0, 0.0, 0.0).unwrap(),
            PKG_CIRCLE
        );
    }

    #[test]
    fn nudge_bad_index_errors() {
        assert!(nudge_layer_package(PKG_CIRCLE, 0, 9, 1.0, 0.0).is_err());
    }

    #[test]
    fn size_targets_circle_radius() {
        use crate::sync::geometry::scale_size_target;
        let targets = collect_size_targets_package(PKG_CIRCLE, 0).unwrap();
        assert_eq!(targets, vec![SizeTarget::CircleR(0)]);
        let out = scale_size_target(PKG_CIRCLE, SizeTarget::CircleR(0), 2.0).unwrap();
        assert!(
            out.contains("(circle 105 148.5 80)"),
            "expected radius scale, got:\n{out}"
        );
    }

    #[test]
    fn nudge_rect_ellipse_text_line_and_translate_wrap() {
        let multi = r#"(import graphics/shapes only circle fill rect ellipse text line stroke translate rotate)
(import graphics/page only a4 page)
(import graphics/color only black red)
(val main
  (page a4
    (list
      (fill (rect 10 20 30 40) black)
      (fill (ellipse 50 60 7 8) red)
      (text 1 2 12 "hi")
      (stroke (line 0 0 10 10) 1 black)
      (rotate 15 (fill (circle 0 0 5) black))
      (translate 3 4 (fill (circle 0 0 2) black)))))
"#;
        let layers = collect_layers_package(multi, 0).unwrap();
        assert_eq!(
            layers.iter().map(|l| l.kind.as_str()).collect::<Vec<_>>(),
            vec!["rect", "ellipse", "text", "line", "circle", "circle"]
        );
        let r = nudge_layer_package(multi, 0, 0, 1.0, 2.0).unwrap();
        assert!(r.contains("(rect 11 22 30 40)"), "{r}");
        let e = nudge_layer_package(multi, 0, 1, 1.0, -1.0).unwrap();
        assert!(e.contains("(ellipse 51 59 7 8)"), "{e}");
        let t = nudge_layer_package(multi, 0, 2, 5.0, 0.0).unwrap();
        assert!(t.contains("(text 6 2 12 \"hi\")"), "{t}");
        let line = nudge_layer_package(multi, 0, 3, 1.0, 1.0).unwrap();
        assert!(line.contains("(line 1 1 11 11)"), "{line}");
        let wrapped = nudge_layer_package(multi, 0, 4, 2.0, -3.0).unwrap();
        assert!(
            wrapped.contains("(translate 2 -3 (rotate 15 (fill (circle 0 0 5) black)))"),
            "{wrapped}"
        );
        let tr = nudge_layer_package(multi, 0, 5, 1.0, 1.0).unwrap();
        assert!(
            tr.contains("(translate 4 5 (fill (circle 0 0 2) black))"),
            "{tr}"
        );
    }

    #[test]
    fn multipage_list_nudge_second_page_only() {
        let src = r#"(import graphics/shapes only circle fill)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main
  (list
    (page a4 (fill (circle 10 20 5) black))
    (page a4 (fill (circle 30 40 6) black))))
"#;
        assert_eq!(count_package_pages(src).unwrap(), 2);
        let out = nudge_layer_package(src, 1, 0, 1.0, 1.0).unwrap();
        assert!(out.contains("(circle 10 20 5)"), "page0 unchanged: {out}");
        assert!(out.contains("(circle 31 41 6)"), "page1 nudged: {out}");
    }

    #[test]
    fn insert_delete_reorder_and_text_content_on_list_page() {
        let src = r#"(import graphics/shapes only circle fill text)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main
  (page a4
    (list
      (fill (text 30 260 8 "Reciplexa") black)
      (fill (circle 105 120 25) black))))
"#;
        let (with_rect, idx) =
            insert_layer_package(src, 0, "(fill (circle 10 20 5) black)").unwrap();
        assert_eq!(idx, 2);
        assert!(with_rect.contains("(circle 10 20 5)"));
        let renamed = set_text_content_package(&with_rect, 0, 0, "Edited").unwrap();
        assert!(renamed.contains("\"Edited\""));
        assert!(!renamed.contains("\"Reciplexa\""));
        let reordered = reorder_layer_package(&renamed, 0, 2, 0).unwrap();
        let layers = collect_layers_package(&reordered, 0).unwrap();
        assert_eq!(layers[0].kind, "circle");
        let deleted = delete_layer_package(&reordered, 0, 0).unwrap();
        assert_eq!(collect_layers_package(&deleted, 0).unwrap().len(), 2);
        let dup = duplicate_layer_package(src, 0, 1).unwrap();
        assert_eq!(collect_layers_package(&dup, 0).unwrap().len(), 3);
    }

    #[test]
    fn text_line_group_insert_reorder_delete_keeps_page_form() {
        let src = r#"(import graphics/shapes only circle line text translate fill stroke group)
(import graphics/page only a4 page)
(import graphics/color only black rgb)
(val main
  (page a4
    (group
      (list
        (fill (text 30 260 8 "Reciplexa") black)
        (stroke (line 30 250 180 250) 1 (rgb 0.784 0.157 0.157))
        (translate 105 120
          (fill (circle 0 0 25) (rgb 0.118 0.353 0.706)))))))
"#;
        let layers = collect_layers_package(src, 0).unwrap();
        let (src4, inserted) =
            insert_layer_package(src, 0, "(fill (circle 40 80 10) (rgb 0.2 0.4 0.6))").unwrap();
        let src5 = reorder_layer_package(&src4, 0, inserted, 0).unwrap();
        let src6 = delete_layer_package(&src5, 0, 0).unwrap();
        assert!(
            src6.contains("(page a4"),
            "page form broken after group reorder/delete:\n{src6}"
        );
        assert!(src6.contains("(group"), "group lost:\n{src6}");
        assert!(
            src6.contains("(fill (text"),
            "text fill wrapper lost:\n{src6}"
        );
        assert!(
            src6.contains("(stroke (line"),
            "line stroke wrapper lost:\n{src6}"
        );
        assert!(
            src6.contains("(translate 105 120"),
            "translate wrapper lost:\n{src6}"
        );
        assert_eq!(
            collect_layers_package(&src6, 0).unwrap().len(),
            layers.len()
        );
        assert!(!src6.contains("(circle 40 80 10)"));
    }

    #[test]
    fn insert_wraps_single_shape_content() {
        let (out, idx) =
            insert_layer_package(PKG_CIRCLE, 0, "(fill (circle 1 2 3) black)").unwrap();
        assert_eq!(idx, 1);
        assert!(out.contains("(list "));
        assert!(out.contains("(circle 1 2 3)"));
        assert!(insert_layer_package(PKG_CIRCLE, 0, "not-a-list").is_err());
        assert!(delete_layer_package(PKG_CIRCLE, 0, 9).is_err());
        assert!(set_text_content_package(PKG_CIRCLE, 0, 0, "x").is_err());
    }
}
