//! Layered visual IR (Phase 7 §9.2).

#![forbid(unsafe_code)]

pub mod domain;
pub mod layout;
pub mod lower;
pub mod provenance;
pub mod render;
pub mod validate;

pub use domain::{DomainDocument, DomainNode, DomainNodeId, DomainPage};
pub use layout::{layout_identity, LayoutDocument, LayoutPage};
pub use lower::{
    layout_to_render, lower_scene_document, lower_scene_document_with_options,
    render_from_glyph_run, scene_to_domain, LowerOptions, NodeSourceHint,
};
pub use provenance::{ArtifactProvenance, ProvenanceMap, RenderProvenance};
pub use render::{RenderDocument, RenderNode, RenderNodeId, RenderPage};
pub use validate::{validate_render_document, RenderValidationError};
