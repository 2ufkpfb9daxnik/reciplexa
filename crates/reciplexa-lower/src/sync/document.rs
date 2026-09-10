//! Document/page CST sync (Step 9): editable heading / paragraph / columns bindings.
//!
//! Mutates `(val …)` sites referenced from `(flow …)` / `(section …)` blocks.
//! Canvas glyph nudge is out of scope — structure is edited via layer props.

use std::collections::HashMap;

use reciplexa_syntax::{
    decode_string_literal, encode_string_literal, format_drag_number, replace_token_text,
    SyntaxKind, SyntaxNode, SyntaxToken,
};

use super::package::find_package_page;
use super::{is_headed, list_atoms, parse_root, LayerInfo, SyncError};
use crate::cst_walk::Child;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValKind {
    Heading,
    Paragraph,
    ParagraphIndented,
    Columns,
}

/// `(import document/page` + `(val main (page … (flow …)))`.
pub fn is_document_page_authoring(src: &str) -> bool {
    if !src.contains("(import document/page") || !src.contains("(val main") {
        return false;
    }
    let Ok(root) = parse_root(src) else {
        return false;
    };
    let Ok(page) = find_package_page(&root, 0) else {
        return false;
    };
    page_has_flow(&page)
}

fn page_content_slot(page: &SyntaxNode) -> usize {
    if is_headed(page, "page-framed") {
        3
    } else {
        2
    }
}

fn page_has_flow(page: &SyntaxNode) -> bool {
    let items = list_atoms(page);
    let Some(Child::Node(content)) = items.get(page_content_slot(page)) else {
        return false;
    };
    if is_headed(content, "flow") {
        return true;
    }
    if is_headed(content, "list") {
        return list_atoms(content).iter().skip(1).any(
            |c| matches!(c, Child::Node(n) if is_headed(n, "flow") || is_headed(n, "section")),
        );
    }
    false
}

fn ident_at(items: &[Child], idx: usize) -> Option<String> {
    match items.get(idx)? {
        Child::Token(t) if t.kind() == SyntaxKind::Ident => Some(t.text().to_string()),
        _ => None,
    }
}

fn string_token_at(items: &[Child], idx: usize) -> Result<SyntaxToken, SyncError> {
    match items.get(idx) {
        Some(Child::Token(t)) if t.kind() == SyntaxKind::String => Ok(t.clone()),
        _ => Err(SyncError::new("expected string literal")),
    }
}

fn number_token_at(items: &[Child], idx: usize) -> Result<SyntaxToken, SyncError> {
    match items.get(idx) {
        Some(Child::Token(t)) if t.kind() == SyntaxKind::Number => Ok(t.clone()),
        _ => Err(SyncError::new("expected number literal")),
    }
}

fn val_kind(expr: &SyntaxNode) -> Option<ValKind> {
    if is_headed(expr, "heading") {
        Some(ValKind::Heading)
    } else if is_headed(expr, "paragraph-indented") {
        Some(ValKind::ParagraphIndented)
    } else if is_headed(expr, "paragraph") {
        Some(ValKind::Paragraph)
    } else if is_headed(expr, "columns") {
        Some(ValKind::Columns)
    } else {
        None
    }
}

fn collect_val_bindings(root: &SyntaxNode) -> HashMap<String, (SyntaxNode, ValKind)> {
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
        let Some(name) = ident_at(&items, 1) else {
            continue;
        };
        let Some(Child::Node(expr)) = items.get(2) else {
            continue;
        };
        if let Some(kind) = val_kind(expr) {
            out.insert(name, (expr.clone(), kind));
        }
    }
    out
}

fn val_expr_range(root: &SyntaxNode, name: &str) -> Option<(usize, usize)> {
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
        let Some(n) = ident_at(&items, 1) else {
            continue;
        };
        if n != name {
            continue;
        }
        let Child::Node(expr) = items.get(2)? else {
            continue;
        };
        let r = expr.text_range();
        return Some((usize::from(r.start()), usize::from(r.end())));
    }
    None
}

fn layer_from_val(
    name: &str,
    expr: &SyntaxNode,
    kind: ValKind,
    root: &SyntaxNode,
) -> Result<LayerInfo, SyncError> {
    let items = list_atoms(expr);
    let (kind_label, string_idx, label, text_content) = match kind {
        ValKind::Heading => {
            let tok = string_token_at(&items, 2)?;
            let text = decode_string_literal(tok.text()).unwrap_or_default();
            let short: String = text.chars().take(20).collect();
            (
                "doc-heading",
                tok,
                format!("heading \"{short}\""),
                Some(text),
            )
        }
        ValKind::Paragraph => {
            let tok = string_token_at(&items, 1)?;
            let text = decode_string_literal(tok.text()).unwrap_or_default();
            let short: String = text.chars().take(20).collect();
            (
                "doc-paragraph",
                tok,
                format!("paragraph \"{short}\""),
                Some(text),
            )
        }
        ValKind::ParagraphIndented => {
            let tok = string_token_at(&items, 1)?;
            let text = decode_string_literal(tok.text()).unwrap_or_default();
            let short: String = text.chars().take(20).collect();
            (
                "doc-paragraph",
                tok,
                format!("paragraph-indented \"{short}\""),
                Some(text),
            )
        }
        ValKind::Columns => {
            let count_tok = number_token_at(&items, 1)?;
            let count_text = count_tok.text().to_string();
            let gutter = number_token_at(&items, 2)
                .map(|t| t.text().to_string())
                .unwrap_or_else(|_| "?".into());
            (
                "doc-columns",
                count_tok,
                format!("columns {count_text} gutter {gutter}"),
                None,
            )
        }
    };
    let s_range = string_idx.text_range();
    let (root_start, root_end) = val_expr_range(root, name)
        .ok_or_else(|| SyncError::new(format!("val `{name}` span missing")))?;
    let mut layer = LayerInfo::new(
        kind_label,
        label,
        usize::from(s_range.start()),
        usize::from(s_range.end()),
        root_start,
        root_end,
    );
    if let Some(text) = text_content {
        layer = layer.with_text_content(text);
    }
    Ok(layer)
}

fn push_val_layer(
    layers: &mut Vec<LayerInfo>,
    vals: &HashMap<String, (SyntaxNode, ValKind)>,
    root: &SyntaxNode,
    name: &str,
) -> Result<(), SyncError> {
    let Some((expr, kind)) = vals.get(name) else {
        return Err(SyncError::new(format!("unknown val `{name}`")));
    };
    let layer = layer_from_val(name, expr, *kind, root)?;
    if layers
        .iter()
        .any(|l| l.byte_start == layer.byte_start && l.byte_end == layer.byte_end)
    {
        return Ok(());
    }
    layers.push(layer);
    Ok(())
}

fn column_paragraph_names(columns_expr: &SyntaxNode) -> Result<Vec<String>, SyncError> {
    let items = list_atoms(columns_expr);
    let Child::Node(list) = items
        .get(4)
        .ok_or_else(|| SyncError::new("columns missing list"))?
    else {
        return Err(SyncError::new("columns list must be node"));
    };
    if !is_headed(list, "list") {
        return Err(SyncError::new("columns fourth arg must be (list …)"));
    }
    let mut names = Vec::new();
    for item in list_atoms(list).into_iter().skip(1) {
        if let Child::Token(t) = item {
            if t.kind() == SyntaxKind::Ident {
                names.push(t.text().to_string());
            }
        }
    }
    if names.is_empty() {
        return Err(SyncError::new("columns list is empty"));
    }
    Ok(names)
}

fn walk_section_blocks(
    section: &SyntaxNode,
    vals: &HashMap<String, (SyntaxNode, ValKind)>,
    root: &SyntaxNode,
    layers: &mut Vec<LayerInfo>,
) -> Result<(), SyncError> {
    let items = list_atoms(section);
    if !is_headed(section, "section") {
        return Err(SyncError::new("expected (section …)"));
    }
    if let Some(title_name) = ident_at(&items, 1) {
        if vals
            .get(&title_name)
            .is_some_and(|(_, k)| *k == ValKind::Heading)
        {
            push_val_layer(layers, vals, root, &title_name)?;
        }
    }
    let Child::Node(blocks) = items
        .get(2)
        .ok_or_else(|| SyncError::new("section missing blocks"))?
    else {
        return Err(SyncError::new("section blocks must be node"));
    };
    for item in list_atoms(blocks).into_iter().skip(1) {
        let Child::Node(block) = item else { continue };
        let bitems = list_atoms(&block);
        let head = bitems.first().and_then(|c| match c {
            Child::Token(t) if t.kind() == SyntaxKind::Ident => Some(t.text()),
            _ => None,
        });
        match head {
            Some("block-heading") => {
                if let Some(name) = ident_at(&bitems, 1) {
                    push_val_layer(layers, vals, root, &name)?;
                }
            }
            Some("block-paragraph") => {
                if let Some(name) = ident_at(&bitems, 1) {
                    push_val_layer(layers, vals, root, &name)?;
                }
            }
            Some("block-columns") => {
                if let Some(name) = ident_at(&bitems, 1) {
                    if let Some((cols_expr, ValKind::Columns)) = vals.get(&name) {
                        layers.push(layer_from_val(&name, cols_expr, ValKind::Columns, root)?);
                        for col in column_paragraph_names(cols_expr)? {
                            push_val_layer(layers, vals, root, &col)?;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn flow_sections(page: &SyntaxNode) -> Result<Vec<SyntaxNode>, SyncError> {
    let items = list_atoms(page);
    let Child::Node(content) = items
        .get(page_content_slot(page))
        .ok_or_else(|| SyncError::new("page missing content"))?
    else {
        return Err(SyncError::new("page content must be node"));
    };
    let flow = if is_headed(content, "flow") {
        content.clone()
    } else if is_headed(content, "list") {
        list_atoms(content)
            .into_iter()
            .skip(1)
            .find_map(|c| match c {
                Child::Node(n) if is_headed(&n, "flow") => Some(n.clone()),
                _ => None,
            })
            .ok_or_else(|| SyncError::new("page list missing (flow …)"))?
    } else {
        return Err(SyncError::new("page content must be (flow …)"));
    };
    let flist = list_atoms(&flow);
    let Child::Node(list) = flist
        .get(1)
        .ok_or_else(|| SyncError::new("flow missing list"))?
    else {
        return Err(SyncError::new("flow body must be (list …)"));
    };
    let mut sections = Vec::new();
    for item in list_atoms(list).into_iter().skip(1) {
        if let Child::Node(n) = item {
            if is_headed(&n, "section") {
                sections.push(n);
            }
        }
    }
    if sections.is_empty() {
        return Err(SyncError::new("flow has no (section …)"));
    }
    Ok(sections)
}

/// Collect heading / paragraph / columns layers from an explicit `(page … (flow …))`.
pub(crate) fn collect_layers_from_flow_page(
    root: &SyntaxNode,
    page: &SyntaxNode,
) -> Result<Vec<LayerInfo>, SyncError> {
    if !page_has_flow(page) {
        return Err(SyncError::new("page is not a document flow page"));
    }
    let vals = collect_val_bindings(root);
    let mut layers = Vec::new();
    for section in flow_sections(page)? {
        walk_section_blocks(&section, &vals, root, &mut layers)?;
    }
    Ok(layers)
}

/// Editable document structure layers in reading order (heading / paragraph / columns).
pub fn collect_layers_document(src: &str, page_index: usize) -> Result<Vec<LayerInfo>, SyncError> {
    let root = parse_root(src)?;
    let pages = super::package::collect_package_pages(&super::package::find_main_expr(&root)?)?;
    let cst_index = if page_index < pages.len() {
        page_index
    } else {
        0
    };
    let page = pages
        .into_iter()
        .nth(cst_index)
        .ok_or_else(|| SyncError::new(format!("no package page #{page_index}")))?;
    let layers = collect_layers_from_flow_page(&root, &page)?;
    if layers.is_empty() {
        return Err(SyncError::new("no editable document layers"));
    }
    Ok(layers)
}

pub(crate) fn document_layer_binding(
    src: &str,
    layer: &LayerInfo,
) -> Result<(SyntaxNode, ValKind, LayerInfo), SyncError> {
    let root = parse_root(src)?;
    let vals = collect_val_bindings(&root);
    for (name, (expr, kind)) in &vals {
        if let Ok(l) = layer_from_val(name, expr, *kind, &root) {
            if l.byte_start == layer.byte_start && l.byte_end == layer.byte_end {
                return Ok((expr.clone(), *kind, layer.clone()));
            }
        }
    }
    Err(SyncError::new("document layer val binding missing"))
}

fn replace_string_in_expr(
    _src: &str,
    expr: &SyntaxNode,
    string_slot: usize,
    text: &str,
) -> Result<String, SyncError> {
    let items = list_atoms(expr);
    let tok = string_token_at(&items, string_slot)?;
    let escaped = encode_string_literal(text);
    let (_, out) = replace_token_text(&tok, &escaped);
    Ok(out)
}

pub(crate) fn set_document_layer_text_for(
    src: &str,
    layer: &LayerInfo,
    text: &str,
) -> Result<String, SyncError> {
    let (expr, kind, layer) = document_layer_binding(src, layer)?;
    if layer.kind == "doc-columns" {
        return Err(SyncError::new("columns layer has no paragraph text"));
    }
    let slot = match kind {
        ValKind::Heading => 2,
        ValKind::Paragraph | ValKind::ParagraphIndented => 1,
        ValKind::Columns => {
            return Err(SyncError::new("columns layer text edit unsupported"));
        }
    };
    replace_string_in_expr(src, &expr, slot, text)
}

/// Replace heading or paragraph string literal for a document layer.
pub fn set_document_layer_text(
    src: &str,
    page_index: usize,
    flat_index: usize,
    text: &str,
) -> Result<String, SyncError> {
    let layer = collect_layers_document(src, page_index)?
        .into_iter()
        .nth(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    set_document_layer_text_for(src, &layer, text)
}

pub(crate) fn set_document_layer_indent_em_for(
    src: &str,
    layer: &LayerInfo,
    indent_em: f64,
) -> Result<String, SyncError> {
    let (expr, kind, _) = document_layer_binding(src, layer)?;
    if kind != ValKind::ParagraphIndented {
        return Err(SyncError::new("indent-em only on paragraph-indented"));
    }
    let items = list_atoms(&expr);
    let tok = number_token_at(&items, 2)?;
    let formatted = format_drag_number(indent_em);
    let (_, out) = replace_token_text(&tok, &formatted);
    Ok(out)
}

/// Set `indent-em` on a `paragraph-indented` layer.
pub fn set_document_layer_indent_em(
    src: &str,
    page_index: usize,
    flat_index: usize,
    indent_em: f64,
) -> Result<String, SyncError> {
    let layer = collect_layers_document(src, page_index)?
        .into_iter()
        .nth(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    set_document_layer_indent_em_for(src, &layer, indent_em)
}

pub(crate) fn set_document_columns_count_for(
    src: &str,
    layer: &LayerInfo,
    count: u32,
) -> Result<String, SyncError> {
    let (expr, kind, layer) = document_layer_binding(src, layer)?;
    if kind != ValKind::Columns || layer.kind != "doc-columns" {
        return Err(SyncError::new("column count only on doc-columns layer"));
    }
    let items = list_atoms(&expr);
    let tok = number_token_at(&items, 1)?;
    let (_, out) = replace_token_text(&tok, &count.to_string());
    Ok(out)
}

/// Set column count on a `columns` val layer.
pub fn set_document_columns_count(
    src: &str,
    page_index: usize,
    flat_index: usize,
    count: u32,
) -> Result<String, SyncError> {
    let layer = collect_layers_document(src, page_index)?
        .into_iter()
        .nth(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    set_document_columns_count_for(src, &layer, count)
}

pub(crate) fn set_document_columns_gutter_for(
    src: &str,
    layer: &LayerInfo,
    gutter: f64,
) -> Result<String, SyncError> {
    let (expr, kind, layer) = document_layer_binding(src, layer)?;
    if kind != ValKind::Columns || layer.kind != "doc-columns" {
        return Err(SyncError::new("column gutter only on doc-columns layer"));
    }
    let items = list_atoms(&expr);
    let tok = number_token_at(&items, 2)?;
    let formatted = format_drag_number(gutter);
    let (_, out) = replace_token_text(&tok, &formatted);
    Ok(out)
}

/// Set column gutter on a `columns` val layer.
pub fn set_document_columns_gutter(
    src: &str,
    page_index: usize,
    flat_index: usize,
    gutter: f64,
) -> Result<String, SyncError> {
    let layer = collect_layers_document(src, page_index)?
        .into_iter()
        .nth(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    set_document_columns_gutter_for(src, &layer, gutter)
}

pub(crate) fn document_layer_text_for(src: &str, layer: &LayerInfo) -> Result<String, SyncError> {
    let (expr, kind, layer) = document_layer_binding(src, layer)?;
    if layer.kind == "doc-columns" {
        return Err(SyncError::new("columns layer has no text"));
    }
    let slot = match kind {
        ValKind::Heading => 2,
        ValKind::Paragraph | ValKind::ParagraphIndented => 1,
        ValKind::Columns => return Err(SyncError::new("columns layer has no text")),
    };
    let items = list_atoms(&expr);
    let tok = string_token_at(&items, slot)?;
    decode_string_literal(tok.text()).map_err(SyncError::new)
}

/// Read paragraph / heading text for the properties panel.
pub fn document_layer_text(
    src: &str,
    page_index: usize,
    flat_index: usize,
) -> Result<String, SyncError> {
    let layer = collect_layers_document(src, page_index)?
        .into_iter()
        .nth(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    document_layer_text_for(src, &layer)
}

pub(crate) fn document_layer_indent_em_for(
    src: &str,
    layer: &LayerInfo,
) -> Result<Option<f64>, SyncError> {
    let (expr, kind, _) = document_layer_binding(src, layer)?;
    if kind != ValKind::ParagraphIndented {
        return Ok(None);
    }
    let items = list_atoms(&expr);
    let tok = number_token_at(&items, 2)?;
    tok.text()
        .parse()
        .map(Some)
        .map_err(|_| SyncError::new("indent-em parse"))
}

pub fn document_layer_indent_em(
    src: &str,
    page_index: usize,
    flat_index: usize,
) -> Result<Option<f64>, SyncError> {
    let layer = collect_layers_document(src, page_index)?
        .into_iter()
        .nth(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    document_layer_indent_em_for(src, &layer)
}

pub(crate) fn document_columns_params_for(
    src: &str,
    layer: &LayerInfo,
) -> Result<(u32, f64), SyncError> {
    let (expr, kind, layer) = document_layer_binding(src, layer)?;
    if kind != ValKind::Columns || layer.kind != "doc-columns" {
        return Err(SyncError::new("columns params only on doc-columns layer"));
    }
    let items = list_atoms(&expr);
    let count: u32 = number_token_at(&items, 1)?
        .text()
        .parse()
        .map_err(|_| SyncError::new("columns count parse"))?;
    let gutter: f64 = number_token_at(&items, 2)?
        .text()
        .parse()
        .map_err(|_| SyncError::new("columns gutter parse"))?;
    Ok((count, gutter))
}

pub fn document_columns_params(
    src: &str,
    page_index: usize,
    flat_index: usize,
) -> Result<(u32, f64), SyncError> {
    let layer = collect_layers_document(src, page_index)?
        .into_iter()
        .nth(flat_index)
        .ok_or_else(|| SyncError::new("layer index out of range"))?;
    document_columns_params_for(src, &layer)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PKG_INDENT: &str = include_str!("../../../../examples/pkg_document_indent.rpx");
    const PKG_COLUMNS: &str = include_str!("../../../../examples/pkg_columns.rpx");
    const PKG_CIRCLE: &str = r#"(import graphics/shapes only circle fill)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main (page a4 (fill (circle 105 148.5 40) black)))
"#;

    #[test]
    fn detects_document_page_not_graphics() {
        assert!(is_document_page_authoring(PKG_INDENT));
        assert!(is_document_page_authoring(PKG_COLUMNS));
        assert!(!is_document_page_authoring(PKG_CIRCLE));
    }

    #[test]
    fn collects_indent_page_layers() {
        let layers = collect_layers_document(PKG_INDENT, 0).unwrap();
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0].kind, "doc-heading");
        assert_eq!(layers[1].kind, "doc-paragraph");
        assert!(layers[0].byte_start > 0);
        assert!(PKG_INDENT[layers[0].byte_start..layers[0].byte_end].contains("字下げ"));
    }

    #[test]
    fn collects_columns_page_layers() {
        let layers = collect_layers_document(PKG_COLUMNS, 0).unwrap();
        assert_eq!(layers.len(), 4, "{layers:?}");
        assert_eq!(layers[0].kind, "doc-heading");
        assert_eq!(layers[1].kind, "doc-columns");
        assert!(layers[2].kind == "doc-paragraph" && layers[2].label.contains("左"));
        assert!(layers[3].kind == "doc-paragraph" && layers[3].label.contains("右"));
    }

    #[test]
    fn edits_column_paragraphs() {
        let layers = collect_layers_document(PKG_COLUMNS, 0).unwrap();
        let left_idx = layers
            .iter()
            .position(|l| l.label.contains("左カラム"))
            .unwrap();
        let out = set_document_layer_text(PKG_COLUMNS, 0, left_idx, "左を編集").unwrap();
        assert!(out.contains("左を編集"));
    }

    #[test]
    fn edits_column_params() {
        let layers = collect_layers_document(PKG_COLUMNS, 0).unwrap();
        let cols_idx = layers.iter().position(|l| l.kind == "doc-columns").unwrap();
        let out = set_document_columns_gutter(PKG_COLUMNS, 0, cols_idx, 2.0).unwrap();
        assert!(out.contains("(columns 2 2 21"));
        let out2 = set_document_columns_count(&out, 0, cols_idx, 3).unwrap();
        assert!(out2.contains("(columns 3 2 21"));
    }

    #[test]
    fn edits_paragraph_and_indent_text() {
        let layers = collect_layers_document(PKG_INDENT, 0).unwrap();
        let body_idx = layers
            .iter()
            .position(|l| l.kind == "doc-paragraph")
            .unwrap();
        let out = set_document_layer_text(PKG_INDENT, 0, body_idx, "編集後。").unwrap();
        assert!(out.contains("編集後。"));
        assert!(!out.contains("インデント付き段落。"));
        let out2 = set_document_layer_indent_em(&out, 0, body_idx, 2.0).unwrap();
        assert!(out2.contains("(paragraph-indented \"編集後。\" 2)"));
    }
}
