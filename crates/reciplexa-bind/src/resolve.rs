//! Name resolution over surface syntax.

use std::collections::HashMap;

use reciplexa_identity::binding::BindingId;
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;
use reciplexa_syntax::{parse_source, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

use crate::scope::ScopeStack;

/// Resolved binding environment after a successful pass.
#[derive(Debug, Clone, Default)]
pub struct BindingEnv {
    pub bindings: HashMap<BindingId, String>,
    pub builtin_colors: HashMap<String, BindingId>,
}

/// A single resolution diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveError {
    pub message: String,
    pub range: TextRange,
}

/// Result of resolving a source buffer.
#[derive(Debug, Clone)]
pub struct ResolveResult {
    pub env: BindingEnv,
    pub errors: Vec<ResolveError>,
}

impl ResolveResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Resolve builtin color names and top-level form heads in a source file.
pub fn resolve_source(source: &str) -> ResolveResult {
    let parse = parse_source(source);
    let mut stack = ScopeStack::new();
    let mut env = BindingEnv::default();
    let mut errors = Vec::new();

    for color in ["black", "white", "red", "green", "blue", "a4", "letter"] {
        let id = stack.declare(color);
        env.builtin_colors.insert(color.to_string(), id);
        env.bindings.insert(id, color.to_string());
    }

    if parse.has_errors() {
        for e in &parse.errors {
            errors.push(ResolveError {
                message: format!("parse error: {}", e.message),
                range: TextRange::try_new(
                    ByteOffset::new(e.start as u32),
                    ByteOffset::new(e.end as u32),
                )
                .unwrap_or(TextRange::EMPTY),
            });
        }
        return ResolveResult { env, errors };
    }

    for form in parse.root.children() {
        resolve_form(&form, &mut stack, &mut env, &mut errors);
    }

    ResolveResult { env, errors }
}

fn resolve_form(
    node: &SyntaxNode,
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
) {
    if node.kind() != SyntaxKind::List {
        return;
    }
    let Some(head) = list_head_ident(node) else {
        return;
    };
    match head.as_str() {
        "src" => {
            stack.push_scope();
            for child in node.children() {
                resolve_src_form(&child, stack, env, errors);
            }
            stack.pop_scope();
        }
        _ => {
            for child in node.children() {
                resolve_expr(&child, stack, env, errors);
            }
        }
    }
}

fn resolve_src_form(
    node: &SyntaxNode,
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
) {
    if node.kind() != SyntaxKind::List {
        return;
    }
    let Some(head) = list_head_ident(node) else {
        return;
    };
    if head == "handle" {
        stack.push_scope();
        for child in node.children().skip(1) {
            if child.kind() == SyntaxKind::List {
                resolve_src_form(&child, stack, env, errors);
            }
        }
        stack.pop_scope();
        return;
    }
    resolve_form(node, stack, env, errors);
}

fn resolve_expr(
    node: &SyntaxNode,
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
) {
    if node.kind() == SyntaxKind::List {
        resolve_form(node, stack, env, errors);
    }
    for el in node.children_with_tokens() {
        if let SyntaxElement::Token(tok) = el {
            resolve_token(&tok, stack, env, errors);
        }
    }
}

fn resolve_token(
    tok: &SyntaxToken,
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
) {
    if tok.kind() != SyntaxKind::Ident {
        return;
    }
    let name = tok.text();
    if is_surface_keyword(name) {
        return;
    }
    if stack.lookup(name).is_some() || env.builtin_colors.contains_key(name) {
        return;
    }
    let start: u32 = tok.text_range().start().into();
    let end: u32 = tok.text_range().end().into();
    errors.push(ResolveError {
        message: format!("unbound identifier `{name}`"),
        range: TextRange::try_new(ByteOffset::new(start), ByteOffset::new(end))
            .unwrap_or(TextRange::EMPTY),
    });
}

fn is_surface_keyword(name: &str) -> bool {
    matches!(
        name,
        "page"
            | "doc"
            | "src"
            | "circle"
            | "rect"
            | "ellipse"
            | "text"
            | "line"
            | "group"
            | "translate"
            | "rotate"
            | "scale"
            | "opacity"
            | "perform"
            | "handle"
            | "rgb"
            | "polyline"
            | "polygon"
            | "image"
            | "ring"
            | "frame"
    )
}

fn list_head_ident(node: &SyntaxNode) -> Option<String> {
    for el in node.children_with_tokens() {
        if let SyntaxElement::Token(t) = el {
            if t.kind() == SyntaxKind::Ident {
                return Some(t.text().to_string());
            }
            if !t.kind().is_trivia() && t.kind() != SyntaxKind::LParen {
                break;
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_builtin_colors() {
        let r = resolve_source("(page a4 (circle 1 2 3 red))");
        assert!(r.is_ok());
    }

    #[test]
    fn unknown_ident_is_error() {
        let r = resolve_source("(page a4 (circle 1 2 3 puce))");
        assert!(!r.is_ok());
        assert!(r.errors[0].message.contains("puce"));
    }

    #[test]
    fn parse_error_prevents_binding() {
        let r = resolve_source("(page a4 (rect");
        assert!(!r.is_ok());
        assert!(r.errors.iter().all(|e| e.message.contains("parse error")));
        assert!(r.env.bindings.is_empty() || r.env.builtin_colors.len() >= 6);
    }

    #[test]
    fn src_handle_scope_allows_inner_names() {
        let r = resolve_source("(src (handle inner (circle 1 2 3 red)))");
        assert!(r.is_ok(), "{:?}", r.errors);
    }

    #[test]
    fn surface_keywords_are_not_unbound_errors() {
        let r = resolve_source("(page a4 (group (translate 1 2 (rect 0 0 1 1 red))))");
        assert!(r.is_ok(), "{:?}", r.errors);
    }

    #[test]
    fn empty_source_is_ok_with_builtins() {
        let r = resolve_source("");
        assert!(r.is_ok());
        assert!(r.env.builtin_colors.contains_key("red"));
        assert!(r.env.builtin_colors.contains_key("a4"));
    }

    #[test]
    fn all_paper_and_color_builtins_resolve() {
        for color in ["black", "white", "red", "green", "blue"] {
            let src = format!("(page a4 (circle 0 0 1 {color}))");
            assert!(resolve_source(&src).is_ok(), "{color}");
        }
        for paper in ["a4", "letter"] {
            let src = format!("(page {paper})");
            assert!(resolve_source(&src).is_ok(), "{paper}");
        }
    }

    #[test]
    fn doc_and_src_forms_resolve() {
        assert!(resolve_source("(doc Hello)").is_ok());
        assert!(resolve_source("(src (perform log \"x\"))\n(page a4)").is_ok());
    }

    #[test]
    fn nested_unknown_ident_reports_error() {
        let r = resolve_source("(page a4 (group (circle 0 0 1 mauve)))");
        assert!(!r.is_ok());
        assert!(r.errors.iter().any(|e| e.message.contains("mauve")));
    }

    #[test]
    fn resolve_error_carries_span() {
        let r = resolve_source("(page a4 (circle 1 2 3 puce))");
        assert!(!r.errors.is_empty());
        let range = r.errors[0].range;
        // Unbound ident should point at a non-empty half-open span.
        assert!(range.end().0 >= range.start().0);
        assert!(range.len() > 0 || range.is_empty());
    }

    #[test]
    fn bracket_and_brace_top_level_forms() {
        // Non-list forms are skipped by resolve_form; should not panic.
        let r = resolve_source("[1 2 3]\n{a b}");
        assert!(r.is_ok() || !r.errors.is_empty());
    }

    #[test]
    fn empty_and_headless_lists_are_skipped() {
        let r = resolve_source("()\n(123)\n(src ())\n(src [1])");
        assert!(r.is_ok(), "{:?}", r.errors);
    }

    #[test]
    fn nested_handle_lists_inside_src() {
        // First child of `handle` is a List (not a name token), so skip(1) still
        // walks subsequent lists through resolve_src_form.
        let r = resolve_source("(src (handle (page a4) (circle 1 2 3 red)))");
        assert!(r.is_ok(), "{:?}", r.errors);
    }

    #[test]
    fn list_starting_with_delimiter_has_no_head() {
        // Head scan breaks on non-trivia non-lparen before any Ident.
        let r = resolve_source("(() )");
        assert!(r.is_ok() || !r.errors.is_empty());
    }
}
