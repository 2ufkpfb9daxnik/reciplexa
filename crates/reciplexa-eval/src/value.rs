//! Runtime values during evaluation.

use std::collections::HashMap;

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
