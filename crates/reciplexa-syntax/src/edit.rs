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
    let mut s = format!("{rounded:.3}");
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{parse_source, unparse};

    // --- validity ---

    #[test]
    fn replace_number_preserves_surrounding_trivia() {
        let src = "(circle  10  20\n  30)";
        let root = parse_source(src).into_result().unwrap();
        let num = root
            .descendants_with_tokens()
            .find_map(|el| {
                let t = el.into_token()?;
                (t.kind() == SyntaxKind::Number && t.text() == "10").then_some(t)
            })
            .unwrap();
        let (new_root, out) = replace_token_text(&num, "99");
        assert_eq!(out, "(circle  99  20\n  30)");
        assert_eq!(unparse(&new_root), out);
    }

    #[test]
    fn format_drag_number_trims() {
        assert_eq!(format_drag_number(10.0), "10");
        assert_eq!(format_drag_number(1.5), "1.5");
        assert_eq!(format_drag_number(1.500_000_1), "1.5");
        assert_eq!(format_drag_number(1.25), "1.25");
        assert_eq!(format_drag_number(1.2), "1.2");
    }

    // --- defect ---

    #[test]
    fn format_non_finite_becomes_zero() {
        assert_eq!(format_drag_number(f64::NAN), "0");
        assert_eq!(format_drag_number(f64::INFINITY), "0");
    }

    #[test]
    fn token_at_offset_misses_gaps() {
        let root = parse_source("(a 1)").into_result().unwrap();
        // whitespace between a and 1
        assert!(token_at_offset(&root, 2, SyntaxKind::Number).is_none());
    }

    #[test]
    fn token_at_offset_hits_interior_and_end() {
        let root = parse_source("(a 12)").into_result().unwrap();
        let interior = token_at_offset(&root, 4, SyntaxKind::Number).unwrap();
        assert_eq!(interior.text(), "12");
        // caret at end of the number token
        let at_end = token_at_offset(&root, 5, SyntaxKind::Number).unwrap();
        assert_eq!(at_end.text(), "12");
    }
}
