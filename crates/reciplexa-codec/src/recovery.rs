//! Crash recovery — never auto-overwrite the original file.

use std::path::{Path, PathBuf};

use crate::atomic::atomic_write;
use crate::codec::{decode_snapshot, encode_snapshot, CodecError};
use reciplexa_document::snapshot::DocumentSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryError {
    Codec(CodecError),
    Io(String),
    NoRecoveryCandidate,
}

impl From<std::io::Error> for RecoveryError {
    fn from(e: std::io::Error) -> Self {
        RecoveryError::Io(e.to_string())
    }
}

/// Paths used for crash recovery next to a primary snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryPaths {
    pub primary: PathBuf,
    pub journal: PathBuf,
    pub recovered: PathBuf,
}

impl RecoveryPaths {
    pub fn for_primary(primary: impl AsRef<Path>) -> Self {
        let primary = primary.as_ref().to_path_buf();
        let journal = primary.with_extension("rpxjournal");
        let recovered = primary.with_extension("rpxrecovered");
        Self {
            primary,
            journal,
            recovered,
        }
    }
}

/// Write a journal snapshot during editing (sidecar — not the primary file).
pub fn write_journal(paths: &RecoveryPaths, snap: &DocumentSnapshot) -> Result<(), RecoveryError> {
    let bytes = encode_snapshot(snap);
    atomic_write(&paths.journal, &bytes)?;
    Ok(())
}

/// Recover from journal into a **new** recovered path — never overwrites primary (§11.2).
pub fn recover_from_journal(paths: &RecoveryPaths) -> Result<DocumentSnapshot, RecoveryError> {
    let bytes = std::fs::read(&paths.journal).map_err(|_| RecoveryError::NoRecoveryCandidate)?;
    let snap = decode_snapshot(&bytes).map_err(RecoveryError::Codec)?;
    let out = encode_snapshot(&snap);
    atomic_write(&paths.recovered, &out)?;
    Ok(snap)
}

/// Compact: keep only the latest committed snapshot bytes (drops journal after success).
pub fn compact_after_save(
    paths: &RecoveryPaths,
    snap: &DocumentSnapshot,
) -> Result<(), RecoveryError> {
    let bytes = encode_snapshot(snap);
    atomic_write(&paths.primary, &bytes)?;
    let _ = std::fs::remove_file(&paths.journal);
    let _ = std::fs::remove_file(&paths.recovered);
    Ok(())
}
