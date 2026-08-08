//! Shared private CST walk helpers for sync/props (trivia-skipping list atoms).

use reciplexa_syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

pub(crate) enum Child {
    Token(SyntaxToken),
    Node(SyntaxNode),
}

pub(crate) fn list_atoms(node: &SyntaxNode) -> Vec<Child> {
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

pub(crate) fn find_list_covering(
    root: &SyntaxNode,
    start: usize,
    end: usize,
) -> Option<SyntaxNode> {
    root.descendants().find(|n| {
        if n.kind() != SyntaxKind::List {
            return false;
        }
        let r = n.text_range();
        usize::from(r.start()) == start && usize::from(r.end()) == end
    })
}
