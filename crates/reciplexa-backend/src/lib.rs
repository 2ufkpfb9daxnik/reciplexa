//! Backend capability, planning, emission, and artifact validation (Phase 7).

#![forbid(unsafe_code)]

pub mod capability;
pub mod emit;
pub mod pipeline;
pub mod plan;
pub mod preview;
pub mod profile;
pub mod verify;

pub use capability::{BackendCapability, CapabilityMismatch, PlanningError};
pub use emit::EmitError;
pub use pipeline::{
    export_scene_to_preview, export_scene_to_preview_with_hints, export_scene_to_svg,
    export_scene_to_svg_with_hints, ExportError, VerifiedPreviewArtifact, VerifiedSvgArtifact,
};
pub use plan::{plan_preview, plan_svg, BackendPlan, BackendTarget, PlannedNode, Representation};
pub use preview::{emit_preview_from_plan, PreviewDrawable};
pub use profile::OutputProfile;
pub use verify::{validate_svg_artifact, ArtifactValidationError};
