//! Core types and effect rows.

use std::fmt;

/// DAT §8 variance classification for a type parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variance {
    Covariant,
    Contravariant,
    Invariant,
    Phantom,
}

/// Internal type variable for unification (Phase 2 step 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeVarId(pub u32);

impl TypeVarId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Surface-aligned core types.
///
/// **ROW-001:** closed records (exact field lists) and open records with a row
/// tail (`OpenRecord`). [`CoreType::Lacks`] is enforced under unify.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreType {
    Number,
    String,
    Color,
    Shape,
    Unit,
    Bool,
    /// SYN §11 immutable byte sequence.
    Bytes,
    /// Gradual typing entry point (TYP-001). Unifies with any type in v0.
    Dynamic,
    /// SYN §16.3 union — unifies loosely like [`CoreType::Dynamic`] for now.
    Union(Vec<CoreType>),
    /// SYN §16.3 intersection — stub structural type for occurrence typing / TYP.
    Intersect(Vec<CoreType>),
    /// SYN §16.3 negation — stub; pairs with [`CoreType::Intersect`] in narrowing.
    Not(Box<CoreType>),
    /// SYN §16.3 difference — `diff(S, T) ≃ intersect(S, not(T))`.
    Diff(Box<CoreType>, Box<CoreType>),
    /// DAT §18.5 / SYN §16.5: optional record field of type `T`.
    ///
    /// Only meaningful as a field type inside [`CoreType::Record`] /
    /// [`CoreType::OpenRecord`]. [`field`](crate::expr::CoreExpr::RecordGet)
    /// access yields an option-shaped [`CoreType::Variant`].
    OptionalField(Box<CoreType>),
    /// Absence constraint: `row` must not contain field `label`.
    Lacks {
        label: String,
        row: Box<CoreType>,
    },
    Var(TypeVarId),
    Fun {
        args: Vec<CoreType>,
        ret: Box<CoreType>,
        effects: EffectRow,
    },
    /// Closed record — fields must match exactly (same names, same order, same types).
    Record {
        fields: Vec<(String, CoreType)>,
    },
    /// Open record `{ fields | ρ }` — `row` is typically a [`CoreType::Var`] row tail.
    OpenRecord {
        fields: Vec<(String, CoreType)>,
        row: Box<CoreType>,
    },
    Variant {
        variants: Vec<(String, Option<CoreType>)>,
    },
    /// SYN §16.1 type application `(option str)` / nullary nominal `option`.
    /// Unifies loosely like [`CoreType::Dynamic`] until DAT instantiation lands.
    App {
        ctor: String,
        args: Vec<CoreType>,
    },
    /// SYN §16.2 surface `forall` — binder names are scoped in the body; stub unify.
    Forall {
        /// `(name, kind)` where kind is `type` / `record-row` / `effect-row`.
        params: Vec<(String, String)>,
        body: Box<CoreType>,
    },
    /// Bound type / row / effect-row variable from surface (`a`, `r`, `e`).
    Name(String),
    /// SYN §18.10 internal error type for [`crate::expr::CoreExpr::Error`].
    Error,
}

/// Thin effect row — grows in Phase 5 lowering.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectRow {
    pub ops: Vec<String>,
}

impl EffectRow {
    pub fn with_op(mut self, op: impl Into<String>) -> Self {
        let op = op.into();
        if !self.ops.iter().any(|o| o == &op) {
            self.ops.push(op);
        }
        self
    }

    pub fn without_op(&self, op: &str) -> Self {
        Self {
            ops: self
                .ops
                .iter()
                .filter(|o| o.as_str() != op)
                .cloned()
                .collect(),
        }
    }

    pub fn merge(&self, other: &Self) -> Self {
        let mut out = self.clone();
        for op in &other.ops {
            if !out.ops.iter().any(|o| o == op) {
                out.ops.push(op.clone());
            }
        }
        out
    }
}

impl fmt::Display for CoreType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
