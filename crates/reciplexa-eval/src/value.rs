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
    Bool(bool),
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
            Self::Bool(_) => CoreType::Bool,
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
        let rendered = match self {
            Self::Unit => "unit".to_string(),
            Self::Number(n) => n.to_string(),
            Self::String(s) => format!("\"{s}\""),
            Self::Bool(b) => b.to_string(),
            Self::ShapeTag(s) => format!("shape:{s}"),
            Self::Closure { param, .. } => format!("closure({param})"),
            Self::Record(fields) => {
                let parts: Vec<String> = fields.iter().map(|(k, v)| format!("{k}: {v}")).collect();
                format!("record{{{}}}", parts.join(", "))
            }
            Self::Variant { tag, payload } => match payload {
                Some(p) => format!("{tag}({p})"),
                None => tag.clone(),
            },
        };
        f.write_str(&rendered)
    }
}
