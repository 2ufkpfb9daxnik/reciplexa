//! Minimal macro expansion for `.rpx` sources.
//!
//! This is the package/macro seam: expand known heads before typecheck/lower.
//! Full hygienic macros come later; for now we rewrite concrete forms in place
//! while preserving surrounding source text outside the matched span.

#![forbid(unsafe_code)]

mod doc_layout;

use reciplexa_syntax::{parse_source, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

use doc_layout::layout_doc_parts;

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
    if let Some((start, end, replacement)) = find_rule(root) {
        return Some((start, end, replacement));
    }
    if let Some((start, end, replacement)) = find_doc(root) {
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
        let y = atom_text(&items[1])?;
        let mut repl = format!("(hline 20 190 {y}");
        for item in items.iter().skip(2) {
            repl.push(' ');
            repl.push_str(&atom_text(item)?);
        }
        repl.push(')');
        let range = node.text_range();
        return Some((range.start().into(), range.end().into(), repl));
    }
    None
}

fn find_doc(root: &SyntaxNode) -> Option<(usize, usize, String)> {
    use reciplexa_syntax::doc_parts;

    use crate::doc_layout::{place_items, DocFrame, PlacedItem};

    for child in root.children() {
        if child.kind() != SyntaxKind::List {
            continue;
        }
        let items = list_atoms(&child);
        let Some(Child::Token(head)) = items.first() else {
            continue;
        };
        if head.kind() != SyntaxKind::Ident || head.text() != "doc" {
            continue;
        }
        let parts = doc_parts(&child).ok()?;
        let laid = layout_doc_parts(&parts);
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

fn escape_lisp_string(s: &str) -> String {
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
    fn expands_rule_to_hline() {
        let src = "(page a4 (rule 200 (gray 0.3) 0.5))";
        let out = expand_source(src).unwrap();
        // gray expands first, then rule → hline → line
        assert!(out.contains("(line 20 "));
        assert!(out.contains(" 190 "));
        assert!(!out.contains("rule"));
        assert!(!out.contains("hline"));
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

    #[test]
    fn expands_plain_doc_to_page_text() {
        let out = expand_source("(doc Hello.)").unwrap();
        assert!(out.contains("(page a4 (text 25 270 8 \"Hello.\" black))"));
        assert!(!out.contains("(doc "));
    }

    #[test]
    fn expands_doc_with_at_identity() {
        let out = expand_source("(doc Hello @em{世界}.)").unwrap();
        assert!(out.contains("Hello 世界."));
        assert!(out.starts_with("(page a4 (text "));
    }

    #[test]
    fn empty_doc_becomes_empty_page() {
        let out = expand_source("(doc)").unwrap();
        assert_eq!(out, "(page a4)");
    }

    #[test]
    fn expands_multiline_doc_to_stacked_text() {
        let out = expand_source("(doc\nFirst\nSecond\n)").unwrap();
        assert!(out.contains("(text 25 270 8 \"First\" black)"));
        assert!(out.contains("(text 25 258 8 \"Second\" black)"));
    }

    // --- @title / @p package meaning (M8) — validity ---

    #[test]
    fn expands_title_at_larger_size() {
        let out = expand_source("(doc @title{Hello})").unwrap();
        assert!(
            out.contains("(text 25 270 14 \"Hello\" black)"),
            "title should use size 14: {out}"
        );
        assert!(!out.contains("(doc "));
    }

    #[test]
    fn expands_title_then_paragraph_with_gap() {
        let out = expand_source("(doc @title{Report}\n@p{Body text})").unwrap();
        assert!(out.contains("(text 25 270 14 \"Report\" black)"), "{out}");
        // title gap 18 → next baseline at 270 - 18 = 252
        assert!(
            out.contains("(text 25 252 8 \"Body text\" black)"),
            "body after title gap: {out}"
        );
    }

    #[test]
    fn em_still_identity_inside_plain_line() {
        let out = expand_source("(doc Hello @em{世界}.)").unwrap();
        assert!(
            out.contains("(text 25 270 8 \"Hello 世界.\" black)"),
            "{out}"
        );
    }

    // --- defect ---

    #[test]
    fn empty_title_brace_skips_empty_text() {
        let out = expand_source("(doc @title{})").unwrap();
        assert_eq!(
            out, "(page a4)",
            "empty title must not emit empty text: {out}"
        );
    }

    #[test]
    fn bare_title_without_brace_does_not_panic() {
        let out = expand_source("(doc @title)").unwrap();
        assert!(out.starts_with("(page a4)"), "{out}");
        assert!(!out.contains("(text "), "bare @title has no content: {out}");
    }

    #[test]
    fn unknown_at_form_identity_flattens() {
        let out = expand_source("(doc see @foo{bar} end)").unwrap();
        assert!(
            out.contains("(text 25 270 8 \"see bar end\" black)"),
            "{out}"
        );
    }

    #[test]
    fn expands_wrapped_long_paragraph() {
        let long = "word ".repeat(12);
        let long = long.trim();
        assert!(long.len() > 40);
        let src = format!("(doc @p{{{long}}})");
        let out = expand_source(&src).unwrap();
        let text_count = out.matches("(text ").count();
        assert!(
            text_count >= 2,
            "long paragraph should wrap to multiple text shapes: {out}"
        );
    }

    // --- @h2 package meaning ---

    #[test]
    fn expands_h2_between_title_and_body() {
        let out = expand_source("(doc @title{T}\n@h2{Section}\n@p{Body})").unwrap();
        assert!(out.contains("(text 25 270 14 \"T\" black)"), "{out}");
        // title gap 18 → 252; h2 size 11
        assert!(
            out.contains("(text 25 252 11 \"Section\" black)"),
            "h2 after title: {out}"
        );
        // h2 gap 14 → 238
        assert!(
            out.contains("(text 25 238 8 \"Body\" black)"),
            "body after h2: {out}"
        );
    }

    #[test]
    fn empty_h2_skips_empty_text() {
        let out = expand_source("(doc @h2{})").unwrap();
        assert_eq!(out, "(page a4)");
    }

    // --- @li list items ---

    #[test]
    fn expands_li_with_bullet_prefix() {
        let out = expand_source("(doc @li{First}\n@li{Second})").unwrap();
        assert!(
            out.contains("(text 25 270 8 \"• First\" black)"),
            "first li: {out}"
        );
        assert!(
            out.contains("(text 25 258 8 \"• Second\" black)"),
            "second li: {out}"
        );
    }

    #[test]
    fn empty_li_skips_empty_text() {
        let out = expand_source("(doc @li{})").unwrap();
        assert_eq!(out, "(page a4)");
    }

    #[test]
    fn long_doc_spills_onto_second_page() {
        // Body gap 12mm from y=270 down; bottom margin 25 → room for many lines.
        let mut body = String::from("(doc");
        for i in 0..30 {
            body.push_str(&format!("\n@p{{line{i}}}"));
        }
        body.push(')');
        let out = expand_source(&body).unwrap();
        let page_count = out.matches("(page a4").count();
        assert!(
            page_count >= 2,
            "expected multipage layout, got {page_count} page(s): {out}"
        );
    }

    // --- @quote ---

    #[test]
    fn expands_quote_indented_smaller() {
        let out = expand_source("(doc @quote{Cited line})").unwrap();
        // Indented left (35) and size 7.
        assert!(
            out.contains("(text 35 270 7 \"Cited line\" black)"),
            "{out}"
        );
    }

    #[test]
    fn empty_quote_skips() {
        assert_eq!(expand_source("(doc @quote{})").unwrap(), "(page a4)");
    }

    // --- numbered list: @ol{ item; item } uses `;` separators inside one brace ---

    #[test]
    fn expands_ol_numbers_items() {
        let out = expand_source("(doc @ol{Alpha; Beta; Gamma})").unwrap();
        assert!(out.contains("(text 25 270 8 \"1. Alpha\" black)"), "{out}");
        assert!(out.contains("(text 25 258 8 \"2. Beta\" black)"), "{out}");
        assert!(out.contains("(text 25 246 8 \"3. Gamma\" black)"), "{out}");
    }

    #[test]
    fn empty_ol_skips() {
        assert_eq!(expand_source("(doc @ol{})").unwrap(), "(page a4)");
    }

    #[test]
    fn ol_trims_blank_segments() {
        let out = expand_source("(doc @ol{A;; B})").unwrap();
        assert!(out.contains("1. A"), "{out}");
        assert!(out.contains("2. B"), "{out}");
        assert!(!out.contains("3."), "{out}");
    }

    // --- @vspace / @hr ---

    #[test]
    fn expands_vspace_shifts_following_text() {
        let out = expand_source("(doc @p{Above}\n@vspace{20}\n@p{Below})").unwrap();
        assert!(out.contains("(text 25 270 8 \"Above\" black)"), "{out}");
        // body gap 12 after Above, then +20 vspace → Below at 270 - 12 - 20 = 238
        assert!(
            out.contains("(text 25 238 8 \"Below\" black)"),
            "vspace should push following line: {out}"
        );
    }

    #[test]
    fn expands_hr_emits_line_shape() {
        let out = expand_source("(doc @p{Head}\n@hr{}\n@p{Tail})").unwrap();
        assert!(out.contains("(text 25 270 8 \"Head\" black)"), "{out}");
        // After Head (gap 12): y=258 for the rule; A4 content width 25..185
        assert!(
            out.contains("(line 25 258 185 258 black 0.4)"),
            "hr should emit a stroked line: {out}"
        );
        assert!(out.contains("(text 25 246 8 \"Tail\" black)"), "{out}");
    }

    #[test]
    fn bad_vspace_payload_is_skipped() {
        let out = expand_source("(doc @p{A}\n@vspace{nope}\n@p{B})").unwrap();
        // Invalid vspace ignored → normal body gap 12 only.
        assert!(out.contains("(text 25 270 8 \"A\" black)"), "{out}");
        assert!(out.contains("(text 25 258 8 \"B\" black)"), "{out}");
    }

    // --- @pagebreak ---

    #[test]
    fn expands_pagebreak_starts_second_page() {
        let out = expand_source("(doc @p{One}\n@pagebreak{}\n@p{Two})").unwrap();
        assert_eq!(out.matches("(page a4").count(), 2, "{out}");
        assert!(out.contains("(text 25 270 8 \"One\" black)"), "{out}");
        // Second page resets to top margin.
        assert!(
            out.contains("\n(page a4 (text 25 270 8 \"Two\" black))")
                || out.ends_with("(page a4 (text 25 270 8 \"Two\" black))"),
            "Two should start at top of page 2: {out}"
        );
    }

    #[test]
    fn pagebreak_alone_yields_empty_pages_skipped() {
        // Only a break with no drawable items → still one empty page is ok,
        // or empty doc page; we accept a single empty page.
        let out = expand_source("(doc @pagebreak{})").unwrap();
        assert!(out.contains("(page a4)"), "{out}");
    }

    // --- @code ---

    #[test]
    fn expands_code_indented_smaller() {
        let out = expand_source("(doc @code{let x = 1})").unwrap();
        // left 25 + indent 8 = 33; size 6.5
        assert!(
            out.contains("(text 33 270 6.5 \"let x = 1\" black)"),
            "{out}"
        );
    }

    #[test]
    fn empty_code_skips() {
        assert_eq!(expand_source("(doc @code{})").unwrap(), "(page a4)");
    }
}
