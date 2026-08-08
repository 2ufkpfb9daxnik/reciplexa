//! Runtime values during evaluation.

use reciplexa_core::ty::CoreType;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Unit,
    Number(f64),
    String(String),
    ShapeTag(String),
}

impl RuntimeValue {
    pub fn ty(&self) -> CoreType {
        match self {
            Self::Unit => CoreType::Unit,
            Self::Number(_) => CoreType::Number,
            Self::String(_) => CoreType::String,
            Self::ShapeTag(_) => CoreType::Shape,
        }
    }
}
