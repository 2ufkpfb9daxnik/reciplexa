//! Wave 14 H3 tip coverage: vertical-orientation, advance swap, class_spacing expand.

use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_std::japanese::{needs_tate_rotation, vertical_advance_em};
use reciplexa_std::math::{class_spacing_em, MathClass, MED_MUSKIP_EM, THIN_MUSKIP_EM};

#[test]
fn tip_wave14_orientation_advance_spacing() {
    let v = eval_source(r#"(val main (vertical-orientation "-"))"#).unwrap();
    assert_eq!(v, RuntimeValue::String("rotated".into()));
    assert!(needs_tate_rotation('-'));
    assert!((vertical_advance_em('-') - 0.5).abs() < 1e-9);
    assert!((vertical_advance_em('東') - 1.0).abs() < 1e-9);

    assert!((class_spacing_em(MathClass::Operator, MathClass::Operator) - THIN_MUSKIP_EM).abs() < 1e-9);
    assert!((class_spacing_em(MathClass::Operator, MathClass::Binary) - MED_MUSKIP_EM).abs() < 1e-9);
}
