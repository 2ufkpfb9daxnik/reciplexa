//! Bidirectional type checking for Core expressions (Phase 2 §4.2 step 9).

use reciplexa_source::range::TextRange;

use crate::elaborate::DataEnv;
use crate::expr::{CoreExpr, CoreLiteral, CorePattern, CoreValue, MatchArm};
use crate::ty::{CoreType, EffectRow};
use crate::unify::{unify, Subst, UnifyError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckError {
    pub message: String,
    pub range: TextRange,
}

impl CheckError {
    fn at(msg: impl Into<String>, range: TextRange) -> Self {
        Self {
            message: msg.into(),
            range,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    pub vars: std::collections::HashMap<String, CoreType>,
    pub data: DataEnv,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, ty: CoreType) {
        self.vars.insert(name.into(), ty);
    }
}

/// Infer the type of a core expression.
pub fn infer_expr(
    expr: &CoreExpr,
    env: &TypeEnv,
    subst: &mut Subst,
    range: TextRange,
) -> Result<CoreType, CheckError> {
    Ok(infer_with_effects(expr, env, subst, range)?.0)
}

/// Infer type and latent/effect residual of a core expression.
///
/// - [`CoreExpr::Perform`] adds `op` to the residual effect row.
/// - [`CoreExpr::Handle`] removes `op` from the body's residual.
/// - [`CoreExpr::Lambda`] suspends body residuals onto [`CoreType::Fun::effects`].
pub fn infer_with_effects(
    expr: &CoreExpr,
    env: &TypeEnv,
    subst: &mut Subst,
    range: TextRange,
) -> Result<(CoreType, EffectRow), CheckError> {
    match expr {
        CoreExpr::Lit(lit) => Ok((
            match lit {
                CoreLiteral::Number(_) => CoreType::Number,
                CoreLiteral::String(_) => CoreType::String,
                CoreLiteral::Color(_) => CoreType::Color,
                CoreLiteral::Bool(_) => CoreType::Bool,
                CoreLiteral::Unit => CoreType::Unit,
            },
            EffectRow::default(),
        )),
        CoreExpr::Var(name) => {
            let ty = env
                .vars
                .get(name)
                .cloned()
                .ok_or_else(|| CheckError::at(format!("unbound variable `{name}`"), range))?;
            Ok((ty, EffectRow::default()))
        }
        CoreExpr::Perform { op, arg } => {
            let (arg_ty, arg_effs) = infer_with_effects(arg, env, subst, range)?;
            match op.as_str() {
                "read-file" | "write-file" | "log" => {
                    if !matches!(arg_ty, CoreType::String | CoreType::Dynamic) {
                        return Err(CheckError::at(
                            format!("perform `{op}` arg must be string"),
                            range,
                        ));
                    }
                }
                _ => {
                    if !matches!(arg_ty, CoreType::String | CoreType::Dynamic) {
                        return Err(CheckError::at("perform arg must be string", range));
                    }
                }
            }
            let ty = if op == "read-file" {
                CoreType::String
            } else {
                CoreType::Unit
            };
            Ok((ty, arg_effs.with_op(op.clone())))
        }
        CoreExpr::Handle {
            op,
            handler_params,
            handler_body,
            body,
        } => {
            let (_body_ty, body_effs) = infer_with_effects(body, env, subst, range)?;
            let residual = body_effs.without_op(op);
            let mut child = env.clone();
            match handler_params.as_slice() {
                [arg] => {
                    child.insert(arg.clone(), CoreType::String);
                }
                [arg, resume] => {
                    child.insert(arg.clone(), CoreType::String);
                    // Resume continues the handled body: residual effects remain.
                    child.insert(
                        resume.clone(),
                        CoreType::Fun {
                            args: vec![CoreType::Var(subst.fresh_var())],
                            ret: Box::new(CoreType::Var(subst.fresh_var())),
                            effects: residual.clone(),
                        },
                    );
                }
                _ => {
                    return Err(CheckError::at(
                        "`handle` handler expects 1 or 2 parameters",
                        range,
                    ));
                }
            }
            let (handler_ty, handler_effs) =
                infer_with_effects(handler_body, &child, subst, range)?;
            Ok((handler_ty, residual.merge(&handler_effs)))
        }
        CoreExpr::Seq(items) => {
            let mut last = CoreType::Unit;
            let mut effs = EffectRow::default();
            for item in items {
                let (ty, e) = infer_with_effects(item, env, subst, range)?;
                last = ty;
                effs = effs.merge(&e);
            }
            Ok((last, effs))
        }
        CoreExpr::Let { name, value, body } => {
            let (v_ty, v_effs) = infer_with_effects(value, env, subst, range)?;
            let mut child = env.clone();
            child.insert(name.clone(), v_ty);
            let (b_ty, b_effs) = infer_with_effects(body, &child, subst, range)?;
            Ok((b_ty, v_effs.merge(&b_effs)))
        }
        CoreExpr::LetRec { bindings, body } => {
            let mut child = env.clone();
            for (name, _) in bindings {
                let f_ty = CoreType::Fun {
                    args: vec![CoreType::Var(subst.fresh_var())],
                    ret: Box::new(CoreType::Var(subst.fresh_var())),
                    effects: EffectRow::default(),
                };
                child.insert(name.clone(), f_ty);
            }
            let mut bind_effs = EffectRow::default();
            for (name, rhs) in bindings {
                let (rhs_ty, rhs_effs) = infer_with_effects(rhs, &child, subst, range)?;
                bind_effs = bind_effs.merge(&rhs_effs);
                if let Some(expected) = child.vars.get(name).cloned() {
                    // Allow effect rows from the concrete lambda to refine the stub.
                    unify_fun_flexible(&rhs_ty, &expected, subst)
                        .map_err(|e| unify_to_check(e, range))?;
                    child.insert(name.clone(), subst.apply(&rhs_ty));
                }
            }
            let (b_ty, b_effs) = infer_with_effects(body, &child, subst, range)?;
            Ok((b_ty, bind_effs.merge(&b_effs)))
        }
        CoreExpr::LocalVar { name, init, body } => {
            let (init_ty, init_effs) = infer_with_effects(init, env, subst, range)?;
            let mut child = env.clone();
            child.insert(name.clone(), init_ty);
            let (b_ty, b_effs) = infer_with_effects(body, &child, subst, range)?;
            Ok((b_ty, init_effs.merge(&b_effs)))
        }
        CoreExpr::Set { name, value } => {
            let (v_ty, v_effs) = infer_with_effects(value, env, subst, range)?;
            let expected = env.vars.get(name).cloned().ok_or_else(|| {
                CheckError::at(format!("unbound variable `{name}` in set"), range)
            })?;
            unify(&v_ty, &expected, subst).map_err(|e| unify_to_check(e, range))?;
            Ok((CoreType::Unit, v_effs))
        }
        CoreExpr::Lambda { params, body } => {
            let mut child = env.clone();
            let mut arg_tys = Vec::with_capacity(params.len());
            for param in params {
                let p_ty = CoreType::Var(subst.fresh_var());
                child.insert(param.clone(), p_ty.clone());
                arg_tys.push(p_ty);
            }
            let (ret, body_effs) = infer_with_effects(body, &child, subst, range)?;
            Ok((
                CoreType::Fun {
                    args: arg_tys,
                    ret: Box::new(ret),
                    effects: body_effs,
                },
                EffectRow::default(),
            ))
        }
        CoreExpr::App { fun, args } => {
            let (fun_ty, fun_effs) = infer_with_effects(fun, env, subst, range)?;
            let mut arg_tys = Vec::with_capacity(args.len());
            let mut arg_effs = EffectRow::default();
            for arg in args {
                let (t, e) = infer_with_effects(arg, env, subst, range)?;
                arg_tys.push(t);
                arg_effs = arg_effs.merge(&e);
            }
            let ret_var = CoreType::Var(subst.fresh_var());
            let fun_ty = subst.apply(&fun_ty);
            let call_effs = match &fun_ty {
                CoreType::Fun { effects, .. } => effects.clone(),
                CoreType::Var(_) => EffectRow::default(),
                _ => EffectRow::default(),
            };
            let expected = CoreType::Fun {
                args: arg_tys,
                ret: Box::new(ret_var.clone()),
                effects: call_effs.clone(),
            };
            unify_fun_flexible(&fun_ty, &expected, subst).map_err(|e| unify_to_check(e, range))?;
            let fun_ty = subst.apply(&fun_ty);
            let latent = match &fun_ty {
                CoreType::Fun { effects, .. } => effects.clone(),
                _ => call_effs,
            };
            Ok((
                subst.apply(&ret_var),
                fun_effs.merge(&arg_effs).merge(&latent),
            ))
        }
        CoreExpr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let (cond_ty, cond_effs) = infer_with_effects(cond, env, subst, range)?;
            unify(&cond_ty, &CoreType::Bool, subst).map_err(|e| unify_to_check(e, range))?;
            let (then_ty, then_effs) = infer_with_effects(then_branch, env, subst, range)?;
            let (else_ty, else_effs) = infer_with_effects(else_branch, env, subst, range)?;
            unify(&then_ty, &else_ty, subst).map_err(|e| unify_to_check(e, range))?;
            Ok((
                subst.apply(&then_ty),
                cond_effs.merge(&then_effs).merge(&else_effs),
            ))
        }
        CoreExpr::Record { fields } => {
            let mut typed = Vec::new();
            let mut effs = EffectRow::default();
            for (k, v) in fields {
                let (t, e) = infer_with_effects(v, env, subst, range)?;
                typed.push((k.clone(), t));
                effs = effs.merge(&e);
            }
            Ok((CoreType::Record { fields: typed }, effs))
        }
        CoreExpr::RecordGet { record, field } => {
            let (rec_ty, rec_effs) = infer_with_effects(record, env, subst, range)?;
            let rec_ty = subst.apply(&rec_ty);
            let ty = match rec_ty {
                CoreType::Record { fields } => fields
                    .into_iter()
                    .find(|(k, _)| k == field)
                    .map(|(_, t)| t)
                    .ok_or_else(|| CheckError::at(format!("unknown field `{field}`"), range))?,
                CoreType::OpenRecord { fields, row } => {
                    if let Some((_, t)) = fields.into_iter().find(|(k, _)| k == field) {
                        t
                    } else {
                        let field_ty = CoreType::Var(subst.fresh_var());
                        let rest = CoreType::Var(subst.fresh_var());
                        let expected = CoreType::OpenRecord {
                            fields: vec![(field.clone(), field_ty.clone())],
                            row: Box::new(rest),
                        };
                        unify(&row, &expected, subst).map_err(|e| unify_to_check(e, range))?;
                        subst.apply(&field_ty)
                    }
                }
                other => {
                    return Err(CheckError::at(
                        format!("expected record, got {other:?}"),
                        range,
                    ));
                }
            };
            Ok((ty, rec_effs))
        }
        CoreExpr::Variant { tag, payload } => {
            let (payload_ty, payload_effs) = if let Some(p) = payload {
                let (t, e) = infer_with_effects(p, env, subst, range)?;
                (Some(t), e)
            } else {
                (None, EffectRow::default())
            };
            let adt = env.data.adt_for_tag(tag);
            let variants = if adt.is_empty() {
                vec![(tag.clone(), payload_ty)]
            } else {
                adt.iter()
                    .map(|(t, arity)| {
                        if t == tag {
                            (t.clone(), payload_ty.clone())
                        } else if *arity == 0 {
                            (t.clone(), None)
                        } else {
                            (t.clone(), Some(CoreType::Dynamic))
                        }
                    })
                    .collect()
            };
            Ok((CoreType::Variant { variants }, payload_effs))
        }
        CoreExpr::Match { scrutinee, arms } => {
            let (scr_ty, scr_effs) = infer_with_effects(scrutinee, env, subst, range)?;
            let scr_ty = subst.apply(&scr_ty);
            let has_catch_all = arms.iter().any(|a| a.is_catch_all());
            if !has_catch_all {
                let expected_adt = arms.iter().find_map(|a| {
                    let tag = a.tag()?;
                    let adt = env.data.adt_for_tag(tag);
                    if adt.is_empty() {
                        None
                    } else {
                        Some(adt)
                    }
                });
                if let Some(adt) = &expected_adt {
                    let covered: std::collections::HashSet<&str> =
                        arms.iter().filter_map(|a| a.tag()).collect();
                    let missing: Vec<&str> = adt
                        .iter()
                        .map(|(t, _)| t.as_str())
                        .filter(|t| !covered.contains(t))
                        .collect();
                    if !missing.is_empty() {
                        return Err(CheckError::at(
                            format!(
                                "non-exhaustive match: missing constructor(s) {}",
                                missing.join(", ")
                            ),
                            range,
                        ));
                    }
                }
            }
            let ret_var = CoreType::Var(subst.fresh_var());
            let mut arm_effs = EffectRow::default();
            for arm in arms {
                let e = check_arm(arm, &scr_ty, &ret_var, env, subst, range)?;
                arm_effs = arm_effs.merge(&e);
            }
            Ok((subst.apply(&ret_var), scr_effs.merge(&arm_effs)))
        }
    }
}

fn unify_fun_flexible(
    found: &CoreType,
    expected: &CoreType,
    subst: &mut Subst,
) -> Result<(), UnifyError> {
    let found = subst.apply(found);
    let expected = subst.apply(expected);
    match (&found, &expected) {
        (
            CoreType::Fun {
                args: a_args,
                ret: a_ret,
                effects: a_eff,
            },
            CoreType::Fun {
                args: b_args,
                ret: b_ret,
                ..
            },
        ) => {
            if a_args.len() != b_args.len() {
                return Err(UnifyError::Mismatch {
                    expected: expected.clone(),
                    found: found.clone(),
                });
            }
            for (x, y) in a_args.iter().zip(b_args.iter()) {
                unify(x, y, subst)?;
            }
            unify(a_ret, b_ret, subst)?;
            // Prefer concrete effect row from `found` when expected was a stub.
            let _ = a_eff;
            Ok(())
        }
        _ => unify(&found, &expected, subst),
    }
}

fn unify_to_check(e: UnifyError, range: TextRange) -> CheckError {
    CheckError::at(format!("{e:?}"), range)
}

fn check_arm(
    arm: &MatchArm,
    scr_ty: &CoreType,
    ret_ty: &CoreType,
    env: &TypeEnv,
    subst: &mut Subst,
    range: TextRange,
) -> Result<EffectRow, CheckError> {
    let mut child = env.clone();
    bind_pattern(&arm.pattern, scr_ty, &mut child);
    let (body_ty, body_effs) = infer_with_effects(&arm.body, &child, subst, range)?;
    unify(&body_ty, ret_ty, subst).map_err(|e| unify_to_check(e, range))?;
    Ok(body_effs)
}

fn bind_pattern(pat: &CorePattern, scr_ty: &CoreType, env: &mut TypeEnv) {
    match pat {
        CorePattern::Wildcard => {}
        CorePattern::Bind(name) => {
            env.insert(name.clone(), scr_ty.clone());
        }
        CorePattern::Variant { tag, payload } => {
            if let CoreType::Variant { variants } = scr_ty {
                if let Some((_, p_ty)) = variants.iter().find(|(t, _)| t == tag) {
                    if let (Some(inner), Some(pty)) = (payload, p_ty) {
                        bind_pattern(inner, pty, env);
                    } else if let (Some(inner), None) = (payload, p_ty) {
                        // Unknown payload type — bind as Dynamic when binder.
                        if let CorePattern::Bind(name) = inner.as_ref() {
                            env.insert(name.clone(), CoreType::Dynamic);
                        } else {
                            bind_pattern(inner, &CoreType::Dynamic, env);
                        }
                    }
                } else if let Some(inner) = payload {
                    // Tag not in scrutinee type; still bind Dynamic for nested binders.
                    bind_pattern(inner, &CoreType::Dynamic, env);
                }
            } else if let Some(inner) = payload {
                bind_pattern(inner, &CoreType::Dynamic, env);
            }
        }
    }
}

/// Type-check and return a typed core value.
pub fn typecheck_value(
    expr: CoreExpr,
    env: &TypeEnv,
    range: TextRange,
) -> Result<CoreValue, CheckError> {
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, env, &mut subst, range)?;
    Ok(CoreValue { ty, expr })
}

/// Language-kernel typecheck: expand → elaborate → Core `infer_expr`.
///
/// Quarantined from the document/page surface checker in `reciplexa-types`.
pub fn typecheck_language_source(src: &str) -> Result<CoreType, CheckError> {
    let expanded = reciplexa_macro::expand_language(src).map_err(|e| CheckError {
        message: e.message,
        range: TextRange::EMPTY,
    })?;
    let (expr, data) =
        crate::elaborate::elaborate_with_data(&expanded).map_err(|e| CheckError {
            message: e.message,
            range: e.range,
        })?;
    let mut subst = Subst::new();
    let mut env = TypeEnv::new();
    env.data = data;
    // KER-001 primitive types
    let num2 = CoreType::Fun {
        args: vec![CoreType::Number, CoreType::Number],
        ret: Box::new(CoreType::Number),
        effects: EffectRow::default(),
    };
    let cmp2 = CoreType::Fun {
        args: vec![CoreType::Number, CoreType::Number],
        ret: Box::new(CoreType::Bool),
        effects: EffectRow::default(),
    };
    let eq2 = CoreType::Fun {
        args: vec![CoreType::Dynamic, CoreType::Dynamic],
        ret: Box::new(CoreType::Bool),
        effects: EffectRow::default(),
    };
    env.insert("+", num2.clone());
    env.insert("-", num2.clone());
    env.insert("*", num2);
    env.insert("<", cmp2);
    env.insert("=", eq2);
    let ty = infer_expr(&expr, &env, &mut subst, TextRange::EMPTY)?;
    Ok(subst.apply(&ty))
}
