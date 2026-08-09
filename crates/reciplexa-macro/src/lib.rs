//! Minimal macro expansion for `.rpx` sources.
//!
//! This is the package/macro seam: expand known heads before typecheck/lower.
//! Full hygienic macros come later; for now we rewrite concrete forms in place
//! while preserving surrounding source text outside the matched span.

#![forbid(unsafe_code)]

pub mod doc_layout;

use reciplexa_syntax::{parse_source, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

use doc_layout::layout_markup_parts;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpandError {
    pub message: String,
}

impl ExpandError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Expand surface macros until a fixed point.
///
/// Known surface macros always rewrite to forms without the same heads, so the
/// loop is guaranteed to terminate without an artificial iteration cap.
pub fn expand_source(input: &str) -> Result<String, ExpandError> {
    let mut src = input.to_string();
    loop {
        let parse = parse_source(&src);
        if !parse.errors.is_empty() {
            return Err(ExpandError::new(format!(
                "parse error: {}",
                parse.errors[0].message
            )));
        }
        let Some((start, end, replacement)) = find_next_rewrite(&parse.root) else {
            return Ok(src);
        };
        src = splice(&src, start, end, &replacement);
    }
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
    if let Some((start, end, replacement)) = find_rule(root) {
        return Some((start, end, replacement));
    }
    if let Some((start, end, replacement)) = find_square(root) {
        return Some((start, end, replacement));
    }
    if let Some((start, end, replacement)) = find_markup(root) {
        return Some((start, end, replacement));
    }
    None
}

pub fn format_frac(v: f64) -> String {
    let rounded = (v * 10_000.0).round() / 10_000.0;
    let mut s = format!("{rounded:.4}");
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
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
        let a = atom_text(&items[1]);
        let b = atom_text(&items[2]);
        let c = atom_text(&items[3]);
        let mut repl = if horizontal {
            // a=x1 b=x2 c=y
            format!("(line {a} {c} {b} {c}")
        } else {
            // a=y1 b=y2 c=x
            format!("(line {c} {a} {c} {b}")
        };
        for item in items.iter().skip(4) {
            repl.push(' ');
            repl.push_str(&atom_text(item));
        }
        repl.push(')');
        let range = node.text_range();
        return Some((range.start().into(), range.end().into(), repl));
    }
    None
}

/// `(square x y size [fill…])` → `(rect x y size size …)`.
fn find_square(root: &SyntaxNode) -> Option<(usize, usize, String)> {
    for node in root.descendants() {
        if node.kind() != SyntaxKind::List {
            continue;
        }
        let items = list_atoms(&node);
        let Some(Child::Token(head)) = items.first() else {
            continue;
        };
        if head.kind() != SyntaxKind::Ident || head.text() != "square" {
            continue;
        }
        if items.len() < 4 {
            continue;
        }
        let x = atom_text(&items[1]);
        let y = atom_text(&items[2]);
        let size = atom_text(&items[3]);
        let mut repl = format!("(rect {x} {y} {size} {size}");
        for item in items.iter().skip(4) {
            repl.push(' ');
            repl.push_str(&atom_text(item));
        }
        repl.push(')');
        let range = node.text_range();
        return Some((range.start().into(), range.end().into(), repl));
    }
    None
}

/// `(rule y [color [width]])` → full-width A4 horizontal rule with 20mm side margins.
fn find_rule(root: &SyntaxNode) -> Option<(usize, usize, String)> {
    for node in root.descendants() {
        if node.kind() != SyntaxKind::List {
            continue;
        }
        let items = list_atoms(&node);
        let Some(Child::Token(head)) = items.first() else {
            continue;
        };
        if head.kind() != SyntaxKind::Ident || head.text() != "rule" {
            continue;
        }
        if items.len() < 2 {
            continue;
        }
        let y = atom_text(&items[1]);
        let mut repl = format!("(hline 20 190 {y}");
        for item in items.iter().skip(2) {
            repl.push(' ');
            repl.push_str(&atom_text(item));
        }
        repl.push(')');
        let range = node.text_range();
        return Some((range.start().into(), range.end().into(), repl));
    }
    None
}

fn find_markup(root: &SyntaxNode) -> Option<(usize, usize, String)> {
    use reciplexa_syntax::markup_parts;

    use crate::doc_layout::{place_items, DocFrame, PlacedItem};

    for child in root.children() {
        if child.kind() != SyntaxKind::List {
            continue;
        }
        let items = list_atoms(&child);
        let Some(Child::Token(head)) = items.first() else {
            continue;
        };
        if head.kind() != SyntaxKind::Ident || head.text() != "markup" {
            continue;
        }
        let parts = markup_parts(&child).ok()?;
        let laid = layout_markup_parts(&parts);
        let range = child.text_range();
        let start: usize = range.start().into();
        let end: usize = range.end().into();
        let replacement = if laid.is_empty() {
            "(page a4)".to_string()
        } else {
            let placed = place_items(&laid, DocFrame::A4);
            let page_count = placed
                .iter()
                .map(PlacedItem::page_index)
                .max()
                .map(|p| p + 1)
                .unwrap_or(1);
            let mut repl = String::new();
            for page in 0..page_count {
                if page > 0 {
                    repl.push('\n');
                }
                repl.push_str("(page a4");
                for p in placed.iter().filter(|p| p.page_index() == page) {
                    match p {
                        PlacedItem::Text(t) => {
                            repl.push_str(&format!(
                                " (text {} {} {} \"{}\" black)",
                                format_frac(t.x_mm),
                                format_frac(t.y_mm),
                                format_frac(t.size_mm),
                                escape_lisp_string(&t.content)
                            ));
                        }
                        PlacedItem::Line {
                            x1_mm,
                            y1_mm,
                            x2_mm,
                            y2_mm,
                            width_mm,
                            ..
                        } => {
                            repl.push_str(&format!(
                                " (line {} {} {} {} black {})",
                                format_frac(*x1_mm),
                                format_frac(*y1_mm),
                                format_frac(*x2_mm),
                                format_frac(*y2_mm),
                                format_frac(*width_mm)
                            ));
                        }
                        PlacedItem::Image {
                            path,
                            x_mm,
                            y_mm,
                            width_mm,
                            height_mm,
                            ..
                        } => {
                            repl.push_str(&format!(
                                " (image \"{}\" {} {} {} {})",
                                escape_lisp_string(path),
                                format_frac(*x_mm),
                                format_frac(*y_mm),
                                format_frac(*width_mm),
                                format_frac(*height_mm)
                            ));
                        }
                    }
                }
                repl.push(')');
            }
            repl
        };
        return Some((start, end, replacement));
    }
    None
}

pub fn escape_lisp_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

fn atom_text(child: &Child) -> String {
    match child {
        Child::Token(t) => t.text().to_string(),
        Child::Node(n) => n.to_string(),
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
