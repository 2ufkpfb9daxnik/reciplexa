//! DD-TYP-DYN-014/015: cast evidence and runtime check planning.

use crate::ty::CoreType;

/// Cast evidence algebra (DD-TYP-DYN-015).
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
    },
    NominalCheck {
        name: String,
    },
    Compose(Vec<CastEvidence>),
}

/// Plan evidence for casting `src` to `dst` at compile time.
///
/// Returns `None` when the cast is statically impossible (`intersect(S,T) ≃ never`).
pub fn plan_cast_evidence(src: &CoreType, dst: &CoreType) -> Option<CastEvidence> {
    if src == dst {
        return Some(CastEvidence::Identity);
    }
    if matches!(dst, CoreType::Dynamic) {
        return Some(CastEvidence::Widen);
    }
    if matches!(src, CoreType::Dynamic) {
        return plan_dynamic_to_static(dst);
    }
    if let CoreType::Union(members) = dst {
        return Some(CastEvidence::UnionCheck {
            members: members.clone(),
        });
    }
    if let CoreType::Intersect(members) = dst {
        return Some(CastEvidence::IntersectionCheck {
            members: members.clone(),
        });
    }
    if let CoreType::Record { fields } = dst {
        return Some(CastEvidence::RecordCheck {
            fields: fields.clone(),
        });
    }
    if let CoreType::Variant { variants } = dst {
        return Some(CastEvidence::VariantCheck {
            variants: variants.clone(),
        });
    }
    if let CoreType::Fun { args, .. } = dst {
        return Some(CastEvidence::FunctionGuard { arity: args.len() });
    }
    if let CoreType::App { ctor, .. } = dst {
        return Some(CastEvidence::NominalCheck { name: ctor.clone() });
    }
    plan_tag_check(src, dst)
}

fn plan_dynamic_to_static(dst: &CoreType) -> Option<CastEvidence> {
    if matches!(dst, CoreType::Never) {
        return None;
    }
    plan_tag_check(&CoreType::Dynamic, dst)
}

fn plan_tag_check(src: &CoreType, dst: &CoreType) -> Option<CastEvidence> {
    let tag = match dst {
        CoreType::Int => "int",
        CoreType::F64 => "f64",
        CoreType::Number => "number",
        CoreType::String => "string",
        CoreType::Bool => "bool",
        CoreType::Unit => "unit",
        CoreType::Bytes => "bytes",
        CoreType::Any => "any",
        CoreType::Never => return None,
        CoreType::Dynamic => return Some(CastEvidence::Identity),
        _ if matches!(src, CoreType::Dynamic) => {
            return Some(CastEvidence::TagCheck {
                tag: type_tag_name(dst),
            })
        }
        _ => return None,
    };
    if matches!(src, CoreType::Dynamic) || src == dst {
        Some(CastEvidence::TagCheck {
            tag: tag.to_string(),
        })
    } else {
        None
    }
}

fn type_tag_name(ty: &CoreType) -> String {
    match ty {
        CoreType::Int => "int".into(),
        CoreType::F64 => "f64".into(),
        CoreType::Number => "number".into(),
        CoreType::String => "string".into(),
        CoreType::Bool => "bool".into(),
        CoreType::Unit => "unit".into(),
        CoreType::Bytes => "bytes".into(),
        CoreType::Any => "any".into(),
        CoreType::Dynamic => "dynamic".into(),
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
            plan_cast_evidence(&CoreType::Dynamic, &CoreType::Int),
            Some(CastEvidence::TagCheck { tag }) if tag == "int"
        ));
    }

    #[test]
    fn never_cast_impossible() {
        assert!(plan_cast_evidence(&CoreType::Dynamic, &CoreType::Never).is_none());
    }
}
