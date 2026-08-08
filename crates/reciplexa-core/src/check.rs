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
        }),
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
        CoreExpr::Lambda { param, body } => {
            let p_ty = CoreType::Var(subst.fresh_var());
            let mut child = env.clone();
            child.insert(param.clone(), p_ty.clone());
            let ret = infer_expr(body, &child, subst, range)?;
            Ok(CoreType::Fun {
                args: vec![p_ty],
                ret: Box::new(ret),
                effects: EffectRow::default(),
            })
        }
        CoreExpr::App { fun, arg } => {
            let fun_ty = infer_expr(fun, env, subst, range)?;
            let arg_ty = infer_expr(arg, env, subst, range)?;
            let ret_var = CoreType::Var(subst.fresh_var());
            let expected = CoreType::Fun {
                args: vec![arg_ty],
                ret: Box::new(ret_var.clone()),
                effects: EffectRow::default(),
            };
            unify_fun(&fun_ty, &expected, subst).map_err(|e| unify_to_check(e, range))?;
            Ok(subst.apply(&ret_var))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::CoreExpr;
    use reciplexa_source::offset::ByteOffset;

    fn range() -> TextRange {
        TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap()
    }

    #[test]
    fn infers_lambda_application() {
        let expr = CoreExpr::App {
            fun: Box::new(CoreExpr::Lambda {
                param: "x".into(),
                body: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
            }),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
        };
        let mut subst = Subst::new();
        let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
        assert_eq!(subst.apply(&ty), CoreType::Number);
    }

    #[test]
    fn infers_record_get() {
        let expr = CoreExpr::RecordGet {
            record: Box::new(CoreExpr::Record {
                fields: vec![
                    ("x".into(), CoreExpr::Lit(CoreLiteral::Number(1.0))),
                    ("y".into(), CoreExpr::Lit(CoreLiteral::Number(2.0))),
                ],
            }),
            field: "y".into(),
        };
        let mut subst = Subst::new();
        let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
        assert_eq!(ty, CoreType::Number);
    }

    #[test]
    fn perform_requires_string_arg() {
        let expr = CoreExpr::Perform {
            op: "log".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
        };
        let mut subst = Subst::new();
        let err = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap_err();
        assert!(err.message.contains("string"));
    }

    #[test]
    fn record_get_unknown_field_errors() {
        let expr = CoreExpr::RecordGet {
            record: Box::new(CoreExpr::Record {
                fields: vec![("a".into(), CoreExpr::Lit(CoreLiteral::Number(1.0)))],
            }),
            field: "missing".into(),
        };
        let mut subst = Subst::new();
        let err = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap_err();
        assert!(err.message.contains("unknown field"));
    }

    #[test]
    fn record_get_on_non_record_errors() {
        let expr = CoreExpr::RecordGet {
            record: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
            field: "x".into(),
        };
        let mut subst = Subst::new();
        let err = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap_err();
        assert!(err.message.contains("expected record"));
    }

    #[test]
    fn infers_seq_returns_last() {
        let expr = CoreExpr::Seq(vec![
            CoreExpr::Lit(CoreLiteral::String("a".into())),
            CoreExpr::Lit(CoreLiteral::Number(2.0)),
        ]);
        let mut subst = Subst::new();
        let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
        assert_eq!(ty, CoreType::Number);
    }

    #[test]
    fn infers_let_and_variant() {
        let expr = CoreExpr::Let {
            name: "v".into(),
            value: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
            body: Box::new(CoreExpr::Variant {
                tag: "Ok".into(),
                payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0)))),
            }),
        };
        let mut subst = Subst::new();
        let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
        assert!(matches!(ty, CoreType::Variant { .. }));
    }

    #[test]
    fn typecheck_value_wraps_expr() {
        let expr = CoreExpr::Lit(CoreLiteral::Color("red".into()));
        let cv = typecheck_value(expr, &TypeEnv::new(), range()).unwrap();
        assert_eq!(cv.ty, CoreType::Color);
    }

    #[test]
    fn type_env_insert() {
        let mut env = TypeEnv::new();
        env.insert("x", CoreType::Number);
        let expr = CoreExpr::Lit(CoreLiteral::Number(1.0));
        let mut subst = Subst::new();
        let ty = infer_expr(&expr, &env, &mut subst, range()).unwrap();
        assert_eq!(ty, CoreType::Number);
    }

    #[test]
    fn match_arm_unifies_return_types() {
        let expr = CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Variant {
                tag: "A".into(),
                payload: None,
            }),
            arms: vec![
                MatchArm {
                    tag: "A".into(),
                    bind: None,
                    body: CoreExpr::Lit(CoreLiteral::Number(1.0)),
                },
                MatchArm {
                    tag: "B".into(),
                    bind: None,
                    body: CoreExpr::Lit(CoreLiteral::Number(2.0)),
                },
            ],
        };
        let mut subst = Subst::new();
        let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
        assert_eq!(subst.apply(&ty), CoreType::Number);
    }

    #[test]
    fn perform_with_string_ok() {
        let expr = CoreExpr::Perform {
            op: "log".into(),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::String("ok".into()))),
        };
        let mut subst = Subst::new();
        let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
        assert_eq!(ty, CoreType::Unit);
    }

    #[test]
    fn match_arm_return_type_mismatch() {
        let expr = CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Variant {
                tag: "A".into(),
                payload: None,
            }),
            arms: vec![
                MatchArm {
                    tag: "A".into(),
                    bind: None,
                    body: CoreExpr::Lit(CoreLiteral::Number(1.0)),
                },
                MatchArm {
                    tag: "B".into(),
                    bind: None,
                    body: CoreExpr::Lit(CoreLiteral::String("no".into())),
                },
            ],
        };
        let mut subst = Subst::new();
        assert!(infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).is_err());
    }

    #[test]
    fn match_binds_payload_in_arm() {
        let expr = CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Variant {
                tag: "Some".into(),
                payload: Some(Box::new(CoreExpr::Lit(CoreLiteral::Number(5.0)))),
            }),
            arms: vec![MatchArm {
                tag: "Some".into(),
                bind: Some("v".into()),
                body: CoreExpr::Lit(CoreLiteral::Number(5.0)),
            }],
        };
        let mut subst = Subst::new();
        let ty = infer_expr(&expr, &TypeEnv::new(), &mut subst, range()).unwrap();
        assert_eq!(subst.apply(&ty), CoreType::Number);
    }

    fn bad_record_get() -> CoreExpr {
        CoreExpr::RecordGet {
            record: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
            field: "missing".into(),
        }
    }

    #[test]
    fn infer_propagates_nested_errors() {
        let bad = bad_record_get();
        let mut subst = Subst::new();
        let env = TypeEnv::new();
        let r = range();

        assert!(infer_expr(
            &CoreExpr::Perform {
                op: "log".into(),
                arg: Box::new(bad.clone()),
            },
            &env,
            &mut subst,
            r
        )
        .is_err());

        assert!(infer_expr(
            &CoreExpr::Seq(vec![CoreExpr::Lit(CoreLiteral::Number(1.0)), bad.clone(),]),
            &env,
            &mut subst,
            r
        )
        .is_err());

        assert!(infer_expr(
            &CoreExpr::Let {
                name: "x".into(),
                value: Box::new(bad.clone()),
                body: Box::new(CoreExpr::Lit(CoreLiteral::Number(0.0))),
            },
            &env,
            &mut subst,
            r
        )
        .is_err());

        assert!(infer_expr(
            &CoreExpr::Record {
                fields: vec![("a".into(), bad.clone())],
            },
            &env,
            &mut subst,
            r
        )
        .is_err());

        assert!(infer_expr(
            &CoreExpr::Variant {
                tag: "Some".into(),
                payload: Some(Box::new(bad.clone())),
            },
            &env,
            &mut subst,
            r
        )
        .is_err());

        assert!(infer_expr(
            &CoreExpr::Let {
                name: "x".into(),
                value: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
                body: Box::new(bad.clone()),
            },
            &env,
            &mut subst,
            r
        )
        .is_err());

        assert!(infer_expr(
            &CoreExpr::Lambda {
                param: "x".into(),
                body: Box::new(bad.clone()),
            },
            &env,
            &mut subst,
            r
        )
        .is_err());

        assert!(infer_expr(
            &CoreExpr::App {
                fun: Box::new(bad.clone()),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
            },
            &env,
            &mut subst,
            r
        )
        .is_err());

        assert!(infer_expr(
            &CoreExpr::App {
                fun: Box::new(CoreExpr::Lambda {
                    param: "x".into(),
                    body: Box::new(CoreExpr::Lit(CoreLiteral::Number(0.0))),
                }),
                arg: Box::new(bad.clone()),
            },
            &env,
            &mut subst,
            r
        )
        .is_err());

        assert!(infer_expr(
            &CoreExpr::Match {
                scrutinee: Box::new(bad.clone()),
                arms: vec![MatchArm {
                    tag: "A".into(),
                    bind: None,
                    body: CoreExpr::Lit(CoreLiteral::Number(0.0)),
                }],
            },
            &env,
            &mut subst,
            r
        )
        .is_err());

        assert!(infer_expr(
            &CoreExpr::RecordGet {
                record: Box::new(bad),
                field: "x".into(),
            },
            &env,
            &mut subst,
            r
        )
        .is_err());
    }

    #[test]
    fn app_non_function_unify_fails() {
        let expr = CoreExpr::App {
            fun: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
            arg: Box::new(CoreExpr::Lit(CoreLiteral::Number(2.0))),
        };
        let err = infer_expr(&expr, &TypeEnv::new(), &mut Subst::new(), range()).unwrap_err();
        assert!(err.message.contains("Mismatch"));
    }

    #[test]
    fn typecheck_value_propagates_infer_error() {
        let expr = bad_record_get();
        assert!(typecheck_value(expr, &TypeEnv::new(), range()).is_err());
    }

    #[test]
    fn infers_empty_seq_and_lambda() {
        let mut subst = Subst::new();
        let empty =
            infer_expr(&CoreExpr::Seq(vec![]), &TypeEnv::new(), &mut subst, range()).unwrap();
        assert_eq!(empty, CoreType::Unit);

        let lam = CoreExpr::Lambda {
            param: "x".into(),
            body: Box::new(CoreExpr::Lit(CoreLiteral::String("ok".into()))),
        };
        let ty = infer_expr(&lam, &TypeEnv::new(), &mut subst, range()).unwrap();
        if let CoreType::Fun { args, .. } = ty {
            assert_eq!(args.len(), 1);
        } else {
            panic!("expected Fun");
        }
    }

    #[test]
    fn match_on_non_variant_scrutinee() {
        let expr = CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Lit(CoreLiteral::Number(1.0))),
            arms: vec![MatchArm {
                tag: "ignored".into(),
                bind: None,
                body: CoreExpr::Lit(CoreLiteral::Number(2.0)),
            }],
        };
        let ty = infer_expr(&expr, &TypeEnv::new(), &mut Subst::new(), range()).unwrap();
        assert_eq!(ty, CoreType::Number);
    }

    #[test]
    fn check_arm_body_infer_error() {
        let expr = CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Variant {
                tag: "A".into(),
                payload: None,
            }),
            arms: vec![MatchArm {
                tag: "A".into(),
                bind: None,
                body: bad_record_get(),
            }],
        };
        assert!(infer_expr(&expr, &TypeEnv::new(), &mut Subst::new(), range()).is_err());
    }

    #[test]
    fn match_arm_bind_without_payload_and_unknown_tag() {
        // Nullary variant + bind name: no payload to insert.
        let nullary = CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Variant {
                tag: "None".into(),
                payload: None,
            }),
            arms: vec![MatchArm {
                tag: "None".into(),
                bind: Some("x".into()),
                body: CoreExpr::Lit(CoreLiteral::Number(0.0)),
            }],
        };
        let mut subst = Subst::new();
        assert!(infer_expr(&nullary, &TypeEnv::new(), &mut subst, range()).is_ok());

        // Arm tag absent from scrutinee type still typechecks the body.
        let mismatch_tag = CoreExpr::Match {
            scrutinee: Box::new(CoreExpr::Variant {
                tag: "A".into(),
                payload: None,
            }),
            arms: vec![MatchArm {
                tag: "B".into(),
                bind: Some("x".into()),
                body: CoreExpr::Lit(CoreLiteral::Number(1.0)),
            }],
        };
        let mut subst = Subst::new();
        assert!(infer_expr(&mismatch_tag, &TypeEnv::new(), &mut subst, range()).is_ok());
    }
}
