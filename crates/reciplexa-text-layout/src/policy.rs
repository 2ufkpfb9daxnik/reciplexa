//! Font policy: metric-changing substitution requires relayout (never silent).

use crate::error::LayoutError;
use crate::font::{FontId, LoadedFont};

/// Decide whether `available` may be used in place of `requested`.
///
/// Same digest → ok. Different digest → [`LayoutError::SubstitutionRequiresRelayout`].
pub fn select_font(requested: &FontId, available: &LoadedFont) -> Result<(), LayoutError> {
    if requested.digest == available.id.digest {
        Ok(())
    } else {
        Err(LayoutError::SubstitutionRequiresRelayout {
            requested: requested.as_key(),
            substitute: available.id.as_key(),
            reason: "content digest differs; glyph advances would not match positioned output"
                .into(),
        })
    }
}

/// Registry of font bytes keyed by digest (preview and export share this).
#[derive(Debug, Clone, Default)]
pub struct FontRegistry {
    faces: Vec<LoadedFont>,
}

impl FontRegistry {
    pub fn new() -> Self {
        Self { faces: Vec::new() }
    }

    pub fn insert(&mut self, font: LoadedFont) {
        if !self.faces.iter().any(|f| f.id.digest == font.id.digest) {
            self.faces.push(font);
        }
    }

    pub fn get_by_digest(&self, digest: &str) -> Option<&LoadedFont> {
        self.faces.iter().find(|f| f.id.digest == digest)
    }

    pub fn get(&self, id: &FontId) -> Result<&LoadedFont, LayoutError> {
        self.get_by_digest(&id.digest)
            .ok_or_else(|| LayoutError::MissingFont {
                font_id: id.as_key(),
            })
    }

    /// Reject using `emit_font` to paint `layout_id` when digests differ.
    pub fn require_same_as_layout(
        &self,
        layout_id: &FontId,
        emit_font: &LoadedFont,
    ) -> Result<(), LayoutError> {
        select_font(layout_id, emit_font)
    }
}
