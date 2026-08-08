//! Normative editable document between language and GUI.
//!
//! Phase 3 document model: stable node identity, transactions, provenance.

#![forbid(unsafe_code)]

pub mod bridge;
pub mod node;
pub mod property;
pub mod provenance;
pub mod reconcile;
pub mod reference;
pub mod snapshot;
pub mod source_sync;
pub mod transaction;

pub use bridge::{
    document_from_scene_page, document_from_scene_page_with_layers, drawable_node_ids,
    scene_shapes_from_document, ApplyEdit, LayerSpan,
};
pub use node::{DocumentNode, DocumentNodeKind, NodeStore};
pub use property::{FillColor, LayoutBox, NodeProperty, TextContent};
pub use provenance::{NodeProvenance, SourceProvenance};
pub use reconcile::ReconcileGuard;
pub use reference::ReferenceGraph;
pub use snapshot::DocumentSnapshot;
pub use source_sync::{apply_provenance_edit, SourceSyncBlockReason, SourceSyncOutcome};
pub use transaction::{
    DocumentEdit, DocumentTransaction, TransactionBuilder, TransactionError, TransactionOutcome,
};
