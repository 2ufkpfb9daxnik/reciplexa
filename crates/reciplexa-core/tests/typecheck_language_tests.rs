//! TYP-001: language-kernel typecheck via Core infer (not document surface).

use reciplexa_core::{typecheck_language_source, CoreType};

#[test]
fn identity_app_types_as_int() {
    let ty = typecheck_language_source("(val main ((fn (x) x) 1))").unwrap();
    assert_eq!(ty, CoreType::Int);
}

#[test]
fn dynamic_unifies_as_gradual_stub() {
    use reciplexa_core::unify::{unify, Subst};
    let mut s = Subst::new();
    assert!(unify(&CoreType::dyn_any(), &CoreType::Number, &mut s).is_ok());
    assert!(unify(&CoreType::String, &CoreType::dyn_any(), &mut s).is_ok());
}

#[test]
fn match_non_exhaustive_errors() {
    let err = typecheck_language_source(
        r#"
(data option (none) (some x))
(val main (match (some 1) (some x -> x)))
"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("non-exhaustive"),
        "unexpected: {}",
        err.message
    );
}

#[test]
fn match_exhaustive_option_ok() {
    let ty = typecheck_language_source(
        r#"
(data option (none) (some x))
(val main (match (some 1) (none -> 0) (some x -> x)))
"#,
    )
    .unwrap();
    assert_eq!(ty, CoreType::Int);
}

#[test]
fn match_unreachable_after_wildcard_errors() {
    let err = typecheck_language_source(
        r#"
(data option (none) (some x))
(val main (match (some 1) (_ -> 0) (some x -> x)))
"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("unreachable"),
        "unexpected: {}",
        err.message
    );
}

#[test]
fn match_unreachable_duplicate_ctor_errors() {
    let err = typecheck_language_source(
        r#"
(data option (none) (some x))
(val main (match (some 1) (none -> 0) (some x -> x) (none -> 1)))
"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("unreachable"),
        "unexpected: {}",
        err.message
    );
}

#[test]
fn unit_literal_types_as_unit() {
    let ty = typecheck_language_source("(val main unit)").unwrap();
    assert_eq!(ty, CoreType::Unit);
}

#[test]
fn record_field_types() {
    let ty = typecheck_language_source(
        r#"
(val report (record (title "Report") (page-count 10)))
(val main (field report title))
"#,
    )
    .unwrap();
    assert_eq!(ty, CoreType::String);
}

#[test]
fn if_branch_union_when_types_differ() {
    let ty = typecheck_language_source(
        r#"
(val main (if true 42 "unknown"))
"#,
    )
    .unwrap();
    match ty {
        CoreType::Union(members) => {
            assert!(members.contains(&CoreType::Int));
            assert!(members.contains(&CoreType::String));
        }
        other => panic!("expected Union, got {other:?}"),
    }
}

#[test]
fn occurrence_typing_is_some_string() {
    use reciplexa_core::check::{infer_expr, TypeEnv};
    use reciplexa_core::expr::{CoreExpr, CoreLiteral};
    use reciplexa_core::unify::Subst;
    use reciplexa_source::range::TextRange;

    let opt = CoreType::Variant {
        variants: vec![
            ("none".into(), None),
            ("some".into(), Some(CoreType::String)),
        ],
    };
    let mut env = TypeEnv::new();
    env.insert("x", opt);
    env.insert(
        "is-some",
        CoreType::Fun {
            args: vec![CoreType::dyn_any()],
            ret: Box::new(CoreType::Bool),
            effects: Default::default(),
        },
    );
    env.insert(
        "string-length",
        CoreType::Fun {
            args: vec![CoreType::String],
            ret: Box::new(CoreType::Number),
            effects: Default::default(),
        },
    );
    // (if (is-some x) (string-length x) 0)
    let expr = CoreExpr::If {
        cond: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("is-some".into())),
            args: vec![CoreExpr::Var("x".into())],
        }),
        then_branch: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("string-length".into())),
            args: vec![CoreExpr::Var("x".into())],
        }),
        else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
    };
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &env, &mut subst, TextRange::EMPTY).unwrap();
    match subst.apply(&ty) {
        CoreType::Union(members) => {
            assert!(members.contains(&CoreType::Number));
            assert!(members.contains(&CoreType::Int));
        }
        CoreType::Number | CoreType::Int => {}
        other => panic!("expected Number/Int or Union, got {other:?}"),
    }
}

#[test]
fn occurrence_typing_number_pred() {
    use reciplexa_core::check::{infer_expr, TypeEnv};
    use reciplexa_core::expr::{CoreExpr, CoreLiteral};
    use reciplexa_core::unify::Subst;
    use reciplexa_source::range::TextRange;

    let mut env = TypeEnv::new();
    env.insert("x", CoreType::Union(vec![CoreType::Int, CoreType::String]));
    env.insert(
        "number?",
        CoreType::Fun {
            args: vec![CoreType::dyn_any()],
            ret: Box::new(CoreType::Bool),
            effects: Default::default(),
        },
    );
    env.insert(
        "+",
        CoreType::Fun {
            args: vec![CoreType::Number, CoreType::Number],
            ret: Box::new(CoreType::Number),
            effects: Default::default(),
        },
    );
    // (if (number? x) (+ x 1) 0)
    let expr = CoreExpr::If {
        cond: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("number?".into())),
            args: vec![CoreExpr::Var("x".into())],
        }),
        then_branch: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("+".into())),
            args: vec![
                CoreExpr::Var("x".into()),
                CoreExpr::Lit(CoreLiteral::Int(1)),
            ],
        }),
        else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
    };
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &env, &mut subst, TextRange::EMPTY).unwrap();
    assert_eq!(subst.apply(&ty), CoreType::Int);
}

#[test]
fn occurrence_typing_is_none() {
    use reciplexa_core::check::{infer_expr, TypeEnv};
    use reciplexa_core::expr::{CoreExpr, CoreLiteral};
    use reciplexa_core::unify::Subst;
    use reciplexa_source::range::TextRange;

    let opt = CoreType::Variant {
        variants: vec![
            ("none".into(), None),
            ("some".into(), Some(CoreType::Number)),
        ],
    };
    let mut env = TypeEnv::new();
    env.insert("x", opt);
    env.insert(
        "is-none",
        CoreType::Fun {
            args: vec![CoreType::dyn_any()],
            ret: Box::new(CoreType::Bool),
            effects: Default::default(),
        },
    );
    // (if (is-none x) 0 x) — else branch narrows x to Number (some payload)
    let expr = CoreExpr::If {
        cond: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("is-none".into())),
            args: vec![CoreExpr::Var("x".into())],
        }),
        then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(0))),
        else_branch: Box::new(CoreExpr::Var("x".into())),
    };
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &env, &mut subst, TextRange::EMPTY).unwrap();
    assert_eq!(subst.apply(&ty), CoreType::Int);
}

#[test]
fn optional_record_field_access_types_as_option() {
    use reciplexa_core::check::{infer_expr, TypeEnv};
    use reciplexa_core::expr::CoreExpr;
    use reciplexa_core::unify::Subst;
    use reciplexa_source::range::TextRange;

    let mut env = TypeEnv::new();
    env.insert(
        "r",
        CoreType::Record {
            fields: vec![
                ("title".into(), CoreType::String),
                (
                    "subtitle".into(),
                    CoreType::OptionalField(Box::new(CoreType::String)),
                ),
            ],
        },
    );
    let expr = CoreExpr::RecordGet {
        record: Box::new(CoreExpr::Var("r".into())),
        field: "subtitle".into(),
    };
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &env, &mut subst, TextRange::EMPTY).unwrap();
    match ty {
        CoreType::Variant { variants } => {
            assert!(variants.iter().any(|(t, p)| t == "none" && p.is_none()));
            assert!(variants
                .iter()
                .any(|(t, p)| t == "some" && matches!(p, Some(CoreType::String))));
        }
        other => panic!("expected option Variant, got {other:?}"),
    }
}

#[test]
fn perform_adds_effect_to_fun() {
    use reciplexa_core::unify::Subst;
    use reciplexa_core::{elaborate_source, infer_with_effects, TypeEnv};
    use reciplexa_source::range::TextRange;

    let expr = elaborate_source(r#"(val main (fn () (perform log "hi")))"#).unwrap();
    // peel let to get the fn
    let reciplexa_core::CoreExpr::Let { value, .. } = expr else {
        panic!("expected Let");
    };
    let mut subst = Subst::new();
    let (ty, residual) =
        infer_with_effects(&value, &TypeEnv::new(), &mut subst, TextRange::EMPTY).unwrap();
    assert!(
        residual.ops.is_empty(),
        "lambda suspends effects: {residual:?}"
    );
    match subst.apply(&ty) {
        CoreType::Fun { effects, .. } => {
            assert_eq!(effects.ops, vec!["log".to_string()]);
        }
        other => panic!("expected Fun, got {other:?}"),
    }
}

#[test]
fn handle_removes_effect_from_residual() {
    use reciplexa_core::elaborate_source;
    use reciplexa_core::unify::Subst;
    use reciplexa_core::{infer_with_effects, typecheck_language_source, TypeEnv};
    use reciplexa_source::range::TextRange;

    let src = r#"(val main (handle log (fn (msg) msg) (perform log "ok")))"#;
    let ty = typecheck_language_source(src).unwrap();
    assert_eq!(ty, CoreType::String);

    let expr = elaborate_source(src).unwrap();
    let reciplexa_core::CoreExpr::Let { value, .. } = expr else {
        panic!("expected Let");
    };
    let mut subst = Subst::new();
    let (_ty, residual) =
        infer_with_effects(&value, &TypeEnv::new(), &mut subst, TextRange::EMPTY).unwrap();
    assert!(
        !residual.ops.iter().any(|o| o == "log"),
        "handle should remove log: {residual:?}"
    );
}

/// MEM-001 §22.1: ordinary heap allocation (records) is not an Effect.
#[test]
fn ordinary_allocation_absent_from_effect_row() {
    use reciplexa_core::elaborate_source;
    use reciplexa_core::unify::Subst;
    use reciplexa_core::{infer_with_effects, TypeEnv};
    use reciplexa_source::range::TextRange;

    let src = r#"(val main (let ((r (record (x 1) (y 2)))) r))"#;
    let expr = elaborate_source(src).unwrap();
    let reciplexa_core::CoreExpr::Let { value, .. } = expr else {
        panic!("expected Let");
    };
    let mut subst = Subst::new();
    let (_ty, residual) =
        infer_with_effects(&value, &TypeEnv::new(), &mut subst, TextRange::EMPTY).unwrap();
    assert!(
        residual.ops.is_empty(),
        "record allocation must not appear in effect row: {residual:?}"
    );
}

#[test]
fn var_removes_local_state_effect_from_residual() {
    use reciplexa_core::elaborate_source;
    use reciplexa_core::unify::Subst;
    use reciplexa_core::{infer_with_effects, TypeEnv};
    use reciplexa_source::range::TextRange;

    let src = r#"(val main (var count 0 (seq (set count 1) count)))"#;
    let expr = elaborate_source(src).unwrap();
    let mut subst = Subst::new();
    let env = TypeEnv::new();
    let (_ty, residual) = infer_with_effects(&expr, &env, &mut subst, TextRange::EMPTY).unwrap();
    assert!(
        !residual.ops.iter().any(|o| o.starts_with("local-state/")),
        "var should strip local-state: {residual:?}"
    );
}

#[test]
fn int_plus_int_types_as_int() {
    let ty = typecheck_language_source("(val main (+ 1 2))").unwrap();
    assert_eq!(ty, CoreType::Int);
}

#[test]
fn mixed_numeric_promotes_to_f64() {
    let ty = typecheck_language_source("(val main (+ 1 1.0))").unwrap();
    assert_eq!(ty, CoreType::F64);
}

#[test]
fn int_division_types_as_f64() {
    let ty = typecheck_language_source("(val main (/ 4 2))").unwrap();
    assert_eq!(ty, CoreType::F64);
}

#[test]
fn integer_literal_types_as_int() {
    let ty = typecheck_language_source("(val main 42)").unwrap();
    assert_eq!(ty, CoreType::Int);
}

#[test]
fn f64_literal_types_as_f64() {
    let ty = typecheck_language_source("(val main 1.5)").unwrap();
    assert_eq!(ty, CoreType::F64);
}

#[test]
fn int_div_and_mod_typecheck() {
    let ty = typecheck_language_source("(val main (int-div 7 2))").unwrap();
    assert_eq!(ty, CoreType::Int);
    let ty = typecheck_language_source("(val main (mod 7 2))").unwrap();
    assert_eq!(ty, CoreType::Int);
}

#[test]
fn singleton_type_alias_elaborates() {
    use reciplexa_core::elaborate::elaborate_with_data;
    use reciplexa_core::ty::SingletonValue;
    let src = r#"(type-alias page-kind (union "page" "slide")) (val main 1)"#;
    let (_, data) = elaborate_with_data(src).unwrap();
    assert_eq!(
        data.type_aliases.get("page-kind"),
        Some(&CoreType::Union(vec![
            CoreType::Singleton(SingletonValue::String("page".into())),
            CoreType::Singleton(SingletonValue::String("slide".into())),
        ]))
    );
}

#[test]
fn parameterized_option_types_as_app() {
    let src = r#"
(data option ((a type)) none (some a))
(val main (some 42))
"#;
    let ty = typecheck_language_source(src).unwrap();
    assert_eq!(
        ty,
        CoreType::App {
            ctor: "option".into(),
            args: vec![CoreType::Int],
        }
    );
}

#[test]
fn annotated_option_binding_checks_against_type_app() {
    let src = r#"
(data option ((a type)) none (some a))
(type value (option int))
(val value none)
(val main value)
"#;
    let ty = typecheck_language_source(src).unwrap();
    assert_eq!(
        ty,
        CoreType::App {
            ctor: "option".into(),
            args: vec![CoreType::Int],
        }
    );
}

#[test]
fn forall_annotation_identity_instantiates_at_use() {
    let src = r#"
(type identity
  (forall ((a type))
    (fn (a) a)))
(val identity
  (fn (x) x))
(val main (identity 7))
"#;
    let ty = typecheck_language_source(src).unwrap();
    assert_eq!(ty, CoreType::Int);
}

#[test]
fn result_ok_generalizes_error_param() {
    let src = r#"
(data result ((a type) (e type)) (ok a) (err e))
(val main (ok 1))
"#;
    let ty = typecheck_language_source(src).unwrap();
    match ty {
        CoreType::Forall { params, body } => {
            assert_eq!(params, vec![("e".into(), "type".into())]);
            assert_eq!(
                *body,
                CoreType::App {
                    ctor: "result".into(),
                    args: vec![CoreType::Int, CoreType::Name("e".into())],
                }
            );
        }
        other => panic!("expected forall, got {other:?}"),
    }
}

#[test]
fn var_result_ok_requires_annotation() {
    let err = typecheck_language_source(
        r#"
(data result ((a type) (e type)) (ok a) (err e))
(val main (var success (ok 1) success))
"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("ungeneralized") || err.message.contains("annotation"),
        "{}",
        err.message
    );
}

#[test]
fn annotation_mismatch_is_static_error() {
    let err = typecheck_language_source(
        r#"
(type value str)
(val value 42)
(val main value)
"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("Mismatch") || err.message.contains("expected"),
        "{}",
        err.message
    );
}

#[test]
fn orphan_type_annotation_rejected() {
    let err = reciplexa_core::elaborate_source("(type missing int)\n(val main 1)").unwrap_err();
    assert!(
        err.message.contains("対応する値binding") || err.message.contains("missing"),
        "{}",
        err.message
    );
}

#[test]
fn forall_annotation_ok_and_expansive_rejected() {
    let ty = typecheck_language_source(
        r#"
(type identity
  (forall ((a type))
    (fn (a) a)))
(val identity (fn (x) x))
(val main (identity 3))
"#,
    )
    .unwrap();
    assert_eq!(ty, CoreType::Int);

    let err = typecheck_language_source(
        r#"
(type identity
  (forall ((a type))
    (fn (a) a)))
(val make-identity (fn () (fn (x) x)))
(val identity (make-identity))
(val main (identity 1))
"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("expansive") || err.message.contains("一般化"),
        "{}",
        err.message
    );
}

#[test]
fn var_store_type_fixed_and_union_annotation() {
    let err = typecheck_language_source(
        r#"
(val main
  (var value 0
    (seq (set value "text") value)))
"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("Mismatch") || err.message.contains("expected"),
        "{}",
        err.message
    );

    let ty = typecheck_language_source(
        r#"
(val main
  (local
    (type value (union int str))
    (var value 0)
    (seq (set value "text") value)))
"#,
    )
    .unwrap();
    assert!(
        matches!(ty, CoreType::Union(_)) || matches!(ty, CoreType::String),
        "{ty:?}"
    );
}

#[test]
fn local_state_escape_rejected_statically() {
    let err = typecheck_language_source(
        r#"
(val main
  (var count 0
    (fn ()
      (set count (+ count 1))
      count)))
"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("escapes") || err.message.contains("local state"),
        "{}",
        err.message
    );
}

#[test]
fn partial_letrec_annotation_infers_partner() {
    let ty = typecheck_language_source(
        r#"
(val main
  (rec
    (type even? (fn (int) bool))
    (val even?
      (fn (n)
        (if (= n 0) true (odd? (- n 1)))))
    (val odd?
      (fn (n)
        (if (= n 0) false (even? (- n 1)))))
    (odd? 9)))
"#,
    )
    .unwrap();
    assert_eq!(ty, CoreType::Bool);
}

#[test]
fn effectful_annotated_fn_generalizes() {
    let ty = typecheck_language_source(
        r#"
(type log-value
  (forall ((a type))
    (fn a a (effects log))))
(val log-value
  (fn (x)
    (seq (perform log "trace") x)))
(val main (log-value 1))
"#,
    )
    .unwrap();
    assert_eq!(ty, CoreType::Int);
}

#[test]
fn markup_fragment_type_name_parses() {
    let (_, data) = reciplexa_core::elaborate_with_data(
        r#"
(type-alias body markup-fragment)
(val main 1)
"#,
    )
    .unwrap();
    assert_eq!(
        data.type_aliases.get("body"),
        Some(&CoreType::Name("markup-fragment".into()))
    );
}
