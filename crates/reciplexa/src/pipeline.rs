//! Expand → typecheck → (optional effects) → lower.
//!
//! Keeping these as small functions (instead of one monolith) lets the GUI
//! preview without performing I/O, while the CLI/export path injects a handler.
//!
//! Package domain path: when source imports `graphics/*` or `document/*` and has
//! no top-level interim `(page …)`, the pipeline opts into package eval +
//! graphics/document value bridge instead of interim CST lower.
//!
//! Package ingest runs top-level effect typecheck, then Core typecheck after
//! package elaborate, then eval/bridge. Load failures use stage `package`;
//! static type mismatches use stage `type`.
//!
//! **S6b:** Interim keyword `(page)/(circle)/…` ingest is refused on the
//! production pipeline. Author with `(import graphics/…)(val main (page …))`.
//! Keyword tables remain behind `interim-surface` for fixture crates only.

use std::path::{Path, PathBuf};

use reciplexa_effect::{list_head_ident, run_source_effects, EffectError, EffectHandler, Value};
use reciplexa_lower::lower_source;
use reciplexa_macro::expand_source;
use reciplexa_package::{
    document_from_package_source, typecheck_package_source, LocalPackageIndex,
    PackageTypecheckError,
};
use reciplexa_scene::Document;
use reciplexa_syntax::{parse_source, SyntaxKind};
use reciplexa_types::{typecheck_source, typecheck_top_level_effects};

use crate::document_pipeline::document_snapshot_from_source;

/// Fail-fast error from one pipeline stage (macro / type / effect / lower).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineError {
    pub stage: &'static str,
    pub message: String,
}

impl PipelineError {
    fn new(stage: &'static str, message: impl Into<String>) -> Self {
        Self {
            stage,
            message: message.into(),
        }
    }

    /// Human-readable form for CLI / GUI status lines.
    pub fn display(&self) -> String {
        format!("{}: {}", self.stage, self.message)
    }
}

impl From<EffectError> for PipelineError {
    fn from(e: EffectError) -> Self {
        Self::new("effect", e.message)
    }
}

/// Macro-expand surface forms (`color-byte`, `(markup …)`, …). Input is not mutated.
pub fn expand(src: &str) -> Result<String, PipelineError> {
    expand_source(src).map_err(|e| PipelineError::new("macro", e.message))
}

/// Type-check an already-expanded buffer.
pub fn typecheck(expanded: &str) -> Result<(), PipelineError> {
    typecheck_source(expanded)
        .map(|_| ())
        .map_err(|e| PipelineError::new("type", format!("{} @{}..{}", e.message, e.start, e.end)))
}

/// Lower expanded source to a drawable [`Document`].
pub fn lower(expanded: &str) -> Result<Document, PipelineError> {
    lower_source(expanded).map_err(|e| PipelineError::new("lower", e.message))
}

/// Expand + typecheck + lower — **no** effect execution (safe for live preview).
///
/// Uses the package domain bridge when [`wants_package_graphics_path`] is true.
pub fn document_from_source(src: &str) -> Result<Document, PipelineError> {
    let expanded = expand(src)?;
    if wants_package_graphics_path(&expanded) {
        return document_from_package_domain(&expanded);
    }
    refuse_interim_if_required(&expanded)?;
    typecheck(&expanded)?;
    lower(&expanded)
}

fn env_flag_true(name: &str) -> bool {
    std::env::var(name)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Reject top-level interim `(page …)` sources (S6b; always on).
///
/// Package-shaped ingest (`(import graphics|document …)(val main …)`) is unaffected.
/// `RECIPLEXA_REQUIRE_PACKAGE` remains accepted as a no-op synonym for compatibility.
pub fn refuse_interim_if_required(expanded: &str) -> Result<(), PipelineError> {
    let _ = env_flag_true("RECIPLEXA_REQUIRE_PACKAGE");
    if has_top_level_interim_page(expanded) {
        return Err(PipelineError::new(
            "package",
            "interim top-level `(page …)` is retired; author with \
             `(import graphics/…)(val main (page …))`",
        ));
    }
    Ok(())
}

/// Whether to route ingest through the package domain bridge instead of interim CST lower.
///
/// Auto-detects package-shaped sources: `(import graphics` or `(import document` plus
/// `(val main` without a top-level interim `(page …)` head. Override with
/// `RECIPLEXA_PACKAGE_GRAPHICS=1` (force on) or `=0` (force off).
pub fn wants_package_graphics_path(expanded: &str) -> bool {
    if !has_package_domain_import(expanded) {
        return false;
    }
    if std::env::var("RECIPLEXA_PACKAGE_GRAPHICS")
        .map(|v| v == "0" || v.eq_ignore_ascii_case("false"))
        .unwrap_or(false)
    {
        return false;
    }
    if env_flag_true("RECIPLEXA_PACKAGE_GRAPHICS") {
        return true;
    }
    is_package_shaped_graphics_source(expanded)
}

fn has_package_domain_import(expanded: &str) -> bool {
    expanded.contains("(import graphics") || expanded.contains("(import document")
}

/// Package-shaped domain ingest: explicit `main` entry and no top-level interim `(page …)`.
pub fn is_package_shaped_graphics_source(expanded: &str) -> bool {
    expanded.contains("(val main") && !has_top_level_interim_page(expanded)
}

fn has_top_level_interim_page(src: &str) -> bool {
    let Ok(root) = parse_source(src).into_result() else {
        return false;
    };
    for form in root.children() {
        if form.kind() == SyntaxKind::StructuredComment {
            continue;
        }
        // Prefer list_head_ident: rowan `children()` skips tokens, so an Ident
        // head is never visible via `children().first()` on a List form.
        if list_head_ident(&form).is_some_and(|h| h == "page") {
            return true;
        }
    }
    false
}

fn package_search_roots() -> Vec<PathBuf> {
    package_search_roots_near(None)
}

fn push_packages_under(start: PathBuf, roots: &mut Vec<PathBuf>) {
    let mut dir = start;
    for _ in 0..8 {
        let packages = dir.join("packages");
        if packages.is_dir() {
            let canon = packages.canonicalize().unwrap_or(packages);
            if !roots.iter().any(|r| r == &canon) {
                roots.push(canon);
            }
            return;
        }
        if !dir.pop() {
            return;
        }
    }
}

/// Discover `packages/` from cwd / env, optionally walking up from an entry path.
pub(crate) fn package_search_roots_near(hint: Option<&Path>) -> Vec<PathBuf> {
    if let Ok(p) = std::env::var("RECIPLEXA_PACKAGE_ROOT") {
        return vec![PathBuf::from(p)];
    }
    let mut roots = Vec::new();
    if let Some(parent) = hint.and_then(|p| p.parent()) {
        push_packages_under(parent.to_path_buf(), &mut roots);
    }
    if roots.is_empty() {
        if let Ok(dir) = std::env::current_dir() {
            push_packages_under(dir, &mut roots);
        }
    }
    if roots.is_empty() {
        roots.push(PathBuf::from("packages"));
    }
    roots
}

fn document_from_package_domain(expanded: &str) -> Result<Document, PipelineError> {
    typecheck_top_level_effects(expanded).map_err(pipeline_type_error_from_surface)?;
    let package_src = strip_top_level_effect_forms(expanded);
    let search_roots = package_search_roots();
    let roots: Vec<&Path> = search_roots.iter().map(PathBuf::as_path).collect();
    let idx = LocalPackageIndex::discover(&roots)
        .map_err(|e| PipelineError::new("package", e.to_string()))?;
    typecheck_package_source(&package_src, "entry", &idx)
        .map_err(pipeline_type_error_from_package)?;
    document_from_package_source(&package_src, "entry", &idx)
        .map_err(|e| PipelineError::new("package", e.to_string()))
}

fn pipeline_type_error_from_surface(e: reciplexa_types::TypeError) -> PipelineError {
    PipelineError::new("type", format!("{} @{}..{}", e.message, e.start, e.end))
}

fn pipeline_type_error_from_package(e: PackageTypecheckError) -> PipelineError {
    match e {
        PackageTypecheckError::Load(load) => PipelineError::new("package", load.to_string()),
        PackageTypecheckError::Check(check) => {
            let start = check.range.start().get();
            let end = check.range.end().get();
            PipelineError::new("type", format!("{} @{start}..{end}", check.message))
        }
    }
}

/// Drop top-level `(perform …)` / `(handle …)` / `(src …)` before package elaboration.
fn strip_top_level_effect_forms(expanded: &str) -> String {
    let Ok(root) = parse_source(expanded).into_result() else {
        return expanded.to_string();
    };
    let mut kept = Vec::new();
    for form in root.children() {
        if form.kind() == SyntaxKind::StructuredComment {
            continue;
        }
        if is_top_level_effect_form(&form) {
            continue;
        }
        kept.push(form.to_string());
    }
    kept.join("\n")
}

fn is_top_level_effect_form(form: &reciplexa_syntax::SyntaxNode) -> bool {
    list_head_ident(form).is_some_and(|h| h == "perform" || h == "handle" || h == "src")
}

/// Pipeline output with optional editable document snapshot (Phase 4).
#[derive(Debug)]
pub struct PipelineDocument {
    pub scene: Document,
    pub editable: Option<reciplexa_document::DocumentSnapshot>,
}

/// Expand + typecheck + lower, optionally building an editable snapshot in parallel.
pub fn document_from_source_with_snapshot(
    src: &str,
    with_editable: bool,
) -> Result<PipelineDocument, PipelineError> {
    let expanded = expand(src)?;
    let scene = if wants_package_graphics_path(&expanded) {
        document_from_package_domain(&expanded)?
    } else {
        refuse_interim_if_required(&expanded)?;
        typecheck(&expanded)?;
        lower(&expanded)?
    };
    let editable = if with_editable {
        Some(
            document_snapshot_from_source(
                &expanded,
                reciplexa_identity::document::DocumentIdentity::new(1),
            )
            .map_err(|e| PipelineError::new("document", e))?,
        )
    } else {
        None
    };
    Ok(PipelineDocument { scene, editable })
}

/// Run top-level `(src …)` / `(handle …)` forms against a host handler.
pub fn run_effects(
    handler: &mut dyn EffectHandler,
    expanded: &str,
) -> Result<Vec<Value>, PipelineError> {
    run_source_effects(handler, expanded).map_err(PipelineError::from)
}

/// Full export path: expand → typecheck → effects → lower.
///
/// Returns the scene and the expanded source (useful for diagnostics).
pub fn document_for_export(
    handler: &mut dyn EffectHandler,
    src: &str,
) -> Result<(Document, String), PipelineError> {
    let expanded = expand(src)?;
    if wants_package_graphics_path(&expanded) {
        run_effects(handler, &expanded)?;
        let doc = document_from_package_domain(&expanded)?;
        return Ok((doc, expanded));
    }
    refuse_interim_if_required(&expanded)?;
    typecheck(&expanded)?;
    run_effects(handler, &expanded)?;
    let doc = lower(&expanded)?;
    Ok((doc, expanded))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_effect::TestHandler;
    use reciplexa_scene::Shape;

    // --- validity ---

    #[test]
    fn document_from_source_package_graphics_shapes_example() {
        let src = include_str!("../../../examples/shapes.rpx");
        let doc = document_from_source(src).expect("package graphics ingest");
        assert_eq!(doc.pages.len(), 1);
        assert!(doc.pages[0].paper.is_positive());
    }

    #[test]
    fn wants_package_graphics_path_detects_import_without_top_level_page() {
        let src = include_str!("../../../examples/shapes.rpx");
        let expanded = expand(src).unwrap();
        assert!(wants_package_graphics_path(&expanded));
        assert!(is_package_shaped_graphics_source(&expanded));
        let interim = expand("(page a4 (circle 1 2 3))").unwrap();
        assert!(!wants_package_graphics_path(&interim));
    }

    #[test]
    fn document_from_source_pkg_document_uses_package_domain_bridge() {
        let src = include_str!("../../../examples/pkg_document.rpx");
        let expanded = expand(src).unwrap();
        assert!(
            wants_package_graphics_path(&expanded),
            "document import should auto-route to package domain path"
        );
        let doc = document_from_source(src).expect("pkg_document via pipeline bridge");
        assert_eq!(doc.pages.len(), 1);
        assert!(doc.pages[0].paper.is_positive());
    }

    /// LL24: pkg_live_layout / pkg_live_math / phantom open via wants_package → package bridge.
    #[test]
    fn document_from_source_live_layout_demos_use_package_path() {
        for name in [
            "pkg_live_layout.rpx",
            "pkg_live_math.rpx",
            "pkg_live_math_phantom.rpx",
        ] {
            let src = match name {
                "pkg_live_layout.rpx" => include_str!("../../../examples/pkg_live_layout.rpx"),
                "pkg_live_math.rpx" => include_str!("../../../examples/pkg_live_math.rpx"),
                _ => include_str!("../../../examples/pkg_live_math_phantom.rpx"),
            };
            let expanded = expand(src).unwrap();
            assert!(
                wants_package_graphics_path(&expanded),
                "{name}: should auto-route to package domain path"
            );
            let doc =
                document_from_source(src).unwrap_or_else(|e| panic!("{name} pipeline open: {e:?}"));
            let texts = doc.pages[0]
                .shapes
                .iter()
                .filter(|s| matches!(s, Shape::Text(_)))
                .count();
            assert!(
                texts >= 2,
                "{name}: expected Text shapes via package path, got {texts}"
            );
        }
    }

    #[test]
    fn interim_page_fixture_is_refused_on_production_pipeline() {
        let src = include_str!("../../reciplexa-lower/tests/fixtures/interim_page.rpx");
        let expanded = expand(src).unwrap();
        assert!(
            !wants_package_graphics_path(&expanded),
            "interim fixture must not route through package domain bridge"
        );
        let err = document_from_source(src).expect_err("S6b refuses interim keyword page");
        assert_eq!(err.stage, "package");
        assert!(err.message.contains("retired") || err.message.contains("import graphics"));
    }

    #[test]
    fn black_circle_gui_golden_uses_package_path() {
        let src = include_str!("../../../examples/black_circle.rpx");
        let expanded = expand(src).unwrap();
        assert!(
            wants_package_graphics_path(&expanded),
            "GUI golden black_circle.rpx must be package-shaped"
        );
        let doc = document_from_source(src).expect("black_circle package bridge");
        assert_eq!(doc.pages.len(), 1);
    }

    #[test]
    fn pkg_black_circle_has_expected_circle_geometry() {
        let pkg = document_from_source(include_str!("../../../examples/black_circle.rpx"))
            .expect("package black_circle golden");
        assert!(wants_package_graphics_path(
            &expand(include_str!("../../../examples/black_circle.rpx")).unwrap()
        ));
        assert_eq!(pkg.pages.len(), 1);
        let Shape::Circle(pc) = &pkg.pages[0].shapes[0] else {
            panic!("pkg expected circle, got {:?}", pkg.pages[0].shapes[0]);
        };
        assert_eq!(pc.x_mm, 105.0);
        assert_eq!(pc.y_mm, 148.5);
        assert_eq!(pc.radius_mm, 40.0);
    }

    #[test]
    fn document_from_source_auto_detect_package_text_line() {
        use reciplexa_scene::Shape;

        fn leaf_shape_count(shapes: &[Shape]) -> usize {
            shapes
                .iter()
                .map(|s| match s {
                    Shape::Group { children, .. } | Shape::Opacity { children, .. } => {
                        leaf_shape_count(children)
                    }
                    _ => 1,
                })
                .sum()
        }

        let pkg_src = include_str!("../../../examples/text_line.rpx");
        let from_pkg = document_from_source(pkg_src).expect("package auto-detect");
        assert_eq!(from_pkg.pages.len(), 1);
        assert!(leaf_shape_count(&from_pkg.pages[0].shapes) >= 3);
    }

    #[test]
    fn document_from_source_effects_example_uses_package_bridge() {
        let src = include_str!("../../../examples/effects.rpx");
        let expanded = expand(src).unwrap();
        assert!(wants_package_graphics_path(&expanded));
        let doc = document_from_source(src).expect("effects package page");
        assert_eq!(doc.pages.len(), 1);
        assert!(doc.pages[0].paper.is_positive());
    }

    #[test]
    fn document_from_source_macros_example_expands_then_packages() {
        let src = include_str!("../../../examples/macros.rpx");
        let expanded = expand(src).unwrap();
        assert!(wants_package_graphics_path(&expanded));
        let doc = document_from_source(src).expect("macros package page");
        assert_eq!(doc.pages.len(), 1);
    }

    #[test]
    fn strip_top_level_effect_forms_keeps_package_main() {
        let expanded = expand(include_str!("../../../examples/effects.rpx")).unwrap();
        let stripped = strip_top_level_effect_forms(&expanded);
        assert!(!stripped.contains("(perform"));
        assert!(stripped.contains("(val main"));
        assert!(stripped.contains("(import graphics"));
    }

    #[test]
    fn document_from_source_expands_doc_title() {
        let doc = document_from_source("(markup @title{Hi})").unwrap();
        assert_eq!(doc.pages.len(), 1);
        let Shape::Text(t) = &doc.pages[0].shapes[0] else {
            panic!("expected text");
        };
        assert_eq!(t.content, "Hi");
        assert_eq!(t.size_mm, 14.0);
    }

    #[test]
    fn document_from_source_expands_doc_image() {
        let doc = document_from_source(r#"(markup @image["figures/demo.png"])"#).unwrap();
        assert_eq!(doc.pages.len(), 1);
        assert!(doc.pages[0]
            .shapes
            .iter()
            .any(|s| matches!(s, Shape::Image(_))));
    }

    const PKG_CIRCLE: &str = r#"
(import graphics/shapes only circle fill)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main (page a4 (fill (circle 1 2 3) black)))
"#;

    #[test]
    fn document_from_source_does_not_run_effects() {
        // Preview path must not touch the handler (export will).
        let mut h = TestHandler::default();
        let expanded = expand(&format!("(src (perform log \"x\"))\n{PKG_CIRCLE}")).unwrap();
        assert!(wants_package_graphics_path(&expanded));
        let _doc = document_from_package_domain(&strip_top_level_effect_forms(&expanded)).unwrap();
        assert!(h.logs.is_empty());
        run_effects(&mut h, &expanded).unwrap();
        assert_eq!(h.logs, vec!["x"]);
    }

    #[test]
    fn document_for_export_runs_handle_log() {
        let mut h = TestHandler::default();
        h.random_seq = vec![0.5];
        let src = format!(
            r#"
(src
  (perform log "a")
  (handle log (perform log "mute") (perform random))
  (perform log "b"))
{PKG_CIRCLE}
"#
        );
        let (doc, _) = document_for_export(&mut h, &src).unwrap();
        assert_eq!(doc.pages.len(), 1);
        assert_eq!(h.logs, vec!["a", "b"]);
    }

    #[test]
    fn document_for_export_mutes_write_path() {
        let mut h = TestHandler::default();
        let src = format!(
            r#"
(src
  (perform write-path "a.pdf")
  (handle write-path (perform write-path "mute.pdf"))
  (perform write-path "b.pdf"))
{PKG_CIRCLE}
"#
        );
        let (doc, _) = document_for_export(&mut h, &src).unwrap();
        assert_eq!(doc.pages.len(), 1);
        assert_eq!(h.writes, vec!["a.pdf", "b.pdf"]);
    }

    // --- defect ---

    #[test]
    fn interim_page_surfaces_package_stage() {
        let err = document_from_source("(page a4 (circle (circle 0 0 1) 0 1))").unwrap_err();
        assert_eq!(err.stage, "package");
        assert!(err.message.contains("retired") || err.message.contains("import"));
    }

    #[test]
    fn optional_snapshot_path_builds_editable() {
        let out = document_from_source_with_snapshot(PKG_CIRCLE, true).unwrap();
        assert_eq!(out.scene.pages.len(), 1);
        let snap = out.editable.expect("snapshot");
        assert!(snap.nodes.iter().count() >= 1);
    }

    #[test]
    fn optional_snapshot_skipped_by_default() {
        let out = document_from_source_with_snapshot(
            "(import graphics/page only a4 page)\n(val main (page a4 (list)))",
            false,
        )
        .unwrap();
        assert!(out.editable.is_none());
    }

    #[test]
    fn package_route_effect_type_error_is_type_stage() {
        let src = format!("(src (perform log 1))\n{PKG_CIRCLE}");
        let err = document_from_source(&src).unwrap_err();
        assert_eq!(err.stage, "type");
    }

    #[test]
    fn package_route_core_type_mismatch_is_type_stage() {
        let src = r#"
(import graphics/shapes only circle fill)
(import graphics/page only a4 page)
(import graphics/color only black)
(val main (page a4 (fill (circle 1 2) black)))
"#;
        let err = document_from_source(src).unwrap_err();
        assert_eq!(err.stage, "type", "{}", err.message);
    }

    #[test]
    fn package_route_unknown_import_is_package_stage() {
        let src = "(import graphics/no-such only x)\n(val main 1)\n";
        let err = document_from_source(src).unwrap_err();
        assert_eq!(err.stage, "package");
    }

    #[test]
    fn package_route_non_page_main_is_package_or_type_stage() {
        let src = "(import graphics/shapes only circle)\n(val main 42)\n";
        let err = document_from_source(src).unwrap_err();
        assert!(
            err.stage == "package" || err.stage == "type",
            "unexpected stage {}: {}",
            err.stage,
            err.message
        );
    }

    #[test]
    fn parse_error_is_macro_stage() {
        let err = expand("(page").unwrap_err();
        assert_eq!(err.stage, "macro");
    }

    #[test]
    fn pipeline_error_display() {
        let err = PipelineError::new("test", "message");
        assert_eq!(err.display(), "test: message");
    }

    #[test]
    fn lower_error_surfaces_retired_keyword() {
        let err = lower("(page a4 (bogus 1))").unwrap_err();
        assert_eq!(err.stage, "lower");
        assert!(err.message.contains("retired") || err.message.contains("interim"));
    }

    #[test]
    fn typecheck_standalone_rejects_interim_keywords() {
        let err = typecheck("(page a4 (circle 1 2 3))").unwrap_err();
        assert_eq!(err.stage, "type");
        assert!(err.message.contains("retired") || err.message.contains("interim"));
        assert!(typecheck("(src (perform log \"x\"))").is_ok());
    }

    #[test]
    fn expand_doc_title() {
        let expanded = expand("(markup @title{Hi})").unwrap();
        assert!(expanded.contains("Hi"));
    }

    #[test]
    fn document_from_source_with_effects_in_src_only() {
        let h = TestHandler::default();
        let doc =
            document_from_source(&format!("(src (perform log \"skip\"))\n{PKG_CIRCLE}")).unwrap();
        assert_eq!(doc.pages.len(), 1);
        assert!(h.logs.is_empty());
    }

    #[test]
    fn run_effects_unknown_op_uses_effect_stage() {
        let expanded = expand(&format!("(src (perform draw \"x\"))\n{PKG_CIRCLE}")).unwrap();
        let err = run_effects(&mut TestHandler::default(), &expanded).unwrap_err();
        assert_eq!(err.stage, "effect");
    }

    #[test]
    fn expand_empty_markup_still_parses() {
        let expanded = expand("(markup)").unwrap();
        assert!(expanded.contains("(page"));
        assert!(expanded.contains("(import graphics"));
    }

    #[test]
    fn document_for_export_interim_refused() {
        let err = document_for_export(&mut TestHandler::default(), "(page a4 (circle x 2 3))")
            .unwrap_err();
        assert_eq!(err.stage, "package");
    }

    #[test]
    fn document_snapshot_interim_refused() {
        let err = document_from_source_with_snapshot("(page a4 (circle x 2 3))", true).unwrap_err();
        assert_eq!(err.stage, "package");
    }

    #[test]
    fn pipeline_error_from_effect_error() {
        let e = EffectError {
            message: "boom".into(),
        };
        let err = PipelineError::from(e);
        assert_eq!(err.stage, "effect");
        assert!(err.message.contains("boom"));
    }

    #[test]
    fn round26_page_ident_strip_and_error_display() {
        // Ident-first-child page detection + structured-comment skip
        assert!(has_top_level_interim_page("(page a4 (circle 1 2 3))"));
        assert!(has_top_level_interim_page("(// note)\n(page a4)"));
        assert!(!has_top_level_interim_page("(val main 1)"));
        assert!(!has_top_level_interim_page("(")); // parse fail → false
        assert!(!has_top_level_interim_page(""));

        // strip: parse fail returns input unchanged; effect forms dropped
        let bad = "(";
        assert_eq!(strip_top_level_effect_forms(bad), bad);
        let stripped = strip_top_level_effect_forms(
            "(perform log \"x\")\n(handle ask (fn (m) m) 1)\n(src 1)\n(val main 1)",
        );
        assert!(!stripped.contains("perform"));
        assert!(stripped.contains("val main"));

        let pe = PipelineError::new("package", "x");
        assert!(!pe.display().is_empty());
        let _ = format!("{pe:?}");
    }
}
