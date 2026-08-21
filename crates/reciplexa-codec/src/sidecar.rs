//! Sidecar snapshot load: migration graph + partial extension recovery.

use crate::codec::{encode_snapshot, snapshot_from_portable, CodecError, SCHEMA_VERSION};
use crate::extension::{
    decode_envelope, partial_recover, ExtensibleEnvelope, PartialRecoveryError,
};
use crate::migration::{migrate_snapshot, MigrationError};
use reciplexa_document::snapshot::DocumentSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidecarLoadError {
    Partial(PartialRecoveryError),
    Migration(MigrationError),
    UnsupportedSchema(u32),
}

impl From<PartialRecoveryError> for SidecarLoadError {
    fn from(e: PartialRecoveryError) -> Self {
        SidecarLoadError::Partial(e)
    }
}

impl From<MigrationError> for SidecarLoadError {
    fn from(e: MigrationError) -> Self {
        SidecarLoadError::Migration(e)
    }
}

/// Result of loading a `.rpxsnap` / journal blob through migration + partial recovery.
#[derive(Debug, Clone)]
pub struct SidecarLoadReport {
    pub snapshot: DocumentSnapshot,
    pub dropped_extensions: Vec<String>,
    pub schema_migrated: bool,
}

/// Load sidecar bytes: partial-recover extensions, migrate schema to current, preserve nodes.
pub fn load_sidecar_bytes(bytes: &[u8]) -> Result<SidecarLoadReport, SidecarLoadError> {
    if let Err(CodecError::UnsupportedSchema(version)) = decode_envelope(bytes) {
        return Err(SidecarLoadError::UnsupportedSchema(version));
    }
    let before = decode_envelope(bytes).ok();
    let env = partial_recover(bytes)?;
    let dropped_extensions = dropped_extension_names(before.as_ref(), &env);
    let snapshot = load_snapshot_from_envelope(&env)?;
    Ok(SidecarLoadReport {
        snapshot,
        dropped_extensions,
        schema_migrated: env.snapshot.schema_version < SCHEMA_VERSION,
    })
}

fn load_snapshot_from_envelope(
    env: &ExtensibleEnvelope,
) -> Result<DocumentSnapshot, SidecarLoadError> {
    if env.snapshot.schema_version > SCHEMA_VERSION {
        return Err(SidecarLoadError::UnsupportedSchema(
            env.snapshot.schema_version,
        ));
    }
    let snap = snapshot_from_portable(&env.snapshot);
    migrate_snapshot(&encode_snapshot(&snap)).map_err(SidecarLoadError::Migration)
}

fn dropped_extension_names(
    before: Option<&ExtensibleEnvelope>,
    after: &ExtensibleEnvelope,
) -> Vec<String> {
    let Some(before) = before else {
        return Vec::new();
    };
    before
        .extensions
        .iter()
        .filter(|b| {
            !after
                .extensions
                .iter()
                .any(|a| a.name == b.name && a.version == b.version)
        })
        .map(|e| format!("{}@v{}", e.name, e.version))
        .collect()
}

/// Human-readable notice for GUI status when extensions were dropped or schema migrated.
pub fn format_sidecar_notice(report: &SidecarLoadReport) -> Option<String> {
    let mut parts = Vec::new();
    if report.schema_migrated {
        parts.push(format!(
            "Sidecar snapshot migrated to schema v{SCHEMA_VERSION}."
        ));
    }
    if !report.dropped_extensions.is_empty() {
        parts.push(format!(
            "Dropped unsupported sidecar extensions: {}.",
            report.dropped_extensions.join(", ")
        ));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{encode_snapshot, snapshot_to_portable_public, PortableSnapshot};
    use crate::extension::{ExtensibleEnvelope, ExtensionBlock};
    use reciplexa_document::snapshot::DocumentSnapshot;
    use reciplexa_identity::document::DocumentIdentity;
    use serde_json::json;

    #[test]
    fn load_bare_snapshot_sidecar() {
        let snap = DocumentSnapshot::new(DocumentIdentity::new(4));
        let bytes = encode_snapshot(&snap);
        let report = load_sidecar_bytes(&bytes).unwrap();
        assert_eq!(report.snapshot.identity.get(), 4);
        assert!(report.dropped_extensions.is_empty());
    }

    #[test]
    fn load_drops_unknown_extensions() {
        let snap = DocumentSnapshot::new(DocumentIdentity::new(2));
        let env = ExtensibleEnvelope {
            snapshot: snapshot_to_portable_public(&snap),
            extensions: vec![
                ExtensionBlock {
                    name: "vendor.unknown".into(),
                    version: 99,
                    payload: json!(null),
                },
                ExtensionBlock {
                    name: "rpx.keep".into(),
                    version: 99,
                    payload: json!({}),
                },
            ],
        };
        let bytes = serde_json::to_vec(&env).unwrap();
        let report = load_sidecar_bytes(&bytes).unwrap();
        assert_eq!(report.dropped_extensions, vec!["vendor.unknown@v99"]);
        assert_eq!(report.snapshot.identity.get(), 2);
    }

    #[test]
    fn load_unsupported_schema_fails() {
        let env = ExtensibleEnvelope {
            snapshot: PortableSnapshot {
                schema_version: SCHEMA_VERSION + 50,
                document_id: 1,
                revision: 0,
                nodes: vec![],
            },
            extensions: vec![],
        };
        let bytes = serde_json::to_vec(&env).unwrap();
        let err = load_sidecar_bytes(&bytes).unwrap_err();
        assert!(matches!(err, SidecarLoadError::UnsupportedSchema(_)));
    }
}
