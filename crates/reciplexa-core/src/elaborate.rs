//! Surface syntax → Core elaborator (BND-001 / EVAL-001 / DAT-001).
//!
//! Supports language-kernel forms only: `val` / `fn` / `let` / `letrec` / `var` /
//! `set` / `if` / `seq` / `data` / `match` / `record` / `field` / `list` / `tuple` /
//! `local` / `rec` / app / lit / perform / handle / handler / with.
//! Graphics / page / markup forms are rejected (quarantined to the document pipeline).

use std::collections::HashMap;

use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;
use reciplexa_syntax::{
    decode_string_literal, is_reserved_special_form, parse_number_literal, parse_source,
    SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken,
};

use crate::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};

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

/// Constructor / ADT table from top-level `(data …)` (for exhaustiveness).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DataEnv {
    /// tag → arity
    pub ctors: HashMap<String, usize>,
    /// tag → owning data type name
    pub ctor_type: HashMap<String, String>,
    /// type name → ordered constructors `(tag, arity)`
    pub data_ctors: HashMap<String, Vec<(String, usize)>>,
}

impl DataEnv {
    /// Full constructor list for a tag's ADT, if known.
    pub fn adt_for_tag(&self, tag: &str) -> Vec<(String, usize)> {
        self.ctor_type
            .get(tag)
            .and_then(|ty| self.data_ctors.get(ty))
            .cloned()
            .unwrap_or_default()
    }
}

/// Constructor table from top-level `(data …)`.
#[derive(Debug, Default, Clone)]
struct ElabCtx {
    data: DataEnv,
}

/// Parse `src` and elaborate top-level `val` / `fn` / expressions to [`CoreExpr`].
///
/// Top-level bindings become nested Core `let`s. The result expression is
/// `main` when that binding exists, otherwise the last binding's name (as a
/// variable reference), or a trailing bare expression / `seq` of them.
pub fn elaborate_source(src: &str) -> Result<CoreExpr, ElaborateError> {
    Ok(elaborate_with_data(src)?.0)
}

/// Elaborate source and return the ADT/`data` environment for typechecking.
pub fn elaborate_with_data(src: &str) -> Result<(CoreExpr, DataEnv), ElaborateError> {
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
    let (expr, ctx) = elaborate_file(&parse.root)?;
    Ok((expr, ctx.data))
}

fn elaborate_file(root: &SyntaxNode) -> Result<(CoreExpr, ElabCtx), ElaborateError> {
    let mut ctx = ElabCtx::default();
    let mut bindings: Vec<TopBinding> = Vec::new();
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
            .find_map(|b| match b {
                TopBinding::Single(name, _) if name == "main" => Some(name.clone()),
                TopBinding::Rec(bs) => bs
                    .iter()
                    .rev()
                    .find(|(n, _)| n == "main")
                    .map(|(n, _)| n.clone()),
                _ => None,
            })
            .or_else(|| match bindings.last() {
                Some(TopBinding::Single(name, _)) => Some(name.clone()),
                Some(TopBinding::Rec(bs)) => bs.last().map(|(n, _)| n.clone()),
                None => None,
            })
            .expect("bindings non-empty when trailing empty");
        CoreExpr::Var(result_name)
    } else {
        seq_or_one(trailing)
    };

    Ok((nest_top_bindings(bindings, body), ctx))
}

enum TopBinding {
    Single(String, CoreExpr),
    Rec(Vec<(String, CoreExpr)>),
}

fn nest_top_bindings(bindings: Vec<TopBinding>, body: CoreExpr) -> CoreExpr {
    bindings.into_iter().rev().fold(body, |body, b| match b {
        TopBinding::Single(name, value) => CoreExpr::Let {
            name,
            value: Box::new(value),
            body: Box::new(body),
        },
        TopBinding::Rec(recs) => CoreExpr::LetRec {
            bindings: recs,
            body: Box::new(body),
        },
    })
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
    let type_name = name_tok.text().to_string();
    let mut ctors = Vec::new();
    for ctor in &atoms[2..] {
        match ctor {
            Atom::Token(t) if t.kind() == SyntaxKind::Ident => {
                let tag = t.text().to_string();
                ctx.data.ctors.insert(tag.clone(), 0);
                ctx.data.ctor_type.insert(tag.clone(), type_name.clone());
                ctors.push((tag, 0));
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
                let tag = tag_tok.text().to_string();
                ctx.data.ctors.insert(tag.clone(), arity);
                ctx.data.ctor_type.insert(tag.clone(), type_name.clone());
                ctors.push((tag, arity));
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
    ctx.data.data_ctors.insert(type_name, ctors);
    Ok(())
}

fn try_top_decl(node: &SyntaxNode, ctx: &ElabCtx) -> Result<Option<TopBinding>, ElaborateError> {
    let atoms = list_atoms(node);
    let Some(Atom::Token(head)) = atoms.first() else {
        return Ok(None);
    };
    if head.kind() != SyntaxKind::Ident {
        return Ok(None);
    }
    match head.text() {
        "val" => {
            let (name, value) = elaborate_val(&atoms[1..], node, ctx)?;
            Ok(Some(TopBinding::Single(name, value)))
        }
        "rec" => {
            // SYN §13.4: top-level `rec` is a declaration group (no result expr).
            let bindings = parse_rec_val_bindings(&atoms[1..], node, ctx)?;
            if bindings.is_empty() {
                return Err(ElaborateError::at_node(
                    "`rec` requires at least one `(val …)` binding",
                    node,
                ));
            }
            Ok(Some(TopBinding::Rec(bindings)))
        }
        "fn" => {
            // Top-level named function: (fn name (params...) body...)
            if atoms.len() >= 3 {
                if let (Atom::Token(name_tok), Atom::Node(params)) = (&atoms[1], &atoms[2]) {
                    if name_tok.kind() == SyntaxKind::Ident && is_param_list(params) {
                        if is_reserved_special_form(name_tok.text()) {
                            return Err(ElaborateError::at_token(
                                format!(
                                    "cannot bind reserved special-form name `{}`",
                                    name_tok.text()
                                ),
                                name_tok,
                            ));
                        }
                        let params = elaborate_params(params)?;
                        let body = elaborate_body(&atoms[3..], node, ctx)?;
                        return Ok(Some(TopBinding::Single(
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
            if is_reserved_special_form(name_tok.text()) {
                return Err(ElaborateError::at_token(
                    format!(
                        "cannot bind reserved special-form name `{}`",
                        name_tok.text()
                    ),
                    name_tok,
                ));
            }
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
            if is_reserved_special_form(name_tok.text()) {
                return Err(ElaborateError::at_token(
                    format!(
                        "cannot bind reserved special-form name `{}`",
                        name_tok.text()
                    ),
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
                "letrec" => return elaborate_letrec(&atoms[1..], node, ctx),
                "local" => return elaborate_local(&atoms[1..], node, ctx),
                "rec" => return elaborate_rec(&atoms[1..], node, ctx),
                "var" => return elaborate_var(&atoms[1..], node, ctx),
                "set" => return elaborate_set(&atoms[1..], node, ctx),
                "if" => return elaborate_if(&atoms[1..], node, ctx),
                "match" => return elaborate_match(&atoms[1..], node, ctx),
                "record" => return elaborate_record(&atoms[1..], node, ctx),
                "field" => return elaborate_field(&atoms[1..], node, ctx),
                "list" => return elaborate_list_lit(&atoms[1..], ctx),
                "tuple" => return elaborate_tuple(&atoms[1..], node, ctx),
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
                "handler" => return elaborate_handler(&atoms[1..], node, ctx),
                "with" => return elaborate_with(&atoms[1..], node, ctx),
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
                tag if ctx.data.ctors.contains_key(tag) => {
                    let arity = ctx.data.ctors[tag];
                    if atoms.len() - 1 != arity {
                        return Err(ElaborateError::at_node(
                            format!("constructor `{tag}` expects {arity} payload(s)"),
                            node,
                        ));
                    }
                    let payload = match arity {
                        0 => None,
                        1 => Some(Box::new(elaborate_atom(&atoms[1], ctx)?)),
                        _ => {
                            let mut fields = Vec::with_capacity(arity);
                            for (i, atom) in atoms[1..].iter().enumerate() {
                                fields.push((i.to_string(), elaborate_atom(atom, ctx)?));
                            }
                            Some(Box::new(CoreExpr::Record { fields }))
                        }
                    };
                    return Ok(CoreExpr::Variant {
                        tag: tag.to_string(),
                        payload,
                    });
                }
                // EFF-001 ambient effect ops: `(log "msg")` / `(random)` → Perform
                tag if is_ambient_effect_op(tag) => {
                    return elaborate_ambient_perform(tag, &atoms[1..], node, ctx);
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

/// EFF-001 ambient ops callable without `perform` (DD-EFF-013).
fn is_ambient_effect_op(name: &str) -> bool {
    matches!(
        name,
        "log" | "random" | "read-file" | "write-file" | "write-path" | "load-image"
    )
}

fn elaborate_ambient_perform(
    op: &str,
    args: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    let arg = match (op, args.len()) {
        ("random", 0) => CoreExpr::Lit(CoreLiteral::Unit),
        ("random", _) => {
            return Err(ElaborateError::at_node(
                "ambient `(random)` takes no arguments",
                parent,
            ));
        }
        (_, 1) => elaborate_atom(&args[0], ctx)?,
        (_, n) => {
            return Err(ElaborateError::at_node(
                format!("ambient `({op} …)` expects exactly one argument, got {n}"),
                parent,
            ));
        }
    };
    Ok(CoreExpr::Perform {
        op: op.to_string(),
        arg: Box::new(arg),
    })
}

/// SYN §15.3: `(record (label expr)…)` → [`CoreExpr::Record`].
fn elaborate_record(
    rest: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    let mut fields = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for atom in rest {
        let Atom::Node(pair) = atom else {
            return Err(ElaborateError::at_node(
                "`record` field must be `(label expr)`",
                parent,
            ));
        };
        if pair.kind() != SyntaxKind::List {
            return Err(ElaborateError::at_node(
                "`record` field must be `(label expr)`",
                pair,
            ));
        }
        let pair_atoms = list_atoms(pair);
        if pair_atoms.len() != 2 {
            return Err(ElaborateError::at_node(
                "`record` field must be `(label expr)`",
                pair,
            ));
        }
        let Atom::Token(label_tok) = &pair_atoms[0] else {
            return Err(ElaborateError::at_node(
                "`record` field label must be an identifier",
                pair,
            ));
        };
        if label_tok.kind() != SyntaxKind::Ident {
            return Err(ElaborateError::at_token(
                "`record` field label must be an identifier",
                label_tok,
            ));
        }
        let label = label_tok.text().to_string();
        if !seen.insert(label.clone()) {
            return Err(ElaborateError::at_token(
                format!("duplicate record field `{label}`"),
                label_tok,
            ));
        }
        let value = elaborate_atom(&pair_atoms[1], ctx)?;
        fields.push((label, value));
    }
    Ok(CoreExpr::Record { fields })
}

/// SYN §15.4: `(field record label)` → [`CoreExpr::RecordGet`].
fn elaborate_field(
    rest: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    if rest.len() != 2 {
        return Err(ElaborateError::at_node(
            "`field` requires a record expression and a static label",
            parent,
        ));
    }
    let record = elaborate_atom(&rest[0], ctx)?;
    let Atom::Token(label_tok) = &rest[1] else {
        return Err(ElaborateError::at_node(
            "`field` label must be an identifier (static LabelId)",
            parent,
        ));
    };
    if label_tok.kind() != SyntaxKind::Ident {
        return Err(ElaborateError::at_token(
            "`field` label must be an identifier (static LabelId)",
            label_tok,
        ));
    }
    Ok(CoreExpr::RecordGet {
        record: Box::new(record),
        field: label_tok.text().to_string(),
    })
}

/// SYN §15.1 list encoding: nested variants `Cons` / `Nil`.
/// `Cons` payload is a 2-field record `("head", x) ("tail", rest)`.
fn elaborate_list_lit(rest: &[Atom], ctx: &ElabCtx) -> Result<CoreExpr, ElaborateError> {
    let mut acc = CoreExpr::Variant {
        tag: "Nil".into(),
        payload: None,
    };
    for atom in rest.iter().rev() {
        let head = elaborate_atom(atom, ctx)?;
        acc = CoreExpr::Variant {
            tag: "Cons".into(),
            payload: Some(Box::new(CoreExpr::Record {
                fields: vec![("head".into(), head), ("tail".into(), acc)],
            })),
        };
    }
    Ok(acc)
}

/// SYN §15.2 tuple encoding: closed record with positional labels `"0"`, `"1"`, …
/// - 0 elems → `unit`
/// - 1 elem → the element itself
/// - 2+ → [`CoreExpr::Record`]
fn elaborate_tuple(
    rest: &[Atom],
    _parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    match rest.len() {
        0 => Ok(CoreExpr::Lit(CoreLiteral::Unit)),
        1 => elaborate_atom(&rest[0], ctx),
        _ => {
            let mut fields = Vec::with_capacity(rest.len());
            for (i, atom) in rest.iter().enumerate() {
                fields.push((i.to_string(), elaborate_atom(atom, ctx)?));
            }
            Ok(CoreExpr::Record { fields })
        }
    }
}

fn elaborate_match(
    rest: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    // DAT-001 §14: (match scrutinee (pat… -> expr)…)
    // Patterns before `->`: Tag | Tag binder
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
                "`match` arm must be `(pattern -> expression)`",
                parent,
            ));
        };
        if arm_node.kind() != SyntaxKind::List {
            return Err(ElaborateError::at_node(
                "`match` arm must be `(pattern -> expression)`",
                arm_node,
            ));
        }
        let arm_atoms = list_atoms(arm_node);
        let arrow_idx = arm_atoms
            .iter()
            .position(|a| matches!(a, Atom::Token(t) if t.kind() == SyntaxKind::Arrow));
        let Some(arrow_idx) = arrow_idx else {
            return Err(ElaborateError::at_node(
                "`match` arm must use `->` between pattern and expression (DAT-001); \
                 old `(pattern body)` form is rejected",
                arm_node,
            ));
        };
        if arrow_idx == 0 {
            return Err(ElaborateError::at_node(
                "`match` arm requires a pattern before `->`",
                arm_node,
            ));
        }
        let body_atoms = &arm_atoms[arrow_idx + 1..];
        if body_atoms.is_empty() {
            return Err(ElaborateError::at_node(
                "`match` arm requires an expression after `->`",
                arm_node,
            ));
        }
        if body_atoms.len() > 1 {
            return Err(ElaborateError::at_node(
                "`match` arm body must be a single expression after `->`; \
                 wrap multiple forms in `seq` (DAT-001 §14.5)",
                arm_node,
            ));
        }
        let pattern = elaborate_pattern_atoms(&arm_atoms[..arrow_idx], arm_node)?;
        let body = elaborate_atom(&body_atoms[0], ctx)?;
        arms.push(MatchArm { pattern, body });
    }
    // Static exhaustiveness from known `(data …)` constructors (via arm tags
    // or a direct constructor scrutinee). Catch-all `_` / `bind` cover all.
    let has_catch_all = arms.iter().any(|a| a.is_catch_all());
    if !has_catch_all {
        let adt = if let CoreExpr::Variant { tag, .. } = &scrutinee {
            ctx.data.adt_for_tag(tag)
        } else {
            Vec::new()
        };
        let adt = if adt.is_empty() {
            arms.iter()
                .find_map(|a| {
                    let tag = a.tag()?;
                    let a = ctx.data.adt_for_tag(tag);
                    if a.is_empty() {
                        None
                    } else {
                        Some(a)
                    }
                })
                .unwrap_or_default()
        } else {
            adt
        };
        if !adt.is_empty() {
            let covered: std::collections::HashSet<&str> =
                arms.iter().filter_map(|a| a.tag()).collect();
            let missing: Vec<&str> = adt
                .iter()
                .map(|(t, _)| t.as_str())
                .filter(|t| !covered.contains(t))
                .collect();
            if !missing.is_empty() {
                return Err(ElaborateError::at_node(
                    format!(
                        "non-exhaustive match: missing constructor(s) {}",
                        missing.join(", ")
                    ),
                    parent,
                ));
            }
        }
    }
    Ok(CoreExpr::Match {
        scrutinee: Box::new(scrutinee),
        arms,
    })
}

/// Pattern atoms before `->` (DAT-001 §15–17): `_`, `bind`, literals, `tuple`, `Tag`…
fn elaborate_pattern_atoms(
    atoms: &[Atom],
    parent: &SyntaxNode,
) -> Result<CorePattern, ElaborateError> {
    if atoms.is_empty() {
        return Err(ElaborateError::at_node(
            "match pattern must not be empty",
            parent,
        ));
    }
    // Nested list as the sole pattern atom: `(some item)` etc.
    if atoms.len() == 1 {
        if let Atom::Node(n) = &atoms[0] {
            if n.kind() != SyntaxKind::List {
                return Err(ElaborateError::at_node(
                    "match pattern must be a list or identifier",
                    n,
                ));
            }
            return elaborate_pattern_atoms(&list_atoms(n), n);
        }
        // §15.5 literal patterns as a sole atom.
        if let Atom::Token(t) = &atoms[0] {
            if let Some(lit) = pattern_literal_token(t)? {
                return Ok(CorePattern::Lit(lit));
            }
        }
    }

    let Atom::Token(head) = &atoms[0] else {
        return Err(ElaborateError::at_node(
            "match pattern head must be an identifier or literal",
            parent,
        ));
    };

    // Literal head with extra atoms is invalid.
    if matches!(head.kind(), SyntaxKind::Number | SyntaxKind::String) {
        if atoms.len() != 1 {
            return Err(ElaborateError::at_token(
                "literal pattern takes no arguments",
                head,
            ));
        }
        let lit = pattern_literal_token(head)?
            .ok_or_else(|| ElaborateError::at_token("unsupported literal pattern", head))?;
        return Ok(CorePattern::Lit(lit));
    }

    if head.kind() != SyntaxKind::Ident {
        return Err(ElaborateError::at_token(
            "match pattern head must be an identifier or literal",
            head,
        ));
    }
    let head_text = head.text();

    // §15.3 wildcard
    if head_text == "_" {
        if atoms.len() != 1 {
            return Err(ElaborateError::at_node(
                "wildcard pattern `_` takes no arguments",
                parent,
            ));
        }
        return Ok(CorePattern::Wildcard);
    }

    // §15.4 catch-all binder
    if head_text == "bind" {
        if atoms.len() != 2 {
            return Err(ElaborateError::at_node(
                "`bind` pattern requires a single binder name",
                parent,
            ));
        }
        let Atom::Token(b) = &atoms[1] else {
            return Err(ElaborateError::at_node(
                "`bind` pattern binder must be an identifier",
                parent,
            ));
        };
        if b.kind() != SyntaxKind::Ident {
            return Err(ElaborateError::at_token(
                "`bind` pattern binder must be an identifier",
                b,
            ));
        }
        return Ok(CorePattern::Bind(b.text().to_string()));
    }

    // §15.5 bool / unit literals (identifiers)
    if matches!(head_text, "true" | "false" | "unit") {
        if atoms.len() != 1 {
            return Err(ElaborateError::at_token(
                "literal pattern takes no arguments",
                head,
            ));
        }
        let lit = pattern_literal_token(head)?.expect("true/false/unit");
        return Ok(CorePattern::Lit(lit));
    }

    // §17 tuple pattern: `(tuple p0 p1 …)`
    if head_text == "tuple" {
        if atoms.len() < 2 {
            return Err(ElaborateError::at_node(
                "`tuple` pattern requires at least one element pattern",
                parent,
            ));
        }
        let mut elems = Vec::with_capacity(atoms.len() - 1);
        for atom in &atoms[1..] {
            elems.push(elaborate_payload_pattern(atom, parent)?);
        }
        return Ok(CorePattern::Tuple(elems));
    }

    // Constructor patterns: nullary, single payload, or multi-payload (§14.3).
    match atoms.len() {
        1 => Ok(CorePattern::Variant {
            tag: head_text.to_string(),
            payload: None,
        }),
        2 => {
            let payload = elaborate_payload_pattern(&atoms[1], parent)?;
            Ok(CorePattern::Variant {
                tag: head_text.to_string(),
                payload: Some(Box::new(payload)),
            })
        }
        _ => {
            let mut elems = Vec::with_capacity(atoms.len() - 1);
            for atom in &atoms[1..] {
                elems.push(elaborate_payload_pattern(atom, parent)?);
            }
            Ok(CorePattern::Variant {
                tag: head_text.to_string(),
                payload: Some(Box::new(CorePattern::Tuple(elems))),
            })
        }
    }
}

fn pattern_literal_token(tok: &SyntaxToken) -> Result<Option<CoreLiteral>, ElaborateError> {
    match tok.kind() {
        SyntaxKind::Number => {
            let text = tok.text();
            if text.contains('.') || text.contains('e') {
                return Err(ElaborateError::at_token(
                    "f64 literal patterns are not allowed (DAT-001 §15.5)",
                    tok,
                ));
            }
            let n = parse_number_literal(text).map_err(|msg| ElaborateError::at_token(msg, tok))?;
            Ok(Some(CoreLiteral::Number(n)))
        }
        SyntaxKind::String => {
            let value = decode_string_literal(tok.text())
                .map_err(|msg| ElaborateError::at_token(msg, tok))?;
            Ok(Some(CoreLiteral::String(value)))
        }
        SyntaxKind::Ident => match tok.text() {
            "true" => Ok(Some(CoreLiteral::Bool(true))),
            "false" => Ok(Some(CoreLiteral::Bool(false))),
            "unit" => Ok(Some(CoreLiteral::Unit)),
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

fn elaborate_payload_pattern(
    atom: &Atom,
    _parent: &SyntaxNode,
) -> Result<CorePattern, ElaborateError> {
    match atom {
        Atom::Token(t) => {
            if let Some(lit) = pattern_literal_token(t)? {
                return Ok(CorePattern::Lit(lit));
            }
            if t.kind() == SyntaxKind::Ident {
                if t.text() == "_" {
                    Ok(CorePattern::Wildcard)
                } else {
                    Ok(CorePattern::Bind(t.text().to_string()))
                }
            } else {
                Err(ElaborateError::at_token(
                    "match payload pattern must be an identifier, literal, or nested constructor",
                    t,
                ))
            }
        }
        Atom::Node(n) => {
            if n.kind() != SyntaxKind::List {
                return Err(ElaborateError::at_node(
                    "nested match pattern must be a list",
                    n,
                ));
            }
            elaborate_pattern_atoms(&list_atoms(n), n)
        }
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

/// DD-EFF-011: `(handler op (fn (params…) body…))` → first-class handler value.
fn elaborate_handler(
    rest: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    if rest.len() != 2 {
        return Err(ElaborateError::at_node(
            "`handler` requires an op identifier and `(fn (params…) …)`",
            parent,
        ));
    }
    let Atom::Token(op_tok) = &rest[0] else {
        return Err(ElaborateError::at_node(
            "`handler` op must be an identifier",
            parent,
        ));
    };
    if op_tok.kind() != SyntaxKind::Ident {
        return Err(ElaborateError::at_token(
            "`handler` op must be an identifier",
            op_tok,
        ));
    }
    let handler_expr = elaborate_atom(&rest[1], ctx)?;
    let CoreExpr::Lambda { params, body } = handler_expr else {
        return Err(ElaborateError::at_node(
            "`handler` body must be `(fn (params…) …)`",
            parent,
        ));
    };
    if !(1..=2).contains(&params.len()) {
        return Err(ElaborateError::at_node(
            "`handler` expects 1 or 2 parameters (arg) or (arg resume)",
            parent,
        ));
    }
    Ok(CoreExpr::HandlerValue {
        op: op_tok.text().to_string(),
        handler_params: params,
        handler_body: body,
    })
}

/// DD-EFF-012: `(with handler-expr body…)` → install handler around body seq.
fn elaborate_with(
    rest: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    if rest.len() < 2 {
        return Err(ElaborateError::at_node(
            "`with` requires a handler expression and at least one body expression",
            parent,
        ));
    }
    let handler = elaborate_atom(&rest[0], ctx)?;
    let body = seq_or_one(elaborate_atoms(&rest[1..], ctx)?);
    Ok(CoreExpr::With {
        handler: Box::new(handler),
        body: Box::new(body),
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

fn elaborate_letrec(
    rest: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    // (letrec ((name (fn ...))...) body...) — RHS must elaborate to Lambda.
    let Some(Atom::Node(bindings_node)) = rest.first() else {
        return Err(ElaborateError::at_node(
            "`letrec` requires a binding list",
            parent,
        ));
    };
    if bindings_node.kind() != SyntaxKind::List {
        return Err(ElaborateError::at_node(
            "`letrec` binding list must be a parenthesized list",
            bindings_node,
        ));
    }
    let binding_atoms = list_atoms(bindings_node);
    if binding_atoms.is_empty() {
        return Err(ElaborateError::at_node(
            "`letrec` binding list must not be empty",
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
                    "`letrec` binding must be `(name (fn …))`",
                    t,
                ));
            }
        };
        if pair.kind() != SyntaxKind::List {
            return Err(ElaborateError::at_node(
                "`letrec` binding must be `(name (fn …))`",
                pair,
            ));
        }
        let pair_atoms = list_atoms(pair);
        if pair_atoms.len() != 2 {
            return Err(ElaborateError::at_node(
                "`letrec` binding must be `(name (fn …))`",
                pair,
            ));
        }
        let Atom::Token(name_tok) = &pair_atoms[0] else {
            return Err(ElaborateError::at_node(
                "`letrec` binder must be an identifier",
                pair,
            ));
        };
        if name_tok.kind() != SyntaxKind::Ident {
            return Err(ElaborateError::at_token(
                "`letrec` binder must be an identifier",
                name_tok,
            ));
        }
        let name = name_tok.text().to_string();
        if !seen.insert(name.clone()) {
            return Err(ElaborateError::at_token(
                format!("duplicate binder `{name}` in the same `letrec`"),
                name_tok,
            ));
        }
        let value = elaborate_atom(&pair_atoms[1], ctx)?;
        if !matches!(value, CoreExpr::Lambda { .. }) {
            return Err(ElaborateError::at_node(
                "`letrec` right-hand side must be `(fn …)`",
                pair,
            ));
        }
        bindings.push((name, value));
    }

    let body = elaborate_body(&rest[1..], parent, ctx)?;
    Ok(CoreExpr::LetRec {
        bindings,
        body: Box::new(body),
    })
}

/// SYN §13.5: `(local declaration* result)` — `val`, `var`, nested `rec`.
fn elaborate_local(
    rest: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    if rest.is_empty() {
        return Err(ElaborateError::at_node(
            "`local` requires declarations and a result expression",
            parent,
        ));
    }
    let result_atom = rest.last().expect("non-empty");
    let decl_atoms = &rest[..rest.len() - 1];
    elaborate_local_decls(decl_atoms, result_atom, parent, ctx)
}

fn elaborate_local_decls(
    decls: &[Atom],
    result: &Atom,
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    if decls.is_empty() {
        return elaborate_atom(result, ctx);
    }
    let Atom::Node(decl) = &decls[0] else {
        return Err(ElaborateError::at_node(
            "`local` declarations must be `(val …)`, `(var …)`, or `(rec …)` forms",
            parent,
        ));
    };
    if decl.kind() != SyntaxKind::List {
        return Err(ElaborateError::at_node(
            "`local` declarations must be `(val …)`, `(var …)`, or `(rec …)` forms",
            decl,
        ));
    }
    let da = list_atoms(decl);
    let Some(Atom::Token(head)) = da.first() else {
        return Err(ElaborateError::at_node(
            "`local` declaration must start with `val`, `var`, `rec`, or `type`",
            decl,
        ));
    };
    if head.kind() != SyntaxKind::Ident {
        return Err(ElaborateError::at_token(
            "`local` declaration head must be an identifier",
            head,
        ));
    }
    let rest_decls = &decls[1..];
    match head.text() {
        "type" => elaborate_local_decls(rest_decls, result, parent, ctx),
        "val" => {
            if da.len() != 3 {
                return Err(ElaborateError::at_node(
                    "`val` in `local` must be `(val name expr)`",
                    decl,
                ));
            }
            let Atom::Token(name_tok) = &da[1] else {
                return Err(ElaborateError::at_node(
                    "`val` binder must be an identifier",
                    decl,
                ));
            };
            if name_tok.kind() != SyntaxKind::Ident {
                return Err(ElaborateError::at_token(
                    "`val` binder must be an identifier",
                    name_tok,
                ));
            }
            let name = name_tok.text().to_string();
            if is_reserved_special_form(&name) {
                return Err(ElaborateError::at_token(
                    format!("cannot bind reserved special-form `{name}`"),
                    name_tok,
                ));
            }
            let value = elaborate_atom(&da[2], ctx)?;
            Ok(CoreExpr::Let {
                name,
                value: Box::new(value),
                body: Box::new(elaborate_local_decls(rest_decls, result, parent, ctx)?),
            })
        }
        "var" => {
            // Declaration form `(var name init)` scopes over the rest of `local`.
            if da.len() != 3 {
                return Err(ElaborateError::at_node(
                    "`var` in `local` must be `(var name init)`",
                    decl,
                ));
            }
            let Atom::Token(name_tok) = &da[1] else {
                return Err(ElaborateError::at_node(
                    "`var` binder must be an identifier",
                    decl,
                ));
            };
            if name_tok.kind() != SyntaxKind::Ident {
                return Err(ElaborateError::at_token(
                    "`var` binder must be an identifier",
                    name_tok,
                ));
            }
            let name = name_tok.text().to_string();
            if is_reserved_special_form(&name) {
                return Err(ElaborateError::at_token(
                    format!("cannot bind reserved special-form `{name}`"),
                    name_tok,
                ));
            }
            let init = elaborate_atom(&da[2], ctx)?;
            Ok(CoreExpr::LocalVar {
                name,
                init: Box::new(init),
                body: Box::new(elaborate_local_decls(rest_decls, result, parent, ctx)?),
            })
        }
        "rec" => {
            let bindings = parse_rec_val_bindings(&da[1..], decl, ctx)?;
            if bindings.is_empty() {
                return Err(ElaborateError::at_node(
                    "`rec` requires at least one `(val …)` binding",
                    decl,
                ));
            }
            Ok(CoreExpr::LetRec {
                bindings,
                body: Box::new(elaborate_local_decls(rest_decls, result, parent, ctx)?),
            })
        }
        other => Err(ElaborateError::at_node(
            format!("`local` does not support `{other}` declarations"),
            decl,
        )),
    }
}

/// SYN §13.4: `(rec (val name (fn …))* result)` → [`CoreExpr::LetRec`].
fn elaborate_rec(
    rest: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    if rest.len() < 2 {
        return Err(ElaborateError::at_node(
            "`rec` requires at least one `(val …)` and a result expression",
            parent,
        ));
    }
    let result_atom = rest.last().expect("len >= 2");
    // Expression `rec` requires a non-declaration result.
    if is_rec_decl_atom(result_atom) {
        return Err(ElaborateError::at_node(
            "`rec` expression requires a result expression after the bindings",
            parent,
        ));
    }
    let decl_atoms = &rest[..rest.len() - 1];
    let bindings = parse_rec_val_bindings(decl_atoms, parent, ctx)?;
    if bindings.is_empty() {
        return Err(ElaborateError::at_node(
            "`rec` requires at least one `(val …)` binding",
            parent,
        ));
    }
    Ok(CoreExpr::LetRec {
        bindings,
        body: Box::new(elaborate_atom(result_atom, ctx)?),
    })
}

fn is_rec_decl_atom(atom: &Atom) -> bool {
    let Atom::Node(n) = atom else {
        return false;
    };
    matches!(list_head_ident(n).as_deref(), Some("val" | "type"))
}

fn parse_rec_val_bindings(
    decl_atoms: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<Vec<(String, CoreExpr)>, ElaborateError> {
    let mut bindings = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for atom in decl_atoms {
        let Atom::Node(decl) = atom else {
            return Err(ElaborateError::at_node(
                "`rec` entries must be `(val name (fn …))` forms",
                parent,
            ));
        };
        if decl.kind() != SyntaxKind::List {
            return Err(ElaborateError::at_node(
                "`rec` entries must be `(val name (fn …))` forms",
                decl,
            ));
        }
        let da = list_atoms(decl);
        let Some(Atom::Token(head)) = da.first() else {
            return Err(ElaborateError::at_node(
                "`rec` entry must start with `val` or `type`",
                decl,
            ));
        };
        if head.kind() != SyntaxKind::Ident {
            return Err(ElaborateError::at_token(
                "`rec` entry head must be an identifier",
                head,
            ));
        }
        if head.text() == "type" {
            continue;
        }
        if head.text() != "val" {
            return Err(ElaborateError::at_node(
                "`rec` currently supports `(val name (fn …))` entries only",
                decl,
            ));
        }
        if da.len() != 3 {
            return Err(ElaborateError::at_node(
                "`val` in `rec` must be `(val name (fn …))`",
                decl,
            ));
        }
        let Atom::Token(name_tok) = &da[1] else {
            return Err(ElaborateError::at_node(
                "`val` binder must be an identifier",
                decl,
            ));
        };
        if name_tok.kind() != SyntaxKind::Ident {
            return Err(ElaborateError::at_token(
                "`val` binder must be an identifier",
                name_tok,
            ));
        }
        let name = name_tok.text().to_string();
        if is_reserved_special_form(&name) {
            return Err(ElaborateError::at_token(
                format!("cannot bind reserved special-form `{name}`"),
                name_tok,
            ));
        }
        if !seen.insert(name.clone()) {
            return Err(ElaborateError::at_token(
                format!("duplicate binder `{name}` in the same `rec`"),
                name_tok,
            ));
        }
        let value = elaborate_atom(&da[2], ctx)?;
        if !matches!(value, CoreExpr::Lambda { .. }) {
            return Err(ElaborateError::at_node(
                "`rec` right-hand side must be `(fn …)`",
                decl,
            ));
        }
        bindings.push((name, value));
    }
    Ok(bindings)
}

fn elaborate_var(
    rest: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    // (var name init body...)
    if rest.len() < 3 {
        return Err(ElaborateError::at_node(
            "`var` requires name, initializer, and body",
            parent,
        ));
    }
    let Atom::Token(name_tok) = &rest[0] else {
        return Err(ElaborateError::at_node(
            "`var` name must be an identifier",
            parent,
        ));
    };
    if name_tok.kind() != SyntaxKind::Ident {
        return Err(ElaborateError::at_token(
            "`var` name must be an identifier",
            name_tok,
        ));
    }
    Ok(CoreExpr::LocalVar {
        name: name_tok.text().to_string(),
        init: Box::new(elaborate_atom(&rest[1], ctx)?),
        body: Box::new(elaborate_body(&rest[2..], parent, ctx)?),
    })
}

fn elaborate_set(
    rest: &[Atom],
    parent: &SyntaxNode,
    ctx: &ElabCtx,
) -> Result<CoreExpr, ElaborateError> {
    // (set name expr)
    if rest.len() != 2 {
        return Err(ElaborateError::at_node(
            "`set` requires a name and an expression",
            parent,
        ));
    }
    let Atom::Token(name_tok) = &rest[0] else {
        return Err(ElaborateError::at_node(
            "`set` name must be an identifier",
            parent,
        ));
    };
    if name_tok.kind() != SyntaxKind::Ident {
        return Err(ElaborateError::at_token(
            "`set` name must be an identifier",
            name_tok,
        ));
    }
    Ok(CoreExpr::Set {
        name: name_tok.text().to_string(),
        value: Box::new(elaborate_atom(&rest[1], ctx)?),
    })
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
            let n = parse_number_literal(text).map_err(|msg| ElaborateError::at_token(msg, tok))?;
            Ok(CoreExpr::Lit(CoreLiteral::Number(n)))
        }
        SyntaxKind::String => {
            let raw = tok.text();
            let value =
                decode_string_literal(raw).map_err(|msg| ElaborateError::at_token(msg, tok))?;
            Ok(CoreExpr::Lit(CoreLiteral::String(value)))
        }
        SyntaxKind::Ident => match tok.text() {
            "true" => Ok(CoreExpr::Lit(CoreLiteral::Bool(true))),
            "false" => Ok(CoreExpr::Lit(CoreLiteral::Bool(false))),
            "unit" => Ok(CoreExpr::Lit(CoreLiteral::Unit)),
            name if ctx.data.ctors.get(name) == Some(&0) => Ok(CoreExpr::Variant {
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
(val main (match (Some 1) (None -> 0) (Some x -> x)))
"#,
        )
        .unwrap();
        let CoreExpr::Let { value, .. } = expr else {
            panic!("expected Let");
        };
        assert!(matches!(*value, CoreExpr::Match { .. }));
    }
}
