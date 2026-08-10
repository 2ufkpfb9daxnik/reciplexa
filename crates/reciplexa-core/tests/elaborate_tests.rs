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
        name: x,
        body: mid,
        ..
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
