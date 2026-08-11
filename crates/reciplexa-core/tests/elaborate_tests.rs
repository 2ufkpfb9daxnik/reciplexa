//! Surface → Core elaborator tests (BND-001).

use reciplexa_core::check::{infer_expr, TypeEnv};
use reciplexa_core::elaborate::{elaborate_source, elaborate_with_data};
use reciplexa_core::expr::{CoreExpr, CoreLiteral};
use reciplexa_core::ty::CoreType;
use reciplexa_core::unify::Subst;
use reciplexa_source::offset::ByteOffset;
use reciplexa_source::range::TextRange;

fn range() -> TextRange {
    TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap()
}

#[test]
fn elaborates_val_main_identity_app() {
    let expr = elaborate_source("(val main ((fn (x) x) 42))").unwrap();
    assert!(matches!(
        expr,
        CoreExpr::Let {
            name,
            value,
            body,
        } if name == "main"
            && matches!(*body, CoreExpr::Var(ref n) if n == "main")
            && matches!(
                *value,
                CoreExpr::App {
                    ref fun,
                    ref args,
                } if matches!(**fun, CoreExpr::Lambda { ref params, .. } if params == &["x".to_string()])
                    && args == &[CoreExpr::Lit(CoreLiteral::Int(42))]
            )
    ));
}

#[test]
fn elaborates_if_true_branch() {
    let expr = elaborate_source("(val main (if true 1 2))").unwrap();
    let CoreExpr::Let { value, .. } = expr else {
        panic!("expected Let");
    };
    assert_eq!(
        *value,
        CoreExpr::If {
            cond: Box::new(CoreExpr::Lit(CoreLiteral::Bool(true))),
            then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Int(2))),
        }
    );
}

#[test]
fn elaborates_sequential_vals_preferring_main() {
    let expr = elaborate_source(
        r#"
(val f (fn (x) x))
(val main (f 42))
"#,
    )
    .unwrap();
    // let f = … in let main = … in main
    let CoreExpr::Let {
        name: f_name,
        body: outer_body,
        ..
    } = expr
    else {
        panic!("expected outer Let");
    };
    assert_eq!(f_name, "f");
    let CoreExpr::Let {
        name: main_name,
        body,
        ..
    } = *outer_body
    else {
        panic!("expected inner Let");
    };
    assert_eq!(main_name, "main");
    assert!(matches!(*body, CoreExpr::Var(ref n) if n == "main"));
}

#[test]
fn elaborates_surface_let_to_nested_core_lets() {
    let expr = elaborate_source(
        r#"
(val main
  (let ((x 1)
        (y x))
    y))
"#,
    )
    .unwrap();
    let CoreExpr::Let { value, .. } = expr else {
        panic!("expected Let");
    };
    // let x = 1 in let y = x in y
    let CoreExpr::Let {
        name: x, body: mid, ..
    } = *value
    else {
        panic!("expected let x");
    };
    assert_eq!(x, "x");
    let CoreExpr::Let {
        name: y,
        body,
        value: y_init,
        ..
    } = *mid
    else {
        panic!("expected let y");
    };
    assert_eq!(y, "y");
    assert!(matches!(*y_init, CoreExpr::Var(ref n) if n == "x"));
    assert!(matches!(*body, CoreExpr::Var(ref n) if n == "y"));
}

#[test]
fn typechecks_elaborated_identity_app() {
    let expr = elaborate_source("(val main ((fn (x) x) 42))").unwrap();
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
    assert_eq!(subst.apply(&ty), CoreType::Int);
}

#[test]
fn typechecks_elaborated_if() {
    let expr = elaborate_source("(val main (if true 1 2))").unwrap();
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
    assert_eq!(subst.apply(&ty), CoreType::Int);
}

#[test]
fn rejects_graphics_forms() {
    let err = elaborate_source("(circle 10 20 5)").unwrap_err();
    assert!(err.message.contains("circle"));
}

#[test]
fn elaborates_named_fn_sugar() {
    let expr = elaborate_source("(fn id (x) x)\n(val main (id 7))").unwrap();
    let CoreExpr::Let { name, .. } = &expr else {
        panic!("expected Let");
    };
    assert_eq!(name, "id");
}

#[test]
fn elaborates_data_option_and_match() {
    let expr = elaborate_source(
        r#"
(data option (none) (some x))
(val main (match (some 1) (none -> 0) (some x -> x)))
"#,
    )
    .unwrap();
    let CoreExpr::Let { value, .. } = expr else {
        panic!("expected Let");
    };
    let CoreExpr::Match { scrutinee, arms } = *value else {
        panic!("expected Match");
    };
    assert!(matches!(
        *scrutinee,
        CoreExpr::Variant {
            ref tag,
            payload: Some(_),
        } if tag == "some"
    ));
    assert_eq!(arms.len(), 2);
    assert_eq!(arms[0].tag(), Some("none"));
    assert!(matches!(
        arms[0].pattern,
        reciplexa_core::CorePattern::Variant { payload: None, .. }
    ));
    assert_eq!(arms[1].tag(), Some("some"));
    assert!(matches!(
        &arms[1].pattern,
        reciplexa_core::CorePattern::Variant {
            payload: Some(p),
            ..
        } if matches!(p.as_ref(), reciplexa_core::CorePattern::Bind(n) if n == "x")
    ));
}

#[test]
fn elaborates_parameterized_data_option() {
    use reciplexa_core::elaborate_with_data;
    let (expr, data) = elaborate_with_data(
        r#"
(data option
  ((a type))
  none
  (some a))
(val main (match (some 1) (none -> 0) (some x -> x)))
"#,
    )
    .unwrap();
    assert_eq!(data.type_params.get("option"), Some(&vec!["a".to_string()]));
    assert_eq!(data.ctors.get("none"), Some(&0));
    assert_eq!(data.ctors.get("some"), Some(&1));
    assert_eq!(
        data.ctor_type.get("some").map(String::as_str),
        Some("option")
    );
    let CoreExpr::Let { value, .. } = expr else {
        panic!("expected Let");
    };
    assert!(matches!(*value, CoreExpr::Match { .. }));

    let empty = elaborate_with_data(
        r#"
(data option
  ()
  none)
(val main none)
"#,
    )
    .unwrap_err();
    assert!(
        empty.message.contains("empty") || empty.message.contains("constructor"),
        "got: {}",
        empty.message
    );
}

#[test]
fn rejects_optional_in_record_pattern() {
    let err = elaborate_source(
        r#"
(val main
  (match (record (title "t"))
    (record (optional subtitle x) -> x)
    (_ -> unit)))
"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("optional") && err.message.contains("18.5"),
        "got: {}",
        err.message
    );
}

#[test]
fn rejects_negative_recursion_in_data() {
    let err = elaborate_source(
        r#"
(data bad
  (bad (fn bad unit)))
(val main (bad (fn (x) unit)))
"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("positivity") || err.message.contains("negative"),
        "got: {}",
        err.message
    );
}

#[test]
fn allows_positive_recursion_in_data() {
    let (expr, data) = elaborate_with_data(
        r#"
(data tree
  ((a type))
  empty
  (node a (tree a) (tree a)))
(val main empty)
"#,
    )
    .unwrap();
    assert!(data.data_ctors.contains_key("tree"));
    assert_eq!(
        data.type_variances.get("tree").and_then(|v| v.get("a")),
        Some(&reciplexa_core::Variance::Covariant)
    );
    assert!(matches!(expr, CoreExpr::Let { .. }));
}

#[test]
fn infers_phantom_type_param() {
    let (_, data) = elaborate_with_data(
        r#"
(data identifier
  ((domain type))
  (identifier int))
(val main unit)
"#,
    )
    .unwrap();
    assert_eq!(
        data.type_variances
            .get("identifier")
            .and_then(|v| v.get("domain")),
        Some(&reciplexa_core::Variance::Phantom)
    );
}

#[test]
fn rejects_mutual_negative_recursion_in_data_rec_group() {
    let err = elaborate_source(
        r#"
(rec
  (data a (a-con (fn b unit)))
  (data b (b-con (fn a unit))))
(val main unit)
"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("positivity") || err.message.contains("negative"),
        "got: {}",
        err.message
    );
}

#[test]
fn allows_mutual_positive_recursion_in_data_rec_group() {
    elaborate_source(
        r#"
(rec
  (data expression
    (literal int)
    (sequence (list statement)))
  (data statement
    (evaluate expression)
    (return expression)))
(val main unit)
"#,
    )
    .unwrap();
}

#[test]
fn rejects_mixed_data_val_rec_group() {
    let err = elaborate_source(
        r#"
(rec
  (data option (none) (some x))
  (val f (fn (x) x)))
(val main unit)
"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("mix") || err.message.contains("10.3"),
        "got: {}",
        err.message
    );
}

#[test]
fn allows_recursive_in_fn_result_position() {
    elaborate_source(
        r#"
(data stream-node
  ((a type))
  (stream-node a (fn unit (stream-node a))))
(val main unit)
"#,
    )
    .unwrap();
}

#[test]
fn elaborates_wildcard_bind_and_nested_patterns() {
    let expr = elaborate_source(
        r#"
(data option (none) (some x))
(val main
  (match (some (some 1))
    (some (some item) -> item)
    (some _ -> 0)
    (_ -> -1)))
"#,
    )
    .unwrap();
    let CoreExpr::Let { value, .. } = expr else {
        panic!("expected Let");
    };
    let CoreExpr::Match { arms, .. } = *value else {
        panic!("expected Match");
    };
    assert_eq!(arms.len(), 3);
    assert!(matches!(
        &arms[0].pattern,
        reciplexa_core::CorePattern::Variant {
            tag,
            payload: Some(inner),
        } if tag == "some"
            && matches!(
                inner.as_ref(),
                reciplexa_core::CorePattern::Variant {
                    tag: t2,
                    payload: Some(b),
                } if t2 == "some"
                    && matches!(b.as_ref(), reciplexa_core::CorePattern::Bind(n) if n == "item")
            )
    ));
    assert!(matches!(
        &arms[1].pattern,
        reciplexa_core::CorePattern::Variant {
            tag,
            payload: Some(inner),
        } if tag == "some" && matches!(inner.as_ref(), reciplexa_core::CorePattern::Wildcard)
    ));
    assert!(matches!(
        arms[2].pattern,
        reciplexa_core::CorePattern::Wildcard
    ));

    let bind = elaborate_source(
        r#"
(data option (none) (some x))
(val main (match (some 1) (bind v -> v)))
"#,
    )
    .unwrap();
    let CoreExpr::Let { value, .. } = bind else {
        panic!("expected Let");
    };
    let CoreExpr::Match { arms, .. } = *value else {
        panic!("expected Match");
    };
    assert!(matches!(
        &arms[0].pattern,
        reciplexa_core::CorePattern::Bind(n) if n == "v"
    ));
}

#[test]
fn elaborates_record_patterns() {
    let expr = elaborate_source(
        r#"
(val report (record (title "Report") (page-count 10) (extra true)))
(val main
  (match report
    (record
      (title title)
      (page-count count)
    ->
      (tuple title count))))
"#,
    )
    .unwrap();
    let CoreExpr::Let { body, .. } = expr else {
        panic!("expected Let");
    };
    let CoreExpr::Let { value, .. } = *body else {
        panic!("expected inner Let");
    };
    let CoreExpr::Match { arms, .. } = *value else {
        panic!("expected Match");
    };
    assert!(matches!(
        &arms[0].pattern,
        reciplexa_core::CorePattern::Record { fields }
            if fields.len() == 2
                && fields[0].0 == "title"
                && matches!(&fields[0].1, reciplexa_core::CorePattern::Bind(n) if n == "title")
                && fields[1].0 == "page-count"
                && matches!(&fields[1].1, reciplexa_core::CorePattern::Bind(n) if n == "count")
    ));

    let dup = elaborate_source(
        r#"
(val main
  (match (record (title "T"))
    (record (title a) (title b) -> a)))
"#,
    )
    .unwrap_err();
    assert!(
        dup.message.contains("duplicate"),
        "expected duplicate field error, got: {}",
        dup.message
    );
}

#[test]
fn rejects_old_match_arm_without_arrow() {
    let err = elaborate_source(
        r#"
(data option (none) (some x))
(val main (match (some 1) (none 0) ((some x) x)))
"#,
    )
    .unwrap_err();
    assert!(
        err.message.contains("->"),
        "expected arrow requirement, got: {}",
        err.message
    );
}

#[test]
fn elaborates_ambient_effect_apps() {
    let log = elaborate_source(r#"(val main (log "hi"))"#).unwrap();
    let CoreExpr::Let { value, .. } = log else {
        panic!("expected Let");
    };
    match *value {
        CoreExpr::Perform { op, arg } => {
            assert_eq!(op, "log");
            assert_eq!(*arg, CoreExpr::Lit(CoreLiteral::String("hi".into())));
        }
        other => panic!("expected Perform, got {other:?}"),
    }

    let rand = elaborate_source("(val main (random))").unwrap();
    let CoreExpr::Let { value, .. } = rand else {
        panic!("expected Let");
    };
    match *value {
        CoreExpr::Perform { op, arg } => {
            assert_eq!(op, "random");
            assert_eq!(*arg, CoreExpr::Lit(CoreLiteral::Unit));
        }
        other => panic!("expected Perform, got {other:?}"),
    }
}

#[test]
fn elaborates_local_and_rec_groups() {
    let local = elaborate_source(
        r#"
(val main
  (local
    (val x 1)
    (val y (+ x 1))
    y))
"#,
    )
    .unwrap();
    let CoreExpr::Let { value, .. } = local else {
        panic!("expected Let");
    };
    let CoreExpr::Let {
        name: x, body: mid, ..
    } = *value
    else {
        panic!("expected local→Let x");
    };
    assert_eq!(x, "x");
    let CoreExpr::Let { name: y, body, .. } = *mid else {
        panic!("expected local→Let y");
    };
    assert_eq!(y, "y");
    assert!(matches!(*body, CoreExpr::Var(ref n) if n == "y"));

    let rec = elaborate_source(
        r#"
(val main
  (rec
    (val even?
      (fn (n)
        (if (= n 0) true (odd? (- n 1)))))
    (val odd?
      (fn (n)
        (if (= n 0) false (even? (- n 1)))))
    (even? 4)))
"#,
    )
    .unwrap();
    let CoreExpr::Let { value, .. } = rec else {
        panic!("expected Let");
    };
    let CoreExpr::LetRec { bindings, body } = *value else {
        panic!("expected LetRec");
    };
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].0, "even?");
    assert_eq!(bindings[1].0, "odd?");
    assert!(matches!(*body, CoreExpr::App { .. }));
}

#[test]
fn elaborates_toplevel_rec_and_local_var() {
    let expr = elaborate_source(
        r#"
(rec
  (val even?
    (fn (n)
      (if (= n 0) true (odd? (- n 1)))))
  (val odd?
    (fn (n)
      (if (= n 0) false (even? (- n 1))))))
(val main (even? 4))
"#,
    )
    .unwrap();
    let CoreExpr::LetRec { bindings, body } = expr else {
        panic!("expected top-level LetRec");
    };
    assert_eq!(bindings.len(), 2);
    let CoreExpr::Let { name, .. } = *body else {
        panic!("expected main after rec");
    };
    assert_eq!(name, "main");

    let local_var = elaborate_source(
        r#"
(val main
  (local
    (var count 0)
    (seq
      (set count 5)
      count)))
"#,
    )
    .unwrap();
    let CoreExpr::Let { value, .. } = local_var else {
        panic!("expected Let");
    };
    assert!(matches!(*value, CoreExpr::LocalVar { ref name, .. } if name == "count"));
}

#[test]
fn elaborates_unit_literal() {
    let expr = elaborate_source("(val main unit)").unwrap();
    let CoreExpr::Let { value, .. } = expr else {
        panic!("expected Let");
    };
    assert_eq!(*value, CoreExpr::Lit(CoreLiteral::Unit));
}

#[test]
fn rejects_empty_parens_as_unit() {
    let err = elaborate_source("(val main ())").unwrap_err();
    assert!(
        err.message.contains("empty list"),
        "() must not be unit; got: {}",
        err.message
    );
}

#[test]
fn rejects_reserved_val_binder() {
    let err = elaborate_source("(val if 1)").unwrap_err();
    assert!(
        err.message.contains("reserved special-form"),
        "got: {}",
        err.message
    );
}

#[test]
fn elaborates_record_field_list_tuple() {
    let expr = elaborate_source(
        r#"
(val report (record (title "Report") (page-count 10)))
(val main (field report title))
"#,
    )
    .unwrap();
    let CoreExpr::Let { value, body, .. } = expr else {
        panic!("expected outer Let");
    };
    assert!(matches!(
        *value,
        CoreExpr::Record { ref fields } if fields.len() == 2
            && fields[0].0 == "title"
            && fields[1].0 == "page-count"
    ));
    let CoreExpr::Let { value: main_v, .. } = *body else {
        panic!("expected main Let");
    };
    assert!(matches!(
        *main_v,
        CoreExpr::RecordGet { ref field, .. } if field == "title"
    ));

    let list = elaborate_source("(val main (list 1 2))").unwrap();
    let CoreExpr::Let { value, .. } = list else {
        panic!("expected Let");
    };
    assert!(matches!(
        *value,
        CoreExpr::Variant { ref tag, payload: Some(_) } if tag == "cons"
    ));

    let tup = elaborate_source(r#"(val main (tuple 1 "title" true))"#).unwrap();
    let CoreExpr::Let { value, .. } = tup else {
        panic!("expected Let");
    };
    assert!(matches!(
        *value,
        CoreExpr::Record { ref fields } if fields.len() == 3
            && fields[0].0 == "0"
            && fields[1].0 == "1"
            && fields[2].0 == "2"
    ));

    let empty_tup = elaborate_source("(val main (tuple))").unwrap();
    let CoreExpr::Let { value, .. } = empty_tup else {
        panic!("expected Let");
    };
    assert_eq!(*value, CoreExpr::Lit(CoreLiteral::Unit));

    let one_tup = elaborate_source("(val main (tuple 42))");
    assert!(
        one_tup.is_err(),
        "1-element tuple must be rejected (SYN §20)"
    );
    let err = one_tup.expect_err("checked");
    assert!(
        err.message.contains("1-element"),
        "unexpected error: {}",
        err.message
    );
}

#[test]
fn elaborates_record_update_and_extend() {
    let expr = elaborate_source(
        r#"
(val report (record (title "Old") (page-count 1)))
(val main (record-update report (title "New")))
"#,
    )
    .unwrap();
    let CoreExpr::Let { body, .. } = expr else {
        panic!("expected Let");
    };
    let CoreExpr::Let { value, .. } = *body else {
        panic!("expected main Let");
    };
    assert!(matches!(*value, CoreExpr::RecordUpdate { .. }));

    let ext = elaborate_source(
        r#"
(val report (record (title "T")))
(val main (record-extend report (author "A")))
"#,
    )
    .unwrap();
    let CoreExpr::Let { body, .. } = ext else {
        panic!("expected Let");
    };
    let CoreExpr::Let { value, .. } = *body else {
        panic!("expected main Let");
    };
    assert!(matches!(*value, CoreExpr::RecordExtend { .. }));
}

#[test]
fn elaborates_radix_and_scientific_numbers() {
    let hex = elaborate_source("(val main 0x2a)").unwrap();
    let CoreExpr::Let { value, .. } = hex else {
        panic!("expected Let");
    };
    assert_eq!(*value, CoreExpr::Lit(CoreLiteral::Int(42)));

    let sci = elaborate_source("(val main 1e2)").unwrap();
    let CoreExpr::Let { value, .. } = sci else {
        panic!("expected Let");
    };
    assert_eq!(*value, CoreExpr::Lit(CoreLiteral::F64(100.0)));

    let sep = elaborate_source("(val main 1_000)").unwrap();
    let CoreExpr::Let { value, .. } = sep else {
        panic!("expected Let");
    };
    assert_eq!(*value, CoreExpr::Lit(CoreLiteral::Int(1000)));
}

#[test]
fn intersect_not_diff_type_syntax_parses() {
    use reciplexa_core::unify::{unify, Subst};
    let (_, data) = elaborate_with_data(
        r#"
(type-alias narrow (intersect number string))
(type-alias removed (diff (union number string) number))
(val main unit)
"#,
    )
    .unwrap();
    assert!(matches!(
        data.type_aliases.get("narrow"),
        Some(CoreType::Intersect(_))
    ));
    assert!(matches!(
        data.type_aliases.get("removed"),
        Some(CoreType::Diff(_, _))
    ));
    let mut s = Subst::new();
    assert!(unify(
        data.type_aliases.get("narrow").unwrap(),
        &CoreType::dyn_any(),
        &mut s
    )
    .is_ok());
}

#[test]
fn registers_surface_type_aliases_and_dynamic() {
    let (expr, data) = elaborate_with_data(
        r#"
(type-alias title str)
(type-alias maybe (union int str))
(type-alias any-val (dynamic any))
(val main "Hello")
"#,
    )
    .unwrap();
    assert!(matches!(
        data.type_aliases.get("title"),
        Some(CoreType::String)
    ));
    assert!(matches!(
        data.type_aliases.get("maybe"),
        Some(CoreType::Union(m)) if m.len() == 2
    ));
    assert!(matches!(
        data.type_aliases.get("any-val"),
        Some(CoreType::Dynamic(b)) if matches!(b.as_ref(), CoreType::Any)
    ));
    assert!(data.value_annotations.is_empty());
    let CoreExpr::Let { value, .. } = expr else {
        panic!("expected Let");
    };
    assert_eq!(*value, CoreExpr::Lit(CoreLiteral::String("Hello".into())));
}

#[test]
fn parses_surface_fn_app_forall_row_and_effects_types() {
    let (_, data) = elaborate_with_data(
        r#"
(data option ((a type)) none (some a))
(type-alias id-fn (fn int int))
(type-alias opt-str (option str))
(type-alias poly (forall ((a type)) (fn a a)))
(type-alias open-rec (record (title str) (row r)))
(type-alias eff-fn (fn str unit (effects console resource)))
(type-alias tup (tuple int str))
(val main unit)
"#,
    )
    .unwrap();
    assert!(matches!(
        data.type_aliases.get("id-fn"),
        Some(CoreType::Fun { args, ret, effects })
            if args.len() == 1
                && matches!(args[0], CoreType::Int)
                && matches!(ret.as_ref(), CoreType::Int)
                && effects.ops.is_empty()
    ));
    assert!(matches!(
        data.type_aliases.get("opt-str"),
        Some(CoreType::App { ctor, args }) if ctor == "option"
            && args.len() == 1
            && matches!(args[0], CoreType::String)
    ));
    assert!(matches!(
        data.type_aliases.get("poly"),
        Some(CoreType::Forall { params, body })
            if params == &[("a".into(), "type".into())]
                && matches!(
                    body.as_ref(),
                    CoreType::Fun { args, ret, .. }
                        if args.len() == 1
                            && matches!(&args[0], CoreType::Name(n) if n == "a")
                            && matches!(ret.as_ref(), CoreType::Name(n) if n == "a")
                )
    ));
    assert!(matches!(
        data.type_aliases.get("open-rec"),
        Some(CoreType::OpenRecord { fields, row })
            if fields.len() == 1
                && fields[0].0 == "title"
                && matches!(row.as_ref(), CoreType::Name(n) if n == "r")
    ));
    assert!(matches!(
        data.type_aliases.get("eff-fn"),
        Some(CoreType::Fun { effects, .. })
            if effects.ops == ["console".to_string(), "resource".to_string()]
    ));
    assert!(matches!(
        data.type_aliases.get("tup"),
        Some(CoreType::Record { fields }) if fields.len() == 2
            && fields[0].0 == "0"
            && fields[1].0 == "1"
    ));

    let one = elaborate_with_data("(type-alias bad (tuple int))\n(val main unit)").unwrap_err();
    assert!(one.message.contains("1-element"), "got: {}", one.message);
}

#[test]
fn elaborates_bytes_literal() {
    let expr = elaborate_source("(val main (bytes 0x00 255 42))").unwrap();
    let CoreExpr::Let { value, .. } = expr else {
        panic!("expected Let");
    };
    assert_eq!(*value, CoreExpr::Lit(CoreLiteral::Bytes(vec![0, 255, 42])));

    let bad = elaborate_source("(val main (bytes 256))").unwrap_err();
    assert!(
        bad.message.contains("0..255") || bad.message.contains("out of range"),
        "got: {}",
        bad.message
    );
}

#[test]
fn rejects_duplicate_top_level_binding() {
    let err = elaborate_source("(val x 1)\n(val x 2)").unwrap_err();
    assert!(
        err.message.contains("duplicate top-level binding"),
        "got: {}",
        err.message
    );
}

#[test]
fn rejects_top_level_var() {
    let err = elaborate_source("(var x 0 x)").unwrap_err();
    assert!(
        err.message.contains("top-level `var`"),
        "got: {}",
        err.message
    );
}
