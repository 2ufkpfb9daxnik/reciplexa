//! Core expressions.

use crate::ty::CoreType;

#[derive(Debug, Clone, PartialEq)]
pub enum CoreLiteral {
    Number(f64),
    String(String),
    Color(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoreExpr {
    Lit(CoreLiteral),
    /// Variable reference by binder name (BindingId later).
    Var(String),
    Perform {
        op: String,
        arg: Box<CoreExpr>,
    },
    Seq(Vec<CoreExpr>),
    Let {
        name: String,
        value: Box<CoreExpr>,
        body: Box<CoreExpr>,
    },
    Lambda {
        param: String,
        body: Box<CoreExpr>,
    },
    App {
        fun: Box<CoreExpr>,
        arg: Box<CoreExpr>,
    },
    If {
        cond: Box<CoreExpr>,
        then_branch: Box<CoreExpr>,
        else_branch: Box<CoreExpr>,
    },
    Record {
        fields: Vec<(String, CoreExpr)>,
    },
    RecordGet {
        record: Box<CoreExpr>,
        field: String,
    },
    Variant {
        tag: String,
        payload: Option<Box<CoreExpr>>,
    },
    Match {
        scrutinee: Box<CoreExpr>,
        arms: Vec<MatchArm>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub tag: String,
    pub bind: Option<String>,
    pub body: CoreExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoreValue {
    pub ty: CoreType,
    pub expr: CoreExpr,
}
