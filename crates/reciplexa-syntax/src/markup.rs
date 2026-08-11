//! Walk Scribble `(markup …)` CSTs into structured parts (M8 / SYN-001).
//!
//! Reading (`@`, TextChunk, modes) lives here; document *meaning* (layout,
//! packages) belongs to macro / later packages.

#![forbid(unsafe_code)]

use crate::{SyntaxElement, SyntaxKind, SyntaxNode};

/// One piece of a Scribble markup body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkupPart {
    Text(String),
    Newline,
    At {
        name: String,
        /// Bracket-list text without outer `[` `]` (args as source), if present.
        bracket_args: Option<String>,
        /// Nested scribble body from `{…}`, if present.
        brace_body: Vec<MarkupPart>,
    },
    /// SYN §17.6: `@(…)` arbitrary code embedding (source of the parenthesized form).
    Embed {
        source: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkupWalkError {
    pub message: String,
}

impl MarkupWalkError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Walk a `(markup …)` List node into ordered [`MarkupPart`]s.
pub fn markup_parts(markup_list: &SyntaxNode) -> Result<Vec<MarkupPart>, MarkupWalkError> {
    if markup_list.kind() != SyntaxKind::List {
        return Err(MarkupWalkError::new("markup_parts expects a List node"));
    }
    let mut head_seen = false;
    let mut body_started = false;
    let mut parts = Vec::new();
    for el in markup_list.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) => {
                if matches!(t.kind(), SyntaxKind::LParen | SyntaxKind::RParen) {
                    continue;
                }
                if !head_seen {
                    if t.kind().is_trivia() {
                        continue;
                    }
                    if t.kind() == SyntaxKind::Ident && t.text() == "markup" {
                        head_seen = true;
                        continue;
                    }
                    return Err(MarkupWalkError::new("expected head `markup`"));
                }
                if t.kind().is_trivia() {
                    // Leading trivia after `markup` is separator, not content.
                    if !body_started {
                        continue;
                    }
                    match t.kind() {
                        SyntaxKind::Whitespace => {
                            parts.push(MarkupPart::Text(t.text().to_string()));
                        }
                        SyntaxKind::Newline => parts.push(MarkupPart::Newline),
                        _ => {}
                    }
                    continue;
                }
                body_started = true;
                match t.kind() {
                    SyntaxKind::TextChunk => parts.push(MarkupPart::Text(t.text().to_string())),
                    // Newline is trivia and handled above; any other non-trivia token is invalid.
                    other => {
                        return Err(MarkupWalkError::new(format!(
                            "unexpected token `{other:?}` in markup body"
                        )));
                    }
                }
            }
            SyntaxElement::Node(n) => {
                if !head_seen {
                    return Err(MarkupWalkError::new("expected head `markup` before body"));
                }
                body_started = true;
                match n.kind() {
                    SyntaxKind::AtExpr => parts.push(walk_at_expr(&n)?),
                    SyntaxKind::BraceList => {
                        // Rare nested `{…}` at markup top level: unwrap to body parts.
                        parts.extend(walk_scribble_container(&n)?);
                    }
                    SyntaxKind::ErrorNode => {
                        return Err(MarkupWalkError::new("markup body contains an error node"));
                    }
                    other => {
                        return Err(MarkupWalkError::new(format!(
                            "unexpected node `{other:?}` in markup body"
                        )));
                    }
                }
            }
        }
    }
    if !head_seen {
        return Err(MarkupWalkError::new("expected head `markup`"));
    }
    Ok(parts)
}

/// Flatten parts to readable plain text.
///
/// Newlines become a single space. `@name{body}` uses the brace body;
/// `@name[args]` without braces uses the bracket args; bare `@name` is empty.
pub fn flatten_readable(parts: &[MarkupPart]) -> String {
    flatten_lines(parts).join(" ")
}

/// Flatten into visual lines (newline-separated), trimming each line.
pub fn flatten_lines(parts: &[MarkupPart]) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    flush_part_lines(parts, &mut lines, &mut cur);
    if !cur.is_empty() || lines.is_empty() {
        lines.push(collapse_ws(&cur));
    }
    lines
        .into_iter()
        .map(|l| collapse_ws(&l))
        .filter(|l| !l.is_empty())
        .collect()
}

fn flush_part_lines(parts: &[MarkupPart], lines: &mut Vec<String>, cur: &mut String) {
    for part in parts {
        match part {
            MarkupPart::Text(t) => cur.push_str(t),
            MarkupPart::Newline => {
                lines.push(std::mem::take(cur));
            }
            MarkupPart::At {
                bracket_args,
                brace_body,
                ..
            } => {
                if !brace_body.is_empty() {
                    flush_part_lines(brace_body, lines, cur);
                } else if let Some(args) = bracket_args {
                    cur.push_str(args.trim());
                }
            }
            MarkupPart::Embed { source } => cur.push_str(source),
        }
    }
}

fn collapse_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_space && !out.is_empty() {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

fn walk_at_expr(node: &SyntaxNode) -> Result<MarkupPart, MarkupWalkError> {
    let mut name: Option<String> = None;
    let mut bracket_args: Option<String> = None;
    let mut brace_body: Vec<MarkupPart> = Vec::new();
    let mut bare_list: Option<SyntaxNode> = None;

    for el in node.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) => {
                if t.kind() == SyntaxKind::At || t.kind().is_trivia() {
                    continue;
                }
                if t.kind() == SyntaxKind::Ident {
                    name.get_or_insert_with(|| t.text().to_string());
                }
            }
            SyntaxElement::Node(n) => match n.kind() {
                SyntaxKind::BracketList => {
                    bracket_args = Some(inner_delimited_text(&n));
                }
                SyntaxKind::BraceList => {
                    brace_body = walk_scribble_container(&n)?;
                }
                SyntaxKind::List => {
                    if name.is_some() {
                        // SYN-001 `@name(markup-body)` — paren body is scribble, not a Lisp call.
                        brace_body = walk_scribble_container(&n)?;
                    } else {
                        // SYN §17.6: `@(…)` code embedding — keep the parenthesized form.
                        bare_list = Some(n);
                    }
                }
                _ => {}
            },
        }
    }

    if let Some(name) = name {
        return Ok(MarkupPart::At {
            name,
            bracket_args,
            brace_body,
        });
    }

    if let Some(list) = bare_list {
        let source = list.to_string();
        if source == "()" {
            return Err(MarkupWalkError::new("empty `@(…)` embedding"));
        }
        return Ok(MarkupPart::Embed { source });
    }

    Err(MarkupWalkError::new("expected identifier after `@`"))
}

fn walk_scribble_container(node: &SyntaxNode) -> Result<Vec<MarkupPart>, MarkupWalkError> {
    let mut parts = Vec::new();
    for el in node.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) => {
                if matches!(
                    t.kind(),
                    SyntaxKind::LBrace
                        | SyntaxKind::RBrace
                        | SyntaxKind::LBracket
                        | SyntaxKind::RBracket
                        | SyntaxKind::LParen
                        | SyntaxKind::RParen
                ) {
                    continue;
                }
                if t.kind().is_trivia() {
                    if t.kind() == SyntaxKind::Whitespace {
                        parts.push(MarkupPart::Text(t.text().to_string()));
                    } else {
                        // Non-whitespace trivia (newline/comment) is skipped.
                    }
                    continue;
                }
                match t.kind() {
                    SyntaxKind::TextChunk => parts.push(MarkupPart::Text(t.text().to_string())),
                    // Newline is trivia and handled above; any other non-trivia token is invalid.
                    other => {
                        return Err(MarkupWalkError::new(format!(
                            "unexpected token `{other:?}` in scribble brace"
                        )));
                    }
                }
            }
            SyntaxElement::Node(n) => match n.kind() {
                SyntaxKind::AtExpr => parts.push(walk_at_expr(&n)?),
                SyntaxKind::BraceList => parts.extend(walk_scribble_container(&n)?),
                SyntaxKind::ErrorNode => {
                    return Err(MarkupWalkError::new("scribble body contains an error node"));
                }
                other => {
                    return Err(MarkupWalkError::new(format!(
                        "unexpected node `{other:?}` in scribble brace"
                    )));
                }
            },
        }
    }
    Ok(parts)
}

fn inner_delimited_text(node: &SyntaxNode) -> String {
    let full = node.to_string();
    if full.len() >= 2 {
        full[1..full.len() - 1].to_string()
    } else {
        full
    }
}
