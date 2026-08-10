//! Runtime values during evaluation.

use std::cell::Cell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use reciplexa_core::expr::CoreExpr;
use reciplexa_core::ty::CoreType;
use reciplexa_core::ty::TypeVarId;

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Unit,
    Number(f64),
    String(String),
    Bool(bool),
    ShapeTag(String),
    Closure {
        params: Vec<String>,
        body: CoreExpr,
        /// Shared so [`CoreExpr::LetRec`] bindings see each other.
        env: Rc<std::cell::RefCell<HashMap<String, RuntimeValue>>>,
    },
    /// Mutable cell for [`CoreExpr::LocalVar`] / [`CoreExpr::Set`].
    Cell {
        value: Rc<std::cell::RefCell<RuntimeValue>>,
        /// Cleared when the owning `var` scope exits (non-escape check).
        alive: Rc<Cell<bool>>,
    },
    /// One-shot resume continuation for [`CoreExpr::Handle`].
    OneShotResume {
        used: Rc<Cell<bool>>,
    },
    Record(Vec<(String, RuntimeValue)>),
    Variant {
        tag: String,
        payload: Option<Box<RuntimeValue>>,
    },
}

impl PartialEq for RuntimeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unit, Self::Unit) => true,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::ShapeTag(a), Self::ShapeTag(b)) => a == b,
            (
                Self::Closure {
                    params: p1,
                    body: b1,
                    ..
                },
                Self::Closure {
                    params: p2,
                    body: b2,
                    ..
                },
            ) => p1 == p2 && b1 == b2,
            (
                Self::Cell {
                    value: a,
                    alive: aa,
                },
                Self::Cell {
                    value: b,
                    alive: ba,
                },
            ) => aa.get() == ba.get() && *a.borrow() == *b.borrow(),
            (Self::OneShotResume { used: a }, Self::OneShotResume { used: b }) => {
                a.get() == b.get()
            }
            (Self::Record(a), Self::Record(b)) => a == b,
            (
                Self::Variant {
                    tag: t1,
                    payload: p1,
                },
                Self::Variant {
                    tag: t2,
                    payload: p2,
                },
            ) => t1 == t2 && p1 == p2,
            _ => false,
        }
    }
}

impl RuntimeValue {
    pub fn ty(&self) -> CoreType {
        match self {
            Self::Unit => CoreType::Unit,
            Self::Number(_) => CoreType::Number,
            Self::String(_) => CoreType::String,
            Self::Bool(_) => CoreType::Bool,
            Self::ShapeTag(_) => CoreType::Shape,
            Self::Cell { value, .. } => value.borrow().ty(),
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
            Self::Cell { value, alive } => {
                if alive.get() {
                    format!("cell({})", value.borrow())
                } else {
                    "cell(dead)".into()
                }
            }
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
