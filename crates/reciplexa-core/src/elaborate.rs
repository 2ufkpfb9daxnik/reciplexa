//! Surface syntax → Core elaborator (BND-001 / EVAL-001 / DAT-001).
//!
//! Supports language-kernel forms only: `val` / `fn` / `let` / `if` / `seq` /
//! `data` / `match` / app / lit / perform / handle.
//! Graphics / page / markup forms are rejected (quarantined to the document pipeline).

use std::collections::HashMap;

use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;
use reciplexa_syntax::{parse_source, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

use crate::expr::{CoreExpr, CoreLiteral, MatchArm};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElaborateError {
    pub message: String,
    pub range: TextRange,
}

impl ElaborateError {
    fn new(message: impl Into<String>, range: TextRange) -> Self {
        Self {
            message: message.into(),
            range,
        }
    }

    fn at_node(message: impl Into<String>, node: &SyntaxNode) -> Self {
        Self::new(message, node_range(node))
    }

    fn at_token(message: impl Into<String>, tok: &SyntaxToken) -> Self {
        Self::new(message, token_range(tok))
    }
}

enum Atom {
    Token(SyntaxToken),
    Node(SyntaxNode),
}

/// Constructor table from top-level `(data …)` (tag → arity).
#[derive(Debug, Default, Clone)]
struct ElabCtx {
    ctors: HashMap<String, usize>,
}

/// Parse `src` and elaborate top-level `val` / `fn` / expressions to [`CoreExpr`].
///
/// Top-level bindings become nested Core `let`s. The result expression is
/// `main` when that binding exists, otherwise the last binding's name (as a
/// variable reference), or a trailing bare expression / `seq` of them.
pub fn elaborate_source(src: &str) -> Result<CoreExpr, ElaborateError> {
    let parse = parse_source(src);
    if let Some(err) = parse.errors.first() {
        return Err(ElaborateError::new(
            format!("parse error: {}", err.message),
            TextRange::try_new(
                ByteOffset::new(err.start as u32),
                ByteOffset::new(err.end as u32),
            )
            .unwrap_or(TextRange::EMPTY),
        ));
    }
    elaborate_file(&parse.root)
}

fn elaborate_file(root: &SyntaxNode) -> Result<CoreExpr, ElaborateError> {
    let mut ctx = ElabCtx::default();
    let mut bindings: Vec<(String, CoreExpr)> = Vec::new();
    let mut trailing: Vec<CoreExpr> = Vec::new();

    for el in root.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) => {
                if t.kind().is_trivia() {
                    continue;
                }
                trailing.push(elaborate_token(&t, &ctx)?);
            }
            SyntaxElement::Node(n) => match n.kind() {
                SyntaxKind::StructuredComment => continue,
                SyntaxKind::List => {
                    if list_head_ident(&n).as_deref() == Some("data") {
                        register_data(&n, &mut ctx)?;
                        continue;
                    }
                    match try_top_decl(&n, &ctx)? {
                        Some(binding) => bindings.push(binding),
                        None => {
                            if is_quarantined_head(&n) {
                                let head = list_head_ident(&n).unwrap_or_else(|| "?".into());
                                return Err(ElaborateError::at_node(
                                    format!(
                                        "language-kernel elaborator does not support `{head}` forms"
                                    ),
                                    &n,
                                ));
                            }
                            if list_head_ident(&n).as_deref() == Some("type") {
                                // Type declarations are ignored until TYP/MOD land.
                                continue;
                            }
                            trailing.push(elaborate_expr_node(&n, &ctx)?);
                        }
                    }
                }
                other => {
                    return Err(ElaborateError::at_node(
                        format!("unsupported top-level form `{other:?}`"),
                        &n,
                    ));
                }
            },
        }
    }

    if bindings.is_empty() && trailing.is_empty() {
        return Err(ElaborateError::new(
            "empty source: expected at least one form",
            TextRange::EMPTY,
        ));
    }

    let body = if trailing.is_empty() {
        let result_name = bindings
            .iter()
            .rev()
            .find(|(name, _)| name == "main")
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| bindings.last().expect("bindings non-empty").0.clone());
        CoreExpr::Var(result_name)
    } else {
        seq_or_one(trailing)
    };

    Ok(nest_lets(bindings, body))
}

fn nest_lets(bindings: Vec<(String, CoreExpr)>, body: CoreExpr) -> CoreExpr {
    bindings
        .into_iter()
        .rev()
        .fold(body, |body, (name, value)| CoreExpr::Let {
            name,
            value: Box::new(value),
            body: Box::new(body),
        })
}

/// `(data Name (Tag) (Tag payload) …)` — registers constructors; no Core binding.
fn register_data(node: &SyntaxNode, ctx: &mut ElabCtx) -> Result<(), ElaborateError> {
    let atoms = list_atoms(node);
    if atoms.len() < 3 {
        return Err(ElaborateError::at_node(
            "`data` requires a type name and at least one constructor",
            node,
        ));
    }
    let Atom::Token(name_tok) = &atoms[1] else {
        return Err(ElaborateError::at_node(
            "`data` type name must be an identifier",
            node,
        ));
    };
    if name_tok.kind() != SyntaxKind::Ident {
        return Err(ElaborateError::at_token(
            "`data` type name must be an identifier",
            name_tok,
        ));
    }
    let _type_name = name_tok.text();
    for ctor in &atoms[2..] {
        match ctor {
            Atom::Token(t) if t.kind() == SyntaxKind::Ident => {
                ctx.ctors.insert(t.text().to_string(), 0);
            }
            Atom::Node(n) if n.kind() == SyntaxKind::List => {
                let ca = list_atoms(n);
                if ca.is_empty() {
                    return Err(ElaborateError::at_node(
                        "`data` constructor list must not be empty",
                        n,
                    ));
                }
                let Atom::Token(tag_tok) = &ca[0] else {
                    return Err(ElaborateError::at_node(
                        "`data` constructor tag must be an identifier",
                        n,
                    ));
                };
                if tag_tok.kind() != SyntaxKind::Ident {
                    return Err(ElaborateError::at_token(
                        "`data` constructor tag must be an identifier",
                        tag_tok,
                    ));
                }
                let arity = ca.len() - 1;
                if arity > 1 {
                    return Err(ElaborateError::at_node(
                        "DAT-001 v0 supports at most one payload per constructor",
                        n,
                    ));
                }
                for payload in &ca[1..] {
                    match payload {
                        Atom::Token(p) if p.kind() == SyntaxKind::Ident => {}
                        Atom::Token(p) => {
                            return Err(ElaborateError::at_token(
                                "constructor payload binder must be an identifier",
                                p,
                            ));
                        }
                        Atom::Node(pn) => {
                            return Err(ElaborateError::at_node(
                                "constructor payload binder must be an identifier",
                                pn,
                            ));
                        }
                    }
                }
                ctx.ctors.insert(tag_tok.text().to_string(), arity);
            }
            Atom::Token(t) => {
                return Err(ElaborateError::at_token(
                    "`data` constructor must be an identifier or `(Tag …)`",
                    t,
                ));
            }
            Atom::Node(n) => {
                return Err(ElaborateError::at_node(
                    "`data` constructor must be an identifier or `(Tag …)`",
                    n,
                ));
            }
        }
    }
    Ok(())
}

fn try_top_decl(
    node: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<Option<(String, CoreExpr)>, ElaborateError> {
    let atoms = list_atoms(node);
    let Some(Atom::Token(head)) = atoms.first() else {
        return Ok(None);
    };
    if head.kind() != SyntaxKind::Ident {
        return Ok(None);
    }
    match head.text() {
        "val" => Ok(Some(elaborate_val(&atoms[1..], node, ctx)?)),
        "fn" => {
            // Top-level named function: (fn name (params...) body...)
            if atoms.len() >= 3 {
                if let (Atom::Token(name_tok), Atom::Node(params)) = (&atoms[1], &atoms[2]) {
                    if name_tok.kind() == SyntaxKind::Ident && is_param_list(params) {
                        let params = elaborate_params(params)?;
                        let body = elaborate_body(&atoms[3..], node, ctx)?;
                        return Ok(Some((
                            name_tok.text().to_string(),
                            CoreExpr::Lambda {
                                params,
                                body: Box::new(body),
                            },
                        )));
                    }
                }
            }
            // Anonymous `(fn (params...) body...)` is an expression, not a decl.
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn elaborate_val(
    rest: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<(String, CoreExpr), ElaborateError> {
    if rest.is_empty() {
        return Err(ElaborateError::at_node(
            "`val` requires a name and expression",
            parent,
        ));
    }
    match &rest[0] {
        Atom::Token(name_tok) if name_tok.kind() == SyntaxKind::Ident => {
            if rest.len() < 2 {
                return Err(ElaborateError::at_token(
                    "`val` requires an initializer expression",
                    name_tok,
                ));
            }
            let value = elaborate_body(&rest[1..], parent, ctx)?;
            Ok((name_tok.text().to_string(), value))
        }
        // (val (name params...) body...) named-function sugar
        Atom::Node(binder) if binder.kind() == SyntaxKind::List => {
            let binder_atoms = list_atoms(binder);
            let Some(Atom::Token(name_tok)) = binder_atoms.first() else {
                return Err(ElaborateError::at_node(
                    "`val` binder list requires a function name",
                    binder,
                ));
            };
            if name_tok.kind() != SyntaxKind::Ident {
                return Err(ElaborateError::at_token(
                    "`val` binder list requires a function name",
                    name_tok,
                ));
            }
            let mut params = Vec::new();
            for atom in &binder_atoms[1..] {
                match atom {
                    Atom::Token(t) if t.kind() == SyntaxKind::Ident => {
                        params.push(t.text().to_string());
                    }
                    Atom::Token(t) => {
                        return Err(ElaborateError::at_token(
                            "function parameter must be an identifier",
                            t,
                        ));
                    }
                    Atom::Node(n) => {
                        return Err(ElaborateError::at_node(
                            "function parameter must be an identifier",
                            n,
                        ));
                    }
                }
            }
            let body = elaborate_body(&rest[1..], parent, ctx)?;
            Ok((
                name_tok.text().to_string(),
                CoreExpr::Lambda {
                    params,
                    body: Box::new(body),
                },
            ))
        }
        Atom::Token(t) => Err(ElaborateError::at_token(
            "`val` name must be an identifier or `(name params...)` binder",
            t,
        )),
        Atom::Node(n) => Err(ElaborateError::at_node(
            "`val` name must be an identifier or `(name params...)` binder",
            n,
        )),
    }
}

fn elaborate_expr_node(node: &SyntaxNode, ctx: &ElabCtx) -> Result<CoreExpr, ElaborateError> {
    match node.kind() {
        SyntaxKind::List => elaborate_list(node, ctx),
        other => Err(ElaborateError::at_node(
            format!("expected expression list, got `{other:?}`"),
            node,
        )),
    }
}

fn elaborate_list(node: &SyntaxNode, ctx: &ElabCtx) -> Result<CoreExpr, ElaborateError> {
    let atoms = list_atoms(node);
    if atoms.is_empty() {
        return Err(ElaborateError::at_node(
            "empty list is not an expression",
            node,
        ));
    }

    if let Atom::Token(head) = &atoms[0] {
        if head.kind() == SyntaxKind::Ident {
            match head.text() {
                "fn" => return elaborate_fn_expr(&atoms[1..], node, ctx),
                "let" => return elaborate_let(&atoms[1..], node, ctx),
                "if" => return elaborate_if(&atoms[1..], node, ctx),
                "match" => return elaborate_match(&atoms[1..], node, ctx),
                "seq" => {
                    if atoms.len() < 2 {
                        return Err(ElaborateError::at_node(
                            "`seq` requires at least one expression",
                            node,
                        ));
                    }
                    return Ok(seq_or_one(elaborate_atoms(&atoms[1..], ctx)?));
                }
                "perform" => return elaborate_perform(&atoms[1..], node, ctx),
                "handle" => return elaborate_handle(&atoms[1..], node, ctx),
                "val" => {
                    return Err(ElaborateError::at_node(
                        "`val` is only allowed at top level; use `let` for local bindings",
                        node,
                    ));
                }
                "data" => {
                    return Err(ElaborateError::at_node(
                        "`data` is only allowed at top level",
                        node,
                    ));
                }
                tag if ctx.ctors.contains_key(tag) => {
                    let arity = ctx.ctors[tag];
                    if atoms.len() - 1 != arity {
                        return Err(ElaborateError::at_node(
                            format!("constructor `{tag}` expects {arity} payload(s)"),
                            node,
                        ));
                    }
                    let payload = if arity == 0 {
                        None
                    } else {
                        Some(Box::new(elaborate_atom(&atoms[1], ctx)?))
                    };
                    return Ok(CoreExpr::Variant {
                        tag: tag.to_string(),
                        payload,
                    });
                }
                _ => {}
            }
        }
    }

    // Application: (f a b ...) — operator first, then args left-to-right.
    let fun = elaborate_atom(&atoms[0], ctx)?;
    let args = elaborate_atoms(&atoms[1..], ctx)?;
    Ok(CoreExpr::App {
        fun: Box::new(fun),
        args,
    })
}

fn elaborate_match(
    rest: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    // (match scrutinee (pattern body...)...)
    // Patterns: Tag | (Tag) | (Tag binder)
    if rest.len() < 2 {
        return Err(ElaborateError::at_node(
            "`match` requires a scrutinee and at least one arm",
            parent,
        ));
    }
    let scrutinee = elaborate_atom(&rest[0], ctx)?;
    let mut arms = Vec::new();
    for arm_atom in &rest[1..] {
        let Atom::Node(arm_node) = arm_atom else {
            return Err(ElaborateError::at_node(
                "`match` arm must be `(pattern body...)`",
                parent,
            ));
        };
        if arm_node.kind() != SyntaxKind::List {
            return Err(ElaborateError::at_node(
                "`match` arm must be `(pattern body...)`",
                arm_node,
            ));
        }
        let arm_atoms = list_atoms(arm_node);
        if arm_atoms.len() < 2 {
            return Err(ElaborateError::at_node(
                "`match` arm must be `(pattern body...)`",
                arm_node,
            ));
        }
        let (tag, bind) = elaborate_pattern(&arm_atoms[0], arm_node)?;
        let body = elaborate_body(&arm_atoms[1..], arm_node, ctx)?;
        arms.push(MatchArm { tag, bind, body });
    }
    Ok(CoreExpr::Match {
        scrutinee: Box::new(scrutinee),
        arms,
    })
}

fn elaborate_pattern(
    atom: &Atom,
    parent: &SyntaxNode,
) -> Result<(String, Option<String>), ElaborateError> {
    match atom {
        Atom::Token(t) if t.kind() == SyntaxKind::Ident => Ok((t.text().to_string(), None)),
        Atom::Node(n) if n.kind() == SyntaxKind::List => {
            let atoms = list_atoms(n);
            if atoms.is_empty() {
                return Err(ElaborateError::at_node(
                    "match pattern list must not be empty",
                    n,
                ));
            }
            let Atom::Token(tag_tok) = &atoms[0] else {
                return Err(ElaborateError::at_node(
                    "match pattern tag must be an identifier",
                    n,
                ));
            };
            if tag_tok.kind() != SyntaxKind::Ident {
                return Err(ElaborateError::at_token(
                    "match pattern tag must be an identifier",
                    tag_tok,
                ));
            }
            let bind = match atoms.get(1) {
                None => None,
                Some(Atom::Token(b)) if b.kind() == SyntaxKind::Ident => {
                    if atoms.len() > 2 {
                        return Err(ElaborateError::at_node(
                            "DAT-001 v0 patterns support at most one binder",
                            n,
                        ));
                    }
                    Some(b.text().to_string())
                }
                Some(Atom::Token(b)) => {
                    return Err(ElaborateError::at_token(
                        "match pattern binder must be an identifier",
                        b,
                    ));
                }
                Some(Atom::Node(bn)) => {
                    return Err(ElaborateError::at_node(
                        "match pattern binder must be an identifier",
                        bn,
                    ));
                }
            };
            Ok((tag_tok.text().to_string(), bind))
        }
        Atom::Token(t) => Err(ElaborateError::at_token(
            "match pattern must be a tag or `(Tag binder)`",
            t,
        )),
        Atom::Node(_) => Err(ElaborateError::at_node(
            "match pattern must be a tag or `(Tag binder)`",
            parent,
        )),
    }
}

fn elaborate_perform(
    rest: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    // (perform op arg)
    if rest.len() != 2 {
        return Err(ElaborateError::at_node(
            "`perform` requires an op identifier and one argument",
            parent,
        ));
    }
    let Atom::Token(op_tok) = &rest[0] else {
        return Err(ElaborateError::at_node(
            "`perform` op must be an identifier",
            parent,
        ));
    };
    if op_tok.kind() != SyntaxKind::Ident {
        return Err(ElaborateError::at_token(
            "`perform` op must be an identifier",
            op_tok,
        ));
    }
    Ok(CoreExpr::Perform {
        op: op_tok.text().to_string(),
        arg: Box::new(elaborate_atom(&rest[1], ctx)?),
    })
}

fn elaborate_handle(
    rest: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    // (handle op (fn (params...) handler-body...) body)
    if rest.len() != 3 {
        return Err(ElaborateError::at_node(
            "`handle` requires op, handler fn, and body",
            parent,
        ));
    }
    let Atom::Token(op_tok) = &rest[0] else {
        return Err(ElaborateError::at_node(
            "`handle` op must be an identifier",
            parent,
        ));
    };
    if op_tok.kind() != SyntaxKind::Ident {
        return Err(ElaborateError::at_token(
            "`handle` op must be an identifier",
            op_tok,
        ));
    }
    let handler_expr = elaborate_atom(&rest[1], ctx)?;
    let CoreExpr::Lambda { params, body } = handler_expr else {
        return Err(ElaborateError::at_node(
            "`handle` handler must be `(fn (params...) ...)`",
            parent,
        ));
    };
    if !(1..=2).contains(&params.len()) {
        return Err(ElaborateError::at_node(
            "`handle` handler expects 1 or 2 parameters (arg) or (arg resume)",
            parent,
        ));
    }
    Ok(CoreExpr::Handle {
        op: op_tok.text().to_string(),
        handler_params: params,
        handler_body: body,
        body: Box::new(elaborate_atom(&rest[2], ctx)?),
    })
}

fn elaborate_fn_expr(
    rest: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    // (fn (params...) body...)
    // (fn name (params...) body...) — name ignored in expression position (produces lambda)
    let (params_node, body_atoms) = match rest {
        [Atom::Node(params), body @ ..] if is_param_list(params) => (params, body),
        [Atom::Token(name), Atom::Node(params), body @ ..]
            if name.kind() == SyntaxKind::Ident && is_param_list(params) =>
        {
            let _ = name;
            (params, body)
        }
        _ => {
            return Err(ElaborateError::at_node(
                "`fn` expects `(params...)` then one or more body expressions",
                parent,
            ));
        }
    };
    let params = elaborate_params(params_node)?;
    let body = elaborate_body(body_atoms, parent, ctx)?;
    Ok(CoreExpr::Lambda {
        params,
        body: Box::new(body),
    })
}

fn elaborate_let(
    rest: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    // (let ((name expr)...) body...)
    let Some(Atom::Node(bindings_node)) = rest.first() else {
        return Err(ElaborateError::at_node(
            "`let` requires a binding list",
            parent,
        ));
    };
    if bindings_node.kind() != SyntaxKind::List {
        return Err(ElaborateError::at_node(
            "`let` binding list must be a parenthesized list",
            bindings_node,
        ));
    }
    let binding_atoms = list_atoms(bindings_node);
    if binding_atoms.is_empty() {
        return Err(ElaborateError::at_node(
            "`let` binding list must not be empty",
            bindings_node,
        ));
    }

    let mut bindings = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for atom in &binding_atoms {
        let pair = match atom {
            Atom::Node(n) => n,
            Atom::Token(t) => {
                return Err(ElaborateError::at_token(
                    "`let` binding must be `(name expr)`",
                    t,
                ));
            }
        };
        if pair.kind() != SyntaxKind::List {
            return Err(ElaborateError::at_node(
                "`let` binding must be `(name expr)`",
                pair,
            ));
        }
        let pair_atoms = list_atoms(pair);
        if pair_atoms.len() != 2 {
            return Err(ElaborateError::at_node(
                "`let` binding must be `(name expr)`",
                pair,
            ));
        }
        let Atom::Token(name_tok) = &pair_atoms[0] else {
            return Err(ElaborateError::at_node(
                "`let` binder must be an identifier",
                pair,
            ));
        };
        if name_tok.kind() != SyntaxKind::Ident {
            return Err(ElaborateError::at_token(
                "`let` binder must be an identifier",
                name_tok,
            ));
        }
        let name = name_tok.text().to_string();
        if !seen.insert(name.clone()) {
            return Err(ElaborateError::at_token(
                format!("duplicate binder `{name}` in the same `let`"),
                name_tok,
            ));
        }
        let value = elaborate_atom(&pair_atoms[1], ctx)?;
        bindings.push((name, value));
    }

    let body = elaborate_body(&rest[1..], parent, ctx)?;
    Ok(nest_lets(bindings, body))
}

fn elaborate_if(
    rest: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    if rest.len() != 3 {
        return Err(ElaborateError::at_node(
            "`if` requires exactly three expressions: condition, then, else",
            parent,
        ));
    }
    Ok(CoreExpr::If {
        cond: Box::new(elaborate_atom(&rest[0], ctx)?),
        then_branch: Box::new(elaborate_atom(&rest[1], ctx)?),
        else_branch: Box::new(elaborate_atom(&rest[2], ctx)?),
    })
}

fn elaborate_body(
    atoms: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    if atoms.is_empty() {
        return Err(ElaborateError::at_node(
            "body requires at least one expression",
            parent,
        ));
    }
    Ok(seq_or_one(elaborate_atoms(atoms, ctx)?))
}

fn elaborate_atoms(atoms: &[Atom], ctx: &ElabCtx) -> Result<Vec<CoreExpr>, ElaborateError> {
    atoms.iter().map(|a| elaborate_atom(a, ctx)).collect()
}

fn elaborate_atom(atom: &Atom, ctx: &ElabCtx) -> Result<CoreExpr, ElaborateError> {
    match atom {
        Atom::Token(t) => elaborate_token(t, ctx),
        Atom::Node(n) => elaborate_expr_node(n, ctx),
    }
}

fn elaborate_token(tok: &SyntaxToken, ctx: &ElabCtx) -> Result<CoreExpr, ElaborateError> {
    match tok.kind() {
        SyntaxKind::Number => {
            let text = tok.text();
            let n: f64 = text.parse().map_err(|_| {
                ElaborateError::at_token(format!("invalid number literal `{text}`"), tok)
            })?;
            Ok(CoreExpr::Lit(CoreLiteral::Number(n)))
        }
        SyntaxKind::String => {
            let raw = tok.text();
            if raw.len() < 2 || !raw.starts_with('"') || !raw.ends_with('"') {
                return Err(ElaborateError::at_token("malformed string literal", tok));
            }
            Ok(CoreExpr::Lit(CoreLiteral::String(unescape_string(
                &raw[1..raw.len() - 1],
            ))))
        }
        SyntaxKind::Ident => match tok.text() {
            "true" => Ok(CoreExpr::Lit(CoreLiteral::Bool(true))),
            "false" => Ok(CoreExpr::Lit(CoreLiteral::Bool(false))),
            name if ctx.ctors.get(name) == Some(&0) => Ok(CoreExpr::Variant {
                tag: name.to_string(),
                payload: None,
            }),
            name => Ok(CoreExpr::Var(name.to_string())),
        },
        other => Err(ElaborateError::at_token(
            format!("unexpected token `{other:?}` in expression"),
            tok,
        )),
    }
}

fn elaborate_params(params: &SyntaxNode) -> Result<Vec<String>, ElaborateError> {
    let mut out = Vec::new();
    for atom in list_atoms(params) {
        match atom {
            Atom::Token(t) if t.kind() == SyntaxKind::Ident => {
                out.push(t.text().to_string());
            }
            Atom::Token(t) => {
                return Err(ElaborateError::at_token(
                    "function parameter must be an identifier",
                    &t,
                ));
            }
            Atom::Node(n) => {
                return Err(ElaborateError::at_node(
                    "function parameter must be an identifier",
                    &n,
                ));
            }
        }
    }
    Ok(out)
}

fn is_param_list(node: &SyntaxNode) -> bool {
    matches!(node.kind(), SyntaxKind::List | SyntaxKind::BracketList)
}

fn is_quarantined_head(node: &SyntaxNode) -> bool {
    matches!(
        list_head_ident(node).as_deref(),
        Some("page" | "markup" | "src" | "circle" | "rect" | "text" | "group")
    )
}

fn seq_or_one(exprs: Vec<CoreExpr>) -> CoreExpr {
    match exprs.len() {
        1 => exprs.into_iter().next().expect("len checked"),
        _ => CoreExpr::Seq(exprs),
    }
}

fn list_atoms(node: &SyntaxNode) -> Vec<Atom> {
    let mut items = Vec::new();
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
                items.push(Atom::Token(t));
            }
            SyntaxElement::Node(n) => {
                if n.kind() == SyntaxKind::StructuredComment {
                    continue;
                }
                items.push(Atom::Node(n));
            }
        }
    }
    items
}

fn list_head_ident(node: &SyntaxNode) -> Option<String> {
    for atom in list_atoms(node) {
        if let Atom::Token(t) = atom {
            if t.kind() == SyntaxKind::Ident {
                return Some(t.text().to_string());
            }
            return None;
        }
    }
    None
}

fn node_range(node: &SyntaxNode) -> TextRange {
    let start: u32 = node.text_range().start().into();
    let end: u32 = node.text_range().end().into();
    TextRange::try_new(ByteOffset::new(start), ByteOffset::new(end)).unwrap_or(TextRange::EMPTY)
}

fn token_range(tok: &SyntaxToken) -> TextRange {
    let start: u32 = tok.text_range().start().into();
    let end: u32 = tok.text_range().end().into();
    TextRange::try_new(ByteOffset::new(start), ByteOffset::new(end)).unwrap_or(TextRange::EMPTY)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elaborates_identity_application() {
        let expr = elaborate_source("(val main ((fn (x) x) 42))").unwrap();
        match expr {
            CoreExpr::Let { name, body, .. } => {
                assert_eq!(name, "main");
                assert!(matches!(*body, CoreExpr::Var(ref n) if n == "main"));
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn rejects_page_forms() {
        let err = elaborate_source("(page a4)").unwrap_err();
        assert!(err.message.contains("page"));
    }

    #[test]
    fn elaborates_data_and_match() {
        let expr = elaborate_source(
            r#"
(data Option (None) (Some x))
(val main (match (Some 1) (None 0) ((Some x) x)))
"#,
        )
        .unwrap();
        let CoreExpr::Let { value, .. } = expr else {
            panic!("expected Let");
        };
        assert!(matches!(*value, CoreExpr::Match { .. }));
    }
}
