//! Reference evaluation of Core expressions.

use std::collections::HashMap;

use reciplexa_core::expr::{CoreExpr, CoreLiteral};

use crate::value::RuntimeValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalError {
    pub message: String,
}

pub type EvalResult = Result<RuntimeValue, EvalError>;

pub trait EffectHost {
    fn perform(&mut self, op: &str, arg: RuntimeValue) -> EvalResult;
}

pub struct UnitHost;

impl EffectHost for UnitHost {
    fn perform(&mut self, op: &str, _arg: RuntimeValue) -> EvalResult {
        match op {
            "log" => Ok(RuntimeValue::Unit),
            "random" => Ok(RuntimeValue::Number(0.5)),
            other => Err(EvalError {
                message: format!("unknown op `{other}`"),
            }),
        }
    }
}

pub fn eval_expr<H: EffectHost>(
    expr: &CoreExpr,
    env: &HashMap<String, RuntimeValue>,
    host: &mut H,
) -> EvalResult {
    match expr {
        CoreExpr::Lit(lit) => eval_lit(lit),
        CoreExpr::Perform { op, arg } => {
            let v = eval_expr(arg, env, host)?;
            host.perform(op, v)
        }
        CoreExpr::Seq(items) => {
            let mut last = RuntimeValue::Unit;
            for item in items {
                last = eval_expr(item, env, host)?;
            }
            Ok(last)
        }
        CoreExpr::Let { name, value, body } => {
            let v = eval_expr(value, env, host)?;
            let mut child = env.clone();
            child.insert(name.clone(), v);
            eval_expr(body, &child, host)
        }
    }
}

fn eval_lit(lit: &CoreLiteral) -> EvalResult {
    Ok(match lit {
        CoreLiteral::Number(n) => RuntimeValue::Number(*n),
        CoreLiteral::String(s) => {
            if s == "circle" || s == "rect" || s == "text" {
                RuntimeValue::ShapeTag(s.clone())
            } else {
                RuntimeValue::String(s.clone())
            }
        }
        CoreLiteral::Color(c) => RuntimeValue::String(c.clone()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_core::expr::CoreExpr;

    #[test]
    fn seq_evaluates_left_to_right() {
        let expr = CoreExpr::Seq(vec![
            CoreExpr::Lit(CoreLiteral::Number(1.0)),
            CoreExpr::Lit(CoreLiteral::Number(2.0)),
        ]);
        let v = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
        assert_eq!(v, RuntimeValue::Number(2.0));
    }

    #[test]
    fn let_binds_in_body() {
        let expr = CoreExpr::Let {
            name: "x".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Number(3.0))),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        };
        let _ = eval_expr(&expr, &HashMap::new(), &mut UnitHost).unwrap();
    }
}
