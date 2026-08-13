//! Wave 10 D5 tip coverage: stackrel/aligned_column_x/math-box/vertical ruby+bou.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_identity::document::StableNodeId;
use reciplexa_std::japanese::{
    bou_estimate_box, vertical_ruby_estimate_box, Ruby, VERTICAL_RUBY_SIDE_EM,
};
use reciplexa_std::math::{
    aligned_column_x, stackrel_spacing_offsets, underbrace_spacing, MathAtom, MathBox, MathClass,
    MathStackKind, ALIGNED_COLUMN_GUTTER_EM, STACKREL_GAP_EM, UNDERBRACE_CLEARANCE_EM,
};

fn id(n: u64) -> StableNodeId {
    StableNodeId::new(n)
}

#[test]
fn tip_wave10_stackrel_aligned_math_box() {
    let (uy, ly) =
        stackrel_spacing_offsets(MathBox::new(0.5, 0.4, 0.1), MathBox::new(1.0, 0.7, 0.2));
    assert!(uy > 0.0 && ly < 0.0);
    assert!((underbrace_spacing() - UNDERBRACE_CLEARANCE_EM).abs() < 1e-9);

    let a = MathAtom::symbol(id(1), "a", MathClass::Ordinary);
    let bb = MathAtom::symbol(id(2), "bb", MathClass::Ordinary);
    let rows = vec![vec![a.clone(), bb.clone()]];
    assert!((aligned_column_x(&rows, 0)).abs() < 1e-9);
    assert!(aligned_column_x(&rows, 1) >= ALIGNED_COLUMN_GUTTER_EM);
    let _ = STACKREL_GAP_EM;
    let _ = MathStackKind::Stackrel;

    let v = eval_source(
        r#"(val main (math-box (record (tag "math-symbol") (glyph "x") (class "ord"))))"#,
    )
    .unwrap();
    match v {
        RuntimeValue::Record(fields) => {
            assert!(fields.iter().any(|(k, _)| k == "width"));
        }
        other => panic!("math-box record, got {other}"),
    }
    let _ = typecheck_language_source(r#"(val main (math-box "x"))"#).unwrap();
}

#[test]
fn tip_wave10_vertical_ruby_bou() {
    let vb = vertical_ruby_estimate_box(&Ruby::simple("東", "とう"));
    assert!((vb.annotation_side_x - VERTICAL_RUBY_SIDE_EM).abs() < 1e-9);
    assert!(vb.advance > 0.0);
    let bou = bou_estimate_box("傍");
    assert!(bou.advance > 0.0);
    assert!(bou.mark_size > 0.0);
}
