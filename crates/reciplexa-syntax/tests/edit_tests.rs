//! Integration tests moved from src/edit.rs for region coverage.

use reciplexa_syntax::edit::*;
use reciplexa_syntax::kind::SyntaxKind;
use reciplexa_syntax::parse::{parse_source, unparse};

// --- validity ---

#[test]
fn replace_number_preserves_surrounding_trivia() {
    let src = "(circle  10  20\n  30)";
    let root = parse_source(src).into_result().unwrap();
    let num = root
        .descendants_with_tokens()
        .find_map(|el| {
            let t = el.into_token()?;
            (t.kind() == SyntaxKind::Number && t.text() == "10").then_some(t)
        })
        .unwrap();
    let (new_root, out) = replace_token_text(&num, "99");
    assert_eq!(out, "(circle  99  20\n  30)");
    assert_eq!(unparse(&new_root), out);
}

#[test]
fn format_drag_number_trims() {
    assert_eq!(format_drag_number(10.0), "10");
    assert_eq!(format_drag_number(1.5), "1.5");
    assert_eq!(format_drag_number(1.500_000_1), "1.5");
    assert_eq!(format_drag_number(1.25), "1.25");
    assert_eq!(format_drag_number(1.2), "1.2");
}

// --- defect ---

#[test]
fn format_non_finite_becomes_zero() {
    assert_eq!(format_drag_number(f64::NAN), "0");
    assert_eq!(format_drag_number(f64::INFINITY), "0");
}

#[test]
fn token_at_offset_misses_gaps() {
    let root = parse_source("(a 1)").into_result().unwrap();
    // whitespace between a and 1
    assert!(token_at_offset(&root, 2, SyntaxKind::Number).is_none());
}

#[test]
fn token_at_offset_hits_interior_and_end() {
    let root = parse_source("(a 12)").into_result().unwrap();
    let interior = token_at_offset(&root, 4, SyntaxKind::Number).unwrap();
    assert_eq!(interior.text(), "12");
    // caret at end of the number token
    let at_end = token_at_offset(&root, 5, SyntaxKind::Number).unwrap();
    assert_eq!(at_end.text(), "12");
}
