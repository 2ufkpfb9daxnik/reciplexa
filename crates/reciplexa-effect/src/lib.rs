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

/// Resolve the host RNG seed from an optional env string.
///
/// Missing / empty / non-decimal → `1` (the CLI/GUI default).
pub fn seed_from_env_var(raw: Option<&str>) -> u64 {
    let Some(s) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return 1;
    };
    s.parse::<u64>().unwrap_or(1)
}

/// Read `RECIPLEXA_SEED` (or default `1`).
pub fn seed_from_env() -> u64 {
    seed_from_env_var(std::env::var("RECIPLEXA_SEED").ok().as_deref())
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
    debug_assert!(seen_op, "handle op name validated before body walk");
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
    let mut result = None;
    for el in node.children_with_tokens() {
        if let SyntaxElement::Token(t) = el {
            if t.kind().is_trivia() || t.kind() == SyntaxKind::LParen {
                continue;
            }
            if t.kind() == SyntaxKind::Ident {
                result = Some(t.text().to_string());
            }
            break;
        }
    }
    result
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
            // Lexer string tokens always include the surrounding quotes.
            unescape_string(&raw[1..raw.len() - 1])
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

/// Exposed for conformance tests of perform string payloads.
pub fn unescape_string_for_test(s: &str) -> String {
    unescape_string(s)
}

/// Default handler used in unit tests: logs are collected, random is fixed.
#[derive(Debug, Default)]
pub struct TestHandler {
    pub logs: Vec<String>,
    pub writes: Vec<String>,
    pub random_seq: Vec<f64>,
    random_i: usize,
}

impl TestHandler {
    pub fn with_random_seq(random_seq: Vec<f64>) -> Self {
        Self {
            random_seq,
            ..Default::default()
        }
    }
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
mod private_path_tests {
    use super::*;
    use reciplexa_syntax::parse_source;

    #[test]
    fn list_head_ident_edges() {
        let root = parse_source("[1]\n()\n(123)\n(src [1])")
            .into_result()
            .unwrap();
        let mut forms = root.children();
        let bracket = forms.next().unwrap();
        assert!(list_head_ident(&bracket).is_none());
        let empty = forms.next().unwrap();
        assert!(list_head_ident(&empty).is_none());
        let numbered = forms.next().unwrap();
        assert!(list_head_ident(&numbered).is_none());
        let src = forms.next().unwrap();
        assert_eq!(list_head_ident(&src).as_deref(), Some("src"));
        // Non-list children of src are skipped by run_src_forms / collect.
        let mut h = TestHandler::default();
        assert!(run_src_forms(&mut h, &src).unwrap().is_empty());
        assert!(collect_performs("(src [1] (noop))").unwrap().is_empty());
    }

    #[test]
    fn parse_perform_node_rejects_non_perform() {
        let root = parse_source("(page a4)").into_result().unwrap();
        let page = root.children().next().unwrap();
        let err = parse_perform_node(&page).unwrap_err();
        assert!(err.message.contains("perform"));
    }

    #[test]
    fn handle_skips_non_list_nodes() {
        // BracketList body child hits the `_` arm in run_handle.
        let mut h = TestHandler::default();
        let vals = run_source_effects(
            &mut h,
            r#"(src (handle write-path [1] (perform log "ok")))"#,
        )
        .unwrap();
        assert_eq!(h.logs, vec!["ok"]);
        assert_eq!(vals.len(), 1);
    }
}
