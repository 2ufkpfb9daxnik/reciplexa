//! Backend capability, planning, emission, and artifact validation (Phase 7 / 12).

#![forbid(unsafe_code)]

pub mod capability;
pub mod emit;
pub mod emit_raster;
pub mod loss;
pub mod pipeline;
pub mod plan;
pub mod preview;
pub mod profile;
pub mod verify;

pub use capability::{BackendCapability, BackendFamily, CapabilityMismatch, PlanningError};
pub use emit::EmitError;
pub use emit_raster::{emit_raster_page_from_plan, EmittedRasterPage};
pub use loss::{LossDisposition, LossReport, OutputLoss, OutputLossKind};
pub use pipeline::{
    export_scene_to_preview, export_scene_to_preview_with_hints, export_scene_to_raster,
    export_scene_to_raster_with_hints, export_scene_to_svg, export_scene_to_svg_with_hints,
    finalize_preview_export, finalize_raster_export, finalize_svg_export, ExportError,
    VerifiedPreviewArtifact, VerifiedRasterArtifact, VerifiedSvgArtifact,
};
pub use plan::{
    plan_preview, plan_raster, plan_svg, BackendPlan, BackendTarget, PlannedNode, Representation,
};
pub use preview::{emit_preview_from_plan, PreviewDrawable};
pub use profile::{OutputProfile, ProfileKind};
pub use verify::{
    validate_png_artifact, validate_raster_loss_report, validate_svg_artifact,
    ArtifactValidationError,
};
