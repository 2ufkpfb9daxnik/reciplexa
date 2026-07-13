//! Minimal macro expansion for `.rpx` sources.
//!
//! This is the package/macro seam: expand known heads before typecheck/lower.
//! Full hygienic macros come later; for now we rewrite concrete forms in place
//! while preserving surrounding source text outside the matched span.

#![forbid(unsafe_code)]

use reciplexa_syntax::{parse_source, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

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
        match find_next_rewrite(&parse.root) {
            Some((start, end, replacement)) => {
                src = splice(&src, start, end, &replacement);
            }
            None => return Ok(src),
        }
    }
    Err(ExpandError::new("macro expansion did not converge"))
}

fn find_next_rewrite(root: &SyntaxNode) -> Option<(usize, usize, String)> {
    if let Some((start, end, r, g, b)) = find_color_byte(root) {
        let replacement = format!(
            "(rgb {} {} {})",
            format_frac(r / 255.0),
            format_frac(g / 255.0),
            format_frac(b / 255.0)
        );
        return Some((start, end, replacement));
    }
    if let Some((start, end, g)) = find_gray(root) {
        let c = format_frac(g);
        return Some((start, end, format!("(rgb {c} {c} {c})")));
    }
    if let Some((start, end, replacement)) = find_hline(root) {
        return Some((start, end, replacement));
    }
    if let Some((start, end, replacement)) = find_vline(root) {
        return Some((start, end, replacement));
    }
    None
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
        if !(0.0..=255.0).contains(&r) || !(0.0..=255.0).contains(&g) || !(0.0..=255.0).contains(&b)
        {
            continue;
        }
        let range = node.text_range();
        return Some((range.start().into(), range.end().into(), r, g, b));
    }
    None
}

fn find_hline(root: &SyntaxNode) -> Option<(usize, usize, String)> {
    // (hline x1 x2 y [color [width]]) → (line x1 y x2 y …)
    find_axis_line(root, "hline", true)
}

fn find_vline(root: &SyntaxNode) -> Option<(usize, usize, String)> {
    // (vline y1 y2 x [color [width]]) → (line x y1 x y2 …)
    find_axis_line(root, "vline", false)
}

fn find_axis_line(
    root: &SyntaxNode,
    head_name: &str,
    horizontal: bool,
) -> Option<(usize, usize, String)> {
    for node in root.descendants() {
        if node.kind() != SyntaxKind::List {
            continue;
        }
        let items = list_atoms(&node);
        let Some(Child::Token(head)) = items.first() else {
            continue;
        };
        if head.kind() != SyntaxKind::Ident || head.text() != head_name {
            continue;
        }
        if items.len() < 4 {
            continue;
        }
        let a = atom_text(&items[1])?;
        let b = atom_text(&items[2])?;
        let c = atom_text(&items[3])?;
        let mut repl = if horizontal {
            // a=x1 b=x2 c=y
            format!("(line {a} {c} {b} {c}")
        } else {
            // a=y1 b=y2 c=x
            format!("(line {c} {a} {c} {b}")
        };
        for item in items.iter().skip(4) {
            repl.push(' ');
            repl.push_str(&atom_text(item)?);
        }
        repl.push(')');
        let range = node.text_range();
        return Some((range.start().into(), range.end().into(), repl));
    }
    None
}

fn atom_text(child: &Child) -> Option<String> {
    match child {
        Child::Token(t) => Some(t.text().to_string()),
        Child::Node(n) => Some(n.to_string()),
    }
}

fn find_gray(root: &SyntaxNode) -> Option<(usize, usize, f64)> {
    for node in root.descendants() {
        if node.kind() != SyntaxKind::List {
            continue;
        }
        let items = list_atoms(&node);
        let Some(Child::Token(head)) = items.first() else {
            continue;
        };
        if head.kind() != SyntaxKind::Ident || head.text() != "gray" {
            continue;
        }
        if items.len() != 2 {
            continue;
        }
        let g = number_val(&items[1])?;
        if !(0.0..=1.0).contains(&g) {
            continue;
        }
        let range = node.text_range();
        return Some((range.start().into(), range.end().into(), g));
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
    fn expands_hline_to_line() {
        let src = "(page a4 (hline 10 100 50 red 1.5))";
        let out = expand_source(src).unwrap();
        assert!(out.contains("(line 10 50 100 50 red 1.5)"));
        assert!(!out.contains("hline"));
    }

    #[test]
    fn expands_vline_to_line() {
        let src = "(page a4 (vline 10 100 50 blue))";
        let out = expand_source(src).unwrap();
        assert!(out.contains("(line 50 10 50 100 blue)"));
        assert!(!out.contains("vline"));
    }

    #[test]
    fn expands_gray_to_rgb() {
        let src = "(page a4 (circle 1 2 3 (gray 0.25)))";
        let out = expand_source(src).unwrap();
        assert!(out.contains("(rgb 0.25 0.25 0.25)"));
        assert!(!out.contains("gray"));
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
