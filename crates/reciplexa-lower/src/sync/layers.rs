//! Layer list CRUD and page-root reorder for CST sync.

use reciplexa_syntax::{SyntaxKind, SyntaxNode};

use super::pages::{find_page, page_body_start};
use super::{extent_with_leading_ws, is_headed, parse_root, LayerInfo, SyncError};
use crate::cst_walk::{find_list_covering, list_atoms, Child};

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

/// Wrap contiguous page-level roots that own `flat_indices` in `(group …)`.
///
/// Returns the rewritten source and the flatten indices of every leaf now inside
/// the new group (same relative order).
pub fn group_layers_page(
    src: &str,
    page_index: usize,
    flat_indices: &[usize],
) -> Result<(String, Vec<usize>), SyncError> {
    let mut indices: Vec<usize> = flat_indices.to_vec();
    indices.sort_unstable();
    indices.dedup();
    if indices.len() < 2 {
        return Err(SyncError::new("group needs at least two layers"));
    }
    let layers = collect_layers_page(src, page_index)?;
    let mut roots: Vec<(usize, usize)> = Vec::new();
    for &i in &indices {
        let layer = layers
            .get(i)
            .ok_or_else(|| SyncError::new("layer index out of range"))?;
        let root = (layer.root_start, layer.root_end);
        if roots.last() != Some(&root) {
            roots.push(root);
        }
    }
    if roots.len() < 2 {
        return Err(SyncError::new(
            "group needs layers from at least two page-level forms",
        ));
    }

    let root = parse_root(src)?;
    let page = find_page(&root, page_index)?;
    let items = list_atoms(&page);
    let start = page_body_start(&items);
    let forms: Vec<(usize, usize)> = items
        .iter()
        .skip(start)
        .filter_map(|item| match item {
            Child::Node(n) => {
                let r = n.text_range();
                Some((usize::from(r.start()), usize::from(r.end())))
            }
            _ => None,
        })
        .collect();

    let mut form_idxs = Vec::with_capacity(roots.len());
    for r in &roots {
        let fi = forms
            .iter()
            .position(|f| f == r)
            .ok_or_else(|| SyncError::new("selected layer is not a top-level page root"))?;
        form_idxs.push(fi);
    }
    form_idxs.sort_unstable();
    form_idxs.dedup();
    for w in form_idxs.windows(2) {
        if w[1] != w[0] + 1 {
            return Err(SyncError::new(
                "group requires contiguous layer roots in source order",
            ));
        }
    }
    let first = form_idxs[0];
    let last = *form_idxs.last().unwrap();
    let span_start = forms[first].0;
    let span_end = forms[last].1;

    let mut body = String::from("(group");
    for form in forms.iter().take(last + 1).skip(first) {
        let (a, b) = *form;
        body.push_str("\n    ");
        body.push_str(src[a..b].trim());
    }
    body.push_str("\n  )");

    let mut out = String::with_capacity(src.len() + body.len());
    out.push_str(&src[..span_start]);
    out.push_str(&body);
    out.push_str(&src[span_end..]);

    let mut before = 0usize;
    for f in &forms[..first] {
        before += layers
            .iter()
            .filter(|l| (l.root_start, l.root_end) == *f)
            .count();
    }
    let mut count = 0usize;
    for form in forms.iter().take(last + 1).skip(first) {
        count += layers
            .iter()
            .filter(|l| (l.root_start, l.root_end) == *form)
            .count();
    }
    let new_sel: Vec<usize> = (before..before + count).collect();
    Ok((out, new_sel))
}

/// Peel a top-level `(group …)` owning `flat_index` into sibling page forms.
pub fn ungroup_layer_page(
    src: &str,
    page_index: usize,
    flat_index: usize,
) -> Result<(String, Vec<usize>), SyncError> {
    let layers = collect_layers_page(src, page_index)?;
    let layer = layers
        .get(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    let root_span = (layer.root_start, layer.root_end);
    let root = parse_root(src)?;
    let Some(node) = find_list_covering(&root, root_span.0, root_span.1) else {
        return Err(SyncError::new("group root node not found"));
    };
    if !is_headed(&node, "group") {
        return Err(SyncError::new("layer root is not a group"));
    }
    let items = list_atoms(&node);
    let mut children = Vec::new();
    for item in items.iter().skip(1) {
        if let Child::Node(n) = item {
            let r = n.text_range();
            children.push(src[usize::from(r.start())..usize::from(r.end())].to_string());
        }
    }
    if children.is_empty() {
        return Err(SyncError::new("group has no children"));
    }
    let replacement = children.join("\n  ");
    let mut out = String::with_capacity(src.len() + replacement.len());
    out.push_str(&src[..root_span.0]);
    out.push_str(&replacement);
    out.push_str(&src[root_span.1..]);

    let start = layers
        .iter()
        .position(|l| (l.root_start, l.root_end) == root_span)
        .unwrap_or(flat_index);
    let count = layers
        .iter()
        .filter(|l| (l.root_start, l.root_end) == root_span)
        .count();
    let new_sel: Vec<usize> = (start..start + count).collect();
    Ok((out, new_sel))
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
