//! Schema migration graph.

use crate::codec::{decode_snapshot, encode_snapshot, CodecError, SCHEMA_VERSION};
use reciplexa_document::snapshot::DocumentSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    Unsupported(u32),
    Codec(CodecError),
}

#[derive(Debug, Clone, Default)]
pub struct MigrationGraph;

impl MigrationGraph {
    pub fn migrate_to_current(&self, bytes: &[u8]) -> Result<Vec<u8>, MigrationError> {
        let snap = decode_snapshot(bytes).map_err(MigrationError::Codec)?;
        if SCHEMA_VERSION == 1 {
            encode_snapshot(&snap).map_err(MigrationError::Codec)
        } else {
            Err(MigrationError::Unsupported(SCHEMA_VERSION))
        }
    }
}

pub fn migrate_snapshot(bytes: &[u8]) -> Result<DocumentSnapshot, MigrationError> {
    let graph = MigrationGraph;
    let current = graph.migrate_to_current(bytes)?;
    decode_snapshot(&current).map_err(MigrationError::Codec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::CodecError;

    #[test]
    fn migrate_corrupt_bytes_fails() {
        let err = migrate_snapshot(b"not valid json").unwrap_err();
        assert!(matches!(err, MigrationError::Codec(CodecError::Decode(_))));
    }

    #[test]
    fn migrate_valid_snapshot_roundtrips() {
        use crate::codec::encode_snapshot;
        use reciplexa_document::snapshot::DocumentSnapshot;
        use reciplexa_identity::document::DocumentIdentity;

        let snap = DocumentSnapshot::new(DocumentIdentity::new(11));
        let bytes = encode_snapshot(&snap).unwrap();
        let migrated = migrate_snapshot(&bytes).unwrap();
        assert_eq!(migrated.identity.get(), 11);
    }

    #[test]
    fn unsupported_migration_variant_is_documented() {
        let err = MigrationError::Unsupported(99);
        assert!(matches!(err, MigrationError::Unsupported(99)));
    }

    #[test]
    fn migration_graph_roundtrips_current_schema() {
        use crate::codec::encode_snapshot;
        use reciplexa_document::snapshot::DocumentSnapshot;
        use reciplexa_identity::document::DocumentIdentity;

        let snap = DocumentSnapshot::new(DocumentIdentity::new(7));
        let bytes = encode_snapshot(&snap).unwrap();
        let graph = MigrationGraph;
        let migrated = graph.migrate_to_current(&bytes).unwrap();
        let decoded = decode_snapshot(&migrated).unwrap();
        assert_eq!(decoded.identity.get(), 7);
    }
}
