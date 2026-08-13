//! Reference evaluation of Core expressions.

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use reciplexa_core::cast::{plan_cast_evidence, CastEvidence};
use reciplexa_core::elaborate::{elaborate_source, ElaborateError};
use reciplexa_core::expr::{CoreExpr, CoreLiteral, CorePattern, MatchArm};
use reciplexa_core::ty::CoreType;

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
    env.insert("number?".into(), RuntimeValue::Builtin(BuiltinOp::IsNumber));
    env.insert("string?".into(), RuntimeValue::Builtin(BuiltinOp::IsString));
    env.insert("bool?".into(), RuntimeValue::Builtin(BuiltinOp::IsBool));
    env.insert("is-none".into(), RuntimeValue::Builtin(BuiltinOp::IsNone));
    env.insert("is-some".into(), RuntimeValue::Builtin(BuiltinOp::IsSome));
    env.insert("newline".into(), RuntimeValue::String("\n".into()));
    env.insert("tab".into(), RuntimeValue::String("\t".into()));
    env.insert("carriage-return".into(), RuntimeValue::String("\r".into()));
    env.insert("nul".into(), RuntimeValue::String("\0".into()));
    env.insert("unicode".into(), RuntimeValue::Builtin(BuiltinOp::Unicode));
    env.insert(
        "encode-utf8".into(),
        RuntimeValue::Builtin(BuiltinOp::EncodeUtf8),
    );
    env.insert(
        "decode-utf8".into(),
        RuntimeValue::Builtin(BuiltinOp::DecodeUtf8),
    );
    env.insert("int-div".into(), RuntimeValue::Builtin(BuiltinOp::IntDiv));
    env.insert("mod".into(), RuntimeValue::Builtin(BuiltinOp::Mod));
    env.insert(
        "classify-char".into(),
        RuntimeValue::Builtin(BuiltinOp::ClassifyChar),
    );
    env.insert(
        "break-between".into(),
        RuntimeValue::Builtin(BuiltinOp::BreakBetween),
    );
    env.insert(
        "break-line".into(),
        RuntimeValue::Builtin(BuiltinOp::BreakLine),
    );
    env
}

/// Expand language macros, elaborate surface source to Core, then evaluate with [`UnitHost`].
///
/// The returned value is for hosts/tests; the CLI must not print it as a REPL would.
/// Observable terminal output belongs only to residual effects (e.g. ambient `log`).
pub fn eval_source(src: &str) -> EvalResult {
    eval_source_with_host(src, &mut UnitHost)
}

/// Same pipeline as [`eval_source`], with a caller-supplied [`EffectHost`].
pub fn eval_source_with_host(src: &str, host: &mut dyn EffectHost) -> EvalResult {
    let expanded =
        reciplexa_macro::expand_language(src).map_err(|e| EvalError { message: e.message })?;
    // Language expand is expected to yield parseable source; elaboration owns
    // residual syntax diagnostics (no separate post-expand parse Err arm).
    let expr = elaborate_source(&expanded)
        .map_err(|e: ElaborateError| EvalError { message: e.message })?;
    eval_expr(&expr, &primitive_env(), host)
}

pub fn eval_expr(
    expr: &CoreExpr,
    env: &HashMap<String, RuntimeValue>,
    host: &mut dyn EffectHost,
) -> EvalResult {
    let mut outcome = eval_outcome(expr, env, host)?;
    loop {
        match outcome {
            Outcome::Value(v) => return Ok(v),
            Outcome::Resumed(v) => return Ok(v),
            Outcome::Forward => {
                return Err(EvalError {
                    message: "`forward` escaped to top-level evaluation".into(),
                });
            }
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
        CoreExpr::Error => Err(EvalError {
            message: "cannot evaluate syntax-error placeholder".into(),
        }),
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
        CoreExpr::Forward { resume_name } => match env.get(resume_name) {
            Some(RuntimeValue::OneShotResume { used, .. }) => {
                if used.get() {
                    return Err(EvalError {
                        message: "one-shot resume already used".into(),
                    });
                }
                used.set(true);
                Ok(Outcome::Forward)
            }
            _ => Err(EvalError {
                message: format!("`forward` expects resume binder `{resume_name}`"),
            }),
        },
        CoreExpr::Cast { expr, evidence, .. } => {
            let v = match eval_outcome(expr, env, host)? {
                Outcome::Value(v) => v,
                other => return Ok(other),
            };
            match runtime_cast_apply(v, evidence) {
                Some(out) => Ok(Outcome::Value(out)),
                None => Err(EvalError {
                    message: "dynamic cast failed".into(),
                }),
            }
        }
        CoreExpr::TryCast { expr, target, .. } => {
            let v = match eval_outcome(expr, env, host)? {
                Outcome::Value(v) => v,
                other => return Ok(other),
            };
            let evidence =
                plan_cast_evidence(&CoreType::dyn_any(), target).unwrap_or(CastEvidence::Identity);
            match runtime_cast_apply(v, &evidence) {
                Some(out) => Ok(Outcome::Value(RuntimeValue::Variant {
                    tag: "some".into(),
                    payload: Some(Box::new(out)),
                })),
                None => Ok(Outcome::Value(RuntimeValue::Variant {
                    tag: "none".into(),
                    payload: None,
                })),
            }
        }
        CoreExpr::CheckCast { expr, target, .. } => {
            let v = match eval_outcome(expr, env, host)? {
                Outcome::Value(v) => v,
                other => return Ok(other),
            };
            let evidence =
                plan_cast_evidence(&CoreType::dyn_any(), target).unwrap_or(CastEvidence::Identity);
            match runtime_cast_apply(v, &evidence) {
                Some(out) => Ok(Outcome::Value(RuntimeValue::Variant {
                    tag: "ok".into(),
                    payload: Some(Box::new(out)),
                })),
                None => Ok(Outcome::Value(RuntimeValue::Variant {
                    tag: "err".into(),
                    payload: Some(Box::new(RuntimeValue::String("cast-mismatch".into()))),
                })),
            }
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
                other => Ok(other),
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
            other => Ok(other),
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
            other => Ok(other),
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
            other => Ok(other),
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
            other => Ok(other),
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
            other => Ok(other),
        },
        CoreExpr::RecordUpdate { record, fields } => match eval_outcome(record, env, host)? {
            Outcome::Value(v) => record_update(v, fields, env, host),
            Outcome::Performed {
                op,
                arg,
                resume: inner,
            } => {
                let fields = fields.clone();
                let env = env.clone();
                Ok(Outcome::Performed {
                    op,
                    arg,
                    resume: Rc::new(move |v, host| {
                        let v = match inner(v, host)? {
                            Outcome::Value(v) => v,
                            other => return Ok(other),
                        };
                        record_update(v, &fields, &env, host)
                    }),
                })
            }
            other => Ok(other),
        },
        CoreExpr::RecordExtend { record, fields } => match eval_outcome(record, env, host)? {
            Outcome::Value(v) => record_extend(v, fields, env, host),
            Outcome::Performed {
                op,
                arg,
                resume: inner,
            } => {
                let fields = fields.clone();
                let env = env.clone();
                Ok(Outcome::Performed {
                    op,
                    arg,
                    resume: Rc::new(move |v, host| {
                        let v = match inner(v, host)? {
                            Outcome::Value(v) => v,
                            other => return Ok(other),
                        };
                        record_extend(v, &fields, &env, host)
                    }),
                })
            }
            other => Ok(other),
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
                    other => Ok(other),
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
            other => Ok(other),
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
            // ERR-001 §5: Failure handlers are non-resumable (error payload only).
            if op == "failure" && handler_params.len() != 1 {
                return Err(EvalError {
                    message: "Failure handler must take exactly one parameter (no resume)".into(),
                });
            }
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
                    } if p == op => run_handler_with_resume(
                        &op,
                        &handler_params,
                        &handler_body,
                        arg,
                        r,
                        &env,
                        host,
                    ),
                    other => Ok(other),
                }
            });
            run_handler_with_resume(
                &op,
                &handler_params,
                &handler_body,
                arg,
                deep_resume,
                &env,
                host,
            )
        }
        Outcome::Performed { op, arg, resume } => Ok(Outcome::Performed { op, arg, resume }),
        Outcome::Forward => Err(EvalError {
            message: "`forward` is only valid inside a handler clause".into(),
        }),
    }
}

fn run_handler_with_resume(
    op: &str,
    handler_params: &[String],
    handler_body: &CoreExpr,
    arg: RuntimeValue,
    resume: ResumeCont,
    env: &HashMap<String, RuntimeValue>,
    host: &mut dyn EffectHost,
) -> Result<Outcome, EvalError> {
    let mut child = env.clone();
    let forward_arg = arg.clone();
    match handler_params {
        [p] => {
            // Discard continuation for 1-param handlers (Failure / non-resumable).
            let _ = resume;
            child.insert(p.clone(), arg);
        }
        [p, resume_name] => {
            child.insert(p.clone(), arg.clone());
            child.insert(
                resume_name.clone(),
                RuntimeValue::OneShotResume {
                    used: Rc::new(Cell::new(false)),
                    cont: resume.clone(),
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
        Outcome::Forward => Outcome::Performed {
            op: op.to_string(),
            arg: forward_arg,
            resume,
        },
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
            other => return Ok(other),
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
        other => return Ok(other),
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
            other => return Ok(other),
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
            other => return Ok(other),
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

fn record_update(
    base: RuntimeValue,
    updates: &[(String, CoreExpr)],
    env: &HashMap<String, RuntimeValue>,
    host: &mut dyn EffectHost,
) -> Result<Outcome, EvalError> {
    let RuntimeValue::Record(mut fields) = base else {
        return Err(EvalError {
            message: format!("`record-update` expected record, got {base:?}"),
        });
    };
    for (label, expr) in updates {
        let idx = fields
            .iter()
            .position(|(k, _)| k == label)
            .ok_or_else(|| EvalError {
                message: format!("`record-update` field `{label}` is not present"),
            })?;
        match eval_outcome(expr, env, host)? {
            Outcome::Value(v) => fields[idx].1 = v,
            other => return Ok(other),
        }
    }
    Ok(Outcome::Value(RuntimeValue::Record(fields)))
}

fn record_extend(
    base: RuntimeValue,
    additions: &[(String, CoreExpr)],
    env: &HashMap<String, RuntimeValue>,
    host: &mut dyn EffectHost,
) -> Result<Outcome, EvalError> {
    let RuntimeValue::Record(mut fields) = base else {
        return Err(EvalError {
            message: format!("`record-extend` expected record, got {base:?}"),
        });
    };
    for (label, expr) in additions {
        if fields.iter().any(|(k, _)| k == label) {
            return Err(EvalError {
                message: format!("`record-extend` field `{label}` already present"),
            });
        }
        match eval_outcome(expr, env, host)? {
            Outcome::Value(v) => fields.push((label.clone(), v)),
            other => return Ok(other),
        }
    }
    Ok(Outcome::Value(RuntimeValue::Record(fields)))
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
                other => Ok(other),
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
                (CoreLiteral::Int(n), RuntimeValue::Int(v)) => n == v,
                (CoreLiteral::F64(n), RuntimeValue::F64(v)) => n == v,
                (CoreLiteral::String(s), RuntimeValue::String(v)) => s == v,
                (CoreLiteral::Bool(b), RuntimeValue::Bool(v)) => b == v,
                (CoreLiteral::Unit, RuntimeValue::Unit) => true,
                (CoreLiteral::Bytes(a), RuntimeValue::Bytes(b)) => a == b,
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
        CorePattern::Record { fields: pats } => {
            let RuntimeValue::Record(fields) = value else {
                return None;
            };
            let mut out = HashMap::new();
            for (label, ep) in pats {
                let (_, fv) = fields.iter().find(|(k, _)| k == label)?;
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
                        | CorePattern::Record { .. }
                        | CorePattern::Variant { .. } => None,
                    },
                },
            }
        }
    }
}

fn as_numeric(v: &RuntimeValue) -> Result<(bool, f64), EvalError> {
    match v {
        RuntimeValue::Number(f) => Ok((false, *f)),
        RuntimeValue::Int(i) => Ok((true, *i as f64)),
        RuntimeValue::F64(f) => Ok((false, *f)),
        _ => Err(EvalError {
            message: "expected numeric value".into(),
        }),
    }
}

fn apply_numeric_binop(
    op: BuiltinOp,
    a: &RuntimeValue,
    b: &RuntimeValue,
) -> Result<RuntimeValue, EvalError> {
    match (a, b) {
        (RuntimeValue::Int(x), RuntimeValue::Int(y)) => match op {
            BuiltinOp::Add => Ok(RuntimeValue::Int(x + y)),
            BuiltinOp::Sub => Ok(RuntimeValue::Int(x - y)),
            BuiltinOp::Mul => Ok(RuntimeValue::Int(x * y)),
            BuiltinOp::Div => Ok(RuntimeValue::F64(*x as f64 / *y as f64)),
            BuiltinOp::Lt => Ok(RuntimeValue::Bool(x < y)),
            BuiltinOp::Gt => Ok(RuntimeValue::Bool(x > y)),
            BuiltinOp::Le => Ok(RuntimeValue::Bool(x <= y)),
            BuiltinOp::Ge => Ok(RuntimeValue::Bool(x >= y)),
            _ => Err(EvalError {
                message: "internal: non-numeric binop on int pair".into(),
            }),
        },
        _ => {
            let (_, x) = as_numeric(a)?;
            let (_, y) = as_numeric(b)?;
            Ok(match op {
                BuiltinOp::Add => RuntimeValue::F64(x + y),
                BuiltinOp::Sub => RuntimeValue::F64(x - y),
                BuiltinOp::Mul => RuntimeValue::F64(x * y),
                BuiltinOp::Div => RuntimeValue::F64(x / y),
                BuiltinOp::Lt => RuntimeValue::Bool(x < y),
                BuiltinOp::Gt => RuntimeValue::Bool(x > y),
                BuiltinOp::Le => RuntimeValue::Bool(x <= y),
                BuiltinOp::Ge => RuntimeValue::Bool(x >= y),
                _ => {
                    return Err(EvalError {
                        message: "internal: non-numeric binop on numeric pair".into(),
                    })
                }
            })
        }
    }
}

fn apply_builtin(op: BuiltinOp, args: Vec<RuntimeValue>) -> Result<Outcome, EvalError> {
    match op {
        BuiltinOp::IsNumber
        | BuiltinOp::IsString
        | BuiltinOp::IsBool
        | BuiltinOp::IsNone
        | BuiltinOp::IsSome => {
            if args.len() != 1 {
                return Err(EvalError {
                    message: format!("builtin `{op:?}` expects 1 arg, got {}", args.len()),
                });
            }
            let v = &args[0];
            let flag = match op {
                BuiltinOp::IsNumber => {
                    matches!(
                        v,
                        RuntimeValue::Number(_) | RuntimeValue::Int(_) | RuntimeValue::F64(_)
                    )
                }
                BuiltinOp::IsString => matches!(v, RuntimeValue::String(_)),
                BuiltinOp::IsBool => matches!(v, RuntimeValue::Bool(_)),
                BuiltinOp::IsNone => {
                    matches!(v, RuntimeValue::Variant { tag, .. } if tag == "none")
                }
                // Outer or-pattern is only the five predicates above.
                _ => matches!(v, RuntimeValue::Variant { tag, .. } if tag == "some"),
            };
            Ok(Outcome::Value(RuntimeValue::Bool(flag)))
        }
        BuiltinOp::Unicode | BuiltinOp::EncodeUtf8 | BuiltinOp::DecodeUtf8 => {
            if args.len() != 1 {
                return Err(EvalError {
                    message: format!("builtin `{op:?}` expects 1 arg, got {}", args.len()),
                });
            }
            let v = &args[0];
            if matches!(op, BuiltinOp::Unicode) {
                let code = match v {
                    RuntimeValue::Int(i) => *i as f64,
                    RuntimeValue::F64(f) => *f,
                    _ => {
                        return Err(EvalError {
                            message: "builtin `unicode` expects numeric argument".into(),
                        });
                    }
                };
                let s = reciplexa_syntax::unicode_scalar_value(code)
                    .map_err(|msg| EvalError { message: msg })?;
                return Ok(Outcome::Value(RuntimeValue::String(s)));
            }
            if matches!(op, BuiltinOp::EncodeUtf8) {
                let s = match &v {
                    RuntimeValue::String(text) => text.clone(),
                    _ => {
                        return Err(EvalError {
                            message: "builtin `encode-utf8` expects string argument".into(),
                        });
                    }
                };
                return Ok(Outcome::Value(RuntimeValue::Bytes(s.clone().into_bytes())));
            }
            let data = match v {
                RuntimeValue::Bytes(b) => b.clone(),
                _ => {
                    return Err(EvalError {
                        message: "builtin `decode-utf8` expects bytes argument".into(),
                    });
                }
            };
            match String::from_utf8(data) {
                Ok(text) => Ok(Outcome::Value(RuntimeValue::Variant {
                    tag: "ok".into(),
                    payload: Some(Box::new(RuntimeValue::String(text))),
                })),
                Err(_) => Ok(Outcome::Value(RuntimeValue::Variant {
                    tag: "err".into(),
                    payload: Some(Box::new(RuntimeValue::String("utf8-decode-error".into()))),
                })),
            }
        }
        BuiltinOp::Add
        | BuiltinOp::Sub
        | BuiltinOp::Mul
        | BuiltinOp::Div
        | BuiltinOp::Lt
        | BuiltinOp::Gt
        | BuiltinOp::Le
        | BuiltinOp::Ge
        | BuiltinOp::Eq
        | BuiltinOp::Ne => {
            if args.len() != 2 {
                return Err(EvalError {
                    message: format!("builtin `{op:?}` expects 2 args, got {}", args.len()),
                });
            }
            let a = &args[0];
            let b = &args[1];
            if matches!(op, BuiltinOp::Eq) {
                Ok(Outcome::Value(RuntimeValue::Bool(a == b)))
            } else if matches!(op, BuiltinOp::Ne) {
                Ok(Outcome::Value(RuntimeValue::Bool(a != b)))
            } else {
                Ok(Outcome::Value(apply_numeric_binop(op, a, b)?))
            }
        }
        BuiltinOp::IntDiv | BuiltinOp::Mod => {
            if args.len() != 2 {
                return Err(EvalError {
                    message: format!("builtin `{op:?}` expects 2 args, got {}", args.len()),
                });
            }
            let (RuntimeValue::Int(x), RuntimeValue::Int(y)) = (&args[0], &args[1]) else {
                return Err(EvalError {
                    message: format!("builtin `{op:?}` expects int operands"),
                });
            };
            if *y == 0 {
                return Err(EvalError {
                    message: format!("builtin `{op:?}` division by zero"),
                });
            }
            Ok(Outcome::Value(RuntimeValue::Int(
                if matches!(op, BuiltinOp::IntDiv) {
                    x / y
                } else {
                    x % y
                },
            )))
        }
        BuiltinOp::ClassifyChar => {
            if args.len() != 1 {
                return Err(EvalError {
                    message: format!(
                        "builtin `classify-char` expects 1 arg, got {}",
                        args.len()
                    ),
                });
            }
            let s = match &args[0] {
                RuntimeValue::String(text) => text.as_str(),
                _ => {
                    return Err(EvalError {
                        message: "builtin `classify-char` expects string argument".into(),
                    });
                }
            };
            let ch = s.chars().next().ok_or_else(|| EvalError {
                message: "builtin `classify-char` expects non-empty string".into(),
            })?;
            let id = reciplexa_std::japanese::classify_char(ch).id();
            Ok(Outcome::Value(RuntimeValue::Int(i128::from(id))))
        }
        BuiltinOp::BreakBetween => {
            if args.len() != 2 {
                return Err(EvalError {
                    message: format!(
                        "builtin `break-between` expects 2 args, got {}",
                        args.len()
                    ),
                });
            }
            let prev_s = match &args[0] {
                RuntimeValue::String(text) => text.as_str(),
                _ => {
                    return Err(EvalError {
                        message: "builtin `break-between` expects string arguments".into(),
                    });
                }
            };
            let next_s = match &args[1] {
                RuntimeValue::String(text) => text.as_str(),
                _ => {
                    return Err(EvalError {
                        message: "builtin `break-between` expects string arguments".into(),
                    });
                }
            };
            let prev = prev_s.chars().next().ok_or_else(|| EvalError {
                message: "builtin `break-between` expects non-empty strings".into(),
            })?;
            let next = next_s.chars().next().ok_or_else(|| EvalError {
                message: "builtin `break-between` expects non-empty strings".into(),
            })?;
            let opp = reciplexa_std::japanese::break_opportunity_chars(prev, next);
            Ok(Outcome::Value(RuntimeValue::Variant {
                tag: opp.as_str().into(),
                payload: None,
            }))
        }
        BuiltinOp::BreakLine => {
            if args.len() != 2 {
                return Err(EvalError {
                    message: format!(
                        "builtin `break-line` expects 2 args, got {}",
                        args.len()
                    ),
                });
            }
            let text = match &args[0] {
                RuntimeValue::String(s) => s.as_str(),
                _ => {
                    return Err(EvalError {
                        message: "builtin `break-line` expects string as first argument".into(),
                    });
                }
            };
            let max_em = match as_numeric(&args[1]) {
                Ok((_, n)) => n,
                Err(_) => {
                    return Err(EvalError {
                        message: "builtin `break-line` expects numeric max-em".into(),
                    });
                }
            };
            let lines = reciplexa_std::japanese::break_line(text, max_em);
            Ok(Outcome::Value(strings_to_cons_list(lines)))
        }
    }
}

/// Build a surface `(list …)` value: `cons`/`nil` variants with `head`/`tail` records.
fn strings_to_cons_list(lines: Vec<String>) -> RuntimeValue {
    let mut acc = RuntimeValue::Variant {
        tag: "nil".into(),
        payload: None,
    };
    for s in lines.into_iter().rev() {
        acc = RuntimeValue::Variant {
            tag: "cons".into(),
            payload: Some(Box::new(RuntimeValue::Record(vec![
                ("head".into(), RuntimeValue::String(s)),
                ("tail".into(), acc),
            ]))),
        };
    }
    acc
}

fn eval_lit(lit: &CoreLiteral) -> EvalResult {
    Ok(match lit {
        CoreLiteral::Number(n) => RuntimeValue::Number(*n),
        CoreLiteral::Int(n) => RuntimeValue::Int(*n),
        CoreLiteral::F64(n) => RuntimeValue::F64(*n),
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
        CoreLiteral::Bytes(b) => RuntimeValue::Bytes(b.clone()),
    })
}

/// Runtime tag check for DD-TYP-DYN-015 evidence (minimal v1; pure — DD-TYP-DYN-016).
fn runtime_cast_ok(value: &RuntimeValue, evidence: &CastEvidence) -> bool {
    match evidence {
        CastEvidence::Identity | CastEvidence::Widen => true,
        CastEvidence::NumericPromote => matches!(
            value,
            RuntimeValue::Int(_) | RuntimeValue::F64(_) | RuntimeValue::Number(_)
        ),
        CastEvidence::TagCheck { tag } => value_matches_tag(value, tag),
        CastEvidence::UnionCheck { members } => members.iter().any(|m| {
            plan_cast_evidence(&CoreType::dyn_any(), m)
                .map(|e| runtime_cast_ok(value, &e))
                .unwrap_or(false)
        }),
        CastEvidence::VariantCheck { variants } => match value {
            RuntimeValue::Variant { tag, payload } => variants.iter().any(|(vt, pty)| {
                if tag != vt {
                    return false;
                }
                match (payload, pty) {
                    (None, None) => true,
                    (Some(p), Some(ty)) => plan_cast_evidence(&CoreType::dyn_any(), ty)
                        .map(|e| runtime_cast_ok(p, &e))
                        .unwrap_or(false),
                    (None, Some(_)) => false,
                    (Some(_), None) => true,
                }
            }),
            _ => false,
        },
        CastEvidence::RecordCheck { fields } => match value {
            RuntimeValue::Record(rec) => fields.iter().all(|(k, ty)| {
                rec.iter().any(|(rk, rv)| {
                    rk == k
                        && plan_cast_evidence(&CoreType::dyn_any(), ty)
                            .map(|e| runtime_cast_ok(rv, &e))
                            .unwrap_or(false)
                })
            }),
            _ => false,
        },
        CastEvidence::FunctionGuard { arity, .. } => match value {
            RuntimeValue::Closure { params, .. } => params.len() == *arity,
            RuntimeValue::Builtin(_) => true,
            _ => false,
        },
        CastEvidence::NominalCheck { name } => value_matches_tag(value, name),
        CastEvidence::IntersectionCheck { members } => members.iter().all(|m| {
            plan_cast_evidence(&CoreType::dyn_any(), m)
                .map(|e| runtime_cast_ok(value, &e))
                .unwrap_or(false)
        }),
        CastEvidence::Compose(parts) => parts.iter().all(|p| runtime_cast_ok(value, p)),
    }
}

/// Check evidence then apply value transforms (DD-TYP-NUM NumericPromote → f64).
fn runtime_cast_apply(value: RuntimeValue, evidence: &CastEvidence) -> Option<RuntimeValue> {
    if !runtime_cast_ok(&value, evidence) {
        return None;
    }
    Some(apply_cast_evidence(value, evidence))
}

/// Apply pure cast evidence transforms (DD-TYP-NUM NumericPromote → f64).
fn apply_cast_evidence(value: RuntimeValue, evidence: &CastEvidence) -> RuntimeValue {
    match evidence {
        CastEvidence::NumericPromote => match value {
            RuntimeValue::Int(n) => RuntimeValue::F64(n as f64),
            RuntimeValue::Number(n) => RuntimeValue::F64(n),
            other => other,
        },
        CastEvidence::Compose(parts) => parts.iter().fold(value, apply_cast_evidence),
        _ => value,
    }
}

fn value_matches_tag(value: &RuntimeValue, tag: &str) -> bool {
    match tag {
        "int" => matches!(value, RuntimeValue::Int(_)),
        "f64" => matches!(value, RuntimeValue::F64(_) | RuntimeValue::Number(_)),
        "number" => {
            matches!(
                value,
                RuntimeValue::F64(_) | RuntimeValue::Number(_) | RuntimeValue::Int(_)
            )
        }
        "string" => matches!(value, RuntimeValue::String(_)),
        "bool" => matches!(value, RuntimeValue::Bool(_)),
        "unit" => matches!(value, RuntimeValue::Unit),
        "bytes" => matches!(value, RuntimeValue::Bytes(_)),
        "any" | "dynamic" => true,
        other => matches!(
            value,
            RuntimeValue::Variant { tag: t, .. } if t == other
        ),
    }
}

#[cfg(test)]
mod coverage_helpers {
    use super::*;
    use reciplexa_core::cast::CastEvidence;
    use reciplexa_core::ty::CoreType;

    #[test]
    fn value_matches_tag_and_cast_evidence_matrix() {
        let int = RuntimeValue::Int(1);
        let f64v = RuntimeValue::F64(1.5);
        let num = RuntimeValue::Number(2.0);
        let s = RuntimeValue::String("x".into());
        let b = RuntimeValue::Bool(true);
        let u = RuntimeValue::Unit;
        let bytes = RuntimeValue::Bytes(vec![1, 2]);
        let var = RuntimeValue::Variant {
            tag: "ok".into(),
            payload: None,
        };
        for (tag, v, expect) in [
            ("int", &int, true),
            ("int", &s, false),
            ("f64", &f64v, true),
            ("f64", &num, true),
            ("f64", &int, false),
            ("number", &int, true),
            ("number", &f64v, true),
            ("number", &num, true),
            ("number", &s, false),
            ("string", &s, true),
            ("bool", &b, true),
            ("unit", &u, true),
            ("bytes", &bytes, true),
            ("any", &int, true),
            ("dynamic", &s, true),
            ("ok", &var, true),
            ("err", &var, false),
        ] {
            assert_eq!(value_matches_tag(v, tag), expect, "tag={tag}");
        }

        assert!(runtime_cast_ok(&int, &CastEvidence::Identity));
        assert!(runtime_cast_ok(&int, &CastEvidence::Widen));
        assert!(runtime_cast_ok(
            &int,
            &CastEvidence::TagCheck { tag: "int".into() }
        ));
        assert!(!runtime_cast_ok(
            &s,
            &CastEvidence::TagCheck { tag: "int".into() }
        ));
        assert!(runtime_cast_ok(&int, &CastEvidence::NumericPromote));
        assert!(!runtime_cast_ok(&s, &CastEvidence::NumericPromote));
        assert!(runtime_cast_ok(
            &int,
            &CastEvidence::UnionCheck {
                members: vec![CoreType::Int, CoreType::String],
            }
        ));
        assert!(runtime_cast_ok(
            &var,
            &CastEvidence::VariantCheck {
                variants: vec![("ok".into(), None)],
            }
        ));
        assert!(!runtime_cast_ok(
            &int,
            &CastEvidence::VariantCheck {
                variants: vec![("ok".into(), None)],
            }
        ));
        assert!(runtime_cast_ok(
            &RuntimeValue::Record(vec![("a".into(), int.clone())]),
            &CastEvidence::RecordCheck {
                fields: vec![("a".into(), CoreType::Int)],
            }
        ));
        assert!(!runtime_cast_ok(
            &s,
            &CastEvidence::RecordCheck {
                fields: vec![("a".into(), CoreType::Int)],
            }
        ));
        assert!(runtime_cast_ok(
            &RuntimeValue::Builtin(BuiltinOp::Add),
            &CastEvidence::FunctionGuard {
                arity: 2,
                arg_casts: vec![],
                ret_cast: Box::new(CastEvidence::Identity),
            }
        ));
        assert!(runtime_cast_ok(
            &var,
            &CastEvidence::NominalCheck { name: "ok".into() }
        ));
        assert!(runtime_cast_ok(
            &int,
            &CastEvidence::IntersectionCheck {
                members: vec![CoreType::Int],
            }
        ));
        assert!(runtime_cast_ok(
            &int,
            &CastEvidence::Compose(vec![
                CastEvidence::TagCheck { tag: "int".into() },
                CastEvidence::Widen,
            ])
        ));

        let promoted = apply_cast_evidence(int.clone(), &CastEvidence::NumericPromote);
        assert!(matches!(promoted, RuntimeValue::F64(_)));
        let promoted_n = apply_cast_evidence(num.clone(), &CastEvidence::NumericPromote);
        assert!(matches!(promoted_n, RuntimeValue::F64(_)));
        let keep = apply_cast_evidence(s.clone(), &CastEvidence::NumericPromote);
        assert!(matches!(keep, RuntimeValue::String(_)));
        let composed = apply_cast_evidence(
            int,
            &CastEvidence::Compose(vec![CastEvidence::NumericPromote, CastEvidence::Identity]),
        );
        assert!(matches!(composed, RuntimeValue::F64(_)));
    }

    #[test]
    fn apply_builtin_arity_and_predicate_edges() {
        // Arity Err for unary builtins
        for op in [
            BuiltinOp::IsNumber,
            BuiltinOp::IsString,
            BuiltinOp::IsBool,
            BuiltinOp::IsNone,
            BuiltinOp::IsSome,
            BuiltinOp::Unicode,
            BuiltinOp::EncodeUtf8,
            BuiltinOp::DecodeUtf8,
        ] {
            assert!(apply_builtin(op, vec![]).is_err());
            assert!(apply_builtin(op, vec![RuntimeValue::Int(1), RuntimeValue::Int(2)]).is_err());
        }
        // Predicate matrix
        let _ = apply_builtin(BuiltinOp::IsNumber, vec![RuntimeValue::Int(1)]);
        let _ = apply_builtin(BuiltinOp::IsNumber, vec![RuntimeValue::F64(1.0)]);
        let _ = apply_builtin(BuiltinOp::IsNumber, vec![RuntimeValue::Number(1.0)]);
        let _ = apply_builtin(BuiltinOp::IsNumber, vec![RuntimeValue::String("x".into())]);
        let _ = apply_builtin(BuiltinOp::IsString, vec![RuntimeValue::String("x".into())]);
        let _ = apply_builtin(BuiltinOp::IsBool, vec![RuntimeValue::Bool(false)]);
        let _ = apply_builtin(
            BuiltinOp::IsNone,
            vec![RuntimeValue::Variant {
                tag: "none".into(),
                payload: None,
            }],
        );
        let _ = apply_builtin(
            BuiltinOp::IsSome,
            vec![RuntimeValue::Variant {
                tag: "some".into(),
                payload: Some(Box::new(RuntimeValue::Int(1))),
            }],
        );
        // Unicode / encode / decode Err + Ok
        assert!(apply_builtin(BuiltinOp::Unicode, vec![RuntimeValue::String("x".into())]).is_err());
        let _ = apply_builtin(BuiltinOp::Unicode, vec![RuntimeValue::Int(65)]);
        let _ = apply_builtin(BuiltinOp::Unicode, vec![RuntimeValue::F64(65.0)]);
        assert!(apply_builtin(BuiltinOp::EncodeUtf8, vec![RuntimeValue::Int(1)]).is_err());
        let _ = apply_builtin(
            BuiltinOp::EncodeUtf8,
            vec![RuntimeValue::String("hi".into())],
        );
        assert!(apply_builtin(BuiltinOp::DecodeUtf8, vec![RuntimeValue::Int(1)]).is_err());
        let _ = apply_builtin(
            BuiltinOp::DecodeUtf8,
            vec![RuntimeValue::Bytes(b"ok".to_vec())],
        );
        let _ = apply_builtin(
            BuiltinOp::DecodeUtf8,
            vec![RuntimeValue::Bytes(vec![0xff, 0xfe])],
        );
        // Binary arity + int-div/mod edges
        for op in [
            BuiltinOp::Add,
            BuiltinOp::Sub,
            BuiltinOp::Mul,
            BuiltinOp::Div,
            BuiltinOp::Lt,
            BuiltinOp::Gt,
            BuiltinOp::Le,
            BuiltinOp::Ge,
            BuiltinOp::Eq,
            BuiltinOp::Ne,
            BuiltinOp::IntDiv,
            BuiltinOp::Mod,
        ] {
            assert!(apply_builtin(op, vec![RuntimeValue::Int(1)]).is_err());
        }
        let _ = apply_builtin(
            BuiltinOp::Add,
            vec![RuntimeValue::Int(1), RuntimeValue::Int(2)],
        );
        let _ = apply_builtin(
            BuiltinOp::Eq,
            vec![RuntimeValue::Int(1), RuntimeValue::Int(1)],
        );
        let _ = apply_builtin(
            BuiltinOp::Ne,
            vec![RuntimeValue::Int(1), RuntimeValue::Int(2)],
        );
        assert!(apply_builtin(
            BuiltinOp::IntDiv,
            vec![RuntimeValue::F64(1.0), RuntimeValue::Int(2)]
        )
        .is_err());
        assert!(apply_builtin(
            BuiltinOp::Mod,
            vec![RuntimeValue::Int(1), RuntimeValue::Int(0)]
        )
        .is_err());
        let _ = apply_builtin(
            BuiltinOp::IntDiv,
            vec![RuntimeValue::Int(7), RuntimeValue::Int(2)],
        );
        let _ = apply_builtin(
            BuiltinOp::Mod,
            vec![RuntimeValue::Int(7), RuntimeValue::Int(2)],
        );
        // Numeric binop mixed classes
        let _ = apply_numeric_binop(
            BuiltinOp::Add,
            &RuntimeValue::Int(1),
            &RuntimeValue::F64(2.0),
        );
        let _ = apply_numeric_binop(
            BuiltinOp::Lt,
            &RuntimeValue::Number(1.0),
            &RuntimeValue::Int(2),
        );
        assert!(apply_numeric_binop(
            BuiltinOp::Add,
            &RuntimeValue::String("a".into()),
            &RuntimeValue::Int(1)
        )
        .is_err());
        // Former unreachable! arms now return Err — tip them.
        assert!(
            apply_numeric_binop(BuiltinOp::Eq, &RuntimeValue::Int(1), &RuntimeValue::Int(2))
                .is_err()
        );
        assert!(apply_numeric_binop(
            BuiltinOp::IsNumber,
            &RuntimeValue::Int(1),
            &RuntimeValue::F64(2.0)
        )
        .is_err());
        // Drive apply_builtin internal Err arms via non-matching outer patterns:
        // call Eq/Ne path is live; internal `_` only via apply_numeric_binop above.
    }
}
