use reciplexa_codec::sidecar::load_sidecar_bytes;
use reciplexa_codec::snapshot_to_portable_public;
use reciplexa_codec::{encode_snapshot, ExtensibleEnvelope, ExtensionBlock};
use reciplexa_document::snapshot::DocumentSnapshot;
use reciplexa_identity::document::DocumentIdentity;
use serde_json::json;

#[test]
fn sidecar_load_roundtrip_through_migration() {
    let snap = DocumentSnapshot::new(DocumentIdentity::new(8));
    let bytes = encode_snapshot(&snap);
    let report = load_sidecar_bytes(&bytes).unwrap();
    assert_eq!(report.snapshot.identity.get(), 8);
}

#[test]
fn sidecar_migration_rewrites_envelope_with_dropped_extensions() {
    let snap = DocumentSnapshot::new(DocumentIdentity::new(3));
    let env = ExtensibleEnvelope {
        snapshot: snapshot_to_portable_public(&snap),
        extensions: vec![ExtensionBlock {
            name: "vendor.unknown".into(),
            version: 99,
            payload: json!(null),
        }],
    };
    let bytes = serde_json::to_vec(&env).unwrap();
    let report = load_sidecar_bytes(&bytes).unwrap();
    assert_eq!(report.dropped_extensions, vec!["vendor.unknown@v99"]);
}
