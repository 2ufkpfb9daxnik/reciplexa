//! Minimal CST → scene lowering for the M3 black-circle milestone.
//!
//! Supported forms (Lisp mode only):
//!
//! ```text
//! (page a4
//!   (circle <x-mm> <y-mm> <r-mm>))
//! ```
//!
//! Fill defaults to black. Macros/packages will replace this hard-wired
//! vocabulary later; the seam is intentional.

#![forbid(unsafe_code)]

use reciplexa_scene::{Circle, Color, Document, Page, PaperSize, Shape};
use reciplexa_syntax::{parse_source, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

/// Lowering / validation error (fail-fast: no partial scene for rendering).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LowerError {
    pub message: String,
}

impl LowerError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Parse source and lower it to a [`Document`].
pub fn lower_source(input: &str) -> Result<Document, LowerError> {
    let parse = parse_source(input);
    let root = parse
        .into_result()
        .map_err(|errs| LowerError::new(format!("parse error: {}", errs[0].message)))?;
    lower_syntax(&root)
}

/// Lower an already-parsed CST root (`SourceFile`).
pub fn lower_syntax(root: &SyntaxNode) -> Result<Document, LowerError> {
    if root.kind() != SyntaxKind::SourceFile {
        return Err(LowerError::new(format!(
            "expected SourceFile, got {:?}",
            root.kind()
        )));
    }

    let forms: Vec<SyntaxNode> = root.children().collect();
    if forms.is_empty() {
        return Err(LowerError::new("empty source: expected a (page …) form"));
    }

    let mut pages = Vec::with_capacity(forms.len());
    for form in forms {
        pages.push(lower_page(&form)?);
    }
    Ok(Document { pages })
}

fn lower_page(node: &SyntaxNode) -> Result<Page, LowerError> {
    let items = list_items(node, "page")?;
    let head = ident_at(&items, 0, "page")?;
    if head != "page" {
        return Err(LowerError::new(format!(
            "expected head `page`, found `{head}`"
        )));
    }
    if items.len() < 2 {
        return Err(LowerError::new(
            "`page` requires a paper size, e.g. (page a4 …)",
        ));
    }

    let paper = match atom_ident(&items[1])? {
        "a4" => PaperSize::a4(),
        other => {
            return Err(LowerError::new(format!(
                "unknown paper size `{other}` (only `a4` for now)"
            )))
        }
    };

    let mut shapes = Vec::new();
    for item in items.iter().skip(2) {
        match item {
            Child::Node(n) => shapes.push(lower_shape(n)?),
            Child::Token(t) => {
                return Err(LowerError::new(format!(
                    "page body expected a shape list, got token {:?}",
                    t.kind()
                )))
            }
        }
    }
    Ok(Page { paper, shapes })
}

fn lower_shape(node: &SyntaxNode) -> Result<Shape, LowerError> {
    let items = list_items(node, "shape")?;
    let head = ident_at(&items, 0, "shape")?;
    match head {
        "circle" => lower_circle(&items),
        other => Err(LowerError::new(format!("unknown shape `{other}`"))),
    }
}

fn lower_circle(items: &[Child]) -> Result<Shape, LowerError> {
    if items.len() != 4 {
        return Err(LowerError::new(
            "`circle` expects exactly 3 numbers: x-mm y-mm radius-mm",
        ));
    }
    let x = number_at(items, 1, "circle x")?;
    let y = number_at(items, 2, "circle y")?;
    let r = number_at(items, 3, "circle radius")?;
    let circle = Circle {
        x_mm: x,
        y_mm: y,
        radius_mm: r,
        fill: Color::BLACK,
    };
    if !circle.is_drawable() {
        return Err(LowerError::new(format!(
            "circle is not drawable (radius={r})"
        )));
    }
    Ok(Shape::Circle(circle))
}

#[derive(Debug)]
enum Child {
    Node(SyntaxNode),
    Token(SyntaxToken),
}

fn list_items(node: &SyntaxNode, ctx: &str) -> Result<Vec<Child>, LowerError> {
    if node.kind() != SyntaxKind::List {
        return Err(LowerError::new(format!(
            "{ctx}: expected a (…) list, got {:?}",
            node.kind()
        )));
    }
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
    Ok(items)
}

fn ident_at<'a>(items: &'a [Child], index: usize, ctx: &str) -> Result<&'a str, LowerError> {
    let Some(child) = items.get(index) else {
        return Err(LowerError::new(format!("{ctx}: missing element {index}")));
    };
    match child {
        Child::Token(t) if t.kind() == SyntaxKind::Ident => Ok(t.text()),
        Child::Token(t) => Err(LowerError::new(format!(
            "{ctx}: expected Ident, got {:?}",
            t.kind()
        ))),
        Child::Node(n) => Err(LowerError::new(format!(
            "{ctx}: expected Ident, got node {:?}",
            n.kind()
        ))),
    }
}

fn atom_ident(child: &Child) -> Result<&str, LowerError> {
    match child {
        Child::Token(t) if t.kind() == SyntaxKind::Ident => Ok(t.text()),
        Child::Token(t) => Err(LowerError::new(format!(
            "expected paper Ident, got {:?}",
            t.kind()
        ))),
        Child::Node(n) => Err(LowerError::new(format!(
            "expected paper Ident, got node {:?}",
            n.kind()
        ))),
    }
}

fn number_at(items: &[Child], index: usize, ctx: &str) -> Result<f64, LowerError> {
    let Some(child) = items.get(index) else {
        return Err(LowerError::new(format!("{ctx}: missing number")));
    };
    match child {
        Child::Token(t) if t.kind() == SyntaxKind::Number => t
            .text()
            .parse()
            .map_err(|_| LowerError::new(format!("{ctx}: cannot parse number `{}`", t.text()))),
        Child::Token(t) => Err(LowerError::new(format!(
            "{ctx}: expected Number, got {:?}",
            t.kind()
        ))),
        Child::Node(n) => Err(LowerError::new(format!(
            "{ctx}: expected Number, got node {:?}",
            n.kind()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_pdf::document_to_pdf;
    use reciplexa_scene::Shape;

    const BLACK_CIRCLE: &str = r#"
; A4 black circle (M3)
(page a4
  (circle 105 148.5 40))
"#;

    // --- validity ---

    #[test]
    fn lowers_a4_black_circle() {
        let doc = lower_source(BLACK_CIRCLE).expect("lower");
        assert_eq!(doc.pages.len(), 1);
        let page = &doc.pages[0];
        assert_eq!(page.paper, PaperSize::a4());
        match &page.shapes[0] {
            Shape::Circle(c) => {
                assert_eq!(c.x_mm, 105.0);
                assert_eq!(c.y_mm, 148.5);
                assert_eq!(c.radius_mm, 40.0);
                assert_eq!(c.fill, Color::BLACK);
            }
        }
    }

    #[test]
    fn end_to_end_source_to_pdf_bytes() {
        let doc = lower_source(BLACK_CIRCLE).unwrap();
        let pdf = document_to_pdf(&doc).unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
    }

    #[test]
    fn fixture_file_round_trips_to_pdf() {
        let src = include_str!("../../../examples/black_circle.rpx");
        let pdf = document_to_pdf(&lower_source(src).unwrap()).unwrap();
        assert!(pdf.windows(5).any(|w| w == b"%%EOF"));
    }

    #[test]
    fn trivia_does_not_affect_lowering() {
        let src = "(page   a4\n\n  (circle 1 2 3))";
        let doc = lower_source(src).unwrap();
        match &doc.pages[0].shapes[0] {
            Shape::Circle(c) => {
                assert_eq!(c.x_mm, 1.0);
                assert_eq!(c.y_mm, 2.0);
                assert_eq!(c.radius_mm, 3.0);
            }
        }
    }

    // --- defect ---

    #[test]
    fn empty_source_fails() {
        assert!(lower_source("").is_err());
        assert!(lower_source("   \n").is_err());
    }

    #[test]
    fn parse_error_fails_fast() {
        let err = lower_source("(page a4").unwrap_err();
        assert!(err.message.contains("parse error"));
    }

    #[test]
    fn unknown_shape_fails() {
        let err = lower_source("(page a4 (square 1 2 3))").unwrap_err();
        assert!(err.message.contains("unknown shape"));
    }

    #[test]
    fn wrong_circle_arity_fails() {
        assert!(lower_source("(page a4 (circle 1 2))").is_err());
        assert!(lower_source("(page a4 (circle 1 2 3 4))").is_err());
    }

    #[test]
    fn zero_radius_fails() {
        let err = lower_source("(page a4 (circle 1 2 0))").unwrap_err();
        assert!(err.message.contains("not drawable"));
    }

    #[test]
    fn unknown_paper_fails() {
        let err = lower_source("(page letter (circle 1 2 3))").unwrap_err();
        assert!(err.message.contains("unknown paper"));
    }

    #[test]
    fn non_page_head_fails() {
        let err = lower_source("(sheet a4)").unwrap_err();
        assert!(err.message.contains("expected head `page`"));
    }
}
