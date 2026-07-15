//! Algebraic-effect skeleton for reciplexa (M9).
//!
//! Side effects are expressed as `perform` operations run under top-level
//! `(src …)` blocks (and nested `(handle log|write-path …)` for muted ops). A tiny
//! sequential interpreter returns [`Value`]s so hosts can grow against a stable
//! surface without embedding effects in the PDF or GUI crates.

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

/// Deterministic linear congruential generator for `(perform random)`.
///
/// Produces values in `[0, 1)`. Same seed ⇒ same sequence (CLI/GUI export
/// and tests can share this instead of a hard-coded `0.0` stub).
#[derive(Debug, Clone)]
pub struct LcgRng {
    state: u64,
}

impl LcgRng {
    /// Numerical Recipes–style parameters; `seed == 0` is remapped so the
    /// stream is never stuck at zero.
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0xC0FFEE } else { seed },
        }
    }

    /// Next sample in `[0, 1)`.
    pub fn next_unit(&mut self) -> f64 {
        // LCG: X_{n+1} = (a X_n + c) mod 2^64
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        // Take high 53 bits → float in [0, 1)
        let mantissa = (self.state >> 11) as f64;
        mantissa / ((1u64 << 53) as f64)
    }
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
    let parse = parse_root(input)?;
    let mut out = Vec::new();
    for form in parse.root.children() {
        if !is_list_headed(&form, "src") {
            continue;
        }
        collect_performs_in_list(&form, &mut out)?;
    }
    Ok(out)
}

/// Run every top-level `(src …)` block in order, returning performed values.
pub fn run_source_effects(
    handler: &mut dyn EffectHandler,
    input: &str,
) -> Result<Vec<Value>, EffectError> {
    let parse = parse_root(input)?;
    let mut values = Vec::new();
    for form in parse.root.children() {
        if is_list_headed(&form, "src") {
            values.extend(run_src_forms(handler, &form)?);
        }
    }
    Ok(values)
}

/// Interpret direct children of a `(src …)` list: `perform` and `handle`.
pub fn run_src_forms(
    handler: &mut dyn EffectHandler,
    src_list: &SyntaxNode,
) -> Result<Vec<Value>, EffectError> {
    if !is_list_headed(src_list, "src") {
        return Err(EffectError::new("run_src_forms expects a `(src …)` list"));
    }
    let mut values = Vec::new();
    for child in src_list.children() {
        if child.kind() != reciplexa_syntax::SyntaxKind::List {
            continue;
        }
        values.push(run_src_form(handler, &child)?);
    }
    Ok(values)
}

fn run_src_form(handler: &mut dyn EffectHandler, form: &SyntaxNode) -> Result<Value, EffectError> {
    let head = list_head_ident(form).ok_or_else(|| EffectError::new("src form needs a head"))?;
    match head.as_str() {
        "perform" => {
            let perf = parse_perform_node(form)?;
            run_perform(handler, &perf)
        }
        "handle" => run_handle(handler, form),
        other => Err(EffectError::new(format!(
            "unsupported form in `src`: `{other}` (only `perform` / `handle`)"
        ))),
    }
}

/// `(handle log|write-path BODY…)` runs BODY with that op muted (returns Unit
/// without host I/O). Other ops still forward to the outer handler.
fn run_handle(handler: &mut dyn EffectHandler, form: &SyntaxNode) -> Result<Value, EffectError> {
    use reciplexa_syntax::{SyntaxElement, SyntaxKind};

    let atoms = list_ident_tokens(form);
    if atoms.is_empty() || atoms[0].text() != "handle" {
        return Err(EffectError::new("internal: expected handle"));
    }
    if atoms.len() < 2 || atoms[1].kind() != SyntaxKind::Ident {
        return Err(EffectError::new(
            "`handle` needs an effect op name: (handle log BODY…)",
        ));
    }
    let op_name = atoms[1].text().to_string();
    let Some(op) = EffectOp::parse(&op_name) else {
        return Err(EffectError::new(format!(
            "unknown effect op in handle `{op_name}`"
        )));
    };
    if !matches!(op, EffectOp::Log | EffectOp::WritePath) {
        return Err(EffectError::new(format!(
            "only `(handle log …)` / `(handle write-path …)` are implemented; got `{op_name}`"
        )));
    }

    // Body = List children after the op ident tokens (skip handle + op).
    let mut seen_op = false;
    let mut muted = MuteOp {
        muted: op,
        inner: handler,
    };
    let mut last = Value::Unit;
    for el in form.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) => {
                if t.kind() == SyntaxKind::Ident && t.text() == "handle" {
                    continue;
                }
                if t.kind() == SyntaxKind::Ident && !seen_op {
                    seen_op = true;
                    continue;
                }
            }
            SyntaxElement::Node(n) if n.kind() == SyntaxKind::List => {
                if !seen_op {
                    return Err(EffectError::new("`handle` needs an op before the body"));
                }
                last = run_src_form(&mut muted, &n)?;
            }
            _ => {}
        }
    }
    if !seen_op {
        return Err(EffectError::new("`handle` needs an effect op name"));
    }
    Ok(last)
}

/// Handler adapter that swallows one muted op while forwarding others.
struct MuteOp<'a> {
    muted: EffectOp,
    inner: &'a mut dyn EffectHandler,
}

impl EffectHandler for MuteOp<'_> {
    fn on_log(&mut self, message: &str) -> Result<Value, EffectError> {
        if self.muted == EffectOp::Log {
            Ok(Value::Unit)
        } else {
            self.inner.on_log(message)
        }
    }

    fn on_random(&mut self) -> Result<Value, EffectError> {
        self.inner.on_random()
    }

    fn on_write_path(&mut self, path: &str) -> Result<Value, EffectError> {
        if self.muted == EffectOp::WritePath {
            Ok(Value::Unit)
        } else {
            self.inner.on_write_path(path)
        }
    }
}

fn parse_root(input: &str) -> Result<reciplexa_syntax::Parse, EffectError> {
    use reciplexa_syntax::parse_source;

    let parse = parse_source(input);
    if !parse.errors.is_empty() {
        return Err(EffectError::new(format!(
            "parse error: {}",
            parse.errors[0].message
        )));
    }
    Ok(parse)
}

fn is_list_headed(node: &SyntaxNode, name: &str) -> bool {
    list_head_ident(node).is_some_and(|h| h == name)
}

fn list_head_ident(node: &SyntaxNode) -> Option<String> {
    use reciplexa_syntax::{SyntaxElement, SyntaxKind};
    if node.kind() != SyntaxKind::List {
        return None;
    }
    for el in node.children_with_tokens() {
        if let SyntaxElement::Token(t) = el {
            if t.kind().is_trivia() || t.kind() == SyntaxKind::LParen {
                continue;
            }
            if t.kind() == SyntaxKind::Ident {
                return Some(t.text().to_string());
            }
            return None;
        }
    }
    None
}

fn list_ident_tokens(node: &SyntaxNode) -> Vec<reciplexa_syntax::SyntaxToken> {
    use reciplexa_syntax::{SyntaxElement, SyntaxKind};
    let mut atoms = Vec::new();
    for el in node.children_with_tokens() {
        if let SyntaxElement::Token(t) = el {
            if t.kind().is_trivia() || matches!(t.kind(), SyntaxKind::LParen | SyntaxKind::RParen) {
                continue;
            }
            atoms.push(t);
        }
    }
    atoms
}

fn collect_performs_in_list(node: &SyntaxNode, out: &mut Vec<Perform>) -> Result<(), EffectError> {
    for child in node.children() {
        if child.kind() != reciplexa_syntax::SyntaxKind::List {
            continue;
        }
        let head = list_head_ident(&child);
        match head.as_deref() {
            Some("perform") => out.push(parse_perform_node(&child)?),
            Some("handle") => {
                // Nested performs inside handle still count for collect_performs.
                collect_performs_in_list(&child, out)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_perform_node(node: &SyntaxNode) -> Result<Perform, EffectError> {
    use reciplexa_syntax::SyntaxKind;

    let atoms = list_ident_tokens(node);
    // perform also needs String tokens — list_ident_tokens only keeps all non-trivia tokens actually
    // Wait, it pushes ALL non-delimiter tokens including String. Good - rename was wrong in my head.
    if atoms.is_empty() || atoms[0].kind() != SyntaxKind::Ident || atoms[0].text() != "perform" {
        return Err(EffectError::new("expected `(perform …)`"));
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
    Ok(Perform { op, payload })
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

    #[test]
    fn lcg_same_seed_same_sequence() {
        let mut a = LcgRng::new(42);
        let mut b = LcgRng::new(42);
        let seq_a: Vec<f64> = (0..8).map(|_| a.next_unit()).collect();
        let seq_b: Vec<f64> = (0..8).map(|_| b.next_unit()).collect();
        assert_eq!(seq_a, seq_b);
        for v in &seq_a {
            assert!(*v >= 0.0 && *v < 1.0, "out of range: {v}");
        }
    }

    #[test]
    fn lcg_different_seeds_diverge() {
        let mut a = LcgRng::new(1);
        let mut b = LcgRng::new(2);
        let first_a = a.next_unit();
        let first_b = b.next_unit();
        assert_ne!(first_a, first_b);
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

    #[test]
    fn run_src_returns_values_in_order() {
        let src = r#"(src (perform log "a") (perform random))"#;
        let mut h = TestHandler {
            random_seq: vec![0.42],
            ..Default::default()
        };
        let vals = run_source_effects(&mut h, src).unwrap();
        assert_eq!(h.logs, vec!["a"]);
        assert_eq!(vals, vec![Value::Unit, Value::Number(0.42)]);
    }

    #[test]
    fn handle_log_mutes_nested_log() {
        let src = r#"
(src
  (perform log "outer")
  (handle log
    (perform log "silent")
    (perform random))
  (perform log "after"))
"#;
        let mut h = TestHandler {
            random_seq: vec![0.1],
            ..Default::default()
        };
        let vals = run_source_effects(&mut h, src).unwrap();
        assert_eq!(h.logs, vec!["outer", "after"]);
        assert!(!h.logs.iter().any(|l| l == "silent"));
        assert_eq!(vals.len(), 3);
        assert_eq!(vals[1], Value::Number(0.1));
    }

    #[test]
    fn handle_write_path_mutes_nested_writes() {
        let src = r#"
(src
  (perform write-path "outer.pdf")
  (handle write-path
    (perform write-path "silent.pdf")
    (perform log "still logs"))
  (perform write-path "after.pdf"))
"#;
        let mut h = TestHandler::default();
        run_source_effects(&mut h, src).unwrap();
        assert_eq!(h.writes, vec!["outer.pdf", "after.pdf"]);
        assert!(!h.writes.iter().any(|w| w == "silent.pdf"));
        assert_eq!(h.logs, vec!["still logs"]);
    }

    #[test]
    fn nested_handles_mute_independently() {
        let src = r#"
(src
  (handle log
    (handle write-path
      (perform log "silent-log")
      (perform write-path "silent.pdf")
      (perform random)))
  (perform log "after")
  (perform write-path "after.pdf"))
"#;
        let mut h = TestHandler {
            random_seq: vec![0.5],
            ..Default::default()
        };
        let vals = run_source_effects(&mut h, src).unwrap();
        assert_eq!(h.logs, vec!["after"]);
        assert_eq!(h.writes, vec!["after.pdf"]);
        assert_eq!(vals[0], Value::Number(0.5));
    }

    #[test]
    fn handle_random_is_rejected() {
        let err = run_source_effects(
            &mut TestHandler::default(),
            "(src (handle random (perform log \"x\")))",
        )
        .unwrap_err();
        assert!(
            err.message.contains("handle") && err.message.contains("random"),
            "{}",
            err.message
        );
    }

    #[test]
    fn unsupported_src_form_errors() {
        let err =
            run_source_effects(&mut TestHandler::default(), "(src (define x 1))").unwrap_err();
        assert!(err.message.contains("unsupported"));
    }
}
