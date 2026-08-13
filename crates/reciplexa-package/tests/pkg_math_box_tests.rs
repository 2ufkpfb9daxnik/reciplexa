//! Example `pkg_math_box.rpx` — math-box builtin demo.

use reciplexa_eval::{eval_expr, primitive_env, RuntimeValue, UnitHost};
use reciplexa_package::{elaborate_with_packages, LocalPackageIndex};
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

#[test]
fn pkg_math_box_example_elaborates_and_eval() {
    let entry =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_math_box.rpx");
    let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
    let units = elaborate_with_packages(&entry, &idx).unwrap();
    assert!(units.iter().any(|u| u.name == "pkg_math_box"));

    let demo = units.iter().find(|u| u.name == "pkg_math_box").unwrap();
    let v = eval_expr(&demo.expr, &primitive_env(), &mut UnitHost).unwrap();
    assert_eq!(
        field(&v, "tag"),
        &RuntimeValue::String("math-box-demo".into())
    );

    let from_record = field(&v, "from-record");
    assert_eq!(
        field(from_record, "tag"),
        &RuntimeValue::String("math-box".into())
    );
    match field(from_record, "width") {
        RuntimeValue::Number(w) => assert!(*w > 0.0),
        other => panic!("width number, got {other}"),
    }

    let from_string = field(&v, "from-string");
    match field(from_string, "width") {
        RuntimeValue::Number(w) => assert!(*w > 0.0),
        other => panic!("from-string width, got {other}"),
    }

    let display = field(&v, "scripts-display");
    let text = field(&v, "scripts-text");
    assert_eq!(field(display, "style"), &RuntimeValue::String("display".into()));
    assert_eq!(field(text, "style"), &RuntimeValue::String("text".into()));
    match (field(display, "width"), field(text, "width")) {
        (RuntimeValue::Number(dw), RuntimeValue::Number(tw)) => {
            assert!(tw < dw, "text scripts narrower than display: {tw} vs {dw}");
        }
        other => panic!("script widths, got {other:?}"),
    }
}
