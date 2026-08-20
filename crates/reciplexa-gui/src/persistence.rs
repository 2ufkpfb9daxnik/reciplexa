//! Crash recovery and revision undo helpers for the GUI host.

use std::path::Path;

use reciplexa::document_pipeline::document_snapshot_from_source;
use reciplexa_codec::{
    clear_source_journal, compact_after_save, decode_authoring_frame, decode_snapshot,
    encode_authoring_frame, read_source_journal, recover_from_journal, snapshot_to_portable_public,
    write_source_journal, AuthoringUndoFrame, RecoveryError, RecoveryPaths, RevisionUndoLog,
};
use reciplexa_identity::document::DocumentIdentity;

const AUTHORING_DOC_ID: DocumentIdentity = DocumentIdentity::new(1);

/// Revision undo/redo wired to source plus `document_snapshot_from_source`.
#[derive(Debug, Clone)]
pub struct AuthoringUndo {
    log: RevisionUndoLog,
}

impl AuthoringUndo {
    pub fn new(max_depth: usize) -> Self {
        Self {
            log: RevisionUndoLog::new(max_depth),
        }
    }

    pub fn capture_frame(source: &str) -> AuthoringUndoFrame {
        let snapshot = document_snapshot_from_source(source, AUTHORING_DOC_ID)
            .ok()
            .map(|snap| snapshot_to_portable_public(&snap));
        AuthoringUndoFrame::new(source, snapshot)
    }

    pub fn push(&mut self, source_before: &str) {
        let frame = Self::capture_frame(source_before);
        self.log.push_undo(encode_authoring_frame(&frame));
    }

    pub fn undo(&mut self, current_source: &str) -> Option<String> {
        let current = encode_authoring_frame(&Self::capture_frame(current_source));
        let prev = self.log.undo(current)?;
        decode_authoring_frame(&prev).ok().map(|frame| frame.source)
    }

    pub fn redo(&mut self, current_source: &str) -> Option<String> {
        let current = encode_authoring_frame(&Self::capture_frame(current_source));
        let next = self.log.redo(current)?;
        decode_authoring_frame(&next).ok().map(|frame| frame.source)
    }

    pub fn cancel_last(&mut self) {
        let _ = self.log.cancel_last_undo();
    }

    pub fn clear(&mut self) {
        self.log.clear();
    }

    pub fn can_undo(&self) -> bool {
        self.log.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.log.can_redo()
    }
}

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

    #[test]
    fn authoring_undo_redo_roundtrip_preserves_source() {
        let mut undo = AuthoringUndo::new(10);
        let v1 = "(import graphics/page only a4 page)\n(val main (page a4 (list)))\n";
        let v2 = "(import graphics/page only a4 page)\n(val main (page a4 (circle 1 2 3)))\n";
        undo.push(v1);
        assert_eq!(undo.undo(v2).as_deref(), Some(v1));
        assert_eq!(undo.redo(v1).as_deref(), Some(v2));
    }

    #[test]
    fn authoring_undo_frame_carries_snapshot_when_valid() {
        let src = include_str!("../../../examples/text_line.rpx");
        let frame = AuthoringUndo::capture_frame(src);
        assert!(frame.snapshot.is_some());
        let round = decode_authoring_frame(&encode_authoring_frame(&frame)).expect("roundtrip");
        assert_eq!(round.source, src);
        assert!(round.snapshot.is_some());
    }
}
