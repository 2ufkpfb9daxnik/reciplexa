//! Name resolution over surface syntax.
//!
//! Two entry points:
//! - [`resolve_language_source`] — lexical resolve for language-kernel forms
//!   (`val` / `fn` / `let` / `type`), with shadowing and unbound diagnostics.
//! - [`resolve_source`] — document/graphics surface (colors, page, circle, …).

use std::collections::HashMap;

use reciplexa_identity::binding::BindingId;
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;
use reciplexa_syntax::{
    coalesce_slash_paths, is_reserved_special_form, is_wildcard_ident, normalize_ident,
    parse_source, validate_ident, SlashAtom, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken,
};

use crate::scope::ScopeStack;

/// Resolved binding environment after a successful pass.
#[derive(Debug, Clone, Default)]
pub struct BindingEnv {
    pub bindings: HashMap<BindingId, String>,
    pub builtin_colors: HashMap<String, BindingId>,
}

/// Use-site → declaration [`BindingId`] map (EDT-001).
#[derive(Debug, Clone, Default)]
pub struct BindingMap {
    /// Source range of an identifier use → declaring binding.
    pub uses: HashMap<TextRange, BindingId>,
}

impl BindingMap {
    pub fn record(&mut self, range: TextRange, id: BindingId) {
        self.uses.insert(range, id);
    }

    pub fn binding_at(&self, range: TextRange) -> Option<BindingId> {
        self.uses.get(&range).copied()
    }
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
    /// Populated by [`resolve_language_source`] for use-site BindingId lookup.
    pub binding_map: BindingMap,
}

impl ResolveResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

enum Atom {
    Token(SyntaxToken),
    /// Joined module / qualified path (`color/black`).
    Path(String),
    Node(SyntaxNode),
}

/// Lexical name resolution for the language-kernel surface.
///
/// Walks top-level `val` / `fn` / `type` binders and nested `let` / `fn`
/// scopes. Does **not** install document builtins (colors / paper sizes) and
/// skips quarantined graphics forms (`page`, `circle`, …).
pub fn resolve_language_source(source: &str) -> ResolveResult {
    let parse = parse_source(source);
    let mut stack = ScopeStack::new();
    let mut env = BindingEnv::default();
    let mut errors = Vec::new();
    let mut binding_map = BindingMap::default();

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
        return ResolveResult {
            env,
            errors,
            binding_map,
        };
    }

    for el in parse.root.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) => {
                if t.kind().is_trivia() {
                    continue;
                }
                lang_resolve_token(&t, &stack, &mut errors, &mut binding_map);
            }
            SyntaxElement::Node(n) => match n.kind() {
                SyntaxKind::StructuredComment => continue,
                SyntaxKind::List => {
                    lang_resolve_top_form(&n, &mut stack, &mut env, &mut errors, &mut binding_map)
                }
                _ => {
                    lang_resolve_expr_node(&n, &mut stack, &mut env, &mut errors, &mut binding_map)
                }
            },
        }
    }

    ResolveResult {
        env,
        errors,
        binding_map,
    }
}

/// Resolve builtin color names and top-level form heads in a document source.
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
        return ResolveResult {
            env,
            errors,
            binding_map: BindingMap::default(),
        };
    }

    for form in parse.root.children() {
        // Top-level `(// …)` forms are not binding targets.
        if form.kind() == SyntaxKind::StructuredComment {
            continue;
        }
        resolve_form(&form, &mut stack, &mut env, &mut errors);
    }

    ResolveResult {
        env,
        errors,
        binding_map: BindingMap::default(),
    }
}

fn lang_resolve_top_form(
    node: &SyntaxNode,
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
    map: &mut BindingMap,
) {
    if is_quarantined_head(node) {
        return;
    }
    let atoms = list_atoms(node);
    let Some(Atom::Token(head)) = atoms.first() else {
        lang_resolve_expr_node(node, stack, env, errors, map);
        return;
    };
    if head.kind() != SyntaxKind::Ident {
        lang_resolve_expr_node(node, stack, env, errors, map);
        return;
    }
    match head.text() {
        "data" => {
            // `(data Name [((a type)…)] ctor…)` — declare type name; ctors are value constructors.
            if let Some(Atom::Token(name_tok)) = atoms.get(1) {
                if name_tok.kind() == SyntaxKind::Ident {
                    declare_binding(name_tok, stack, env, errors);
                }
            }
            let mut ctor_atoms = &atoms[2..];
            // Skip DAT-001 type-parameter section `((a type)…)`.
            if let Some(Atom::Node(params)) = ctor_atoms.first() {
                if params.kind() == SyntaxKind::List {
                    let inner = list_atoms(params);
                    if matches!(inner.first(), Some(Atom::Node(_))) {
                        ctor_atoms = &ctor_atoms[1..];
                    }
                }
            }
            for ctor in ctor_atoms {
                match ctor {
                    Atom::Token(t) if t.kind() == SyntaxKind::Ident => {
                        declare_binding(t, stack, env, errors);
                    }
                    Atom::Node(n) if n.kind() == SyntaxKind::List => {
                        if let Some(Atom::Token(tag)) = list_atoms(n).first() {
                            if tag.kind() == SyntaxKind::Ident {
                                declare_binding(tag, stack, env, errors);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        "val" => lang_resolve_val_decl(&atoms[1..], stack, env, errors, map),
        "fn" => {
            // Top-level named function: (fn name (params...) body...)
            if atoms.len() >= 3 {
                if let (Atom::Token(name_tok), Atom::Node(params)) = (&atoms[1], &atoms[2]) {
                    if name_tok.kind() == SyntaxKind::Ident && is_param_list(params) {
                        lang_resolve_fn_body(params, &atoms[3..], stack, env, errors, map);
                        declare_binding(name_tok, stack, env, errors);
                        return;
                    }
                }
            }
            // Anonymous `(fn …)` at top level is an expression.
            lang_resolve_expr_node(node, stack, env, errors, map);
        }
        "type" => {
            // Declare the type binder; type-expression bodies are a separate
            // namespace and are not walked as value use-sites yet.
            if let Some(Atom::Token(name_tok)) = atoms.get(1) {
                if name_tok.kind() == SyntaxKind::Ident {
                    declare_binding(name_tok, stack, env, errors);
                }
            }
        }
        _ => lang_resolve_expr_node(node, stack, env, errors, map),
    }
}

fn lang_resolve_val_decl(
    rest: &[Atom],
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
    map: &mut BindingMap,
) {
    if rest.is_empty() {
        return;
    }
    match &rest[0] {
        Atom::Token(name_tok) if name_tok.kind() == SyntaxKind::Ident => {
            // Resolve initializer before declaring (non-recursive).
            for atom in &rest[1..] {
                lang_resolve_atom(atom, stack, env, errors, map);
            }
            declare_binding(name_tok, stack, env, errors);
        }
        // (val (name params...) body...) named-function sugar
        Atom::Node(binder) if binder.kind() == SyntaxKind::List => {
            let binder_atoms = list_atoms(binder);
            let Some(Atom::Token(name_tok)) = binder_atoms.first() else {
                return;
            };
            if name_tok.kind() != SyntaxKind::Ident {
                return;
            }
            stack.push_scope();
            for atom in &binder_atoms[1..] {
                if let Atom::Token(t) = atom {
                    if t.kind() == SyntaxKind::Ident {
                        declare_binding(t, stack, env, errors);
                    }
                }
            }
            for atom in &rest[1..] {
                lang_resolve_atom(atom, stack, env, errors, map);
            }
            stack.pop_scope();
            declare_binding(name_tok, stack, env, errors);
        }
        other => {
            // Malformed binder — still try to resolve remaining atoms.
            lang_resolve_atom(other, stack, env, errors, map);
            for atom in &rest[1..] {
                lang_resolve_atom(atom, stack, env, errors, map);
            }
        }
    }
}

fn lang_resolve_fn_body(
    params: &SyntaxNode,
    body: &[Atom],
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
    map: &mut BindingMap,
) {
    stack.push_scope();
    for atom in list_atoms(params) {
        if let Atom::Token(t) = atom {
            if t.kind() == SyntaxKind::Ident {
                declare_binding(&t, stack, env, errors);
            }
        }
    }
    for atom in body {
        lang_resolve_atom(atom, stack, env, errors, map);
    }
    stack.pop_scope();
}

fn lang_resolve_expr_node(
    node: &SyntaxNode,
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
    map: &mut BindingMap,
) {
    match node.kind() {
        SyntaxKind::List | SyntaxKind::BracketList => {
            lang_resolve_list(node, stack, env, errors, map)
        }
        _ => {
            for el in node.children_with_tokens() {
                if let SyntaxElement::Token(t) = el {
                    lang_resolve_token(&t, stack, errors, map);
                } else if let SyntaxElement::Node(n) = el {
                    lang_resolve_expr_node(&n, stack, env, errors, map);
                }
            }
        }
    }
}

fn lang_resolve_list(
    node: &SyntaxNode,
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
    map: &mut BindingMap,
) {
    if is_quarantined_head(node) {
        return;
    }
    let atoms = list_atoms(node);
    if atoms.is_empty() {
        return;
    }

    if let Atom::Token(head) = &atoms[0] {
        if head.kind() == SyntaxKind::Ident {
            match head.text() {
                "fn" => {
                    lang_resolve_fn_expr(&atoms[1..], stack, env, errors, map);
                    return;
                }
                "let" => {
                    lang_resolve_let(&atoms[1..], stack, env, errors, map);
                    return;
                }
                "letrec" => {
                    lang_resolve_letrec(&atoms[1..], stack, env, errors, map);
                    return;
                }
                "var" => {
                    lang_resolve_var(&atoms[1..], stack, env, errors, map);
                    return;
                }
                "set" => {
                    // (set name expr) — name is a use of a var binder.
                    for atom in &atoms[1..] {
                        lang_resolve_atom(atom, stack, env, errors, map);
                    }
                    return;
                }
                "match" => {
                    lang_resolve_match(&atoms[1..], stack, env, errors, map);
                    return;
                }
                "record" => {
                    // (record (label expr)…) — labels are static; resolve values only.
                    for atom in &atoms[1..] {
                        match atom {
                            Atom::Node(pair) if pair.kind() == SyntaxKind::List => {
                                let pair_atoms = list_atoms(pair);
                                for a in pair_atoms.iter().skip(1) {
                                    lang_resolve_atom(a, stack, env, errors, map);
                                }
                            }
                            other => lang_resolve_atom(other, stack, env, errors, map),
                        }
                    }
                    return;
                }
                "field" => {
                    // (field record label) — label is static LabelId, not a use-site.
                    if let Some(rec) = atoms.get(1) {
                        lang_resolve_atom(rec, stack, env, errors, map);
                    }
                    return;
                }
                "list" | "tuple" => {
                    for atom in &atoms[1..] {
                        lang_resolve_atom(atom, stack, env, errors, map);
                    }
                    return;
                }
                "perform" => {
                    // (perform op arg) — op is an effect name, not a value binding.
                    for atom in atoms.iter().skip(2) {
                        lang_resolve_atom(atom, stack, env, errors, map);
                    }
                    return;
                }
                "handle" => {
                    // (handle op handler body) — op is an effect name.
                    for atom in atoms.iter().skip(2) {
                        lang_resolve_atom(atom, stack, env, errors, map);
                    }
                    return;
                }
                "handler" => {
                    // (handler op (fn …)) — op is an effect name.
                    for atom in atoms.iter().skip(2) {
                        lang_resolve_atom(atom, stack, env, errors, map);
                    }
                    return;
                }
                "with" => {
                    // (with handler-expr body…) — all atoms are value positions.
                    for atom in &atoms[1..] {
                        lang_resolve_atom(atom, stack, env, errors, map);
                    }
                    return;
                }
                "if" | "seq" => {
                    for atom in &atoms[1..] {
                        lang_resolve_atom(atom, stack, env, errors, map);
                    }
                    return;
                }
                "val" | "type" | "data" => {
                    // Nested `val`/`type` are not local binders on the language
                    // surface; treat remaining atoms as expressions.
                    for atom in &atoms[1..] {
                        lang_resolve_atom(atom, stack, env, errors, map);
                    }
                    return;
                }
                _ => {}
            }
        }
    }

    // Application / other list: every atom is a use-site (or nested form).
    for atom in &atoms {
        lang_resolve_atom(atom, stack, env, errors, map);
    }
}

fn lang_resolve_fn_expr(
    rest: &[Atom],
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
    map: &mut BindingMap,
) {
    // (fn (params...) body...)
    // (fn name (params...) body...) — optional name is not bound in expression position
    let (params, body) = match rest {
        [Atom::Node(params), body @ ..] if is_param_list(params) => (params, body),
        [Atom::Token(name), Atom::Node(params), body @ ..]
            if name.kind() == SyntaxKind::Ident && is_param_list(params) =>
        {
            (params, body)
        }
        _ => {
            for atom in rest {
                lang_resolve_atom(atom, stack, env, errors, map);
            }
            return;
        }
    };
    lang_resolve_fn_body(params, body, stack, env, errors, map);
}

fn lang_resolve_var(
    rest: &[Atom],
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
    map: &mut BindingMap,
) {
    // (var name init body...)
    if rest.is_empty() {
        return;
    }
    // Resolve init before declaring (non-recursive).
    if let Some(init) = rest.get(1) {
        lang_resolve_atom(init, stack, env, errors, map);
    }
    stack.push_scope();
    if let Atom::Token(name_tok) = &rest[0] {
        if name_tok.kind() == SyntaxKind::Ident {
            declare_binding(name_tok, stack, env, errors);
        }
    }
    for atom in rest.iter().skip(2) {
        lang_resolve_atom(atom, stack, env, errors, map);
    }
    stack.pop_scope();
}

fn lang_resolve_letrec(
    rest: &[Atom],
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
    map: &mut BindingMap,
) {
    // (letrec ((name (fn …))...) body...) — all binders visible in every RHS.
    let Some(Atom::Node(bindings_node)) = rest.first() else {
        for atom in rest {
            lang_resolve_atom(atom, stack, env, errors, map);
        }
        return;
    };
    if bindings_node.kind() != SyntaxKind::List {
        for atom in rest {
            lang_resolve_atom(atom, stack, env, errors, map);
        }
        return;
    }

    stack.push_scope();
    let pairs: Vec<_> = list_atoms(bindings_node)
        .into_iter()
        .filter_map(|atom| match atom {
            Atom::Node(n) if n.kind() == SyntaxKind::List => Some(n),
            _ => None,
        })
        .collect();
    for pair in &pairs {
        let pair_atoms = list_atoms(pair);
        if let Some(Atom::Token(name_tok)) = pair_atoms.first() {
            if name_tok.kind() == SyntaxKind::Ident {
                declare_binding(name_tok, stack, env, errors);
            }
        }
    }
    for pair in &pairs {
        let pair_atoms = list_atoms(pair);
        for a in pair_atoms.iter().skip(1) {
            lang_resolve_atom(a, stack, env, errors, map);
        }
    }
    for atom in &rest[1..] {
        lang_resolve_atom(atom, stack, env, errors, map);
    }
    stack.pop_scope();
}

fn lang_resolve_match(
    rest: &[Atom],
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
    map: &mut BindingMap,
) {
    // DAT-001: (match scrutinee (pat… -> expr)…)
    if rest.is_empty() {
        return;
    }
    lang_resolve_atom(&rest[0], stack, env, errors, map);
    for arm in &rest[1..] {
        let Atom::Node(arm_node) = arm else {
            lang_resolve_atom(arm, stack, env, errors, map);
            continue;
        };
        let arm_atoms = list_atoms(arm_node);
        if arm_atoms.is_empty() {
            continue;
        }
        let arrow_idx = arm_atoms
            .iter()
            .position(|a| matches!(a, Atom::Token(t) if t.kind() == SyntaxKind::Arrow));
        let Some(arrow_idx) = arrow_idx else {
            // Missing `->`: still walk atoms so diagnostics stay useful.
            for a in &arm_atoms {
                lang_resolve_atom(a, stack, env, errors, map);
            }
            continue;
        };
        stack.push_scope();
        // Pattern before `->`: Tag | Tag binder | tuple | record — declare binders.
        let pat = &arm_atoms[..arrow_idx];
        lang_declare_pattern(pat, stack, env, errors, map);
        for body in &arm_atoms[arrow_idx + 1..] {
            lang_resolve_atom(body, stack, env, errors, map);
        }
        stack.pop_scope();
    }
}

/// Walk a match pattern and declare binders (`_`, `bind`, literals, tuple, record, ctor).
fn lang_declare_pattern(
    pat: &[Atom],
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
    map: &mut BindingMap,
) {
    if pat.is_empty() {
        return;
    }
    // Nested list as sole pattern atom.
    if pat.len() == 1 {
        if let Atom::Node(n) = &pat[0] {
            if n.kind() == SyntaxKind::List {
                lang_declare_pattern(&list_atoms(n), stack, env, errors, map);
                return;
            }
        }
    }

    let Some(Atom::Token(head)) = pat.first() else {
        for a in pat {
            lang_resolve_atom(a, stack, env, errors, map);
        }
        return;
    };
    if head.kind() != SyntaxKind::Ident {
        return;
    }
    let head_text = head.text();

    if head_text == "_" || matches!(head_text, "true" | "false" | "unit") {
        return;
    }
    if head_text == "bind" {
        if let Some(Atom::Token(b)) = pat.get(1) {
            if b.kind() == SyntaxKind::Ident {
                declare_binding(b, stack, env, errors);
            }
        }
        return;
    }
    if head_text == "tuple" {
        for atom in pat.iter().skip(1) {
            lang_declare_payload_pattern(atom, stack, env, errors, map);
        }
        return;
    }
    if head_text == "record" {
        for atom in pat.iter().skip(1) {
            let Atom::Node(pair) = atom else {
                continue;
            };
            if pair.kind() != SyntaxKind::List {
                continue;
            }
            let pa = list_atoms(pair);
            // (label pat) — label is static; second atom is the binder/pattern.
            if let Some(payload) = pa.get(1) {
                lang_declare_payload_pattern(payload, stack, env, errors, map);
            }
        }
        return;
    }

    // Constructor: resolve tag as use; remaining atoms are payload patterns.
    lang_resolve_token(head, stack, errors, map);
    for binder in pat.iter().skip(1) {
        lang_declare_payload_pattern(binder, stack, env, errors, map);
    }
}

fn lang_declare_payload_pattern(
    atom: &Atom,
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
    map: &mut BindingMap,
) {
    match atom {
        Atom::Token(b) if b.kind() == SyntaxKind::Ident => {
            if b.text() != "_" && !matches!(b.text(), "true" | "false" | "unit") {
                declare_binding(b, stack, env, errors);
            }
        }
        Atom::Token(_) | Atom::Path(_) => {}
        Atom::Node(n) if n.kind() == SyntaxKind::List => {
            lang_declare_pattern(&list_atoms(n), stack, env, errors, map);
        }
        other => lang_resolve_atom(other, stack, env, errors, map),
    }
}

fn lang_resolve_let(
    rest: &[Atom],
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
    map: &mut BindingMap,
) {
    // (let ((name expr)...) body...) — sequential (let*) so later inits see
    // earlier binders; matches Core elaborator nesting.
    let Some(Atom::Node(bindings_node)) = rest.first() else {
        for atom in rest {
            lang_resolve_atom(atom, stack, env, errors, map);
        }
        return;
    };
    if bindings_node.kind() != SyntaxKind::List {
        for atom in rest {
            lang_resolve_atom(atom, stack, env, errors, map);
        }
        return;
    }

    stack.push_scope();
    for atom in list_atoms(bindings_node) {
        let Atom::Node(pair) = atom else {
            continue;
        };
        if pair.kind() != SyntaxKind::List {
            continue;
        }
        let pair_atoms = list_atoms(&pair);
        if pair_atoms.len() < 2 {
            for a in &pair_atoms {
                lang_resolve_atom(a, stack, env, errors, map);
            }
            continue;
        }
        let Atom::Token(name_tok) = &pair_atoms[0] else {
            for a in &pair_atoms {
                lang_resolve_atom(a, stack, env, errors, map);
            }
            continue;
        };
        // Resolve init in current (already extended) scope, then declare.
        for a in &pair_atoms[1..] {
            lang_resolve_atom(a, stack, env, errors, map);
        }
        if name_tok.kind() == SyntaxKind::Ident {
            declare_binding(name_tok, stack, env, errors);
        }
    }
    for atom in &rest[1..] {
        lang_resolve_atom(atom, stack, env, errors, map);
    }
    stack.pop_scope();
}

fn lang_resolve_atom(
    atom: &Atom,
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
    map: &mut BindingMap,
) {
    match atom {
        Atom::Token(t) => lang_resolve_token(t, stack, errors, map),
        Atom::Path(path) => lang_resolve_path(path, stack, errors, map),
        Atom::Node(n) => lang_resolve_expr_node(n, stack, env, errors, map),
    }
}

fn lang_resolve_path(
    path: &str,
    stack: &mut ScopeStack,
    errors: &mut Vec<ResolveError>,
    map: &mut BindingMap,
) {
    let name = normalize_ident(path);
    if let Err(msg) = validate_ident(&name) {
        errors.push(ResolveError {
            message: msg,
            range: TextRange::EMPTY,
        });
        return;
    }
    if let Some(id) = stack.lookup(&name) {
        // No precise range for joined paths; BindingMap skip is ok.
        let _ = (id, map);
        return;
    }
    errors.push(ResolveError {
        message: format!("unbound identifier `{name}`"),
        range: TextRange::EMPTY,
    });
}

fn lang_resolve_token(
    tok: &SyntaxToken,
    stack: &ScopeStack,
    errors: &mut Vec<ResolveError>,
    map: &mut BindingMap,
) {
    if tok.kind() != SyntaxKind::Ident {
        return;
    }
    let name = normalize_ident(tok.text());
    if is_wildcard_ident(&name) {
        return;
    }
    if is_language_keyword(&name) {
        return;
    }
    if let Some(id) = stack.lookup(&name) {
        map.record(token_range(tok), id);
        return;
    }
    errors.push(ResolveError {
        message: format!("unbound identifier `{name}`"),
        range: token_range(tok),
    });
}

fn declare_binding(
    tok: &SyntaxToken,
    stack: &mut ScopeStack,
    env: &mut BindingEnv,
    errors: &mut Vec<ResolveError>,
) {
    let raw = tok.text();
    let name = normalize_ident(raw);
    if is_wildcard_ident(&name) {
        // `_` does not create a binding (SYN §3.5).
        return;
    }
    if let Err(msg) = validate_ident(&name) {
        errors.push(ResolveError {
            message: msg,
            range: token_range(tok),
        });
        return;
    }
    if is_reserved_special_form(&name) {
        errors.push(ResolveError {
            message: format!("cannot bind reserved special-form name `{name}`"),
            range: token_range(tok),
        });
        return;
    }
    let id = stack.declare(name.clone());
    env.bindings.insert(id, name);
}

fn is_language_keyword(name: &str) -> bool {
    is_reserved_special_form(name)
        || matches!(
            name,
            // KER-001 primitives (SYN §5)
            "+" | "-" | "*" | "/" | "<" | ">" | "<=" | ">=" | "=" | "!="
        )
}

fn is_param_list(node: &SyntaxNode) -> bool {
    matches!(node.kind(), SyntaxKind::List | SyntaxKind::BracketList)
}

fn is_quarantined_head(node: &SyntaxNode) -> bool {
    matches!(
        list_head_ident(node).as_deref(),
        Some(
            "page"
                | "markup"
                | "src"
                | "circle"
                | "rect"
                | "text"
                | "group"
                | "ellipse"
                | "line"
                | "translate"
                | "rotate"
                | "scale"
                | "opacity"
                | "perform"
                | "raise"
                | "handle"
        )
    )
}

fn list_atoms(node: &SyntaxNode) -> Vec<Atom> {
    let mut raw = Vec::new();
    for el in node.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) => {
                if t.kind().is_trivia()
                    || matches!(
                        t.kind(),
                        SyntaxKind::LParen
                            | SyntaxKind::RParen
                            | SyntaxKind::LBracket
                            | SyntaxKind::RBracket
                    )
                {
                    continue;
                }
                raw.push(Atom::Token(t));
            }
            SyntaxElement::Node(n) => {
                if n.kind() == SyntaxKind::StructuredComment {
                    continue;
                }
                raw.push(Atom::Node(n));
            }
        }
    }
    coalesce_slash_paths(raw, |atom| match atom {
        Atom::Token(t) if t.kind() == SyntaxKind::Ident => Some(t.text()),
        _ => None,
    })
    .into_iter()
    .map(|a| match a {
        SlashAtom::Path(p) => Atom::Path(p),
        SlashAtom::Item(item) => item,
    })
    .collect()
}

fn token_range(tok: &SyntaxToken) -> TextRange {
    let start: u32 = tok.text_range().start().into();
    let end: u32 = tok.text_range().end().into();
    TextRange::try_new(ByteOffset::new(start), ByteOffset::new(end))
        .expect("token ranges are ordered")
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
        // Markup @commands are package names per SYN-001; not unbound lisp idents.
        "markup" => {}
        // SYN-001 stubs: declare the binding name; skip deep walk of type/value bodies.
        "type" | "val" => {
            declare_named_binding(node, stack, env);
        }
        "handle" => {
            stack.push_scope();
            for child in node.children().skip(1) {
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
    let name = normalize_ident(tok.text());
    if is_surface_keyword(&name) {
        return;
    }
    if stack.lookup(&name).is_some() || env.builtin_colors.contains_key(&name) {
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
            | "markup"
            | "src"
            | "type"
            | "val"
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
            | "raise"
            | "handle"
            | "with"
            | "handler"
            | "rgb"
            | "color-byte"
            | "polyline"
            | "polygon"
            | "image"
            | "ring"
            | "frame"
    )
}

/// Declare the first argument name of `(type name …)` / `(val name …)`.
fn declare_named_binding(node: &SyntaxNode, stack: &mut ScopeStack, env: &mut BindingEnv) {
    let mut seen_head = false;
    for el in node.children_with_tokens() {
        let SyntaxElement::Token(t) = el else {
            continue;
        };
        if t.kind() != SyntaxKind::Ident {
            if !t.kind().is_trivia() && t.kind() != SyntaxKind::LParen {
                break;
            }
            continue;
        }
        if !seen_head {
            seen_head = true;
            continue;
        }
        let name = normalize_ident(t.text());
        if is_wildcard_ident(&name) {
            break;
        }
        let id = stack.declare(name.clone());
        env.bindings.insert(id, name);
        break;
    }
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
