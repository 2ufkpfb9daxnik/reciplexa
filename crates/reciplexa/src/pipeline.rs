//! Expand → typecheck → (optional effects) → lower.
//!
//! Keeping these as small functions (instead of one monolith) lets the GUI
//! preview without performing I/O, while the CLI/export path injects a handler.

use reciplexa_effect::{run_source_effects, EffectError, EffectHandler, Value};
use reciplexa_lower::lower_source;
use reciplexa_macro::expand_source;
use reciplexa_scene::Document;
use reciplexa_types::typecheck_source;

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

/// Macro-expand surface forms (`color-byte`, `(doc …)`, …). Input is not mutated.
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
pub fn document_from_source(src: &str) -> Result<Document, PipelineError> {
    let expanded = expand(src)?;
    typecheck(&expanded)?;
    lower(&expanded)
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
    fn document_from_source_expands_doc_title() {
        let doc = document_from_source("(doc @title{Hi})").unwrap();
        assert_eq!(doc.pages.len(), 1);
        match &doc.pages[0].shapes[0] {
            Shape::Text(t) => {
                assert_eq!(t.content, "Hi");
                assert_eq!(t.size_mm, 14.0);
            }
            other => panic!("expected text, got {other:?}"),
        }
    }

    #[test]
    fn document_from_source_expands_doc_image() {
        let doc = document_from_source(r#"(doc @image["figures/demo.png"])"#).unwrap();
        assert_eq!(doc.pages.len(), 1);
        assert!(
            doc.pages[0]
                .shapes
                .iter()
                .any(|s| matches!(s, Shape::Image(_))),
            "expected an Image shape: {:?}",
            doc.pages[0].shapes
        );
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
    fn parse_error_is_macro_stage() {
        let err = expand("(page").unwrap_err();
        assert_eq!(err.stage, "macro");
    }
}
