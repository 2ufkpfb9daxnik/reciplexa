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
    /// Shallow effect handler (EFF-001 v0).
    ///
    /// Catches matching [`CoreExpr::Perform`] in `body`. Handler params are:
    /// - `[arg]` — shallow abort; handler result is the handle result
    /// - `[arg, resume]` — one-shot resume: applying `resume` to `v` makes `v`
    ///   the handle result (does not continue the aborted body — shallow)
    Handle {
        op: String,
        handler_params: Vec<String>,
        handler_body: Box<CoreExpr>,
        body: Box<CoreExpr>,
    },
    Seq(Vec<CoreExpr>),
    Let {
        name: String,
        value: Box<CoreExpr>,
        body: Box<CoreExpr>,
    },
    /// Recursive function bindings; each RHS must be a [`CoreExpr::Lambda`].
    LetRec {
        bindings: Vec<(String, CoreExpr)>,
        body: Box<CoreExpr>,
    },
    Lambda {
        params: Vec<String>,
        body: Box<CoreExpr>,
    },
    App {
        fun: Box<CoreExpr>,
        args: Vec<CoreExpr>,
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
