use reciplexa_codec::codec::{
    encode_snapshot, snapshot_to_portable_public, CodecError, PortableSnapshot, SCHEMA_VERSION,
};
use reciplexa_codec::extension::*;
use reciplexa_document::snapshot::DocumentSnapshot;
use reciplexa_identity::document::DocumentIdentity;
use serde_json::json;

#[test]
fn decode_envelope_with_extensions() {
    let snap = DocumentSnapshot::new(DocumentIdentity::new(5));
    let portable = snapshot_to_portable_public(&snap);
    let env = ExtensibleEnvelope {
        snapshot: portable,
        extensions: vec![ExtensionBlock {
            name: "rpx.test".into(),
            version: 1,
            payload: json!({"key": "value"}),
        }],
    };
    let bytes = serde_json::to_vec(&env).unwrap();
    let decoded = decode_envelope(&bytes).unwrap();
    assert_eq!(decoded.snapshot.document_id, 5);
    assert_eq!(decoded.extensions.len(), 1);
    assert_eq!(decoded.extensions[0].name, "rpx.test");
}

#[test]
fn decode_envelope_falls_back_to_bare_snapshot() {
    let snap = DocumentSnapshot::new(DocumentIdentity::new(3));
    let bytes = encode_snapshot(&snap);
    let env = decode_envelope(&bytes).unwrap();
    assert_eq!(env.snapshot.document_id, 3);
    assert!(env.extensions.is_empty());
}

#[test]
fn partial_recover_drops_high_version_unknown() {
    let snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let env = ExtensibleEnvelope {
        snapshot: snapshot_to_portable_public(&snap),
        extensions: vec![
            ExtensionBlock {
                name: "vendor.unknown".into(),
                version: 99,
                payload: json!(null),
            },
            ExtensionBlock {
                name: "rpx.keepme".into(),
                version: 99,
                payload: json!({}),
            },
        ],
    };
    let bytes = serde_json::to_vec(&env).unwrap();
    let recovered = partial_recover(&bytes).unwrap();
    assert_eq!(recovered.extensions.len(), 1);
    assert_eq!(recovered.extensions[0].name, "rpx.keepme");
}

#[test]
fn partial_recover_keeps_low_version_extensions() {
    let snap = DocumentSnapshot::new(DocumentIdentity::new(2));
    let env = ExtensibleEnvelope {
        snapshot: snapshot_to_portable_public(&snap),
        extensions: vec![ExtensionBlock {
            name: "legacy".into(),
            version: 1,
            payload: json!([]),
        }],
    };
    let bytes = serde_json::to_vec(&env).unwrap();
    let recovered = partial_recover(&bytes).unwrap();
    assert_eq!(recovered.extensions.len(), 1);
}

#[test]
fn partial_recover_unusable_bytes() {
    let err = partial_recover(b"garbage").unwrap_err();
    assert!(matches!(err, PartialRecoveryError::NoUsableSnapshot));
}

#[test]
fn decode_envelope_unsupported_schema() {
    let env = ExtensibleEnvelope {
        snapshot: PortableSnapshot {
            schema_version: SCHEMA_VERSION + 100,
            document_id: 1,
            revision: 0,
            nodes: vec![],
        },
        extensions: vec![],
    };
    let bytes = serde_json::to_vec(&env).unwrap();
    let err = decode_envelope(&bytes).unwrap_err();
    assert!(matches!(err, CodecError::UnsupportedSchema(_)));
}
