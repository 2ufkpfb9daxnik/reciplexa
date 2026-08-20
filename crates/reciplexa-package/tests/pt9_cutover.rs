//! PT-9: declared examples through stub vs product engines.

use std::path::PathBuf;

use reciplexa_package::{
    document_from_package_entry, document_from_package_entry_host,
    document_from_package_entry_with_engine, eval_package_entry_main, LocalPackageIndex,
};
use reciplexa_text_layout::TypesetEngine;

fn on_host_stack<T, F>(name: &str, f: F) -> T
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    std::thread::Builder::new()
        .name(name.into())
        .stack_size(8 * 1024 * 1024)
        .spawn(f)
        .expect("spawn host stack")
        .join()
        .unwrap_or_else(|_| panic!("{name} panicked"))
}

fn workspace_packages() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages")
}

fn examples() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn index() -> LocalPackageIndex {
    LocalPackageIndex::discover(&[workspace_packages()]).expect("discover packages/")
}

fn example(name: &str) -> PathBuf {
    examples().join(name)
}

fn text_count(doc: &reciplexa_scene::Document) -> usize {
    doc.pages
        .iter()
        .flat_map(|p| p.shapes.iter())
        .filter(|s| s.is_text_like())
        .count()
}

#[test]
fn pt9_pkg_columns_and_indent_host_product() {
    on_host_stack("pt9-columns", || {
        let idx = index();
        for name in ["pkg_columns.rpx", "pkg_document_indent.rpx"] {
            let entry = example(name);
            let stub =
                document_from_package_entry(&entry, &idx).unwrap_or_else(|e| panic!("{name}: {e}"));
            let product =
                document_from_package_entry_with_engine(&entry, &idx, TypesetEngine::Product)
                    .unwrap_or_else(|e| panic!("{name} product: {e}"));
            let host = document_from_package_entry_host(&entry, &idx)
                .unwrap_or_else(|e| panic!("{name} host: {e}"));
            assert!(text_count(&stub) >= 1, "{name} stub");
            assert!(text_count(&product) >= 1, "{name} product");
            assert_eq!(
                text_count(&host),
                text_count(&product),
                "{name} host==product"
            );
            assert!(
                host.pages[0]
                    .shapes
                    .iter()
                    .any(|s| s.text_content().is_some_and(|c| !c.is_empty())),
                "{name} host has text"
            );
        }
    });
}

#[test]
fn pt9_pkg_live_math_product_keeps_delimiter_and_fraction() {
    on_host_stack("pt9-live-math", || {
        let idx = index();
        let entry = example("pkg_live_math.rpx");
        let stub = document_from_package_entry(&entry, &idx).expect("live math stub");
        let product = document_from_package_entry_with_engine(&entry, &idx, TypesetEngine::Product)
            .expect("live math product");
        let texts: Vec<_> = product.pages[0]
            .shapes
            .iter()
            .filter_map(|s| s.text_content())
            .collect();
        let joined: String = texts.concat();
        assert!(
            joined.contains('括') || joined.contains("Live"),
            "JA paragraph survives: {texts:?}"
        );
        assert!(
            texts.contains(&"a") && texts.contains(&"b"),
            "fraction glyphs placed separately: {texts:?}"
        );
        let ja_min = product.pages[0]
            .shapes
            .iter()
            .filter(|s| s.text_content().is_some_and(|c| c.contains('括')))
            .filter_map(|s| s.text_y_mm())
            .fold(f64::INFINITY, f64::min);
        let math_left = product.pages[0]
            .shapes
            .iter()
            .find(|s| s.text_content() == Some("("))
            .and_then(|s| s.text_y_mm())
            .expect("math left delimiter");
        const MATH_BELOW_DOC_GAP_MM: f64 = 12.0;
        assert!(
            ja_min.is_finite(),
            "product JA GlyphRun baseline missing: {texts:?}"
        );
        assert!(
            (math_left - (ja_min - MATH_BELOW_DOC_GAP_MM)).abs() < 1.0,
            "math origin must follow JA GlyphRun baseline, not Text-only fallback; math_left={math_left} ja_min={ja_min}"
        );
        assert!(
            math_left < ja_min,
            "math sits below JA in page Y-up; math_left={math_left} ja_min={ja_min}"
        );
        let _ = stub;
    });
}

#[test]
fn pt9_pkg_math_demo_uses_product_engine() {
    on_host_stack("pt9-pkg-math", || {
        let idx = index();
        let entry = example("pkg_math.rpx");
        let doc = document_from_package_entry_with_engine(&entry, &idx, TypesetEngine::Product)
            .expect("pkg_math product page");
        assert_eq!(doc.pages.len(), 1);
        let n = text_count(&doc);
        assert!(n > 5, "math-demo should emit multiple glyphs, got {n}");
        let contents: Vec<_> = doc.pages[0]
            .shapes
            .iter()
            .filter_map(|s| s.text_content())
            .collect();
        assert!(contents.contains(&"x"), "expected x in {contents:?}");
    });
}

#[test]
fn pt9_pkg_math_spacing_and_jlreq_still_eval() {
    on_host_stack("pt9-eval-demos", || {
        let idx = index();
        let spacing = eval_package_entry_main(example("pkg_math_spacing.rpx"), &idx)
            .expect("pkg_math_spacing eval");
        match spacing {
            reciplexa_eval::RuntimeValue::Record(fields) => {
                let tag = fields
                    .iter()
                    .find(|(k, _)| k == "tag")
                    .and_then(|(_, v)| match v {
                        reciplexa_eval::RuntimeValue::String(s)
                        | reciplexa_eval::RuntimeValue::ShapeTag(s) => Some(s.as_str()),
                        _ => None,
                    });
                assert_eq!(tag, Some("math-spacing-demo"));
            }
            other => panic!("expected record, got {other:?}"),
        }
        let jlreq = eval_package_entry_main(example("pkg_japanese_jlreq.rpx"), &idx)
            .expect("pkg_japanese_jlreq eval");
        match jlreq {
            reciplexa_eval::RuntimeValue::Record(fields) => {
                let tag = fields
                    .iter()
                    .find(|(k, _)| k == "tag")
                    .and_then(|(_, v)| match v {
                        reciplexa_eval::RuntimeValue::String(s)
                        | reciplexa_eval::RuntimeValue::ShapeTag(s) => Some(s.as_str()),
                        _ => None,
                    });
                assert_eq!(tag, Some("ja-jlreq-demo"));
            }
            other => panic!("expected record, got {other:?}"),
        }
    });
}

#[test]
fn pt9_pkg_math_spacing_row_product_layout() {
    use reciplexa_identity::document::StableNodeId;
    use reciplexa_std::math::{EstimateStyle, MathAtom, MathClass};
    use reciplexa_text_layout::{layout_math_atom, LoadedFont};

    let font = LoadedFont::fixture();
    let row = MathAtom::row(
        StableNodeId::new(1),
        vec![
            MathAtom::symbol(StableNodeId::new(2), "a", MathClass::Ordinary),
            MathAtom::symbol(StableNodeId::new(3), "+", MathClass::Binary),
            MathAtom::symbol(StableNodeId::new(4), "b", MathClass::Ordinary),
        ],
    );
    let laid = layout_math_atom(&font, &row, EstimateStyle::Display).expect("row");
    let xs: Vec<_> = laid.glyphs.iter().map(|g| (g.glyph.ch, g.x_em)).collect();
    let a = xs.iter().find(|(c, _)| *c == 'a').unwrap();
    let plus = xs.iter().find(|(c, _)| *c == '+').unwrap();
    assert!(plus.1 > a.1, "class spacing / advances: {xs:?}");
}

#[test]
fn pt9_declared_page_examples_export_pdf_svg() {
    on_host_stack("pt9-export", || {
        let idx = index();
        for name in [
            "pkg_columns.rpx",
            "pkg_document_indent.rpx",
            "pkg_live_math.rpx",
            "pkg_math.rpx",
            "text_line.rpx",
        ] {
            let doc = document_from_package_entry_host(example(name), &idx)
                .unwrap_or_else(|e| panic!("{name} host: {e}"));
            let pdf = reciplexa_pdf::document_to_pdf_with_host_fonts(&doc, None)
                .unwrap_or_else(|e| panic!("{name} pdf: {e:?}"));
            assert!(pdf.starts_with(b"%PDF"), "{name} pdf header");
            let svg =
                reciplexa_svg::document_to_svg(&doc).unwrap_or_else(|e| panic!("{name} svg: {e}"));
            assert!(svg.contains("<svg"), "{name} svg");
            assert!(svg.contains("<text"), "{name} svg text");
        }
    });
}

#[test]
fn pt9_text_line_host_is_one_cluster_glyph_run() {
    on_host_stack("pt9-text-line", || {
        let idx = index();
        let entry = example("text_line.rpx");
        let stub = document_from_package_entry(&entry, &idx).expect("stub");
        let product = document_from_package_entry_with_engine(&entry, &idx, TypesetEngine::Product)
            .expect("product");
        let stub_text = stub.pages[0]
            .shapes
            .iter()
            .flat_map(walk_text)
            .find(|(_, c)| *c == "Reciplexa");
        assert!(
            matches!(stub_text, Some((false, _))),
            "stub stays Shape::Text"
        );
        let runs: Vec<_> = product.pages[0]
            .shapes
            .iter()
            .flat_map(walk_glyph_runs)
            .collect();
        assert_eq!(runs.len(), 1, "one authoring text → one GlyphRun");
        assert_eq!(runs[0].0, "Reciplexa");
        assert_eq!(runs[0].1, "Reciplexa".chars().count());
        let host = document_from_package_entry_host(&entry, &idx).expect("host");
        let host_runs: Vec<_> = host.pages[0]
            .shapes
            .iter()
            .flat_map(walk_glyph_runs)
            .collect();
        assert_eq!(host_runs.len(), runs.len());
    });
}

fn walk_text(shape: &reciplexa_scene::Shape) -> Vec<(bool, &str)> {
    match shape {
        reciplexa_scene::Shape::Text(t) => vec![(false, t.content.as_str())],
        reciplexa_scene::Shape::GlyphRun(g) => vec![(true, g.content.as_str())],
        reciplexa_scene::Shape::Group { children, .. }
        | reciplexa_scene::Shape::Opacity { children, .. } => {
            children.iter().flat_map(walk_text).collect()
        }
        _ => Vec::new(),
    }
}

fn walk_glyph_runs(shape: &reciplexa_scene::Shape) -> Vec<(&str, usize)> {
    match shape {
        reciplexa_scene::Shape::GlyphRun(g) => vec![(g.content.as_str(), g.gids.len())],
        reciplexa_scene::Shape::Group { children, .. }
        | reciplexa_scene::Shape::Opacity { children, .. } => {
            children.iter().flat_map(walk_glyph_runs).collect()
        }
        _ => Vec::new(),
    }
}
