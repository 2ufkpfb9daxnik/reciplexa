//! Example `pkg_math_profile_v2.rpx` — combined Math Profile v2 page.

use reciplexa_eval::RuntimeValue;
use reciplexa_package::{
    document_from_package_entry_host, document_from_package_entry_with_engine,
    elaborate_with_packages, eval_elaborated_package_expr, LocalPackageIndex,
};
use reciplexa_scene::Shape;
use reciplexa_text_layout::{LoadedFont, TypesetEngine};
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

fn walk_runs(shapes: &[Shape], out: &mut Vec<(String, f64, u16)>) {
    for s in shapes {
        match s {
            Shape::GlyphRun(g) => {
                let gid = g.gids.first().copied().unwrap_or(0);
                out.push((g.content.clone(), g.x_mm, gid));
            }
            Shape::Group { children, .. } | Shape::Opacity { children, .. } => {
                walk_runs(children, out);
            }
            _ => {}
        }
    }
}

#[test]
fn pkg_math_profile_v2_example_elaborates_and_eval() {
    std::thread::Builder::new()
        .name("pkg-math-profile-eval".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/pkg_math_profile_v2.rpx");
            let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
            let units = elaborate_with_packages(&entry, &idx).unwrap();
            assert!(units.iter().any(|u| u.name == "pkg_math_profile_v2"));
            let demo = units
                .iter()
                .find(|u| u.name == "pkg_math_profile_v2")
                .unwrap();
            let v = eval_elaborated_package_expr(&demo.expr, &idx).unwrap();
            assert_eq!(
                field(&v, "tag"),
                &RuntimeValue::String("live-layout-demo".into())
            );
            match field(&v, "style") {
                RuntimeValue::String(s) | RuntimeValue::ShapeTag(s) => {
                    assert_eq!(s, "display");
                }
                other => panic!("style field: {other}"),
            }
        })
        .expect("spawn pkg-math-profile-eval")
        .join()
        .expect("pkg-math-profile-eval thread");
}

#[test]
fn pkg_math_profile_v2_host_product_combines_features() {
    std::thread::Builder::new()
        .name("pkg-math-profile-host".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let entry = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/pkg_math_profile_v2.rpx");
            let idx = LocalPackageIndex::discover(&[workspace_packages()]).unwrap();
            let stub = document_from_package_entry_with_engine(&entry, &idx, TypesetEngine::Stub)
                .expect("stub nested page");
            assert!(stub.pages[0]
                .shapes
                .iter()
                .any(|s| s.text_content().is_some()));
            let host = document_from_package_entry_host(&entry, &idx).expect("host profile");
            let mut runs = Vec::new();
            walk_runs(&host.pages[0].shapes, &mut runs);
            let joined: String = runs.iter().map(|(c, _, _)| c.as_str()).collect();
            assert!(
                joined.contains("Math Profile v2") || joined.contains("assembly"),
                "nested page caption: {runs:?}"
            );
            let cmap = LoadedFont::fixture().glyph_id('(').expect("cmap (");
            let assembled: Vec<_> = runs
                .iter()
                .filter(|(c, _, gid)| c == "(" && *gid != cmap)
                .collect();
            assert!(
                assembled.len() > 1,
                "assembly left parts on the page, got {assembled:?} from {runs:?}"
            );
            let gids: Vec<u16> = assembled.iter().map(|(_, _, gid)| *gid).collect();
            assert!(
                gids.windows(2).any(|w| w[0] != w[1]),
                "assembly should mix GIDs, got {gids:?}"
            );
            assert!(
                joined.contains('f') && joined.contains('2') && joined.contains('y'),
                "kern / script row: {joined}"
            );
            let eqs: Vec<_> = runs.iter().filter(|(c, _, _)| c == "=").collect();
            assert_eq!(eqs.len(), 2, "two aligned relations: {runs:?}");
            assert!(
                (eqs[0].1 - eqs[1].1).abs() < 1e-6,
                "aligned = column x: {eqs:?}"
            );
            assert!(
                joined.contains('1') && joined.contains('p') && joined.contains('q'),
                "eqno + inline fraction: {joined}"
            );
            let pdf =
                reciplexa_pdf::document_to_pdf_with_host_fonts(&host, None).expect("profile pdf");
            assert!(pdf.starts_with(b"%PDF"));
        })
        .expect("spawn pkg-math-profile-host")
        .join()
        .expect("pkg-math-profile-host thread");
}
