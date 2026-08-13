//! N6 tip p: assert empty-codes + Display residual.

use reciplexa_test::assert::{assert_diagnostic_codes, assert_eq_structured, StructuredDiff};

#[test]
fn assert_n6p_empty_and_display() {
    assert_diagnostic_codes(&[], &[]);
    assert_eq_structured(&0u8, &0u8);
    let d = StructuredDiff {
        path: String::new(),
        expected: String::new(),
        actual: "x".into(),
    };
    assert!(d.to_string().contains("got"));
    let d2 = StructuredDiff {
        path: "p".into(),
        expected: "1".into(),
        actual: "1".into(),
    };
    let _ = format!("{d2}");
}
