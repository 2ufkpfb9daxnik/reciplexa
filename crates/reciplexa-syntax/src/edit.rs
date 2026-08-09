//! Lossless CST token text replacement for Glisp-style GUI edits.

use rowan::GreenToken;

use crate::kind::{SyntaxKind, SyntaxNode, SyntaxToken};
use crate::parse::unparse;

/// Replace a token's text and return the new root + unparsed source.
///
/// The rest of the tree (including trivia) is preserved via rowan's green
/// node sharing — this is the mechanical heart of bidirectional sync.
pub fn replace_token_text(token: &SyntaxToken, new_text: &str) -> (SyntaxNode, String) {
    let green = GreenToken::new(rowan::SyntaxKind(token.kind() as u16), new_text);
    let new_root_green = token.replace_with(green);
    let root = SyntaxNode::new_root(new_root_green);
    let src = unparse(&root);
    (root, src)
}

/// Format a finite number the way we want numeric leaves to look after a drag.
///
/// Trims noisy floats (`1.5000` → `1.5`) while keeping integers clean (`10`).
pub fn format_drag_number(value: f64) -> String {
    if !value.is_finite() {
        return "0".into();
    }
    let rounded = (value * 1000.0).round() / 1000.0;
    // Fixed decimals then trim so GUI edits stay compact (`1.500` → `1.5`, `10.000` → `10`).
    // `{:.3}` always emits a decimal point for finite values.
    let mut s = format!("{rounded:.3}");
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    s
}

/// Find the first token covering `byte_offset` with the given kind.
pub fn token_at_offset(
    root: &SyntaxNode,
    byte_offset: usize,
    kind: SyntaxKind,
) -> Option<SyntaxToken> {
    root.descendants_with_tokens().find_map(|el| {
        let t = el.into_token()?;
        if t.kind() != kind {
            return None;
        }
        let r = t.text_range();
        let start: usize = r.start().into();
        let end: usize = r.end().into();
        if start <= byte_offset && byte_offset < end {
            Some(t)
        } else if byte_offset == end && start < end {
            // allow caret at end of token
            Some(t)
        } else {
            None
        }
    })
}
