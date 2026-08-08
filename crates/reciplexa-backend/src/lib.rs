//! Backend capability, planning, emission, and artifact validation (Phase 7).

#![forbid(unsafe_code)]

pub mod capability;
pub mod emit;
pub mod pipeline;
pub mod plan;
pub mod profile;
pub mod verify;

pub use capability::{BackendCapability, CapabilityMismatch, PlanningError};
pub use pipeline::{export_scene_to_svg, ExportError, VerifiedSvgArtifact};
pub use plan::{BackendPlan, BackendTarget, PlannedNode, Representation};
pub use profile::OutputProfile;
pub use verify::{validate_svg_artifact, ArtifactValidationError};
