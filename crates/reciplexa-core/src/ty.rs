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
/// **ROW-001:** records are closed (exact field lists) under unify. Open row
/// variables are not implemented yet; [`CoreType::Lacks`] is a stub for future
/// absence constraints.
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
    /// ROW-001 stub: asserts that `row` lacks field `label` (not enforced by unify yet).
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
    Variant {
        variants: Vec<(String, Option<CoreType>)>,
    },
}

/// Thin effect row — grows in Phase 5 lowering.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectRow {
    pub ops: Vec<String>,
}

impl fmt::Display for CoreType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
