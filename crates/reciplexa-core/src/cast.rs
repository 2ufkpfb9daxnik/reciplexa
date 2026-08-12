//! DD-TYP-DYN: cast evidence, three-way dynamic use, and success-type intersection.

use crate::ty::{CoreType, EffectRow};

/// Cast evidence algebra (DD-TYP-DYN-015).
///
/// Evidence execution is pure (DD-TYP-DYN-016): inspect / wrap / identity only —
/// never I/O, ambient effects, or handler-stack observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CastEvidence {
    Identity,
    Widen,
    TagCheck {
        tag: String,
    },
    UnionCheck {
        members: Vec<CoreType>,
    },
    IntersectionCheck {
        members: Vec<CoreType>,
    },
    RecordCheck {
        fields: Vec<(String, CoreType)>,
    },
    VariantCheck {
        variants: Vec<(String, Option<CoreType>)>,
    },
    FunctionGuard {
        arity: usize,
        /// Contravariant argument casts (target arg → source arg) planned at call time.
        arg_casts: Vec<CastEvidence>,
        /// Covariant result cast (source ret → target ret).
        ret_cast: Box<CastEvidence>,
    },
    NominalCheck {
        name: String,
    },
    /// DD-TYP-NUM-002/003: `int` → `f64` promotion (not subtyping).
    ///
    /// Accepts `int` / legacy `number` / already-`f64` values and normalizes to `f64`.
    NumericPromote,
    Compose(Vec<CastEvidence>),
}

/// Three-valued algorithmic subtype result (DD-TYP-ALG-002).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecideResult {
    Proved,
    Disproved,
    Unknown,
}

/// Diagnostic taxonomy for algorithmic judgments (DD-TYP-ALG-003).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeDiagClass {
    TypeError,
    AnnotationRequired,
    CheckerLimitation,
    CheckerResourceLimit,
    UnsupportedLanguageFeature,
}

/// Map an internal three-valued decide result to a user-facing diagnostic class.
pub fn classify_decide(result: DecideResult) -> Option<TypeDiagClass> {
    match result {
        DecideResult::Proved => None,
        DecideResult::Disproved => Some(TypeDiagClass::TypeError),
        DecideResult::Unknown => Some(TypeDiagClass::CheckerLimitation),
    }
}

/// Diagnostic provenance for a cast site (DD-TYP-DYN-018). Kept separate from
/// executable evidence so optimizations cannot erase introduction/use sites.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CastProvenance {
    pub cast_id: u32,
    pub source: CoreType,
    pub target: CoreType,
    pub introduction: Option<String>,
    pub module_path: Option<String>,
}

/// Three-way judgment for using `dynamic S` where static `T` is required (DD-TYP-DYN-005).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynamicUseJudgment {
    /// `S <: T` — no runtime check.
    FullyIncluded,
    /// `intersect(S, T) ≃ never` — static reject.
    Disjoint,
    /// Partial overlap — insert runtime cast; success type is `intersect(S, T)`.
    PartialOverlap {
        evidence: CastEvidence,
        success: CoreType,
    },
}

/// Plan evidence for casting `src` to `dst` at compile time.
///
/// Returns `None` when the cast is statically impossible (`intersect(S,T) ≃ never`).
pub fn plan_cast_evidence(src: &CoreType, dst: &CoreType) -> Option<CastEvidence> {
    let src = normalize_type(src);
    let dst = normalize_type(dst);
    if type_eq(&src, &dst) {
        return Some(CastEvidence::Identity);
    }
    if matches!(dst, CoreType::Never) || matches!(src, CoreType::Never) {
        return None;
    }
    // DD-TYP-NUM-002: int→f64 is promotion, not semantic subtyping.
    if let Some(ev) = plan_numeric_promote_evidence(&src, &dst) {
        return Some(ev);
    }
    // static → dynamic S when src <: S (DD-TYP-DYN-008); otherwise reject.
    if let CoreType::Dynamic(bound) = &dst {
        return if is_subtype(&src, bound) {
            Some(CastEvidence::Widen)
        } else {
            None
        };
    }
    // dynamic S → T: three-way (DD-TYP-DYN-005).
    if let CoreType::Dynamic(bound) = &src {
        return match judge_dynamic_use(bound, &dst) {
            DynamicUseJudgment::FullyIncluded => Some(CastEvidence::Identity),
            DynamicUseJudgment::Disjoint => None,
            DynamicUseJudgment::PartialOverlap { evidence, .. } => Some(evidence),
        };
    }
    // static → static narrowing when overlap exists.
    if is_subtype(&src, &dst) {
        return Some(CastEvidence::Identity);
    }
    if types_disjoint(&src, &dst) {
        return None;
    }
    plan_structural_check(&dst)
}

/// DD-TYP-DYN-005 three-way judgment for bound `S` needed as static `T`.
pub fn judge_dynamic_use(bound: &CoreType, needed: &CoreType) -> DynamicUseJudgment {
    let bound = normalize_type(bound);
    let needed = normalize_type(needed);
    if is_subtype(&bound, &needed) {
        return DynamicUseJudgment::FullyIncluded;
    }
    // DD-TYP-NUM-003: narrow then promote — promotion is checked before disjointness,
    // because `int` and `f64` are disjoint under subtyping.
    if let Some(evidence) = plan_numeric_promote_evidence(&bound, &needed) {
        return DynamicUseJudgment::PartialOverlap {
            evidence,
            success: needed.clone(),
        };
    }
    if types_disjoint(&bound, &needed) {
        return DynamicUseJudgment::Disjoint;
    }
    let success = cast_success_type(&bound, &needed);
    let evidence = plan_structural_check(&needed).unwrap_or(CastEvidence::TagCheck {
        tag: type_tag_name(&needed),
    });
    DynamicUseJudgment::PartialOverlap { evidence, success }
}

/// DD-TYP-NUM-002/003: evidence when `needed` is `f64` and `src` can supply an int/f64.
fn plan_numeric_promote_evidence(src: &CoreType, needed: &CoreType) -> Option<CastEvidence> {
    if !matches!(needed, CoreType::F64) {
        return None;
    }
    match src {
        CoreType::F64 => Some(CastEvidence::Identity),
        CoreType::Int | CoreType::Number | CoreType::Any => Some(CastEvidence::NumericPromote),
        CoreType::Singleton(crate::ty::SingletonValue::Int(_)) => {
            Some(CastEvidence::NumericPromote)
        }
        CoreType::Union(members) => {
            let numeric: Vec<CoreType> = members
                .iter()
                .filter(|m| {
                    matches!(
                        m,
                        CoreType::Int
                            | CoreType::F64
                            | CoreType::Number
                            | CoreType::Singleton(crate::ty::SingletonValue::Int(_))
                    )
                })
                .cloned()
                .collect();
            if numeric.is_empty() {
                None
            } else if numeric.len() == members.len() {
                Some(CastEvidence::NumericPromote)
            } else {
                Some(CastEvidence::Compose(vec![
                    CastEvidence::UnionCheck { members: numeric },
                    CastEvidence::NumericPromote,
                ]))
            }
        }
        _ => None,
    }
}

/// DD-TYP-ALG-002: algorithmic `decide-subtype(s, t)`.
///
/// Uses the approximate static subtype / disjointness algebra. Types outside the
/// fully-decidable fragment yield [`DecideResult::Unknown`].
pub fn decide_subtype(s: &CoreType, t: &CoreType) -> DecideResult {
    if is_subtype(s, t) {
        return DecideResult::Proved;
    }
    if types_disjoint(s, t) {
        return DecideResult::Disproved;
    }
    if is_fully_decidable_fragment(s) && is_fully_decidable_fragment(t) {
        // Complete fragment: not subtype and not empty-diff means false.
        DecideResult::Disproved
    } else {
        DecideResult::Unknown
    }
}

fn is_fully_decidable_fragment(ty: &CoreType) -> bool {
    match normalize_type(ty) {
        CoreType::Never
        | CoreType::Any
        | CoreType::Int
        | CoreType::F64
        | CoreType::Number
        | CoreType::String
        | CoreType::Bool
        | CoreType::Unit
        | CoreType::Bytes
        | CoreType::Color
        | CoreType::Shape
        | CoreType::Singleton(_) => true,
        CoreType::Union(ms) | CoreType::Intersect(ms) => ms.iter().all(is_fully_decidable_fragment),
        CoreType::Not(inner) | CoreType::OptionalField(inner) => {
            is_fully_decidable_fragment(&inner)
        }
        CoreType::Diff(a, b) => is_fully_decidable_fragment(&a) && is_fully_decidable_fragment(&b),
        CoreType::Record { fields } => fields.iter().all(|(_, t)| is_fully_decidable_fragment(t)),
        CoreType::Variant { variants } => variants
            .iter()
            .all(|(_, p)| p.as_ref().is_none_or(is_fully_decidable_fragment)),
        CoreType::Fun { args, ret, .. } => {
            args.iter().all(is_fully_decidable_fragment) && is_fully_decidable_fragment(&ret)
        }
        // Open rows, vars, forall, apps: outside complete fragment A.
        _ => false,
    }
}

/// DD-TYP-DYN-006: cast success type is `intersect(S, T)` (normalized).
pub fn cast_success_type(bound: &CoreType, needed: &CoreType) -> CoreType {
    normalize_type(&intersect_types(bound, needed))
}

/// DD-TYP-DYN-017: compose evidence sequentially, then simplify.
pub fn compose_evidence(parts: Vec<CastEvidence>) -> CastEvidence {
    simplify_evidence(CastEvidence::Compose(parts))
}

/// DD-TYP-DYN-017 identity/widen composition shortcuts (provenance preserved by caller).
pub fn simplify_evidence(ev: CastEvidence) -> CastEvidence {
    match ev {
        CastEvidence::Compose(parts) => {
            let mut out = Vec::new();
            for p in parts {
                let p = simplify_evidence(p);
                match p {
                    CastEvidence::Identity => {}
                    CastEvidence::Compose(inner) => out.extend(inner),
                    other => out.push(other),
                }
            }
            match out.len() {
                0 => CastEvidence::Identity,
                1 => out.pop().expect("len 1"),
                _ => CastEvidence::Compose(out),
            }
        }
        other => other,
    }
}

/// DD-TYP-DYN-012: structural / tagged types that may be validated at a boundary.
pub fn is_runtime_checkable(ty: &CoreType) -> bool {
    match normalize_type(ty) {
        CoreType::Bool
        | CoreType::Unit
        | CoreType::Int
        | CoreType::F64
        | CoreType::Number
        | CoreType::String
        | CoreType::Bytes
        | CoreType::Any => true,
        CoreType::Never => false,
        CoreType::Dynamic(b) => is_runtime_checkable(&b),
        CoreType::Union(ms) | CoreType::Intersect(ms) => {
            !ms.is_empty() && ms.iter().all(is_runtime_checkable)
        }
        CoreType::Record { fields } => fields.iter().all(|(_, t)| is_runtime_checkable(t)),
        CoreType::Variant { variants } => variants
            .iter()
            .all(|(_, p)| p.as_ref().is_none_or(is_runtime_checkable)),
        CoreType::Fun { args, ret, effects } => {
            // Stage-1/2 fixed-arity: known latent effects + checkable args/ret.
            effects.ops.iter().all(|op| !op.is_empty())
                && args.iter().all(is_runtime_checkable)
                && is_runtime_checkable(&ret)
        }
        CoreType::App { .. } => true,
        CoreType::Singleton(_) => true,
        CoreType::Not(_) | CoreType::Diff(_, _) => false,
        _ => false,
    }
}

/// Approximate static subtype used by gradual planning (not full semantic subtyping).
pub fn is_subtype(a: &CoreType, b: &CoreType) -> bool {
    let a = normalize_type(a);
    let b = normalize_type(b);
    if type_eq(&a, &b) {
        return true;
    }
    match (&a, &b) {
        (CoreType::Never, _) => true,
        (_, CoreType::Any) => true,
        (CoreType::Any, _) => false,
        (CoreType::Dynamic(sa), CoreType::Dynamic(sb)) => is_subtype(sa, sb),
        (CoreType::Int | CoreType::F64, CoreType::Number) => true,
        (CoreType::Number, CoreType::Int | CoreType::F64) => false,
        (CoreType::Singleton(v), other) => {
            is_subtype(&CoreType::singleton_domain(v), other)
                || matches!(
                    (v, other),
                    (crate::ty::SingletonValue::Int(_), CoreType::Number)
                )
        }
        (CoreType::OptionalField(inner), CoreType::OptionalField(other)) => {
            is_subtype(inner, other)
        }
        (inner, CoreType::OptionalField(other)) => is_subtype(inner, other),
        (CoreType::Union(ms), other) => ms.iter().all(|m| is_subtype(m, other)),
        (other, CoreType::Union(ms)) => ms.iter().any(|m| is_subtype(other, m)),
        (CoreType::Intersect(ms), other) => ms.iter().any(|m| is_subtype(m, other)),
        (other, CoreType::Intersect(ms)) => ms.iter().all(|m| is_subtype(other, m)),
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
            a_args.len() == b_args.len()
                && a_args
                    .iter()
                    .zip(b_args.iter())
                    .all(|(x, y)| is_subtype(y, x)) // contravariant args
                && is_subtype(a_ret, b_ret)
                && effect_subrow(a_eff, b_eff)
        }
        (CoreType::Record { fields: a_f }, CoreType::Record { fields: b_f }) => {
            record_fields_subtype(a_f, b_f)
        }
        (CoreType::Variant { variants: a_v }, CoreType::Variant { variants: b_v }) => {
            a_v.iter().all(|(at, ap)| {
                b_v.iter().any(|(bt, bp)| {
                    at == bt
                        && match (ap, bp) {
                            (None, None) => true,
                            (Some(a_t), Some(b_t)) => is_subtype(a_t, b_t),
                            _ => false,
                        }
                })
            })
        }
        _ => false,
    }
}

/// DD-TYP-ROW-007: required fields subtype optional; absence of an optional is ok.
fn record_fields_subtype(a_f: &[(String, CoreType)], b_f: &[(String, CoreType)]) -> bool {
    // Every required field in `b` must appear in `a`; optional fields may be absent.
    b_f.iter().all(|(bk, bv)| {
        if let Some((_, av)) = a_f.iter().find(|(ak, _)| ak == bk) {
            is_subtype(av, bv)
        } else {
            matches!(bv, CoreType::OptionalField(_))
        }
    }) && a_f.iter().all(|(ak, _)| b_f.iter().any(|(bk, _)| ak == bk))
}

/// `intersect(S, T) ≃ never` under the same approximate algebra.
pub fn types_disjoint(a: &CoreType, b: &CoreType) -> bool {
    matches!(normalize_type(&intersect_types(a, b)), CoreType::Never)
}

pub fn intersect_types(a: &CoreType, b: &CoreType) -> CoreType {
    let a = normalize_type(a);
    let b = normalize_type(b);
    if type_eq(&a, &b) {
        return a;
    }
    match (&a, &b) {
        (CoreType::Never, _) | (_, CoreType::Never) => CoreType::Never,
        (CoreType::Any, other) | (other, CoreType::Any) => other.clone(),
        (CoreType::Dynamic(sa), other) | (other, CoreType::Dynamic(sa)) => {
            CoreType::dynamic_bound(intersect_types(sa, other))
        }
        (CoreType::Int, CoreType::Number) | (CoreType::Number, CoreType::Int) => CoreType::Int,
        (CoreType::F64, CoreType::Number) | (CoreType::Number, CoreType::F64) => CoreType::F64,
        (CoreType::Int, CoreType::F64) | (CoreType::F64, CoreType::Int) => CoreType::Never,
        (CoreType::Singleton(sa), CoreType::Singleton(sb)) => {
            if sa == sb {
                CoreType::Singleton(sa.clone())
            } else {
                CoreType::Never
            }
        }
        (CoreType::Singleton(s), other) | (other, CoreType::Singleton(s)) => {
            let domain = CoreType::singleton_domain(s);
            let it = intersect_types(&domain, other);
            if matches!(it, CoreType::Never) {
                CoreType::Never
            } else if type_eq(&it, &domain) {
                CoreType::Singleton(s.clone())
            } else {
                // e.g. singleton int ∩ number → singleton
                CoreType::Singleton(s.clone())
            }
        }
        (CoreType::OptionalField(a_i), CoreType::OptionalField(b_i)) => {
            let it = intersect_types(a_i, b_i);
            if matches!(it, CoreType::Never) {
                CoreType::Never
            } else {
                CoreType::OptionalField(Box::new(it))
            }
        }
        (CoreType::Union(ms), other) => {
            let parts: Vec<_> = ms
                .iter()
                .map(|m| intersect_types(m, other))
                .filter(|t| !matches!(t, CoreType::Never))
                .collect();
            normalize_union(parts)
        }
        (other, CoreType::Union(ms)) => {
            let parts: Vec<_> = ms
                .iter()
                .map(|m| intersect_types(other, m))
                .filter(|t| !matches!(t, CoreType::Never))
                .collect();
            normalize_union(parts)
        }
        (CoreType::Intersect(ms), other) => {
            let mut all = ms.clone();
            all.push(other.clone());
            normalize_intersect(all)
        }
        (other, CoreType::Intersect(ms)) => {
            let mut all = ms.clone();
            all.push(other.clone());
            normalize_intersect(all)
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
        ) if a_args.len() == b_args.len() && effect_subrow(a_eff, b_eff) => {
            // Arg intersection is union (contravariant); result is intersect.
            let args: Vec<_> = a_args
                .iter()
                .zip(b_args.iter())
                .map(|(x, y)| normalize_union(vec![x.clone(), y.clone()]))
                .collect();
            CoreType::Fun {
                args,
                ret: Box::new(intersect_types(a_ret, b_ret)),
                effects: a_eff.clone(),
            }
        }
        (CoreType::Record { fields: a_f }, CoreType::Record { fields: b_f }) => {
            // Require same labels; intersect field types.
            if a_f.len() != b_f.len() {
                return CoreType::Never;
            }
            let mut fields = Vec::new();
            for (ak, av) in a_f {
                let Some((_, bv)) = b_f.iter().find(|(bk, _)| bk == ak) else {
                    return CoreType::Never;
                };
                let it = intersect_types(av, bv);
                if matches!(it, CoreType::Never) {
                    return CoreType::Never;
                }
                fields.push((ak.clone(), it));
            }
            CoreType::Record { fields }
        }
        (CoreType::Variant { variants: a_v }, CoreType::Variant { variants: b_v }) => {
            let mut variants = Vec::new();
            for (at, ap) in a_v {
                if let Some((_, bp)) = b_v.iter().find(|(bt, _)| bt == at) {
                    match (ap, bp) {
                        (None, None) => variants.push((at.clone(), None)),
                        (Some(a_t), Some(b_t)) => {
                            let it = intersect_types(a_t, b_t);
                            if !matches!(it, CoreType::Never) {
                                variants.push((at.clone(), Some(it)));
                            }
                        }
                        _ => {}
                    }
                }
            }
            if variants.is_empty() {
                CoreType::Never
            } else {
                CoreType::Variant { variants }
            }
        }
        // Distinct concrete bases with no shared values.
        (
            CoreType::Int
            | CoreType::F64
            | CoreType::Number
            | CoreType::String
            | CoreType::Bool
            | CoreType::Unit
            | CoreType::Bytes
            | CoreType::Color
            | CoreType::Shape,
            CoreType::Int
            | CoreType::F64
            | CoreType::Number
            | CoreType::String
            | CoreType::Bool
            | CoreType::Unit
            | CoreType::Bytes
            | CoreType::Color
            | CoreType::Shape,
        ) => CoreType::Never,
        // Conservative: keep an intersection node for unresolved shapes.
        _ => CoreType::Intersect(vec![a, b]),
    }
}

pub fn normalize_type(ty: &CoreType) -> CoreType {
    match ty {
        CoreType::Dynamic(inner) => {
            let inner = normalize_type(inner);
            match inner {
                CoreType::Never => CoreType::Never,
                CoreType::Dynamic(nested) => CoreType::Dynamic(nested),
                other => CoreType::Dynamic(Box::new(other)),
            }
        }
        CoreType::Union(ms) => normalize_union(ms.iter().map(normalize_type).collect()),
        CoreType::Intersect(ms) => normalize_intersect(ms.iter().map(normalize_type).collect()),
        CoreType::Diff(a, b) => {
            // diff(S,T) ≃ intersect(S, not(T)) — keep when both survive.
            let a = normalize_type(a);
            let b = normalize_type(b);
            if is_subtype(&a, &b) {
                CoreType::Never
            } else {
                CoreType::Diff(Box::new(a), Box::new(b))
            }
        }
        other => other.clone(),
    }
}

fn normalize_union(members: Vec<CoreType>) -> CoreType {
    let mut out = Vec::new();
    for m in members {
        let m = normalize_type(&m);
        match m {
            CoreType::Never => {}
            CoreType::Union(inner) => {
                for i in inner {
                    if !out.iter().any(|e| type_eq(e, &i)) {
                        out.push(i);
                    }
                }
            }
            other => {
                if !out.iter().any(|e| type_eq(e, &other)) {
                    out.push(other);
                }
            }
        }
    }
    match out.len() {
        0 => CoreType::Never,
        1 => out.pop().expect("len 1"),
        _ => CoreType::Union(out),
    }
}

fn normalize_intersect(members: Vec<CoreType>) -> CoreType {
    let mut out = Vec::new();
    for m in members {
        let m = normalize_type(&m);
        match m {
            CoreType::Any => {}
            CoreType::Never => return CoreType::Never,
            CoreType::Intersect(inner) => {
                for i in inner {
                    if !out.iter().any(|e| type_eq(e, &i)) {
                        out.push(i);
                    }
                }
            }
            other => {
                if !out.iter().any(|e| type_eq(e, &other)) {
                    out.push(other);
                }
            }
        }
    }
    // Pairwise collapse known concrete clashes.
    for i in 0..out.len() {
        for j in (i + 1)..out.len() {
            if types_disjoint_bases(&out[i], &out[j]) {
                return CoreType::Never;
            }
        }
    }
    match out.len() {
        0 => CoreType::Any,
        1 => out.pop().expect("len 1"),
        _ => CoreType::Intersect(out),
    }
}

fn types_disjoint_bases(a: &CoreType, b: &CoreType) -> bool {
    matches!(
        (a, b),
        (
            CoreType::String
                | CoreType::Bool
                | CoreType::Unit
                | CoreType::Bytes
                | CoreType::Color
                | CoreType::Shape,
            CoreType::String
                | CoreType::Bool
                | CoreType::Unit
                | CoreType::Bytes
                | CoreType::Color
                | CoreType::Shape
        ) if !type_eq(a, b)
    ) || matches!(
        (a, b),
        (CoreType::Int, CoreType::F64) | (CoreType::F64, CoreType::Int)
    ) || matches!(
        (a, b),
        (
            CoreType::Int | CoreType::F64 | CoreType::Number,
            CoreType::String
                | CoreType::Bool
                | CoreType::Unit
                | CoreType::Bytes
                | CoreType::Color
                | CoreType::Shape
        ) | (
            CoreType::String
                | CoreType::Bool
                | CoreType::Unit
                | CoreType::Bytes
                | CoreType::Color
                | CoreType::Shape,
            CoreType::Int | CoreType::F64 | CoreType::Number
        )
    )
}

fn effect_subrow(src: &EffectRow, dst: &EffectRow) -> bool {
    // Es ⊑ Et: every source op appears in the target ambient allowance.
    src.ops.iter().all(|op| dst.ops.iter().any(|d| d == op))
}

fn type_eq(a: &CoreType, b: &CoreType) -> bool {
    a == b
}

fn plan_structural_check(dst: &CoreType) -> Option<CastEvidence> {
    match dst {
        CoreType::Never => None,
        CoreType::Union(members) => Some(CastEvidence::UnionCheck {
            members: members.clone(),
        }),
        CoreType::Intersect(members) => Some(CastEvidence::IntersectionCheck {
            members: members.clone(),
        }),
        CoreType::Record { fields } => Some(CastEvidence::RecordCheck {
            fields: fields.clone(),
        }),
        CoreType::Variant { variants } => Some(CastEvidence::VariantCheck {
            variants: variants.clone(),
        }),
        CoreType::Fun { args, ret, .. } => {
            let arg_casts = args
                .iter()
                .map(|a| plan_cast_evidence(a, &CoreType::dyn_any()).unwrap_or(CastEvidence::Widen))
                .collect();
            let ret_cast =
                plan_cast_evidence(&CoreType::dyn_any(), ret).unwrap_or(CastEvidence::Identity);
            Some(CastEvidence::FunctionGuard {
                arity: args.len(),
                arg_casts,
                ret_cast: Box::new(ret_cast),
            })
        }
        CoreType::App { ctor, .. } => Some(CastEvidence::NominalCheck { name: ctor.clone() }),
        CoreType::Int
        | CoreType::F64
        | CoreType::Number
        | CoreType::String
        | CoreType::Bool
        | CoreType::Unit
        | CoreType::Bytes
        | CoreType::Any
        | CoreType::Singleton(_) => Some(CastEvidence::TagCheck {
            tag: type_tag_name(dst),
        }),
        CoreType::Dynamic(_) => Some(CastEvidence::Identity),
        _ => Some(CastEvidence::TagCheck {
            tag: type_tag_name(dst),
        }),
    }
}

fn type_tag_name(ty: &CoreType) -> String {
    match ty {
        CoreType::Int => "int".into(),
        CoreType::F64 => "f64".into(),
        CoreType::Number => "number".into(),
        CoreType::Singleton(crate::ty::SingletonValue::Int(_)) => "int".into(),
        CoreType::Singleton(crate::ty::SingletonValue::Bool(_)) => "bool".into(),
        CoreType::Singleton(crate::ty::SingletonValue::String(_)) => "string".into(),
        CoreType::Singleton(crate::ty::SingletonValue::Unit) => "unit".into(),
        CoreType::String => "string".into(),
        CoreType::Bool => "bool".into(),
        CoreType::Unit => "unit".into(),
        CoreType::Bytes => "bytes".into(),
        CoreType::Any => "any".into(),
        CoreType::Dynamic(_) => "dynamic".into(),
        CoreType::App { ctor, .. } => ctor.clone(),
        _ => "value".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_cast() {
        assert_eq!(
            plan_cast_evidence(&CoreType::Int, &CoreType::Int),
            Some(CastEvidence::Identity)
        );
    }

    #[test]
    fn dynamic_to_int_tag_check() {
        assert!(matches!(
            plan_cast_evidence(&CoreType::dyn_any(), &CoreType::Int),
            Some(CastEvidence::TagCheck { tag }) if tag == "int"
        ));
    }

    #[test]
    fn never_cast_impossible() {
        assert!(plan_cast_evidence(&CoreType::dyn_any(), &CoreType::Never).is_none());
    }

    #[test]
    fn three_way_fully_included() {
        assert_eq!(
            judge_dynamic_use(&CoreType::Int, &CoreType::Number),
            DynamicUseJudgment::FullyIncluded
        );
    }

    #[test]
    fn three_way_disjoint() {
        assert_eq!(
            judge_dynamic_use(
                &CoreType::Union(vec![CoreType::Number, CoreType::String]),
                &CoreType::Bool
            ),
            DynamicUseJudgment::Disjoint
        );
    }

    #[test]
    fn three_way_partial_overlap_success_is_intersect() {
        match judge_dynamic_use(
            &CoreType::Union(vec![CoreType::Int, CoreType::String]),
            &CoreType::Number,
        ) {
            DynamicUseJudgment::PartialOverlap { success, .. } => {
                assert_eq!(success, CoreType::Int);
            }
            other => panic!("expected partial, got {other:?}"),
        }
    }

    #[test]
    fn compose_drops_identity() {
        assert_eq!(
            compose_evidence(vec![
                CastEvidence::Identity,
                CastEvidence::TagCheck { tag: "int".into() },
                CastEvidence::Identity,
            ]),
            CastEvidence::TagCheck { tag: "int".into() }
        );
    }

    #[test]
    fn dynamic_never_normalizes_to_never() {
        assert_eq!(
            normalize_type(&CoreType::Dynamic(Box::new(CoreType::Never))),
            CoreType::Never
        );
    }

    #[test]
    fn int_to_f64_is_numeric_promote() {
        assert_eq!(
            plan_cast_evidence(&CoreType::Int, &CoreType::F64),
            Some(CastEvidence::NumericPromote)
        );
    }

    #[test]
    fn dynamic_int_to_f64_promotes_not_disjoint() {
        match judge_dynamic_use(&CoreType::Int, &CoreType::F64) {
            DynamicUseJudgment::PartialOverlap { evidence, success } => {
                assert_eq!(evidence, CastEvidence::NumericPromote);
                assert_eq!(success, CoreType::F64);
            }
            other => panic!("expected promote partial, got {other:?}"),
        }
    }

    #[test]
    fn decide_subtype_int_number_proved() {
        assert_eq!(
            decide_subtype(&CoreType::Int, &CoreType::Number),
            DecideResult::Proved
        );
        assert_eq!(classify_decide(DecideResult::Proved), None);
        assert_eq!(
            decide_subtype(&CoreType::String, &CoreType::Number),
            DecideResult::Disproved
        );
        assert_eq!(
            classify_decide(DecideResult::Disproved),
            Some(TypeDiagClass::TypeError)
        );
    }

    #[test]
    fn singleton_subtypes_domain() {
        let s = CoreType::Singleton(crate::ty::SingletonValue::Int(42));
        assert!(is_subtype(&s, &CoreType::Int));
        assert!(is_subtype(&s, &CoreType::Number));
        assert!(!is_subtype(&CoreType::Int, &s));
    }

    #[test]
    fn optional_field_required_subtypes_optional() {
        let req = CoreType::Record {
            fields: vec![("author".into(), CoreType::String)],
        };
        let opt = CoreType::Record {
            fields: vec![(
                "author".into(),
                CoreType::OptionalField(Box::new(CoreType::String)),
            )],
        };
        assert!(is_subtype(&req, &opt));
        assert!(!is_subtype(&opt, &req));
        let empty = CoreType::Record { fields: vec![] };
        let only_opt = CoreType::Record {
            fields: vec![(
                "author".into(),
                CoreType::OptionalField(Box::new(CoreType::String)),
            )],
        };
        assert!(is_subtype(&empty, &only_opt));
    }

    #[test]
    fn fixed_arity_function_subtyping_contravariant_args() {
        use crate::ty::EffectRow;
        let number_to_int = CoreType::Fun {
            args: vec![CoreType::Number],
            ret: Box::new(CoreType::Int),
            effects: EffectRow::default(),
        };
        let int_to_number = CoreType::Fun {
            args: vec![CoreType::Int],
            ret: Box::new(CoreType::Number),
            effects: EffectRow::default(),
        };
        // (fn Number Int) <: (fn Int Number) — arg contra, ret cov.
        assert!(is_subtype(&number_to_int, &int_to_number));
        assert!(!is_subtype(&int_to_number, &number_to_int));
    }

    #[test]
    fn numeric_promote_and_intersect_residuals() {
        assert!(plan_numeric_promote_evidence(&CoreType::F64, &CoreType::F64).is_some());
        assert!(plan_numeric_promote_evidence(&CoreType::Int, &CoreType::F64).is_some());
        assert!(plan_numeric_promote_evidence(&CoreType::Number, &CoreType::F64).is_some());
        assert!(plan_numeric_promote_evidence(&CoreType::Any, &CoreType::F64).is_some());
        assert!(plan_numeric_promote_evidence(
            &CoreType::Singleton(crate::ty::SingletonValue::Int(1)),
            &CoreType::F64
        )
        .is_some());
        assert!(plan_numeric_promote_evidence(
            &CoreType::Union(vec![CoreType::Int, CoreType::F64]),
            &CoreType::F64
        )
        .is_some());
        assert!(plan_numeric_promote_evidence(
            &CoreType::Union(vec![CoreType::Int, CoreType::String]),
            &CoreType::F64
        )
        .is_some());
        assert!(plan_numeric_promote_evidence(
            &CoreType::Union(vec![CoreType::String]),
            &CoreType::F64
        )
        .is_none());
        assert!(plan_numeric_promote_evidence(&CoreType::Int, &CoreType::Int).is_none());

        let _ = intersect_types(
            &CoreType::Singleton(crate::ty::SingletonValue::Int(1)),
            &CoreType::Singleton(crate::ty::SingletonValue::Int(2)),
        );
        let _ = intersect_types(
            &CoreType::Singleton(crate::ty::SingletonValue::Int(1)),
            &CoreType::Number,
        );
        let _ = intersect_types(
            &CoreType::OptionalField(Box::new(CoreType::Int)),
            &CoreType::OptionalField(Box::new(CoreType::Int)),
        );
        let _ = intersect_types(
            &CoreType::OptionalField(Box::new(CoreType::Int)),
            &CoreType::OptionalField(Box::new(CoreType::String)),
        );
        let _ = intersect_types(
            &CoreType::Union(vec![CoreType::Int, CoreType::String]),
            &CoreType::Int,
        );
        let _ = intersect_types(
            &CoreType::Int,
            &CoreType::Union(vec![CoreType::Int, CoreType::String]),
        );
        let _ = intersect_types(
            &CoreType::Intersect(vec![CoreType::Int, CoreType::Number]),
            &CoreType::Int,
        );
        let _ = intersect_types(
            &CoreType::Int,
            &CoreType::Intersect(vec![CoreType::Int, CoreType::Number]),
        );
        let _ = intersect_types(
            &CoreType::Fun {
                args: vec![CoreType::Int],
                ret: Box::new(CoreType::Int),
                effects: EffectRow::default(),
            },
            &CoreType::Fun {
                args: vec![CoreType::Int],
                ret: Box::new(CoreType::Number),
                effects: EffectRow::default(),
            },
        );
        assert!(is_fully_decidable_fragment(&CoreType::Int));
        assert!(!is_fully_decidable_fragment(&CoreType::Var(
            crate::ty::TypeVarId(0)
        )));
        // Record / Variant / Fun / Diff / Not recursion residual
        assert!(is_fully_decidable_fragment(&CoreType::Record {
            fields: vec![("a".into(), CoreType::Int)],
        }));
        assert!(is_fully_decidable_fragment(&CoreType::Record {
            fields: vec![],
        }));
        assert!(!is_fully_decidable_fragment(&CoreType::Record {
            fields: vec![("a".into(), CoreType::Var(crate::ty::TypeVarId(0)))],
        }));
        assert!(is_fully_decidable_fragment(&CoreType::Variant {
            variants: vec![("none".into(), None), ("some".into(), Some(CoreType::Int)),],
        }));
        assert!(!is_fully_decidable_fragment(&CoreType::Variant {
            variants: vec![("x".into(), Some(CoreType::Var(crate::ty::TypeVarId(1))))],
        }));
        assert!(is_fully_decidable_fragment(&CoreType::Fun {
            args: vec![CoreType::Int],
            ret: Box::new(CoreType::Bool),
            effects: EffectRow::default(),
        }));
        assert!(!is_fully_decidable_fragment(&CoreType::Fun {
            args: vec![CoreType::Var(crate::ty::TypeVarId(0))],
            ret: Box::new(CoreType::Int),
            effects: EffectRow::default(),
        }));
        assert!(is_fully_decidable_fragment(&CoreType::Diff(
            Box::new(CoreType::Number),
            Box::new(CoreType::Int)
        )));
        assert!(!is_fully_decidable_fragment(&CoreType::Diff(
            Box::new(CoreType::Number),
            Box::new(CoreType::Var(crate::ty::TypeVarId(0)))
        )));
        assert!(is_fully_decidable_fragment(&CoreType::Not(Box::new(
            CoreType::String
        ))));
        assert!(is_fully_decidable_fragment(&CoreType::OptionalField(
            Box::new(CoreType::Bool)
        )));
        assert!(is_fully_decidable_fragment(&CoreType::Union(vec![
            CoreType::Int,
            CoreType::String
        ])));
        assert!(is_fully_decidable_fragment(&CoreType::Intersect(vec![
            CoreType::Int,
            CoreType::Number
        ])));
        assert!(!is_fully_decidable_fragment(&CoreType::Union(vec![
            CoreType::Int,
            CoreType::Var(crate::ty::TypeVarId(0))
        ])));
        let _ = normalize_union(vec![CoreType::Int, CoreType::Int, CoreType::Never]);
        let _ = normalize_intersect(vec![CoreType::Int, CoreType::Any, CoreType::Number]);
        let _ = normalize_intersect(vec![
            CoreType::Intersect(vec![CoreType::Int, CoreType::Number]),
            CoreType::Int,
        ]);
        let _ = normalize_intersect(vec![CoreType::Int, CoreType::Int]);
        let _ = types_disjoint_bases(&CoreType::Int, &CoreType::String);
        let _ = type_tag_name(&CoreType::Int);
        let _ = type_tag_name(&CoreType::App {
            ctor: "opt".into(),
            args: vec![CoreType::Int],
        });
        let _ = plan_structural_check(&CoreType::Record {
            fields: vec![("a".into(), CoreType::Int)],
        });
        let _ = plan_structural_check(&CoreType::Variant {
            variants: vec![("none".into(), None)],
        });
        // Broader residual for structural / decidable / tags / normalize clashes
        for dst in [
            CoreType::Never,
            CoreType::Union(vec![CoreType::Int, CoreType::String]),
            CoreType::Intersect(vec![CoreType::Int, CoreType::Number]),
            CoreType::Fun {
                args: vec![CoreType::Int, CoreType::String],
                ret: Box::new(CoreType::Bool),
                effects: EffectRow::default(),
            },
            CoreType::App {
                ctor: "box".into(),
                args: vec![CoreType::Int],
            },
            CoreType::F64,
            CoreType::Number,
            CoreType::String,
            CoreType::Bool,
            CoreType::Unit,
            CoreType::Bytes,
            CoreType::Any,
            CoreType::Singleton(crate::ty::SingletonValue::Int(1)),
            CoreType::Singleton(crate::ty::SingletonValue::Bool(true)),
            CoreType::Singleton(crate::ty::SingletonValue::String("x".into())),
            CoreType::Singleton(crate::ty::SingletonValue::Unit),
            CoreType::dyn_any(),
            CoreType::Diff(Box::new(CoreType::Number), Box::new(CoreType::Int)),
            CoreType::Not(Box::new(CoreType::String)),
            CoreType::OptionalField(Box::new(CoreType::Int)),
            CoreType::Color,
            CoreType::Shape,
            CoreType::OpenRecord {
                fields: vec![],
                row: Box::new(CoreType::Unit),
            },
            CoreType::Name("t".into()),
            CoreType::Forall {
                params: vec![("a".into(), "type".into())],
                body: Box::new(CoreType::Name("a".into())),
            },
            CoreType::Lacks {
                label: "a".into(),
                row: Box::new(CoreType::Unit),
            },
            CoreType::Error,
        ] {
            let _ = plan_structural_check(&dst);
            let _ = is_fully_decidable_fragment(&dst);
            let _ = type_tag_name(&dst);
            let _ = is_runtime_checkable(&dst);
            let _ = plan_cast_evidence(&CoreType::dyn_any(), &dst);
            let _ = plan_cast_evidence(&dst, &CoreType::dyn_any());
            let _ = decide_subtype(&dst, &CoreType::Any);
            let _ = types_disjoint(&dst, &CoreType::Never);
        }
        // normalize_intersect clash → Never
        let _ = normalize_intersect(vec![CoreType::Int, CoreType::String]);
        let _ = normalize_intersect(vec![CoreType::Int, CoreType::F64]);
        let _ = normalize_intersect(vec![
            CoreType::Intersect(vec![CoreType::Int]),
            CoreType::Never,
        ]);
        let _ = normalize_intersect(vec![]);
        let _ = normalize_union(vec![]);
        let _ = normalize_union(vec![CoreType::Union(vec![CoreType::Int]), CoreType::Any]);
        let _ = types_disjoint_bases(&CoreType::Bool, &CoreType::Unit);
        let _ = types_disjoint_bases(&CoreType::Color, &CoreType::Shape);
        let _ = types_disjoint_bases(&CoreType::Int, &CoreType::Int);
        let _ = types_disjoint_bases(&CoreType::String, &CoreType::Number);
        // Compose / simplify residual nests
        let _ = simplify_evidence(CastEvidence::Compose(vec![
            CastEvidence::Compose(vec![CastEvidence::Identity, CastEvidence::Widen]),
            CastEvidence::Identity,
            CastEvidence::TagCheck { tag: "int".into() },
            CastEvidence::Widen,
        ]));
        let _ = compose_evidence(vec![]);
        let _ = compose_evidence(vec![CastEvidence::Identity]);
    }

    #[test]
    fn variant_intersect_and_singleton_domain_residuals() {
        // Both payloads Some: keep intersected arm when non-Never.
        let kept = intersect_types(
            &CoreType::Variant {
                variants: vec![
                    ("some".into(), Some(CoreType::Int)),
                    ("none".into(), None),
                ],
            },
            &CoreType::Variant {
                variants: vec![
                    ("some".into(), Some(CoreType::Number)),
                    ("none".into(), None),
                ],
            },
        );
        assert!(matches!(kept, CoreType::Variant { .. }));

        // Payload intersect → Never drops the arm; empty → Never.
        let dropped = intersect_types(
            &CoreType::Variant {
                variants: vec![("x".into(), Some(CoreType::Int))],
            },
            &CoreType::Variant {
                variants: vec![("x".into(), Some(CoreType::String))],
            },
        );
        assert!(matches!(dropped, CoreType::Never));

        // Mismatched Some/None payload skips the arm.
        let skipped = intersect_types(
            &CoreType::Variant {
                variants: vec![("x".into(), Some(CoreType::Int))],
            },
            &CoreType::Variant {
                variants: vec![("x".into(), None)],
            },
        );
        assert!(matches!(skipped, CoreType::Never));

        // Singleton ∩ non-domain → Never; ∩ domain keeps singleton.
        let never = intersect_types(
            &CoreType::Singleton(crate::ty::SingletonValue::Int(1)),
            &CoreType::String,
        );
        assert!(matches!(never, CoreType::Never));
        let keep = intersect_types(
            &CoreType::Singleton(crate::ty::SingletonValue::Int(1)),
            &CoreType::Int,
        );
        assert!(matches!(
            keep,
            CoreType::Singleton(crate::ty::SingletonValue::Int(1))
        ));

        // Nested Compose flatten (inner Compose branch).
        let nested = simplify_evidence(CastEvidence::Compose(vec![
            CastEvidence::Compose(vec![
                CastEvidence::Compose(vec![CastEvidence::Widen]),
                CastEvidence::Identity,
            ]),
            CastEvidence::TagCheck { tag: "num".into() },
        ]));
        assert!(matches!(nested, CastEvidence::Compose(_)));

        // Fun intersect with effect mismatch (exercise effect_subrow / Fun arm).
        let _ = intersect_types(
            &CoreType::Fun {
                args: vec![CoreType::Int],
                ret: Box::new(CoreType::Int),
                effects: crate::ty::EffectRow::default().with_op("ask"),
            },
            &CoreType::Fun {
                args: vec![CoreType::Int],
                ret: Box::new(CoreType::Int),
                effects: crate::ty::EffectRow::default().with_op("log"),
            },
        );

        let _ = types_disjoint_bases(&CoreType::Color, &CoreType::Shape);
        let _ = types_disjoint_bases(&CoreType::Int, &CoreType::F64);
        let _ = types_disjoint(
            &CoreType::Intersect(vec![CoreType::Color, CoreType::Shape]),
            &CoreType::Never,
        );
    }
}
