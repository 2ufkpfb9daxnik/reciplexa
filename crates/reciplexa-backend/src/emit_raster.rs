//! Raster emission from planning IR via `reciplexa-raster`.

use reciplexa_raster::{frame_to_png, rasterize_page, RasterError, RasterLoss, RasterOptions};
use reciplexa_scene::Document;

use crate::emit::EmitError;
use crate::loss::{LossDisposition, LossReport, OutputLoss, OutputLossKind};
use crate::plan::{BackendPlan, BackendTarget};
use crate::profile::OutputProfile;

/// One verified-ready PNG page plus the Loss report for this profile.
#[derive(Debug, Clone, PartialEq)]
pub struct EmittedRasterPage {
    pub page_index: usize,
    pub png: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub losses: LossReport,
}

/// Emit a single document page as PNG strictly under a Raster plan.
pub fn emit_raster_page_from_plan(
    plan: &BackendPlan,
    doc: &Document,
    page_index: usize,
) -> Result<EmittedRasterPage, EmitError> {
    if plan.target != BackendTarget::Raster {
        return Err(EmitError::PlanNodeMismatch {
            render_id: 0,
            representation: format!("expected Raster target, got {:?}", plan.target),
        });
    }
    if !plan.profile.forbid_silent_loss {
        return Err(EmitError::PlanNodeMismatch {
            render_id: 0,
            representation: "raster emit forbids silent loss profiles".into(),
        });
    }
    let opts = RasterOptions {
        px_per_mm: plan.profile.px_per_mm,
        ..RasterOptions::default()
    };
    let frame = rasterize_page(doc, page_index, &opts).map_err(map_raster_err)?;
    // Successful rasterize always yields an RGB8 frame the PNG encoder accepts.
    let png = frame_to_png(&frame).expect("rasterize_page frame is PNG-encodable");
    let mut losses = plan.losses.clone();
    // Ensure runtime discoveries are never dropped (never silent).
    for rl in &frame.losses {
        if !losses_cover(&losses, rl) {
            losses.push(runtime_loss_to_output(rl, &plan.profile));
        }
    }
    Ok(EmittedRasterPage {
        page_index,
        png,
        width: frame.width,
        height: frame.height,
        losses,
    })
}

fn map_raster_err(e: RasterError) -> EmitError {
    EmitError::PlanNodeMismatch {
        render_id: 0,
        representation: format!("raster: {e:?}"),
    }
}

fn losses_cover(report: &LossReport, rl: &RasterLoss) -> bool {
    let kind = match rl {
        RasterLoss::TextSkipped => OutputLossKind::SemanticText,
        RasterLoss::ImageSkipped => OutputLossKind::Editability,
        RasterLoss::PathAsPolyline => OutputLossKind::VectorRepresentation,
    };
    report.losses.iter().any(|l| l.kind == kind)
}

fn runtime_loss_to_output(rl: &RasterLoss, profile: &OutputProfile) -> OutputLoss {
    match rl {
        RasterLoss::TextSkipped => OutputLoss {
            kind: OutputLossKind::SemanticText,
            disposition: profile.text_omit_disposition,
            profile: profile.kind,
            detail: "text skipped by raster backend (runtime)".into(),
        },
        RasterLoss::ImageSkipped => OutputLoss {
            kind: OutputLossKind::Editability,
            disposition: profile.image_omit_disposition,
            profile: profile.kind,
            detail: "image skipped by raster backend (runtime)".into(),
        },
        RasterLoss::PathAsPolyline => OutputLoss {
            kind: OutputLossKind::VectorRepresentation,
            disposition: LossDisposition::Warn,
            profile: profile.kind,
            detail: "path lowered to stroked polyline (runtime)".into(),
        },
    }
}
