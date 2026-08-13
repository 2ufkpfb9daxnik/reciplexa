//! Wave 7 A4 tip coverage: ruby-box / tate-chu-yoko-width / doc-paragraph break_line.

use reciplexa_core::check::typecheck_language_source;
use reciplexa_eval::document_value::document_from_doc_value;
use reciplexa_eval::{eval_source, RuntimeValue};
use reciplexa_scene::Shape;
use reciplexa_std::japanese::{
    break_line, break_opportunity, break_pair_matrix_cell, Ruby, TateChuYoko, CharClass,
};

fn rec(fields: Vec<(&str, RuntimeValue)>) -> RuntimeValue {
    RuntimeValue::Record(
        fields
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

fn cons(items: Vec<RuntimeValue>) -> RuntimeValue {
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
fn tip_wave7_ruby_box_and_tate_width_builtins() {
    let ruby = eval_source(r#"(val main (ruby-box "東京" "とうきょう"))"#).unwrap();
    let RuntimeValue::Record(fields) = &ruby else {
        panic!("ruby-box → record");
    };
    let advance = fields
        .iter()
        .find(|(k, _)| k == "advance-width")
        .and_then(|(_, v)| match v {
            RuntimeValue::Number(n) => Some(*n),
            _ => None,
        })
        .expect("advance-width");
    let expected = Ruby::simple("東京", "とうきょう").estimate_box();
    assert!((advance - expected.advance_width).abs() < 1e-9);

    let tcy = eval_source(r#"(val main (tate-chu-yoko-width "2024"))"#).unwrap();
    let RuntimeValue::Number(w) = tcy else {
        panic!("tate-chu-yoko-width → number, got {tcy:?}");
    };
    assert!((w - TateChuYoko::new("2024").estimate_box().advance_width).abs() < 1e-9);

    let _ = typecheck_language_source(r#"(val main (ruby-box "a" "b"))"#).unwrap();
    let _ = typecheck_language_source(r#"(val main (tate-chu-yoko-width "1"))"#).unwrap();
}

#[test]
fn tip_wave7_break_pair_matrix_and_doc_paragraph_wrap() {
    // Matrix helper ↔ break_opportunity (digit-open corner).
    assert_eq!(
        break_pair_matrix_cell(20, 1),
        break_opportunity(CharClass::Numeric, CharClass::OpeningBrackets)
    );

    let long = "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん";
    let expected = break_line(long, 40.0);
    assert!(expected.len() > 1);

    let paragraph = rec(vec![
        ("tag", RuntimeValue::String("doc-paragraph".into())),
        ("text", RuntimeValue::String(long.into())),
    ]);
    let page = rec(vec![
        ("tag", RuntimeValue::String("doc-page".into())),
        (
            "paper",
            rec(vec![
                ("width", RuntimeValue::Int(210)),
                ("height", RuntimeValue::Int(297)),
            ]),
        ),
        (
            "flow",
            rec(vec![
                ("tag", RuntimeValue::String("doc-flow".into())),
                (
                    "sections",
                    cons(vec![rec(vec![
                        ("tag", RuntimeValue::String("doc-section".into())),
                        (
                            "blocks",
                            cons(vec![rec(vec![
                                ("tag", RuntimeValue::String("doc-block".into())),
                                ("kind", RuntimeValue::String("paragraph".into())),
                                ("paragraph", paragraph),
                            ])]),
                        ),
                    ])]),
                ),
            ]),
        ),
    ]);
    let doc = document_from_doc_value(&page).expect("lower");
    let texts: Vec<_> = doc.pages[0]
        .shapes
        .iter()
        .filter_map(|s| match s {
            Shape::Text(t) => Some(t.content.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(texts, expected.iter().map(String::as_str).collect::<Vec<_>>());
}
