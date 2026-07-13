//! Algebraic-effect skeleton for reciplexa (M9 entry).
//!
//! Side effects (I/O, draw, randomness, …) will eventually be expressed as
//! `perform` operations handled by `handle` forms. This crate only defines the
//! labels and a tiny pure interpreter stub so the rest of the stack can grow
//! against a stable surface without embedding effects in the PDF or GUI crates.

#![forbid(unsafe_code)]

use std::fmt;

use reciplexa_syntax::SyntaxNode;

/// Named effect operations the language will eventually expose.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectOp {
    /// Emit a log line (testing / debugging).
    Log,
    /// Request a fresh random `f64` in `0..1` (placeholder).
    Random,
    /// Ask the host to flush/write an artifact path.
    WritePath,
}

impl fmt::Display for EffectOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EffectOp::Log => write!(f, "log"),
            EffectOp::Random => write!(f, "random"),
            EffectOp::WritePath => write!(f, "write-path"),
        }
    }
}

impl EffectOp {
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "log" => Self::Log,
            "random" => Self::Random,
            "write-path" => Self::WritePath,
            _ => return None,
        })
    }
}

/// A performed effect with a string payload (until a richer IR exists).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Perform {
    pub op: EffectOp,
    pub payload: String,
}

/// Pure values returned by the stub interpreter.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Number(f64),
    String(String),
}

/// Host-supplied handlers for performed effects.
pub trait EffectHandler {
    fn on_log(&mut self, message: &str) -> Result<Value, EffectError>;
    fn on_random(&mut self) -> Result<Value, EffectError>;
    fn on_write_path(&mut self, path: &str) -> Result<Value, EffectError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectError {
    pub message: String,
}

impl EffectError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Run a single `perform` against a handler (no full expression language yet).
pub fn run_perform(handler: &mut dyn EffectHandler, perf: &Perform) -> Result<Value, EffectError> {
    match perf.op {
        EffectOp::Log => handler.on_log(&perf.payload),
        EffectOp::Random => handler.on_random(),
        EffectOp::WritePath => handler.on_write_path(&perf.payload),
    }
}

/// Collect `(perform …)` forms nested under top-level `(src …)` blocks.
pub fn collect_performs(input: &str) -> Result<Vec<Perform>, EffectError> {
    use reciplexa_syntax::parse_source;

    let parse = parse_source(input);
    if !parse.errors.is_empty() {
        return Err(EffectError::new(format!(
            "parse error: {}",
            parse.errors[0].message
        )));
    }
    let mut out = Vec::new();
    for form in parse.root.children() {
        if !is_list_headed(&form, "src") {
            continue;
        }
        collect_performs_in_list(&form, &mut out)?;
    }
    Ok(out)
}

fn is_list_headed(node: &SyntaxNode, name: &str) -> bool {
    use reciplexa_syntax::{SyntaxElement, SyntaxKind};
    if node.kind() != SyntaxKind::List {
        return false;
    }
    for el in node.children_with_tokens() {
        if let SyntaxElement::Token(t) = el {
            if t.kind().is_trivia() || t.kind() == SyntaxKind::LParen {
                continue;
            }
            return t.kind() == SyntaxKind::Ident && t.text() == name;
        }
    }
    false
}

fn collect_performs_in_list(node: &SyntaxNode, out: &mut Vec<Perform>) -> Result<(), EffectError> {
    use reciplexa_syntax::{SyntaxElement, SyntaxKind};

    for child in node.children() {
        if child.kind() != SyntaxKind::List {
            continue;
        }
        let mut atoms = Vec::new();
        for el in child.children_with_tokens() {
            match el {
                SyntaxElement::Token(t) => {
                    if t.kind().is_trivia()
                        || matches!(t.kind(), SyntaxKind::LParen | SyntaxKind::RParen)
                    {
                        continue;
                    }
                    atoms.push(t);
                }
                SyntaxElement::Node(_) => {}
            }
        }
        if atoms.is_empty() || atoms[0].kind() != SyntaxKind::Ident {
            continue;
        }
        if atoms[0].text() != "perform" {
            continue;
        }
        if atoms.len() < 2 || atoms[1].kind() != SyntaxKind::Ident {
            return Err(EffectError::new("perform needs an op identifier"));
        }
        let op_name = atoms[1].text();
        let Some(op) = EffectOp::parse(op_name) else {
            return Err(EffectError::new(format!("unknown effect op `{op_name}`")));
        };
        let payload = match op {
            EffectOp::Random => String::new(),
            EffectOp::Log | EffectOp::WritePath => {
                if atoms.len() < 3 || atoms[2].kind() != SyntaxKind::String {
                    return Err(EffectError::new(format!(
                        "perform {op_name} needs a string payload"
                    )));
                }
                let raw = atoms[2].text();
                if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
                    unescape_string(&raw[1..raw.len() - 1])
                } else {
                    return Err(EffectError::new("malformed string payload"));
                }
            }
        };
        out.push(Perform { op, payload });
    }
    Ok(())
}

fn unescape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Default handler used in unit tests: logs are collected, random is fixed.
#[derive(Debug, Default)]
pub struct TestHandler {
    pub logs: Vec<String>,
    pub writes: Vec<String>,
    pub random_seq: Vec<f64>,
    random_i: usize,
}

impl EffectHandler for TestHandler {
    fn on_log(&mut self, message: &str) -> Result<Value, EffectError> {
        self.logs.push(message.to_string());
        Ok(Value::Unit)
    }

    fn on_random(&mut self) -> Result<Value, EffectError> {
        let v = self.random_seq.get(self.random_i).copied().unwrap_or(0.0);
        self.random_i += 1;
        Ok(Value::Number(v))
    }

    fn on_write_path(&mut self, path: &str) -> Result<Value, EffectError> {
        self.writes.push(path.to_string());
        Ok(Value::Unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- validity ---

    #[test]
    fn parses_known_ops() {
        assert_eq!(EffectOp::parse("log"), Some(EffectOp::Log));
        assert_eq!(EffectOp::parse("write-path"), Some(EffectOp::WritePath));
    }

    #[test]
    fn test_handler_collects_log_and_write() {
        let mut h = TestHandler::default();
        run_perform(
            &mut h,
            &Perform {
                op: EffectOp::Log,
                payload: "hi".into(),
            },
        )
        .unwrap();
        run_perform(
            &mut h,
            &Perform {
                op: EffectOp::WritePath,
                payload: "out.pdf".into(),
            },
        )
        .unwrap();
        assert_eq!(h.logs, vec!["hi"]);
        assert_eq!(h.writes, vec!["out.pdf"]);
    }

    #[test]
    fn random_consumes_sequence() {
        let mut h = TestHandler {
            random_seq: vec![0.25, 0.5],
            ..Default::default()
        };
        assert_eq!(
            run_perform(
                &mut h,
                &Perform {
                    op: EffectOp::Random,
                    payload: String::new(),
                }
            )
            .unwrap(),
            Value::Number(0.25)
        );
        assert_eq!(
            run_perform(
                &mut h,
                &Perform {
                    op: EffectOp::Random,
                    payload: String::new(),
                }
            )
            .unwrap(),
            Value::Number(0.5)
        );
    }

    // --- defect ---

    #[test]
    fn unknown_op_name_is_none() {
        assert_eq!(EffectOp::parse("draw"), None);
        assert_eq!(EffectOp::parse(""), None);
    }

    #[test]
    fn collects_performs_from_src_blocks() {
        let src = r#"
(src
  (perform log "hello")
  (perform write-path "out.pdf"))
(page a4 (circle 1 2 3))
"#;
        let ps = collect_performs(src).unwrap();
        assert_eq!(ps.len(), 2);
        assert_eq!(ps[0].op, EffectOp::Log);
        assert_eq!(ps[0].payload, "hello");
        assert_eq!(ps[1].op, EffectOp::WritePath);
    }
}
