//! GUI runtime: description, reconciliation, widget state (Phase 8).

#![forbid(unsafe_code)]

pub mod a11y;
pub mod description;
pub mod dragdrop;
pub mod focus;
pub mod gesture;
pub mod host;
pub mod ime;
pub mod inspector;
pub mod lifecycle;
pub mod reconcile;
pub mod selection;
pub mod state;
pub mod timeline;
pub mod virtualize;

pub use a11y::{access_label, role_for_kind, AccessRole};
pub use description::{GuiDescription, GuiNode, GuiNodeKind};
pub use dragdrop::{DragPayload, DragSession};
pub use focus::{FocusOwner, FocusState};
pub use gesture::{GestureArena, GestureClaim, GestureKind};
pub use host::GuiRuntimeHost;
pub use ime::{ImeSession, ImeState};
pub use inspector::{layer_rows, layer_tree_description, InspectorModel, LayerRow};
pub use lifecycle::{commit_reconcile, lifecycle_events, validate_description, LifecycleEvent};
pub use reconcile::{
    rebase_caret, reconcile, ReconcileError, ReconcileOp, ReconcilePlan, ReconcileResult,
};
pub use selection::NodeSelection;
pub use state::{MountedInstance, MountedTree, ViewState, WidgetState};
pub use timeline::{TimeMs, Timeline, TimelineTrack};
pub use virtualize::VirtualWindow;
