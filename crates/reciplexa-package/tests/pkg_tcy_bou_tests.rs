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

fn run_y(runs: &[(String, f64, f64, f64)], ch: &str) -> f64 {
    runs.iter()
        .find(|(c, _, _, _)| c == ch)
        .map(|(_, _, y, _)| *y)
        .unwrap_or_else(|| panic!("missing {ch} in {runs:?}"))
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
            let paper_h = host.pages[0].paper.height_mm;
            let mut runs = Vec::new();
            let mut circles = Vec::new();
            walk_shapes(&host.pages[0].shapes, &mut runs, &mut circles);
            let joined: String = runs.iter().map(|(c, _, _, _)| c.as_str()).collect();
            assert!(
                joined.contains('令') && joined.contains('1') && joined.contains('重'),
                "tcy column and bou body: {runs:?}"
            );
            let title_t_ys: Vec<f64> = runs
                .iter()
                .filter(|(c, _, _, _)| c == "T")
                .map(|(_, _, y, _)| *y)
                .collect();
            assert_eq!(
                title_t_ys.len(),
                1,
                "section title must not be painted twice, T ys={title_t_ys:?}"
            );
            let heading_y = title_t_ys[0];
            let rei_y = run_y(&runs, "令");
            let year_y = run_y(&runs, "年");
            let one = runs.iter().find(|(c, _, _, _)| c == "1").expect("digit 1");
            let two = runs.iter().find(|(c, _, _, _)| c == "2").expect("digit 2");
            for (ch, y) in [("T", heading_y), ("令", rei_y), ("1", one.2), ("年", year_y)] {
                assert!(
                    y > 20.0 && y < paper_h - 10.0,
                    "{ch} must stay on A4, y={y} paper_h={paper_h}"
                );
            }
            assert!(
                heading_y > 250.0,
                "English heading belongs at the top, y={heading_y}"
            );
            assert!(
                rei_y < heading_y && rei_y > 100.0,
                "令和 must sit below the heading and not at the page foot, 令 y={rei_y} heading y={heading_y}"
            );
            assert!(
                (one.2 - two.2).abs() < 1e-6,
                "tate-chu-yoko digits share a baseline, got {} vs {}",
                one.2,
                two.2
            );
            assert!(
                two.1 > one.1,
                "digits must sit side by side, not stacked"
            );
            assert!(
                circles.len() >= 4,
                "horizontal + vertical bou marks, got {}",
                circles.len()
            );
            let juu: Vec<_> = runs.iter().filter(|(c, _, _, _)| c == "重").collect();
            assert_eq!(juu.len(), 2, "horizontal and vertical 重, got {juu:?}");
            let (horiz, vert) = if juu[0].1 < juu[1].1 {
                (juu[0], juu[1])
            } else {
                (juu[1], juu[0])
            };
            let size = horiz.3;
            assert!(
                circles.iter().any(|(cx, cy)| {
                    (*cx - (horiz.1 + size * 0.5)).abs() < size * 0.6 && *cy > horiz.2 + size
                }),
                "horizontal bou must sit above 重, glyph={horiz:?} circles={circles:?}"
            );
            assert!(
                circles.iter().any(|(cx, cy)| {
                    *cx > vert.1 + size && (*cy - (vert.2 + size * 0.5)).abs() < size * 0.6
                }),
                "vertical bou must sit beside 重, glyph={vert:?} circles={circles:?}"
            );
            let pdf = reciplexa_pdf::document_to_pdf_with_host_fonts(&host, None).expect("pdf");
            assert!(pdf.starts_with(b"%PDF"));
        })
        .expect("spawn pkg-tcy-host")
        .join()
        .expect("pkg-tcy-host thread");
}
