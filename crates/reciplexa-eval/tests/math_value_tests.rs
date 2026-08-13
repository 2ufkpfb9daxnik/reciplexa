//! Package math tagged records → MathAtom → estimate_box.

use reciplexa_eval::{
    estimate_math_box_from_value, eval_source, math_atom_from_value, MathValueError, RuntimeValue,
};
use reciplexa_std::math::MathAtom;

fn rec(fields: Vec<(&str, RuntimeValue)>) -> RuntimeValue {
    RuntimeValue::Record(
        fields
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

fn sym(glyph: &str, class: &str) -> RuntimeValue {
    rec(vec![
        ("tag", RuntimeValue::String("math-symbol".into())),
        ("glyph", RuntimeValue::String(glyph.into())),
        ("class", RuntimeValue::String(class.into())),
    ])
}

#[test]
fn symbol_and_fraction_estimate_boxes() {
    let a = math_atom_from_value(&sym("a", "ord")).unwrap();
    assert!(matches!(a, MathAtom::Symbol { .. }));
    let a_box = estimate_math_box_from_value(&sym("a", "ord")).unwrap();

    let frac = rec(vec![
        ("tag", RuntimeValue::String("math-fraction".into())),
        ("numerator", sym("1", "ord")),
        ("denominator", sym("2", "ord")),
    ]);
    let fb = estimate_math_box_from_value(&frac).unwrap();
    assert!(fb.height + fb.depth > a_box.height + a_box.depth);
}

#[test]
fn scripts_and_radical_from_tagged_records() {
    let absent = rec(vec![("tag", RuntimeValue::String("math-absent".into()))]);
    let scripts = rec(vec![
        ("tag", RuntimeValue::String("math-scripts".into())),
        ("base", sym("x", "ord")),
        ("superscript", sym("2", "ord")),
        ("subscript", absent.clone()),
    ]);
    let atom = math_atom_from_value(&scripts).unwrap();
    assert!(matches!(
        atom,
        MathAtom::Scripts {
            superscript: Some(_),
            subscript: None,
            ..
        }
    ));
    assert!(estimate_math_box_from_value(&scripts).unwrap().width > 0.0);

    let radical = rec(vec![
        ("tag", RuntimeValue::String("math-radical".into())),
        ("index", absent),
        ("radicand", sym("y", "ord")),
    ]);
    assert!(matches!(
        math_atom_from_value(&radical).unwrap(),
        MathAtom::Radical { index: None, .. }
    ));
}

#[test]
fn eval_package_shaped_fraction_then_estimate() {
    // Mirrors packages/math fraction constructor shape without import load.
    let v = eval_source(
        r#"(val main
  (record (tag "math-fraction")
    (numerator (record (tag "math-symbol") (glyph "a") (class "ord")))
    (denominator (record (tag "math-symbol") (glyph "b") (class "ord")))))"#,
    )
    .unwrap();
    let b = estimate_math_box_from_value(&v).unwrap();
    assert!(b.width > 0.0);
    assert!(b.height > 0.0);
}

#[test]
fn rejects_unsupported_and_absent_root() {
    let err = math_atom_from_value(&rec(vec![(
        "tag",
        RuntimeValue::String("math-unknown".into()),
    )]))
    .unwrap_err();
    assert!(err.message.contains("unsupported"));

    let err = math_atom_from_value(&rec(vec![(
        "tag",
        RuntimeValue::String("math-absent".into()),
    )]))
    .unwrap_err();
    assert!(err.message.contains("math-absent"));

    let _ = MathValueError {
        message: "x".into(),
    };
}

#[test]
fn delimiter_and_row_bridge() {
    let row = rec(vec![
        ("tag", RuntimeValue::String("math-row".into())),
        (
            "children",
            RuntimeValue::Variant {
                tag: "cons".into(),
                payload: Some(Box::new(rec(vec![
                    ("head", sym("+", "bin")),
                    (
                        "tail",
                        RuntimeValue::Variant {
                            tag: "nil".into(),
                            payload: None,
                        },
                    ),
                ]))),
            },
        ),
    ]);
    assert!(matches!(
        math_atom_from_value(&row).unwrap(),
        MathAtom::Row { .. }
    ));

    let delim = rec(vec![
        ("tag", RuntimeValue::String("math-delimiter".into())),
        ("left", RuntimeValue::String("(".into())),
        ("right", RuntimeValue::String(")".into())),
        ("body", sym("z", "ord")),
    ]);
    assert!(matches!(
        math_atom_from_value(&delim).unwrap(),
        MathAtom::Delimiter { .. }
    ));
    assert!(estimate_math_box_from_value(&delim).unwrap().width > 0.0);
}
