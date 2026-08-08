//! Walk Scribble `(doc …)` CSTs into structured parts (M8).
//!
//! Reading (`@`, TextChunk, modes) lives here; document *meaning* (layout,
//! packages) belongs to macro / later packages.

#![forbid(unsafe_code)]

use crate::{SyntaxElement, SyntaxKind, SyntaxNode};

/// One piece of a Scribble document body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocPart {
    Text(String),
    Newline,
    At {
        name: String,
        /// Bracket-list text without outer `[` `]` (args as source), if present.
        bracket_args: Option<String>,
        /// Nested scribble body from `{…}`, if present.
        brace_body: Vec<DocPart>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocWalkError {
    pub message: String,
}

impl DocWalkError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Walk a `(doc …)` List node into ordered [`DocPart`]s.
pub fn doc_parts(doc_list: &SyntaxNode) -> Result<Vec<DocPart>, DocWalkError> {
    if doc_list.kind() != SyntaxKind::List {
        return Err(DocWalkError::new("doc_parts expects a List node"));
    }
    let mut head_seen = false;
    let mut body_started = false;
    let mut parts = Vec::new();
    for el in doc_list.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) => {
                if matches!(t.kind(), SyntaxKind::LParen | SyntaxKind::RParen) {
                    continue;
                }
                if !head_seen {
                    if t.kind().is_trivia() {
                        continue;
                    }
                    if t.kind() == SyntaxKind::Ident && t.text() == "doc" {
                        head_seen = true;
                        continue;
                    }
                    return Err(DocWalkError::new("expected head `doc`"));
                }
                if t.kind().is_trivia() {
                    // Leading trivia after `doc` is separator, not content.
                    if !body_started {
                        continue;
                    }
                    match t.kind() {
                        SyntaxKind::Whitespace => {
                            parts.push(DocPart::Text(t.text().to_string()));
                        }
                        SyntaxKind::Newline => parts.push(DocPart::Newline),
                        _ => {}
                    }
                    continue;
                }
                body_started = true;
                match t.kind() {
                    SyntaxKind::TextChunk => parts.push(DocPart::Text(t.text().to_string())),
                    SyntaxKind::Newline => parts.push(DocPart::Newline),
                    other => {
                        return Err(DocWalkError::new(format!(
                            "unexpected token `{other:?}` in doc body"
                        )));
                    }
                }
            }
            SyntaxElement::Node(n) => {
                if !head_seen {
                    return Err(DocWalkError::new("expected head `doc` before body"));
                }
                body_started = true;
                match n.kind() {
                    SyntaxKind::AtExpr => parts.push(walk_at_expr(&n)?),
                    SyntaxKind::BraceList => {
                        // Rare nested `{…}` at doc top level: unwrap to body parts.
                        parts.extend(walk_scribble_container(&n)?);
                    }
                    SyntaxKind::ErrorNode => {
                        return Err(DocWalkError::new("doc body contains an error node"));
                    }
                    other => {
                        return Err(DocWalkError::new(format!(
                            "unexpected node `{other:?}` in doc body"
                        )));
                    }
                }
            }
        }
    }
    if !head_seen {
        return Err(DocWalkError::new("expected head `doc`"));
    }
    Ok(parts)
}

/// Flatten parts to readable plain text.
///
/// Newlines become a single space. `@name{body}` uses the brace body;
/// `@name[args]` without braces uses the bracket args; bare `@name` is empty.
pub fn flatten_readable(parts: &[DocPart]) -> String {
    flatten_lines(parts).join(" ")
}

/// Flatten into visual lines (newline-separated), trimming each line.
pub fn flatten_lines(parts: &[DocPart]) -> Vec<String> {
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

fn flush_part_lines(parts: &[DocPart], lines: &mut Vec<String>, cur: &mut String) {
    for part in parts {
        match part {
            DocPart::Text(t) => cur.push_str(t),
            DocPart::Newline => {
                lines.push(std::mem::take(cur));
            }
            DocPart::At {
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

fn walk_at_expr(node: &SyntaxNode) -> Result<DocPart, DocWalkError> {
    let mut name: Option<String> = None;
    let mut bracket_args: Option<String> = None;
    let mut brace_body: Vec<DocPart> = Vec::new();

    for el in node.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) => {
                if t.kind() == SyntaxKind::At || t.kind().is_trivia() {
                    continue;
                }
                if t.kind() == SyntaxKind::Ident && name.is_none() {
                    name = Some(t.text().to_string());
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
                    // `@` then a list form is unusual for M8 identity expand; reject.
                    return Err(DocWalkError::new(
                        "list form after `@` is not supported in doc walk yet",
                    ));
                }
                _ => {}
            },
        }
    }

    let Some(name) = name else {
        return Err(DocWalkError::new("expected identifier after `@`"));
    };
    Ok(DocPart::At {
        name,
        bracket_args,
        brace_body,
    })
}

fn walk_scribble_container(node: &SyntaxNode) -> Result<Vec<DocPart>, DocWalkError> {
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
                ) {
                    continue;
                }
                if t.kind().is_trivia() {
                    if t.kind() == SyntaxKind::Whitespace {
                        parts.push(DocPart::Text(t.text().to_string()));
                    }
                    continue;
                }
                match t.kind() {
                    SyntaxKind::TextChunk => parts.push(DocPart::Text(t.text().to_string())),
                    SyntaxKind::Newline => parts.push(DocPart::Newline),
                    other => {
                        return Err(DocWalkError::new(format!(
                            "unexpected token `{other:?}` in scribble brace"
                        )));
                    }
                }
            }
            SyntaxElement::Node(n) => match n.kind() {
                SyntaxKind::AtExpr => parts.push(walk_at_expr(&n)?),
                SyntaxKind::BraceList => parts.extend(walk_scribble_container(&n)?),
                SyntaxKind::ErrorNode => {
                    return Err(DocWalkError::new("scribble body contains an error node"));
                }
                other => {
                    return Err(DocWalkError::new(format!(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_source;

    fn doc_list(src: &str) -> SyntaxNode {
        let root = parse_source(src).into_result().unwrap();
        root.children()
            .find(|n| n.kind() == SyntaxKind::List)
            .expect("top-level list")
    }

    // --- validity ---

    #[test]
    fn plain_text_doc() {
        let parts = doc_parts(&doc_list("(doc Hello world)")).unwrap();
        // Scribble TextChunks include the space after `doc`.
        assert_eq!(parts, vec![DocPart::Text(" Hello world".into())]);
        assert_eq!(flatten_readable(&parts), "Hello world");
    }

    #[test]
    fn newlines_become_spaces_when_flattened() {
        let parts = doc_parts(&doc_list("(doc\nline1\nline2\n)")).unwrap();
        assert!(parts.contains(&DocPart::Newline));
        assert_eq!(flatten_readable(&parts), "line1 line2");
        assert_eq!(super::flatten_lines(&parts), vec!["line1", "line2"]);
    }

    #[test]
    fn at_em_structure_and_flatten() {
        let parts = doc_parts(&doc_list("(doc Hello @em{世界}.)")).unwrap();
        assert_eq!(
            parts,
            vec![
                DocPart::Text(" Hello ".into()),
                DocPart::At {
                    name: "em".into(),
                    bracket_args: None,
                    brace_body: vec![DocPart::Text("世界".into())],
                },
                DocPart::Text(".".into()),
            ]
        );
        assert_eq!(flatten_readable(&parts), "Hello 世界.");
    }

    #[test]
    fn at_link_and_cite() {
        let parts = doc_parts(&doc_list(
            "(doc See @link[\"https://example.com\"]{docs}. Cite @cite[42].)",
        ))
        .unwrap();
        assert_eq!(flatten_readable(&parts), "See docs. Cite 42.");
    }

    // --- defect ---

    #[test]
    fn non_doc_list_errors() {
        let err = doc_parts(&doc_list("(page a4)")).unwrap_err();
        assert!(err.message.contains("doc"));
    }

    #[test]
    fn bare_at_ident_flattens_empty() {
        let parts = doc_parts(&doc_list("(doc see @ref)")).unwrap();
        assert_eq!(flatten_readable(&parts), "see");
    }

    #[test]
    fn at_with_brace_only_body() {
        let parts = doc_parts(&doc_list("(doc @em{emphasis})")).unwrap();
        assert_eq!(flatten_readable(&parts), "emphasis");
    }

    #[test]
    fn empty_doc_body() {
        let parts = doc_parts(&doc_list("(doc)")).unwrap();
        assert!(parts.is_empty());
        assert_eq!(flatten_readable(&parts), "");
    }

    #[test]
    fn doc_part_debug() {
        let part = DocPart::Text("x".into());
        assert!(format!("{part:?}").contains("Text"));
    }

    #[test]
    fn non_list_node_errors() {
        let root = parse_source("(doc hi)").into_result().unwrap();
        let err = doc_parts(&root).unwrap_err();
        assert!(err.message.contains("List"));
    }

    #[test]
    fn wrong_head_errors() {
        let list = doc_list("(page a4)");
        let err = doc_parts(&list).unwrap_err();
        assert!(err.message.contains("doc"));
    }

    #[test]
    fn error_node_in_body_errors() {
        let parse = parse_source("(doc @em{hi)");
        let list = parse.root.children().next().unwrap();
        assert!(doc_parts(&list).is_err());
    }

    #[test]
    fn nested_brace_list_at_top_level() {
        let parts = doc_parts(&doc_list("(doc {nested})")).unwrap();
        assert_eq!(flatten_readable(&parts), "nested");
    }

    #[test]
    fn flatten_lines_skips_empty_lines() {
        let parts = vec![
            DocPart::Text("  ".into()),
            DocPart::Newline,
            DocPart::Text("ok".into()),
        ];
        assert_eq!(flatten_lines(&parts), vec!["ok"]);
    }

    #[test]
    fn at_with_only_bracket_args() {
        let parts = doc_parts(&doc_list("(doc prefix @unknown[args-only])")).unwrap();
        assert_eq!(flatten_readable(&parts), "prefix args-only");
    }

    #[test]
    fn doc_walk_error_debug() {
        let err = DocWalkError::new("msg");
        assert!(format!("{err:?}").contains("msg"));
    }
}
