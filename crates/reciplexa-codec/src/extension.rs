//! Unknown extension handling and partial recovery.

use serde::{Deserialize, Serialize};

use crate::codec::{decode_snapshot, CodecError, PortableSnapshot, SCHEMA_VERSION};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionBlock {
    pub name: String,
    pub version: u32,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensibleEnvelope {
    pub snapshot: PortableSnapshot,
    #[serde(default)]
    pub extensions: Vec<ExtensionBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartialRecoveryError {
    Codec(CodecError),
    NoUsableSnapshot,
}

/// Decode envelope; unknown extensions are retained but ignored for semantics.
pub fn decode_envelope(bytes: &[u8]) -> Result<ExtensibleEnvelope, CodecError> {
    // Prefer extensible envelope; fall back to bare snapshot.
    if let Ok(env) = serde_json::from_slice::<ExtensibleEnvelope>(bytes) {
        if env.snapshot.schema_version > SCHEMA_VERSION {
            return Err(CodecError::UnsupportedSchema(env.snapshot.schema_version));
        }
        return Ok(env);
    }
    let snap = decode_snapshot(bytes)?;
    Ok(ExtensibleEnvelope {
        snapshot: crate::codec::snapshot_to_portable_public(&snap),
        extensions: Vec::new(),
    })
}

/// Partial recovery: keep known snapshot nodes; drop undecodable extension payloads.
pub fn partial_recover(bytes: &[u8]) -> Result<ExtensibleEnvelope, PartialRecoveryError> {
    match decode_envelope(bytes) {
        Ok(mut env) => {
            env.extensions
                .retain(|e| e.version <= 1 || e.name.starts_with("rpx."));
            Ok(env)
        }
        Err(_) => Err(PartialRecoveryError::NoUsableSnapshot),
    }
}
