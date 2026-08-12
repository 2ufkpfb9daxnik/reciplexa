//! Bidirectional type checking for Core expressions (Phase 2 §4.2 step 9).

#![allow(clippy::result_large_err)]

use std::collections::{HashMap, HashSet};

use reciplexa_source::range::TextRange;
use reciplexa_syntax::is_wildcard_ident;

use crate::elaborate::DataEnv;
use crate::expr::{first_unreachable_arm, CoreExpr, CoreLiteral, CorePattern, CoreValue, MatchArm};
use crate::ty::{CoreType, EffectRow, NumericClass, TypeVarId};
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
            // TYP forall instantiate: prenex ∀ is opened at each use site.
            let ty = instantiate_forall(&ty, subst);
            // DD-BND-018/021: reading a local `var` uses get<s>.
            let effs = if env.local_state.contains(name) {
                EffectRow::default().with_op(TypeEnv::local_state_op(name))
            } else {
                EffectRow::default()
            };
            Ok((ty, effs))
        }
        CoreExpr::Perform { op, arg } => {
            let (arg_ty, arg_effs) = infer_with_effects(arg, env, subst, range)?;
            match op.as_str() {
                "random" => {
                    if !matches!(arg_ty, CoreType::Unit | CoreType::Dynamic(_)) {
                        return Err(CheckError::at(
                            "perform `random` takes unit (no payload)",
                            range,
                        ));
                    }
                }
                // ERR-001 Failure payload is an ordinary RPX value (any type).
                "failure" => {}
                "read-file" | "write-file" | "log" | "write-path" | "load-image" => {
                    if !matches!(arg_ty, CoreType::String | CoreType::Dynamic(_)) {
                        return Err(CheckError::at(
                            format!("perform `{op}` arg must be string"),
                            range,
                        ));
                    }
                }
                _ => {
                    if !matches!(
                        arg_ty,
                        CoreType::String | CoreType::Dynamic(_) | CoreType::Unit
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
                CoreType::Fun { .. } | CoreType::Var(_) | CoreType::Dynamic(_)
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
                            CoreType::dyn_any()
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
            Ok((CoreType::dyn_any(), hb_effs))
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
            let (v_ty, v_effs) =
                infer_binding_init(name, value, env, subst, range, /*generalize*/ true)?;
            let mut child = env.clone();
            child.insert(name.clone(), v_ty.clone());
            // When the body is just the binder (top-level `val` sugar), report the
            // principle type of the binding rather than a use-site instantiation.
            if let CoreExpr::Var(n) = body.as_ref() {
                if n == name {
                    return Ok((v_ty, v_effs));
                }
            }
            let (b_ty, b_effs) = infer_with_effects(body, &child, subst, range)?;
            Ok((b_ty, v_effs.merge(&b_effs)))
        }
        CoreExpr::LetRec { bindings, body } => {
            let mut child = env.clone();
            for (name, _) in bindings {
                // Prefer an explicit `(type name …)` stub when present (DD-BND-007).
                let f_ty = if let Some(ann) = env.data.type_aliases.get(name).cloned() {
                    instantiate_forall(&ann, subst)
                } else {
                    CoreType::Fun {
                        args: vec![CoreType::Var(subst.fresh_var())],
                        ret: Box::new(CoreType::Var(subst.fresh_var())),
                        effects: EffectRow::default(),
                    }
                };
                child.insert(name.clone(), f_ty);
            }
            let mut bind_effs = EffectRow::default();
            for (name, rhs) in bindings {
                if !matches!(rhs, CoreExpr::Lambda { .. }) {
                    return Err(CheckError::at(
                        "letrecの値binding initializerはfnでなければならない",
                        range,
                    ));
                }
                let expected = child.vars.get(name).cloned();
                let (rhs_ty, rhs_effs) =
                    infer_letrec_rhs(rhs, expected.as_ref(), &child, subst, range)?;
                bind_effs = bind_effs.merge(&rhs_effs);
                if let Some(expected) = expected {
                    // Allow effect rows from the concrete lambda to refine the stub.
                    unify_fun_flexible(&rhs_ty, &expected, subst)
                        .map_err(|e| unify_to_check(e, range))?;
                    // Keep annotated (possibly ∀) principle type when present.
                    if let Some(ann) = env.data.type_aliases.get(name).cloned() {
                        child.insert(name.clone(), ann);
                    } else {
                        child.insert(name.clone(), subst.apply(&rhs_ty));
                    }
                }
            }
            let (b_ty, b_effs) = infer_with_effects(body, &child, subst, range)?;
            Ok((b_ty, bind_effs.merge(&b_effs)))
        }
        CoreExpr::LocalVar { name, init, body } => {
            // DAT §7.4 / DD-BND-025: value restriction — do not generalize `var`.
            let (init_ty, init_effs) =
                infer_binding_init(name, init, env, subst, range, /*generalize*/ false)?;
            let state_op = TypeEnv::local_state_op(name);
            let mut child = env.clone();
            child.insert(name.clone(), init_ty);
            child.local_state.insert(name.clone());
            let (b_ty, b_effs) = infer_with_effects(body, &child, subst, range)?;
            let b_ty = subst.apply(&b_ty);
            // DD-BND-019: local-state identity must not escape in the result type.
            if type_mentions_effect(&b_ty, &state_op) {
                return Err(CheckError::at(
                    format!("local state `{name}` escapes its declaring scope"),
                    range,
                ));
            }
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
            let fun_ty = instantiate_forall(&subst.apply(&fun_ty), subst);
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
            if let Some(result) =
                try_infer_parameterized_ctor(tag, payload_ty.as_ref(), env, subst, range)
            {
                let ty = result?;
                return Ok((ty, payload_effs));
            }
            let adt = env.data.adt_for_tag(tag);
            let variants = if adt.is_empty() {
                vec![(tag.clone(), payload_ty)]
            } else {
                adt.iter()
                    .map(|(t, arity)| {
                        if t == tag {
                            (t.clone(), payload_ty.clone())
                        } else if let Some(schemas) = env.data.ctor_payloads.get(t) {
                            (t.clone(), schema_payload_type(schemas))
                        } else if *arity == 0 {
                            (t.clone(), None)
                        } else {
                            (t.clone(), Some(CoreType::dyn_any()))
                        }
                    })
                    .collect()
            };
            Ok((CoreType::Variant { variants }, payload_effs))
        }
        CoreExpr::Match { scrutinee, arms } => {
            let (scr_ty, scr_effs) = infer_with_effects(scrutinee, env, subst, range)?;
            let scr_ty = expand_type_app(&subst.apply(&scr_ty), &env.data);
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
        CoreExpr::Cast {
            expr,
            evidence: _,
            target,
            ..
        } => {
            let (src, effs) = infer_with_effects(expr, env, subst, range)?;
            let src = subst.apply(&src);
            crate::cast::plan_cast_evidence(&src, target)
                .ok_or_else(|| CheckError::at("cast is statically impossible", range))?;
            let bound = src.as_dyn_bound().cloned().unwrap_or(src);
            let success = crate::cast::cast_success_type(&bound, target);
            Ok((success, effs))
        }
        CoreExpr::TryCast { expr, target, .. } => {
            let (src, effs) = infer_with_effects(expr, env, subst, range)?;
            let src = subst.apply(&src);
            crate::cast::plan_cast_evidence(&src, target)
                .ok_or_else(|| CheckError::at("`try-cast` is statically impossible", range))?;
            let bound = src.as_dyn_bound().cloned().unwrap_or(src);
            let success = crate::cast::cast_success_type(&bound, target);
            Ok((
                CoreType::App {
                    ctor: "option".into(),
                    args: vec![success],
                },
                effs,
            ))
        }
        CoreExpr::CheckCast { expr, target, .. } => {
            let (src, effs) = infer_with_effects(expr, env, subst, range)?;
            let src = subst.apply(&src);
            crate::cast::plan_cast_evidence(&src, target)
                .ok_or_else(|| CheckError::at("`check-cast` is statically impossible", range))?;
            let bound = src.as_dyn_bound().cloned().unwrap_or(src);
            let success = crate::cast::cast_success_type(&bound, target);
            Ok((
                CoreType::App {
                    ctor: "result".into(),
                    args: vec![success, CoreType::String],
                },
                effs,
            ))
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

/// Infer a letrec RHS, seeding lambda params from the recursion stub when known.
fn infer_letrec_rhs(
    rhs: &CoreExpr,
    expected: Option<&CoreType>,
    env: &TypeEnv,
    subst: &mut Subst,
    range: TextRange,
) -> Result<(CoreType, EffectRow), CheckError> {
    let CoreExpr::Lambda { params, body } = rhs else {
        // Callers gate on Lambda; retain Err for misuse / cfg(test) matrices.
        return Err(CheckError::at(
            "internal: letrec/annotated rhs must be a lambda",
            range,
        ));
    };
    let expected = expected.map(|t| subst.apply(t));
    if let Some(CoreType::Fun {
        args: exp_args,
        ret: exp_ret,
        effects: exp_effects,
    }) = expected
    {
        if exp_args.len() == params.len() {
            let mut child = env.clone();
            let mut arg_tys = Vec::with_capacity(params.len());
            for (param, exp) in params.iter().zip(exp_args.iter()) {
                let p_ty = exp.clone();
                if !is_wildcard_ident(param) {
                    child.insert(param.clone(), p_ty.clone());
                }
                arg_tys.push(p_ty);
            }
            let (ret, body_effs) = infer_with_effects(body, &child, subst, range)?;
            unify(&ret, &exp_ret, subst).map_err(|e| unify_to_check(e, range))?;
            // Residual effects of the lambda body should be consistent with the stub.
            let _ = exp_effects;
            return Ok((
                CoreType::Fun {
                    args: arg_tys,
                    ret: Box::new(subst.apply(&ret)),
                    effects: body_effs,
                },
                EffectRow::default(),
            ));
        }
    }
    infer_with_effects(rhs, env, subst, range)
}

/// BIDI-002/003: infer a binding initializer, optionally checking against a same-named type annotation.
fn infer_binding_init(
    name: &str,
    init: &CoreExpr,
    env: &TypeEnv,
    subst: &mut Subst,
    range: TextRange,
    generalize: bool,
) -> Result<(CoreType, EffectRow), CheckError> {
    if let Some(ann) = env.data.type_aliases.get(name).cloned() {
        // DD-BND / value restriction: explicit ∀ cannot rescue an expansive initializer.
        if matches!(ann, CoreType::Forall { .. }) && is_expansive(init) {
            return Err(CheckError::at(
                "expansive initializerを明示注釈で多相一般化できない",
                range,
            ));
        }
        let expected = instantiate_forall(&ann, subst);
        let expected = expand_type_app(&expected, &env.data);
        // Prefer checking lambdas against the annotation so param types seed numerics.
        let (inferred, effs) = if matches!(init, CoreExpr::Lambda { .. }) {
            infer_letrec_rhs(init, Some(&expected), env, subst, range)?
        } else {
            infer_with_effects(init, env, subst, range)?
        };
        let found = expand_type_app(&subst.apply(&inferred), &env.data);
        unify(&found, &expected, subst).map_err(|e| unify_to_check(e, range))?;
        // Bind the annotated (possibly ∀) type so uses re-instantiate.
        return Ok((ann, effs));
    }
    let (inferred, effs) = infer_with_effects(init, env, subst, range)?;
    let inferred = subst.apply(&inferred);
    if generalize {
        Ok((generalize_type(inferred, env, subst), effs))
    } else {
        // Value restriction: undetermined parameters need an annotation.
        if has_free_unification_vars(&inferred, env, subst) {
            return Err(CheckError::at(
                "cannot infer ungeneralized error type; add a type annotation",
                range,
            ));
        }
        Ok((inferred, effs))
    }
}

/// Syntactic expansiveness for the value restriction (app / effect / allocation).
fn is_expansive(expr: &CoreExpr) -> bool {
    match expr {
        CoreExpr::Lit(_) | CoreExpr::Var(_) | CoreExpr::Lambda { .. } | CoreExpr::Error => false,
        CoreExpr::HandlerValue { .. } => false,
        CoreExpr::App { .. }
        | CoreExpr::Perform { .. }
        | CoreExpr::Forward { .. }
        | CoreExpr::Set { .. } => true,
        CoreExpr::Handle {
            handler_body, body, ..
        } => is_expansive(handler_body) || is_expansive(body),
        CoreExpr::With { handler, body } => is_expansive(handler) || is_expansive(body),
        CoreExpr::Seq(items) => items.iter().any(is_expansive),
        CoreExpr::Let { value, body, .. }
        | CoreExpr::LocalVar {
            init: value, body, ..
        } => is_expansive(value) || is_expansive(body),
        CoreExpr::LetRec { bindings, body } => {
            bindings.iter().any(|(_, v)| is_expansive(v)) || is_expansive(body)
        }
        CoreExpr::If {
            cond,
            then_branch,
            else_branch,
        } => is_expansive(cond) || is_expansive(then_branch) || is_expansive(else_branch),
        CoreExpr::Record { fields } => fields.iter().any(|(_, v)| is_expansive(v)),
        CoreExpr::RecordUpdate { record, fields } | CoreExpr::RecordExtend { record, fields } => {
            is_expansive(record) || fields.iter().any(|(_, v)| is_expansive(v))
        }
        CoreExpr::RecordGet { record, .. } => is_expansive(record),
        CoreExpr::Variant { payload, .. } => payload.as_ref().is_some_and(|p| is_expansive(p)),
        CoreExpr::Match { scrutinee, arms } => {
            is_expansive(scrutinee) || arms.iter().any(|a| is_expansive(&a.body))
        }
        CoreExpr::Cast { expr, .. }
        | CoreExpr::TryCast { expr, .. }
        | CoreExpr::CheckCast { expr, .. } => is_expansive(expr),
    }
}

/// True when `op` appears in any effect row nested inside `ty` (escape analysis).
fn type_mentions_effect(ty: &CoreType, op: &str) -> bool {
    match ty {
        CoreType::Fun { args, ret, effects } => {
            effects.ops.iter().any(|o| o == op)
                || args.iter().any(|a| type_mentions_effect(a, op))
                || type_mentions_effect(ret, op)
        }
        CoreType::Record { fields } | CoreType::OpenRecord { fields, .. } => {
            fields.iter().any(|(_, t)| type_mentions_effect(t, op))
        }
        CoreType::Variant { variants } => variants
            .iter()
            .any(|(_, p)| p.as_ref().is_some_and(|t| type_mentions_effect(t, op))),
        CoreType::Lacks { row, .. } => type_mentions_effect(row, op),
        CoreType::Union(members) | CoreType::Intersect(members) => {
            members.iter().any(|m| type_mentions_effect(m, op))
        }
        CoreType::Not(inner)
        | CoreType::OptionalField(inner)
        | CoreType::Dynamic(inner)
        | CoreType::Forall { body: inner, .. } => type_mentions_effect(inner, op),
        CoreType::Diff(a, b) => type_mentions_effect(a, op) || type_mentions_effect(b, op),
        CoreType::App { args, .. } => args.iter().any(|a| type_mentions_effect(a, op)),
        CoreType::Var(_)
        | CoreType::Int
        | CoreType::F64
        | CoreType::Number
        | CoreType::Singleton(_)
        | CoreType::String
        | CoreType::Color
        | CoreType::Shape
        | CoreType::Unit
        | CoreType::Bool
        | CoreType::Bytes
        | CoreType::Any
        | CoreType::Name(_)
        | CoreType::Error
        | CoreType::Never => false,
    }
}

/// Open prenex `forall` binders to fresh unification variables.
fn instantiate_forall(ty: &CoreType, subst: &mut Subst) -> CoreType {
    match ty {
        CoreType::Forall { params, body } => {
            let mut map = HashMap::new();
            for (name, _kind) in params {
                map.insert(name.clone(), CoreType::Var(subst.fresh_var()));
            }
            let body = subst_type_names(body, &map);
            instantiate_forall(&body, subst)
        }
        other => other.clone(),
    }
}

/// Generalize free unification variables not free in the environment (rank-1 / prenex).
fn generalize_type(ty: CoreType, env: &TypeEnv, subst: &Subst) -> CoreType {
    let ty = subst.apply(&ty);
    let env_vars = env_free_vars(env, subst);
    // Prefer declared ADT parameter names when generalizing type applications (ADT-07).
    if let CoreType::App { ctor, args } = &ty {
        if let Some(params) = env.data.type_params.get(ctor) {
            if params.len() == args.len() {
                let mut forall_params = Vec::new();
                let mut new_args = Vec::new();
                let mut used_names = HashSet::new();
                for (pname, arg) in params.iter().zip(args.iter()) {
                    let arg = subst.apply(arg);
                    if let CoreType::Var(v) = &arg {
                        if !env_vars.contains(v) {
                            forall_params.push((pname.clone(), "type".to_string()));
                            used_names.insert(pname.clone());
                            new_args.push(CoreType::Name(pname.clone()));
                            continue;
                        }
                    }
                    new_args.push(generalize_type(arg, env, subst));
                }
                if !forall_params.is_empty() {
                    // Nested forall from args is unusual; flatten to App under outer binders.
                    let body = CoreType::App {
                        ctor: ctor.clone(),
                        args: new_args
                            .into_iter()
                            .map(|a| match a {
                                CoreType::Forall { body, .. } => *body,
                                other => other,
                            })
                            .collect(),
                    };
                    let _ = used_names;
                    return CoreType::Forall {
                        params: forall_params,
                        body: Box::new(body),
                    };
                }
            }
        }
    }
    let mut free = HashSet::new();
    collect_free_vars(&ty, &mut free);
    let mut params = Vec::new();
    let mut map = HashMap::new();
    let mut free_sorted: Vec<TypeVarId> =
        free.into_iter().filter(|v| !env_vars.contains(v)).collect();
    free_sorted.sort();
    for (i, var) in free_sorted.into_iter().enumerate() {
        let name = format!("t{i}");
        params.push((name.clone(), "type".to_string()));
        map.insert(var, CoreType::Name(name));
    }
    if params.is_empty() {
        return ty;
    }
    let body = subst_type_vars(&ty, &map);
    CoreType::Forall {
        params,
        body: Box::new(body),
    }
}

fn has_free_unification_vars(ty: &CoreType, env: &TypeEnv, subst: &Subst) -> bool {
    let ty = subst.apply(ty);
    let env_vars = env_free_vars(env, subst);
    let mut free = HashSet::new();
    collect_free_vars(&ty, &mut free);
    free.iter().any(|v| !env_vars.contains(v))
}

fn env_free_vars(env: &TypeEnv, subst: &Subst) -> HashSet<TypeVarId> {
    let mut out = HashSet::new();
    for ty in env.vars.values() {
        collect_free_vars(&subst.apply(ty), &mut out);
    }
    out
}

fn collect_free_vars(ty: &CoreType, out: &mut HashSet<TypeVarId>) {
    match ty {
        CoreType::Var(v) => {
            out.insert(*v);
        }
        CoreType::Fun { args, ret, .. } => {
            for a in args {
                collect_free_vars(a, out);
            }
            collect_free_vars(ret, out);
        }
        CoreType::Record { fields } | CoreType::OpenRecord { fields, .. } => {
            for (_, t) in fields {
                collect_free_vars(t, out);
            }
            if let CoreType::OpenRecord { row, .. } = ty {
                collect_free_vars(row, out);
            }
        }
        CoreType::Variant { variants } => {
            for (_, p) in variants {
                if let Some(t) = p {
                    collect_free_vars(t, out);
                }
            }
        }
        CoreType::App { args, .. } => {
            for a in args {
                collect_free_vars(a, out);
            }
        }
        CoreType::Forall { body, .. } => collect_free_vars(body, out),
        CoreType::Union(ms) | CoreType::Intersect(ms) => {
            for m in ms {
                collect_free_vars(m, out);
            }
        }
        CoreType::Not(inner)
        | CoreType::OptionalField(inner)
        | CoreType::Dynamic(inner)
        | CoreType::Lacks { row: inner, .. } => collect_free_vars(inner, out),
        CoreType::Diff(a, b) => {
            collect_free_vars(a, out);
            collect_free_vars(b, out);
        }
        _ => {}
    }
}

fn subst_type_vars(ty: &CoreType, map: &HashMap<TypeVarId, CoreType>) -> CoreType {
    match ty {
        CoreType::Var(v) => map.get(v).cloned().unwrap_or(CoreType::Var(*v)),
        CoreType::Fun { args, ret, effects } => CoreType::Fun {
            args: args.iter().map(|a| subst_type_vars(a, map)).collect(),
            ret: Box::new(subst_type_vars(ret, map)),
            effects: effects.clone(),
        },
        CoreType::Record { fields } => CoreType::Record {
            fields: fields
                .iter()
                .map(|(k, v)| (k.clone(), subst_type_vars(v, map)))
                .collect(),
        },
        CoreType::OpenRecord { fields, row } => CoreType::OpenRecord {
            fields: fields
                .iter()
                .map(|(k, v)| (k.clone(), subst_type_vars(v, map)))
                .collect(),
            row: Box::new(subst_type_vars(row, map)),
        },
        CoreType::Variant { variants } => CoreType::Variant {
            variants: variants
                .iter()
                .map(|(k, v)| (k.clone(), v.as_ref().map(|t| subst_type_vars(t, map))))
                .collect(),
        },
        CoreType::App { ctor, args } => CoreType::App {
            ctor: ctor.clone(),
            args: args.iter().map(|a| subst_type_vars(a, map)).collect(),
        },
        CoreType::Forall { params, body } => CoreType::Forall {
            params: params.clone(),
            body: Box::new(subst_type_vars(body, map)),
        },
        CoreType::Union(ms) => {
            CoreType::Union(ms.iter().map(|m| subst_type_vars(m, map)).collect())
        }
        CoreType::Intersect(ms) => {
            CoreType::Intersect(ms.iter().map(|m| subst_type_vars(m, map)).collect())
        }
        CoreType::Not(inner) => CoreType::Not(Box::new(subst_type_vars(inner, map))),
        CoreType::Diff(a, b) => CoreType::Diff(
            Box::new(subst_type_vars(a, map)),
            Box::new(subst_type_vars(b, map)),
        ),
        CoreType::OptionalField(inner) => {
            CoreType::OptionalField(Box::new(subst_type_vars(inner, map)))
        }
        CoreType::Dynamic(inner) => CoreType::Dynamic(Box::new(subst_type_vars(inner, map))),
        CoreType::Lacks { label, row } => CoreType::Lacks {
            label: label.clone(),
            row: Box::new(subst_type_vars(row, map)),
        },
        other => other.clone(),
    }
}

fn subst_type_names(ty: &CoreType, map: &HashMap<String, CoreType>) -> CoreType {
    match ty {
        CoreType::Name(n) => map
            .get(n)
            .cloned()
            .unwrap_or_else(|| CoreType::Name(n.clone())),
        CoreType::Fun { args, ret, effects } => CoreType::Fun {
            args: args.iter().map(|a| subst_type_names(a, map)).collect(),
            ret: Box::new(subst_type_names(ret, map)),
            effects: effects.clone(),
        },
        CoreType::Record { fields } => CoreType::Record {
            fields: fields
                .iter()
                .map(|(k, v)| (k.clone(), subst_type_names(v, map)))
                .collect(),
        },
        CoreType::OpenRecord { fields, row } => CoreType::OpenRecord {
            fields: fields
                .iter()
                .map(|(k, v)| (k.clone(), subst_type_names(v, map)))
                .collect(),
            row: Box::new(subst_type_names(row, map)),
        },
        CoreType::Variant { variants } => CoreType::Variant {
            variants: variants
                .iter()
                .map(|(k, v)| (k.clone(), v.as_ref().map(|t| subst_type_names(t, map))))
                .collect(),
        },
        CoreType::App { ctor, args } => CoreType::App {
            ctor: ctor.clone(),
            args: args.iter().map(|a| subst_type_names(a, map)).collect(),
        },
        CoreType::Forall { params, body } => {
            let mut inner = map.clone();
            for (p, _) in params {
                inner.remove(p);
            }
            CoreType::Forall {
                params: params.clone(),
                body: Box::new(subst_type_names(body, &inner)),
            }
        }
        CoreType::Union(ms) => {
            CoreType::Union(ms.iter().map(|m| subst_type_names(m, map)).collect())
        }
        CoreType::Intersect(ms) => {
            CoreType::Intersect(ms.iter().map(|m| subst_type_names(m, map)).collect())
        }
        CoreType::Not(inner) => CoreType::Not(Box::new(subst_type_names(inner, map))),
        CoreType::Diff(a, b) => CoreType::Diff(
            Box::new(subst_type_names(a, map)),
            Box::new(subst_type_names(b, map)),
        ),
        CoreType::OptionalField(inner) => {
            CoreType::OptionalField(Box::new(subst_type_names(inner, map)))
        }
        CoreType::Dynamic(inner) => CoreType::Dynamic(Box::new(subst_type_names(inner, map))),
        CoreType::Lacks { label, row } => CoreType::Lacks {
            label: label.clone(),
            row: Box::new(subst_type_names(row, map)),
        },
        other => other.clone(),
    }
}

/// Expand `(option int)` / nominal apps to sealed constructor Variant shapes when known.
fn expand_type_app(ty: &CoreType, data: &DataEnv) -> CoreType {
    match ty {
        CoreType::App { ctor, args } => {
            let Some(params) = data.type_params.get(ctor) else {
                // Nullary / non-param ADT: expand data_ctors with schemas if present.
                return expand_nominal_variant(ctor, &HashMap::new(), data)
                    .unwrap_or_else(|| ty.clone());
            };
            if params.len() != args.len() {
                return ty.clone();
            }
            let map: HashMap<String, CoreType> =
                params.iter().cloned().zip(args.iter().cloned()).collect();
            expand_nominal_variant(ctor, &map, data).unwrap_or_else(|| ty.clone())
        }
        other => other.clone(),
    }
}

fn expand_nominal_variant(
    type_name: &str,
    map: &HashMap<String, CoreType>,
    data: &DataEnv,
) -> Option<CoreType> {
    let ctors = data.data_ctors.get(type_name)?;
    let mut variants = Vec::with_capacity(ctors.len());
    for (tag, _) in ctors {
        let schemas = data.ctor_payloads.get(tag).cloned().unwrap_or_default();
        let schemas: Vec<CoreType> = schemas.iter().map(|s| subst_type_names(s, map)).collect();
        variants.push((tag.clone(), schema_payload_type(&schemas)));
    }
    Some(CoreType::Variant { variants })
}

fn schema_payload_type(schemas: &[CoreType]) -> Option<CoreType> {
    match schemas {
        [] => None,
        [one] => Some(one.clone()),
        many => Some(CoreType::Record {
            fields: many
                .iter()
                .enumerate()
                .map(|(i, t)| (i.to_string(), t.clone()))
                .collect(),
        }),
    }
}

/// Parameterized ADT constructor → `App { ctor: type_name, args }` (DAT §7).
fn try_infer_parameterized_ctor(
    tag: &str,
    payload_ty: Option<&CoreType>,
    env: &TypeEnv,
    subst: &mut Subst,
    range: TextRange,
) -> Option<Result<CoreType, CheckError>> {
    let type_name = env.data.ctor_type.get(tag)?.clone();
    let params = env.data.type_params.get(&type_name)?;
    if params.is_empty() {
        return None;
    }
    let schemas = env.data.ctor_payloads.get(tag).cloned().unwrap_or_default();
    let mut name_map = HashMap::new();
    let mut arg_vars = Vec::with_capacity(params.len());
    for p in params {
        let v = CoreType::Var(subst.fresh_var());
        name_map.insert(p.clone(), v.clone());
        arg_vars.push(v);
    }
    let expected_schemas: Vec<CoreType> = schemas
        .iter()
        .map(|s| subst_type_names(s, &name_map))
        .collect();
    if let Err(e) = unify_ctor_payload(payload_ty, &expected_schemas, subst) {
        return Some(Err(unify_to_check(e, range)));
    }
    let args: Vec<CoreType> = arg_vars.iter().map(|t| subst.apply(t)).collect();
    Some(Ok(CoreType::App {
        ctor: type_name,
        args,
    }))
}

fn unify_ctor_payload(
    actual: Option<&CoreType>,
    schemas: &[CoreType],
    subst: &mut Subst,
) -> Result<(), UnifyError> {
    match (actual, schemas) {
        (None, []) => Ok(()),
        (Some(act), [exp]) => unify(act, exp, subst),
        (Some(CoreType::Record { fields }), schemas) if schemas.len() > 1 => {
            for (i, exp) in schemas.iter().enumerate() {
                let key = i.to_string();
                if let Some((_, act)) = fields.iter().find(|(k, _)| *k == key) {
                    unify(act, exp, subst)?;
                } else {
                    return Err(UnifyError::Mismatch {
                        expected: exp.clone(),
                        found: CoreType::dyn_any(),
                    });
                }
            }
            Ok(())
        }
        (None, _) | (Some(_), []) => Err(UnifyError::Mismatch {
            expected: schema_payload_type(schemas).unwrap_or(CoreType::Unit),
            found: actual.cloned().unwrap_or(CoreType::Unit),
        }),
        (Some(act), schemas) => {
            // Fallback: unify against packed schema shape.
            if let Some(exp) = schema_payload_type(schemas) {
                unify(act, &exp, subst)
            } else {
                Ok(())
            }
        }
    }
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
    // DD-TYP-DYN-007: predicates on dynamic S refine then→intersect(S,T), else→dynamic(diff(S,T)).
    if let CoreType::Dynamic(bound) = scr {
        let static_pred = |t: CoreType| {
            let then_ty = crate::cast::cast_success_type(bound, &t);
            let else_bound = crate::cast::normalize_type(&CoreType::Diff(
                Box::new(bound.as_ref().clone()),
                Box::new(t),
            ));
            let else_ty = match else_bound {
                CoreType::Never => CoreType::Never,
                other => CoreType::dynamic_bound(other),
            };
            (then_ty, else_ty)
        };
        return match pred {
            "number?" => Some(static_pred(CoreType::Number)),
            "string?" => Some(static_pred(CoreType::String)),
            "bool?" => Some(static_pred(CoreType::Bool)),
            _ => None,
        };
    }
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
                [] => CoreType::dyn_any(),
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
                    variants: vec![(tag.into(), Some(CoreType::dyn_any()))],
                },
            ])
        }
        other => CoreType::Intersect(vec![
            other.clone(),
            CoreType::Variant {
                variants: vec![(tag.into(), Some(CoreType::dyn_any()))],
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
    let scr_ty = expand_type_app(scr_ty, &env.data);
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
                        bind_pattern(ep, &CoreType::dyn_any(), env);
                    }
                }
            } else {
                for ep in elems {
                    bind_pattern(ep, &CoreType::dyn_any(), env);
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
                            bind_pattern(ep, &CoreType::dyn_any(), env);
                        } else {
                            bind_pattern(ep, ty, env);
                        }
                    } else {
                        // §18.6 unknown open-row fields: bind Dynamic interim.
                        bind_pattern(ep, &CoreType::dyn_any(), env);
                    }
                }
            }
            _ => {
                for (_, ep) in pats {
                    bind_pattern(ep, &CoreType::dyn_any(), env);
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
                            env.insert(name.clone(), CoreType::dyn_any());
                        } else {
                            bind_pattern(inner, &CoreType::dyn_any(), env);
                        }
                    }
                } else if let Some(inner) = payload {
                    // Tag not in scrutinee type; still bind Dynamic for nested binders.
                    bind_pattern(inner, &CoreType::dyn_any(), env);
                }
            } else if let Some(inner) = payload {
                bind_pattern(inner, &CoreType::dyn_any(), env);
            }
        }
    }
}

const NUMERIC_BINOPS: &[&str] = &["+", "-", "*", "/", "<", ">", "<=", ">="];

/// Resolve a numeric operand class, including `number?` occurrence refinements.
fn operand_numeric_class(ty: &CoreType) -> Option<NumericClass> {
    if let Some(c) = ty.numeric_class() {
        return Some(c);
    }
    match ty {
        CoreType::Dynamic(bound) => operand_numeric_class(bound),
        CoreType::Intersect(members) if members.len() == 2 => {
            let [scr, constraint] = members.as_slice() else {
                return None;
            };
            if matches!(constraint, CoreType::Number) {
                return numeric_union_class(scr);
            }
            // Prefer a concrete member if one is numeric.
            members.iter().find_map(operand_numeric_class)
        }
        CoreType::Union(_arms) => numeric_union_class(ty),
        _ => None,
    }
}

fn numeric_union_class(ty: &CoreType) -> Option<NumericClass> {
    let arms = match ty {
        CoreType::Union(arms) => arms.as_slice(),
        CoreType::Int => return Some(NumericClass::Int),
        CoreType::F64 => return Some(NumericClass::F64),
        _ => return None,
    };
    let mut int_ok = false;
    let mut f64_ok = false;
    for arm in arms {
        match arm {
            CoreType::Int => int_ok = true,
            CoreType::F64 => f64_ok = true,
            CoreType::Number => {
                int_ok = true;
                f64_ok = true;
            }
            _ => {}
        }
    }
    if int_ok && !f64_ok {
        Some(NumericClass::Int)
    } else if f64_ok && !int_ok {
        Some(NumericClass::F64)
    } else {
        None
    }
}

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
    let a_class = match operand_numeric_class(&a_ty) {
        Some(c) => c,
        None if matches!(a_ty, CoreType::Number | CoreType::Dynamic(_)) => {
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
    let b_class = match operand_numeric_class(&b_ty) {
        Some(c) => c,
        None if matches!(b_ty, CoreType::Number | CoreType::Dynamic(_)) => {
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
        args: vec![CoreType::dyn_any(), CoreType::dyn_any()],
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
        args: vec![CoreType::dyn_any()],
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
    env.insert(
        "encode-utf8",
        CoreType::Fun {
            args: vec![CoreType::String],
            ret: Box::new(CoreType::Bytes),
            effects: EffectRow::default(),
        },
    );
    env.insert(
        "decode-utf8",
        CoreType::Fun {
            args: vec![CoreType::Bytes],
            ret: Box::new(CoreType::dyn_any()),
            effects: EffectRow::default(),
        },
    );
    let int2 = CoreType::Fun {
        args: vec![CoreType::Int, CoreType::Int],
        ret: Box::new(CoreType::Int),
        effects: EffectRow::default(),
    };
    env.insert("int-div", int2.clone());
    env.insert("mod", int2);
    let ty = infer_expr(&expr, &env, &mut subst, TextRange::EMPTY)?;
    Ok(subst.apply(&ty))
}

/// DD-TYP-DYN-005: coerce a value of `found` to static `needed`, inserting `Cast` when needed.
pub fn coerce_to_static(
    expr: CoreExpr,
    found: &CoreType,
    needed: &CoreType,
    cast_id: u32,
) -> Result<CoreExpr, CheckError> {
    let found = found.clone();
    let needed = needed.clone();
    if crate::cast::is_subtype(&found, &needed) {
        return Ok(expr);
    }
    if let CoreType::Dynamic(bound) = &found {
        match crate::cast::judge_dynamic_use(bound, &needed) {
            crate::cast::DynamicUseJudgment::FullyIncluded => Ok(expr),
            crate::cast::DynamicUseJudgment::Disjoint => Err(CheckError::at(
                format!(
                    "dynamic bound is disjoint from required type (intersect ≃ never): {:?} vs {:?}",
                    bound, needed
                ),
                TextRange::EMPTY,
            )),
            crate::cast::DynamicUseJudgment::PartialOverlap { evidence, .. } => {
                Ok(CoreExpr::Cast {
                    expr: Box::new(expr),
                    evidence,
                    target: needed,
                    cast_id,
                })
            }
        }
    } else if crate::cast::types_disjoint(&found, &needed) {
        Err(CheckError::at(
            "cast is statically impossible (intersect ≃ never)",
            TextRange::EMPTY,
        ))
    } else if let Some(evidence) = crate::cast::plan_cast_evidence(&found, &needed) {
        if matches!(evidence, crate::cast::CastEvidence::Identity) {
            Ok(expr)
        } else {
            Ok(CoreExpr::Cast {
                expr: Box::new(expr),
                evidence,
                target: needed,
                cast_id,
            })
        }
    } else {
        Err(CheckError::at(
            "cast is statically impossible",
            TextRange::EMPTY,
        ))
    }
}

fn next_implicit_cast_id() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static ID: AtomicU32 = AtomicU32::new(10_000);
    ID.fetch_add(1, Ordering::Relaxed)
}

/// Rewrite Core, inserting implicit casts at application sites (DD-TYP-DYN-005).
pub fn insert_implicit_casts(expr: &CoreExpr, env: &TypeEnv) -> Result<CoreExpr, CheckError> {
    insert_casts_rec(expr, env, &mut Subst::new())
}

fn insert_casts_rec(
    expr: &CoreExpr,
    env: &TypeEnv,
    subst: &mut Subst,
) -> Result<CoreExpr, CheckError> {
    match expr {
        CoreExpr::App { fun, args } => {
            let fun2 = insert_casts_rec(fun, env, subst)?;
            let mut args2 = Vec::with_capacity(args.len());
            for a in args {
                args2.push(insert_casts_rec(a, env, subst)?);
            }
            let fun_ty = infer_expr(&fun2, env, subst, TextRange::EMPTY)?;
            let fun_ty = subst.apply(&fun_ty);
            if let CoreType::Fun {
                args: param_tys, ..
            } = fun_ty
            {
                if param_tys.len() == args2.len() {
                    for (arg, pty) in args2.iter_mut().zip(param_tys.iter()) {
                        let aty = infer_expr(arg, env, subst, TextRange::EMPTY)?;
                        let aty = subst.apply(&aty);
                        *arg = coerce_to_static(
                            std::mem::replace(arg, CoreExpr::Lit(CoreLiteral::Unit)),
                            &aty,
                            pty,
                            next_implicit_cast_id(),
                        )?;
                    }
                }
            }
            Ok(CoreExpr::App {
                fun: Box::new(fun2),
                args: args2,
            })
        }
        CoreExpr::Let { name, value, body } => {
            let value = insert_casts_rec(value, env, subst)?;
            let v_ty = infer_expr(&value, env, subst, TextRange::EMPTY)?;
            let mut child = env.clone();
            child.insert(name.clone(), v_ty);
            let body = insert_casts_rec(body, &child, subst)?;
            Ok(CoreExpr::Let {
                name: name.clone(),
                value: Box::new(value),
                body: Box::new(body),
            })
        }
        CoreExpr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let cond = insert_casts_rec(cond, env, subst)?;
            let (then_env, else_env) = occurrence_envs(&cond, env);
            Ok(CoreExpr::If {
                cond: Box::new(cond),
                then_branch: Box::new(insert_casts_rec(then_branch, &then_env, subst)?),
                else_branch: Box::new(insert_casts_rec(else_branch, &else_env, subst)?),
            })
        }
        CoreExpr::Seq(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(insert_casts_rec(item, env, subst)?);
            }
            Ok(CoreExpr::Seq(out))
        }
        CoreExpr::Lambda { params, body } => {
            let mut child = env.clone();
            for p in params {
                if !is_wildcard_ident(p) {
                    child.insert(p.clone(), CoreType::dyn_any());
                }
            }
            Ok(CoreExpr::Lambda {
                params: params.clone(),
                body: Box::new(insert_casts_rec(body, &child, subst)?),
            })
        }
        CoreExpr::Cast {
            expr: inner,
            evidence,
            target,
            cast_id,
        } => Ok(CoreExpr::Cast {
            expr: Box::new(insert_casts_rec(inner, env, subst)?),
            evidence: evidence.clone(),
            target: target.clone(),
            cast_id: *cast_id,
        }),
        CoreExpr::TryCast {
            expr: inner,
            target,
            cast_id,
        } => Ok(CoreExpr::TryCast {
            expr: Box::new(insert_casts_rec(inner, env, subst)?),
            target: target.clone(),
            cast_id: *cast_id,
        }),
        CoreExpr::CheckCast {
            expr: inner,
            target,
            cast_id,
        } => Ok(CoreExpr::CheckCast {
            expr: Box::new(insert_casts_rec(inner, env, subst)?),
            target: target.clone(),
            cast_id: *cast_id,
        }),
        other => Ok(other.clone()),
    }
}

#[cfg(test)]
mod coverage_helpers {
    use super::*;
    use crate::expr::{CoreLiteral, CorePattern, MatchArm};
    use crate::ty::{SingletonValue, TypeVarId};

    #[test]
    fn is_expansive_all_forms() {
        assert!(!is_expansive(&CoreExpr::Lit(CoreLiteral::Int(1))));
        assert!(!is_expansive(&CoreExpr::Var("x".into())));
        assert!(!is_expansive(&CoreExpr::Error));
        assert!(!is_expansive(&CoreExpr::Lambda {
            params: vec![],
            body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        }));
        assert!(!is_expansive(&CoreExpr::HandlerValue {
            op: "ask".into(),
            handler_params: vec!["x".into()],
            handler_body: Box::new(CoreExpr::Var("x".into())),
        }));
        assert!(is_expansive(&CoreExpr::App {
            fun: Box::new(CoreExpr::Var("f".into())),
            args: vec![],
        }));
        assert!(is_expansive(&CoreExpr::Perform {
            op: "log".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        }));
        assert!(is_expansive(&CoreExpr::Forward {
            resume_name: "k".into(),
        }));
        assert!(is_expansive(&CoreExpr::Set {
            name: "x".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
        }));
        assert!(is_expansive(&CoreExpr::Handle {
            op: "ask".into(),
            handler_params: vec!["m".into()],
            handler_body: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("f".into())),
                args: vec![],
            }),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        }));
        assert!(is_expansive(&CoreExpr::With {
            handler: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("f".into())),
                args: vec![],
            }),
            body: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        }));
        assert!(is_expansive(&CoreExpr::Seq(vec![CoreExpr::App {
            fun: Box::new(CoreExpr::Var("f".into())),
            args: vec![],
        }])));
        assert!(is_expansive(&CoreExpr::Let {
            name: "x".into(),
            value: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("f".into())),
                args: vec![],
            }),
            body: Box::new(CoreExpr::Var("x".into())),
        }));
        assert!(is_expansive(&CoreExpr::LocalVar {
            name: "c".into(),
            init: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("f".into())),
                args: vec![],
            }),
            body: Box::new(CoreExpr::Var("c".into())),
        }));
        assert!(is_expansive(&CoreExpr::LetRec {
            bindings: vec![(
                "f".into(),
                CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("g".into())),
                    args: vec![],
                },
            )],
            body: Box::new(CoreExpr::Var("f".into())),
        }));
        assert!(is_expansive(&CoreExpr::If {
            cond: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("f".into())),
                args: vec![],
            }),
            then_branch: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
            else_branch: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
        }));
        assert!(is_expansive(&CoreExpr::Record {
            fields: vec![(
                "a".into(),
                CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("f".into())),
                    args: vec![],
                },
            )],
        }));
        assert!(is_expansive(&CoreExpr::RecordUpdate {
            record: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("f".into())),
                args: vec![],
            }),
            fields: vec![],
        }));
        assert!(is_expansive(&CoreExpr::RecordExtend {
            record: Box::new(CoreExpr::Lit(CoreLiteral::Unit)),
            fields: vec![(
                "a".into(),
                CoreExpr::App {
                    fun: Box::new(CoreExpr::Var("f".into())),
                    args: vec![],
                },
            )],
        }));
        assert!(is_expansive(&CoreExpr::RecordGet {
            record: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("f".into())),
                args: vec![],
            }),
            field: "a".into(),
        }));
        assert!(is_expansive(&CoreExpr::Variant {
            tag: "some".into(),
            payload: Some(Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("f".into())),
                args: vec![],
            })),
        }));
        assert!(is_expansive(&CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("f".into())),
                args: vec![],
            }),
            arms: vec![MatchArm {
                pattern: CorePattern::Wildcard,
                body: CoreExpr::Lit(CoreLiteral::Unit),
            }],
        }));
        assert!(is_expansive(&CoreExpr::Cast {
            expr: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("f".into())),
                args: vec![],
            }),
            evidence: crate::cast::CastEvidence::Identity,
            target: CoreType::Int,
            cast_id: 0,
        }));
        assert!(is_expansive(&CoreExpr::TryCast {
            expr: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("f".into())),
                args: vec![],
            }),
            target: CoreType::Int,
            cast_id: 1,
        }));
        assert!(is_expansive(&CoreExpr::CheckCast {
            expr: Box::new(CoreExpr::App {
                fun: Box::new(CoreExpr::Var("f".into())),
                args: vec![],
            }),
            target: CoreType::Int,
            cast_id: 2,
        }));
    }

    #[test]
    fn type_mentions_effect_nested_shapes() {
        let op = "local-state/x";
        assert!(type_mentions_effect(
            &CoreType::Fun {
                args: vec![CoreType::Int],
                ret: Box::new(CoreType::Unit),
                effects: EffectRow::default().with_op(op),
            },
            op
        ));
        assert!(type_mentions_effect(
            &CoreType::Record {
                fields: vec![(
                    "f".into(),
                    CoreType::Fun {
                        args: vec![],
                        ret: Box::new(CoreType::Unit),
                        effects: EffectRow::default().with_op(op),
                    },
                )],
            },
            op
        ));
        assert!(type_mentions_effect(
            &CoreType::OpenRecord {
                fields: vec![(
                    "f".into(),
                    CoreType::Fun {
                        args: vec![],
                        ret: Box::new(CoreType::Unit),
                        effects: EffectRow::default().with_op(op),
                    },
                )],
                row: Box::new(CoreType::Unit),
            },
            op
        ));
        assert!(type_mentions_effect(
            &CoreType::Variant {
                variants: vec![(
                    "v".into(),
                    Some(CoreType::Fun {
                        args: vec![],
                        ret: Box::new(CoreType::Unit),
                        effects: EffectRow::default().with_op(op),
                    }),
                )],
            },
            op
        ));
        assert!(type_mentions_effect(
            &CoreType::Lacks {
                label: "a".into(),
                row: Box::new(CoreType::Fun {
                    args: vec![],
                    ret: Box::new(CoreType::Unit),
                    effects: EffectRow::default().with_op(op),
                }),
            },
            op
        ));
        assert!(type_mentions_effect(
            &CoreType::Union(vec![CoreType::Fun {
                args: vec![],
                ret: Box::new(CoreType::Unit),
                effects: EffectRow::default().with_op(op),
            }]),
            op
        ));
        assert!(type_mentions_effect(
            &CoreType::Intersect(vec![CoreType::Fun {
                args: vec![],
                ret: Box::new(CoreType::Unit),
                effects: EffectRow::default().with_op(op),
            }]),
            op
        ));
        assert!(type_mentions_effect(
            &CoreType::Not(Box::new(CoreType::Fun {
                args: vec![],
                ret: Box::new(CoreType::Unit),
                effects: EffectRow::default().with_op(op),
            })),
            op
        ));
        assert!(type_mentions_effect(
            &CoreType::OptionalField(Box::new(CoreType::Fun {
                args: vec![],
                ret: Box::new(CoreType::Unit),
                effects: EffectRow::default().with_op(op),
            })),
            op
        ));
        assert!(type_mentions_effect(
            &CoreType::Dynamic(Box::new(CoreType::Fun {
                args: vec![],
                ret: Box::new(CoreType::Unit),
                effects: EffectRow::default().with_op(op),
            })),
            op
        ));
        assert!(type_mentions_effect(
            &CoreType::Forall {
                params: vec![("a".into(), "type".into())],
                body: Box::new(CoreType::Fun {
                    args: vec![],
                    ret: Box::new(CoreType::Unit),
                    effects: EffectRow::default().with_op(op),
                }),
            },
            op
        ));
        assert!(type_mentions_effect(
            &CoreType::Diff(
                Box::new(CoreType::Fun {
                    args: vec![],
                    ret: Box::new(CoreType::Unit),
                    effects: EffectRow::default().with_op(op),
                }),
                Box::new(CoreType::Int),
            ),
            op
        ));
        assert!(type_mentions_effect(
            &CoreType::App {
                ctor: "box".into(),
                args: vec![CoreType::Fun {
                    args: vec![],
                    ret: Box::new(CoreType::Unit),
                    effects: EffectRow::default().with_op(op),
                }],
            },
            op
        ));
        assert!(!type_mentions_effect(&CoreType::Int, op));
        assert!(!type_mentions_effect(
            &CoreType::Singleton(SingletonValue::Int(1)),
            op
        ));
        // Fun: effect miss short-circuits into args/ret recursion.
        assert!(type_mentions_effect(
            &CoreType::Fun {
                args: vec![CoreType::Fun {
                    args: vec![],
                    ret: Box::new(CoreType::Unit),
                    effects: EffectRow::default().with_op(op),
                }],
                ret: Box::new(CoreType::Unit),
                effects: EffectRow::default(),
            },
            op
        ));
        assert!(type_mentions_effect(
            &CoreType::Fun {
                args: vec![CoreType::Int],
                ret: Box::new(CoreType::Fun {
                    args: vec![],
                    ret: Box::new(CoreType::Unit),
                    effects: EffectRow::default().with_op(op),
                }),
                effects: EffectRow::default().with_op("other"),
            },
            op
        ));
        assert!(!type_mentions_effect(
            &CoreType::Fun {
                args: vec![CoreType::Int],
                ret: Box::new(CoreType::Unit),
                effects: EffectRow::default().with_op("other"),
            },
            op
        ));
        // OpenRecord / Diff / Name / Error / Color / Shape leaves
        assert!(!type_mentions_effect(&CoreType::Color, op));
        assert!(!type_mentions_effect(&CoreType::Shape, op));
        assert!(!type_mentions_effect(&CoreType::Error, op));
        assert!(!type_mentions_effect(&CoreType::Name("t".into()), op));
        assert!(!type_mentions_effect(&CoreType::Never, op));
        assert!(!type_mentions_effect(&CoreType::Any, op));
        assert!(!type_mentions_effect(&CoreType::Bytes, op));
        assert!(!type_mentions_effect(&CoreType::Unit, op));
        assert!(!type_mentions_effect(&CoreType::Bool, op));
        assert!(!type_mentions_effect(&CoreType::F64, op));
        assert!(!type_mentions_effect(&CoreType::Number, op));
        assert!(!type_mentions_effect(&CoreType::String, op));
        assert!(!type_mentions_effect(&CoreType::Var(TypeVarId(0)), op));
    }

    #[test]
    fn subst_type_vars_and_names_complex() {
        let mut subst = Subst::new();
        let v = subst.fresh_var();
        let mut map = HashMap::new();
        map.insert(v, CoreType::Int);
        for ty in [
            CoreType::Var(v),
            CoreType::Fun {
                args: vec![CoreType::Var(v)],
                ret: Box::new(CoreType::Var(v)),
                effects: EffectRow::default(),
            },
            CoreType::Record {
                fields: vec![("a".into(), CoreType::Var(v))],
            },
            CoreType::OpenRecord {
                fields: vec![("a".into(), CoreType::Var(v))],
                row: Box::new(CoreType::Var(v)),
            },
            CoreType::Variant {
                variants: vec![("a".into(), Some(CoreType::Var(v)))],
            },
            CoreType::App {
                ctor: "t".into(),
                args: vec![CoreType::Var(v)],
            },
            CoreType::Forall {
                params: vec![("a".into(), "type".into())],
                body: Box::new(CoreType::Var(v)),
            },
            CoreType::Union(vec![CoreType::Var(v)]),
            CoreType::Intersect(vec![CoreType::Var(v)]),
            CoreType::Not(Box::new(CoreType::Var(v))),
            CoreType::Diff(Box::new(CoreType::Var(v)), Box::new(CoreType::Int)),
            CoreType::OptionalField(Box::new(CoreType::Var(v))),
            CoreType::Dynamic(Box::new(CoreType::Var(v))),
            CoreType::Lacks {
                label: "a".into(),
                row: Box::new(CoreType::Var(v)),
            },
            CoreType::String,
        ] {
            let _ = subst_type_vars(&ty, &map);
            let mut free = HashSet::new();
            collect_free_vars(&ty, &mut free);
        }

        let mut names = HashMap::new();
        names.insert("a".into(), CoreType::Int);
        for ty in [
            CoreType::Name("a".into()),
            CoreType::Name("b".into()),
            CoreType::Fun {
                args: vec![CoreType::Name("a".into())],
                ret: Box::new(CoreType::Name("a".into())),
                effects: EffectRow::default(),
            },
            CoreType::Record {
                fields: vec![("x".into(), CoreType::Name("a".into()))],
            },
            CoreType::OpenRecord {
                fields: vec![],
                row: Box::new(CoreType::Name("a".into())),
            },
            CoreType::Variant {
                variants: vec![("v".into(), Some(CoreType::Name("a".into())))],
            },
            CoreType::App {
                ctor: "t".into(),
                args: vec![CoreType::Name("a".into())],
            },
            CoreType::Forall {
                params: vec![("a".into(), "type".into())],
                body: Box::new(CoreType::Name("a".into())),
            },
            CoreType::Union(vec![CoreType::Name("a".into())]),
            CoreType::Intersect(vec![CoreType::Name("a".into())]),
            CoreType::Not(Box::new(CoreType::Name("a".into()))),
            CoreType::Diff(
                Box::new(CoreType::Name("a".into())),
                Box::new(CoreType::Int),
            ),
            CoreType::OptionalField(Box::new(CoreType::Name("a".into()))),
            CoreType::Dynamic(Box::new(CoreType::Name("a".into()))),
            CoreType::Lacks {
                label: "x".into(),
                row: Box::new(CoreType::Name("a".into())),
            },
            CoreType::Int,
        ] {
            let _ = subst_type_names(&ty, &names);
        }
    }

    #[test]
    fn refine_predicate_and_numeric_helpers() {
        assert!(refine_predicate("number?", &CoreType::dyn_any()).is_some());
        assert!(refine_predicate("string?", &CoreType::Dynamic(Box::new(CoreType::Any))).is_some());
        assert!(refine_predicate("bool?", &CoreType::Dynamic(Box::new(CoreType::Any))).is_some());
        assert!(refine_predicate("number?", &CoreType::Int).is_some());
        assert!(refine_predicate("nope", &CoreType::Int).is_none());
        // Static predicates cover then/else refine shapes for common domains.
        for pred in ["number?", "string?", "bool?", "is-none", "is-some"] {
            let _ = refine_predicate(pred, &CoreType::Any);
            let _ = refine_predicate(pred, &CoreType::Number);
            let _ = refine_predicate(pred, &CoreType::String);
            let _ = refine_predicate(pred, &CoreType::dyn_any());
            let option = CoreType::Variant {
                variants: vec![("none".into(), None), ("some".into(), Some(CoreType::Int))],
            };
            let _ = refine_predicate(pred, &option);
        }

        assert_eq!(
            operand_numeric_class(&CoreType::Int),
            Some(NumericClass::Int)
        );
        assert_eq!(
            operand_numeric_class(&CoreType::Dynamic(Box::new(CoreType::Int))),
            Some(NumericClass::Int)
        );
        assert_eq!(
            operand_numeric_class(&CoreType::Intersect(vec![
                CoreType::Union(vec![CoreType::Int]),
                CoreType::Number,
            ])),
            Some(NumericClass::Int)
        );
        assert_eq!(
            operand_numeric_class(&CoreType::Intersect(vec![CoreType::Int, CoreType::String,])),
            Some(NumericClass::Int)
        );
        assert_eq!(
            operand_numeric_class(&CoreType::Intersect(vec![
                CoreType::String,
                CoreType::Bool,
                CoreType::Unit,
            ])),
            None
        );
        assert_eq!(
            numeric_union_class(&CoreType::Union(vec![CoreType::Int, CoreType::F64])),
            None
        );
        assert_eq!(numeric_union_class(&CoreType::F64), Some(NumericClass::F64));
        assert_eq!(
            numeric_union_class(&CoreType::Union(vec![CoreType::Number])),
            None
        );
        assert_eq!(
            numeric_union_class(&CoreType::Union(vec![CoreType::Int, CoreType::String])),
            Some(NumericClass::Int)
        );
        assert_eq!(
            numeric_union_class(&CoreType::Union(vec![CoreType::F64, CoreType::String])),
            Some(NumericClass::F64)
        );
        assert_eq!(numeric_union_class(&CoreType::String), None);
    }

    #[test]
    fn variant_tag_keep_strip_and_field_access() {
        let option = CoreType::Variant {
            variants: vec![("none".into(), None), ("some".into(), Some(CoreType::Int))],
        };
        // strip leaving only `some` unwraps payload
        assert_eq!(strip_variant_tag(&option, "none"), CoreType::Int);
        // strip leaving empty → Dynamic
        let only_none = CoreType::Variant {
            variants: vec![("none".into(), None)],
        };
        assert!(matches!(
            strip_variant_tag(&only_none, "none"),
            CoreType::Dynamic(_)
        ));
        // strip leaving multiple → Variant rest
        let three = CoreType::Variant {
            variants: vec![
                ("a".into(), None),
                ("b".into(), Some(CoreType::Int)),
                ("c".into(), None),
            ],
        };
        assert!(matches!(
            strip_variant_tag(&three, "a"),
            CoreType::Variant { .. }
        ));
        // non-variant passthrough
        assert_eq!(strip_variant_tag(&CoreType::Int, "x"), CoreType::Int);

        // keep: payload tag → Intersect(scr, payload)
        assert!(matches!(
            keep_variant_tag(&option, "some"),
            CoreType::Intersect(_)
        ));
        // keep: nullary tag → Intersect with singleton Variant
        assert!(matches!(
            keep_variant_tag(&option, "none"),
            CoreType::Intersect(_)
        ));
        // keep: unknown tag on Variant → Intersect with Dynamic payload
        assert!(matches!(
            keep_variant_tag(&option, "maybe"),
            CoreType::Intersect(_)
        ));
        // keep: non-Variant scrutinee
        assert!(matches!(
            keep_variant_tag(&CoreType::String, "some"),
            CoreType::Intersect(_)
        ));

        assert!(matches!(
            field_access_type(CoreType::OptionalField(Box::new(CoreType::Int))),
            CoreType::Variant { .. }
        ));
        assert_eq!(field_access_type(CoreType::Int), CoreType::Int);

        assert_eq!(schema_payload_type(&[]), None);
        assert_eq!(schema_payload_type(&[CoreType::Int]), Some(CoreType::Int));
        assert!(matches!(
            schema_payload_type(&[CoreType::Int, CoreType::String]),
            Some(CoreType::Record { .. })
        ));
    }

    #[test]
    fn occurrence_envs_and_expand_type_app_matrix() {
        let mut env = TypeEnv::new();
        env.insert("x", CoreType::dyn_any());
        // Non-App / non-pred → identity envs
        let (a, b) = occurrence_envs(&CoreExpr::Lit(CoreLiteral::Bool(true)), &env);
        assert_eq!(a.vars.get("x"), b.vars.get("x"));
        // App but not Var fun / not single Var arg
        let app = CoreExpr::App {
            fun: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            args: vec![CoreExpr::Var("x".into())],
        };
        let _ = occurrence_envs(&app, &env);
        let app2 = CoreExpr::App {
            fun: Box::new(CoreExpr::Var("number?".into())),
            args: vec![
                CoreExpr::Var("x".into()),
                CoreExpr::Lit(CoreLiteral::Int(1)),
            ],
        };
        let _ = occurrence_envs(&app2, &env);
        // Unbound scrutinee name
        let app3 = CoreExpr::App {
            fun: Box::new(CoreExpr::Var("number?".into())),
            args: vec![CoreExpr::Var("missing".into())],
        };
        let _ = occurrence_envs(&app3, &env);
        // Unknown predicate
        let app4 = CoreExpr::App {
            fun: Box::new(CoreExpr::Var("weird?".into())),
            args: vec![CoreExpr::Var("x".into())],
        };
        let _ = occurrence_envs(&app4, &env);
        // Known predicate refines
        let app5 = CoreExpr::App {
            fun: Box::new(CoreExpr::Var("number?".into())),
            args: vec![CoreExpr::Var("x".into())],
        };
        let (then_env, else_env) = occurrence_envs(&app5, &env);
        assert_ne!(then_env.vars.get("x"), else_env.vars.get("x"));

        // expand_type_app / expand_nominal_variant
        let mut data = DataEnv::default();
        data.data_ctors.insert(
            "option".into(),
            vec![("none".into(), 0), ("some".into(), 1)],
        );
        data.ctor_payloads.insert("none".into(), vec![]);
        data.ctor_payloads
            .insert("some".into(), vec![CoreType::Name("a".into())]);
        data.type_params.insert("option".into(), vec!["a".into()]);
        let expanded = expand_type_app(
            &CoreType::App {
                ctor: "option".into(),
                args: vec![CoreType::Int],
            },
            &data,
        );
        assert!(matches!(expanded, CoreType::Variant { .. }));
        // arity mismatch → unchanged App
        let mismatch = expand_type_app(
            &CoreType::App {
                ctor: "option".into(),
                args: vec![],
            },
            &data,
        );
        assert!(matches!(mismatch, CoreType::App { .. }));
        // nullary unknown ADT
        let unknown = expand_type_app(
            &CoreType::App {
                ctor: "ghost".into(),
                args: vec![],
            },
            &data,
        );
        assert!(matches!(unknown, CoreType::App { .. }));
        // non-App passthrough
        assert_eq!(expand_type_app(&CoreType::Int, &data), CoreType::Int);
        // Nullary ADT without type_params entry but with data_ctors
        data.data_ctors
            .insert("color".into(), vec![("red".into(), 0)]);
        data.ctor_payloads.insert("red".into(), vec![]);
        let color = expand_type_app(
            &CoreType::App {
                ctor: "color".into(),
                args: vec![],
            },
            &data,
        );
        assert!(matches!(color, CoreType::Variant { .. }));
    }

    #[test]
    fn bind_pattern_and_unify_ctor_payload_edges() {
        let mut env = TypeEnv::new();
        // Tuple against Record keys
        bind_pattern(
            &CorePattern::Tuple(vec![
                CorePattern::Bind("a".into()),
                CorePattern::Bind("b".into()),
                CorePattern::Wildcard,
            ]),
            &CoreType::Record {
                fields: vec![("0".into(), CoreType::Int), ("1".into(), CoreType::String)],
            },
            &mut env,
        );
        assert_eq!(env.vars.get("a"), Some(&CoreType::Int));
        // Tuple against non-Record
        bind_pattern(
            &CorePattern::Tuple(vec![CorePattern::Bind("z".into())]),
            &CoreType::Int,
            &mut env,
        );
        // Record pattern / OpenRecord / OptionalField
        bind_pattern(
            &CorePattern::Record {
                fields: vec![
                    ("a".into(), CorePattern::Bind("ra".into())),
                    ("miss".into(), CorePattern::Bind("rm".into())),
                ],
            },
            &CoreType::Record {
                fields: vec![
                    ("a".into(), CoreType::Int),
                    (
                        "opt".into(),
                        CoreType::OptionalField(Box::new(CoreType::String)),
                    ),
                ],
            },
            &mut env,
        );
        bind_pattern(
            &CorePattern::Record {
                fields: vec![("a".into(), CorePattern::Bind("oa".into()))],
            },
            &CoreType::OpenRecord {
                fields: vec![("a".into(), CoreType::Int)],
                row: Box::new(CoreType::Unit),
            },
            &mut env,
        );
        bind_pattern(
            &CorePattern::Record {
                fields: vec![("a".into(), CorePattern::Bind("xa".into()))],
            },
            &CoreType::Int,
            &mut env,
        );
        // optional field label explicitly
        bind_pattern(
            &CorePattern::Record {
                fields: vec![("opt".into(), CorePattern::Bind("ro".into()))],
            },
            &CoreType::Record {
                fields: vec![(
                    "opt".into(),
                    CoreType::OptionalField(Box::new(CoreType::String)),
                )],
            },
            &mut env,
        );
        // Variant patterns
        bind_pattern(
            &CorePattern::Variant {
                tag: "some".into(),
                payload: Some(Box::new(CorePattern::Bind("v".into()))),
            },
            &CoreType::Variant {
                variants: vec![("some".into(), Some(CoreType::Int))],
            },
            &mut env,
        );
        bind_pattern(
            &CorePattern::Variant {
                tag: "none".into(),
                payload: None,
            },
            &CoreType::Variant {
                variants: vec![("none".into(), None)],
            },
            &mut env,
        );
        bind_pattern(
            &CorePattern::Variant {
                tag: "some".into(),
                payload: Some(Box::new(CorePattern::Bind("w".into()))),
            },
            &CoreType::Int,
            &mut env,
        );

        let mut subst = Subst::new();
        // unify_ctor_payload shape matrix
        assert!(unify_ctor_payload(None, &[], &mut subst).is_ok());
        assert!(unify_ctor_payload(Some(&CoreType::Int), &[], &mut subst).is_err());
        assert!(unify_ctor_payload(None, &[CoreType::Int], &mut subst).is_err());
        assert!(unify_ctor_payload(Some(&CoreType::Int), &[CoreType::Int], &mut subst).is_ok());
        assert!(unify_ctor_payload(
            Some(&CoreType::Record {
                fields: vec![("0".into(), CoreType::Int), ("1".into(), CoreType::String),],
            }),
            &[CoreType::Int, CoreType::String],
            &mut subst
        )
        .is_ok());
        // Fallback packed schema when actual not Record
        assert!(unify_ctor_payload(
            Some(&CoreType::Int),
            &[CoreType::Int, CoreType::String],
            &mut subst
        )
        .is_err());
        assert!(unify_ctor_payload(Some(&CoreType::Unit), &[], &mut subst).is_err());
    }

    #[test]
    fn infer_letrec_rhs_matrix() {
        let r = TextRange::EMPTY;
        let env = TypeEnv::new();
        let mut subst = Subst::new();
        // Non-lambda → Err (defensive)
        assert!(infer_letrec_rhs(
            &CoreExpr::Lit(CoreLiteral::Int(1)),
            None,
            &env,
            &mut subst,
            r
        )
        .is_err());
        // Lambda, no expected
        assert!(infer_letrec_rhs(
            &CoreExpr::Lambda {
                params: vec!["x".into()],
                body: Box::new(CoreExpr::Var("x".into())),
            },
            None,
            &env,
            &mut subst,
            r
        )
        .is_ok());
        // Lambda with Fun expected matching arity
        let expected = CoreType::Fun {
            args: vec![CoreType::Int],
            ret: Box::new(CoreType::Int),
            effects: EffectRow::default(),
        };
        assert!(infer_letrec_rhs(
            &CoreExpr::Lambda {
                params: vec!["x".into()],
                body: Box::new(CoreExpr::Var("x".into())),
            },
            Some(&expected),
            &env,
            &mut subst,
            r
        )
        .is_ok());
        // Arity mismatch falls through to plain infer
        let expected2 = CoreType::Fun {
            args: vec![CoreType::Int, CoreType::Int],
            ret: Box::new(CoreType::Int),
            effects: EffectRow::default(),
        };
        assert!(infer_letrec_rhs(
            &CoreExpr::Lambda {
                params: vec!["x".into()],
                body: Box::new(CoreExpr::Var("x".into())),
            },
            Some(&expected2),
            &env,
            &mut subst,
            r
        )
        .is_ok());
        // Wildcard params skip insert
        assert!(infer_letrec_rhs(
            &CoreExpr::Lambda {
                params: vec!["_".into()],
                body: Box::new(CoreExpr::Lit(CoreLiteral::Int(1))),
            },
            Some(&CoreType::Fun {
                args: vec![CoreType::Int],
                ret: Box::new(CoreType::Int),
                effects: EffectRow::default(),
            }),
            &env,
            &mut subst,
            r
        )
        .is_ok());
        // Ret unify fail
        assert!(infer_letrec_rhs(
            &CoreExpr::Lambda {
                params: vec!["x".into()],
                body: Box::new(CoreExpr::Lit(CoreLiteral::String("x".into()))),
            },
            Some(&CoreType::Fun {
                args: vec![CoreType::Int],
                ret: Box::new(CoreType::Int),
                effects: EffectRow::default(),
            }),
            &env,
            &mut subst,
            r
        )
        .is_err());
        // Non-Fun expected
        assert!(infer_letrec_rhs(
            &CoreExpr::Lambda {
                params: vec!["x".into()],
                body: Box::new(CoreExpr::Var("x".into())),
            },
            Some(&CoreType::Int),
            &env,
            &mut subst,
            r
        )
        .is_ok());
    }

    #[test]
    fn numeric_binop_ambiguous_and_dynamic() {
        let r = TextRange::EMPTY;
        let env = TypeEnv::new();
        let mut subst = Subst::new();
        // Ambiguous Number operands
        let args_num = [
            CoreExpr::Lit(CoreLiteral::Number(1.0)),
            CoreExpr::Lit(CoreLiteral::Int(2)),
        ];
        let _ = try_infer_numeric_builtin("+", &args_num, &env, &mut subst, r);
        // Dynamic operand
        let mut env2 = TypeEnv::new();
        env2.insert("d", CoreType::dyn_any());
        let app2 = [
            CoreExpr::Var("d".into()),
            CoreExpr::Lit(CoreLiteral::Int(1)),
        ];
        let _ = try_infer_numeric_builtin("+", &app2, &env2, &mut subst, r);
        let app3 = [
            CoreExpr::Lit(CoreLiteral::Int(1)),
            CoreExpr::Var("d".into()),
        ];
        let _ = try_infer_numeric_builtin("+", &app3, &env2, &mut subst, r);
        // Non-numeric
        let app4 = [
            CoreExpr::Lit(CoreLiteral::String("a".into())),
            CoreExpr::Lit(CoreLiteral::Int(1)),
        ];
        let _ = try_infer_numeric_builtin("+", &app4, &env, &mut subst, r);
        let app5 = [
            CoreExpr::Lit(CoreLiteral::Int(1)),
            CoreExpr::Lit(CoreLiteral::String("a".into())),
        ];
        let _ = try_infer_numeric_builtin("-", &app5, &env, &mut subst, r);
        // Wrong arity / non-binop → None
        assert!(try_infer_numeric_builtin(
            "+",
            &[CoreExpr::Lit(CoreLiteral::Int(1))],
            &env,
            &mut subst,
            r
        )
        .is_none());
        assert!(try_infer_numeric_builtin(
            "foo",
            &[
                CoreExpr::Lit(CoreLiteral::Int(1)),
                CoreExpr::Lit(CoreLiteral::Int(2))
            ],
            &env,
            &mut subst,
            r
        )
        .is_none());
        for op in ["+", "-", "*", "/", "<", ">", "<=", ">="] {
            let args = [
                CoreExpr::Lit(CoreLiteral::Int(1)),
                CoreExpr::Lit(CoreLiteral::Int(2)),
            ];
            let _ = try_infer_numeric_builtin(op, &args, &env, &mut subst, r);
            let args_f = [
                CoreExpr::Lit(CoreLiteral::F64(1.0)),
                CoreExpr::Lit(CoreLiteral::F64(2.0)),
            ];
            let _ = try_infer_numeric_builtin(op, &args_f, &env, &mut subst, r);
        }
        // OpenRecord RecordGet present/absent + OptionalField + binding-init residuals
        let r = TextRange::EMPTY;
        let mut env_or = TypeEnv::new();
        let mut subst_or = Subst::new();
        let row = CoreType::Var(subst_or.fresh_var());
        env_or.insert(
            "r",
            CoreType::OpenRecord {
                fields: vec![("a".into(), CoreType::Int)],
                row: Box::new(row),
            },
        );
        let _ = infer_with_effects(
            &CoreExpr::RecordGet {
                record: Box::new(CoreExpr::Var("r".into())),
                field: "a".into(),
            },
            &env_or,
            &mut subst_or,
            r,
        );
        let _ = infer_with_effects(
            &CoreExpr::RecordGet {
                record: Box::new(CoreExpr::Var("r".into())),
                field: "z".into(),
            },
            &env_or,
            &mut subst_or,
            r,
        );
        env_or.insert(
            "ro",
            CoreType::Record {
                fields: vec![(
                    "opt".into(),
                    CoreType::OptionalField(Box::new(CoreType::String)),
                )],
            },
        );
        let _ = infer_with_effects(
            &CoreExpr::RecordGet {
                record: Box::new(CoreExpr::Var("ro".into())),
                field: "opt".into(),
            },
            &env_or,
            &mut subst_or,
            r,
        );
        let mut env_ann = TypeEnv::new();
        env_ann.data.type_aliases.insert(
            "x".into(),
            CoreType::Forall {
                params: vec![("a".into(), "type".into())],
                body: Box::new(CoreType::Name("a".into())),
            },
        );
        let expansive = CoreExpr::App {
            fun: Box::new(CoreExpr::Var("missing".into())),
            args: vec![],
        };
        let _ = infer_binding_init("x", &expansive, &env_ann, &mut subst_or, r, true);
        env_ann.data.type_aliases.insert(
            "id".into(),
            CoreType::Fun {
                args: vec![CoreType::Int],
                ret: Box::new(CoreType::Int),
                effects: EffectRow::default(),
            },
        );
        let id_lam = CoreExpr::Lambda {
            params: vec!["x".into()],
            body: Box::new(CoreExpr::Var("x".into())),
        };
        let _ = infer_binding_init("id", &id_lam, &env_ann, &mut subst_or, r, true);
    }
}
