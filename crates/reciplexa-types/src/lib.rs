//! Minimal structural type checker for the M3–M9 surface vocabulary.
//!
//! This is intentionally a **slice** of the eventual type system: fixed
//! builtin signatures, no polymorphism, thin effect rows via `perform`.
//! Fail-fast: the first error aborts (SATySFi-style for this milestone).

#![forbid(unsafe_code)]

use reciplexa_effect::EffectOp;
use reciplexa_syntax::{parse_source, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

/// Types inhabited by the current surface vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Number,
    Color,
    Shape,
    Paper,
    Page,
    /// Scribble `(doc …)` block (body not deeply checked yet).
    Doc,
    /// Lisp `(src …)` block for logic / effects (not drawn).
    Src,
    String,
    /// Result of `(perform …)` and similar side-effecting forms.
    Unit,
    /// Top-level file: pages / docs / src blocks.
    Document,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeError {
    pub message: String,
    pub start: usize,
    pub end: usize,
}

impl TypeError {
    fn at(message: impl Into<String>, start: usize, end: usize) -> Self {
        Self {
            message: message.into(),
            start,
            end,
        }
    }
}

/// Parse and type-check a source buffer.
pub fn typecheck_source(input: &str) -> Result<Type, TypeError> {
    let parse = parse_source(input);
    if !parse.errors.is_empty() {
        let e = &parse.errors[0];
        return Err(TypeError::at(
            format!("parse error: {}", e.message),
            e.start,
            e.end,
        ));
    }
    typecheck_syntax(&parse.root)
}

pub fn typecheck_syntax(root: &SyntaxNode) -> Result<Type, TypeError> {
    if root.kind() != SyntaxKind::SourceFile {
        return Err(TypeError::at(
            format!("expected SourceFile, got {:?}", root.kind()),
            0,
            0,
        ));
    }
    let forms: Vec<_> = root.children().collect();
    if forms.is_empty() {
        return Err(TypeError::at("empty document", 0, 0));
    }
    for form in &forms {
        let ty = check_form(form)?;
        match ty {
            Type::Page | Type::Doc | Type::Src => {}
            other => {
                return Err(TypeError::at(
                    format!("top-level form must be page, doc, or src, got {other:?}"),
                    form.text_range().start().into(),
                    form.text_range().end().into(),
                ));
            }
        }
    }
    Ok(Type::Document)
}

fn check_form(node: &SyntaxNode) -> Result<Type, TypeError> {
    let (head, args, span) = split_list(node)?;
    match head.as_str() {
        "page" => {
            if args.is_empty() {
                return Err(TypeError::at(
                    "`page` needs a paper size (`a4` / `letter`) or width/height in mm",
                    span.0,
                    span.1,
                ));
            }
            let shape_start = match check_child(&args[0])? {
                Type::Paper => 1,
                Type::Number => {
                    if args.len() < 2 {
                        return Err(TypeError::at(
                            "`page` numeric paper needs (page width-mm height-mm …)",
                            span.0,
                            span.1,
                        ));
                    }
                    expect_ty(check_child(&args[1])?, Type::Number, node)?;
                    2
                }
                other => {
                    return Err(TypeError::at(
                        format!("`page` paper must be Paper or Num, got {other:?}"),
                        span.0,
                        span.1,
                    ));
                }
            };
            for a in args.iter().skip(shape_start) {
                expect_ty(check_child(a)?, Type::Shape, node)?;
            }
            Ok(Type::Page)
        }
        "doc" => {
            // M8: accept scribble docs without typing TextChunk/@ bodies yet.
            Ok(Type::Doc)
        }
        "src" => {
            for a in &args {
                expect_ty(check_src_child(a)?, Type::Unit, node)?;
            }
            Ok(Type::Src)
        }
        "perform" => check_perform(&args, node, span),
        "circle" => {
            // (circle Num Num Num) | (circle Num Num Num Color)
            if args.len() != 3 && args.len() != 4 {
                return Err(TypeError::at(
                    "`circle` has type (Num Num Num) or (Num Num Num Color) -> Shape",
                    span.0,
                    span.1,
                ));
            }
            for a in args.iter().take(3) {
                expect_ty(check_child(a)?, Type::Number, node)?;
            }
            if args.len() == 4 {
                expect_ty(check_child(&args[3])?, Type::Color, node)?;
            }
            Ok(Type::Shape)
        }
        "rect" => {
            // (rect Num Num Num Num) | (rect Num Num Num Num Color)
            if args.len() != 4 && args.len() != 5 {
                return Err(TypeError::at(
                    "`rect` has type (Num×4) or (Num×4 Color) -> Shape",
                    span.0,
                    span.1,
                ));
            }
            for a in args.iter().take(4) {
                expect_ty(check_child(a)?, Type::Number, node)?;
            }
            if args.len() == 5 {
                expect_ty(check_child(&args[4])?, Type::Color, node)?;
            }
            Ok(Type::Shape)
        }
        "ellipse" => {
            if args.len() != 4 && args.len() != 5 {
                return Err(TypeError::at(
                    "`ellipse` has type (Num×4 [Color]) -> Shape",
                    span.0,
                    span.1,
                ));
            }
            for a in args.iter().take(4) {
                expect_ty(check_child(a)?, Type::Number, node)?;
            }
            if args.len() == 5 {
                expect_ty(check_child(&args[4])?, Type::Color, node)?;
            }
            Ok(Type::Shape)
        }
        "ring" => {
            if args.len() != 4 && args.len() != 5 {
                return Err(TypeError::at(
                    "`ring` has type (Num×4 [Color]) -> Shape",
                    span.0,
                    span.1,
                ));
            }
            for a in args.iter().take(4) {
                expect_ty(check_child(a)?, Type::Number, node)?;
            }
            if args.len() == 5 {
                expect_ty(check_child(&args[4])?, Type::Color, node)?;
            }
            Ok(Type::Shape)
        }
        "frame" => {
            if args.len() != 5 && args.len() != 6 {
                return Err(TypeError::at(
                    "`frame` has type (Num×5 [Color]) -> Shape",
                    span.0,
                    span.1,
                ));
            }
            for a in args.iter().take(5) {
                expect_ty(check_child(a)?, Type::Number, node)?;
            }
            if args.len() == 6 {
                expect_ty(check_child(&args[5])?, Type::Color, node)?;
            }
            Ok(Type::Shape)
        }
        "text" => {
            // (text Num Num Num String) | (+ Color)
            if args.len() != 4 && args.len() != 5 {
                return Err(TypeError::at(
                    "`text` has type (Num Num Num String [Color]) -> Shape",
                    span.0,
                    span.1,
                ));
            }
            for a in args.iter().take(3) {
                expect_ty(check_child(a)?, Type::Number, node)?;
            }
            expect_ty(check_child(&args[3])?, Type::String, node)?;
            if args.len() == 5 {
                expect_ty(check_child(&args[4])?, Type::Color, node)?;
            }
            Ok(Type::Shape)
        }
        "line" => {
            if args.len() < 4 || args.len() > 6 {
                return Err(TypeError::at(
                    "`line` has type (Num×4 [Color [Num]]) -> Shape",
                    span.0,
                    span.1,
                ));
            }
            for a in args.iter().take(4) {
                expect_ty(check_child(a)?, Type::Number, node)?;
            }
            if args.len() >= 5 {
                expect_ty(check_child(&args[4])?, Type::Color, node)?;
            }
            if args.len() == 6 {
                expect_ty(check_child(&args[5])?, Type::Number, node)?;
            }
            Ok(Type::Shape)
        }
        "polyline" => check_polyline(&args, node, span),
        "polygon" => check_polygon(&args, node, span),
        "image" => {
            if args.len() != 5 {
                return Err(TypeError::at(
                    "`image` has type (String Num Num Num Num) -> Shape",
                    span.0,
                    span.1,
                ));
            }
            expect_ty(check_child(&args[0])?, Type::String, node)?;
            for a in args.iter().skip(1) {
                expect_ty(check_child(a)?, Type::Number, node)?;
            }
            Ok(Type::Shape)
        }
        "rgb" => {
            if args.len() != 3 {
                return Err(TypeError::at(
                    "`rgb` has type (Num Num Num) -> Color",
                    span.0,
                    span.1,
                ));
            }
            for a in &args {
                expect_ty(check_child(a)?, Type::Number, node)?;
            }
            Ok(Type::Color)
        }
        "translate" => {
            if args.len() < 3 {
                return Err(TypeError::at(
                    "`translate` has type (Num Num Shape+) -> Shape",
                    span.0,
                    span.1,
                ));
            }
            expect_ty(check_child(&args[0])?, Type::Number, node)?;
            expect_ty(check_child(&args[1])?, Type::Number, node)?;
            for a in args.iter().skip(2) {
                expect_ty(check_child(a)?, Type::Shape, node)?;
            }
            Ok(Type::Shape)
        }
        "rotate" => {
            if args.len() < 2 {
                return Err(TypeError::at(
                    "`rotate` has type (Num Shape+) -> Shape",
                    span.0,
                    span.1,
                ));
            }
            expect_ty(check_child(&args[0])?, Type::Number, node)?;
            for a in args.iter().skip(1) {
                expect_ty(check_child(a)?, Type::Shape, node)?;
            }
            Ok(Type::Shape)
        }
        "scale" => check_scale(&args, node, span),
        "group" => {
            if args.is_empty() {
                return Err(TypeError::at(
                    "`group` has type (Shape+) -> Shape",
                    span.0,
                    span.1,
                ));
            }
            for a in &args {
                expect_ty(check_child(a)?, Type::Shape, node)?;
            }
            Ok(Type::Shape)
        }
        "opacity" => {
            if args.len() < 2 {
                return Err(TypeError::at(
                    "`opacity` has type (Num Shape+) -> Shape",
                    span.0,
                    span.1,
                ));
            }
            expect_ty(check_child(&args[0])?, Type::Number, node)?;
            for a in args.iter().skip(1) {
                expect_ty(check_child(a)?, Type::Shape, node)?;
            }
            Ok(Type::Shape)
        }
        other => Err(TypeError::at(
            format!("unknown form `{other}`"),
            span.0,
            span.1,
        )),
    }
}

fn check_perform(
    args: &[Child],
    node: &SyntaxNode,
    span: (usize, usize),
) -> Result<Type, TypeError> {
    if args.is_empty() {
        return Err(TypeError::at(
            "`perform` needs an effect op name",
            span.0,
            span.1,
        ));
    }
    let op_name = match &args[0] {
        Child::Token(t) if t.kind() == SyntaxKind::Ident => t.text().to_string(),
        _ => {
            return Err(TypeError::at(
                "`perform` op must be an identifier",
                span.0,
                span.1,
            ));
        }
    };
    let Some(op) = EffectOp::parse(&op_name) else {
        return Err(TypeError::at(
            format!("unknown effect op `{op_name}`"),
            span.0,
            span.1,
        ));
    };
    match op {
        EffectOp::Log | EffectOp::WritePath => {
            if args.len() != 2 {
                return Err(TypeError::at(
                    format!("`perform {op_name}` has type (String) -> Unit"),
                    span.0,
                    span.1,
                ));
            }
            expect_ty(check_child(&args[1])?, Type::String, node)?;
        }
        EffectOp::Random => {
            if args.len() != 1 {
                return Err(TypeError::at(
                    "`perform random` has type () -> Number (payload forbidden)",
                    span.0,
                    span.1,
                ));
            }
            // Typed as Unit for src bodies for now; Number result comes with an evaluator.
        }
    }
    Ok(Type::Unit)
}

fn check_src_child(child: &Child) -> Result<Type, TypeError> {
    match child {
        Child::Node(n) => {
            let (head, args, span) = split_list(n)?;
            match head.as_str() {
                "perform" => check_perform(&args, n, span),
                other => Err(TypeError::at(
                    format!("unsupported form in `src`: `{other}` (only `perform` for now)"),
                    span.0,
                    span.1,
                )),
            }
        }
        Child::Token(t) => {
            let range = t.text_range();
            Err(TypeError::at(
                "src body must be list forms",
                range.start().into(),
                range.end().into(),
            ))
        }
    }
}

fn check_polyline(
    args: &[Child],
    node: &SyntaxNode,
    span: (usize, usize),
) -> Result<Type, TypeError> {
    if args.len() < 4 {
        return Err(TypeError::at(
            "`polyline` needs ≥2 points (Num×even) [Color [Num]]",
            span.0,
            span.1,
        ));
    }
    let mut end = args.len();
    if end >= 2
        && synthesizes_number(&args[end - 1])
        && matches!(check_child(&args[end - 2]), Ok(Type::Color))
    {
        expect_ty(check_child(&args[end - 1])?, Type::Number, node)?;
        expect_ty(check_child(&args[end - 2])?, Type::Color, node)?;
        end -= 2;
    } else if end >= 1 && matches!(check_child(&args[end - 1]), Ok(Type::Color)) {
        expect_ty(check_child(&args[end - 1])?, Type::Color, node)?;
        end -= 1;
    }
    let coords = &args[..end];
    if coords.len() < 4 || coords.len() % 2 != 0 {
        return Err(TypeError::at(
            "`polyline` needs an even number of Num coordinates (≥4)",
            span.0,
            span.1,
        ));
    }
    for a in coords {
        expect_ty(check_child(a)?, Type::Number, node)?;
    }
    Ok(Type::Shape)
}

fn check_polygon(
    args: &[Child],
    node: &SyntaxNode,
    span: (usize, usize),
) -> Result<Type, TypeError> {
    if args.len() < 6 {
        return Err(TypeError::at(
            "`polygon` needs ≥3 points (Num×even) [Color]",
            span.0,
            span.1,
        ));
    }
    let mut end = args.len();
    if end >= 1 && matches!(check_child(&args[end - 1]), Ok(Type::Color)) {
        expect_ty(check_child(&args[end - 1])?, Type::Color, node)?;
        end -= 1;
    }
    let coords = &args[..end];
    if coords.len() < 6 || coords.len() % 2 != 0 {
        return Err(TypeError::at(
            "`polygon` needs an even number of Num coordinates (≥6)",
            span.0,
            span.1,
        ));
    }
    for a in coords {
        expect_ty(check_child(a)?, Type::Number, node)?;
    }
    Ok(Type::Shape)
}

fn check_scale(args: &[Child], node: &SyntaxNode, span: (usize, usize)) -> Result<Type, TypeError> {
    if args.is_empty() {
        return Err(TypeError::at(
            "`scale` has type (Num Shape+) or (Num Num Shape+) -> Shape",
            span.0,
            span.1,
        ));
    }
    expect_ty(check_child(&args[0])?, Type::Number, node)?;
    let rest = if args.len() >= 2 && synthesizes_number(&args[1]) {
        expect_ty(check_child(&args[1])?, Type::Number, node)?;
        &args[2..]
    } else {
        &args[1..]
    };
    if rest.is_empty() {
        return Err(TypeError::at(
            "`scale` needs at least one Shape body",
            span.0,
            span.1,
        ));
    }
    for a in rest {
        expect_ty(check_child(a)?, Type::Shape, node)?;
    }
    Ok(Type::Shape)
}

fn synthesizes_number(child: &Child) -> bool {
    matches!(child, Child::Token(t) if t.kind() == SyntaxKind::Number)
}

fn check_child(child: &Child) -> Result<Type, TypeError> {
    match child {
        Child::Token(t) => check_token(t),
        Child::Node(n) => check_form(n),
    }
}

fn check_token(t: &SyntaxToken) -> Result<Type, TypeError> {
    let range = t.text_range();
    let start: usize = range.start().into();
    let end: usize = range.end().into();
    match t.kind() {
        SyntaxKind::Number => Ok(Type::Number),
        SyntaxKind::String => Ok(Type::String),
        SyntaxKind::Ident => match t.text() {
            "a4" | "letter" => Ok(Type::Paper),
            "black" | "white" | "red" | "green" | "blue" => Ok(Type::Color),
            other => Err(TypeError::at(
                format!("unbound identifier `{other}`"),
                start,
                end,
            )),
        },
        other => Err(TypeError::at(
            format!("cannot type token `{other:?}`"),
            start,
            end,
        )),
    }
}

fn expect_ty(got: Type, want: Type, node: &SyntaxNode) -> Result<(), TypeError> {
    if got == want {
        Ok(())
    } else {
        let range = node.text_range();
        Err(TypeError::at(
            format!("type mismatch: expected {want:?}, got {got:?}"),
            range.start().into(),
            range.end().into(),
        ))
    }
}

#[derive(Debug)]
enum Child {
    Node(SyntaxNode),
    Token(SyntaxToken),
}

type SplitList = (String, Vec<Child>, (usize, usize));

fn split_list(node: &SyntaxNode) -> Result<SplitList, TypeError> {
    let range = node.text_range();
    let span = (range.start().into(), range.end().into());
    if node.kind() != SyntaxKind::List {
        return Err(TypeError::at(
            format!("expected list form, got {:?}", node.kind()),
            span.0,
            span.1,
        ));
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
    let Some(Child::Token(head_tok)) = items.first() else {
        return Err(TypeError::at("empty list", span.0, span.1));
    };
    if head_tok.kind() != SyntaxKind::Ident {
        return Err(TypeError::at(
            "list head must be an identifier",
            span.0,
            span.1,
        ));
    }
    let head = head_tok.text().to_string();
    Ok((head, items.into_iter().skip(1).collect(), span))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- validity ---

    #[test]
    fn black_circle_page_is_document() {
        let ty = typecheck_source("(page a4 (circle 105 148.5 40))").unwrap();
        assert_eq!(ty, Type::Document);
    }

    #[test]
    fn letter_and_opacity_typecheck() {
        let src = "(page letter (opacity 0.5 (circle 1 2 3 red)))";
        assert_eq!(typecheck_source(src).unwrap(), Type::Document);
    }

    #[test]
    fn numeric_paper_typecheck() {
        let src = "(page 210 297 (circle 1 2 3))";
        assert_eq!(typecheck_source(src).unwrap(), Type::Document);
    }

    #[test]
    fn transformed_colored_program_typechecks() {
        let src = r#"
(page a4
  (translate 105 148.5
    (rotate 30
      (scale 1.5
        (circle 0 0 20 red)))))
"#;
        assert_eq!(typecheck_source(src).unwrap(), Type::Document);
    }

    #[test]
    fn rgb_and_nonuniform_scale_typecheck() {
        let src = "(page a4 (scale 2 3 (circle 0 0 5 (rgb 0.2 0.4 0.6))))";
        assert_eq!(typecheck_source(src).unwrap(), Type::Document);
    }

    #[test]
    fn doc_block_typechecks_as_document() {
        assert_eq!(
            typecheck_source("(doc Hello @em{x})").unwrap(),
            Type::Document
        );
    }

    #[test]
    fn mixed_page_and_doc_typecheck() {
        assert_eq!(
            typecheck_source("(page a4 (circle 1 2 3))\n(doc hi)").unwrap(),
            Type::Document
        );
    }

    #[test]
    fn text_and_line_typecheck() {
        let src = r#"(page a4 (text 1 2 3 "Hi" red) (line 0 0 10 10 blue 0.5))"#;
        assert_eq!(typecheck_source(src).unwrap(), Type::Document);
    }

    #[test]
    fn ellipse_typecheck() {
        assert_eq!(
            typecheck_source("(page a4 (ellipse 1 2 3 4 red))").unwrap(),
            Type::Document
        );
    }

    #[test]
    fn ring_and_frame_typecheck() {
        let src = "(page a4 (ring 1 2 3 0.5) (frame 0 0 10 10 1 red))";
        assert_eq!(typecheck_source(src).unwrap(), Type::Document);
    }

    #[test]
    fn polyline_typecheck() {
        assert_eq!(
            typecheck_source("(page a4 (polyline 0 0 1 1 2 0 red 1))").unwrap(),
            Type::Document
        );
    }

    #[test]
    fn src_with_perform_typechecks() {
        let src = r#"
(src
  (perform log "building")
  (perform random))
(page a4 (circle 1 2 3))
"#;
        assert_eq!(typecheck_source(src).unwrap(), Type::Document);
    }

    // --- defect ---

    #[test]
    fn circle_with_shape_where_number_fails() {
        let err = typecheck_source("(page a4 (circle (circle 0 0 1) 0 1))").unwrap_err();
        assert!(err.message.contains("type mismatch"));
    }

    #[test]
    fn page_body_must_be_shape() {
        let err = typecheck_source("(page a4 12)").unwrap_err();
        assert!(err.message.contains("type mismatch"));
    }

    #[test]
    fn unknown_ident_fails() {
        let err = typecheck_source("(page a4 (circle 0 0 1 puce))").unwrap_err();
        assert!(err.message.contains("unbound identifier"));
    }

    #[test]
    fn unknown_effect_op_fails() {
        let err =
            typecheck_source("(src (perform draw \"x\"))\n(page a4 (circle 1 2 3))").unwrap_err();
        assert!(err.message.contains("unknown effect op"));
    }

    #[test]
    fn unknown_form_fails() {
        let err = typecheck_source("(page a4 (square 1))").unwrap_err();
        assert!(err.message.contains("unknown form"));
    }

    #[test]
    fn parse_error_surfaces() {
        let err = typecheck_source("(page a4").unwrap_err();
        assert!(err.message.contains("parse error"));
    }

    #[test]
    fn scale_without_body_fails() {
        assert!(typecheck_source("(page a4 (scale 2))").is_err());
    }

    #[test]
    fn empty_source_fails() {
        assert!(typecheck_source("").is_err());
    }
}
