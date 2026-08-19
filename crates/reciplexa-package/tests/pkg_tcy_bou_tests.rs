//! Example `pkg_tcy_bou.rpx` — tate-chu-yoko + bou product consume.

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

fn walk_shapes(shapes: &[reciplexa_scene::Shape], runs: &mut Vec<String>, circles: &mut usize) {
    for s in shapes {
        match s {
            reciplexa_scene::Shape::GlyphRun(g) => runs.push(g.content.clone()),
            reciplexa_scene::Shape::Circle(_) => *circles += 1,
            reciplexa_scene::Shape::Group { children, .. }
            | reciplexa_scene::Shape::Opacity { children, .. } => {
                walk_shapes(children, runs, circles);
            }
            _ => {}
        }
    }
}

#[test]
fn pkg_tcy_bou_example_elaborates_and_eval() {
    std::thread::Builder::new()
        .name("pkg-tcy-eval".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let entry =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_tcy_bou.rpx");
            let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
            let units = elaborate_with_packages(&entry, &idx).unwrap();
            assert!(units.iter().any(|u| u.name == "pkg_tcy_bou"));
            let demo = units.iter().find(|u| u.name == "pkg_tcy_bou").unwrap();
            let v = eval_elaborated_package_expr(&demo.expr, &idx).unwrap();
            assert_eq!(
                field(&v, "tag"),
                &RuntimeValue::String("ja-tcy-bou-demo".into())
            );
            assert!(matches!(field(&v, "tcy"), RuntimeValue::String(s) if s == "12"));
        })
        .expect("spawn pkg-tcy-eval")
        .join()
        .expect("pkg-tcy-eval thread");
}

#[test]
fn pkg_tcy_bou_host_product_emits_tcy_and_marks() {
    std::thread::Builder::new()
        .name("pkg-tcy-host".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let entry =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_tcy_bou.rpx");
            let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
            let stub = document_from_package_entry_with_engine(&entry, &idx, TypesetEngine::Stub)
                .expect("stub nested page");
            assert!(stub.pages[0]
                .shapes
                .iter()
                .any(|s| s.text_content().is_some()));
            let host = document_from_package_entry_host(&entry, &idx).expect("host tcy/bou");
            let mut runs = Vec::new();
            let mut circles = 0usize;
            walk_shapes(&host.pages[0].shapes, &mut runs, &mut circles);
            let joined: String = runs.concat();
            assert!(
                joined.contains('令') && joined.contains('1') && joined.contains('重'),
                "tcy column and bou body: {runs:?}"
            );
            let ones: Vec<_> = host.pages[0]
                .shapes
                .iter()
                .filter_map(|s| match s {
                    reciplexa_scene::Shape::GlyphRun(g) if g.content == "1" => Some(g),
                    _ => None,
                })
                .collect();
            let twos: Vec<_> = host.pages[0]
                .shapes
                .iter()
                .filter_map(|s| match s {
                    reciplexa_scene::Shape::GlyphRun(g) if g.content == "2" => Some(g),
                    _ => None,
                })
                .collect();
            assert!(!ones.is_empty() && !twos.is_empty());
            let (one, two) = (ones[0], twos[0]);
            assert!(
                (one.y_mm - two.y_mm).abs() < 1e-6,
                "tate-chu-yoko digits share a baseline, got {} vs {}",
                one.y_mm,
                two.y_mm
            );
            assert!(
                two.x_mm > one.x_mm,
                "digits must sit side by side, not stacked"
            );
            assert!(
                circles >= 4,
                "horizontal + vertical bou marks, got {circles}"
            );
            let pdf = reciplexa_pdf::document_to_pdf_with_host_fonts(&host, None).expect("pdf");
            assert!(pdf.starts_with(b"%PDF"));
        })
        .expect("spawn pkg-tcy-host")
        .join()
        .expect("pkg-tcy-host thread");
}
