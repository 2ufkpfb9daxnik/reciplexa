//! Host preview helpers: text metrics from package document sources.
//!
//! Bridges package-shaped document sources to a scene [`Document`], then reports
//! line / text-shape counts and light em box estimates for tests and future GUI.
//! Optional ruby / tate-chu-yoko counts walk the eval value tree when present.
//! Not production JLReq measure — see `lang/implemented-features.md` (OPEN-TEXT-JA-001).

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use reciplexa_eval::{document_from_graphics_value, RuntimeValue};
use reciplexa_scene::{Document, Shape};
use reciplexa_std::japanese::char_em_width;

use crate::graphics_bridge::GraphicsBridgeError;
use crate::load::{eval_package_entry_main, LocalPackageIndex};

/// Soft-wrap em budget aligned with `reciplexa_eval::document_value` paragraphs.
pub const DOC_TEXT_MAX_EM: f64 = 40.0;

static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);

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
    /// Optional: `ruby-box` / `vertical-ruby-box` / `ja-ruby` nodes in the eval tree.
    /// Zero when metrics come from a scene document only.
    pub ruby_count: usize,
    /// Optional: `ja-tate-chu-yoko` / `tate-chu-yoko` nodes in the eval tree.
    /// Zero when metrics come from a scene document only.
    pub tate_chu_yoko_count: usize,
}

impl DocTextPreviewMetrics {
    /// One-line diagnostic for CLI / GUI status.
    pub fn diagnostic_note(&self) -> String {
        let mut note = format!(
            "doc text preview: {} line(s), {} char(s), max {:.1}em wide, ~{:.1}em tall (budget {:.0}em)",
            self.line_count,
            self.total_content_chars,
            self.max_line_width_em,
            self.approx_block_height_em,
            self.soft_wrap_budget_em
        );
        if self.ruby_count > 0 || self.tate_chu_yoko_count > 0 {
            note.push_str(&format!(
                "; ruby={}, tate-chu-yoko={}",
                self.ruby_count, self.tate_chu_yoko_count
            ));
        }
        note
    }

    /// Compact tooltip-like summary (no UI chrome required).
    pub fn tooltip_summary(&self) -> String {
        let mut s = format!(
            "{} line(s), {} text shape(s), {} char(s)",
            self.line_count, self.text_shape_count, self.total_content_chars
        );
        if self.ruby_count > 0 || self.tate_chu_yoko_count > 0 {
            s.push_str(&format!(
                "; ruby={}, tate={}",
                self.ruby_count, self.tate_chu_yoko_count
            ));
        }
        s
    }
}

/// Fail-soft layout summary for inspect / GUI tooltips (HC10 follow-on).
///
/// Uses [`preview_doc_text_metrics`]; does not invent status-bar chrome.
pub fn debug_layout_summary(
    source: &str,
    index: &LocalPackageIndex,
) -> Result<String, GraphicsBridgeError> {
    Ok(preview_doc_text_metrics(source, index)?.tooltip_summary())
}

/// Count ruby / tate-chu-yoko tagged records in an eval value tree (HC12).
pub fn count_ruby_tate_in_value(v: &RuntimeValue) -> (usize, usize) {
    match v {
        RuntimeValue::Record(fields) => {
            let tag = fields.iter().find(|(k, _)| k == "tag").and_then(|(_, t)| {
                if let RuntimeValue::String(s) = t {
                    Some(s.as_str())
                } else {
                    None
                }
            });
            let mut ruby = 0usize;
            let mut tate = 0usize;
            if let Some(t) = tag {
                if is_ruby_tag(t) {
                    ruby += 1;
                }
                if is_tate_tag(t) {
                    tate += 1;
                }
            }
            for (_, child) in fields {
                let (r, t) = count_ruby_tate_in_value(child);
                ruby += r;
                tate += t;
            }
            (ruby, tate)
        }
        RuntimeValue::Variant { payload, .. } => payload
            .as_ref()
            .map(|p| count_ruby_tate_in_value(p))
            .unwrap_or((0, 0)),
        RuntimeValue::Cell { value, .. } => count_ruby_tate_in_value(&value.borrow()),
        _ => (0, 0),
    }
}

fn is_ruby_tag(tag: &str) -> bool {
    matches!(tag, "ruby-box" | "vertical-ruby-box" | "ja-ruby")
}

fn is_tate_tag(tag: &str) -> bool {
    matches!(tag, "ja-tate-chu-yoko" | "tate-chu-yoko")
}

/// Metrics from an already-bridged scene document (page 0).
///
/// Ruby / tate counts are zero — the scene has no markup identity.
pub fn preview_doc_text_metrics_from_document(doc: &Document) -> DocTextPreviewMetrics {
    preview_doc_text_metrics_from_document_with_counts(doc, 0, 0)
}

fn preview_doc_text_metrics_from_document_with_counts(
    doc: &Document,
    ruby_count: usize,
    tate_chu_yoko_count: usize,
) -> DocTextPreviewMetrics {
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
        ruby_count,
        tate_chu_yoko_count,
    }
}

/// Prefer a direct graphics/doc value; otherwise a `page` field on a host record.
fn document_from_preview_value(v: &RuntimeValue) -> Result<Document, GraphicsBridgeError> {
    match document_from_graphics_value(v) {
        Ok(doc) => Ok(doc),
        Err(direct) => {
            if let RuntimeValue::Record(fields) = v {
                if let Some((_, page)) = fields.iter().find(|(k, _)| k == "page") {
                    return document_from_graphics_value(page).map_err(Into::into);
                }
            }
            Err(GraphicsBridgeError::Bridge(direct.message))
        }
    }
}

/// Bridge `source` via package document path, then compute preview metrics.
///
/// Walks the eval tree for optional ruby / tate-chu-yoko counts (HC12).
pub fn preview_doc_text_metrics(
    source: &str,
    index: &LocalPackageIndex,
) -> Result<DocTextPreviewMetrics, GraphicsBridgeError> {
    let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "reciplexa-pkg-{}-preview_doc_metrics-{seq}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).map_err(|e| GraphicsBridgeError::Load(e.to_string()))?;
    let entry = dir.join("preview_doc_metrics.rpx");
    std::fs::write(&entry, source).map_err(|e| GraphicsBridgeError::Load(e.to_string()))?;
    preview_doc_text_metrics_from_entry(&entry, index)
}

/// Elaborate/eval package entry `main`, count ruby/tate, bridge page, metrics.
pub fn preview_doc_text_metrics_from_entry(
    entry_path: impl AsRef<Path>,
    index: &LocalPackageIndex,
) -> Result<DocTextPreviewMetrics, GraphicsBridgeError> {
    let entry = entry_path.as_ref();
    let v = eval_package_entry_main(entry, index).map_err(GraphicsBridgeError::from)?;
    let (ruby_count, tate_chu_yoko_count) = count_ruby_tate_in_value(&v);
    let doc = document_from_preview_value(&v)?;
    Ok(preview_doc_text_metrics_from_document_with_counts(
        &doc,
        ruby_count,
        tate_chu_yoko_count,
    ))
}
