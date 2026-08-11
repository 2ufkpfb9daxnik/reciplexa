//! Runtime values during evaluation.

use std::cell::Cell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use reciplexa_core::expr::CoreExpr;
use reciplexa_core::ty::CoreType;
use reciplexa_core::ty::TypeVarId;

use crate::control::ResumeCont;

#[derive(Clone)]
pub enum RuntimeValue {
    Unit,
    /// Legacy float runtime value; prefer [`RuntimeValue::F64`].
    Number(f64),
    /// DD-TYP-NUM-001 exact integer runtime value (i128 kernel subset).
    Int(i128),
    /// DD-TYP-NUM-001 IEEE binary64.
    F64(f64),
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
        cont: ResumeCont,
    },
    /// First-class handler value from [`CoreExpr::HandlerValue`].
    Handler {
        op: String,
        params: Vec<String>,
        body: CoreExpr,
        env: Rc<std::cell::RefCell<HashMap<String, RuntimeValue>>>,
    },
    /// KER-001 numeric / comparison primitive.
    Builtin(BuiltinOp),
    Record(Vec<(String, RuntimeValue)>),
    Bytes(Vec<u8>),
    Variant {
        tag: String,
        payload: Option<Box<RuntimeValue>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    /// DD-TYP-IF-001 recognized type predicates (unary → Bool).
    IsNumber,
    IsString,
    IsBool,
    IsNone,
    IsSome,
    /// SYN §8.4 `(unicode codepoint)` — one Number arg → one-char String.
    Unicode,
    /// SYN §11 `(encode-utf8 text)` — str → bytes.
    EncodeUtf8,
    /// SYN §11 `(decode-utf8 data)` — bytes → result variant.
    DecodeUtf8,
    /// DD-TYP-NUM-002: integer division `int-div : int × int -> int`.
    IntDiv,
    /// DD-TYP-NUM-002: remainder `mod : int × int -> int`.
    Mod,
}

impl fmt::Debug for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unit => write!(f, "Unit"),
            Self::Number(n) => f.debug_tuple("Number").field(n).finish(),
            Self::Int(n) => f.debug_tuple("Int").field(n).finish(),
            Self::F64(n) => f.debug_tuple("F64").field(n).finish(),
            Self::String(s) => f.debug_tuple("String").field(s).finish(),
            Self::Bool(b) => f.debug_tuple("Bool").field(b).finish(),
            Self::ShapeTag(s) => f.debug_tuple("ShapeTag").field(s).finish(),
            Self::Closure { params, body, .. } => f
                .debug_struct("Closure")
                .field("params", params)
                .field("body", body)
                .finish(),
            Self::Cell { value, alive } => f
                .debug_struct("Cell")
                .field("value", &*value.borrow())
                .field("alive", &alive.get())
                .finish(),
            Self::OneShotResume { used, .. } => f
                .debug_struct("OneShotResume")
                .field("used", &used.get())
                .finish(),
            Self::Handler {
                op, params, body, ..
            } => f
                .debug_struct("Handler")
                .field("op", op)
                .field("params", params)
                .field("body", body)
                .finish(),
            Self::Builtin(op) => f.debug_tuple("Builtin").field(op).finish(),
            Self::Record(fields) => f.debug_tuple("Record").field(fields).finish(),
            Self::Bytes(b) => f.debug_tuple("Bytes").field(b).finish(),
            Self::Variant { tag, payload } => f
                .debug_struct("Variant")
                .field("tag", tag)
                .field("payload", payload)
                .finish(),
        }
    }
}

impl PartialEq for RuntimeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unit, Self::Unit) => true,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::F64(a), Self::F64(b)) => a == b,
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
            (Self::OneShotResume { used: a, .. }, Self::OneShotResume { used: b, .. }) => {
                a.get() == b.get()
            }
            (
                Self::Handler {
                    op: o1,
                    params: p1,
                    body: b1,
                    ..
                },
                Self::Handler {
                    op: o2,
                    params: p2,
                    body: b2,
                    ..
                },
            ) => o1 == o2 && p1 == p2 && b1 == b2,
            (Self::Builtin(a), Self::Builtin(b)) => a == b,
            (Self::Record(a), Self::Record(b)) => a == b,
            (Self::Bytes(a), Self::Bytes(b)) => a == b,
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
            Self::Number(_) => CoreType::F64,
            Self::Int(_) => CoreType::Int,
            Self::F64(_) => CoreType::F64,
            Self::String(_) => CoreType::String,
            Self::Bool(_) => CoreType::Bool,
            Self::ShapeTag(_) => CoreType::Shape,
            Self::Cell { value, .. } => value.borrow().ty(),
            Self::OneShotResume { .. } => CoreType::Fun {
                args: vec![CoreType::dyn_any()],
                ret: Box::new(CoreType::dyn_any()),
                effects: Default::default(),
            },
            Self::Handler { .. } => CoreType::dyn_any(),
            Self::Builtin(_) => CoreType::Fun {
                args: vec![CoreType::Number, CoreType::Number],
                ret: Box::new(CoreType::dyn_any()),
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
            Self::Bytes(_) => CoreType::Bytes,
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
            Self::Int(n) => n.to_string(),
            Self::F64(n) => n.to_string(),
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
            Self::Handler { op, .. } => format!("handler({op})"),
            Self::Builtin(op) => format!("builtin({op:?})"),
            Self::Closure { params, .. } => format!("closure({})", params.join(", ")),
            Self::Record(fields) => {
                let parts: Vec<String> = fields.iter().map(|(k, v)| format!("{k}: {v}")).collect();
                format!("record{{{}}}", parts.join(", "))
            }
            Self::Bytes(b) => {
                let parts: Vec<String> = b.iter().map(|x| format!("0x{x:02x}")).collect();
                format!("bytes({})", parts.join(" "))
            }
            Self::Variant { tag, payload } => match payload {
                Some(p) => format!("{tag}({p})"),
                None => tag.clone(),
            },
        };
        f.write_str(&rendered)
    }
}
