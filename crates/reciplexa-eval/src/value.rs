//! Runtime values during evaluation.

use std::collections::HashMap;
use std::fmt;

use reciplexa_core::expr::CoreExpr;
use reciplexa_core::ty::CoreType;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Unit,
    Number(f64),
    String(String),
    ShapeTag(String),
    Closure {
        param: String,
        body: CoreExpr,
        env: HashMap<String, RuntimeValue>,
    },
    Record(Vec<(String, RuntimeValue)>),
    Variant {
        tag: String,
        payload: Option<Box<RuntimeValue>>,
    },
}

impl RuntimeValue {
    pub fn ty(&self) -> CoreType {
        match self {
            Self::Unit => CoreType::Unit,
            Self::Number(_) => CoreType::Number,
            Self::String(_) => CoreType::String,
            Self::ShapeTag(_) => CoreType::Shape,
            Self::Closure { .. } => CoreType::Fun {
                args: vec![CoreType::Var(TypeVarId::new(0))],
                ret: Box::new(CoreType::Unit),
                effects: Default::default(),
            },
            Self::Record(fields) => CoreType::Record {
                fields: fields.iter().map(|(k, v)| (k.clone(), v.ty())).collect(),
            },
            Self::Variant { tag, payload } => CoreType::Variant {
                variants: vec![(tag.clone(), payload.as_ref().map(|p| p.ty()))],
            },
        }
    }
}

use reciplexa_core::ty::TypeVarId;

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "unit"),
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "\"{s}\""),
            Self::ShapeTag(s) => write!(f, "shape:{s}"),
            Self::Closure { param, .. } => write!(f, "closure({param})"),
            Self::Record(fields) => {
                write!(f, "record{{")?;
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k}: {v}")?;
                }
                write!(f, "}}")
            }
            Self::Variant { tag, payload } => match payload {
                Some(p) => write!(f, "{tag}({p})"),
                None => write!(f, "{tag}"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_core::expr::CoreExpr;
    use reciplexa_core::ty::{CoreType, EffectRow};

    #[test]
    fn display_all_variants() {
        assert_eq!(RuntimeValue::Unit.to_string(), "unit");
        assert_eq!(RuntimeValue::Number(3.5).to_string(), "3.5");
        assert_eq!(RuntimeValue::String("hi".into()).to_string(), "\"hi\"");
        assert_eq!(RuntimeValue::ShapeTag("circle".into()).to_string(), "shape:circle");
        assert_eq!(
            RuntimeValue::Closure {
                param: "x".into(),
                body: CoreExpr::Lit(reciplexa_core::expr::CoreLiteral::Number(0.0)),
                env: HashMap::new(),
            }
            .to_string(),
            "closure(x)"
        );
        assert_eq!(
            RuntimeValue::Record(vec![("a".into(), RuntimeValue::Number(1.0))]).to_string(),
            "record{a: 1}"
        );
        assert_eq!(
            RuntimeValue::Variant {
                tag: "Ok".into(),
                payload: Some(Box::new(RuntimeValue::Number(1.0))),
            }
            .to_string(),
            "Ok(1)"
        );
        assert_eq!(
            RuntimeValue::Variant {
                tag: "Done".into(),
                payload: None,
            }
            .to_string(),
            "Done"
        );
    }

    #[test]
    fn debug_format_is_stable() {
        let v = RuntimeValue::Number(42.0);
        assert!(format!("{v:?}").contains("42.0"));
        let rec = RuntimeValue::Record(vec![]);
        assert!(format!("{rec:?}").contains("Record"));
    }

    #[test]
    fn equality_and_clone() {
        let a = RuntimeValue::String("x".into());
        let b = a.clone();
        assert_eq!(a, b);
        assert_ne!(a, RuntimeValue::Unit);
    }

    #[test]
    fn ty_maps_all_variants() {
        assert_eq!(RuntimeValue::Unit.ty(), CoreType::Unit);
        assert_eq!(RuntimeValue::Number(1.0).ty(), CoreType::Number);
        assert_eq!(RuntimeValue::String("s".into()).ty(), CoreType::String);
        assert_eq!(RuntimeValue::ShapeTag("rect".into()).ty(), CoreType::Shape);
        let closure_ty = RuntimeValue::Closure {
            param: "x".into(),
            body: CoreExpr::Lit(reciplexa_core::expr::CoreLiteral::Number(0.0)),
            env: HashMap::new(),
        }
        .ty();
        assert!(matches!(closure_ty, CoreType::Fun { .. }));
        let rec_ty = RuntimeValue::Record(vec![
            ("x".into(), RuntimeValue::Number(1.0)),
            ("y".into(), RuntimeValue::String("z".into())),
        ])
        .ty();
        assert!(matches!(rec_ty, CoreType::Record { .. }));
        let var_ty = RuntimeValue::Variant {
            tag: "Some".into(),
            payload: Some(Box::new(RuntimeValue::Number(2.0))),
        }
        .ty();
        assert!(matches!(var_ty, CoreType::Variant { .. }));
        let nullary = RuntimeValue::Variant {
            tag: "None".into(),
            payload: None,
        }
        .ty();
        if let CoreType::Variant { variants } = nullary {
            assert_eq!(variants[0].1, None);
        } else {
            panic!("expected variant type");
        }
    }

    #[test]
    fn closure_ty_has_fun_shape() {
        let v = RuntimeValue::Closure {
            param: "n".into(),
            body: CoreExpr::Lit(reciplexa_core::expr::CoreLiteral::Number(0.0)),
            env: HashMap::new(),
        };
        if let CoreType::Fun { args, ret, effects } = v.ty() {
            assert_eq!(args.len(), 1);
            assert_eq!(*ret, CoreType::Unit);
            assert_eq!(effects, EffectRow::default());
        } else {
            panic!("expected Fun");
        }
    }

    #[test]
    fn record_ty_preserves_field_types() {
        if let CoreType::Record { fields } =
            RuntimeValue::Record(vec![("n".into(), RuntimeValue::Number(1.0))]).ty()
        {
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].0, "n");
            assert_eq!(fields[0].1, CoreType::Number);
        } else {
            panic!("expected Record");
        }
    }
}
