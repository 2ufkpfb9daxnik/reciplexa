//! Example `pkg_math_phantom.rpx` — math-phantom / math-smash builtin demo.

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

fn num(v: &RuntimeValue) -> f64 {
    match v {
        RuntimeValue::Number(n) => *n,
        RuntimeValue::F64(n) => *n,
        RuntimeValue::Int(n) => *n as f64,
        other => panic!("expected number, got {other}"),
    }
}

#[test]
fn pkg_math_phantom_example_elaborates_and_eval() {
    let entry =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_math_phantom.rpx");
    let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
    let units = elaborate_with_packages(&entry, &idx).unwrap();
    assert!(units.iter().any(|u| u.name == "pkg_math_phantom"));

    let demo = units.iter().find(|u| u.name == "pkg_math_phantom").unwrap();
    let v = eval_expr(&demo.expr, &primitive_env(), &mut UnitHost).unwrap();
    assert_eq!(
        field(&v, "tag"),
        &RuntimeValue::String("math-phantom-smash-demo".into())
    );

    let from_record = field(&v, "from-record");
    assert_eq!(
        field(from_record, "tag"),
        &RuntimeValue::String("math-phantom".into())
    );
    assert!((num(field(from_record, "width")) - 0.0).abs() < 1e-9);
    assert!(num(field(from_record, "height")) > 0.0 || num(field(from_record, "depth")) >= 0.0);

    let from_string = field(&v, "from-string");
    assert_eq!(
        field(from_string, "tag"),
        &RuntimeValue::String("math-smash".into())
    );
    assert!(num(field(from_string, "width")) > 0.0);
    assert!((num(field(from_string, "height")) - 0.0).abs() < 1e-9);
    assert!((num(field(from_string, "depth")) - 0.0).abs() < 1e-9);

    let from_box = field(&v, "from-math-box");
    assert_eq!(
        field(from_box, "tag"),
        &RuntimeValue::String("math-phantom".into())
    );
    assert!((num(field(from_box, "width")) - 0.0).abs() < 1e-9);
}
