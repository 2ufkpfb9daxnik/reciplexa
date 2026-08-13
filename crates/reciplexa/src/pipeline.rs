//! Expand → typecheck → (optional effects) → lower.
//!
//! Keeping these as small functions (instead of one monolith) lets the GUI
//! preview without performing I/O, while the CLI/export path injects a handler.
//!
//! Slice D: when source imports `graphics/*` and has no top-level interim `(page …)`,
//! the pipeline can opt into package eval + `document_from_graphics_value` instead.

use std::path::{Path, PathBuf};

use reciplexa_effect::{list_head_ident, run_source_effects, EffectError, EffectHandler, Value};
use reciplexa_lower::lower_source;
use reciplexa_macro::expand_source;
use reciplexa_package::{document_from_package_source, LocalPackageIndex};
use reciplexa_scene::Document;
use reciplexa_syntax::{parse_source, SyntaxKind};
use reciplexa_types::typecheck_source;

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
/// Uses the package graphics bridge when [`wants_package_graphics_path`] is true.
pub fn document_from_source(src: &str) -> Result<Document, PipelineError> {
    let expanded = expand(src)?;
    if wants_package_graphics_path(&expanded) {
        return document_from_package_graphics(&expanded);
    }
    typecheck(&expanded)?;
    lower(&expanded)
}

/// Whether to route document ingest through package graphics eval instead of interim CST lower.
///
/// Auto-detects package-shaped sources: `(import graphics` + `(val main` without a top-level
/// interim `(page …)` head. Override with `RECIPLEXA_PACKAGE_GRAPHICS=1` (force on) or `=0`
/// (force off).
pub fn wants_package_graphics_path(expanded: &str) -> bool {
    if !expanded.contains("(import graphics") {
        return false;
    }
    if std::env::var("RECIPLEXA_PACKAGE_GRAPHICS")
        .map(|v| v == "0" || v.eq_ignore_ascii_case("false"))
        .unwrap_or(false)
    {
        return false;
    }
    if std::env::var("RECIPLEXA_PACKAGE_GRAPHICS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return true;
    }
    is_package_shaped_graphics_source(expanded)
}

/// Package-shaped graphics ingest: explicit `main` entry and no top-level interim `(page …)`.
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
    if let Ok(p) = std::env::var("RECIPLEXA_PACKAGE_ROOT") {
        return vec![PathBuf::from(p)];
    }
    let mut roots = Vec::new();
    if let Ok(mut dir) = std::env::current_dir() {
        for _ in 0..8 {
            let packages = dir.join("packages");
            if packages.is_dir() {
                roots.push(packages);
                break;
            }
            if !dir.pop() {
                break;
            }
        }
    }
    if roots.is_empty() {
        roots.push(PathBuf::from("packages"));
    }
    roots
}

fn document_from_package_graphics(expanded: &str) -> Result<Document, PipelineError> {
    let package_src = strip_top_level_effect_forms(expanded);
    let search_roots = package_search_roots();
    let roots: Vec<&Path> = search_roots.iter().map(PathBuf::as_path).collect();
    let idx = LocalPackageIndex::discover(&roots)
        .map_err(|e| PipelineError::new("package", e.to_string()))?;
    document_from_package_source(&package_src, "entry", &idx)
        .map_err(|e| PipelineError::new("package", format!("{e:?}")))
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
        document_from_package_graphics(&expanded)?
    } else {
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
        let doc = document_from_package_graphics(&expanded)?;
        return Ok((doc, expanded));
    }
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
    fn document_from_source_auto_detect_matches_interim_scene_bounds() {
        use reciplexa_lower::lower_source;
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
        let interim = lower_source(
            r#"(page a4
  (text 30 260 8 "Reciplexa" black)
  (line 30 250 180 250 (rgb 0.784 0.157 0.157) 1)
  (translate 105 120
    (circle 0 0 25 (rgb 0.118 0.353 0.706))))"#,
        )
        .expect("interim lower");
        assert_eq!(from_pkg.pages.len(), 1);
        assert_eq!(from_pkg.pages[0].paper, interim.pages[0].paper);
        assert_eq!(
            leaf_shape_count(&from_pkg.pages[0].shapes),
            leaf_shape_count(&interim.pages[0].shapes)
        );
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

    #[test]
    fn document_from_source_does_not_run_effects() {
        // Preview path must not touch the handler (export will).
        let mut h = TestHandler::default();
        let expanded = expand(
            r#"(src (perform log "x"))
(page a4 (circle 1 2 3))"#,
        )
        .unwrap();
        typecheck(&expanded).unwrap();
        let _doc = lower(&expanded).unwrap();
        assert!(h.logs.is_empty());
        run_effects(&mut h, &expanded).unwrap();
        assert_eq!(h.logs, vec!["x"]);
    }

    #[test]
    fn document_for_export_runs_handle_log() {
        let mut h = TestHandler::default();
        h.random_seq = vec![0.5];
        let src = r#"
(src
  (perform log "a")
  (handle log (perform log "mute") (perform random))
  (perform log "b"))
(page a4 (circle 1 2 3))
"#;
        let (doc, _) = document_for_export(&mut h, src).unwrap();
        assert_eq!(doc.pages.len(), 1);
        assert_eq!(h.logs, vec!["a", "b"]);
    }

    #[test]
    fn document_for_export_mutes_write_path() {
        let mut h = TestHandler::default();
        let src = r#"
(src
  (perform write-path "a.pdf")
  (handle write-path (perform write-path "mute.pdf"))
  (perform write-path "b.pdf"))
(page a4 (circle 1 2 3))
"#;
        let (doc, _) = document_for_export(&mut h, src).unwrap();
        assert_eq!(doc.pages.len(), 1);
        assert_eq!(h.writes, vec!["a.pdf", "b.pdf"]);
    }

    // --- defect ---

    #[test]
    fn type_error_surfaces_stage() {
        let err = document_from_source("(page a4 (circle (circle 0 0 1) 0 1))").unwrap_err();
        assert_eq!(err.stage, "type");
        assert!(err.message.contains("type mismatch"));
    }

    #[test]
    fn optional_snapshot_path_builds_editable() {
        let out = document_from_source_with_snapshot("(page a4 (rect 1 2 3 4))", true).unwrap();
        assert_eq!(out.scene.pages.len(), 1);
        let snap = out.editable.expect("snapshot");
        assert!(snap.nodes.iter().count() > 2);
    }

    #[test]
    fn optional_snapshot_skipped_by_default() {
        let out = document_from_source_with_snapshot("(page a4)", false).unwrap();
        assert!(out.editable.is_none());
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
    fn lower_error_surfaces() {
        let err = lower("(page a4 (bogus 1))").unwrap_err();
        assert_eq!(err.stage, "lower");
    }

    #[test]
    fn typecheck_standalone() {
        assert!(typecheck("(page a4 (circle 1 2 3))").is_ok());
        assert!(typecheck("(page a4 (circle x 2 3))").is_err());
    }

    #[test]
    fn expand_doc_title() {
        let expanded = expand("(markup @title{Hi})").unwrap();
        assert!(expanded.contains("Hi"));
    }

    #[test]
    fn document_from_source_with_effects_in_src_only() {
        let h = TestHandler::default();
        let doc = document_from_source(
            r#"(src (perform log "skip"))
(page a4 (circle 1 2 3))"#,
        )
        .unwrap();
        assert_eq!(doc.pages.len(), 1);
        assert!(h.logs.is_empty());
    }

    #[test]
    fn run_effects_unknown_op_uses_effect_stage() {
        let expanded = expand(
            r#"(src (perform draw "x"))
(page a4 (circle 1 2 3))"#,
        )
        .unwrap();
        let err = run_effects(&mut TestHandler::default(), &expanded).unwrap_err();
        assert_eq!(err.stage, "effect");
    }

    #[test]
    fn expand_empty_markup_still_parses() {
        let expanded = expand("(markup)").unwrap();
        assert!(expanded.contains("(page"));
    }

    #[test]
    fn document_for_export_type_error() {
        let err = document_for_export(&mut TestHandler::default(), "(page a4 (circle x 2 3))")
            .unwrap_err();
        assert_eq!(err.stage, "type");
    }

    #[test]
    fn document_snapshot_failure_surfaces_document_stage() {
        let err = document_from_source_with_snapshot("(page a4 (circle x 2 3))", true).unwrap_err();
        assert_eq!(err.stage, "type");
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
