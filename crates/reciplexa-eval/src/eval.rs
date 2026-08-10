//! Reference evaluation of Core expressions.

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::elaborate::{elaborate_source, ElaborateError};
use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};

use crate::control::{
    identity_resume, EffectHost, EvalError, EvalResult, Outcome, ResumeCont, UnitHost,
};
use crate::value::{BuiltinOp, RuntimeValue};

/// Seed environment with KER-001 numeric / comparison primitives.
pub fn primitive_env() -> HashMap<String, RuntimeValue> {
    let mut env = HashMap::new();
    env.insert("+".into(), RuntimeValue::Builtin(BuiltinOp::Add));
    env.insert("-".into(), RuntimeValue::Builtin(BuiltinOp::Sub));
    env.insert("*".into(), RuntimeValue::Builtin(BuiltinOp::Mul));
    env.insert("/".into(), RuntimeValue::Builtin(BuiltinOp::Div));
    env.insert("<".into(), RuntimeValue::Builtin(BuiltinOp::Lt));
    env.insert(">".into(), RuntimeValue::Builtin(BuiltinOp::Gt));
    env.insert("<=".into(), RuntimeValue::Builtin(BuiltinOp::Le));
    env.insert(">=".into(), RuntimeValue::Builtin(BuiltinOp::Ge));
    env.insert("=".into(), RuntimeValue::Builtin(BuiltinOp::Eq));
    env.insert("!=".into(), RuntimeValue::Builtin(BuiltinOp::Ne));
    env
}

/// Expand language macros, elaborate surface source to Core, then evaluate with [`UnitHost`].
pub fn eval_source(src: &str) -> EvalResult {
    let expanded =
        reciplexa_macro::expand_language(src).map_err(|e| EvalError { message: e.message })?;
    let expr = elaborate_source(&expanded)
        .map_err(|e: ElaborateError| EvalError { message: e.message })?;
    eval_expr(&expr, &primitive_env(), &mut UnitHost)
}

pub fn eval_expr<H: EffectHost>(
    expr: &CoreExpr,
    env: &HashMap<String, RuntimeValue>,
    host: &mut H,
) -> EvalResult {
    let mut outcome = eval_outcome(expr, env, host)?;
    loop {
        match outcome {
            Outcome::Value(v) => return Ok(v),
            Outcome::Resumed(v) => return Ok(v),
            Outcome::Performed { op, arg, resume } => {
                let host_v = host.perform(&op, arg)?;
                outcome = resume(host_v, host)?;
            }
        }
    }
}

fn eval_outcome(
    expr: &CoreExpr,
    env: &HashMap<String, RuntimeValue>,
    host: &mut dyn EffectHost,
) -> Result<Outcome, EvalError> {
    match expr {
        CoreExpr::Lit(lit) => Ok(Outcome::Value(eval_lit(lit)?)),
        CoreExpr::Var(name) => match env.get(name) {
            Some(RuntimeValue::Cell { value, alive }) => {
                if !alive.get() {
                    return Err(EvalError {
                        message: format!("var `{name}` used after scope exit (escaped)"),
                    });
                }
                Ok(Outcome::Value(value.borrow().clone()))
            }
            Some(v) => Ok(Outcome::Value(v.clone())),
            None => Err(EvalError {
                message: format!("unbound variable `{name}`"),
            }),
        },
        CoreExpr::Perform { op, arg } => {
            let v = match eval_outcome(arg, env, host)? {
                Outcome::Value(v) => v,
                other => return Ok(other),
            };
            Ok(Outcome::Performed {
                op: op.clone(),
                arg: v,
                resume: identity_resume(),
            })
        }
        CoreExpr::Handle {
            op,
            handler_params,
            handler_body,
            body,
        } => eval_handle(op, handler_params, handler_body, body, env, host),
        CoreExpr::HandlerValue {
            op,
            handler_params,
            handler_body,
        } => Ok(Outcome::Value(RuntimeValue::Handler {
            op: op.clone(),
            params: handler_params.clone(),
            body: *handler_body.clone(),
            env: Rc::new(std::cell::RefCell::new(env.clone())),
        })),
        CoreExpr::With { handler, body } => {
            match eval_outcome(handler, env, host)? {
                Outcome::Value(RuntimeValue::Handler {
                    op,
                    params,
                    body: handler_body,
                    env: hen,
                }) => {
                    // Install using the handler's captured lexical env as base,
                    // overlaying the current env for free uses of ambient bindings.
                    let mut install_env = hen.borrow().clone();
                    for (k, v) in env {
                        install_env.insert(k.clone(), v.clone());
                    }
                    eval_handle(&op, &params, &handler_body, body, &install_env, host)
                }
                Outcome::Value(other) => Err(EvalError {
                    message: format!("`with` expects a handler value, got {other}"),
                }),
                Outcome::Performed { op, arg, resume } => {
                    Ok(Outcome::Performed { op, arg, resume })
                }
                Outcome::Resumed(v) => Ok(Outcome::Resumed(v)),
            }
        }
        CoreExpr::Seq(items) => eval_seq(items, env, host),
        CoreExpr::Let { name, value, body } => match eval_outcome(value, env, host)? {
            Outcome::Value(v) => {
                let mut child = env.clone();
                child.insert(name.clone(), v);
                eval_outcome(body, &child, host)
            }
            Outcome::Performed {
                op,
                arg,
                resume: inner,
            } => {
                let name = name.clone();
                let body = body.clone();
                let env = env.clone();
                Ok(Outcome::Performed {
                    op,
                    arg,
                    resume: Rc::new(move |v, host| {
                        let v = match inner(v, host)? {
                            Outcome::Value(v) => v,
                            other => return Ok(other),
                        };
                        let mut child = env.clone();
                        child.insert(name.clone(), v);
                        eval_outcome(&body, &child, host)
                    }),
                })
            }
            Outcome::Resumed(v) => Ok(Outcome::Resumed(v)),
        },
        CoreExpr::LetRec { bindings, body } => {
            let shared = Rc::new(RefCell::new(env.clone()));
            for (name, _) in bindings {
                shared.borrow_mut().insert(name.clone(), RuntimeValue::Unit);
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
        CoreExpr::LocalVar { name, init, body } => match eval_outcome(init, env, host)? {
            Outcome::Value(init_v) => run_local_var(name, init_v, body, env, host),
            Outcome::Performed {
                op,
                arg,
                resume: inner,
            } => {
                let name = name.clone();
                let body = body.clone();
                let env = env.clone();
                Ok(Outcome::Performed {
                    op,
                    arg,
                    resume: Rc::new(move |v, host| {
                        let init_v = match inner(v, host)? {
                            Outcome::Value(v) => v,
                            other => return Ok(other),
                        };
                        run_local_var(&name, init_v, &body, &env, host)
                    }),
                })
            }
            Outcome::Resumed(v) => Ok(Outcome::Resumed(v)),
        },
        CoreExpr::Set { name, value } => match eval_outcome(value, env, host)? {
            Outcome::Value(v) => apply_set(name, v, env),
            Outcome::Performed {
                op,
                arg,
                resume: inner,
            } => {
                let name = name.clone();
                let env = env.clone();
                Ok(Outcome::Performed {
                    op,
                    arg,
                    resume: Rc::new(move |v, host| {
                        let v = match inner(v, host)? {
                            Outcome::Value(v) => v,
                            other => return Ok(other),
                        };
                        apply_set(&name, v, &env)
                    }),
                })
            }
            Outcome::Resumed(v) => Ok(Outcome::Resumed(v)),
        },
        CoreExpr::Lambda { params, body } => Ok(Outcome::Value(RuntimeValue::Closure {
            params: params.clone(),
            body: *body.clone(),
            env: Rc::new(RefCell::new(env.clone())),
        })),
        CoreExpr::App { fun, args } => eval_app(fun, args, env, host),
        CoreExpr::If {
            cond,
            then_branch,
            else_branch,
        } => match eval_outcome(cond, env, host)? {
            Outcome::Value(cond_v) => match cond_v {
                RuntimeValue::Bool(true) => eval_outcome(then_branch, env, host),
                RuntimeValue::Bool(false) => eval_outcome(else_branch, env, host),
                other => Err(EvalError {
                    message: format!("if condition must be Bool, got {other:?}"),
                }),
            },
            Outcome::Performed {
                op,
                arg,
                resume: inner,
            } => {
                let then_branch = then_branch.clone();
                let else_branch = else_branch.clone();
                let env = env.clone();
                Ok(Outcome::Performed {
                    op,
                    arg,
                    resume: Rc::new(move |v, host| {
                        let cond_v = match inner(v, host)? {
                            Outcome::Value(v) => v,
                            other => return Ok(other),
                        };
                        match cond_v {
                            RuntimeValue::Bool(true) => eval_outcome(&then_branch, &env, host),
                            RuntimeValue::Bool(false) => eval_outcome(&else_branch, &env, host),
                            other => Err(EvalError {
                                message: format!("if condition must be Bool, got {other:?}"),
                            }),
                        }
                    }),
                })
            }
            Outcome::Resumed(v) => Ok(Outcome::Resumed(v)),
        },
        CoreExpr::Record { fields } => eval_record(fields, env, host),
        CoreExpr::RecordGet { record, field } => match eval_outcome(record, env, host)? {
            Outcome::Value(v) => record_get(v, field),
            Outcome::Performed {
                op,
                arg,
                resume: inner,
            } => {
                let field = field.clone();
                Ok(Outcome::Performed {
                    op,
                    arg,
                    resume: Rc::new(move |v, host| {
                        let v = match inner(v, host)? {
                            Outcome::Value(v) => v,
                            other => return Ok(other),
                        };
                        record_get(v, &field)
                    }),
                })
            }
            Outcome::Resumed(v) => Ok(Outcome::Resumed(v)),
        },
        CoreExpr::Variant { tag, payload } => {
            if let Some(e) = payload {
                match eval_outcome(e, env, host)? {
                    Outcome::Value(v) => Ok(Outcome::Value(RuntimeValue::Variant {
                        tag: tag.clone(),
                        payload: Some(Box::new(v)),
                    })),
                    Outcome::Performed {
                        op,
                        arg,
                        resume: inner,
                    } => {
                        let tag = tag.clone();
                        Ok(Outcome::Performed {
                            op,
                            arg,
                            resume: Rc::new(move |v, host| {
                                let v = match inner(v, host)? {
                                    Outcome::Value(v) => v,
                                    other => return Ok(other),
                                };
                                Ok(Outcome::Value(RuntimeValue::Variant {
                                    tag: tag.clone(),
                                    payload: Some(Box::new(v)),
                                }))
                            }),
                        })
                    }
                    Outcome::Resumed(v) => Ok(Outcome::Resumed(v)),
                }
            } else {
                Ok(Outcome::Value(RuntimeValue::Variant {
                    tag: tag.clone(),
                    payload: None,
                }))
            }
        }
        CoreExpr::Match { scrutinee, arms } => match eval_outcome(scrutinee, env, host)? {
            Outcome::Value(v) => eval_match(&v, arms, env, host),
            Outcome::Performed {
                op,
                arg,
                resume: inner,
            } => {
                let arms = arms.clone();
                let env = env.clone();
                Ok(Outcome::Performed {
                    op,
                    arg,
                    resume: Rc::new(move |v, host| {
                        let v = match inner(v, host)? {
                            Outcome::Value(v) => v,
                            other => return Ok(other),
                        };
                        eval_match(&v, &arms, &env, host)
                    }),
                })
            }
            Outcome::Resumed(v) => Ok(Outcome::Resumed(v)),
        },
    }
}

fn run_local_var(
    name: &str,
    init_v: RuntimeValue,
    body: &CoreExpr,
    env: &HashMap<String, RuntimeValue>,
    host: &mut dyn EffectHost,
) -> Result<Outcome, EvalError> {
    let alive = Rc::new(Cell::new(true));
    let cell = RuntimeValue::Cell {
        value: Rc::new(RefCell::new(init_v)),
        alive: Rc::clone(&alive),
    };
    let mut child = env.clone();
    child.insert(name.to_string(), cell);
    let result = eval_outcome(body, &child, host);
    if matches!(result, Ok(Outcome::Value(_)) | Err(_)) {
        alive.set(false);
    }
    result
}

fn eval_handle(
    op: &str,
    handler_params: &[String],
    handler_body: &CoreExpr,
    body: &CoreExpr,
    env: &HashMap<String, RuntimeValue>,
    host: &mut dyn EffectHost,
) -> Result<Outcome, EvalError> {
    match eval_outcome(body, env, host)? {
        Outcome::Value(v) => Ok(Outcome::Value(v)),
        Outcome::Resumed(v) => Ok(Outcome::Resumed(v)),
        Outcome::Performed {
            op: performed_op,
            arg,
            resume,
        } if performed_op == op => {
            let op = op.to_string();
            let handler_params = handler_params.to_vec();
            let handler_body = Rc::new(handler_body.clone());
            let env = env.clone();
            let deep_resume: ResumeCont = Rc::new({
                let handler_params = handler_params.clone();
                let handler_body = Rc::clone(&handler_body);
                let env = env.clone();
                let op = op.clone();
                move |v, host| match resume(v, host)? {
                    Outcome::Value(v) => Ok(Outcome::Value(v)),
                    Outcome::Performed {
                        op: p,
                        arg,
                        resume: r,
                    } if p == op => {
                        run_handler_with_resume(&handler_params, &handler_body, arg, r, &env, host)
                    }
                    other => Ok(other),
                }
            });
            run_handler_with_resume(&handler_params, &handler_body, arg, deep_resume, &env, host)
        }
        Outcome::Performed { op, arg, resume } => Ok(Outcome::Performed { op, arg, resume }),
    }
}

fn run_handler_with_resume(
    handler_params: &[String],
    handler_body: &CoreExpr,
    arg: RuntimeValue,
    resume: ResumeCont,
    env: &HashMap<String, RuntimeValue>,
    host: &mut dyn EffectHost,
) -> Result<Outcome, EvalError> {
    let mut child = env.clone();
    match handler_params {
        [p] => {
            child.insert(p.clone(), arg);
        }
        [p, resume_name] => {
            child.insert(p.clone(), arg);
            child.insert(
                resume_name.clone(),
                RuntimeValue::OneShotResume {
                    used: Rc::new(Cell::new(false)),
                    cont: resume,
                },
            );
        }
        _ => {
            return Err(EvalError {
                message: "handle handler expects 1 or 2 parameters".into(),
            });
        }
    }
    Ok(match eval_outcome(handler_body, &child, host)? {
        // Resume aborts the handler; its value is the handle result.
        Outcome::Resumed(v) => Outcome::Value(v),
        other => other,
    })
}

fn eval_seq(
    items: &[CoreExpr],
    env: &HashMap<String, RuntimeValue>,
    host: &mut dyn EffectHost,
) -> Result<Outcome, EvalError> {
    let mut last = RuntimeValue::Unit;
    for (i, item) in items.iter().enumerate() {
        match eval_outcome(item, env, host)? {
            Outcome::Value(v) => last = v,
            Outcome::Resumed(v) => return Ok(Outcome::Resumed(v)),
            Outcome::Performed {
                op,
                arg,
                resume: inner,
            } => {
                let rest: Vec<CoreExpr> = items[i + 1..].to_vec();
                let env = env.clone();
                return Ok(Outcome::Performed {
                    op,
                    arg,
                    resume: Rc::new(move |v, host| {
                        let mut last = match inner(v, host)? {
                            Outcome::Value(v) => v,
                            other => return Ok(other),
                        };
                        for item in &rest {
                            match eval_outcome(item, &env, host)? {
                                Outcome::Value(v) => last = v,
                                other => return Ok(other),
                            }
                        }
                        Ok(Outcome::Value(last))
                    }),
                });
            }
        }
    }
    Ok(Outcome::Value(last))
}

fn eval_app(
    fun: &CoreExpr,
    args: &[CoreExpr],
    env: &HashMap<String, RuntimeValue>,
    host: &mut dyn EffectHost,
) -> Result<Outcome, EvalError> {
    let fun_v = match eval_outcome(fun, env, host)? {
        Outcome::Value(v) => v,
        Outcome::Performed {
            op,
            arg,
            resume: inner,
        } => {
            let args = args.to_vec();
            let env = env.clone();
            return Ok(Outcome::Performed {
                op,
                arg,
                resume: Rc::new(move |v, host| {
                    let fun_v = match inner(v, host)? {
                        Outcome::Value(v) => v,
                        other => return Ok(other),
                    };
                    eval_app_args(fun_v, &args, &env, host)
                }),
            });
        }
        Outcome::Resumed(v) => return Ok(Outcome::Resumed(v)),
    };
    eval_app_args(fun_v, args, env, host)
}

fn eval_app_args(
    fun_v: RuntimeValue,
    args: &[CoreExpr],
    env: &HashMap<String, RuntimeValue>,
    host: &mut dyn EffectHost,
) -> Result<Outcome, EvalError> {
    let mut arg_vs = Vec::with_capacity(args.len());
    for (i, arg) in args.iter().enumerate() {
        match eval_outcome(arg, env, host)? {
            Outcome::Value(v) => arg_vs.push(v),
            Outcome::Performed {
                op,
                arg: performed_arg,
                resume: inner,
            } => {
                let fun_v = fun_v.clone();
                let done = arg_vs.clone();
                let rest: Vec<CoreExpr> = args[i + 1..].to_vec();
                let env = env.clone();
                return Ok(Outcome::Performed {
                    op,
                    arg: performed_arg,
                    resume: Rc::new(move |v, host| {
                        let v = match inner(v, host)? {
                            Outcome::Value(v) => v,
                            other => return Ok(other),
                        };
                        let mut collected = done.clone();
                        collected.push(v);
                        for a in &rest {
                            match eval_outcome(a, &env, host)? {
                                Outcome::Value(v) => collected.push(v),
                                other => return Ok(other),
                            }
                        }
                        apply_value(fun_v.clone(), collected, host)
                    }),
                });
            }
            Outcome::Resumed(v) => return Ok(Outcome::Resumed(v)),
        }
    }
    apply_value(fun_v, arg_vs, host)
}

fn eval_record(
    fields: &[(String, CoreExpr)],
    env: &HashMap<String, RuntimeValue>,
    host: &mut dyn EffectHost,
) -> Result<Outcome, EvalError> {
    let mut out = Vec::new();
    for (i, (k, v)) in fields.iter().enumerate() {
        match eval_outcome(v, env, host)? {
            Outcome::Value(val) => out.push((k.clone(), val)),
            Outcome::Performed {
                op,
                arg,
                resume: inner,
            } => {
                let key = k.clone();
                let done = out.clone();
                let rest: Vec<(String, CoreExpr)> = fields[i + 1..].to_vec();
                let env = env.clone();
                return Ok(Outcome::Performed {
                    op,
                    arg,
                    resume: Rc::new(move |v, host| {
                        let v = match inner(v, host)? {
                            Outcome::Value(v) => v,
                            other => return Ok(other),
                        };
                        let mut collected = done.clone();
                        collected.push((key.clone(), v));
                        for (k, e) in &rest {
                            match eval_outcome(e, &env, host)? {
                                Outcome::Value(val) => collected.push((k.clone(), val)),
                                other => return Ok(other),
                            }
                        }
                        Ok(Outcome::Value(RuntimeValue::Record(collected)))
                    }),
                });
            }
            Outcome::Resumed(v) => return Ok(Outcome::Resumed(v)),
        }
    }
    Ok(Outcome::Value(RuntimeValue::Record(out)))
}

fn apply_set(
    name: &str,
    v: RuntimeValue,
    env: &HashMap<String, RuntimeValue>,
) -> Result<Outcome, EvalError> {
    match env.get(name) {
        Some(RuntimeValue::Cell { value: cell, alive }) => {
            if !alive.get() {
                return Err(EvalError {
                    message: format!("set on var `{name}` after scope exit (escaped)"),
                });
            }
            *cell.borrow_mut() = v;
            Ok(Outcome::Value(RuntimeValue::Unit))
        }
        Some(_) => Err(EvalError {
            message: format!("`set` target `{name}` is not a var cell"),
        }),
        None => Err(EvalError {
            message: format!("unbound variable `{name}` in set"),
        }),
    }
}

fn record_get(v: RuntimeValue, field: &str) -> Result<Outcome, EvalError> {
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

fn apply_value(
    fun_v: RuntimeValue,
    arg_vs: Vec<RuntimeValue>,
    host: &mut dyn EffectHost,
) -> Result<Outcome, EvalError> {
    match fun_v {
        RuntimeValue::OneShotResume { used, cont } => {
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
            // Transfer to deep continuation; abandon rest of handler.
            match cont(arg_vs.into_iter().next().expect("len 1"), host)? {
                Outcome::Value(v) => Ok(Outcome::Resumed(v)),
                Outcome::Resumed(v) => Ok(Outcome::Resumed(v)),
                Outcome::Performed { op, arg, resume } => {
                    Ok(Outcome::Performed { op, arg, resume })
                }
            }
        }
        RuntimeValue::Builtin(op) => apply_builtin(op, arg_vs),
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
                if param != "_" {
                    child.insert(param, arg_v);
                }
            }
            eval_outcome(&body, &child, host)
        }
        other => Err(EvalError {
            message: format!("expected closure, got {other:?}"),
        }),
    }
}

fn eval_match(
    value: &RuntimeValue,
    arms: &[MatchArm],
    env: &HashMap<String, RuntimeValue>,
    host: &mut dyn EffectHost,
) -> Result<Outcome, EvalError> {
    for arm in arms {
        if let Some(bindings) = match_pattern(&arm.pattern, value) {
            let mut child = env.clone();
            child.extend(bindings);
            return eval_outcome(&arm.body, &child, host);
        }
    }
    Err(EvalError {
        message: format!("no matching arm for value `{value:?}`"),
    })
}

fn match_pattern(pat: &CorePattern, value: &RuntimeValue) -> Option<HashMap<String, RuntimeValue>> {
    match pat {
        CorePattern::Wildcard => Some(HashMap::new()),
        CorePattern::Bind(name) => {
            let mut m = HashMap::new();
            m.insert(name.clone(), value.clone());
            Some(m)
        }
        CorePattern::Lit(lit) => {
            let ok = match (lit, value) {
                (CoreLiteral::Number(n), RuntimeValue::Number(v)) => n == v,
                (CoreLiteral::String(s), RuntimeValue::String(v)) => s == v,
                (CoreLiteral::Bool(b), RuntimeValue::Bool(v)) => b == v,
                (CoreLiteral::Unit, RuntimeValue::Unit) => true,
                (CoreLiteral::Color(_), _) => false,
                _ => false,
            };
            if ok {
                Some(HashMap::new())
            } else {
                None
            }
        }
        CorePattern::Tuple(elems) => {
            let RuntimeValue::Record(fields) = value else {
                return None;
            };
            if fields.len() != elems.len() {
                return None;
            }
            let mut out = HashMap::new();
            for (i, ep) in elems.iter().enumerate() {
                let key = i.to_string();
                let (_, fv) = fields.iter().find(|(k, _)| *k == key)?;
                let sub = match_pattern(ep, fv)?;
                out.extend(sub);
            }
            Some(out)
        }
        CorePattern::Variant { tag, payload } => {
            let RuntimeValue::Variant {
                tag: vtag,
                payload: vp,
            } = value
            else {
                return None;
            };
            if vtag != tag {
                return None;
            }
            match payload {
                None => Some(HashMap::new()),
                Some(inner) => match vp {
                    Some(v) => match_pattern(inner, v),
                    None => match inner.as_ref() {
                        // Nullary variant with a simple binder/wildcard: match tag, no bind.
                        CorePattern::Wildcard | CorePattern::Bind(_) => Some(HashMap::new()),
                        CorePattern::Lit(_)
                        | CorePattern::Tuple(_)
                        | CorePattern::Variant { .. } => None,
                    },
                },
            }
        }
    }
}

fn apply_builtin(op: BuiltinOp, args: Vec<RuntimeValue>) -> Result<Outcome, EvalError> {
    if args.len() != 2 {
        return Err(EvalError {
            message: format!("builtin `{op:?}` expects 2 args, got {}", args.len()),
        });
    }
    let a = &args[0];
    let b = &args[1];
    match op {
        BuiltinOp::Add
        | BuiltinOp::Sub
        | BuiltinOp::Mul
        | BuiltinOp::Div
        | BuiltinOp::Lt
        | BuiltinOp::Gt
        | BuiltinOp::Le
        | BuiltinOp::Ge => {
            let (RuntimeValue::Number(x), RuntimeValue::Number(y)) = (a, b) else {
                return Err(EvalError {
                    message: format!("builtin `{op:?}` expects Number arguments"),
                });
            };
            Ok(Outcome::Value(match op {
                BuiltinOp::Add => RuntimeValue::Number(x + y),
                BuiltinOp::Sub => RuntimeValue::Number(x - y),
                BuiltinOp::Mul => RuntimeValue::Number(x * y),
                BuiltinOp::Div => RuntimeValue::Number(x / y),
                BuiltinOp::Lt => RuntimeValue::Bool(x < y),
                BuiltinOp::Gt => RuntimeValue::Bool(x > y),
                BuiltinOp::Le => RuntimeValue::Bool(x <= y),
                BuiltinOp::Ge => RuntimeValue::Bool(x >= y),
                BuiltinOp::Eq | BuiltinOp::Ne => unreachable!(),
            }))
        }
        BuiltinOp::Eq => Ok(Outcome::Value(RuntimeValue::Bool(a == b))),
        BuiltinOp::Ne => Ok(Outcome::Value(RuntimeValue::Bool(a != b))),
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
        CoreLiteral::Bool(b) => RuntimeValue::Bool(*b),
        CoreLiteral::Unit => RuntimeValue::Unit,
    })
}
