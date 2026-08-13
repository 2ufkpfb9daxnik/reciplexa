//! Example `pkg_math_spacing.rpx` — Row class_spacing_em via math-box.

use reciplexa_eval::{eval_expr, primitive_env, RuntimeValue, UnitHost};
use reciplexa_identity::document::StableNodeId;
use reciplexa_package::{elaborate_with_packages, LocalPackageIndex};
use reciplexa_std::math::{class_spacing_em, MathAtom, MathClass, MED_MUSKIP_EM, THICK_MUSKIP_EM};
use std::path::PathBuf;

fn workspace_packages() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages")
}

fn field<'a>(rec: &'a RuntimeValue, name: &str) -> &'a RuntimeValue {
    match rec {
        RuntimeValue::Record(fields) => fields
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v)
            .unwrap_or_else(|| panic!("missing field {name} in {rec}")),
        other => panic!("expected record, got {other}"),
    }
}

fn width(rec: &RuntimeValue) -> f64 {
    match field(rec, "width") {
        RuntimeValue::Number(w) => *w,
        other => panic!("width number, got {other}"),
    }
}

#[test]
fn pkg_math_spacing_example_row_gaps_match_class_spacing_em() {
    let entry =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_math_spacing.rpx");
    let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
    let units = elaborate_with_packages(&entry, &idx).unwrap();
    let demo = units.iter().find(|u| u.name == "pkg_math_spacing").unwrap();
    let v = eval_expr(&demo.expr, &primitive_env(), &mut UnitHost).unwrap();
    assert_eq!(
        field(&v, "tag"),
        &RuntimeValue::String("math-spacing-demo".into())
    );

    let a = MathAtom::symbol(StableNodeId::new(1), "a", MathClass::Ordinary);
    let b = MathAtom::symbol(StableNodeId::new(2), "b", MathClass::Ordinary);
    let plus = MathAtom::symbol(StableNodeId::new(3), "+", MathClass::Binary);
    let eq = MathAtom::symbol(StableNodeId::new(4), "=", MathClass::Relation);

    let tight_w = a.estimate_box().width + b.estimate_box().width;
    let spaced_w = a.estimate_box().width
        + plus.estimate_box().width
        + b.estimate_box().width
        + class_spacing_em(MathClass::Ordinary, MathClass::Binary)
        + class_spacing_em(MathClass::Binary, MathClass::Ordinary);
    let related_w = a.estimate_box().width
        + eq.estimate_box().width
        + b.estimate_box().width
        + class_spacing_em(MathClass::Ordinary, MathClass::Relation)
        + class_spacing_em(MathClass::Relation, MathClass::Ordinary);

    assert!((width(field(&v, "tight")) - tight_w).abs() < 1e-9);
    assert!((width(field(&v, "spaced")) - spaced_w).abs() < 1e-9);
    assert!((width(field(&v, "related")) - related_w).abs() < 1e-9);
    assert!(spaced_w > tight_w);
    assert!((spaced_w - tight_w - plus.estimate_box().width - 2.0 * MED_MUSKIP_EM).abs() < 1e-9);
    assert!((related_w - tight_w - eq.estimate_box().width - 2.0 * THICK_MUSKIP_EM).abs() < 1e-9);
}
