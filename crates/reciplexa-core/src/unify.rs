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
