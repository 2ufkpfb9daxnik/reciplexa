//! Minimal structural type checker for the M3/M4 surface vocabulary.
//!
//! This is intentionally a **slice** of the eventual type system: fixed
//! builtin signatures, no polymorphism, no effect rows yet. Inference here
//! means synthesizing result types of known forms from checked arguments.
//! Fail-fast: the first error aborts (SATySFi-style for this milestone).

#![forbid(unsafe_code)]

use reciplexa_syntax::{parse_source, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

/// Types inhabited by M4 surface values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Number,
    Color,
    Shape,
    Paper,
    Page,
    /// Scribble `(doc …)` block (body not deeply checked yet).
    Doc,
    /// Top-level file: one or more pages/docs.
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
            Type::Page | Type::Doc => {}
            other => {
                return Err(TypeError::at(
                    format!("top-level form must be page or doc, got {other:?}"),
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
                return Err(TypeError::at("`page` needs a paper size", span.0, span.1));
            }
            expect_ty(check_child(&args[0])?, Type::Paper, node)?;
            for a in args.iter().skip(1) {
                expect_ty(check_child(a)?, Type::Shape, node)?;
            }
            Ok(Type::Page)
        }
        "doc" => {
            // M8: accept scribble docs without typing TextChunk/@ bodies yet.
            Ok(Type::Doc)
        }
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
        other => Err(TypeError::at(
            format!("unknown form `{other}`"),
            span.0,
            span.1,
        )),
    }
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
        SyntaxKind::Ident => match t.text() {
            "a4" => Ok(Type::Paper),
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
