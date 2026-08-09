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
                .expect("parse error spans are ordered"),
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
        // `.children()` yields nodes only; resolve_src_form no-ops non-lists.
        for child in node.children().skip(1) {
            resolve_src_form(&child, stack, env, errors);
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
        let SyntaxElement::Token(tok) = &el else {
            continue;
        };
        resolve_token(tok, stack, env, errors);
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
            .expect("token ranges are ordered"),
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
