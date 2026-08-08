use reciplexa_codec::undo::*;

#[test]
fn undo_redo_stack() {
    let mut log = RevisionUndoLog::new(10);
    log.push_undo(b"v1".to_vec());
    assert!(log.can_undo());
    let cur = b"v2".to_vec();
    let prev = log.undo(cur).unwrap();
    assert_eq!(prev, b"v1");
    assert!(log.can_redo());
}

#[test]
fn redo_restores_undone_state() {
    let mut log = RevisionUndoLog::new(10);
    log.push_undo(b"v1".to_vec());
    let cur = b"v2".to_vec();
    let prev = log.undo(cur.clone()).unwrap();
    assert_eq!(prev, b"v1");
    let again = log.redo(b"v1".to_vec()).unwrap();
    assert_eq!(again, b"v2");
}

#[test]
fn max_depth_evicts_oldest() {
    let mut log = RevisionUndoLog::new(2);
    log.push_undo(b"a".to_vec());
    log.push_undo(b"b".to_vec());
    log.push_undo(b"c".to_vec());
    // Eviction leaves the two newest entries; oldest "a" is gone.
    let first = log.undo(b"current".to_vec()).unwrap();
    assert_eq!(first, b"c");
    let second = log.undo(b"after_c".to_vec()).unwrap();
    assert_eq!(second, b"b");
    assert!(log.undo(b"after_b".to_vec()).is_none());
}

#[test]
fn undo_empty_returns_none() {
    let mut log = RevisionUndoLog::new(5);
    assert!(log.undo(b"x".to_vec()).is_none());
}

#[test]
fn redo_empty_returns_none() {
    let mut log = RevisionUndoLog::new(5);
    assert!(log.redo(b"x".to_vec()).is_none());
}

#[test]
fn push_clears_redo_stack() {
    let mut log = RevisionUndoLog::new(10);
    log.push_undo(b"v1".to_vec());
    log.undo(b"v2".to_vec()).unwrap();
    assert!(log.can_redo());
    log.push_undo(b"v3".to_vec());
    assert!(!log.can_redo());
}
