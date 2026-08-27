//! Example `pkg_ja_profile_v2.rpx` — combined Japanese Document Profile v2 page.

use reciplexa_eval::RuntimeValue;
use reciplexa_package::{
    document_from_package_entry_host, document_from_package_entry_with_engine,
    elaborate_with_packages, eval_elaborated_package_expr, LocalPackageIndex,
};
use reciplexa_text_layout::TypesetEngine;
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

fn walk_shapes(
    shapes: &[reciplexa_scene::Shape],
    runs: &mut Vec<(String, f64, f64, f64)>,
    circles: &mut Vec<(f64, f64)>,
) {
    for s in shapes {
        match s {
            reciplexa_scene::Shape::GlyphRun(g) => {
                runs.push((g.content.clone(), g.x_mm, g.y_mm, g.size_mm));
            }
            reciplexa_scene::Shape::Circle(c) => circles.push((c.x_mm, c.y_mm)),
            reciplexa_scene::Shape::Group { children, .. }
            | reciplexa_scene::Shape::Opacity { children, .. } => {
                walk_shapes(children, runs, circles);
            }
            _ => {}
        }
    }
}

#[test]
fn pkg_ja_profile_v2_example_elaborates_and_eval() {
    std::thread::Builder::new()
        .name("pkg-ja-profile-eval".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/pkg_ja_profile_v2.rpx");
            let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
            let units = elaborate_with_packages(&entry, &idx).unwrap();
            assert!(units.iter().any(|u| u.name == "pkg_ja_profile_v2"));
            let demo = units
                .iter()
                .find(|u| u.name == "pkg_ja_profile_v2")
                .unwrap();
            let v = eval_elaborated_package_expr(&demo.expr, &idx).unwrap();
            assert_eq!(
                field(&v, "tag"),
                &RuntimeValue::String("ja-profile-v2-demo".into())
            );
            assert!(matches!(field(&v, "tcy"), RuntimeValue::String(s) if s == "12"));
            assert!(matches!(field(&v, "bou"), RuntimeValue::String(s) if s == "重要"));
        })
        .expect("spawn pkg-ja-profile-eval")
        .join()
        .expect("pkg-ja-profile-eval thread");
}

#[test]
fn pkg_ja_profile_v2_host_product_combines_features() {
    std::thread::Builder::new()
        .name("pkg-ja-profile-host".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/pkg_ja_profile_v2.rpx");
            let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
            let stub = document_from_package_entry_with_engine(&entry, &idx, TypesetEngine::Stub)
                .expect("stub nested page");
            assert!(stub.pages[0]
                .shapes
                .iter()
                .any(|s| s.text_content().is_some()));
            let host = document_from_package_entry_host(&entry, &idx).expect("host profile");
            let mut runs = Vec::new();
            let mut circles = Vec::new();
            walk_shapes(&host.pages[0].shapes, &mut runs, &mut circles);
            let joined: String = runs.iter().map(|(c, _, _, _)| c.as_str()).collect();
            assert!(
                joined.contains("Japanese Profile v2") || joined.contains("横組ルビ"),
                "nested page caption: {runs:?}"
            );
            assert!(
                joined.contains('東') && joined.contains('と'),
                "horizontal/vertical ruby: {runs:?}"
            );
            assert!(
                joined.contains('縦') && joined.contains('。') && joined.contains('A'),
                "vertical samples: {runs:?}"
            );
            assert!(
                joined.contains('令') && joined.contains('1') && joined.contains('2'),
                "tate-chu-yoko column: {runs:?}"
            );
            assert!(
                joined.contains('重') && joined.contains('要'),
                "bou body: {runs:?}"
            );
            assert!(circles.len() >= 2, "bou marks as circles, got {circles:?}");
            let pdf =
                reciplexa_pdf::document_to_pdf_with_host_fonts(&host, None).expect("profile pdf");
            assert!(pdf.starts_with(b"%PDF"));
        })
        .expect("spawn pkg-ja-profile-host")
        .join()
        .expect("pkg-ja-profile-host thread");
}
