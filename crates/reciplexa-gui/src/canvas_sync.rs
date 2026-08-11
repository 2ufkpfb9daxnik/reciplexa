//! Authoring-source canvas sync helpers.
//!
//! Preview layers may come from macro-expanded buffers (`expanded_for_sync`), but
//! CST mutations must always target the authoring `.rpx`. When those layer lists
//! diverge (e.g. `(markup …)` → synthetic page shapes), refuse the edit softly.

use reciplexa_lower::{collect_layers_page, nudge_layer_page, LayerInfo, SyncError};

/// Why a canvas edit cannot be written back to authoring source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncRefuse {
    pub message: String,
}

impl SyncRefuse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// True when flattened layers in `authoring` line up with `expanded` for `page_index`.
///
/// Spans may differ (e.g. `color-byte` expansion) as long as count and kinds match;
/// mutations re-resolve spans from the authoring buffer.
pub fn authoring_layers_align(
    authoring: &str,
    expanded: &str,
    page_index: usize,
) -> Result<bool, SyncError> {
    let auth = match collect_layers_page(authoring, page_index) {
        Ok(l) => l,
        Err(_) if authoring != expanded => return Ok(false),
        Err(e) => return Err(e),
    };
    let exp = collect_layers_page(expanded, page_index)?;
    Ok(layer_kinds_match(&auth, &exp))
}

fn layer_kinds_match(a: &[LayerInfo], b: &[LayerInfo]) -> bool {
    a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.kind == y.kind)
}

/// Nudge flattened layers in **authoring** source when they align with `expanded`.
///
/// Returns [`SyncRefuse`] (soft) when expanded layers are synthetic / not editable
/// in authoring — callers must not treat this as a hard document error.
pub fn nudge_authoring_layers(
    authoring: &str,
    expanded: &str,
    page_index: usize,
    flat_indices: &[usize],
    dx: f64,
    dy: f64,
) -> Result<String, SyncRefuse> {
    if dx == 0.0 && dy == 0.0 {
        return Ok(authoring.to_string());
    }
    match authoring_layers_align(authoring, expanded, page_index) {
        Ok(true) => {}
        Ok(false) => {
            return Err(SyncRefuse::new(
                "canvas move skipped: expanded layers are not editable in authoring source \
                 (markup-generated text is read-only — edit the markup block instead)",
            ));
        }
        Err(e) => return Err(SyncRefuse::new(e.message)),
    }

    let mut src = authoring.to_string();
    let mut indices = flat_indices.to_vec();
    indices.sort_unstable();
    indices.dedup();
    for &idx in &indices {
        match nudge_layer_page(&src, page_index, idx, dx, dy) {
            Ok(next) => src = next,
            Err(e) => {
                return Err(SyncRefuse::new(format!(
                    "canvas move skipped: {} (authoring source unchanged)",
                    e.message
                )));
            }
        }
    }
    Ok(src)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_page_layers_align_after_color_byte_expand() {
        let authoring = r#"(page a4
  (text 30 260 8 "Reciplexa" black)
  (line 30 250 180 250 (color-byte 200 40 40) 1)
  (translate 105 120
    (circle 0 0 25 (color-byte 30 90 180))))
"#;
        let expanded = reciplexa_macro::expand_source(authoring).unwrap();
        assert_ne!(authoring, expanded);
        assert!(authoring_layers_align(authoring, &expanded, 0).unwrap());
    }

    #[test]
    fn markup_only_does_not_align() {
        let authoring = "(markup @heading(Hi)\n\nbody\n)";
        let expanded = reciplexa_macro::expand_source(authoring).unwrap();
        assert!(!authoring_layers_align(authoring, &expanded, 0).unwrap());
    }

    #[test]
    fn nudge_text_layer_updates_authoring() {
        let src = r#"(page a4 (text 30 260 8 "Hi" black))"#;
        let expanded = src.to_string();
        let out = nudge_authoring_layers(src, &expanded, 0, &[0], 5.0, -3.0).unwrap();
        assert!(
            out.contains("(translate 5 -3 (text 30 260 8 \"Hi\" black))"),
            "{out}"
        );
    }

    #[test]
    fn nudge_zero_delta_is_identity() {
        let src = r#"(page a4 (text 30 260 8 "Hi" black))"#;
        let out = nudge_authoring_layers(src, src, 0, &[0], 0.0, 0.0).unwrap();
        assert_eq!(out, src);
    }

    #[test]
    fn authoring_parse_error_with_identical_buffers_propagates() {
        let bad = "(page";
        let err = authoring_layers_align(bad, bad, 0).unwrap_err();
        assert!(!err.message.is_empty());
    }

    #[test]
    fn nudge_propagates_align_hard_error() {
        let bad = "(page";
        let err = nudge_authoring_layers(bad, bad, 0, &[0], 1.0, 0.0).unwrap_err();
        assert!(!err.message.is_empty());
    }

    #[test]
    fn nudge_bad_flat_index_soft_refuses() {
        let src = r#"(page a4 (text 30 260 8 "Hi" black))"#;
        let err = nudge_authoring_layers(src, src, 0, &[99], 1.0, 0.0).unwrap_err();
        assert!(err.message.contains("skipped"), "{}", err.message);
    }
}
