//! Surface → Core elaborator tests (BND-001).

use reciplexa_core::check::{infer_expr, TypeEnv};
use reciplexa_core::elaborate::elaborate_source;
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
                    && args == &[CoreExpr::Lit(CoreLiteral::Number(42.0))]
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
            then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
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
    assert_eq!(subst.apply(&ty), CoreType::Number);
}

#[test]
fn typechecks_elaborated_if() {
    let expr = elaborate_source("(val main (if true 1 2))").unwrap();
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
    assert_eq!(subst.apply(&ty), CoreType::Number);
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
(data Option (None) (Some x))
(val main (match (Some 1) (None -> 0) (Some x -> x)))
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
        } if tag == "Some"
    ));
    assert_eq!(arms.len(), 2);
    assert_eq!(arms[0].tag(), Some("None"));
    assert!(matches!(arms[0].pattern, reciplexa_core::CorePattern::Variant { payload: None, .. }));
    assert_eq!(arms[1].tag(), Some("Some"));
    assert!(matches!(
        &arms[1].pattern,
        reciplexa_core::CorePattern::Variant {
            payload: Some(p),
            ..
        } if matches!(p.as_ref(), reciplexa_core::CorePattern::Bind(n) if n == "x")
    ));
}

#[test]
fn elaborates_wildcard_bind_and_nested_patterns() {
    let expr = elaborate_source(
        r#"
(data Option (None) (Some x))
(val main
  (match (Some (Some 1))
    (Some (Some item) -> item)
    (Some _ -> 0)
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
        } if tag == "Some"
            && matches!(
                inner.as_ref(),
                reciplexa_core::CorePattern::Variant {
                    tag: t2,
                    payload: Some(b),
                } if t2 == "Some"
                    && matches!(b.as_ref(), reciplexa_core::CorePattern::Bind(n) if n == "item")
            )
    ));
    assert!(matches!(
        &arms[1].pattern,
        reciplexa_core::CorePattern::Variant {
            tag,
            payload: Some(inner),
        } if tag == "Some" && matches!(inner.as_ref(), reciplexa_core::CorePattern::Wildcard)
    ));
    assert!(matches!(arms[2].pattern, reciplexa_core::CorePattern::Wildcard));

    let bind = elaborate_source(
        r#"
(data Option (None) (Some x))
(val main (match (Some 1) (bind v -> v)))
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
fn rejects_old_match_arm_without_arrow() {
    let err = elaborate_source(
        r#"
(data Option (None) (Some x))
(val main (match (Some 1) (None 0) ((Some x) x)))
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
        CoreExpr::Variant { ref tag, payload: Some(_) } if tag == "Cons"
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

    let one_tup = elaborate_source("(val main (tuple 42))").unwrap();
    let CoreExpr::Let { value, .. } = one_tup else {
        panic!("expected Let");
    };
    assert_eq!(*value, CoreExpr::Lit(CoreLiteral::Number(42.0)));
}
