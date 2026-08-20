//! Crash recovery — never auto-overwrite the original file.

use std::path::{Path, PathBuf};

use crate::atomic::atomic_write;
use crate::codec::{encode_snapshot, CodecError};
use crate::sidecar::load_sidecar_bytes;
use reciplexa_document::snapshot::DocumentSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryError {
    Codec(CodecError),
    Io(String),
    NoRecoveryCandidate,
    Sidecar(crate::sidecar::SidecarLoadError),
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

    /// Sidecar paths for a `.rpx` source file (`doc.rpx` → `doc.rpxsnap` primary).
    pub fn for_rpx_source(rpx: impl AsRef<Path>) -> Self {
        Self::for_primary(rpx.as_ref().with_extension("rpxsnap"))
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
    let report = load_sidecar_bytes(&bytes).map_err(RecoveryError::Sidecar)?;
    let out = encode_snapshot(&report.snapshot);
    atomic_write(&paths.recovered, &out)?;
    Ok(report.snapshot)
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

/// Path for unsaved `.rpx` source journal (`doc.rpx` → `doc.rpjsrc`).
pub fn source_journal_path(rpx: impl AsRef<Path>) -> PathBuf {
    rpx.as_ref().with_extension("rpjsrc")
}

/// Autosave dirty source during editing; never overwrites the primary `.rpx`.
pub fn write_source_journal(rpx: impl AsRef<Path>, source: &[u8]) -> Result<(), RecoveryError> {
    atomic_write(&source_journal_path(rpx), source)?;
    Ok(())
}

/// Read a source journal when present.
pub fn read_source_journal(rpx: impl AsRef<Path>) -> Result<Option<String>, RecoveryError> {
    match std::fs::read_to_string(source_journal_path(rpx)) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Drop source journal after a successful primary save or explicit discard.
pub fn clear_source_journal(rpx: impl AsRef<Path>) {
    let _ = std::fs::remove_file(source_journal_path(rpx));
}
