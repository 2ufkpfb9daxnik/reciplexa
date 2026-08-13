//! graphics_value text lowering: optional `wrap-em` / newlines → wrap_text_shape_content.

use reciplexa_eval::graphics_value::shape_from_graphics_value;
use reciplexa_eval::RuntimeValue;
use reciplexa_scene::Shape;
use reciplexa_std::japanese::break_line;

fn rec(fields: Vec<(&str, RuntimeValue)>) -> RuntimeValue {
    RuntimeValue::Record(
        fields
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

fn num(n: f64) -> RuntimeValue {
    RuntimeValue::F64(n)
}

fn text_shape(content: &str, wrap_em: Option<f64>) -> RuntimeValue {
    let mut fields = vec![
        ("tag", RuntimeValue::String("text".into())),
        ("x", num(10.0)),
        ("y", num(20.0)),
        ("size", num(12.0)),
        ("content", RuntimeValue::String(content.into())),
    ];
    if let Some(em) = wrap_em {
        fields.push(("wrap-em", num(em)));
    }
    rec(fields)
}

#[test]
fn text_without_wrap_em_stays_single() {
    let shape = shape_from_graphics_value(&text_shape("短い", None)).unwrap();
    match shape {
        Shape::Text(t) => assert_eq!(t.content, "短い"),
        other => panic!("expected single Text, got {other:?}"),
    }
}

#[test]
fn text_wrap_em_soft_wraps_via_break_line() {
    let content = "今日はいい天気です。東京タワーへ行こう。";
    let expected = break_line(content, 5.0);
    assert!(expected.len() >= 2);

    let shape = shape_from_graphics_value(&text_shape(content, Some(5.0))).unwrap();
    let Shape::Group { children, .. } = shape else {
        panic!("expected Group of wrapped Text shapes");
    };
    let texts: Vec<_> = children
        .iter()
        .filter_map(|s| match s {
            Shape::Text(t) => Some(t.content.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        texts,
        expected.iter().map(String::as_str).collect::<Vec<_>>()
    );
}

#[test]
fn text_newlines_split_without_wrap_em() {
    let shape = shape_from_graphics_value(&text_shape("一行目\n二行目", None)).unwrap();
    let Shape::Group { children, .. } = shape else {
        panic!("expected Group for newline split");
    };
    assert_eq!(children.len(), 2);
    match (&children[0], &children[1]) {
        (Shape::Text(a), Shape::Text(b)) => {
            assert_eq!(a.content, "一行目");
            assert_eq!(b.content, "二行目");
            assert!((b.y_mm - (a.y_mm + 12.0)).abs() < 1e-9);
        }
        other => panic!("expected two Text children, got {other:?}"),
    }
}
