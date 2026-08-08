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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreType {
    Number,
    String,
    Color,
    Shape,
    Unit,
    Var(TypeVarId),
    Fun {
        args: Vec<CoreType>,
        ret: Box<CoreType>,
        effects: EffectRow,
    },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_var_id_ordering() {
        let a = TypeVarId::new(1);
        let b = TypeVarId::new(2);
        assert!(a < b);
        assert_eq!(TypeVarId::new(42).0, 42);
    }

    #[test]
    fn display_delegates_to_debug() {
        assert_eq!(CoreType::Number.to_string(), "Number");
        assert_eq!(CoreType::Unit.to_string(), "Unit");
        let fun = CoreType::Fun {
            args: vec![CoreType::Number],
            ret: Box::new(CoreType::String),
            effects: EffectRow::default(),
        };
        assert!(fun.to_string().contains("Fun"));
    }

    #[test]
    fn effect_row_default_is_empty() {
        let row = EffectRow::default();
        assert!(row.ops.is_empty());
        let row2 = EffectRow { ops: vec!["log".into()] };
        assert_ne!(row, row2);
    }

    #[test]
    fn core_type_variants_eq() {
        assert_eq!(CoreType::Color, CoreType::Color);
        assert_ne!(CoreType::Shape, CoreType::String);
        let rec = CoreType::Record {
            fields: vec![("x".into(), CoreType::Number)],
        };
        assert_eq!(
            rec,
            CoreType::Record {
                fields: vec![("x".into(), CoreType::Number)],
            }
        );
        let var = CoreType::Variant {
            variants: vec![("Ok".into(), Some(CoreType::Number))],
        };
        assert!(matches!(var, CoreType::Variant { .. }));
    }
}
