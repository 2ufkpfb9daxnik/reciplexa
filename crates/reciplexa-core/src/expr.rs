//! Core expressions.

use crate::ty::CoreType;

#[derive(Debug, Clone, PartialEq)]
pub enum CoreLiteral {
    Number(f64),
    String(String),
    Color(String),
    Bool(bool),
    /// SYN-001 / DAT-001: bare `unit` literal (not `()`).
    Unit,
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
    /// Shallow effect handler (EFF-001 v0) upgraded to deep one-shot resume.
    ///
    /// Catches matching [`CoreExpr::Perform`] in `body`. Handler params are:
    /// - `[arg]` — abort; handler result is the handle result
    /// - `[arg, resume]` — one-shot deep resume: applying `resume` to `v`
    ///   continues `body` as if `perform` returned `v` (handler is reinstalled)
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
    /// Local mutable cell: evaluate `init`, bind `name` in `body`, invalidate on exit.
    LocalVar {
        name: String,
        init: Box<CoreExpr>,
        body: Box<CoreExpr>,
    },
    /// Mutate a [`LocalVar`] cell; yields unit.
    Set {
        name: String,
        value: Box<CoreExpr>,
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

/// DAT-001 §15–16 match pattern.
#[derive(Debug, Clone, PartialEq)]
pub enum CorePattern {
    /// `_`
    Wildcard,
    /// `bind name` — binds the entire scrutinee.
    Bind(String),
    /// Nullary `Tag` or payload `Tag pat`.
    Variant {
        tag: String,
        payload: Option<Box<CorePattern>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: CorePattern,
    pub body: CoreExpr,
}

impl MatchArm {
    /// Constructor arm with optional simple payload binder (tests / mem helpers).
    pub fn variant(tag: String, bind: Option<String>, body: CoreExpr) -> Self {
        Self {
            pattern: CorePattern::Variant {
                tag,
                payload: bind.map(|n| Box::new(CorePattern::Bind(n))),
            },
            body,
        }
    }

    /// Top-level constructor tag, if this arm is a variant pattern.
    pub fn tag(&self) -> Option<&str> {
        match &self.pattern {
            CorePattern::Variant { tag, .. } => Some(tag.as_str()),
            _ => None,
        }
    }

    /// True when this arm matches any value (`_` or `bind`).
    pub fn is_catch_all(&self) -> bool {
        matches!(
            &self.pattern,
            CorePattern::Wildcard | CorePattern::Bind(_)
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoreValue {
    pub ty: CoreType,
    pub expr: CoreExpr,
}
