//! Package-shaped AST locators for GUI CST sync v2.
//!
//! Finds `(val main …)` and nested `(page …)` forms, treating `fill` / `stroke` /
//! `paint` / `list` as transparent wrappers (matching eval bridge flatten).
//! Locate via CST spans; mutate authoring strings in later units.

use reciplexa_syntax::{SyntaxKind, SyntaxNode};

use super::{is_headed, parse_root, SyncError};
use crate::cst_walk::{list_atoms, Child};

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
/// `CONTENT` may be a single shape or a `(list …)` of shapes.
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
        Child::Node(n) if is_headed(n, "list") => {
            let mut out = Vec::new();
            for item in list_atoms(n).into_iter().skip(1) {
                if let Child::Node(c) = item {
                    out.push(c);
                }
            }
            Ok(out)
        }
        Child::Node(n) => Ok(vec![n.clone()]),
        _ => Err(SyncError::new("page content must be a list form")),
    }
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

/// Parse + [`find_package_page`] convenience.
pub fn find_package_page_in_source(src: &str, page_index: usize) -> Result<SyntaxNode, SyncError> {
    let root = parse_root(src)?;
    find_package_page(&root, page_index)
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
}
