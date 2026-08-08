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
    let bytes = encode_snapshot(snap).map_err(RecoveryError::Codec)?;
    atomic_write(&paths.journal, &bytes).map_err(|e| RecoveryError::Io(e.to_string()))
}

/// Recover from journal into a **new** recovered path — never overwrites primary (§11.2).
pub fn recover_from_journal(paths: &RecoveryPaths) -> Result<DocumentSnapshot, RecoveryError> {
    let bytes = std::fs::read(&paths.journal).map_err(|_| RecoveryError::NoRecoveryCandidate)?;
    let snap = decode_snapshot(&bytes).map_err(RecoveryError::Codec)?;
    let out = encode_snapshot(&snap).map_err(RecoveryError::Codec)?;
    atomic_write(&paths.recovered, &out).map_err(|e| RecoveryError::Io(e.to_string()))?;
    Ok(snap)
}

/// Compact: keep only the latest committed snapshot bytes (drops journal after success).
pub fn compact_after_save(
    paths: &RecoveryPaths,
    snap: &DocumentSnapshot,
) -> Result<(), RecoveryError> {
    let bytes = encode_snapshot(snap).map_err(RecoveryError::Codec)?;
    atomic_write(&paths.primary, &bytes).map_err(|e| RecoveryError::Io(e.to_string()))?;
    let _ = std::fs::remove_file(&paths.journal);
    let _ = std::fs::remove_file(&paths.recovered);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_identity::document::DocumentIdentity;

    #[test]
    fn recovery_writes_sidecar_not_primary() {
        let dir = std::env::temp_dir().join(format!(
            "rpx-rec-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let primary = dir.join("doc.rpxsnap");
        let paths = RecoveryPaths::for_primary(&primary);
        let snap = DocumentSnapshot::new(DocumentIdentity::new(3));
        // Primary does not exist yet; journal does.
        write_journal(&paths, &snap).unwrap();
        assert!(!paths.primary.exists());
        let recovered = recover_from_journal(&paths).unwrap();
        assert_eq!(recovered.identity.get(), 3);
        assert!(paths.recovered.exists());
        assert!(!paths.primary.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn recover_no_journal_returns_no_candidate() {
        let dir = std::env::temp_dir().join(format!(
            "rpx-rec-none-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let paths = RecoveryPaths::for_primary(dir.join("missing.rpxsnap"));
        let err = recover_from_journal(&paths).unwrap_err();
        assert!(matches!(err, RecoveryError::NoRecoveryCandidate));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn compact_drops_sidecar_files() {
        let dir = std::env::temp_dir().join(format!(
            "rpx-rec-compact-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let primary = dir.join("doc.rpxsnap");
        let paths = RecoveryPaths::for_primary(&primary);
        let snap = DocumentSnapshot::new(DocumentIdentity::new(5));
        write_journal(&paths, &snap).unwrap();
        recover_from_journal(&paths).unwrap();
        assert!(paths.journal.exists());
        assert!(paths.recovered.exists());
        compact_after_save(&paths, &snap).unwrap();
        assert!(paths.primary.exists());
        assert!(!paths.journal.exists());
        assert!(!paths.recovered.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn recover_corrupt_journal_returns_codec_error() {
        let dir = std::env::temp_dir().join(format!(
            "rpx-rec-bad-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let paths = RecoveryPaths::for_primary(dir.join("doc.rpxsnap"));
        std::fs::write(&paths.journal, b"not a snapshot").unwrap();
        let err = recover_from_journal(&paths).unwrap_err();
        assert!(matches!(err, RecoveryError::Codec(_)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn recovery_paths_for_primary_sets_sidecars() {
        let primary = std::path::PathBuf::from("docs/snap.rpxsnap");
        let paths = RecoveryPaths::for_primary(&primary);
        assert_eq!(paths.primary, primary);
        assert_eq!(
            paths.journal,
            std::path::PathBuf::from("docs/snap.rpxjournal")
        );
        assert_eq!(
            paths.recovered,
            std::path::PathBuf::from("docs/snap.rpxrecovered")
        );
    }
}
