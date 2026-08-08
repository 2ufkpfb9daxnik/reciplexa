//! Syntax → Core lowering (minimal surface forms).

use reciplexa_source::range::TextRange;

use crate::expr::{CoreExpr, CoreLiteral, CoreValue};
use crate::ty::CoreType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LowerError {
    pub message: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoweredForm {
    Page(Vec<CoreValue>),
    Src(Vec<CoreValue>),
    Doc,
}

/// Lower a list form head + argument count check into core.
pub fn lower_surface_form(
    head: &str,
    num_args: usize,
    range: TextRange,
) -> Result<CoreValue, LowerError> {
    let err = |msg: &str| LowerError {
        message: msg.into(),
        range,
    };
    match head {
        "perform" if num_args == 1 => Ok(CoreValue {
            ty: CoreType::Unit,
            expr: CoreExpr::Perform {
                op: "log".into(),
                arg: Box::new(CoreExpr::Lit(CoreLiteral::String(String::new()))),
            },
        }),
        "circle" if (3..=4).contains(&num_args) => Ok(CoreValue {
            ty: CoreType::Shape,
            expr: CoreExpr::Lit(CoreLiteral::String("circle".into())),
        }),
        "rect" if (4..=5).contains(&num_args) => Ok(CoreValue {
            ty: CoreType::Shape,
            expr: CoreExpr::Lit(CoreLiteral::String("rect".into())),
        }),
        "text" if (4..=7).contains(&num_args) => Ok(CoreValue {
            ty: CoreType::Shape,
            expr: CoreExpr::Lit(CoreLiteral::String("text".into())),
        }),
        "src" => Ok(CoreValue {
            ty: CoreType::Unit,
            expr: CoreExpr::Seq(Vec::new()),
        }),
        other => Err(err(&format!(
            "unsupported form `{other}` for core lowering"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_source::offset::ByteOffset;

    #[test]
    fn lowers_rect_form() {
        let range = TextRange::try_new(ByteOffset::ZERO, ByteOffset::new(1)).unwrap();
        let v = lower_surface_form("rect", 4, range).unwrap();
        assert_eq!(v.ty, CoreType::Shape);
    }
}
