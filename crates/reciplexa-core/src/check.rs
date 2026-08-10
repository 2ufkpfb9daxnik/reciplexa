//! Bidirectional type checking for Core expressions (Phase 2 §4.2 step 9).

use reciplexa_source::range::TextRange;

use crate::expr::{CoreExpr, CoreLiteral, CoreValue, MatchArm};
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
    match expr {
        CoreExpr::Lit(lit) => Ok(match lit {
            CoreLiteral::Number(_) => CoreType::Number,
            CoreLiteral::String(_) => CoreType::String,
            CoreLiteral::Color(_) => CoreType::Color,
            CoreLiteral::Bool(_) => CoreType::Bool,
        }),
        CoreExpr::Var(name) => env
            .vars
            .get(name)
            .cloned()
            .ok_or_else(|| CheckError::at(format!("unbound variable `{name}`"), range)),
        CoreExpr::Perform { op, arg } => {
            let arg_ty = infer_expr(arg, env, subst, range)?;
            if !matches!(arg_ty, CoreType::String) {
                return Err(CheckError::at("perform arg must be string", range));
            }
            let _ = op;
            Ok(CoreType::Unit)
        }
        CoreExpr::Seq(items) => {
            let mut last = CoreType::Unit;
            for item in items {
                last = infer_expr(item, env, subst, range)?;
            }
            Ok(last)
        }
        CoreExpr::Let { name, value, body } => {
            let v_ty = infer_expr(value, env, subst, range)?;
            let mut child = env.clone();
            child.insert(name.clone(), v_ty);
            infer_expr(body, &child, subst, range)
        }
        CoreExpr::Lambda { params, body } => {
            let mut child = env.clone();
            let mut arg_tys = Vec::with_capacity(params.len());
            for param in params {
                let p_ty = CoreType::Var(subst.fresh_var());
                child.insert(param.clone(), p_ty.clone());
                arg_tys.push(p_ty);
            }
            let ret = infer_expr(body, &child, subst, range)?;
            Ok(CoreType::Fun {
                args: arg_tys,
                ret: Box::new(ret),
                effects: EffectRow::default(),
            })
        }
        CoreExpr::App { fun, args } => {
            let fun_ty = infer_expr(fun, env, subst, range)?;
            let mut arg_tys = Vec::with_capacity(args.len());
            for arg in args {
                arg_tys.push(infer_expr(arg, env, subst, range)?);
            }
            let ret_var = CoreType::Var(subst.fresh_var());
            let expected = CoreType::Fun {
                args: arg_tys,
                ret: Box::new(ret_var.clone()),
                effects: EffectRow::default(),
            };
            unify_fun(&fun_ty, &expected, subst).map_err(|e| unify_to_check(e, range))?;
            Ok(subst.apply(&ret_var))
        }
        CoreExpr::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let cond_ty = infer_expr(cond, env, subst, range)?;
            unify(&cond_ty, &CoreType::Bool, subst).map_err(|e| unify_to_check(e, range))?;
            let then_ty = infer_expr(then_branch, env, subst, range)?;
            let else_ty = infer_expr(else_branch, env, subst, range)?;
            unify(&then_ty, &else_ty, subst).map_err(|e| unify_to_check(e, range))?;
            Ok(subst.apply(&then_ty))
        }
        CoreExpr::Record { fields } => {
            let mut typed = Vec::new();
            for (k, v) in fields {
                typed.push((k.clone(), infer_expr(v, env, subst, range)?));
            }
            Ok(CoreType::Record { fields: typed })
        }
        CoreExpr::RecordGet { record, field } => {
            let rec_ty = infer_expr(record, env, subst, range)?;
            match rec_ty {
                CoreType::Record { fields } => fields
                    .into_iter()
                    .find(|(k, _)| k == field)
                    .map(|(_, t)| t)
                    .ok_or_else(|| CheckError::at(format!("unknown field `{field}`"), range)),
                other => Err(CheckError::at(
                    format!("expected record, got {other:?}"),
                    range,
                )),
            }
        }
        CoreExpr::Variant { tag, payload } => {
            let payload_ty = if let Some(p) = payload {
                Some(infer_expr(p, env, subst, range)?)
            } else {
                None
            };
            Ok(CoreType::Variant {
                variants: vec![(tag.clone(), payload_ty)],
            })
        }
        CoreExpr::Match { scrutinee, arms } => {
            let scr_ty = infer_expr(scrutinee, env, subst, range)?;
            let ret_var = CoreType::Var(subst.fresh_var());
            for arm in arms {
                check_arm(arm, &scr_ty, &ret_var, env, subst, range)?;
            }
            Ok(subst.apply(&ret_var))
        }
    }
}

fn unify_fun(found: &CoreType, expected: &CoreType, subst: &mut Subst) -> Result<(), UnifyError> {
    unify(found, expected, subst)
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
) -> Result<(), CheckError> {
    let mut child = env.clone();
    if let CoreType::Variant { variants } = scr_ty {
        if let Some((_, payload)) = variants.iter().find(|(t, _)| t == &arm.tag) {
            if let Some(bind) = &arm.bind {
                if let Some(p_ty) = payload {
                    child.insert(bind.clone(), p_ty.clone());
                }
            }
        }
    }
    let body_ty = infer_expr(&arm.body, &child, subst, range)?;
    unify(&body_ty, ret_ty, subst).map_err(|e| unify_to_check(e, range))?;
    Ok(())
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
    let expr = crate::elaborate::elaborate_source(&expanded).map_err(|e| CheckError {
        message: e.message,
        range: e.range,
    })?;
    let mut subst = Subst::new();
    let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, TextRange::EMPTY)?;
    Ok(subst.apply(&ty))
}
