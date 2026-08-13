//! Wave 12 F4 tip coverage: cl-08 glue, ASCII-space soft-wrap, class_spacing_em.

use reciplexa_identity::document::StableNodeId;
use reciplexa_std::japanese::break_line;
use reciplexa_std::math::{
    class_spacing_em, MathAtom, MathClass, MED_MUSKIP_EM, THIN_MUSKIP_EM,
};

fn id(n: u64) -> StableNodeId {
    StableNodeId::new(n)
}

#[test]
fn tip_wave12_break_glue_space_and_math_spacing() {
    assert_eq!(break_line("………", 2.0), vec!["………".to_string()]);
    assert_eq!(
        break_line("hi there", 3.0),
        vec!["hi".to_string(), " there".to_string()]
    );

    assert!((class_spacing_em(MathClass::Ordinary, MathClass::Operator) - THIN_MUSKIP_EM).abs() < 1e-9);
    let a = MathAtom::symbol(id(1), "a", MathClass::Ordinary);
    let op = MathAtom::symbol(id(2), "∑", MathClass::Operator);
    let row = MathAtom::row(id(3), vec![a.clone(), op.clone()]);
    let expected = a.estimate_box().width + op.estimate_box().width + THIN_MUSKIP_EM;
    assert!((row.estimate_box().width - expected).abs() < 1e-9);
    let _ = MED_MUSKIP_EM;
}
