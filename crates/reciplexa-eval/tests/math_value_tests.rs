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

fn cons_list(items: Vec<RuntimeValue>) -> RuntimeValue {
    let mut cur = RuntimeValue::Variant {
        tag: "nil".into(),
        payload: None,
    };
    for item in items.into_iter().rev() {
        cur = RuntimeValue::Variant {
            tag: "cons".into(),
            payload: Some(Box::new(rec(vec![("head", item), ("tail", cur)]))),
        };
    }
    cur
}

#[test]
fn accent_matrix_bigop_stack_bridge() {
    let accent = rec(vec![
        ("tag", RuntimeValue::String("math-accent".into())),
        ("kind", RuntimeValue::String("hat".into())),
        ("base", sym("x", "ord")),
    ]);
    assert!(matches!(
        math_atom_from_value(&accent).unwrap(),
        MathAtom::Accent { .. }
    ));
    assert!(estimate_math_box_from_value(&accent).unwrap().height > 0.0);

    let row = rec(vec![
        ("tag", RuntimeValue::String("math-matrix-row".into())),
        ("cells", cons_list(vec![sym("a", "ord"), sym("b", "ord")])),
    ]);
    let matrix = rec(vec![
        ("tag", RuntimeValue::String("math-matrix".into())),
        ("kind", RuntimeValue::String("bmatrix".into())),
        ("rows", cons_list(vec![row])),
    ]);
    assert!(matches!(
        math_atom_from_value(&matrix).unwrap(),
        MathAtom::Matrix { .. }
    ));
    assert!(estimate_math_box_from_value(&matrix).unwrap().width > 0.0);

    let bigop = rec(vec![
        ("tag", RuntimeValue::String("math-bigop".into())),
        ("glyph", RuntimeValue::String("∑".into())),
        ("lower", sym("i", "ord")),
        (
            "upper",
            rec(vec![("tag", RuntimeValue::String("math-absent".into()))]),
        ),
        ("body", sym("x", "ord")),
    ]);
    assert!(matches!(
        math_atom_from_value(&bigop).unwrap(),
        MathAtom::BigOp {
            upper: None,
            lower: Some(_),
            ..
        }
    ));

    let bigop_scripts = rec(vec![
        ("tag", RuntimeValue::String("math-bigop-scripts".into())),
        ("glyph", RuntimeValue::String("∑".into())),
        ("superscript", sym("n", "ord")),
        ("subscript", sym("k=1", "ord")),
        ("body", sym("a_k", "ord")),
    ]);
    assert!(matches!(
        math_atom_from_value(&bigop_scripts).unwrap(),
        MathAtom::Row { .. }
    ));
    assert!(estimate_math_box_from_value(&bigop_scripts).unwrap().width > 0.0);

    let stack = rec(vec![
        ("tag", RuntimeValue::String("math-stack".into())),
        ("kind", RuntimeValue::String("atop".into())),
        (
            "children",
            cons_list(vec![sym("a", "ord"), sym("b", "ord")]),
        ),
    ]);
    assert!(matches!(
        math_atom_from_value(&stack).unwrap(),
        MathAtom::Stack { .. }
    ));

    let aligned = rec(vec![
        ("tag", RuntimeValue::String("math-aligned".into())),
        (
            "rows",
            cons_list(vec![rec(vec![
                ("tag", RuntimeValue::String("math-align-row".into())),
                ("cells", cons_list(vec![sym("x", "ord"), sym("=", "rel")])),
            ])]),
        ),
    ]);
    assert!(matches!(
        math_atom_from_value(&aligned).unwrap(),
        MathAtom::Aligned { .. }
    ));

    let stackrel = rec(vec![
        ("tag", RuntimeValue::String("math-stackrel".into())),
        ("relation", sym("def", "ord")),
        ("base", sym("=", "rel")),
    ]);
    assert!(matches!(
        math_atom_from_value(&stackrel).unwrap(),
        MathAtom::Stack { .. }
    ));
}

#[test]
fn under_over_cases_operatorname_bridge() {
    let underbrace = rec(vec![
        ("tag", RuntimeValue::String("math-under".into())),
        ("kind", RuntimeValue::String("underbrace".into())),
        ("body", sym("x+y", "ord")),
    ]);
    assert!(matches!(
        math_atom_from_value(&underbrace).unwrap(),
        MathAtom::Accent { .. }
    ));
    assert!(estimate_math_box_from_value(&underbrace).unwrap().width > 0.0);

    let overset = rec(vec![
        ("tag", RuntimeValue::String("math-over".into())),
        ("kind", RuntimeValue::String("overset".into())),
        ("label", sym("*", "ord")),
        ("body", sym("A", "ord")),
    ]);
    assert!(matches!(
        math_atom_from_value(&overset).unwrap(),
        MathAtom::Stack { .. }
    ));
    let over_box = estimate_math_box_from_value(&overset).unwrap();
    assert!(over_box.height + over_box.depth > 0.0);

    let overline = rec(vec![
        ("tag", RuntimeValue::String("math-over".into())),
        ("kind", RuntimeValue::String("overline".into())),
        ("body", sym("x", "ord")),
    ]);
    assert!(matches!(
        math_atom_from_value(&overline).unwrap(),
        MathAtom::Accent { .. }
    ));

    let arm = rec(vec![
        ("tag", RuntimeValue::String("math-case-arm".into())),
        ("body", sym("x", "ord")),
        ("guard", sym(">0", "rel")),
    ]);
    let cases = rec(vec![
        ("tag", RuntimeValue::String("math-cases".into())),
        ("left", RuntimeValue::String("{".into())),
        ("right", RuntimeValue::String("".into())),
        ("arms", cons_list(vec![arm.clone()])),
    ]);
    assert!(matches!(
        math_atom_from_value(&cases).unwrap(),
        MathAtom::Matrix { .. }
    ));
    assert!(estimate_math_box_from_value(&cases).unwrap().width > 0.0);
    assert!(matches!(
        math_atom_from_value(&arm).unwrap(),
        MathAtom::Row { .. }
    ));

    let opname = rec(vec![
        ("tag", RuntimeValue::String("math-operatorname".into())),
        ("name", RuntimeValue::String("sin".into())),
    ]);
    assert!(matches!(
        math_atom_from_value(&opname).unwrap(),
        MathAtom::Symbol { .. }
    ));
    assert!(estimate_math_box_from_value(&opname).unwrap().width >= 3.0);

    let env = rec(vec![
        ("tag", RuntimeValue::String("math-matrix-env".into())),
        ("kind", RuntimeValue::String("matrix".into())),
        (
            "rows",
            cons_list(vec![rec(vec![
                ("tag", RuntimeValue::String("math-matrix-row".into())),
                ("cells", cons_list(vec![sym("a", "ord"), sym("b", "ord")])),
            ])]),
        ),
    ]);
    assert!(matches!(
        math_atom_from_value(&env).unwrap(),
        MathAtom::Matrix { .. }
    ));

    let align_eq = rec(vec![
        ("tag", RuntimeValue::String("math-align-eq".into())),
        (
            "rows",
            cons_list(vec![rec(vec![
                ("tag", RuntimeValue::String("math-align-row".into())),
                ("cells", cons_list(vec![sym("x", "ord"), sym("=", "rel")])),
            ])]),
        ),
    ]);
    assert!(matches!(
        math_atom_from_value(&align_eq).unwrap(),
        MathAtom::Aligned { .. }
    ));

    let substack = rec(vec![
        ("tag", RuntimeValue::String("math-substack".into())),
        (
            "rows",
            cons_list(vec![rec(vec![
                ("tag", RuntimeValue::String("math-align-row".into())),
                ("cells", cons_list(vec![sym("a", "ord")])),
            ])]),
        ),
    ]);
    assert!(matches!(
        math_atom_from_value(&substack).unwrap(),
        MathAtom::Stack { .. }
    ));
}

#[test]
fn scripts_attachment_offsets_from_package_scripts_tag() {
    use reciplexa_eval::scripts_attachment_offsets_from_value;
    use reciplexa_std::math::scripts_attachment_offsets;

    let absent = rec(vec![("tag", RuntimeValue::String("math-absent".into()))]);
    let scripts = rec(vec![
        ("tag", RuntimeValue::String("math-scripts".into())),
        ("base", sym("x", "ord")),
        ("superscript", sym("2", "ord")),
        ("subscript", sym("i", "ord")),
    ]);
    let (sub_x, sub_y, sup_x, sup_y) = scripts_attachment_offsets_from_value(&scripts).unwrap();
    assert!(sub_x > 0.0 && sup_x > 0.0);
    assert!(sub_y < 0.0, "subscript below baseline: {sub_y}");
    assert!(sup_y > 0.0, "superscript above baseline: {sup_y}");

    // Matches direct std call on the same estimate boxes.
    let atom = math_atom_from_value(&scripts).unwrap();
    let MathAtom::Scripts {
        base,
        superscript,
        subscript,
        ..
    } = atom
    else {
        panic!("expected Scripts");
    };
    let expected = scripts_attachment_offsets(
        base.estimate_box(),
        subscript.as_ref().map(|s| s.estimate_box()),
        superscript.as_ref().map(|s| s.estimate_box()),
    );
    assert_eq!((sub_x, sub_y, sup_x, sup_y), expected);

    let err = scripts_attachment_offsets_from_value(&sym("a", "ord")).unwrap_err();
    assert!(err.message.contains("math-scripts"));

    let sup_only = rec(vec![
        ("tag", RuntimeValue::String("math-scripts".into())),
        ("base", sym("x", "ord")),
        ("superscript", sym("n", "ord")),
        ("subscript", absent),
    ]);
    let (_, _, sx, sy) = scripts_attachment_offsets_from_value(&sup_only).unwrap();
    assert!(sx > 0.0 && sy > 0.0);
}
