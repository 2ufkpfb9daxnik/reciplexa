//! Layered visual IR (Phase 7 §9.2).

#![forbid(unsafe_code)]

pub mod lower;
pub mod provenance;
pub mod render;
pub mod validate;

pub use lower::lower_scene_document;
pub use provenance::{ArtifactProvenance, ProvenanceMap, RenderProvenance};
pub use render::{RenderDocument, RenderNode, RenderNodeId, RenderPage};
pub use validate::{validate_render_document, RenderValidationError};
