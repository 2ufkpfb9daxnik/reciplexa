//! Runtime values during evaluation.

use std::cell::Cell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use reciplexa_core::expr::CoreExpr;
use reciplexa_core::ty::CoreType;
use reciplexa_core::ty::TypeVarId;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Unit,
    Number(f64),
    String(String),
    Bool(bool),
    ShapeTag(String),
    Closure {
        params: Vec<String>,
        body: CoreExpr,
        env: HashMap<String, RuntimeValue>,
    },
    /// One-shot resume continuation for shallow [`CoreExpr::Handle`] (EFF-001 v0).
    OneShotResume {
        used: Rc<Cell<bool>>,
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
            Self::OneShotResume { .. } => CoreType::Fun {
                args: vec![CoreType::Dynamic],
                ret: Box::new(CoreType::Dynamic),
                effects: Default::default(),
            },
            Self::Closure { params, .. } => CoreType::Fun {
                args: params
                    .iter()
                    .enumerate()
                    .map(|(i, _)| CoreType::Var(TypeVarId::new(i as u32)))
                    .collect(),
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

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rendered = match self {
            Self::Unit => "unit".to_string(),
            Self::Number(n) => n.to_string(),
            Self::String(s) => format!("\"{s}\""),
            Self::Bool(b) => b.to_string(),
            Self::ShapeTag(s) => format!("shape:{s}"),
            Self::OneShotResume { .. } => "resume".to_string(),
            Self::Closure { params, .. } => format!("closure({})", params.join(", ")),
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
