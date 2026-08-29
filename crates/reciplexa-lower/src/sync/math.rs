//! Live-layout math tree CST sync (Step 13 slice 5).
//!
//! `live-layout-demo` mains flatten to per-glyph `GlyphRun`s. The layer pane
//! instead lists authoring math records (fraction / scripts / delimiter / …)
//! so GUI edits rewrite the tree, not leaf ink.

use std::collections::{HashMap, HashSet};

use reciplexa_syntax::{
    decode_string_literal, encode_string_literal, replace_token_text, SyntaxKind, SyntaxNode,
    SyntaxToken,
};

use super::document::{collect_layers_from_flow_page, set_document_layer_text_for};
use super::package::find_main_expr;
use super::{is_headed, list_atoms, parse_root, LayerInfo, SyncError};
use crate::cst_walk::{find_list_covering, Child};

/// `(val main (record (tag "live-layout-demo") …))`.
pub fn is_live_layout_authoring(src: &str) -> bool {
    if !src.contains("live-layout-demo") || !src.contains("(val main") {
        return false;
    }
    let Ok(root) = parse_root(src) else {
        return false;
    };
    let Ok(main) = find_main_expr(&root) else {
        return false;
    };
    record_tag(&main).as_deref() == Some("live-layout-demo")
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
            .ok_or_else(|| SyncError::new("live-layout page val is not (page …)")),
        _ => Err(SyncError::new(
            "live-layout-demo page field must be (page …)",
        )),
    }
}

fn short_label(text: &str) -> String {
    text.chars().take(20).collect()
}

fn math_layer_label(tag: &str, role: &str, glyph: Option<&str>) -> String {
    match (tag, glyph, role.is_empty()) {
        ("math-symbol", Some(g), false) => {
            format!("math-symbol \"{}\" ({role})", short_label(g))
        }
        ("math-symbol", Some(g), true) => format!("math-symbol \"{}\"", short_label(g)),
        (_, _, false) => format!("{tag} ({role})"),
        _ => tag.to_string(),
    }
}

fn push_math_record(
    node: &SyntaxNode,
    role: &str,
    vals: &HashMap<String, SyntaxNode>,
    layers: &mut Vec<LayerInfo>,
    visiting: &mut HashSet<(usize, usize)>,
) {
    let range = node.text_range();
    let key = (usize::from(range.start()), usize::from(range.end()));
    if !visiting.insert(key) {
        return;
    }
    if is_headed(node, "list") {
        for (i, item) in list_atoms(node).iter().skip(1).enumerate() {
            let nested_role = if role.is_empty() {
                format!("[{i}]")
            } else {
                format!("{role}[{i}]")
            };
            walk_math_child(item, &nested_role, vals, layers, visiting);
        }
        return;
    }
    if !is_headed(node, "record") {
        return;
    }
    if let Some(tag) = record_tag(node) {
        if tag.starts_with("math-") {
            let glyph = record_string_field(node, "glyph");
            let (byte_start, byte_end) = if tag == "math-symbol" {
                if let Some(tok) = record_string_token(node, "glyph") {
                    let r = tok.text_range();
                    (usize::from(r.start()), usize::from(r.end()))
                } else {
                    key
                }
            } else {
                key
            };
            layers.push(LayerInfo {
                kind: tag.clone(),
                label: math_layer_label(&tag, role, glyph.as_deref()),
                byte_start,
                byte_end,
                root_start: key.0,
                root_end: key.1,
            });
        }
    }
    for item in list_atoms(node).into_iter().skip(1) {
        let Child::Node(field) = item else { continue };
        let atoms = list_atoms(&field);
        let Some(Child::Token(name)) = atoms.first() else {
            continue;
        };
        if name.kind() != SyntaxKind::Ident {
            continue;
        }
        let fname = name.text();
        if fname == "tag" {
            continue;
        }
        if let Some(val) = atoms.get(1) {
            walk_math_child(val, fname, vals, layers, visiting);
        }
    }
}

fn walk_math_child(
    child: &Child,
    role: &str,
    vals: &HashMap<String, SyntaxNode>,
    layers: &mut Vec<LayerInfo>,
    visiting: &mut HashSet<(usize, usize)>,
) {
    match child {
        Child::Node(n) => push_math_record(n, role, vals, layers, visiting),
        Child::Token(t) if t.kind() == SyntaxKind::Ident => {
            if let Some(expr) = vals.get(t.text()) {
                push_math_record(expr, role, vals, layers, visiting);
            }
        }
        _ => {}
    }
}

/// Document flow layers plus preorder math authoring nodes for a live-layout demo.
pub fn collect_layers_live_layout(
    src: &str,
    page_index: usize,
) -> Result<Vec<LayerInfo>, SyncError> {
    if page_index != 0 {
        return Err(SyncError::new("live-layout-demo is a single composed page"));
    }
    let root = parse_root(src)?;
    let main = find_main_expr(&root)?;
    if record_tag(&main).as_deref() != Some("live-layout-demo") {
        return Err(SyncError::new("val main is not live-layout-demo"));
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
    let mut visiting = HashSet::new();
    if let Some(math) = record_field_child(&main, "math") {
        walk_math_child(&math, "math", &vals, &mut layers, &mut visiting);
    }
    if let Some(inline) = record_field_child(&main, "inline-math") {
        walk_math_child(&inline, "inline-math", &vals, &mut layers, &mut visiting);
    }
    if layers.is_empty() {
        return Err(SyncError::new("live-layout-demo has no editable layers"));
    }
    Ok(layers)
}

fn layer_at(src: &str, page_index: usize, flat_index: usize) -> Result<LayerInfo, SyncError> {
    collect_layers_live_layout(src, page_index)?
        .into_iter()
        .nth(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))
}

fn math_record_for_layer(src: &str, layer: &LayerInfo) -> Result<SyntaxNode, SyncError> {
    let root = parse_root(src)?;
    find_list_covering(&root, layer.root_start, layer.root_end)
        .ok_or_else(|| SyncError::new("math record span missing"))
}

fn glyph_refuse(kind: &str) -> SyncError {
    SyncError::new(format!(
        "`{kind}` has no glyph field; edit a child math-symbol (no ASCII substitution)"
    ))
}

/// Read the `glyph` string on a `math-symbol` live-layout layer.
pub fn math_layer_glyph(
    src: &str,
    page_index: usize,
    flat_index: usize,
) -> Result<String, SyncError> {
    let layer = layer_at(src, page_index, flat_index)?;
    if layer.kind != "math-symbol" {
        return Err(glyph_refuse(&layer.kind));
    }
    let rec = math_record_for_layer(src, &layer)?;
    record_string_field(&rec, "glyph")
        .ok_or_else(|| SyncError::new("math-symbol missing glyph (no ASCII substitution)"))
}

/// Rewrite the `glyph` string on a `math-symbol` live-layout layer.
pub fn set_math_layer_glyph(
    src: &str,
    page_index: usize,
    flat_index: usize,
    glyph: &str,
) -> Result<String, SyncError> {
    let layer = layer_at(src, page_index, flat_index)?;
    if layer.kind != "math-symbol" {
        return Err(glyph_refuse(&layer.kind));
    }
    let rec = math_record_for_layer(src, &layer)?;
    let tok = record_string_token(&rec, "glyph")
        .ok_or_else(|| SyncError::new("math-symbol missing glyph (no ASCII substitution)"))?;
    let escaped = encode_string_literal(glyph);
    let (_, out) = replace_token_text(&tok, &escaped);
    Ok(out)
}

/// Heading / paragraph rewrite on a live-layout document layer.
pub fn set_live_layout_document_text(
    src: &str,
    page_index: usize,
    flat_index: usize,
    text: &str,
) -> Result<String, SyncError> {
    let layer = layer_at(src, page_index, flat_index)?;
    if !layer.kind.starts_with("doc-") {
        return Err(SyncError::new(format!(
            "`{}` is not a document text layer",
            layer.kind
        )));
    }
    set_document_layer_text_for(src, &layer, text)
}

const LIVE_LAYOUT_STACK_REFUSE: &str =
    "live-layout math tree layers are edited via properties (glyph), not layer stack operations";

pub(crate) fn live_layout_stack_refuse() -> SyncError {
    SyncError::new(LIVE_LAYOUT_STACK_REFUSE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        collect_layer_props, collect_layers_authoring, set_layer_prop, PropEditContext, PropValue,
    };

    const PKG_LIVE_MATH: &str = include_str!("../../../../examples/pkg_live_math.rpx");
    const PKG_LIVE_ALIGN: &str = include_str!("../../../../examples/pkg_live_align.rpx");

    fn ctx() -> PropEditContext {
        PropEditContext {
            aabb_mm: (0.0, 0.0, 10.0, 10.0),
            paper_w_mm: 210.0,
            paper_h_mm: 297.0,
        }
    }

    const MATRIX_SRC: &str = r#"(import document/page only
  a4 page flow section heading paragraph
  block-paragraph)

(val title (heading 1 "Matrix"))
(val body (paragraph "cell."))
(val math
  (record
    (tag "math-matrix")
    (kind "matrix")
    (rows
      (list
        (record
          (tag "math-matrix-row")
          (cells
            (list
              (record (tag "math-symbol") (glyph "a") (class "ord"))
              (record (tag "math-symbol") (glyph "b") (class "ord")))))))))

(val main
  (record
    (tag "live-layout-demo")
    (page
      (page a4
        (flow
          (list
            (section title
              (list
                (block-paragraph body)))))))
    (math math)))
"#;

    const PHANTOM_SRC: &str = r#"(import document/page only
  a4 page flow section heading paragraph
  block-paragraph)

(val title (heading 1 "Phantom"))
(val body (paragraph "x"))
(val math
  (record
    (tag "math-phantom")
    (body (record (tag "math-symbol") (glyph "Σ") (class "op")))))

(val main
  (record
    (tag "live-layout-demo")
    (page
      (page a4
        (flow
          (list
            (section title
              (list
                (block-paragraph body)))))))
    (math math)))
"#;

    #[test]
    fn live_math_layers_are_tree_not_glyphruns() {
        assert!(is_live_layout_authoring(PKG_LIVE_MATH));
        let layers = collect_layers_authoring(PKG_LIVE_MATH, 0).unwrap();
        let kinds: Vec<&str> = layers.iter().map(|l| l.kind.as_str()).collect();
        assert_eq!(
            kinds,
            [
                "doc-heading",
                "doc-paragraph",
                "math-delimiter",
                "math-scripts",
                "math-fraction",
                "math-symbol",
                "math-symbol",
                "math-symbol",
            ]
        );
        assert!(layers[2].label.contains("math-delimiter"));
        assert!(layers[5].label.contains("numerator"));
        assert!(layers[6].label.contains("denominator"));
        assert!(layers[7].label.contains("subscript"));
        assert!(layers[5].byte_end > layers[5].byte_start);
        assert!(PKG_LIVE_MATH[layers[5].byte_start..layers[5].byte_end].contains("a"));
    }

    #[test]
    fn numerator_glyph_keeps_fraction_tree() {
        let layers = collect_layers_live_layout(PKG_LIVE_MATH, 0).unwrap();
        let num = layers
            .iter()
            .position(|l| l.kind == "math-symbol" && l.label.contains("numerator"))
            .unwrap();
        let out = set_math_layer_glyph(PKG_LIVE_MATH, 0, num, "z").unwrap();
        assert!(out.contains(r#"(tag "math-fraction")"#));
        assert!(out.contains(r#"(glyph "z")"#));
        assert!(out.contains(r#"(glyph "b")"#));
        assert!(!out.contains(r#"(glyph "a")"#));
        let props = collect_layer_props(&out, 0, num, &ctx()).unwrap();
        let glyph = props.iter().find(|p| p.id == "content.glyph").unwrap();
        assert_eq!(glyph.value, PropValue::Text("z".into()));
    }

    #[test]
    fn structural_node_glyph_edit_refuses_without_ascii() {
        let layers = collect_layers_live_layout(PKG_LIVE_MATH, 0).unwrap();
        let frac = layers
            .iter()
            .position(|l| l.kind == "math-fraction")
            .unwrap();
        let err = set_math_layer_glyph(PKG_LIVE_MATH, 0, frac, "x/y").unwrap_err();
        assert!(err.message.contains("math-fraction"));
        assert!(err.message.contains("ASCII"));
        let err = set_layer_prop(
            PKG_LIVE_MATH,
            0,
            frac,
            "content.glyph",
            &PropValue::Text("x/y".into()),
            &ctx(),
        )
        .unwrap_err();
        assert!(err.message.contains("ASCII"));
    }

    #[test]
    fn align_and_inline_fraction_layers() {
        let layers = collect_layers_live_layout(PKG_LIVE_ALIGN, 0).unwrap();
        let kinds: Vec<&str> = layers.iter().map(|l| l.kind.as_str()).collect();
        assert!(kinds.contains(&"math-aligned"));
        assert!(kinds.iter().filter(|k| **k == "math-align-row").count() == 2);
        assert!(kinds.contains(&"math-fraction"));
        let cell = layers
            .iter()
            .position(|l| l.kind == "math-symbol" && l.label.contains("ccc"))
            .unwrap();
        let out = set_math_layer_glyph(PKG_LIVE_ALIGN, 0, cell, "e").unwrap();
        assert!(out.contains(r#"(tag "math-aligned")"#));
        assert!(out.contains(r#"(glyph "e")"#));
        assert!(out.contains(r#"(number "(2)")"#));
        let inline_q = layers
            .iter()
            .position(|l| l.kind == "math-symbol" && l.label.contains("q"))
            .unwrap();
        let out = set_layer_prop(
            PKG_LIVE_ALIGN,
            0,
            inline_q,
            "content.glyph",
            &PropValue::Text("r".into()),
            &ctx(),
        )
        .unwrap();
        assert!(out.contains(r#"(tag "math-fraction")"#));
        assert!(out.contains(r#"(glyph "r")"#));
    }

    #[test]
    fn matrix_cell_glyph_keeps_row() {
        let layers = collect_layers_live_layout(MATRIX_SRC, 0).unwrap();
        assert!(layers.iter().any(|l| l.kind == "math-matrix"));
        assert!(layers.iter().any(|l| l.kind == "math-matrix-row"));
        let b = layers
            .iter()
            .position(|l| l.kind == "math-symbol" && l.label.contains("\"b\""))
            .unwrap();
        let out = set_math_layer_glyph(MATRIX_SRC, 0, b, "c").unwrap();
        assert!(out.contains(r#"(tag "math-matrix-row")"#));
        assert!(out.contains(r#"(glyph "c")"#));
        assert!(out.contains(r#"(glyph "a")"#));
    }

    #[test]
    fn phantom_is_selectable_but_not_ascii_substituted() {
        let layers = collect_layers_live_layout(PHANTOM_SRC, 0).unwrap();
        let phantom = layers
            .iter()
            .position(|l| l.kind == "math-phantom")
            .unwrap();
        let err = set_math_layer_glyph(PHANTOM_SRC, 0, phantom, "S").unwrap_err();
        assert!(err.message.contains("ASCII"));
        let sigma = layers.iter().position(|l| l.kind == "math-symbol").unwrap();
        let out = set_math_layer_glyph(PHANTOM_SRC, 0, sigma, "Ω").unwrap();
        assert!(out.contains(r#"(tag "math-phantom")"#));
        assert!(out.contains(r#"(glyph "Ω")"#));
    }

    #[test]
    fn live_layout_heading_text_still_editable() {
        let layers = collect_layers_live_layout(PKG_LIVE_MATH, 0).unwrap();
        let heading = layers.iter().position(|l| l.kind == "doc-heading").unwrap();
        let out = set_layer_prop(
            PKG_LIVE_MATH,
            0,
            heading,
            "content.text",
            &PropValue::Text("Edited title".into()),
            &ctx(),
        )
        .unwrap();
        assert!(out.contains("Edited title"));
        assert!(out.contains(r#"(tag "math-delimiter")"#));
    }
}
