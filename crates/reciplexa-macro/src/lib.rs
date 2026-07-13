//! Minimal macro expansion for `.rpx` sources.
//!
//! This is the package/macro seam: expand known heads before typecheck/lower.
//! Full hygienic macros come later; for now we rewrite concrete forms in place
//! while preserving surrounding source text outside the matched span.

#![forbid(unsafe_code)]

use reciplexa_syntax::{
    parse_source, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpandError {
    pub message: String,
}

impl ExpandError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Expand surface macros until a fixed point (bounded iterations).
pub fn expand_source(input: &str) -> Result<String, ExpandError> {
    let mut src = input.to_string();
    for _ in 0..64 {
        let parse = parse_source(&src);
        if !parse.errors.is_empty() {
            return Err(ExpandError::new(format!(
                "parse error: {}",
                parse.errors[0].message
            )));
        }
        match find_color_byte(&parse.root) {
            Some((start, end, r, g, b)) => {
                let replacement = format!(
                    "(rgb {} {} {})",
                    format_frac(r / 255.0),
                    format_frac(g / 255.0),
                    format_frac(b / 255.0)
                );
                src = splice(&src, start, end, &replacement);
            }
            None => return Ok(src),
        }
    }
    Err(ExpandError::new("macro expansion did not converge"))
}

fn format_frac(v: f64) -> String {
    let rounded = (v * 10_000.0).round() / 10_000.0;
    let mut s = format!("{rounded}");
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

fn splice(src: &str, start: usize, end: usize, replacement: &str) -> String {
    let mut out = String::with_capacity(src.len() + replacement.len());
    out.push_str(&src[..start]);
    out.push_str(replacement);
    out.push_str(&src[end..]);
    out
}

fn find_color_byte(root: &SyntaxNode) -> Option<(usize, usize, f64, f64, f64)> {
    for node in root.descendants() {
        if node.kind() != SyntaxKind::List {
            continue;
        }
        let items = list_atoms(&node);
        let Some(Child::Token(head)) = items.first() else {
            continue;
        };
        if head.kind() != SyntaxKind::Ident || head.text() != "color-byte" {
            continue;
        }
        if items.len() != 4 {
            continue;
        }
        let r = number_val(&items[1])?;
        let g = number_val(&items[2])?;
        let b = number_val(&items[3])?;
        if !(0.0..=255.0).contains(&r)
            || !(0.0..=255.0).contains(&g)
            || !(0.0..=255.0).contains(&b)
        {
            continue;
        }
        let range = node.text_range();
        return Some((range.start().into(), range.end().into(), r, g, b));
    }
    None
}

fn number_val(child: &Child) -> Option<f64> {
    match child {
        Child::Token(t) if t.kind() == SyntaxKind::Number => t.text().parse().ok(),
        _ => None,
    }
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

    #[test]
    fn expands_color_byte_to_rgb() {
        let src = "(page a4 (circle 1 2 3 (color-byte 255 0 0)))";
        let out = expand_source(src).unwrap();
        assert!(out.contains("(rgb 1 0 0)"));
        assert!(!out.contains("color-byte"));
        // Surrounding form preserved.
        assert!(out.contains("(page a4 (circle 1 2 3 "));
    }

    #[test]
    fn identity_when_no_macros() {
        let src = "(page a4 (circle 1 2 3 red))";
        assert_eq!(expand_source(src).unwrap(), src);
    }

    #[test]
    fn parse_error_surfaces() {
        assert!(expand_source("(page").is_err());
    }

    #[test]
    fn roundtrip_unparse_still_works_on_expanded() {
        use reciplexa_syntax::unparse;
        let out = expand_source("(circle 0 0 1 (color-byte 0 128 255))").unwrap();
        let root = parse_source(&out).into_result().unwrap();
        assert_eq!(unparse(&root), out);
    }
}
