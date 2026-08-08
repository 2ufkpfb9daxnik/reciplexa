//! Top-level `(page …)` CRUD and page-structure helpers for CST sync.

use reciplexa_syntax::{SyntaxKind, SyntaxNode};

use super::{extent_with_leading_ws, parse_root, SyncError};
use crate::cst_walk::{list_atoms, Child};

pub fn find_page(root: &SyntaxNode, page_index: usize) -> Result<SyntaxNode, SyncError> {
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

fn page_spans(root: &SyntaxNode) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for form in root.children() {
        if !is_list_headed(&form, "page") {
            continue;
        }
        let r = form.text_range();
        out.push((usize::from(r.start()), usize::from(r.end())));
    }
    out
}

/// Number of top-level `(page …)` forms.
pub fn count_pages(src: &str) -> Result<usize, SyncError> {
    let root = parse_root(src)?;
    Ok(page_spans(&root).len())
}

/// Insert a new empty page after `after_index` (0-based). Use `after_index == -1` style by
/// passing `None` to insert at the start — here we take `isize` with -1 meaning before first.
///
/// `form` defaults conceptually to `(page a4)`; pass a complete `(page …)` list.
pub fn insert_page_after(
    src: &str,
    after_index: Option<usize>,
    form: &str,
) -> Result<(String, usize), SyncError> {
    let form = form.trim();
    if form.is_empty() || !form.starts_with("(page") || !form.ends_with(')') {
        return Err(SyncError::new(
            "insert_page form must be a complete (page …) list",
        ));
    }
    let root = parse_root(src)?;
    let spans = page_spans(&root);
    let new_index = match after_index {
        None => 0,
        Some(i) if i < spans.len() => i + 1,
        Some(_) => return Err(SyncError::new("page index out of range")),
    };
    let insert_at = if spans.is_empty() {
        src.len()
    } else if new_index == 0 {
        spans[0].0
    } else if new_index >= spans.len() {
        spans[spans.len() - 1].1
    } else {
        spans[new_index].0
    };
    let pad_before = if insert_at > 0 && !src[..insert_at].ends_with('\n') {
        "\n"
    } else {
        ""
    };
    let pad_after = if insert_at < src.len() && !src[insert_at..].starts_with('\n') {
        "\n"
    } else {
        ""
    };
    let mut out = String::with_capacity(src.len() + form.len() + 2);
    out.push_str(&src[..insert_at]);
    out.push_str(pad_before);
    out.push_str(form);
    out.push_str(pad_after);
    out.push_str(&src[insert_at..]);
    Ok((out, new_index))
}

/// Delete top-level `(page …)` at `page_index`. Refuses to delete the last page.
pub fn delete_page(src: &str, page_index: usize) -> Result<String, SyncError> {
    let root = parse_root(src)?;
    let spans = page_spans(&root);
    if spans.len() <= 1 {
        return Err(SyncError::new("cannot delete the only page"));
    }
    if page_index >= spans.len() {
        return Err(SyncError::new("page index out of range"));
    }
    let (start, end) = spans[page_index];
    let (cut_start, cut_end) = extent_with_leading_ws(src, start, end);
    let mut out = String::with_capacity(src.len());
    out.push_str(&src[..cut_start]);
    out.push_str(&src[cut_end..]);
    Ok(out)
}

/// Index of the first shape child under `(page …)`.
pub fn page_body_start(items: &[Child]) -> usize {
    if items.len() >= 3
        && matches!(&items[1], Child::Token(t) if t.kind() == SyntaxKind::Number)
        && matches!(&items[2], Child::Token(t) if t.kind() == SyntaxKind::Number)
    {
        3
    } else {
        2
    }
}

fn is_list_headed(node: &SyntaxNode, name: &str) -> bool {
    // Non-list nodes simply fail the Ident-head match (no separate kind guard).
    let items = list_atoms(node);
    matches!(items.first(), Some(Child::Token(t)) if t.kind() == SyntaxKind::Ident && t.text() == name)
}
