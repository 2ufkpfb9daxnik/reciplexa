//! MAC-001 language macros: user-defined expression macros with gensym hygiene.
//!
//! Primary surface (MAC-001):
//! ```text
//! (macro name ($params... [$rest ...+]) -> template)
//! ```
//! Legacy `(macro name (params) template)` without `->` is rejected.
//! Calls `(name args...)` expand before elaborate/typecheck.
//! Pattern rest `$body ...+` requires ≥1 argument; template `$body ...` splices.
//!
//! Nested `(module name …)` forms follow MAC §8.3–8.4: child scopes inherit
//! parent macros defined earlier; sibling / later macros are not visible.
//! Macro templates form a DAG (§12.4); cycles are rejected at definition time.
//! [`expand_language_with_map`] records call/def SyntaxNodeId + spans (§19).

use std::collections::{HashMap, HashSet};

use reciplexa_identity::syntax::SyntaxNodeId;
use reciplexa_syntax::{
    build_identity_map, is_reserved_special_form, parse_source, SyntaxElement, SyntaxKind,
    SyntaxNode, SyntaxToken,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Loc {
    start: u32,
    end: u32,
    id: SyntaxNodeId,
}

#[derive(Debug, Clone)]
struct MacroDef {
    /// Fixed pattern variables (each starts with `$`).
    params: Vec<String>,
    /// Optional trailing rest variable bound by `$rest ...+` (one-or-more).
    rest: Option<String>,
    template: Sexpr,
    def_loc: Option<Loc>,
}

/// One successful macro rewrite (MAC §19 provenance / source map).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpansionOrigin {
    pub macro_name: String,
    pub call_span: (u32, u32),
    pub def_span: (u32, u32),
    pub call_id: SyntaxNodeId,
    pub def_id: SyntaxNodeId,
    /// Expansion chain from outer call to this rewrite (inclusive).
    pub chain: Vec<String>,
}

/// Source map produced alongside expansion (MAC §19.3).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MacroSourceMap {
    pub origins: Vec<ExpansionOrigin>,
}

/// Expand user `(macro …)` definitions and calls; leave graphics sugar alone.
///
/// Macro definitions are stripped from the output. Remaining forms are rewritten
/// with a fixed expansion budget ([`EXPANSION_BUDGET`]).
pub fn expand_language(input: &str) -> Result<String, ExpandError> {
    Ok(expand_language_with_map(input)?.0)
}

/// Expand language macros and return the MAC §19 provenance / source map.
pub fn expand_language_with_map(input: &str) -> Result<(String, MacroSourceMap), ExpandError> {
    let parse = parse_source(input);
    if !parse.errors.is_empty() {
        return Err(ExpandError::new(format!(
            "parse error: {}",
            parse.errors[0].message
        )));
    }

    let ids = build_identity_map(&parse.root);
    let forms = top_level_forms(&parse.root, &ids)?;
    let mut budget = EXPANSION_BUDGET;
    let mut gensym = 0u64;
    let mut source_map = MacroSourceMap::default();
    let mut chain = Vec::new();
    let out_forms = expand_scope(
        forms,
        &HashMap::new(),
        &mut budget,
        &mut gensym,
        &mut source_map,
        &mut chain,
    )?;
    Ok((render_forms(&out_forms), source_map))
}

/// Expand a declaration scope with inherited parent macros (MAC §8.3–8.4).
fn expand_scope(
    forms: Vec<(Sexpr, Option<Loc>)>,
    inherited: &HashMap<String, MacroDef>,
    budget: &mut u32,
    gensym: &mut u64,
    source_map: &mut MacroSourceMap,
    chain: &mut Vec<String>,
) -> Result<Vec<Sexpr>, ExpandError> {
    // Pass 1: record local macro / value binding positions (MAC-10 / §9.3 / §8.4).
    let mut macro_at: HashMap<String, usize> = HashMap::new();
    let mut value_at: HashMap<String, usize> = HashMap::new();
    for (i, (form, _)) in forms.iter().enumerate() {
        if let Some(name) = peek_macro_def_name(form)? {
            if macro_at.contains_key(&name) {
                return Err(ExpandError::new(format!(
                    "duplicate macro definition `{name}`"
                )));
            }
            if value_at.contains_key(&name) {
                return Err(ExpandError::new(format!(
                    "macro `{name}` conflicts with an existing value binding in the same scope"
                )));
            }
            if is_reserved(&name) {
                return Err(ExpandError::new(format!(
                    "cannot define macro with reserved name `{name}`"
                )));
            }
            macro_at.insert(name, i);
        }
        if let Some(name) = peek_value_def_name(form)? {
            if value_at.contains_key(&name) {
                return Err(ExpandError::new(format!(
                    "duplicate value binding `{name}`"
                )));
            }
            if macro_at.contains_key(&name) {
                return Err(ExpandError::new(format!(
                    "value binding `{name}` conflicts with an existing macro in the same scope"
                )));
            }
            value_at.insert(name, i);
        }
    }

    // Pass 2: expand left-to-right; child modules inherit macros visible so far.
    let mut macros = inherited.clone();
    let mut out_forms = Vec::new();

    for (i, (form, loc)) in forms.into_iter().enumerate() {
        if let Some((name, mut def)) = try_macro_def(&form)? {
            if value_at.contains_key(&name) {
                return Err(ExpandError::new(format!(
                    "macro `{name}` conflicts with an existing value binding in the same scope"
                )));
            }
            def.def_loc = loc;
            macros.insert(name, def);
            assert_macro_graph_dag(&macros)?;
            continue;
        }

        if let Some((mod_name, body)) = try_module_form(&form)? {
            // §8.3: submodule sees parent macros defined earlier (current `macros`).
            let body_out = expand_scope(body, &macros, budget, gensym, source_map, chain)?;
            let mut items = vec![Sexpr::Atom("module".into()), Sexpr::Atom(mod_name)];
            items.extend(body_out);
            out_forms.push(Sexpr::List(items));
            continue;
        }

        // MAC §3.3 / MAC-12: expression macros cannot head a top-level form.
        if let Sexpr::List(items) = &form {
            if let Some(Sexpr::Atom(head)) = items.first() {
                if macros.contains_key(head) {
                    return Err(ExpandError::new(
                        "expression macro cannot be used in declaration position",
                    ));
                }
            }
        }

        // MAC-10 / §8.4: reject uses of macros defined later *in this scope*.
        check_macro_defined_at(&form, i, &macro_at)?;
        let mut cx = ExpandCx {
            macros: &macros,
            budget,
            gensym,
            source_map,
            chain,
        };
        out_forms.push(expand_sexpr(form, loc, &mut cx)?);
    }

    Ok(out_forms)
}

fn peek_value_def_name(form: &Sexpr) -> Result<Option<String>, ExpandError> {
    let Sexpr::List(items) = form else {
        return Ok(None);
    };
    let Some(Sexpr::Atom(head)) = items.first() else {
        return Ok(None);
    };
    match head.as_str() {
        "val" => match items.get(1) {
            Some(Sexpr::Atom(name)) => Ok(Some(name.clone())),
            _ => Err(ExpandError::new("`val` requires a name")),
        },
        "fn" => match items.get(1) {
            Some(Sexpr::Atom(name)) if matches!(items.get(2), Some(Sexpr::List(_))) => {
                Ok(Some(name.clone()))
            }
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

/// Name of a well-formed `(macro name …)` definition, without fully validating
/// the body (validation happens in [`try_macro_def`]).
fn peek_macro_def_name(form: &Sexpr) -> Result<Option<String>, ExpandError> {
    let Sexpr::List(items) = form else {
        return Ok(None);
    };
    let Some(Sexpr::Atom(head)) = items.first() else {
        return Ok(None);
    };
    if head != "macro" {
        return Ok(None);
    }
    match items.get(1) {
        Some(Sexpr::Atom(name)) => Ok(Some(name.clone())),
        _ => Err(ExpandError::new(
            "`macro` requires `(macro name ($params...) -> template)`",
        )),
    }
}

type ModuleBody = Vec<(Sexpr, Option<Loc>)>;

/// `(module name form…)` — body forms keep optional locations when present.
fn try_module_form(form: &Sexpr) -> Result<Option<(String, ModuleBody)>, ExpandError> {
    let Sexpr::List(items) = form else {
        return Ok(None);
    };
    let Some(Sexpr::Atom(head)) = items.first() else {
        return Ok(None);
    };
    if head != "module" {
        return Ok(None);
    }
    let Some(Sexpr::Atom(name)) = items.get(1) else {
        return Err(ExpandError::new("`module` requires `(module name …)`"));
    };
    let body = items[2..]
        .iter()
        .cloned()
        .map(|f| (f, None))
        .collect::<Vec<_>>();
    Ok(Some((name.clone(), body)))
}

/// Walk `expr` and reject calls to macros whose definition appears after `pos`.
fn check_macro_defined_at(
    expr: &Sexpr,
    pos: usize,
    macro_at: &HashMap<String, usize>,
) -> Result<(), ExpandError> {
    match expr {
        Sexpr::Atom(_) => Ok(()),
        Sexpr::List(items) => {
            if let Some(Sexpr::Atom(head)) = items.first() {
                if let Some(&def_pos) = macro_at.get(head) {
                    if def_pos > pos {
                        return Err(ExpandError::new(format!(
                            "macro is not defined at this source position:\n  {head}"
                        )));
                    }
                }
            }
            for item in items {
                check_macro_defined_at(item, pos, macro_at)?;
            }
            Ok(())
        }
    }
}

/// MAC §12.4: templates of defined macros must form a DAG.
fn assert_macro_graph_dag(macros: &HashMap<String, MacroDef>) -> Result<(), ExpandError> {
    let mut edges: HashMap<String, Vec<String>> = HashMap::new();
    for (name, def) in macros {
        let mut refs = Vec::new();
        collect_macro_refs(&def.template, macros, &mut refs);
        edges.insert(name.clone(), refs);
    }

    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    let mut stack = Vec::new();
    for name in edges.keys() {
        if !visited.contains(name) {
            if let Some(cycle) = dfs_cycle(name, &edges, &mut visiting, &mut visited, &mut stack) {
                return Err(ExpandError::new(format!(
                    "macro reference graph contains a cycle: {}",
                    cycle.join(" → ")
                )));
            }
        }
    }
    Ok(())
}

fn collect_macro_refs(expr: &Sexpr, macros: &HashMap<String, MacroDef>, out: &mut Vec<String>) {
    match expr {
        Sexpr::Atom(_) => {}
        Sexpr::List(items) => {
            if let Some(Sexpr::Atom(head)) = items.first() {
                if !head.starts_with('$') && macros.contains_key(head) && !out.contains(head) {
                    out.push(head.clone());
                }
            }
            for item in items {
                collect_macro_refs(item, macros, out);
            }
        }
    }
}

fn dfs_cycle(
    node: &str,
    edges: &HashMap<String, Vec<String>>,
    visiting: &mut HashSet<String>,
    visited: &mut HashSet<String>,
    stack: &mut Vec<String>,
) -> Option<Vec<String>> {
    if visited.contains(node) {
        return None;
    }
    if !visiting.insert(node.to_string()) {
        let start = stack.iter().position(|n| n == node).unwrap_or(0);
        let mut cycle: Vec<String> = stack[start..].to_vec();
        cycle.push(node.to_string());
        return Some(cycle);
    }
    stack.push(node.to_string());
    if let Some(nbrs) = edges.get(node) {
        for n in nbrs {
            if let Some(cycle) = dfs_cycle(n, edges, visiting, visited, stack) {
                return Some(cycle);
            }
        }
    }
    stack.pop();
    visiting.remove(node);
    visited.insert(node.to_string());
    None
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
    check_template_vars(template, &params, rest.as_deref())?;

    Ok(Some((
        name.clone(),
        MacroDef {
            params,
            rest,
            template: template.clone(),
            def_loc: None,
        },
    )))
}

fn parse_macro_params(param_items: &[Sexpr]) -> Result<(Vec<String>, Option<String>), ExpandError> {
    let mut params = Vec::new();
    let mut rest = None;
    let mut seen = HashSet::new();
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
    let mut bound: HashSet<&str> = params.iter().map(|s| s.as_str()).collect();
    if let Some(r) = rest {
        bound.insert(r);
    }
    walk_template_vars(template, &bound)
}

fn walk_template_vars(expr: &Sexpr, bound: &HashSet<&str>) -> Result<(), ExpandError> {
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

struct ExpandCx<'a> {
    macros: &'a HashMap<String, MacroDef>,
    budget: &'a mut u32,
    gensym: &'a mut u64,
    source_map: &'a mut MacroSourceMap,
    chain: &'a mut Vec<String>,
}

fn expand_sexpr(
    expr: Sexpr,
    site: Option<Loc>,
    cx: &mut ExpandCx<'_>,
) -> Result<Sexpr, ExpandError> {
    match expr {
        Sexpr::Atom(_) => Ok(expr),
        Sexpr::List(items) => {
            if let Some(Sexpr::Atom(head)) = items.first() {
                if let Some(def) = cx.macros.get(head).cloned() {
                    return expand_call(head, &def, &items[1..], site, cx);
                }
            }
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                // Preserve enclosing form span as a fallback call site (minimum source map).
                out.push(expand_sexpr(item, site, cx)?);
            }
            Ok(Sexpr::List(out))
        }
    }
}

fn expand_call(
    name: &str,
    def: &MacroDef,
    args: &[Sexpr],
    site: Option<Loc>,
    cx: &mut ExpandCx<'_>,
) -> Result<Sexpr, ExpandError> {
    if *cx.budget == 0 {
        return Err(ExpandError::new("macro expansion limit exceeded"));
    }
    *cx.budget -= 1;

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

    if let (Some(call), Some(def_loc)) = (site, def.def_loc) {
        let mut full_chain = cx.chain.clone();
        full_chain.push(name.to_string());
        cx.source_map.origins.push(ExpansionOrigin {
            macro_name: name.to_string(),
            call_span: (call.start, call.end),
            def_span: (def_loc.start, def_loc.end),
            call_id: call.id,
            def_id: def_loc.id,
            chain: full_chain,
        });
    }

    // Hygiene v0: rename binders introduced by the template, then substitute params.
    let renamed = hygienic_rename(&def.template, cx.gensym);
    let mut subst = HashMap::new();
    for (param, arg) in def.params.iter().zip(args.iter()) {
        subst.insert(param.clone(), arg.clone());
    }
    if let Some(rest_name) = &def.rest {
        let rest_args: Vec<Sexpr> = args[fixed..].to_vec();
        subst.insert(rest_name.clone(), Sexpr::List(rest_args));
    }
    let filled = substitute(&renamed, &subst);
    cx.chain.push(name.to_string());
    let out = expand_sexpr(filled, site, cx)?;
    cx.chain.pop();
    Ok(out)
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
                    "local" => return rename_local(items, env, gensym),
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

fn rename_local(items: &[Sexpr], env: &HashMap<String, String>, gensym: &mut u64) -> Sexpr {
    // (local (decl…) body…)
    if items.len() < 3 {
        return Sexpr::List(
            items
                .iter()
                .map(|i| rename_binders(i, env, gensym))
                .collect(),
        );
    }
    let Sexpr::List(decls) = &items[1] else {
        return Sexpr::List(
            items
                .iter()
                .map(|i| rename_binders(i, env, gensym))
                .collect(),
        );
    };

    let mut child_env = env.clone();
    let mut new_decls = Vec::with_capacity(decls.len());
    for decl in decls {
        if let Sexpr::List(da) = decl {
            if let Some(Sexpr::Atom(kind)) = da.first() {
                match kind.as_str() {
                    "val" | "var" if da.len() >= 2 => {
                        if let Sexpr::Atom(name) = &da[1] {
                            let fresh_name = fresh(gensym);
                            child_env.insert(name.clone(), fresh_name.clone());
                            let mut new_da =
                                vec![Sexpr::Atom(kind.clone()), Sexpr::Atom(fresh_name)];
                            for e in &da[2..] {
                                new_da.push(rename_binders(e, env, gensym));
                            }
                            new_decls.push(Sexpr::List(new_da));
                            continue;
                        }
                    }
                    _ => {}
                }
            }
        }
        new_decls.push(rename_binders(decl, env, gensym));
    }

    let mut out = vec![Sexpr::Atom("local".into()), Sexpr::List(new_decls)];
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

fn loc_of(node: &SyntaxNode, ids: &reciplexa_syntax::SyntaxIdentityMap) -> Loc {
    let range = node.text_range();
    let start = u32::from(range.start());
    let end = u32::from(range.end());
    let id = ids.get(range).unwrap_or(SyntaxNodeId::INVALID);
    Loc { start, end, id }
}

fn top_level_forms(
    root: &SyntaxNode,
    ids: &reciplexa_syntax::SyntaxIdentityMap,
) -> Result<Vec<(Sexpr, Option<Loc>)>, ExpandError> {
    let mut forms = Vec::new();
    for el in root.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) => {
                if t.kind().is_trivia() {
                    continue;
                }
                forms.push((token_to_sexpr(&t)?, None));
            }
            SyntaxElement::Node(n) => match n.kind() {
                SyntaxKind::StructuredComment => continue,
                SyntaxKind::List
                | SyntaxKind::BracketList
                | SyntaxKind::BraceList
                | SyntaxKind::AtExpr => {
                    forms.push((node_to_sexpr(&n)?, Some(loc_of(&n, ids))));
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
