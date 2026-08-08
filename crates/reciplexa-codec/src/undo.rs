//! Revision-based undo/redo log.

use reciplexa_identity::document::DocumentRevision;

#[derive(Debug, Clone, PartialEq)]
pub enum UndoAction {
    ApplySnapshot {
        before_revision: DocumentRevision,
        after_revision: DocumentRevision,
    },
}

#[derive(Debug, Clone, Default)]
pub struct RevisionUndoLog {
    undo: Vec<Vec<u8>>,
    redo: Vec<Vec<u8>>,
    max_depth: usize,
}

impl RevisionUndoLog {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            max_depth,
        }
    }

    pub fn push_undo(&mut self, snapshot_bytes: Vec<u8>) {
        self.undo.push(snapshot_bytes);
        if self.undo.len() > self.max_depth {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn undo(&mut self, current: Vec<u8>) -> Option<Vec<u8>> {
        let prev = self.undo.pop()?;
        self.redo.push(current);
        Some(prev)
    }

    pub fn redo(&mut self, current: Vec<u8>) -> Option<Vec<u8>> {
        let next = self.redo.pop()?;
        self.undo.push(current);
        Some(next)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
