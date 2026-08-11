//! Type unification and substitution (Phase 2 §4.2 steps 2–4).

#![allow(clippy::result_large_err)]

use std::collections::HashMap;

use crate::ty::{CoreType, TypeVarId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnifyError {
    OccursCheck(TypeVarId, CoreType),
    Mismatch {
        expected: CoreType,
        found: CoreType,
    },
    /// `Lacks` violated: `label` is present in `found`.
    LacksViolation {
        label: String,
        found: CoreType,
    },
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
            CoreType::OpenRecord { fields, row } => CoreType::OpenRecord {
                fields: fields
                    .iter()
                    .map(|(k, v)| (k.clone(), self.apply(v)))
                    .collect(),
                row: Box::new(self.apply(row)),
            },
            CoreType::Variant { variants } => CoreType::Variant {
                variants: variants
                    .iter()
                    .map(|(k, v)| (k.clone(), v.as_ref().map(|t| self.apply(t))))
                    .collect(),
            },
            CoreType::Lacks { label, row } => CoreType::Lacks {
                label: label.clone(),
                row: Box::new(self.apply(row)),
            },
            CoreType::Union(members) => {
                CoreType::Union(members.iter().map(|m| self.apply(m)).collect())
            }
            CoreType::Intersect(members) => {
                CoreType::Intersect(members.iter().map(|m| self.apply(m)).collect())
            }
            CoreType::Not(inner) => CoreType::Not(Box::new(self.apply(inner))),
            CoreType::Diff(a, b) => {
                CoreType::Diff(Box::new(self.apply(a)), Box::new(self.apply(b)))
            }
            CoreType::OptionalField(inner) => CoreType::OptionalField(Box::new(self.apply(inner))),
            CoreType::Dynamic(inner) => CoreType::Dynamic(Box::new(self.apply(inner))),
            CoreType::App { ctor, args } => CoreType::App {
                ctor: ctor.clone(),
                args: args.iter().map(|a| self.apply(a)).collect(),
            },
            CoreType::Forall { params, body } => CoreType::Forall {
                params: params.clone(),
                body: Box::new(self.apply(body)),
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
        CoreType::OpenRecord { fields, row } => {
            fields.iter().any(|(_, t)| occurs(var, t)) || occurs(var, row)
        }
        CoreType::Variant { variants } => variants
            .iter()
            .any(|(_, t)| t.as_ref().is_some_and(|x| occurs(var, x))),
        CoreType::Lacks { row, .. } => occurs(var, row),
        CoreType::Union(members) => members.iter().any(|m| occurs(var, m)),
        CoreType::Intersect(members) => members.iter().any(|m| occurs(var, m)),
        CoreType::Not(inner) => occurs(var, inner),
        CoreType::Diff(a, b) => occurs(var, a) || occurs(var, b),
        CoreType::OptionalField(inner) => occurs(var, inner),
        CoreType::Dynamic(inner) => occurs(var, inner),
        CoreType::App { args, .. } => args.iter().any(|a| occurs(var, a)),
        CoreType::Forall { body, .. } => occurs(var, body),
        _ => false,
    }
}

fn record_has_label(fields: &[(String, CoreType)], label: &str) -> bool {
    fields.iter().any(|(k, _)| k == label)
}

/// Enforce that `ty` lacks `label`, then unify the inner row constraint target.
fn enforce_lacks(label: &str, row: &CoreType, subst: &mut Subst) -> Result<(), UnifyError> {
    let row = subst.apply(row);
    match row {
        CoreType::Record { fields } => {
            if record_has_label(&fields, label) {
                return Err(UnifyError::LacksViolation {
                    label: label.to_string(),
                    found: CoreType::Record { fields },
                });
            }
            Ok(())
        }
        CoreType::OpenRecord { fields, row: rest } => {
            if record_has_label(&fields, label) {
                return Err(UnifyError::LacksViolation {
                    label: label.to_string(),
                    found: CoreType::OpenRecord { fields, row: rest },
                });
            }
            enforce_lacks(label, &rest, subst)
        }
        CoreType::Lacks {
            label: _inner_lab,
            row: inner_row,
        } => {
            // Peel nested lacks; underlying concrete row is checked below.
            enforce_lacks(label, &inner_row, subst)
        }
        CoreType::Var(_)
        | CoreType::Unit
        | CoreType::Dynamic(_)
        | CoreType::Union(_)
        | CoreType::Intersect(_)
        | CoreType::Not(_)
        | CoreType::Diff(_, _)
        | CoreType::Name(_)
        | CoreType::App { .. }
        | CoreType::Forall { .. } => Ok(()),
        other => Err(UnifyError::Mismatch {
            expected: CoreType::Lacks {
                label: label.to_string(),
                row: Box::new(CoreType::Unit),
            },
            found: other,
        }),
    }
}

/// Unify open `{a_fields | a_row}` with closed `b_fields` (exact remaining → a_row).
fn unify_open_with_closed(
    a_fields: &[(String, CoreType)],
    a_row: &CoreType,
    b_fields: &[(String, CoreType)],
    subst: &mut Subst,
    expected: &CoreType,
    found: &CoreType,
) -> Result<(), UnifyError> {
    let mut remaining: Vec<(String, CoreType)> = b_fields.to_vec();
    for (ak, av) in a_fields {
        let idx = remaining.iter().position(|(bk, _)| bk == ak);
        let Some(idx) = idx else {
            return Err(UnifyError::Mismatch {
                expected: expected.clone(),
                found: found.clone(),
            });
        };
        let (_, bv) = remaining.remove(idx);
        unify(av, &bv, subst)?;
    }
    let rest_ty = if remaining.is_empty() {
        CoreType::Record { fields: vec![] }
    } else {
        CoreType::Record { fields: remaining }
    };
    unify(a_row, &rest_ty, subst)
}

pub fn unify(a: &CoreType, b: &CoreType, subst: &mut Subst) -> Result<(), UnifyError> {
    let a = subst.apply(a);
    let b = subst.apply(b);
    match (&a, &b) {
        (CoreType::Var(v), _) => subst.bind(*v, b),
        (_, CoreType::Var(v)) => subst.bind(*v, a),
        // SYN §18.10: internal error type absorbs any expected type.
        (CoreType::Error, _) | (_, CoreType::Error) => Ok(()),
        // ERR-001 §4.2: never is a subtype of every type.
        (CoreType::Never, _) | (_, CoreType::Never) => Ok(()),
        // DD-TYP-DYN-003: `any` is static top; `S <: any` but not `any <: T`.
        (CoreType::Any, CoreType::Any) => Ok(()),
        (_, CoreType::Any) => Ok(()),
        (CoreType::Any, _) => Err(UnifyError::Mismatch {
            expected: b.clone(),
            found: a.clone(),
        }),
        // Gradual consistency: `dynamic S` is consistent with every type (≠ subtype).
        // Disjointness is enforced at use sites via `judge_dynamic_use` (DD-TYP-DYN-005).
        (CoreType::Dynamic(_), _) | (_, CoreType::Dynamic(_)) => Ok(()),
        // SYN §16.3 union stub: treat like Dynamic for v0.
        (CoreType::Union(_), _) | (_, CoreType::Union(_)) => Ok(()),
        // SYN §16.3 intersect/not/diff stubs until full semantic subtyping lands.
        (CoreType::Intersect(_), _) | (_, CoreType::Intersect(_)) => Ok(()),
        (CoreType::Not(_), _) | (_, CoreType::Not(_)) => Ok(()),
        (CoreType::Diff(_, _), _) | (_, CoreType::Diff(_, _)) => Ok(()),
        // SYN §16.1/16.2 surface stubs until polymorphic instantiation lands.
        (CoreType::App { .. }, _)
        | (_, CoreType::App { .. })
        | (CoreType::Forall { .. }, _)
        | (_, CoreType::Forall { .. })
        | (CoreType::Name(_), _)
        | (_, CoreType::Name(_)) => Ok(()),
        (CoreType::OptionalField(a_inner), CoreType::OptionalField(b_inner)) => {
            unify(a_inner, b_inner, subst)
        }
        (CoreType::Int, CoreType::Int)
        | (CoreType::F64, CoreType::F64)
        | (CoreType::Int, CoreType::Number)
        | (CoreType::Number, CoreType::Int)
        | (CoreType::F64, CoreType::Number)
        | (CoreType::Number, CoreType::F64)
        | (CoreType::Number, CoreType::Number)
        | (CoreType::String, CoreType::String)
        | (CoreType::Color, CoreType::Color)
        | (CoreType::Shape, CoreType::Shape)
        | (CoreType::Unit, CoreType::Unit)
        | (CoreType::Bool, CoreType::Bool)
        | (CoreType::Bytes, CoreType::Bytes) => Ok(()),
        (CoreType::Singleton(a_s), CoreType::Singleton(b_s)) if a_s == b_s => Ok(()),
        (CoreType::Singleton(a_s), other) | (other, CoreType::Singleton(a_s))
            if other == &CoreType::singleton_domain(a_s) =>
        {
            Ok(())
        }
        (CoreType::Int, CoreType::F64) | (CoreType::F64, CoreType::Int) => {
            Err(UnifyError::Mismatch {
                expected: a,
                found: b,
            })
        }
        (
            CoreType::Lacks {
                label: a_lab,
                row: a_row,
            },
            CoreType::Lacks {
                label: b_lab,
                row: b_row,
            },
        ) if a_lab == b_lab => {
            enforce_lacks(a_lab, a_row, subst)?;
            enforce_lacks(b_lab, b_row, subst)?;
            unify(a_row, b_row, subst)
        }
        (CoreType::Lacks { .. }, CoreType::Lacks { .. }) => Err(UnifyError::Mismatch {
            expected: a,
            found: b,
        }),
        (CoreType::Lacks { label, row }, other) => {
            enforce_lacks(label, other, subst)?;
            unify(row, other, subst)
        }
        (other, CoreType::Lacks { label, row }) => {
            enforce_lacks(label, other, subst)?;
            unify(other, row, subst)
        }
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
        (
            CoreType::OpenRecord {
                fields: a_f,
                row: a_row,
            },
            CoreType::Record { fields: b_f },
        ) => unify_open_with_closed(a_f, a_row, b_f, subst, &a, &b),
        (
            CoreType::Record { fields: a_f },
            CoreType::OpenRecord {
                fields: b_f,
                row: b_row,
            },
        ) => unify_open_with_closed(b_f, b_row, a_f, subst, &b, &a),
        (
            CoreType::OpenRecord {
                fields: a_f,
                row: a_row,
            },
            CoreType::OpenRecord {
                fields: b_f,
                row: b_row,
            },
        ) => {
            // Shared labels must unify; exclusive labels are pushed into the opposite tail.
            let mut a_only = Vec::new();
            let mut b_rest = b_f.to_vec();
            for (ak, av) in a_f {
                if let Some(idx) = b_rest.iter().position(|(bk, _)| bk == ak) {
                    let (_, bv) = b_rest.remove(idx);
                    unify(av, &bv, subst)?;
                } else {
                    a_only.push((ak.clone(), av.clone()));
                }
            }
            let a_tail = if a_only.is_empty() {
                (**a_row).clone()
            } else {
                CoreType::OpenRecord {
                    fields: a_only,
                    row: a_row.clone(),
                }
            };
            let b_tail = if b_rest.is_empty() {
                (**b_row).clone()
            } else {
                CoreType::OpenRecord {
                    fields: b_rest,
                    row: b_row.clone(),
                }
            };
            unify(&a_tail, &b_tail, subst)
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
        // Empty closed record ≈ unit row tail for open-row fragments.
        (CoreType::Record { fields }, CoreType::Unit)
        | (CoreType::Unit, CoreType::Record { fields })
            if fields.is_empty() =>
        {
            Ok(())
        }
        _ => Err(UnifyError::Mismatch {
            expected: a,
            found: b,
        }),
    }
}
