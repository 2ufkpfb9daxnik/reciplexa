//! DOCUMENT SURFACE type checker (not the language kernel).
//!
//! Checks page / markup / shape / perform-handle prelude forms for the
//! document pipeline. Language semantics typecheck lives on Core via
//! `reciplexa_core::typecheck_language_source` (TYP-001).
//!
//! This is intentionally a **slice** of the eventual package surface: fixed
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
    /// `(markup …)` block (body not deeply checked yet).
    Markup,
    /// Deprecated Lisp `(src …)` block for logic / effects (not drawn).
    Src,
    String,
    /// Result of `(perform …)` / `(handle …)` and similar side-effecting forms.
    Unit,
    /// Minimal stub for top-level `(type …)` / `(val …)` (bodies not checked yet).
    Decl,
    /// Top-level file: pages / markup / effects / decls (and deprecated src).
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
    let forms: Vec<_> = root
        .children()
        .filter(|n| n.kind() != SyntaxKind::StructuredComment)
        .collect();
    if forms.is_empty() {
        return Err(TypeError::at("empty document", 0, 0));
    }
    for form in &forms {
        let ty = check_form(form)?;
        match ty {
            Type::Page | Type::Markup | Type::Src | Type::Unit | Type::Decl => {}
            other => {
                return Err(TypeError::at(
                    format!(
                        "top-level form must be page, markup, src, perform/handle, or type/val, got {other:?}"
                    ),
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
                    require_ty(&args[1], Type::Number, node)?;
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
                require_ty(a, Type::Shape, node)?;
            }
            Ok(Type::Page)
        }
        "markup" => {
            // M8 / SYN-001: accept markup without typing TextChunk/@ bodies yet.
            Ok(Type::Markup)
        }
        "src" => {
            for a in &args {
                require_src_ty(a, Type::Unit, node)?;
            }
            Ok(Type::Src)
        }
        "perform" => check_perform(&args, node, span),
        "handle" => check_handle(&args, node, span),
        "type" => check_decl("type", &args, span),
        "val" => check_decl("val", &args, span),
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
                require_ty(a, Type::Number, node)?;
            }
            if args.len() == 4 {
                require_ty(&args[3], Type::Color, node)?;
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
                require_ty(a, Type::Number, node)?;
            }
            if args.len() == 5 {
                require_ty(&args[4], Type::Color, node)?;
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
                require_ty(a, Type::Number, node)?;
            }
            if args.len() == 5 {
                require_ty(&args[4], Type::Color, node)?;
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
                require_ty(a, Type::Number, node)?;
            }
            if args.len() == 5 {
                require_ty(&args[4], Type::Color, node)?;
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
                require_ty(a, Type::Number, node)?;
            }
            if args.len() == 6 {
                require_ty(&args[5], Type::Color, node)?;
            }
            Ok(Type::Shape)
        }
        "text" => {
            // (text Num Num Num String [Color])
            // | (text Num Num Num Num Num String [Color])
            match args.len() {
                4 | 5 => {
                    for a in args.iter().take(3) {
                        require_ty(a, Type::Number, node)?;
                    }
                    require_ty(&args[3], Type::String, node)?;
                    if args.len() == 5 {
                        require_ty(&args[4], Type::Color, node)?;
                    }
                }
                6 | 7 => {
                    for a in args.iter().take(5) {
                        require_ty(a, Type::Number, node)?;
                    }
                    require_ty(&args[5], Type::String, node)?;
                    if args.len() == 7 {
                        require_ty(&args[6], Type::Color, node)?;
                    }
                }
                _ => {
                    return Err(TypeError::at(
                        "`text` has type (Num Num Num [Num Num] String [Color]) -> Shape",
                        span.0,
                        span.1,
                    ));
                }
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
                require_ty(a, Type::Number, node)?;
            }
            if args.len() >= 5 {
                require_ty(&args[4], Type::Color, node)?;
            }
            if args.len() == 6 {
                require_ty(&args[5], Type::Number, node)?;
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
            require_ty(&args[0], Type::String, node)?;
            for a in args.iter().skip(1) {
                require_ty(a, Type::Number, node)?;
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
                require_ty(a, Type::Number, node)?;
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
            require_ty(&args[0], Type::Number, node)?;
            require_ty(&args[1], Type::Number, node)?;
            for a in args.iter().skip(2) {
                require_ty(a, Type::Shape, node)?;
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
            require_ty(&args[0], Type::Number, node)?;
            for a in args.iter().skip(1) {
                require_ty(a, Type::Shape, node)?;
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
                require_ty(a, Type::Shape, node)?;
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
            require_ty(&args[0], Type::Number, node)?;
            for a in args.iter().skip(1) {
                require_ty(a, Type::Shape, node)?;
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
            require_ty(&args[1], Type::String, node)?;
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
                "handle" => check_handle(&args, n, span),
                other => Err(TypeError::at(
                    format!("unsupported form in `src`: `{other}` (only `perform` / `handle`)"),
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

/// Minimal `(type name …)` / `(val name …)` — arity only; bodies not checked yet.
fn check_decl(head: &str, args: &[Child], span: (usize, usize)) -> Result<Type, TypeError> {
    if args.len() < 2 {
        return Err(TypeError::at(
            format!("`{head}` needs a name and a body (≥2 args)"),
            span.0,
            span.1,
        ));
    }
    match &args[0] {
        Child::Token(t) if t.kind() == SyntaxKind::Ident => {}
        _ => {
            return Err(TypeError::at(
                format!("`{head}` name must be an identifier"),
                span.0,
                span.1,
            ));
        }
    }
    Ok(Type::Decl)
}

/// `(handle log BODY…)` / `(handle write-path BODY…)` — body forms must be Unit.
fn check_handle(
    args: &[Child],
    node: &SyntaxNode,
    span: (usize, usize),
) -> Result<Type, TypeError> {
    if args.is_empty() {
        return Err(TypeError::at(
            "`handle` needs an effect op name",
            span.0,
            span.1,
        ));
    }
    let op_name = match &args[0] {
        Child::Token(t) if t.kind() == SyntaxKind::Ident => t.text().to_string(),
        _ => {
            return Err(TypeError::at(
                "`handle` op must be an identifier",
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
    if !matches!(op, EffectOp::Log | EffectOp::WritePath) {
        return Err(TypeError::at(
            format!("only `(handle log …)` / `(handle write-path …)` are typed; got `{op_name}`"),
            span.0,
            span.1,
        ));
    }
    for a in args.iter().skip(1) {
        require_src_ty(a, Type::Unit, node)?;
    }
    Ok(Type::Unit)
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
        // Width/color already validated by the predicates above.
        end -= 2;
    } else if end >= 1 && matches!(check_child(&args[end - 1]), Ok(Type::Color)) {
        end -= 1;
    }
    let coords = &args[..end];
    if coords.len() < 4 || !coords.len().is_multiple_of(2) {
        return Err(TypeError::at(
            "`polyline` needs an even number of Num coordinates (≥4)",
            span.0,
            span.1,
        ));
    }
    for a in coords {
        require_ty(a, Type::Number, node)?;
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
        // Color already validated by the predicate above.
        end -= 1;
    }
    let coords = &args[..end];
    if coords.len() < 6 || !coords.len().is_multiple_of(2) {
        return Err(TypeError::at(
            "`polygon` needs an even number of Num coordinates (≥6)",
            span.0,
            span.1,
        ));
    }
    for a in coords {
        require_ty(a, Type::Number, node)?;
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
    require_ty(&args[0], Type::Number, node)?;
    let rest = if args.len() >= 2 && synthesizes_number(&args[1]) {
        // Second factor already known to be a Number token.
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
        require_ty(a, Type::Shape, node)?;
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

/// check_child + expect_ty without nested `?` (keeps region coverage honest).
fn require_ty(child: &Child, want: Type, node: &SyntaxNode) -> Result<(), TypeError> {
    match check_child(child) {
        Ok(got) => expect_ty(got, want, node),
        Err(e) => Err(e),
    }
}

fn require_src_ty(child: &Child, want: Type, node: &SyntaxNode) -> Result<(), TypeError> {
    match check_src_child(child) {
        Ok(got) => expect_ty(got, want, node),
        Err(e) => Err(e),
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
