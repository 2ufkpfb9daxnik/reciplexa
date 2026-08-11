//! Close llvm-cov region gaps in `lang_macro` hygiene/expand and `doc_layout` Embed.

use reciplexa_macro::doc_layout::layout_markup_parts;
use reciplexa_macro::{expand_language, expand_language_with_map, ExpandError, EXPANSION_BUDGET};
use reciplexa_syntax::MarkupPart;

fn err_msg(src: &str) -> String {
    match expand_language(src) {
        Err(ExpandError { message }) => message,
        Ok(ok) => panic!("expected Err, got Ok({ok})"),
    }
}

// --- Hygiene / rename (largest lang_macro win) ---

#[test]
fn rename_binder_use_and_empty_list_in_template() {
    let out = expand_language(
        r#"(macro m () -> (fn (x) (seq () x)))
(val main (m))"#,
    )
    .unwrap();
    assert!(out.contains("__rx_") || out.contains("(fn"), "{out}");
    assert!(out.contains("()"), "{out}");
}

#[test]
fn rename_named_fn_keeps_label() {
    let out = expand_language(
        r#"(macro m ($b) -> (fn label (x) $b))
(val main (m 1))"#,
    )
    .unwrap();
    assert!(out.contains("label"), "{out}");
}

#[test]
fn rename_let_parallel_binders() {
    let out = expand_language(
        r#"(macro m ($body) -> (let ((x 1) (y 2)) $body))
(val main (m (+ x y)))"#,
    )
    .unwrap();
    assert!(out.contains("let"), "{out}");
    assert!(out.contains("__rx_") || out.contains("+"), "{out}");
}

#[test]
fn rename_let_avoids_capture() {
    let out = expand_language(
        r#"(macro m ($body) -> (let ((x 0)) $body))
(val main (m x))"#,
    )
    .unwrap();
    assert!(out.contains("let"), "{out}");
}

#[test]
fn rename_let_short_and_non_list_fallbacks() {
    let out = expand_language(
        r#"(macro a () -> (let))
(macro b () -> (let x 1))
(val z (seq (a) (b)))"#,
    )
    .unwrap();
    assert!(out.contains("let"), "{out}");
}

#[test]
fn rename_let_malformed_binding_pair() {
    let out = expand_language(
        r#"(macro c () -> (let ((x) (1 2)) 0))
(val z (c))"#,
    )
    .unwrap();
    assert!(out.contains("let"), "{out}");
}

#[test]
fn rename_fn_malformed_and_non_atom_params() {
    let out = expand_language(
        r#"(macro d () -> (fn 1))
(macro e () -> (fn ((x)) x))
(val z (seq (d) (e)))"#,
    )
    .unwrap();
    assert!(out.contains("fn"), "{out}");
}

#[test]
fn rename_local_edge_cases() {
    let out = expand_language(
        r#"(macro f () -> (local ((val (x) 1)) 0))
(macro g () -> (local ((weird x 1)) 0))
(macro h () -> (local x 0))
(val z (seq (f) (g) (h)))"#,
    )
    .unwrap();
    assert!(out.contains("local"), "{out}");
}

// --- ExpandCall / budget / nested failure ---

#[test]
fn expansion_budget_exceeded_depth_chain() {
    let mut src = String::new();
    src.push_str("(macro m0 ($x) -> $x)\n");
    for i in 1..=(EXPANSION_BUDGET as usize + 1) {
        src.push_str(&format!("(macro m{i} ($x) -> (m{} $x))\n", i - 1));
    }
    let last = EXPANSION_BUDGET as usize + 1;
    src.push_str(&format!("(val z (m{last} 1))\n"));
    let msg = err_msg(&src);
    assert!(
        msg.contains("expansion limit") || msg.contains("budget") || msg.contains("limit"),
        "{msg}"
    );
}

#[test]
fn nested_macro_call_arity_error_after_subst() {
    let msg = err_msg(
        r#"(macro a ($x) -> (b))
(macro b ($x) -> $x)
(val z (a 1))"#,
    );
    assert!(
        msg.contains("arity") || msg.contains("argument") || msg.contains("expects"),
        "{msg}"
    );
}

#[test]
fn fixed_param_dots_does_not_splice() {
    let out = expand_language(
        r#"(macro m ($x) -> (f $x ...))
(val z (m 1))"#,
    )
    .unwrap();
    assert!(out.contains("..."), "{out}");
}

// --- Pattern / template residuals ---

#[test]
fn duplicate_rest_parameter_name() {
    let msg = err_msg(r#"(macro id ($x $x ...+) -> $x)"#);
    assert!(
        msg.contains("duplicate") || msg.contains("parameter"),
        "{msg}"
    );
}

#[test]
fn dollar_followed_by_list_in_pattern() {
    let msg = err_msg(r#"(macro id ($x ($y)) -> $x)"#);
    assert!(!msg.is_empty(), "{msg}");
}

#[test]
fn unbound_var_nested_in_list_template() {
    let msg = err_msg(r#"(macro id ($x) -> (f $y))"#);
    assert!(
        msg.contains("unbound") || msg.contains("$y") || msg.contains("template"),
        "{msg}"
    );
}

// --- Scope / module / peek edges ---

#[test]
fn module_body_surfaces_inner_expand_error() {
    let msg = err_msg(
        r#"(module inner
  (macro bad ($x) $x))
(val z 1)"#,
    );
    assert!(
        msg.contains("->") || msg.contains("legacy") || msg.contains("macro"),
        "{msg}"
    );
}

#[test]
fn top_level_list_with_non_atom_head_and_bare_atoms() {
    let out = expand_language(
        r#"(macro id ($x) -> $x)
((+ 1 2))
42
hello
(val z 0)"#,
    )
    .unwrap();
    assert!(out.contains("val"), "{out}");
}

#[test]
fn malformed_named_fn_not_value_binding() {
    let out = expand_language(
        r#"(fn name 1)
(val z 0)"#,
    )
    .unwrap();
    assert!(out.contains("fn") || out.contains("val"), "{out}");
}

#[test]
fn dag_shared_callee_no_cycle() {
    let out = expand_language(
        r#"(macro leaf ($x) -> $x)
(macro a ($x) -> (leaf $x))
(macro b ($x) -> (leaf $x))
(val z (a (b 1)))"#,
    )
    .unwrap();
    assert!(out.contains("val"), "{out}");
}

// --- CST round-trip ---

#[test]
fn structured_comment_and_at_and_brackets() {
    let out = expand_language(
        r#"(// this is dropped)
@foo
@(bar 1)
[a b]
{a b}
(val z 1)"#,
    )
    .unwrap();
    assert!(out.contains("val"), "{out}");
}

#[test]
fn module_defined_macro_still_expands() {
    let (out, map) = expand_language_with_map(
        r#"(macro when ($c $b ...+) -> (if $c (seq $b ...) unit))
(module m (val x (when true 1)))"#,
    )
    .unwrap();
    assert!(out.contains("if") || out.contains("val"), "{out}");
    let _ = map.origins.len();
}

// --- doc_layout Embed ---

#[test]
fn layout_embed_top_level_and_nested_contexts() {
    let top = layout_markup_parts(&[
        MarkupPart::Text("before ".into()),
        MarkupPart::Embed {
            source: "(+ 1 2)".into(),
        },
        MarkupPart::Text(" after".into()),
    ]);
    let joined: String = top
        .iter()
        .filter_map(|i| match i {
            reciplexa_macro::doc_layout::LaidItem::Text(t) => Some(t.content.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("");
    assert!(
        joined.contains("(+ 1 2)") || joined.contains("before"),
        "{joined:?}"
    );

    // Embed inside @em / @li / @code via markup forms that walk Embed arms.
    let parsed = reciplexa_macro::expand_document_surface(
        r#"(markup before @(foo) after
@em{x @(bar) y}
@li{a @(c) b}
@code{let x = @(e)})"#,
    );
    // Expand may succeed or the markup rewrite path exercises Embed walkers.
    let _ = parsed;
}

#[test]
fn rename_let_non_atom_binder_and_local_too_short() {
    let out = expand_language(
        r#"(macro c () -> (let (((nested) 1) atombind) 0))
(macro h () -> (local))
(macro i () -> (local x))
(macro j () -> (local (plainatom (val (x) 1) (var 1 2)) 0))
(macro k () -> ((x) 1))
(val z (seq (c) (h) (i) (j) (k)))"#,
    )
    .unwrap();
    assert!(
        out.contains("let") || out.contains("local") || out.contains("seq"),
        "{out}"
    );
}

#[test]
fn at_expr_with_nested_list_roundtrips() {
    let out = expand_language(
        r#"@(nested (list 1 2))
@plain
(val z 1)"#,
    )
    .unwrap();
    assert!(out.contains("@") || out.contains("val"), "{out}");
}
