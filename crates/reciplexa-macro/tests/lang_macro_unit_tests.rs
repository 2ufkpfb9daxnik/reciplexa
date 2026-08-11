//! Unit-style expand cases moved out of lang_macro.rs so assert-failure
//! regions do not inflate llvm-cov misses on the implementation file.

use reciplexa_macro::{expand_language, expand_language_with_map};

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
fn rejects_self_recursive_macro_graph() {
    let src = "(macro loop ($x) -> (loop $x))\n(val main (loop 1))";
    let err = expand_language(src).unwrap_err();
    assert!(err.message.contains("cycle"), "unexpected: {}", err.message);
}

#[test]
fn rejects_mutual_recursive_macro_graph() {
    let src = r#"
(macro a ($x) -> (b $x))
(macro b ($x) -> (a $x))
(val main (a 1))
"#;
    let err = expand_language(src).unwrap_err();
    assert!(err.message.contains("cycle"), "unexpected: {}", err.message);
}

#[test]
fn allows_forward_use_of_prior_macro() {
    let src = r#"
(macro unless ($condition $expression) -> (if $condition unit $expression))
(macro unless-ready ($body ...+) -> (unless ready? $body ...))
(val main (unless-ready 1))
"#;
    let out = expand_language(src).unwrap();
    assert_eq!(out, "(val main (if ready? unit 1))");
}

#[test]
fn nested_module_sees_parent_macro() {
    let src = r#"
(macro when ($condition $body ...+) -> (if $condition (seq $body ...) unit))
(module rendering
  (val render-if-ready (when ready? 1)))
"#;
    let out = expand_language(src).unwrap();
    assert!(out.contains("(module rendering"), "{out}");
    assert!(out.contains("(if ready? (seq 1) unit)"), "{out}");
    assert!(!out.contains("(when "), "{out}");
}

#[test]
fn sibling_module_macro_not_visible() {
    let src = r#"
(module a
  (macro secret ($x) -> $x)
  (val x (secret 1)))
(module b
  (val y (secret 2)))
"#;
    let out = expand_language(src).unwrap();
    // `secret` stays as a normal call in sibling `b` (not expanded).
    assert!(out.contains("(secret 2)"), "{out}");
    assert!(out.contains("(val x 1)"), "{out}");
}

#[test]
fn child_macro_does_not_leak_to_parent() {
    let src = r#"
(module a
  (macro secret ($x) -> $x)
  (val x (secret 1)))
(val y (secret 2))
"#;
    let out = expand_language(src).unwrap();
    assert!(out.contains("(secret 2)"), "{out}");
}

#[test]
fn provenance_records_call_and_def_ids() {
    let src = "(macro when ($c $b) -> (if $c $b unit))\n(val main (when true 1))";
    let (out, map) = expand_language_with_map(src).unwrap();
    assert_eq!(out, "(val main (if true 1 unit))");
    assert!(!map.origins.is_empty(), "expected provenance origins");
    let o = &map.origins[0];
    assert_eq!(o.macro_name, "when");
    assert!(o.call_id.is_valid(), "call SyntaxNodeId");
    assert!(o.def_id.is_valid(), "def SyntaxNodeId");
    assert!(o.call_span.0 < o.call_span.1);
    assert!(o.def_span.0 < o.def_span.1);
    assert!(o.chain.iter().any(|n| n == "when"));
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
        assert!(err.message.contains("reserved"), "{name}: {}", err.message);
    }
}

#[test]
fn rejects_macro_use_before_definition() {
    let src = r#"
(val result (when true 1))
(macro when ($condition $body ...+) -> (if $condition (seq $body ...) unit))
"#;
    let err = expand_language(src).unwrap_err();
    assert!(
        err.message.contains("not defined") && err.message.contains("when"),
        "unexpected: {}",
        err.message
    );
}

#[test]
fn rejects_macro_in_declaration_position() {
    let src = r#"
(macro define-values ($a $b) -> (seq $a $b))
(define-values first second)
"#;
    let err = expand_language(src).unwrap_err();
    assert!(
        err.message.contains("declaration position"),
        "unexpected: {}",
        err.message
    );
}

#[test]
fn rejects_macro_value_name_collision() {
    let src = r#"
(val when 1)
(macro when ($x) -> $x)
(val main 0)
"#;
    let err = expand_language(src).unwrap_err();
    assert!(
        err.message.contains("conflicts"),
        "unexpected: {}",
        err.message
    );
}

#[test]
fn hygiene_renames_local_var_binder() {
    let src = r#"
(macro m ($body) -> (local ((var x 0)) $body))
(val main (m x))
"#;
    let out = expand_language(src).unwrap();
    assert!(out.contains("__rx_"), "expected gensym: {out}");
    assert!(
        !out.contains("(local ((var x 0)) x)"),
        "must not capture: {out}"
    );
}
