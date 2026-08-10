//! MAC-001 language macros: user-defined expression macros with gensym hygiene.
//!
//! Primary surface (MAC-001):
//! ```text
//! (macro name ($params... [$rest ...+]) -> template)
//! ```
//! Legacy `(macro name (params) template)` without `->` is rejected.
//! Calls `(name args...)` expand before elaborate/typecheck.
//! Pattern rest `$body ...+` requires ≥1 argument; template `$body ...` splices.

use std::collections::HashMap;

use reciplexa_syntax::{
    is_reserved_special_form, parse_source, SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken,
};

use crate::ExpandError;

/// Soft cap on macro applications (each successful rewrite counts once).
pub const EXPANSION_BUDGET: u32 = 256;

/// Fresh binder names use this prefix plus a monotonic counter.
const GENSYM_PREFIX: &str = "__rx_";

#[derive(Debug, Clone, PartialEq, Eq)]
enum Sexpr {
    Atom(String),
    List(Vec<Sexpr>),
}

#[derive(Debug, Clone)]
struct MacroDef {
    /// Fixed pattern variables (each starts with `$`).
    params: Vec<String>,
    /// Optional trailing rest variable bound by `$rest ...+` (one-or-more).
    rest: Option<String>,
    template: Sexpr,
}

/// Expand user `(macro …)` definitions and calls; leave graphics sugar alone.
///
/// Macro definitions are stripped from the output. Remaining forms are rewritten
/// with a fixed expansion budget ([`EXPANSION_BUDGET`]).
pub fn expand_language(input: &str) -> Result<String, ExpandError> {
    let parse = parse_source(input);
    if !parse.errors.is_empty() {
        return Err(ExpandError::new(format!(
            "parse error: {}",
            parse.errors[0].message
        )));
    }

    let forms = top_level_forms(&parse.root)?;
    let mut macros: HashMap<String, MacroDef> = HashMap::new();
    let mut out_forms = Vec::new();
    let mut budget = EXPANSION_BUDGET;
    let mut gensym = 0u64;

    for form in forms {
        if let Some((name, def)) = try_macro_def(&form)? {
            if macros.contains_key(&name) {
                return Err(ExpandError::new(format!(
                    "duplicate macro definition `{name}`"
                )));
            }
            if is_reserved(&name) {
                return Err(ExpandError::new(format!(
                    "cannot define macro with reserved name `{name}`"
                )));
            }
            macros.insert(name, def);
            continue;
        }
        out_forms.push(expand_sexpr(form, &macros, &mut budget, &mut gensym)?);
    }

    Ok(render_forms(&out_forms))
}

fn is_reserved(name: &str) -> bool {
    // SYN §6 Core special forms + surface forms (via shared reserved table).
    is_reserved_special_form(name)
}

fn try_macro_def(form: &Sexpr) -> Result<Option<(String, MacroDef)>, ExpandError> {
    let Sexpr::List(items) = form else {
        return Ok(None);
    };
    let Some(Sexpr::Atom(head)) = items.first() else {
        return Ok(None);
    };
    if head != "macro" {
        return Ok(None);
    }

    // MAC-001: (macro name ($params...) -> template) — `->` is required.
    let (name, param_items, template) = match items.as_slice() {
        [_, Sexpr::Atom(name), Sexpr::List(params), Sexpr::Atom(arrow), template]
            if arrow == "->" =>
        {
            (name, params, template)
        }
        [_, Sexpr::Atom(_), Sexpr::List(_), _] => {
            return Err(ExpandError::new(
                "`macro` requires `(macro name ($params...) -> template)`; \
                 legacy form without `->` is not accepted",
            ));
        }
        _ => {
            return Err(ExpandError::new(
                "`macro` requires `(macro name ($params...) -> template)`",
            ));
        }
    };

    let (params, rest) = parse_macro_params(param_items)?;
    // Unbound template pattern vars are a definition-time error (MAC §18.4).
    check_template_vars(&template, &params, rest.as_deref())?;

    Ok(Some((
        name.clone(),
        MacroDef {
            params,
            rest,
            template: template.clone(),
        },
    )))
}

fn parse_macro_params(
    param_items: &[Sexpr],
) -> Result<(Vec<String>, Option<String>), ExpandError> {
    let mut params = Vec::new();
    let mut rest = None;
    let mut seen = std::collections::HashSet::new();
    let mut i = 0;
    while i < param_items.len() {
        let Sexpr::Atom(pname) = &param_items[i] else {
            return Err(ExpandError::new("macro parameter must be an identifier"));
        };
        if pname == "...+" {
            return Err(ExpandError::new(
                "`...+` must follow a pattern variable (`$body ...+`)",
            ));
        }
        if pname == "..." {
            return Err(ExpandError::new(
                "`...` is only valid in templates, not in macro patterns",
            ));
        }
        if !pname.starts_with('$') {
            return Err(ExpandError::new(format!(
                "MAC-001 pattern variable must start with `$`, got `{pname}`"
            )));
        }
        // Trailing rest: `$body ...+` (must be last; only one).
        if i + 1 < param_items.len() {
            if let Sexpr::Atom(marker) = &param_items[i + 1] {
                if marker == "...+" {
                    if i + 2 != param_items.len() {
                        return Err(ExpandError::new(
                            "macro `...+` rest parameter must be last in the pattern",
                        ));
                    }
                    if rest.is_some() {
                        return Err(ExpandError::new(
                            "macro pattern may contain at most one `...+` rest",
                        ));
                    }
                    if !seen.insert(pname.clone()) {
                        return Err(ExpandError::new(format!(
                            "duplicate macro parameter `{pname}`"
                        )));
                    }
                    rest = Some(pname.clone());
                    i += 2;
                    continue;
                }
            }
        }
        if !seen.insert(pname.clone()) {
            return Err(ExpandError::new(format!(
                "duplicate macro parameter `{pname}`"
            )));
        }
        params.push(pname.clone());
        i += 1;
    }
    Ok((params, rest))
}

fn check_template_vars(
    template: &Sexpr,
    params: &[String],
    rest: Option<&str>,
) -> Result<(), ExpandError> {
    let mut bound: std::collections::HashSet<&str> = params.iter().map(|s| s.as_str()).collect();
    if let Some(r) = rest {
        bound.insert(r);
    }
    walk_template_vars(template, &bound)
}

fn walk_template_vars(
    expr: &Sexpr,
    bound: &std::collections::HashSet<&str>,
) -> Result<(), ExpandError> {
    match expr {
        Sexpr::Atom(name) if name.starts_with('$') && name != "..." && name != "...+" => {
            if !bound.contains(name.as_str()) {
                return Err(ExpandError::new(format!(
                    "unbound macro pattern variable `{name}`"
                )));
            }
            Ok(())
        }
        Sexpr::Atom(_) => Ok(()),
        Sexpr::List(items) => {
            for item in items {
                walk_template_vars(item, bound)?;
            }
            Ok(())
        }
    }
}

fn expand_sexpr(
    expr: Sexpr,
    macros: &HashMap<String, MacroDef>,
    budget: &mut u32,
    gensym: &mut u64,
) -> Result<Sexpr, ExpandError> {
    match expr {
        Sexpr::Atom(_) => Ok(expr),
        Sexpr::List(items) => {
            if let Some(Sexpr::Atom(head)) = items.first() {
                if let Some(def) = macros.get(head) {
                    return expand_call(head, def, &items[1..], macros, budget, gensym);
                }
            }
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(expand_sexpr(item, macros, budget, gensym)?);
            }
            Ok(Sexpr::List(out))
        }
    }
}

fn expand_call(
    name: &str,
    def: &MacroDef,
    args: &[Sexpr],
    macros: &HashMap<String, MacroDef>,
    budget: &mut u32,
    gensym: &mut u64,
) -> Result<Sexpr, ExpandError> {
    if *budget == 0 {
        return Err(ExpandError::new("macro expansion limit exceeded"));
    }
    *budget -= 1;

    let fixed = def.params.len();
    match &def.rest {
        None => {
            if args.len() != fixed {
                return Err(ExpandError::new(format!(
                    "macro `{name}` expects {} arguments, but received {}",
                    fixed,
                    args.len()
                )));
            }
        }
        Some(rest_name) => {
            if args.len() < fixed + 1 {
                return Err(ExpandError::new(format!(
                    "macro `{name}` requires at least one `{rest_name}` expression"
                )));
            }
        }
    }

    // Hygiene v0: rename binders introduced by the template, then substitute params.
    let renamed = hygienic_rename(&def.template, gensym);
    let mut subst = HashMap::new();
    for (param, arg) in def.params.iter().zip(args.iter()) {
        subst.insert(param.clone(), arg.clone());
    }
    if let Some(rest_name) = &def.rest {
        let rest_args: Vec<Sexpr> = args[fixed..].to_vec();
        subst.insert(rest_name.clone(), Sexpr::List(rest_args));
    }
    let filled = substitute(&renamed, &subst);
    expand_sexpr(filled, macros, budget, gensym)
}

/// Rename `fn` / `let` binders in `template` to fresh gensym names.
fn hygienic_rename(template: &Sexpr, gensym: &mut u64) -> Sexpr {
    rename_binders(template, &HashMap::new(), gensym)
}

fn fresh(gensym: &mut u64) -> String {
    let n = *gensym;
    *gensym += 1;
    format!("{GENSYM_PREFIX}{n}")
}

fn rename_binders(expr: &Sexpr, env: &HashMap<String, String>, gensym: &mut u64) -> Sexpr {
    match expr {
        Sexpr::Atom(name) => {
            if let Some(mapped) = env.get(name) {
                Sexpr::Atom(mapped.clone())
            } else {
                Sexpr::Atom(name.clone())
            }
        }
        Sexpr::List(items) if items.is_empty() => Sexpr::List(vec![]),
        Sexpr::List(items) => {
            if let Sexpr::Atom(head) = &items[0] {
                match head.as_str() {
                    "fn" => return rename_fn(items, env, gensym),
                    "let" => return rename_let(items, env, gensym),
                    _ => {}
                }
            }
            Sexpr::List(
                items
                    .iter()
                    .map(|i| rename_binders(i, env, gensym))
                    .collect(),
            )
        }
    }
}

fn rename_fn(items: &[Sexpr], env: &HashMap<String, String>, gensym: &mut u64) -> Sexpr {
    // (fn (params...) body...) or (fn name (params...) body...)
    let (params_idx, body_start) = match items.get(1) {
        Some(Sexpr::List(_)) => (1usize, 2usize),
        Some(Sexpr::Atom(_)) if matches!(items.get(2), Some(Sexpr::List(_))) => (2usize, 3usize),
        _ => {
            return Sexpr::List(
                items
                    .iter()
                    .map(|i| rename_binders(i, env, gensym))
                    .collect(),
            );
        }
    };

    let Sexpr::List(params) = &items[params_idx] else {
        unreachable!("checked above");
    };

    let mut child_env = env.clone();
    let mut new_params = Vec::with_capacity(params.len());
    for p in params {
        match p {
            Sexpr::Atom(name) => {
                let fresh_name = fresh(gensym);
                child_env.insert(name.clone(), fresh_name.clone());
                new_params.push(Sexpr::Atom(fresh_name));
            }
            other => new_params.push(rename_binders(other, env, gensym)),
        }
    }

    let mut out = Vec::with_capacity(items.len());
    out.push(Sexpr::Atom("fn".into()));
    if params_idx == 2 {
        // Keep optional name; expression elaborator ignores it.
        out.push(rename_binders(&items[1], env, gensym));
    }
    out.push(Sexpr::List(new_params));
    for body in &items[body_start..] {
        out.push(rename_binders(body, &child_env, gensym));
    }
    Sexpr::List(out)
}

fn rename_let(items: &[Sexpr], env: &HashMap<String, String>, gensym: &mut u64) -> Sexpr {
    // (let ((name expr)...) body...)
    if items.len() < 3 {
        return Sexpr::List(
            items
                .iter()
                .map(|i| rename_binders(i, env, gensym))
                .collect(),
        );
    }
    let Sexpr::List(bindings) = &items[1] else {
        return Sexpr::List(
            items
                .iter()
                .map(|i| rename_binders(i, env, gensym))
                .collect(),
        );
    };

    // Parallel let: binding exprs see outer env; body sees new binders.
    let mut child_env = env.clone();
    let mut new_bindings = Vec::with_capacity(bindings.len());
    let mut binder_fresh: Vec<Option<String>> = Vec::with_capacity(bindings.len());

    for b in bindings {
        if let Sexpr::List(pair) = b {
            if let Some(Sexpr::Atom(name)) = pair.first() {
                let fresh_name = fresh(gensym);
                child_env.insert(name.clone(), fresh_name.clone());
                binder_fresh.push(Some(fresh_name));
                continue;
            }
        }
        binder_fresh.push(None);
    }

    for (b, fresh_opt) in bindings.iter().zip(binder_fresh.iter()) {
        match (b, fresh_opt) {
            (Sexpr::List(pair), Some(fresh_name)) if pair.len() >= 2 => {
                let mut new_pair = Vec::with_capacity(pair.len());
                new_pair.push(Sexpr::Atom(fresh_name.clone()));
                for e in &pair[1..] {
                    new_pair.push(rename_binders(e, env, gensym));
                }
                new_bindings.push(Sexpr::List(new_pair));
            }
            _ => new_bindings.push(rename_binders(b, env, gensym)),
        }
    }

    let mut out = vec![Sexpr::Atom("let".into()), Sexpr::List(new_bindings)];
    for body in &items[2..] {
        out.push(rename_binders(body, &child_env, gensym));
    }
    Sexpr::List(out)
}

fn substitute(expr: &Sexpr, subst: &HashMap<String, Sexpr>) -> Sexpr {
    match expr {
        Sexpr::Atom(name) => {
            if let Some(replacement) = subst.get(name) {
                replacement.clone()
            } else {
                Sexpr::Atom(name.clone())
            }
        }
        Sexpr::List(items) => {
            // Splice rest: `$body ...` expands to the rest argument sequence.
            let mut out = Vec::with_capacity(items.len());
            let mut i = 0;
            while i < items.len() {
                if i + 1 < items.len() {
                    if let (Sexpr::Atom(var), Sexpr::Atom(dots)) = (&items[i], &items[i + 1]) {
                        if dots == "..." {
                            if let Some(Sexpr::List(rest_args)) = subst.get(var) {
                                out.extend(rest_args.iter().cloned());
                                i += 2;
                                continue;
                            }
                        }
                    }
                }
                out.push(substitute(&items[i], subst));
                i += 1;
            }
            Sexpr::List(out)
        }
    }
}

fn top_level_forms(root: &SyntaxNode) -> Result<Vec<Sexpr>, ExpandError> {
    let mut forms = Vec::new();
    for el in root.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) => {
                if t.kind().is_trivia() {
                    continue;
                }
                forms.push(token_to_sexpr(&t)?);
            }
            SyntaxElement::Node(n) => match n.kind() {
                SyntaxKind::StructuredComment => continue,
                SyntaxKind::List
                | SyntaxKind::BracketList
                | SyntaxKind::BraceList
                | SyntaxKind::AtExpr => {
                    forms.push(node_to_sexpr(&n)?);
                }
                other => {
                    return Err(ExpandError::new(format!(
                        "unsupported top-level form `{other:?}`"
                    )));
                }
            },
        }
    }
    Ok(forms)
}

fn node_to_sexpr(node: &SyntaxNode) -> Result<Sexpr, ExpandError> {
    match node.kind() {
        SyntaxKind::List | SyntaxKind::BracketList | SyntaxKind::BraceList => {
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
                                    | SyntaxKind::LBrace
                                    | SyntaxKind::RBrace
                            )
                        {
                            continue;
                        }
                        items.push(token_to_sexpr(&t)?);
                    }
                    SyntaxElement::Node(n) => items.push(node_to_sexpr(&n)?),
                }
            }
            Ok(Sexpr::List(items))
        }
        SyntaxKind::AtExpr => {
            // Preserve as a list headed by `@` plus children — language macros
            // should not normally see these; keep round-trip fidelity.
            let mut items = vec![Sexpr::Atom("@".into())];
            for el in node.children_with_tokens() {
                match el {
                    SyntaxElement::Token(t) => {
                        if t.kind().is_trivia() || t.kind() == SyntaxKind::At {
                            continue;
                        }
                        items.push(token_to_sexpr(&t)?);
                    }
                    SyntaxElement::Node(n) => items.push(node_to_sexpr(&n)?),
                }
            }
            Ok(Sexpr::List(items))
        }
        other => Err(ExpandError::new(format!(
            "cannot convert `{other:?}` to sexpr"
        ))),
    }
}

fn token_to_sexpr(tok: &SyntaxToken) -> Result<Sexpr, ExpandError> {
    match tok.kind() {
        SyntaxKind::Ident
        | SyntaxKind::Number
        | SyntaxKind::String
        | SyntaxKind::TextChunk
        | SyntaxKind::Arrow => Ok(Sexpr::Atom(tok.text().to_string())),
        other => Err(ExpandError::new(format!(
            "unexpected token `{other:?}` in language expand"
        ))),
    }
}

fn render_forms(forms: &[Sexpr]) -> String {
    let mut out = String::new();
    for (i, form) in forms.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        render_sexpr(form, &mut out);
    }
    out
}

fn render_sexpr(expr: &Sexpr, out: &mut String) {
    match expr {
        Sexpr::Atom(a) => out.push_str(a),
        Sexpr::List(items) => {
            out.push('(');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                render_sexpr(item, out);
            }
            out.push(')');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_call1_identity() {
        let src = "(macro call1 ($f $x) -> ($f $x))\n(val main (call1 (fn (x) x) 42))";
        let out = expand_language(src).unwrap();
        assert!(!out.contains("(macro "));
        // Args are not renamed; template `($f $x)` has no binders.
        assert_eq!(out, "(val main ((fn (x) x) 42))");
    }

    #[test]
    fn expands_unless_mac001_form() {
        let src = r#"
(macro unless ($condition $expression) -> (if $condition unit $expression))
(val main (unless false 7))
"#;
        let out = expand_language(src).unwrap();
        assert_eq!(out, "(val main (if false unit 7))");
    }

    #[test]
    fn expands_when_with_rest_plus() {
        let src = r#"
(macro when ($condition $body ...+) -> (if $condition (seq $body ...) unit))
(val main (when true 1 2 3))
"#;
        let out = expand_language(src).unwrap();
        assert_eq!(out, "(val main (if true (seq 1 2 3) unit))");
    }

    #[test]
    fn rest_plus_rejects_empty_body() {
        let src = r#"
(macro when ($condition $body ...+) -> (if $condition (seq $body ...) unit))
(val main (when true))
"#;
        let err = expand_language(src).unwrap_err();
        assert!(
            err.message.contains("at least one"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn hygiene_avoids_capture() {
        // (m body) → (fn (x) body); call (m x) must not capture the binder.
        let src = "(macro m ($body) -> (fn (x) $body))\n(val main ((m x) 42))";
        let out = expand_language(src).unwrap();
        assert!(out.contains("__rx_"), "binder should be gensym'd: {out}");
        assert!(
            !out.contains("(fn (x) x)"),
            "must not capture call-site x: {out}"
        );
    }

    #[test]
    fn expansion_budget_trips() {
        let src = "(macro loop ($x) -> (loop $x))\n(val main (loop 1))";
        let err = expand_language(src).unwrap_err();
        assert!(err.message.contains("limit exceeded"));
    }

    #[test]
    fn rejects_legacy_macro_form_without_arrow() {
        let src = "(macro call1 (f x) (f x))\n(val main (call1 (fn (x) x) 1))";
        let err = expand_language(src).unwrap_err();
        assert!(
            err.message.contains("->") || err.message.contains("legacy"),
            "unexpected: {}",
            err.message
        );
    }

    #[test]
    fn rejects_reserved_macro_names() {
        for name in ["perform", "handle", "data", "var", "set", "with"] {
            let src = format!("(macro {name} ($x) -> $x)\n(val main 1)");
            let err = expand_language(&src).unwrap_err();
            assert!(
                err.message.contains("reserved"),
                "{name}: {}",
                err.message
            );
        }
    }
}
