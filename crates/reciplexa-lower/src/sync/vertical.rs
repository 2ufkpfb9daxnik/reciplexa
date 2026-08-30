//! `ja-vertical-demo` layer tree and column-origin nudge (Step 14 slice 1).

use reciplexa_syntax::{
    decode_string_literal, format_drag_number, SyntaxKind, SyntaxNode, SyntaxToken,
};

use super::document::collect_layers_from_flow_page;
use super::package::find_main_expr;
use super::{is_headed, parse_root, LayerInfo, SyncError};
use crate::cst_walk::{list_atoms, Child};

use std::collections::HashMap;

/// `(val main (record (tag "ja-vertical-demo") …))`.
pub fn is_vertical_demo_authoring(src: &str) -> bool {
    if !src.contains("ja-vertical-demo") || !src.contains("(val main") {
        return false;
    }
    let Ok(root) = parse_root(src) else {
        return false;
    };
    let Ok(main) = find_main_expr(&root) else {
        return false;
    };
    record_tag(&main).as_deref() == Some("ja-vertical-demo")
}

pub(crate) fn vertical_demo_stack_refuse() -> SyncError {
    SyncError::new(
        "ja-vertical-demo columns are moved as text boxes; restack/insert is not supported",
    )
}

fn collect_val_exprs(root: &SyntaxNode) -> HashMap<String, SyntaxNode> {
    let mut out = HashMap::new();
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
        if name.kind() != SyntaxKind::Ident {
            continue;
        }
        let Some(Child::Node(expr)) = items.get(2) else {
            continue;
        };
        out.insert(name.text().to_string(), expr.clone());
    }
    out
}

fn record_tag(record: &SyntaxNode) -> Option<String> {
    record_string_field(record, "tag")
}

fn record_string_field(record: &SyntaxNode, name: &str) -> Option<String> {
    record_string_token(record, name).and_then(|t| decode_string_literal(t.text()).ok())
}

fn record_string_token(record: &SyntaxNode, name: &str) -> Option<SyntaxToken> {
    for item in list_atoms(record).into_iter().skip(1) {
        let Child::Node(field) = item else { continue };
        let atoms = list_atoms(&field);
        let Some(Child::Token(key)) = atoms.first() else {
            continue;
        };
        if key.kind() != SyntaxKind::Ident || key.text() != name {
            continue;
        }
        match atoms.get(1) {
            Some(Child::Token(t)) if t.kind() == SyntaxKind::String => return Some(t.clone()),
            _ => return None,
        }
    }
    None
}

fn record_field_child(record: &SyntaxNode, name: &str) -> Option<Child> {
    for item in list_atoms(record).into_iter().skip(1) {
        let Child::Node(field) = item else { continue };
        let atoms = list_atoms(&field);
        let Some(Child::Token(key)) = atoms.first() else {
            continue;
        };
        if key.kind() != SyntaxKind::Ident || key.text() != name {
            continue;
        }
        return atoms.get(1).cloned();
    }
    None
}

fn record_field_node(record: &SyntaxNode, name: &str) -> Option<SyntaxNode> {
    for item in list_atoms(record).into_iter().skip(1) {
        let Child::Node(field) = item else { continue };
        let atoms = list_atoms(&field);
        let Some(Child::Token(key)) = atoms.first() else {
            continue;
        };
        if key.kind() != SyntaxKind::Ident || key.text() != name {
            continue;
        }
        return Some(field);
    }
    None
}

fn resolve_page_node(
    child: &Child,
    vals: &HashMap<String, SyntaxNode>,
) -> Result<SyntaxNode, SyncError> {
    match child {
        Child::Node(n) if is_headed(n, "page") => Ok(n.clone()),
        Child::Token(t) if t.kind() == SyntaxKind::Ident => vals
            .get(t.text())
            .filter(|n| is_headed(n, "page"))
            .cloned()
            .ok_or_else(|| SyncError::new("vertical-demo page val is not (page …)")),
        _ => Err(SyncError::new(
            "ja-vertical-demo page field must be (page …)",
        )),
    }
}

fn resolve_node(child: &Child, vals: &HashMap<String, SyntaxNode>) -> Option<SyntaxNode> {
    match child {
        Child::Node(n) => Some(n.clone()),
        Child::Token(t) if t.kind() == SyntaxKind::Ident => vals.get(t.text()).cloned(),
        _ => None,
    }
}

fn list_string_layers(
    list: &SyntaxNode,
    kind: &str,
    label_prefix: &str,
    layers: &mut Vec<LayerInfo>,
) {
    for item in list_atoms(list).into_iter().skip(1) {
        let Child::Token(t) = item else { continue };
        if t.kind() != SyntaxKind::String {
            continue;
        }
        let Ok(text) = decode_string_literal(t.text()) else {
            continue;
        };
        if text.is_empty() {
            continue;
        }
        let short: String = text.chars().take(20).collect();
        let r = t.text_range();
        let list_r = list.text_range();
        layers.push(
            LayerInfo::new(
                kind,
                format!("{label_prefix} \"{short}\""),
                usize::from(r.start()),
                usize::from(r.end()),
                usize::from(list_r.start()),
                usize::from(list_r.end()),
            )
            .with_text_content(text),
        );
    }
}

fn ruby_text_content(ruby: &SyntaxNode) -> Option<(String, usize, usize, usize, usize)> {
    if !is_headed(ruby, "ruby") {
        return None;
    }
    let items = list_atoms(ruby);
    let Child::Token(base) = items.get(1)? else {
        return None;
    };
    let Child::Token(ann) = items.get(2)? else {
        return None;
    };
    if base.kind() != SyntaxKind::String || ann.kind() != SyntaxKind::String {
        return None;
    }
    let base_s = decode_string_literal(base.text()).ok()?;
    let ann_s = decode_string_literal(ann.text()).ok()?;
    // Scene order: annotation column, then base (`positioned_vertical_ruby_to_shapes`).
    let content = format!("{ann_s}{base_s}");
    let r = ruby.text_range();
    Some((
        content,
        usize::from(r.start()),
        usize::from(r.end()),
        usize::from(r.start()),
        usize::from(r.end()),
    ))
}

/// Document flow rows plus vertical-ruby / sample column parents.
pub fn collect_layers_vertical_demo(
    src: &str,
    page_index: usize,
) -> Result<Vec<LayerInfo>, SyncError> {
    if page_index != 0 {
        return Err(SyncError::new("ja-vertical-demo is a single composed page"));
    }
    let root = parse_root(src)?;
    let main = find_main_expr(&root)?;
    if record_tag(&main).as_deref() != Some("ja-vertical-demo") {
        return Err(SyncError::new("val main is not ja-vertical-demo"));
    }
    let vals = collect_val_exprs(&root);
    let mut layers = Vec::new();
    if let Some(page_child) = record_field_child(&main, "page") {
        if let Ok(page) = resolve_page_node(&page_child, &vals) {
            if let Ok(doc_layers) = collect_layers_from_flow_page(&root, &page) {
                layers.extend(doc_layers);
            }
        }
    }
    if let Some(ruby_child) = record_field_child(&main, "vertical-ruby") {
        if let Some(ruby) = resolve_node(&ruby_child, &vals) {
            if let Some((content, b0, b1, r0, r1)) = ruby_text_content(&ruby) {
                let short: String = content.chars().take(20).collect();
                layers.push(
                    LayerInfo::new(
                        "vert-ruby",
                        format!("vertical-ruby \"{short}\""),
                        b0,
                        b1,
                        r0,
                        r1,
                    )
                    .with_text_content(content),
                );
            }
        }
    }
    if let Some(samples_child) = record_field_child(&main, "samples") {
        if let Some(list) = resolve_node(&samples_child, &vals) {
            if is_headed(&list, "list") {
                list_string_layers(&list, "vert-sample", "sample", &mut layers);
            }
        }
    }
    if layers.is_empty() {
        return Err(SyncError::new("ja-vertical-demo has no editable layers"));
    }
    Ok(layers)
}

fn column_slot(layers: &[LayerInfo], layer: &LayerInfo) -> Option<usize> {
    if layer.kind == "vert-ruby" {
        return Some(0);
    }
    if layer.kind != "vert-sample" {
        return None;
    }
    let ruby = layers.iter().any(|l| l.kind == "vert-ruby");
    let i = layers
        .iter()
        .filter(|l| l.kind == "vert-sample")
        .position(|l| l.byte_start == layer.byte_start)?;
    Some(i + usize::from(ruby))
}

fn parse_nudge_pairs(nudge_list: &SyntaxNode) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for item in list_atoms(nudge_list).into_iter().skip(1) {
        let Child::Node(pair) = item else { continue };
        if !is_headed(&pair, "list") {
            continue;
        }
        let atoms = list_atoms(&pair);
        let x = atoms.get(1).and_then(|c| match c {
            Child::Token(t) if t.kind() == SyntaxKind::Number => t.text().parse().ok(),
            _ => None,
        });
        let y = atoms.get(2).and_then(|c| match c {
            Child::Token(t) if t.kind() == SyntaxKind::Number => t.text().parse().ok(),
            _ => None,
        });
        if let (Some(x), Some(y)) = (x, y) {
            out.push((x, y));
        }
    }
    out
}

fn format_nudge_form(pairs: &[(f64, f64)]) -> String {
    let inner: Vec<String> = pairs
        .iter()
        .map(|(x, y)| {
            format!(
                "(list {} {})",
                format_drag_number(*x),
                format_drag_number(*y)
            )
        })
        .collect();
    format!("(nudge (list {}))", inner.join(" "))
}

/// Nudge a vertical-ruby or sample column; document flow rows are refused.
pub fn nudge_vertical_demo_layer(
    src: &str,
    page_index: usize,
    flat_index: usize,
    dx: f64,
    dy: f64,
) -> Result<String, SyncError> {
    if dx == 0.0 && dy == 0.0 {
        return Ok(src.to_string());
    }
    let layers = collect_layers_vertical_demo(src, page_index)?;
    let layer = layers
        .get(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    let slot = column_slot(&layers, layer).ok_or_else(|| {
        SyncError::new(
            "canvas move skipped: document/page structure is edited via the layer properties panel, not glyph positions",
        )
    })?;
    let column_count = layers
        .iter()
        .filter(|l| l.kind == "vert-ruby" || l.kind == "vert-sample")
        .count()
        .max(slot + 1);
    let root = parse_root(src)?;
    let main = find_main_expr(&root)?;
    let mut pairs = match record_field_child(&main, "nudge").and_then(|c| match c {
        Child::Node(n) if is_headed(&n, "list") => Some(n),
        _ => None,
    }) {
        Some(list) => parse_nudge_pairs(&list),
        None => Vec::new(),
    };
    while pairs.len() < column_count {
        pairs.push((0.0, 0.0));
    }
    pairs[slot].0 += dx;
    pairs[slot].1 += dy;
    let form = format_nudge_form(&pairs);
    if let Some(field) = record_field_node(&main, "nudge") {
        let r = field.text_range();
        let (s, e) = (usize::from(r.start()), usize::from(r.end()));
        let mut out = String::with_capacity(src.len() + form.len());
        out.push_str(&src[..s]);
        out.push_str(&form);
        out.push_str(&src[e..]);
        return Ok(out);
    }
    let end = usize::from(main.text_range().end());
    if end == 0 || !src[..end].ends_with(')') {
        return Err(SyncError::new("vertical-demo record missing closer"));
    }
    let mut out = String::with_capacity(src.len() + form.len() + 8);
    out.push_str(&src[..end - 1]);
    out.push_str("\n    ");
    out.push_str(&form);
    out.push(')');
    out.push_str(&src[end..]);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PKG_VERT: &str = include_str!("../../../../examples/pkg_vert.rpx");

    #[test]
    fn vertical_demo_parents_are_not_per_glyph() {
        assert!(is_vertical_demo_authoring(PKG_VERT));
        let layers = collect_layers_vertical_demo(PKG_VERT, 0).unwrap();
        assert!(layers.iter().any(|l| l.kind == "doc-heading"));
        assert!(layers.iter().any(|l| l.kind == "doc-paragraph"));
        assert!(layers.iter().any(|l| l.kind == "vert-ruby"));
        let samples: Vec<_> = layers.iter().filter(|l| l.kind == "vert-sample").collect();
        assert_eq!(samples.len(), 4);
        assert!(samples
            .iter()
            .any(|l| l.text_content.as_deref() == Some("縦書き")));
        assert!(samples
            .iter()
            .any(|l| l.text_content.as_deref() == Some("ABC")));
    }

    #[test]
    fn nudge_sample_writes_nudge_field() {
        let layers = collect_layers_vertical_demo(PKG_VERT, 0).unwrap();
        let idx = layers
            .iter()
            .position(|l| l.kind == "vert-sample" && l.text_content.as_deref() == Some("縦書き"))
            .unwrap();
        let out = nudge_vertical_demo_layer(PKG_VERT, 0, idx, 2.0, -1.0).unwrap();
        assert!(out.contains("(nudge (list"), "{out}");
        assert!(out.contains("縦書き"));
        let out2 = nudge_vertical_demo_layer(&out, 0, idx, 1.0, 0.0).unwrap();
        assert_eq!(out2.matches("(nudge ").count(), 1);
    }
}
