//! Normative editable document between language and GUI.
//!
//! Phase 3 document model: stable node identity, transactions, provenance.

#![forbid(unsafe_code)]

pub mod bridge;
pub mod node;
pub mod property;
pub mod provenance;
pub mod snapshot;
pub mod transaction;

pub use bridge::{document_from_scene_page, scene_shapes_from_document, ApplyEdit};
pub use node::{DocumentNode, DocumentNodeKind, NodeStore};
pub use property::{FillColor, LayoutBox, NodeProperty, TextContent};
pub use provenance::{NodeProvenance, SourceProvenance};
pub use snapshot::DocumentSnapshot;
pub use transaction::{
    DocumentEdit, DocumentTransaction, TransactionBuilder, TransactionError, TransactionOutcome,
};
