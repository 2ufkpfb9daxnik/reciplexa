//! Reference evaluation of Core expressions.

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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

/// Internal control for shallow handlers / one-shot resume (EFF-001 v0).
enum Outcome {
    Value(RuntimeValue),
    /// Uncaught perform — bubbles to the nearest matching [`CoreExpr::Handle`].
    Performed {
        op: String,
        arg: RuntimeValue,
    },
    /// One-shot resume fired inside a handler; becomes the handle result.
    Resumed(RuntimeValue),
}

pub fn eval_expr<H: EffectHost>(
    expr: &CoreExpr,
    env: &HashMap<String, RuntimeValue>,
    host: &mut H,
) -> EvalResult {
    match eval_outcome(expr, env, host)? {
        Outcome::Value(v) => Ok(v),
        Outcome::Performed { op, arg } => host.perform(&op, arg),
        Outcome::Resumed(_) => Err(EvalError {
            message: "resume outside handle".into(),
        }),
    }
}

fn eval_outcome<H: EffectHost>(
    expr: &CoreExpr,
    env: &HashMap<String, RuntimeValue>,
    host: &mut H,
) -> Result<Outcome, EvalError> {
    match expr {
        CoreExpr::Lit(lit) => Ok(Outcome::Value(eval_lit(lit)?)),
        CoreExpr::Var(name) => env
            .get(name)
            .cloned()
            .map(Outcome::Value)
            .ok_or_else(|| EvalError {
                message: format!("unbound variable `{name}`"),
            }),
        CoreExpr::Perform { op, arg } => {
            let v = match eval_outcome(arg, env, host)? {
                Outcome::Value(v) => v,
                other => return Ok(other),
            };
            Ok(Outcome::Performed {
                op: op.clone(),
                arg: v,
            })
        }
        CoreExpr::Handle {
            op,
            handler_params,
            handler_body,
            body,
        } => match eval_outcome(body, env, host)? {
            Outcome::Value(v) => Ok(Outcome::Value(v)),
            Outcome::Resumed(v) => Ok(Outcome::Resumed(v)),
            Outcome::Performed {
                op: performed_op,
                arg,
            } if performed_op == *op => run_handler(handler_params, handler_body, arg, env, host),
            Outcome::Performed { op, arg } => Ok(Outcome::Performed { op, arg }),
        },
        CoreExpr::Seq(items) => {
            let mut last = RuntimeValue::Unit;
            for item in items {
                match eval_outcome(item, env, host)? {
                    Outcome::Value(v) => last = v,
                    other => return Ok(other),
                }
            }
            Ok(Outcome::Value(last))
        }
        CoreExpr::Let { name, value, body } => {
            let v = match eval_outcome(value, env, host)? {
                Outcome::Value(v) => v,
                other => return Ok(other),
            };
            let mut child = env.clone();
            child.insert(name.clone(), v);
            eval_outcome(body, &child, host)
        }
        CoreExpr::LetRec { bindings, body } => {
            let shared = Rc::new(RefCell::new(env.clone()));
            for (name, _) in bindings {
                shared
                    .borrow_mut()
                    .insert(name.clone(), RuntimeValue::Unit);
            }
            for (name, rhs) in bindings {
                let CoreExpr::Lambda {
                    params,
                    body: lam_body,
                } = rhs
                else {
                    return Err(EvalError {
                        message: "letrec binding must be a lambda".into(),
                    });
                };
                let clo = RuntimeValue::Closure {
                    params: params.clone(),
                    body: *lam_body.clone(),
                    env: Rc::clone(&shared),
                };
                shared.borrow_mut().insert(name.clone(), clo);
            }
            let child = shared.borrow().clone();
            eval_outcome(body, &child, host)
        }
        CoreExpr::Lambda { params, body } => Ok(Outcome::Value(RuntimeValue::Closure {
            params: params.clone(),
            body: *body.clone(),
            env: Rc::new(RefCell::new(env.clone())),
        })),
        CoreExpr::App { fun, args } => {
            let fun_v = match eval_outcome(fun, env, host)? {
                Outcome::Value(v) => v,
                other => return Ok(other),
            };
            let mut arg_vs = Vec::with_capacity(args.len());
            for arg in args {
                match eval_outcome(arg, env, host)? {
                    Outcome::Value(v) => arg_vs.push(v),
                    other => return Ok(other),
                }
            }
            apply_value(fun_v, arg_vs, host)
        }
        CoreExpr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let cond_v = match eval_outcome(cond, env, host)? {
                Outcome::Value(v) => v,
                other => return Ok(other),
            };
            match cond_v {
                RuntimeValue::Bool(true) => eval_outcome(then_branch, env, host),
                RuntimeValue::Bool(false) => eval_outcome(else_branch, env, host),
                other => Err(EvalError {
                    message: format!("if condition must be Bool, got {other:?}"),
                }),
            }
        }
        CoreExpr::Record { fields } => {
            let mut out = Vec::new();
            for (k, v) in fields {
                match eval_outcome(v, env, host)? {
                    Outcome::Value(val) => out.push((k.clone(), val)),
                    other => return Ok(other),
                }
            }
            Ok(Outcome::Value(RuntimeValue::Record(out)))
        }
        CoreExpr::RecordGet { record, field } => {
            let v = match eval_outcome(record, env, host)? {
                Outcome::Value(v) => v,
                other => return Ok(other),
            };
            match v {
                RuntimeValue::Record(fields) => fields
                    .into_iter()
                    .find(|(k, _)| k == field)
                    .map(|(_, v)| Outcome::Value(v))
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
                match eval_outcome(e, env, host)? {
                    Outcome::Value(v) => Some(Box::new(v)),
                    other => return Ok(other),
                }
            } else {
                None
            };
            Ok(Outcome::Value(RuntimeValue::Variant {
                tag: tag.clone(),
                payload: p,
            }))
        }
        CoreExpr::Match { scrutinee, arms } => {
            let v = match eval_outcome(scrutinee, env, host)? {
                Outcome::Value(v) => v,
                other => return Ok(other),
            };
            eval_match(&v, arms, env, host)
        }
    }
}

fn run_handler<H: EffectHost>(
    handler_params: &[String],
    handler_body: &CoreExpr,
    arg: RuntimeValue,
    env: &HashMap<String, RuntimeValue>,
    host: &mut H,
) -> Result<Outcome, EvalError> {
    let mut child = env.clone();
    match handler_params {
        [p] => {
            child.insert(p.clone(), arg);
        }
        [p, resume] => {
            child.insert(p.clone(), arg);
            child.insert(
                resume.clone(),
                RuntimeValue::OneShotResume {
                    used: Rc::new(Cell::new(false)),
                },
            );
        }
        _ => {
            return Err(EvalError {
                message: "handle handler expects 1 or 2 parameters".into(),
            });
        }
    }
    match eval_outcome(handler_body, &child, host)? {
        Outcome::Value(v) => Ok(Outcome::Value(v)),
        Outcome::Resumed(v) => Ok(Outcome::Value(v)),
        Outcome::Performed { op, arg } => Ok(Outcome::Performed { op, arg }),
    }
}

fn apply_value<H: EffectHost>(
    fun_v: RuntimeValue,
    arg_vs: Vec<RuntimeValue>,
    host: &mut H,
) -> Result<Outcome, EvalError> {
    match fun_v {
        RuntimeValue::OneShotResume { used } => {
            if arg_vs.len() != 1 {
                return Err(EvalError {
                    message: format!("resume expects 1 arg, got {}", arg_vs.len()),
                });
            }
            if used.replace(true) {
                return Err(EvalError {
                    message: "one-shot resume already used".into(),
                });
            }
            Ok(Outcome::Resumed(arg_vs.into_iter().next().expect("len 1")))
        }
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
            let mut child = closure_env.borrow().clone();
            for (param, arg_v) in params.into_iter().zip(arg_vs) {
                child.insert(param, arg_v);
            }
            eval_outcome(&body, &child, host)
        }
        other => Err(EvalError {
            message: format!("expected closure, got {other:?}"),
        }),
    }
}

fn eval_match<H: EffectHost>(
    value: &RuntimeValue,
    arms: &[MatchArm],
    env: &HashMap<String, RuntimeValue>,
    host: &mut H,
) -> Result<Outcome, EvalError> {
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
            return eval_outcome(&arm.body, &child, host);
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
