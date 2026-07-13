//! Map GUI nudges back onto CST numeric leaves (Glisp-style).

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

/// Nudge the first `(translate tx ty …)` form's translation by `(dx, dy)` mm.
///
/// Returns the new source text with trivia preserved. This is the M7 vertical
/// slice: enough to prove GUI → CST → re-preview without a full binding table.
pub fn nudge_first_translate(src: &str, dx: f64, dy: f64) -> Result<String, SyncError> {
    let parse = parse_source(src);
    let root = parse
        .into_result()
        .map_err(|e| SyncError::new(format!("parse error: {}", e[0].message)))?;
    let (tx_tok, ty_tok) = find_first_translate_numbers(&root)
        .ok_or_else(|| SyncError::new("no (translate tx ty …) form found"))?;

    let tx: f64 = tx_tok
        .text()
        .parse()
        .map_err(|_| SyncError::new("bad tx number"))?;
    let ty: f64 = ty_tok
        .text()
        .parse()
        .map_err(|_| SyncError::new("bad ty number"))?;

    let (_, after_tx) = replace_token_text(&tx_tok, &format_drag_number(tx + dx));
    // Ranges shift after first replace — re-find ty in the new tree.
    let root2 = parse_source(&after_tx)
        .into_result()
        .map_err(|e| SyncError::new(format!("reparse: {}", e[0].message)))?;
    let (_, ty_tok2) = find_first_translate_numbers(&root2)
        .ok_or_else(|| SyncError::new("translate lost after tx patch"))?;
    let (_, after_ty) = replace_token_text(&ty_tok2, &format_drag_number(ty + dy));
    Ok(after_ty)
}

fn find_first_translate_numbers(root: &SyntaxNode) -> Option<(SyntaxToken, SyntaxToken)> {
    for node in root.descendants() {
        if node.kind() != SyntaxKind::List {
            continue;
        }
        let items = list_atoms(&node);
        if items.len() >= 3 {
            if let Child::Token(head) = &items[0] {
                if head.kind() == SyntaxKind::Ident && head.text() == "translate" {
                    let tx = match &items[1] {
                        Child::Token(t) if t.kind() == SyntaxKind::Number => t.clone(),
                        _ => continue,
                    };
                    let ty = match &items[2] {
                        Child::Token(t) if t.kind() == SyntaxKind::Number => t.clone(),
                        _ => continue,
                    };
                    return Some((tx, ty));
                }
            }
        }
    }
    None
}

enum Child {
    Token(SyntaxToken),
    #[allow(dead_code)]
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

    // --- defect ---

    #[test]
    fn no_translate_errors() {
        let err = nudge_first_translate("(page a4 (circle 1 2 3))", 1.0, 1.0).unwrap_err();
        assert!(err.message.contains("no (translate"));
    }

    #[test]
    fn parse_error_surfaces() {
        assert!(nudge_first_translate("(translate 1", 1.0, 1.0).is_err());
    }
}
