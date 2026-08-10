//! Reference evaluation of Core expressions.

use std::collections::HashMap;

use reciplexa_core::expr::{CoreExpr, CoreLiteral, MatchArm};

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
        CoreExpr::Var(name) => env.get(name).cloned().ok_or_else(|| EvalError {
            message: format!("unbound variable `{name}`"),
        }),
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
        CoreExpr::Lambda { param, body } => Ok(RuntimeValue::Closure {
            param: param.clone(),
            body: *body.clone(),
            env: env.clone(),
        }),
        CoreExpr::App { fun, arg } => {
            let fun_v = eval_expr(fun, env, host)?;
            let arg_v = eval_expr(arg, env, host)?;
            match fun_v {
                RuntimeValue::Closure {
                    param,
                    body,
                    env: closure_env,
                } => {
                    let mut child = closure_env;
                    child.insert(param, arg_v);
                    eval_expr(&body, &child, host)
                }
                other => Err(EvalError {
                    message: format!("expected closure, got {other:?}"),
                }),
            }
        }
        CoreExpr::Record { fields } => {
            let mut out = Vec::new();
            for (k, v) in fields {
                out.push((k.clone(), eval_expr(v, env, host)?));
            }
            Ok(RuntimeValue::Record(out))
        }
        CoreExpr::RecordGet { record, field } => {
            let v = eval_expr(record, env, host)?;
            match v {
                RuntimeValue::Record(fields) => fields
                    .into_iter()
                    .find(|(k, _)| k == field)
                    .map(|(_, v)| v)
                    .ok_or_else(|| EvalError {
                        message: format!("unknown field `{field}`"),
                    }),
                other => Err(EvalError {
                    message: format!("expected record, got {other:?}"),
                }),
            }
        }
        CoreExpr::Variant { tag, payload } => {
            let p = if let Some(e) = payload {
                Some(Box::new(eval_expr(e, env, host)?))
            } else {
                None
            };
            Ok(RuntimeValue::Variant {
                tag: tag.clone(),
                payload: p,
            })
        }
        CoreExpr::Match { scrutinee, arms } => {
            let v = eval_expr(scrutinee, env, host)?;
            eval_match(&v, arms, env, host)
        }
    }
}

fn eval_match<H: EffectHost>(
    value: &RuntimeValue,
    arms: &[MatchArm],
    env: &HashMap<String, RuntimeValue>,
    host: &mut H,
) -> EvalResult {
    let RuntimeValue::Variant { tag, payload } = value else {
        return Err(EvalError {
            message: "match scrutinee must be variant".into(),
        });
    };
    for arm in arms {
        if &arm.tag == tag {
            let mut child = env.clone();
            if let Some(bind) = &arm.bind {
                if let Some(p) = payload {
                    child.insert(bind.clone(), (**p).clone());
                }
            }
            return eval_expr(&arm.body, &child, host);
        }
    }
    Err(EvalError {
        message: format!("no match arm for tag `{tag}`"),
    })
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
