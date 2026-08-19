//! Example `pkg_ruby.rpx` — ruby-box builtin + document/page surface.

use reciplexa_eval::RuntimeValue;
use reciplexa_package::{
    document_from_package_entry_host, document_from_package_entry_with_engine,
    elaborate_with_packages, eval_elaborated_package_expr, LocalPackageIndex,
};
use reciplexa_std::japanese::Ruby;
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

            let placed = field(&v, "placed");
            assert_eq!(
                field(placed, "tag"),
                &RuntimeValue::String("ja-ruby".into())
            );

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

#[test]
fn pkg_ruby_host_product_emits_glyph_runs() {
    std::thread::Builder::new()
        .name("pkg-ruby-host".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let entry =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/pkg_ruby.rpx");
            let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
            let stub = document_from_package_entry_with_engine(&entry, &idx, TypesetEngine::Stub)
                .expect("stub nested page");
            assert!(stub.pages[0]
                .shapes
                .iter()
                .any(|s| s.text_content().is_some()));
            let host = document_from_package_entry_host(&entry, &idx).expect("host ruby");
            let runs: Vec<_> = host.pages[0]
                .shapes
                .iter()
                .filter_map(|s| match s {
                    reciplexa_scene::Shape::GlyphRun(g) => Some(g.content.as_str()),
                    _ => None,
                })
                .collect();
            let joined: String = runs.concat();
            assert!(
                joined.contains('東') && joined.contains('と') && joined.contains('漢'),
                "heading-adjacent ruby samples: {runs:?}"
            );
            assert!(runs.iter().any(|c| *c == "東" || *c == "京"));
            assert!(
                joined.contains("Simple ruby") || joined.contains("親文字"),
                "nested page caption should remain on the product page, got {runs:?}"
            );
            let east = host.pages[0]
                .shapes
                .iter()
                .find_map(|s| match s {
                    reciplexa_scene::Shape::GlyphRun(g) if g.content == "東" => Some(g),
                    _ => None,
                })
                .expect("base 東");
            let to = host.pages[0]
                .shapes
                .iter()
                .find_map(|s| match s {
                    reciplexa_scene::Shape::GlyphRun(g)
                        if g.content == "と" && (g.size_mm - east.size_mm * 0.5).abs() < 1e-6 =>
                    {
                        Some(g)
                    }
                    _ => None,
                })
                .expect("annotation と");
            assert!(
                to.y_mm + 1e-9 >= east.y_mm + east.size_mm,
                "annotation must sit above the parent em-square: base y={} size={} ann y={}",
                east.y_mm,
                east.size_mm,
                to.y_mm
            );
            let pdf =
                reciplexa_pdf::document_to_pdf_with_host_fonts(&host, None).expect("ruby pdf");
            assert!(pdf.starts_with(b"%PDF"));
        })
        .expect("spawn pkg-ruby-host")
        .join()
        .expect("pkg-ruby-host thread");
}
