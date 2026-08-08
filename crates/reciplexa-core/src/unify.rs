//! Type unification and substitution (Phase 2 §4.2 steps 2–4).

use std::collections::HashMap;

use crate::ty::{CoreType, TypeVarId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnifyError {
    OccursCheck(TypeVarId, CoreType),
    Mismatch { expected: CoreType, found: CoreType },
}

/// Substitution map for type variables.
#[derive(Debug, Clone, Default)]
pub struct Subst {
    vars: HashMap<TypeVarId, CoreType>,
    next_id: u32,
}

impl Subst {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fresh_var(&mut self) -> TypeVarId {
        let id = TypeVarId::new(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn apply(&self, ty: &CoreType) -> CoreType {
        match ty {
            CoreType::Var(v) => self
                .vars
                .get(v)
                .map(|t| self.apply(t))
                .unwrap_or(CoreType::Var(*v)),
            CoreType::Fun { args, ret, effects } => CoreType::Fun {
                args: args.iter().map(|a| self.apply(a)).collect(),
                ret: Box::new(self.apply(ret)),
                effects: effects.clone(),
            },
            CoreType::Record { fields } => CoreType::Record {
                fields: fields
                    .iter()
                    .map(|(k, v)| (k.clone(), self.apply(v)))
                    .collect(),
            },
            CoreType::Variant { variants } => CoreType::Variant {
                variants: variants
                    .iter()
                    .map(|(k, v)| (k.clone(), v.as_ref().map(|t| self.apply(t))))
                    .collect(),
            },
            other => other.clone(),
        }
    }

    pub fn bind(&mut self, var: TypeVarId, ty: CoreType) -> Result<(), UnifyError> {
        let ty = self.apply(&ty);
        if let CoreType::Var(v) = &ty {
            if *v == var {
                return Ok(());
            }
        }
        if occurs(var, &ty) {
            return Err(UnifyError::OccursCheck(var, ty));
        }
        self.vars.insert(var, ty);
        Ok(())
    }
}

fn occurs(var: TypeVarId, ty: &CoreType) -> bool {
    match ty {
        CoreType::Var(v) => *v == var,
        CoreType::Fun { args, ret, .. } => args.iter().any(|a| occurs(var, a)) || occurs(var, ret),
        CoreType::Record { fields } => fields.iter().any(|(_, t)| occurs(var, t)),
        CoreType::Variant { variants } => variants
            .iter()
            .any(|(_, t)| t.as_ref().is_some_and(|x| occurs(var, x))),
        _ => false,
    }
}

pub fn unify(a: &CoreType, b: &CoreType, subst: &mut Subst) -> Result<(), UnifyError> {
    let a = subst.apply(a);
    let b = subst.apply(b);
    match (&a, &b) {
        (CoreType::Var(v), _) => subst.bind(*v, b),
        (_, CoreType::Var(v)) => subst.bind(*v, a),
        (CoreType::Number, CoreType::Number)
        | (CoreType::String, CoreType::String)
        | (CoreType::Color, CoreType::Color)
        | (CoreType::Shape, CoreType::Shape)
        | (CoreType::Unit, CoreType::Unit) => Ok(()),
        (
            CoreType::Fun {
                args: a_args,
                ret: a_ret,
                effects: a_eff,
            },
            CoreType::Fun {
                args: b_args,
                ret: b_ret,
                effects: b_eff,
            },
        ) => {
            if a_args.len() != b_args.len() {
                return Err(UnifyError::Mismatch {
                    expected: a,
                    found: b,
                });
            }
            for (x, y) in a_args.iter().zip(b_args.iter()) {
                unify(x, y, subst)?;
            }
            unify(a_ret, b_ret, subst)?;
            if a_eff.ops != b_eff.ops {
                return Err(UnifyError::Mismatch {
                    expected: a,
                    found: b,
                });
            }
            Ok(())
        }
        (CoreType::Record { fields: a_f }, CoreType::Record { fields: b_f }) => {
            if a_f.len() != b_f.len() {
                return Err(UnifyError::Mismatch {
                    expected: a,
                    found: b,
                });
            }
            for ((ak, av), (bk, bv)) in a_f.iter().zip(b_f.iter()) {
                if ak != bk {
                    return Err(UnifyError::Mismatch {
                        expected: a,
                        found: b,
                    });
                }
                unify(av, bv, subst)?;
            }
            Ok(())
        }
        (CoreType::Variant { variants: a_v }, CoreType::Variant { variants: b_v }) => {
            if a_v.len() != b_v.len() {
                return Err(UnifyError::Mismatch {
                    expected: a,
                    found: b,
                });
            }
            for ((at, ap), (bt, bp)) in a_v.iter().zip(b_v.iter()) {
                if at != bt {
                    return Err(UnifyError::Mismatch {
                        expected: a,
                        found: b,
                    });
                }
                match (ap, bp) {
                    (None, None) => {}
                    (Some(a_t), Some(b_t)) => unify(a_t, b_t, subst)?,
                    _ => {
                        return Err(UnifyError::Mismatch {
                            expected: a,
                            found: b,
                        });
                    }
                }
            }
            Ok(())
        }
        _ => Err(UnifyError::Mismatch {
            expected: a,
            found: b,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ty::EffectRow;

    #[test]
    fn unifies_function_types() {
        let mut s = Subst::new();
        let a = CoreType::Fun {
            args: vec![CoreType::Number],
            ret: Box::new(CoreType::Number),
            effects: EffectRow::default(),
        };
        let b = CoreType::Fun {
            args: vec![CoreType::Number],
            ret: Box::new(CoreType::Number),
            effects: EffectRow::default(),
        };
        assert!(unify(&a, &b, &mut s).is_ok());
    }

    #[test]
    fn fresh_var_unifies_with_number() {
        let mut s = Subst::new();
        let v = s.fresh_var();
        assert!(unify(&CoreType::Var(v), &CoreType::Number, &mut s).is_ok());
        assert_eq!(s.apply(&CoreType::Var(v)), CoreType::Number);
    }

    #[test]
    fn unifies_primitives() {
        let mut s = Subst::new();
        for ty in [
            CoreType::Number,
            CoreType::String,
            CoreType::Color,
            CoreType::Shape,
            CoreType::Unit,
        ] {
            assert!(unify(&ty, &ty, &mut s).is_ok());
        }
    }

    #[test]
    fn primitive_mismatch() {
        let mut s = Subst::new();
        let err = unify(&CoreType::Number, &CoreType::String, &mut s).unwrap_err();
        assert!(matches!(err, UnifyError::Mismatch { .. }));
    }

    #[test]
    fn occurs_check_rejects_infinite_type() {
        let mut s = Subst::new();
        let v = s.fresh_var();
        let fun = CoreType::Fun {
            args: vec![CoreType::Var(v)],
            ret: Box::new(CoreType::Var(v)),
            effects: EffectRow::default(),
        };
        let err = unify(&CoreType::Var(v), &fun, &mut s).unwrap_err();
        assert!(matches!(err, UnifyError::OccursCheck(_, _)));
    }

    #[test]
    fn self_bind_var_is_ok() {
        let mut s = Subst::new();
        let v = s.fresh_var();
        assert!(s.bind(v, CoreType::Var(v)).is_ok());
    }

    #[test]
    fn function_arg_count_mismatch() {
        let mut s = Subst::new();
        let a = CoreType::Fun {
            args: vec![CoreType::Number],
            ret: Box::new(CoreType::Unit),
            effects: EffectRow::default(),
        };
        let b = CoreType::Fun {
            args: vec![CoreType::Number, CoreType::Number],
            ret: Box::new(CoreType::Unit),
            effects: EffectRow::default(),
        };
        assert!(matches!(
            unify(&a, &b, &mut s),
            Err(UnifyError::Mismatch { .. })
        ));
    }

    #[test]
    fn function_effect_mismatch() {
        let mut s = Subst::new();
        let a = CoreType::Fun {
            args: vec![],
            ret: Box::new(CoreType::Unit),
            effects: EffectRow {
                ops: vec!["log".into()],
            },
        };
        let b = CoreType::Fun {
            args: vec![],
            ret: Box::new(CoreType::Unit),
            effects: EffectRow::default(),
        };
        assert!(matches!(
            unify(&a, &b, &mut s),
            Err(UnifyError::Mismatch { .. })
        ));
    }

    #[test]
    fn record_field_name_mismatch() {
        let mut s = Subst::new();
        let a = CoreType::Record {
            fields: vec![("x".into(), CoreType::Number)],
        };
        let b = CoreType::Record {
            fields: vec![("y".into(), CoreType::Number)],
        };
        assert!(matches!(
            unify(&a, &b, &mut s),
            Err(UnifyError::Mismatch { .. })
        ));
    }

    #[test]
    fn record_unifies_fields() {
        let mut s = Subst::new();
        let a = CoreType::Record {
            fields: vec![("x".into(), CoreType::Number)],
        };
        let b = CoreType::Record {
            fields: vec![("x".into(), CoreType::Number)],
        };
        assert!(unify(&a, &b, &mut s).is_ok());
    }

    #[test]
    fn variant_payload_mismatch() {
        let mut s = Subst::new();
        let a = CoreType::Variant {
            variants: vec![("Some".into(), Some(CoreType::Number))],
        };
        let b = CoreType::Variant {
            variants: vec![("Some".into(), None)],
        };
        assert!(matches!(
            unify(&a, &b, &mut s),
            Err(UnifyError::Mismatch { .. })
        ));
    }

    #[test]
    fn variant_unifies_nullary() {
        let mut s = Subst::new();
        let a = CoreType::Variant {
            variants: vec![("Done".into(), None)],
        };
        let b = CoreType::Variant {
            variants: vec![("Done".into(), None)],
        };
        assert!(unify(&a, &b, &mut s).is_ok());
    }

    #[test]
    fn apply_traverses_nested_types() {
        let mut s = Subst::new();
        let v = s.fresh_var();
        s.bind(v, CoreType::String).unwrap();
        let ty = CoreType::Fun {
            args: vec![CoreType::Var(v)],
            ret: Box::new(CoreType::Record {
                fields: vec![("f".into(), CoreType::Var(v))],
            }),
            effects: EffectRow::default(),
        };
        let applied = s.apply(&ty);
        assert_eq!(
            applied,
            CoreType::Fun {
                args: vec![CoreType::String],
                ret: Box::new(CoreType::Record {
                    fields: vec![("f".into(), CoreType::String)],
                }),
                effects: EffectRow::default(),
            }
        );
    }

    #[test]
    fn fresh_var_ids_increment() {
        let mut s = Subst::new();
        let a = s.fresh_var();
        let b = s.fresh_var();
        assert_ne!(a, b);
    }

    #[test]
    fn record_length_mismatch() {
        let mut s = Subst::new();
        let a = CoreType::Record {
            fields: vec![("x".into(), CoreType::Number)],
        };
        let b = CoreType::Record {
            fields: vec![
                ("x".into(), CoreType::Number),
                ("y".into(), CoreType::Number),
            ],
        };
        assert!(matches!(
            unify(&a, &b, &mut s),
            Err(UnifyError::Mismatch { .. })
        ));
    }

    #[test]
    fn variant_length_and_tag_mismatch() {
        let mut s = Subst::new();
        let a = CoreType::Variant {
            variants: vec![("A".into(), None)],
        };
        let b = CoreType::Variant {
            variants: vec![("A".into(), None), ("B".into(), None)],
        };
        assert!(matches!(
            unify(&a, &b, &mut s),
            Err(UnifyError::Mismatch { .. })
        ));
        let c = CoreType::Variant {
            variants: vec![("B".into(), None)],
        };
        assert!(matches!(
            unify(&a, &c, &mut s),
            Err(UnifyError::Mismatch { .. })
        ));
    }

    #[test]
    fn occurs_check_in_record_and_variant() {
        let mut s = Subst::new();
        let v = s.fresh_var();
        let rec = CoreType::Record {
            fields: vec![("f".into(), CoreType::Var(v))],
        };
        assert!(matches!(
            unify(&CoreType::Var(v), &rec, &mut s),
            Err(UnifyError::OccursCheck(_, _))
        ));
        let mut s2 = Subst::new();
        let v2 = s2.fresh_var();
        let var = CoreType::Variant {
            variants: vec![("Some".into(), Some(CoreType::Var(v2)))],
        };
        assert!(matches!(
            unify(&CoreType::Var(v2), &var, &mut s2),
            Err(UnifyError::OccursCheck(_, _))
        ));
    }

    #[test]
    fn unifies_variant_with_payload() {
        let mut s = Subst::new();
        let a = CoreType::Variant {
            variants: vec![("Some".into(), Some(CoreType::Number))],
        };
        let b = CoreType::Variant {
            variants: vec![("Some".into(), Some(CoreType::Number))],
        };
        assert!(unify(&a, &b, &mut s).is_ok());
    }
}
