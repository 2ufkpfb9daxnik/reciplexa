//! Example `pkg_vert.rpx` — font-backed vertical-rl columns.

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

fn walk_glyph_runs(shapes: &[reciplexa_scene::Shape], out: &mut Vec<(String, f64, f64)>) {
    for s in shapes {
        match s {
            reciplexa_scene::Shape::GlyphRun(g) => {
                out.push((g.content.clone(), g.x_mm, g.y_mm));
            }
            reciplexa_scene::Shape::Group { children, .. }
            | reciplexa_scene::Shape::Opacity { children, .. } => {
                walk_glyph_runs(children, out);
            }
            _ => {}
        }
    }
}

#[test]
fn pkg_vert_example_elaborates_and_eval() {
    std::thread::Builder::new()
        .name("pkg-vert-eval".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let entry =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_vert.rpx");
            let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
            let units = elaborate_with_packages(&entry, &idx).unwrap();
            assert!(units.iter().any(|u| u.name == "pkg_vert"));
            let demo = units.iter().find(|u| u.name == "pkg_vert").unwrap();
            let v = eval_elaborated_package_expr(&demo.expr, &idx).unwrap();
            assert_eq!(
                field(&v, "tag"),
                &RuntimeValue::String("ja-vertical-demo".into())
            );
            assert!(matches!(field(&v, "placed"), RuntimeValue::String(s) if s == "縦書き"));
        })
        .expect("spawn pkg-vert-eval")
        .join()
        .expect("pkg-vert-eval thread");
}

#[test]
fn pkg_vert_host_product_stacks_columns() {
    std::thread::Builder::new()
        .name("pkg-vert-host".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let entry =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_vert.rpx");
            let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
            let stub = document_from_package_entry_with_engine(&entry, &idx, TypesetEngine::Stub)
                .expect("stub nested page");
            assert!(stub.pages[0]
                .shapes
                .iter()
                .any(|s| s.text_content().is_some()));
            let host = document_from_package_entry_host(&entry, &idx).expect("host vert");
            let mut runs = Vec::new();
            walk_glyph_runs(&host.pages[0].shapes, &mut runs);
            let joined: String = runs.iter().map(|(c, _, _)| c.as_str()).collect();
            assert!(
                joined.contains('縦') && joined.contains('東') && joined.contains('A'),
                "vertical samples: {runs:?}"
            );
            let tate: Vec<_> = runs
                .iter()
                .filter(|(c, _, _)| c == "縦" || c == "書")
                .collect();
            assert!(tate.len() >= 2, "縦書き stacked: {runs:?}");
            if let (Some((_, _, y0)), Some((_, _, y1))) = (tate.first(), tate.get(1)) {
                assert!(y1 < y0, "second CJK must sit below the first");
            }
            let pdf =
                reciplexa_pdf::document_to_pdf_with_host_fonts(&host, None).expect("vert pdf");
            assert!(pdf.starts_with(b"%PDF"));
        })
        .expect("spawn pkg-vert-host")
        .join()
        .expect("pkg-vert-host thread");
}
