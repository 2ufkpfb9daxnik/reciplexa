use reciplexa_codec::codec::{decode_snapshot, encode_snapshot, CodecError};
use reciplexa_codec::migration::*;
use reciplexa_document::snapshot::DocumentSnapshot;
use reciplexa_identity::document::DocumentIdentity;

#[test]
fn migrate_corrupt_bytes_fails() {
    let err = migrate_snapshot(b"not valid json").unwrap_err();
    assert!(matches!(err, MigrationError::Codec(CodecError::Decode(_))));
}

#[test]
fn migrate_valid_snapshot_roundtrips() {
    let snap = DocumentSnapshot::new(DocumentIdentity::new(11));
    let bytes = encode_snapshot(&snap);
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
    let snap = DocumentSnapshot::new(DocumentIdentity::new(7));
    let bytes = encode_snapshot(&snap);
    let graph = MigrationGraph;
    let migrated = graph.migrate_to_current(&bytes).unwrap();
    let decoded = decode_snapshot(&migrated).unwrap();
    assert_eq!(decoded.identity.get(), 7);
}
