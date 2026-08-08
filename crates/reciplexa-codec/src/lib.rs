//! Portable document snapshot encoding (Phase 9).

#![forbid(unsafe_code)]

pub mod atomic;
pub mod codec;
pub mod extension;
pub mod manifest;
pub mod migration;
pub mod recovery;
pub mod txnlog;
pub mod undo;

pub use atomic::atomic_write;
pub use codec::{
    decode_snapshot, encode_snapshot, snapshot_to_portable_public, CodecError, PortableSnapshot,
    SCHEMA_VERSION,
};
pub use extension::{
    decode_envelope, partial_recover, ExtensibleEnvelope, ExtensionBlock, PartialRecoveryError,
};
pub use manifest::{ResourceEntry, ResourceManifest};
pub use migration::{migrate_snapshot, MigrationError, MigrationGraph};
pub use recovery::{
    compact_after_save, recover_from_journal, write_journal, RecoveryError, RecoveryPaths,
};
pub use txnlog::{
    decode_segment, encode_segment, LogError, LogRecord, SegmentStatus, TransactionLogSegment,
};
pub use undo::{RevisionUndoLog, UndoAction};
