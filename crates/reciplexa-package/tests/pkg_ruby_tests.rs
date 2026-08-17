//! Example `pkg_ruby.rpx` — ruby-box builtin + document/page surface.

use reciplexa_eval::RuntimeValue;
use reciplexa_package::{elaborate_with_packages, eval_elaborated_package_expr, LocalPackageIndex};
use reciplexa_std::japanese::Ruby;
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
fn pkg_ruby_example_elaborates_and_eval() {
    std::thread::Builder::new()
        .name("pkg-ruby-eval".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let entry =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_ruby.rpx");
            let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
            let units = elaborate_with_packages(&entry, &idx).unwrap();
            assert!(units.iter().any(|u| u.name == "pkg_ruby"));
            assert!(units.iter().any(|u| u.name == "document/page"));

            let demo = units.iter().find(|u| u.name == "pkg_ruby").unwrap();
            let v = eval_elaborated_package_expr(&demo.expr, &idx).unwrap();
            assert_eq!(
                field(&v, "tag"),
                &RuntimeValue::String("ja-ruby-demo".into())
            );

            let ruby = field(&v, "ruby");
            let advance = match field(ruby, "advance-width") {
                RuntimeValue::Number(n) => *n,
                other => panic!("advance-width number, got {other}"),
            };
            let expected = Ruby::simple("東京", "とうきょう").estimate_box();
            assert!((advance - expected.advance_width).abs() < 1e-9);

            let page = field(&v, "page");
            let page_s = format!("{page}");
            assert!(
                page_s.contains("doc-page") || page_s.contains("page"),
                "expected document page record, got {page_s}"
            );
        })
        .expect("spawn pkg-ruby-eval")
        .join()
        .expect("pkg-ruby-eval thread");
}
