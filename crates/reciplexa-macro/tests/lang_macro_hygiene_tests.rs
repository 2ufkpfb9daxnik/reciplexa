//! Language-macro hygiene, error messages, and expand failures (MAC-001).

use reciplexa_macro::{expand_language, expand_language_with_map, ExpandError};

fn err_msg(src: &str) -> String {
    match expand_language(src) {
        Err(ExpandError { message }) => message,
        Ok(ok) => panic!("expected Err, got Ok({ok})"),
    }
}

#[test]
fn language_parse_error_surfaces() {
    let msg = err_msg("(macro");
    assert!(msg.contains("parse error"), "{msg}");
}

#[test]
fn duplicate_macro_definition() {
    let msg = err_msg(
        r#"(macro twice ($x) -> $x)
(macro twice ($y) -> $y)
(val z 1)"#,
    );
    assert!(msg.contains("duplicate macro"), "{msg}");
}

#[test]
fn macro_conflicts_with_prior_value() {
    let msg = err_msg(
        r#"(val m 1)
(macro m ($x) -> $x)
(val z 1)"#,
    );
    assert!(
        msg.contains("conflicts with an existing value binding"),
        "{msg}"
    );
}

#[test]
fn value_conflicts_with_prior_macro() {
    let msg = err_msg(
        r#"(macro m ($x) -> $x)
(val m 1)
(val z 1)"#,
    );
    assert!(
        msg.contains("conflicts with an existing macro"),
        "{msg}"
    );
}

#[test]
fn duplicate_value_binding_rejected() {
    let msg = err_msg(
        r#"(val a 1)
(val a 2)"#,
    );
    assert!(msg.contains("duplicate value binding"), "{msg}");
}

#[test]
fn reserved_macro_name_rejected() {
    let msg = err_msg(r#"(macro if ($x) -> $x)"#);
    assert!(msg.contains("reserved name"), "{msg}");
}

#[test]
fn val_requires_name() {
    let msg = err_msg(r#"(val (bad) 1)"#);
    assert!(msg.contains("`val` requires a name"), "{msg}");
}

#[test]
fn macro_form_requires_name() {
    let msg = err_msg(r#"(macro ($x) -> $x)"#);
    assert!(msg.contains("`macro` requires"), "{msg}");
}

#[test]
fn module_requires_name() {
    let msg = err_msg(r#"(module (bad) (val x 1))"#);
    assert!(msg.contains("`module` requires"), "{msg}");
}

#[test]
fn expression_macro_not_in_declaration_position() {
    let msg = err_msg(
        r#"(macro twice ($x) -> $x)
(twice 1)"#,
    );
    assert!(
        msg.contains("expression macro cannot be used in declaration position"),
        "{msg}"
    );
}

#[test]
fn forward_macro_use_rejected() {
    let msg = err_msg(
        r#"(val x (later 1))
(macro later ($n) -> $n)"#,
    );
    assert!(msg.contains("not defined at this source position"), "{msg}");
}

#[test]
fn macro_cycle_detected() {
    let msg = err_msg(
        r#"(macro a ($x) -> (b $x))
(macro b ($x) -> (a $x))
(val z 1)"#,
    );
    assert!(msg.contains("cycle"), "{msg}");
}

#[test]
fn macro_requires_arrow() {
    let msg = err_msg(r#"(macro id ($x) $x)"#);
    assert!(msg.contains("legacy form without `->`"), "{msg}");
}

#[test]
fn macro_malformed_shape() {
    let msg = err_msg(r#"(macro id)"#);
    assert!(msg.contains("`macro` requires"), "{msg}");
}

#[test]
fn unbound_template_variable() {
    let msg = err_msg(r#"(macro id ($x) -> $y)"#);
    assert!(msg.contains("unbound macro pattern variable"), "{msg}");
}

#[test]
fn pattern_must_be_dollar() {
    let msg = err_msg(r#"(macro id (x) -> x)"#);
    assert!(msg.contains("must start with `$`"), "{msg}");
}

#[test]
fn pattern_param_not_ident() {
    let msg = err_msg(r#"(macro id (($x)) -> $x)"#);
    assert!(msg.contains("must be an identifier"), "{msg}");
}

#[test]
fn dots_alone_in_pattern_rejected() {
    let msg = err_msg(r#"(macro id (...) -> 1)"#);
    assert!(msg.contains("only valid in templates"), "{msg}");
}

#[test]
fn dots_plus_must_follow_var() {
    let msg = err_msg(r#"(macro id (...+) -> 1)"#);
    assert!(msg.contains("`...+` must follow"), "{msg}");
}

#[test]
fn rest_must_be_last() {
    let msg = err_msg(r#"(macro id ($a ...+ $b) -> $a)"#);
    assert!(msg.contains("must be last"), "{msg}");
}

#[test]
fn duplicate_macro_parameter() {
    let msg = err_msg(r#"(macro id ($x $x) -> $x)"#);
    assert!(msg.contains("duplicate macro parameter"), "{msg}");
}

#[test]
fn arity_mismatch_on_call() {
    let msg = err_msg(
        r#"(macro add1 ($x) -> $x)
(val z (add1 1 2))"#,
    );
    assert!(msg.contains("expects 1 arguments"), "{msg}");
}

#[test]
fn rest_requires_at_least_one() {
    let msg = err_msg(
        r#"(macro wrap ($h $body ...+) -> ($h $body))
(val z (wrap head))"#,
    );
    assert!(msg.contains("requires at least one"), "{msg}");
}

#[test]
fn expands_simple_user_macro_and_module() {
    let (out, map) = expand_language_with_map(
        r#"(macro twice ($x) -> (+ $x $x))
(module inner
  (macro id ($x) -> $x)
  (val y (id 3)))
(val z (twice 2))"#,
    )
    .unwrap();
    assert!(out.contains("(+ 2 2)"), "{out}");
    assert!(out.contains("(module inner"), "{out}");
    assert!(out.contains("(val y 3)"), "{out}");
    assert!(
        map.origins.iter().any(|o| o.macro_name == "twice"),
        "{map:?}"
    );
}

#[test]
fn fn_value_binding_participates_in_pass1() {
    let msg = err_msg(
        r#"(fn f (x) x)
(fn f (y) y)"#,
    );
    assert!(msg.contains("duplicate value binding"), "{msg}");
}

#[test]
fn anonymous_fn_does_not_bind_name() {
    // `(fn (params) body)` is not a named value binding for pass-1.
    let out = expand_language(
        r#"(macro id ($x) -> $x)
(val z (id (fn (x) x)))"#,
    )
    .unwrap();
    assert!(out.contains("(fn (x) x)"), "{out}");
}
