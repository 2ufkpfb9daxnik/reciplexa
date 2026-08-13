//! N6 tip q: Cont Record rest-field Perform after Value resume (not Forward).

use std::collections::HashMap;

use reciplexa_core::expr::{CoreExpr, CoreLiteral};
use reciplexa_eval::control::UnitHost;
use reciplexa_eval::eval::eval_expr;

fn handle(body: CoreExpr) -> CoreExpr {
    CoreExpr::Handle {
        op: "ask".into(),
        handler_params: vec!["m".into(), "k".into()],
        handler_body: Box::new(CoreExpr::App {
            fun: Box::new(CoreExpr::Var("k".into())),
            args: vec![CoreExpr::Var("m".into())],
        }),
        body: Box::new(body),
    }
}

fn perform_ask(n: i128) -> CoreExpr {
    CoreExpr::Perform {
        op: "ask".into(),
        arg: Box::new(CoreExpr::Lit(CoreLiteral::Int(n))),
    }
}

#[test]
fn eval_round28_record_rest_field_perform_after_value_resume() {
    // Cont for field `a` resumes with Value (identity Perform resume), then
    // field `b` Performs → hits `other` in the rest-field loop.
    let env = HashMap::new();
    let _ = eval_expr(
        &handle(CoreExpr::Record {
            fields: vec![
                ("a".into(), perform_ask(1)),
                ("b".into(), perform_ask(2)),
            ],
        }),
        &env,
        &mut UnitHost,
    );

    // Seq: same Value-resume then rest Perform pattern.
    let _ = eval_expr(
        &handle(CoreExpr::Seq(vec![
            perform_ask(1),
            perform_ask(2),
            CoreExpr::Lit(CoreLiteral::Int(3)),
        ])),
        &env,
        &mut UnitHost,
    );
}
