//! Portable document snapshot encoding (Phase 9).

#![forbid(unsafe_code)]

pub mod atomic;
pub mod codec;
pub mod migration;
pub mod undo;

pub use atomic::atomic_write;
pub use codec::{
    decode_snapshot, encode_snapshot, CodecError, PortableSnapshot, SCHEMA_VERSION,
};
pub use migration::{migrate_snapshot, MigrationError, MigrationGraph};
pub use undo::{RevisionUndoLog, UndoAction};
