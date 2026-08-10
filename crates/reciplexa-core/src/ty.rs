//! Core types and effect rows.

use std::fmt;

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
    /// Gradual typing entry point (TYP-001). Unifies with any type in v0.
    Dynamic,
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
