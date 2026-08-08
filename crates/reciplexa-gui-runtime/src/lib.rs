//! GUI runtime: description, reconciliation, widget state (Phase 8).

#![forbid(unsafe_code)]

pub mod description;
pub mod focus;
pub mod reconcile;
pub mod state;

pub use description::{GuiDescription, GuiNode, GuiNodeKind};
pub use focus::{FocusOwner, FocusState};
pub use reconcile::{ReconcileError, ReconcileOp, ReconcilePlan, ReconcileResult, reconcile};
pub use state::{MountedInstance, ViewState, WidgetState};
