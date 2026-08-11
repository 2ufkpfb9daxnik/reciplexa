//! Bidirectional type checking for Core expressions (Phase 2 §4.2 step 9).

use reciplexa_source::range::TextRange;
use reciplexa_syntax::is_wildcard_ident;

use crate::elaborate::DataEnv;
use crate::expr::{first_unreachable_arm, CoreExpr, CoreLiteral, CorePattern, CoreValue, MatchArm};
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
    /// Active non-escaping `var` binders (DD-BND-021 local-state effects).
    pub local_state: std::collections::HashSet<String>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, ty: CoreType) {
        self.vars.insert(name.into(), ty);
    }

    fn local_state_op(name: &str) -> String {
        format!("local-state/{name}")
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
        CoreExpr::Error => Ok((CoreType::Error, EffectRow::default())),
        CoreExpr::Lit(lit) => Ok((
            match lit {
                CoreLiteral::Number(_) | CoreLiteral::F64(_) => CoreType::F64,
                CoreLiteral::Int(_) => CoreType::Int,
                CoreLiteral::String(_) => CoreType::String,
                CoreLiteral::Color(_) => CoreType::Color,
                CoreLiteral::Bool(_) => CoreType::Bool,
                CoreLiteral::Unit => CoreType::Unit,
                CoreLiteral::Bytes(_) => CoreType::Bytes,
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
                "random" => {
                    if !matches!(arg_ty, CoreType::Unit | CoreType::Dynamic) {
                        return Err(CheckError::at(
                            "perform `random` takes unit (no payload)",
                            range,
                        ));
                    }
                }
                // ERR-001 Failure payload is an ordinary RPX value (any type).
                "failure" => {}
                "read-file" | "write-file" | "log" | "write-path" | "load-image" => {
                    if !matches!(arg_ty, CoreType::String | CoreType::Dynamic) {
                        return Err(CheckError::at(
                            format!("perform `{op}` arg must be string"),
                            range,
                        ));
                    }
                }
                _ => {
                    if !matches!(
                        arg_ty,
                        CoreType::String | CoreType::Dynamic | CoreType::Unit
                    ) {
                        return Err(CheckError::at("perform arg must be string or unit", range));
                    }
                }
            }
            let ty = match op.as_str() {
                "read-file" => CoreType::String,
                "random" => CoreType::Number,
                "failure" => CoreType::Never,
                _ => CoreType::Unit,
            };
            Ok((ty, arg_effs.with_op(op.clone())))
        }
        CoreExpr::Forward { resume_name } => {
            let resume_ty = env.vars.get(resume_name).cloned().ok_or_else(|| {
                CheckError::at(format!("unbound resume `{resume_name}` in forward"), range)
            })?;
            if !matches!(
                resume_ty,
                CoreType::Fun { .. } | CoreType::Var(_) | CoreType::Dynamic
            ) {
                return Err(CheckError::at(
                    "`forward` expects a resume continuation",
                    range,
                ));
            }
            Ok((CoreType::Unit, EffectRow::default()))
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
            let failure_op = op == "failure";
            match handler_params.as_slice() {
                [arg] => {
                    child.insert(
                        arg.clone(),
                        if failure_op {
                            CoreType::Dynamic
                        } else {
                            CoreType::String
                        },
                    );
                }
                [arg, resume] if !failure_op => {
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
                [_, _] if failure_op => {
                    return Err(CheckError::at(
                        "Failure handler must not bind a resume continuation (ERR-001 §5.1)",
                        range,
                    ));
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
        CoreExpr::HandlerValue {
            handler_params,
            handler_body,
            ..
        } => {
            let mut child = env.clone();
            match handler_params.as_slice() {
                [arg] => {
                    child.insert(arg.clone(), CoreType::String);
                }
                [arg, resume] => {
                    child.insert(arg.clone(), CoreType::String);
                    child.insert(
                        resume.clone(),
                        CoreType::Fun {
                            args: vec![CoreType::Var(subst.fresh_var())],
                            ret: Box::new(CoreType::Var(subst.fresh_var())),
                            effects: EffectRow::default(),
                        },
                    );
                }
                _ => {
                    return Err(CheckError::at("`handler` expects 1 or 2 parameters", range));
                }
            }
            let (_hb_ty, hb_effs) = infer_with_effects(handler_body, &child, subst, range)?;
            // Handler values are Dynamic until a dedicated Handler type lands.
            Ok((CoreType::Dynamic, hb_effs))
        }
        CoreExpr::With { handler, body } => {
            let (_h_ty, h_effs) = infer_with_effects(handler, env, subst, range)?;
            // Conservative: treat as catching an unknown op (Dynamic handler).
            // Body residual is kept; handler clause effects merge.
            let (body_ty, body_effs) = infer_with_effects(body, env, subst, range)?;
            Ok((body_ty, h_effs.merge(&body_effs)))
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
            let state_op = TypeEnv::local_state_op(name);
            let mut child = env.clone();
            child.insert(name.clone(), init_ty);
            child.local_state.insert(name.clone());
            let (b_ty, b_effs) = infer_with_effects(body, &child, subst, range)?;
            // DD-BND-021: strip scoped local-state from the var expression residual.
            Ok((b_ty, init_effs.merge(&b_effs.without_op(&state_op))))
        }
        CoreExpr::Set { name, value } => {
            let (v_ty, v_effs) = infer_with_effects(value, env, subst, range)?;
            let expected = env.vars.get(name).cloned().ok_or_else(|| {
                CheckError::at(format!("unbound variable `{name}` in set"), range)
            })?;
            unify(&v_ty, &expected, subst).map_err(|e| unify_to_check(e, range))?;
            let effs = if env.local_state.contains(name) {
                v_effs.with_op(TypeEnv::local_state_op(name))
            } else {
                v_effs
            };
            Ok((CoreType::Unit, effs))
        }
        CoreExpr::Lambda { params, body } => {
            let mut child = env.clone();
            let mut arg_tys = Vec::with_capacity(params.len());
            for param in params {
                if is_wildcard_ident(param) {
                    arg_tys.push(CoreType::Var(subst.fresh_var()));
                    continue;
                }
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
            if let CoreExpr::Var(op) = fun.as_ref() {
                if let Some(result) = try_infer_numeric_builtin(op, args, env, subst, range) {
                    return result;
                }
            }
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
            // DD-TYP-IF-001: narrow a direct immutable local under known predicates.
            let (then_env, else_env) = occurrence_envs(cond, env);
            let (then_ty, then_effs) = infer_with_effects(then_branch, &then_env, subst, range)?;
            let (else_ty, else_effs) = infer_with_effects(else_branch, &else_env, subst, range)?;
            let then_ty = subst.apply(&then_ty);
            let else_ty = subst.apply(&else_ty);
            // L4983: when branches disagree, result is a union (not a hard error).
            let result_ty = if matches!(then_ty, CoreType::Never) {
                else_ty
            } else if matches!(else_ty, CoreType::Never) {
                then_ty
            } else {
                let mut trial = subst.clone();
                if unify(&then_ty, &else_ty, &mut trial).is_ok() {
                    *subst = trial;
                    subst.apply(&then_ty)
                } else {
                    CoreType::Union(vec![then_ty, else_ty])
                }
            };
            Ok((result_ty, cond_effs.merge(&then_effs).merge(&else_effs)))
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
                CoreType::Record { fields } => {
                    let t = fields
                        .into_iter()
                        .find(|(k, _)| k == field)
                        .map(|(_, t)| t)
                        .ok_or_else(|| CheckError::at(format!("unknown field `{field}`"), range))?;
                    field_access_type(t)
                }
                CoreType::OpenRecord { fields, row } => {
                    if let Some((_, t)) = fields.into_iter().find(|(k, _)| k == field) {
                        field_access_type(t)
                    } else {
                        let field_ty = CoreType::Var(subst.fresh_var());
                        let rest = CoreType::Var(subst.fresh_var());
                        let expected = CoreType::OpenRecord {
                            fields: vec![(field.clone(), field_ty.clone())],
                            row: Box::new(rest),
                        };
                        unify(&row, &expected, subst).map_err(|e| unify_to_check(e, range))?;
                        field_access_type(subst.apply(&field_ty))
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
        CoreExpr::RecordUpdate { record, fields } => {
            let (rec_ty, mut effs) = infer_with_effects(record, env, subst, range)?;
            let rec_ty = subst.apply(&rec_ty);
            let CoreType::Record { fields: base } = &rec_ty else {
                return Err(CheckError::at(
                    format!("`record-update` requires a closed record, got {rec_ty:?}"),
                    range,
                ));
            };
            let mut out = base.clone();
            for (label, value) in fields {
                let (v_ty, v_effs) = infer_with_effects(value, env, subst, range)?;
                effs = effs.merge(&v_effs);
                let Some((_, slot)) = out.iter_mut().find(|(n, _)| n == label) else {
                    return Err(CheckError::at(
                        format!("`record-update` field `{label}` is not present"),
                        range,
                    ));
                };
                unify(&v_ty, slot, subst).map_err(|e| unify_to_check(e, range))?;
                *slot = subst.apply(&v_ty);
            }
            Ok((CoreType::Record { fields: out }, effs))
        }
        CoreExpr::RecordExtend { record, fields } => {
            let (rec_ty, mut effs) = infer_with_effects(record, env, subst, range)?;
            let rec_ty = subst.apply(&rec_ty);
            let CoreType::Record { fields: base } = &rec_ty else {
                return Err(CheckError::at(
                    format!("`record-extend` requires a closed record, got {rec_ty:?}"),
                    range,
                ));
            };
            let mut out = base.clone();
            for (label, value) in fields {
                if out.iter().any(|(n, _)| n == label) {
                    return Err(CheckError::at(
                        format!("`record-extend` field `{label}` already present"),
                        range,
                    ));
                }
                let (v_ty, v_effs) = infer_with_effects(value, env, subst, range)?;
                effs = effs.merge(&v_effs);
                out.push((label.clone(), v_ty));
            }
            Ok((CoreType::Record { fields: out }, effs))
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
            let expected_adt = arms.iter().find_map(|a| {
                let tag = a.tag()?;
                let adt = env.data.adt_for_tag(tag);
                if adt.is_empty() {
                    None
                } else {
                    Some(adt)
                }
            });
            let has_catch_all = arms.iter().any(|a| a.is_catch_all());
            if !has_catch_all {
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
            let adt_tags: Vec<&str> = expected_adt
                .as_ref()
                .map(|adt| adt.iter().map(|(t, _)| t.as_str()).collect())
                .unwrap_or_default();
            if let Some(idx) = first_unreachable_arm(arms, &adt_tags) {
                let detail = arms[idx]
                    .tag()
                    .map(|t| format!(": constructor `{t}` is already covered"))
                    .unwrap_or_default();
                return Err(CheckError::at(
                    format!("unreachable match case{detail}"),
                    range,
                ));
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

/// DAT §18.5: optional field access yields an option-shaped variant type.
fn field_access_type(field_ty: CoreType) -> CoreType {
    match field_ty {
        CoreType::OptionalField(inner) => CoreType::Variant {
            variants: vec![("none".into(), None), ("some".into(), Some(*inner))],
        },
        other => other,
    }
}

/// DD-TYP-IF-001: refine an immutable local under a recognized type predicate.
fn occurrence_envs(cond: &CoreExpr, env: &TypeEnv) -> (TypeEnv, TypeEnv) {
    let CoreExpr::App { fun, args } = cond else {
        return (env.clone(), env.clone());
    };
    let (CoreExpr::Var(pred), [CoreExpr::Var(name)]) = (fun.as_ref(), args.as_slice()) else {
        return (env.clone(), env.clone());
    };
    let Some(scr) = env.vars.get(name).cloned() else {
        return (env.clone(), env.clone());
    };
    let Some((then_ty, else_ty)) = refine_predicate(pred, &scr) else {
        return (env.clone(), env.clone());
    };
    let mut then_env = env.clone();
    let mut else_env = env.clone();
    then_env.insert(name.clone(), then_ty);
    else_env.insert(name.clone(), else_ty);
    (then_env, else_env)
}

fn refine_predicate(pred: &str, scr: &CoreType) -> Option<(CoreType, CoreType)> {
    let intersect = |t: CoreType| CoreType::Intersect(vec![scr.clone(), t]);
    let diff = |t: CoreType| CoreType::Diff(Box::new(scr.clone()), Box::new(t));
    match pred {
        "number?" => Some((intersect(CoreType::Number), diff(CoreType::Number))),
        "string?" => Some((intersect(CoreType::String), diff(CoreType::String))),
        "bool?" => Some((intersect(CoreType::Bool), diff(CoreType::Bool))),
        "is-none" => Some((
            CoreType::Intersect(vec![
                scr.clone(),
                CoreType::Variant {
                    variants: vec![("none".into(), None)],
                },
            ]),
            strip_variant_tag(scr, "none"),
        )),
        "is-some" => {
            let then_ty = match scr {
                CoreType::Variant { variants } => variants
                    .iter()
                    .find(|(t, _)| t == "some")
                    .and_then(|(_, p)| p.clone())
                    .unwrap_or_else(|| keep_variant_tag(scr, "some")),
                _ => keep_variant_tag(scr, "some"),
            };
            Some((
                then_ty,
                CoreType::Intersect(vec![
                    scr.clone(),
                    CoreType::Variant {
                        variants: vec![("none".into(), None)],
                    },
                ]),
            ))
        }
        _ => None,
    }
}

fn strip_variant_tag(scr: &CoreType, tag: &str) -> CoreType {
    match scr {
        CoreType::Variant { variants } => {
            let rest: Vec<_> = variants.iter().filter(|(t, _)| t != tag).cloned().collect();
            match rest.as_slice() {
                [(t, Some(p))] if t == "some" => p.clone(),
                [] => CoreType::Dynamic,
                _ => CoreType::Variant { variants: rest },
            }
        }
        other => other.clone(),
    }
}

fn keep_variant_tag(scr: &CoreType, tag: &str) -> CoreType {
    match scr {
        CoreType::Variant { variants } => {
            if let Some((_, payload)) = variants.iter().find(|(t, _)| t == tag) {
                if let Some(p) = payload {
                    return CoreType::Intersect(vec![scr.clone(), p.clone()]);
                }
                return CoreType::Intersect(vec![
                    scr.clone(),
                    CoreType::Variant {
                        variants: vec![(tag.into(), None)],
                    },
                ]);
            }
            CoreType::Intersect(vec![
                scr.clone(),
                CoreType::Variant {
                    variants: vec![(tag.into(), Some(CoreType::Dynamic))],
                },
            ])
        }
        other => CoreType::Intersect(vec![
            other.clone(),
            CoreType::Variant {
                variants: vec![(tag.into(), Some(CoreType::Dynamic))],
            },
        ]),
    }
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
        CorePattern::Wildcard | CorePattern::Lit(_) => {}
        CorePattern::Bind(name) => {
            env.insert(name.clone(), scr_ty.clone());
        }
        CorePattern::Tuple(elems) => {
            if let CoreType::Record { fields } = scr_ty {
                for (i, ep) in elems.iter().enumerate() {
                    let key = i.to_string();
                    if let Some((_, ty)) = fields.iter().find(|(k, _)| *k == key) {
                        bind_pattern(ep, ty, env);
                    } else {
                        bind_pattern(ep, &CoreType::Dynamic, env);
                    }
                }
            } else {
                for ep in elems {
                    bind_pattern(ep, &CoreType::Dynamic, env);
                }
            }
        }
        CorePattern::Record { fields: pats } => match scr_ty {
            CoreType::Record { fields } | CoreType::OpenRecord { fields, .. } => {
                for (label, ep) in pats {
                    if let Some((_, ty)) = fields.iter().find(|(k, _)| k == label) {
                        // DAT §18.5: optional fields are not pattern-decomposed.
                        if matches!(ty, CoreType::OptionalField(_)) {
                            // Bind Dynamic so typechecking can still proceed; the
                            // elaborator rejects explicit `(optional …)` patterns.
                            // Required-only patterns against optional fields stay Dynamic.
                            bind_pattern(ep, &CoreType::Dynamic, env);
                        } else {
                            bind_pattern(ep, ty, env);
                        }
                    } else {
                        // §18.6 unknown open-row fields: bind Dynamic interim.
                        bind_pattern(ep, &CoreType::Dynamic, env);
                    }
                }
            }
            _ => {
                for (_, ep) in pats {
                    bind_pattern(ep, &CoreType::Dynamic, env);
                }
            }
        },
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

const NUMERIC_BINOPS: &[&str] = &["+", "-", "*", "/", "<", ">", "<=", ">="];

fn try_infer_numeric_builtin(
    op: &str,
    args: &[CoreExpr],
    env: &TypeEnv,
    subst: &mut Subst,
    range: TextRange,
) -> Option<Result<(CoreType, EffectRow), CheckError>> {
    if !NUMERIC_BINOPS.contains(&op) || args.len() != 2 {
        return None;
    }
    let mut effs = EffectRow::default();
    let (a_ty, a_eff) = infer_with_effects(&args[0], env, subst, range).ok()?;
    let (b_ty, b_eff) = infer_with_effects(&args[1], env, subst, range).ok()?;
    effs = effs.merge(&a_eff).merge(&b_eff);
    let a_ty = subst.apply(&a_ty);
    let b_ty = subst.apply(&b_ty);
    let a_class = match a_ty.numeric_class() {
        Some(c) => c,
        None if matches!(a_ty, CoreType::Number | CoreType::Dynamic) => {
            return Some(Err(CheckError::at(
                format!(
                    "numeric operand for `{op}` must be `int` or `f64`, not ambiguous `number`"
                ),
                range,
            )));
        }
        None => {
            return Some(Err(CheckError::at(
                format!("`{op}` expects numeric operands"),
                range,
            )));
        }
    };
    let b_class = match b_ty.numeric_class() {
        Some(c) => c,
        None if matches!(b_ty, CoreType::Number | CoreType::Dynamic) => {
            return Some(Err(CheckError::at(
                format!(
                    "numeric operand for `{op}` must be `int` or `f64`, not ambiguous `number`"
                ),
                range,
            )));
        }
        None => {
            return Some(Err(CheckError::at(
                format!("`{op}` expects numeric operands"),
                range,
            )));
        }
    };
    let ret = CoreType::numeric_binop_result(op, a_class, b_class);
    Some(Ok((ret, effs)))
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
    // KER-001 / DD-TYP-NUM-002: numeric builtins use promotion at application sites.
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
    for op in ["+", "-", "*", "/"] {
        env.insert(
            op,
            CoreType::Fun {
                args: vec![CoreType::Number, CoreType::Number],
                ret: Box::new(CoreType::Number),
                effects: EffectRow::default(),
            },
        );
    }
    env.insert("<", cmp2.clone());
    env.insert(">", cmp2.clone());
    env.insert("<=", cmp2.clone());
    env.insert(">=", cmp2);
    env.insert("=", eq2.clone());
    env.insert("!=", eq2);
    for c in ["black", "white", "red", "green", "blue"] {
        env.insert(c, CoreType::Color);
    }
    let pred1 = CoreType::Fun {
        args: vec![CoreType::Dynamic],
        ret: Box::new(CoreType::Bool),
        effects: EffectRow::default(),
    };
    env.insert("number?", pred1.clone());
    env.insert("string?", pred1.clone());
    env.insert("bool?", pred1.clone());
    env.insert("is-none", pred1.clone());
    env.insert("is-some", pred1);
    env.insert("newline", CoreType::String);
    env.insert("tab", CoreType::String);
    env.insert("carriage-return", CoreType::String);
    env.insert("nul", CoreType::String);
    env.insert(
        "unicode",
        CoreType::Fun {
            args: vec![CoreType::Int],
            ret: Box::new(CoreType::String),
            effects: EffectRow::default(),
        },
    );
    let ty = infer_expr(&expr, &env, &mut subst, TextRange::EMPTY)?;
    Ok(subst.apply(&ty))
}
