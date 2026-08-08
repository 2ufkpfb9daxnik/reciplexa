//! Schema migration graph.

use crate::codec::{decode_snapshot, encode_snapshot, CodecError};
use reciplexa_document::snapshot::DocumentSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    /// Reserved for future schema versions that lack a migration path.
    Unsupported(u32),
    Codec(CodecError),
}

#[derive(Debug, Clone, Default)]
pub struct MigrationGraph;

impl MigrationGraph {
    pub fn migrate_to_current(&self, bytes: &[u8]) -> Result<Vec<u8>, MigrationError> {
        let snap = decode_snapshot(bytes).map_err(MigrationError::Codec)?;
        // Current schema always migrates to itself by re-encoding.
        Ok(encode_snapshot(&snap))
    }
}

pub fn migrate_snapshot(bytes: &[u8]) -> Result<DocumentSnapshot, MigrationError> {
    let graph = MigrationGraph;
    let current = graph.migrate_to_current(bytes)?;
    decode_snapshot(&current).map_err(MigrationError::Codec)
}
