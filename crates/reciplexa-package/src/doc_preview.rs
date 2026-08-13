//! Host preview helpers: text metrics from package document sources.
//!
//! Bridges package-shaped document sources to a scene [`Document`], then reports
//! line / text-shape counts and light em box estimates for tests and future GUI.
//! Not production JLReq measure — see `lang/host-layout-consume-plan.md`.

use reciplexa_scene::{Document, Shape};
use reciplexa_std::japanese::char_em_width;

use crate::graphics_bridge::{document_from_package_source, GraphicsBridgeError};
use crate::load::LocalPackageIndex;

/// Soft-wrap em budget aligned with `reciplexa_eval::document_value` paragraphs.
pub const DOC_TEXT_MAX_EM: f64 = 40.0;

/// Line / box preview for a package document page (host consume).
#[derive(Debug, Clone, PartialEq)]
pub struct DocTextPreviewMetrics {
    /// Scene `Text` shapes on page 0 (one soft-wrapped line ≈ one shape).
    pub text_shape_count: usize,
    /// Same as [`Self::text_shape_count`] after document soft-wrap.
    pub line_count: usize,
    /// Sum of Unicode scalar counts across text shape contents.
    pub total_content_chars: usize,
    /// Max line width in em via [`char_em_width`].
    pub max_line_width_em: f64,
    /// Rough block height in em (`line_count` + small inter-line gap stub).
    pub approx_block_height_em: f64,
    /// Document soft-wrap budget used by the bridge (em).
    pub soft_wrap_budget_em: f64,
}

impl DocTextPreviewMetrics {
    /// One-line diagnostic for CLI / GUI status.
    pub fn diagnostic_note(&self) -> String {
        format!(
            "doc text preview: {} line(s), {} char(s), max {:.1}em wide, ~{:.1}em tall (budget {:.0}em)",
            self.line_count,
            self.total_content_chars,
            self.max_line_width_em,
            self.approx_block_height_em,
            self.soft_wrap_budget_em
        )
    }
}

/// Metrics from an already-bridged scene document (page 0).
pub fn preview_doc_text_metrics_from_document(doc: &Document) -> DocTextPreviewMetrics {
    let texts: Vec<&str> = doc
        .pages
        .first()
        .map(|p| {
            p.shapes
                .iter()
                .filter_map(|s| match s {
                    Shape::Text(t) => Some(t.content.as_str()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();

    let text_shape_count = texts.len();
    let line_count = text_shape_count;
    let total_content_chars = texts.iter().map(|t| t.chars().count()).sum();
    let max_line_width_em = texts
        .iter()
        .map(|t| t.chars().map(char_em_width).sum::<f64>())
        .fold(0.0_f64, f64::max);
    let approx_block_height_em = if line_count == 0 {
        0.0
    } else {
        // 1em per line + 0.3em gap stub (mirrors size_mm+3 at size≈10 loosely).
        line_count as f64 + (line_count.saturating_sub(1) as f64) * 0.3
    };

    DocTextPreviewMetrics {
        text_shape_count,
        line_count,
        total_content_chars,
        max_line_width_em,
        approx_block_height_em,
        soft_wrap_budget_em: DOC_TEXT_MAX_EM,
    }
}

/// Bridge `source` via package document path, then compute preview metrics.
pub fn preview_doc_text_metrics(
    source: &str,
    index: &LocalPackageIndex,
) -> Result<DocTextPreviewMetrics, GraphicsBridgeError> {
    let doc = document_from_package_source(source, "preview_doc_metrics", index)?;
    Ok(preview_doc_text_metrics_from_document(&doc))
}
