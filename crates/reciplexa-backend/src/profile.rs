//! Output profile snapshot for a planned export.

use crate::loss::LossDisposition;

/// Preview vs Final — distinct Capability Snapshot / Planning IR (spec §85).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProfileKind {
    Preview,
    Final,
}

/// User-selected output constraints recorded at planning time.
#[derive(Debug, Clone, PartialEq)]
pub struct OutputProfile {
    pub kind: ProfileKind,
    pub font_family: String,
    pub ellipse_sides: u32,
    pub requires_text: bool,
    pub requires_images: bool,
    /// Pixels per millimeter for Raster targets (ignored by SVG planning).
    pub px_per_mm: f64,
    /// When true, planner/emitter must never omit a Loss entry (Raster contract).
    pub forbid_silent_loss: bool,
    /// Disposition when Raster omits text (Preview=Report, Final=RequireExplicitApproval).
    pub text_omit_disposition: LossDisposition,
    /// Disposition when Raster omits images.
    pub image_omit_disposition: LossDisposition,
}

impl OutputProfile {
    pub fn svg_default() -> Self {
        Self {
            kind: ProfileKind::Final,
            font_family: "sans-serif".into(),
            ellipse_sides: 32,
            requires_text: true,
            requires_images: true,
            px_per_mm: 96.0 / 25.4,
            forbid_silent_loss: false,
            text_omit_disposition: LossDisposition::Report,
            image_omit_disposition: LossDisposition::Report,
        }
    }

    /// Interactive / low-latency Raster Preview — lower resolution; losses always reported.
    pub fn preview_raster() -> Self {
        Self {
            kind: ProfileKind::Preview,
            font_family: "sans-serif".into(),
            ellipse_sides: 16,
            requires_text: false,
            requires_images: false,
            px_per_mm: 72.0 / 25.4,
            forbid_silent_loss: true,
            text_omit_disposition: LossDisposition::Report,
            image_omit_disposition: LossDisposition::Report,
        }
    }

    /// Final Raster export — higher resolution; omit dispositions are stricter than Preview.
    pub fn final_raster() -> Self {
        Self {
            kind: ProfileKind::Final,
            font_family: "sans-serif".into(),
            ellipse_sides: 32,
            requires_text: false,
            requires_images: false,
            px_per_mm: 300.0 / 25.4,
            forbid_silent_loss: true,
            text_omit_disposition: LossDisposition::RequireExplicitApproval,
            image_omit_disposition: LossDisposition::RequireExplicitApproval,
        }
    }
}
