//! Reference evaluation of Core expressions.

use std::collections::HashMap;

use reciplexa_core::elaborate::{elaborate_source, ElaborateError};
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

/// Expand language macros, elaborate surface source to Core, then evaluate with [`UnitHost`].
pub fn eval_source(src: &str) -> EvalResult {
    let expanded =
        reciplexa_macro::expand_language(src).map_err(|e| EvalError { message: e.message })?;
    let expr = elaborate_source(&expanded)
        .map_err(|e: ElaborateError| EvalError { message: e.message })?;
    eval_expr(&expr, &HashMap::new(), &mut UnitHost)
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
        CoreExpr::Lambda { params, body } => Ok(RuntimeValue::Closure {
            params: params.clone(),
            body: *body.clone(),
            env: env.clone(),
        }),
        CoreExpr::App { fun, args } => {
            let fun_v = eval_expr(fun, env, host)?;
            let mut arg_vs = Vec::with_capacity(args.len());
            for arg in args {
                arg_vs.push(eval_expr(arg, env, host)?);
            }
            match fun_v {
                RuntimeValue::Closure {
                    params,
                    body,
                    env: closure_env,
                } => {
                    if params.len() != arg_vs.len() {
                        return Err(EvalError {
                            message: format!(
                                "arity mismatch: expected {} args, got {}",
                                params.len(),
                                arg_vs.len()
                            ),
                        });
                    }
                    let mut child = closure_env;
                    for (param, arg_v) in params.into_iter().zip(arg_vs) {
                        child.insert(param, arg_v);
                    }
                    eval_expr(&body, &child, host)
                }
                other => Err(EvalError {
                    message: format!("expected closure, got {other:?}"),
                }),
            }
        }
        CoreExpr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let cond_v = eval_expr(cond, env, host)?;
            match cond_v {
                RuntimeValue::Bool(true) => eval_expr(then_branch, env, host),
                RuntimeValue::Bool(false) => eval_expr(else_branch, env, host),
                other => Err(EvalError {
                    message: format!("if condition must be Bool, got {other:?}"),
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
        CoreLiteral::Bool(b) => RuntimeValue::Bool(*b),
    })
}
