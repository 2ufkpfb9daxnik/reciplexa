//! Crash recovery helpers for the GUI host.

use std::path::Path;

use reciplexa_codec::{
    clear_source_journal, compact_after_save, decode_snapshot, read_source_journal,
    recover_from_journal, write_source_journal, RecoveryError, RecoveryPaths,
};

/// Result of applying open-time recovery (never overwrites the primary `.rpx` on disk).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenRecovery {
    pub source: String,
    pub status: Option<String>,
}

/// Detect whether a source journal differs from the on-disk primary.
pub fn pending_source_journal(rpx: &Path, primary: &str) -> Option<String> {
    let journal = read_source_journal(rpx).ok()??;
    if journal == primary {
        clear_source_journal(rpx);
        None
    } else {
        Some(journal)
    }
}

/// Recover a pending snapshot journal into `.rpxrecovered` and compact when possible.
pub fn recover_snapshot_sidecar(rpx: &Path) -> Result<Option<String>, RecoveryError> {
    let paths = RecoveryPaths::for_rpx_source(rpx);
    if !paths.journal.exists() {
        return Ok(None);
    }
    recover_from_journal(&paths)?;
    let note = format!(
        "Interrupted snapshot save recovered to {}. Your .rpx file was not modified.",
        paths.recovered.display()
    );
    if let Ok(bytes) = std::fs::read(&paths.recovered) {
        if let Ok(snap) = decode_snapshot(&bytes) {
            let _ = compact_after_save(&paths, &snap);
        }
    }
    Ok(Some(note))
}

/// Apply open-time recovery with an interactive prompt for unsaved source journals.
pub fn resolve_open_recovery(rpx: &Path, primary: String) -> OpenRecovery {
    let mut source = primary;
    let mut status = None;

    if let Some(journal) = pending_source_journal(rpx, &source) {
        let restore = rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Warning)
            .set_title("Recover unsaved edits?")
            .set_description(format!(
                "Found crash-recovery edits for {}.\nRestore them into the editor? Your saved .rpx file will not be overwritten until you save.",
                rpx.display()
            ))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();
        if matches!(restore, rfd::MessageDialogResult::Yes) {
            source = journal;
            status = Some("Restored unsaved edits from crash recovery journal.".into());
        } else {
            clear_source_journal(rpx);
        }
    }

    match recover_snapshot_sidecar(rpx) {
        Ok(Some(note)) => {
            status = Some(match status {
                Some(s) => format!("{s} {note}"),
                None => note,
            });
        }
        Err(e) => {
            let err = format!("Snapshot recovery failed: {e:?}");
            status = Some(match status {
                Some(s) => format!("{s} {err}"),
                None => err,
            });
        }
        Ok(None) => {}
    }

    OpenRecovery { source, status }
}

/// Persist dirty source to the sidecar journal (primary `.rpx` untouched).
pub fn persist_dirty_source_journal(rpx: &Path, source: &str) -> Result<(), RecoveryError> {
    write_source_journal(rpx, source.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use reciplexa_codec::atomic_write;

    #[test]
    fn pending_source_journal_ignores_matching_content() {
        let dir = std::env::temp_dir().join(format!(
            "reciplexa-pend-src-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("doc.rpx");
        atomic_write(&path, b"same").expect("write primary");
        write_source_journal(&path, b"same").expect("write journal");
        assert!(pending_source_journal(&path, "same").is_none());
        assert!(read_source_journal(&path).unwrap().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn pending_source_journal_returns_diff() {
        let dir = std::env::temp_dir().join(format!(
            "reciplexa-pend-diff-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("doc.rpx");
        atomic_write(&path, b"disk").expect("write primary");
        write_source_journal(&path, b"edited").expect("write journal");
        assert_eq!(
            pending_source_journal(&path, "disk").as_deref(),
            Some("edited")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
