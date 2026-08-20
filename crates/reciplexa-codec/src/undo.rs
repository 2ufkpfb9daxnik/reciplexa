//! Revision-based undo/redo log.

use serde::{Deserialize, Serialize};

use crate::codec::{CodecError, PortableSnapshot, SCHEMA_VERSION};
use reciplexa_identity::document::DocumentRevision;

#[derive(Debug, Clone, PartialEq)]
pub enum UndoAction {
    ApplySnapshot {
        before_revision: DocumentRevision,
        after_revision: DocumentRevision,
    },
}

/// One GUI authoring undo step: source text plus an optional document snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthoringUndoFrame {
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<PortableSnapshot>,
}

impl AuthoringUndoFrame {
    pub fn new(source: impl Into<String>, snapshot: Option<PortableSnapshot>) -> Self {
        Self {
            source: source.into(),
            snapshot,
        }
    }
}

pub fn encode_authoring_frame(frame: &AuthoringUndoFrame) -> Vec<u8> {
    serde_json::to_vec(frame).expect("AuthoringUndoFrame is always JSON-serializable")
}

pub fn decode_authoring_frame(bytes: &[u8]) -> Result<AuthoringUndoFrame, CodecError> {
    let frame: AuthoringUndoFrame =
        serde_json::from_slice(bytes).map_err(|e| CodecError::Decode(e.to_string()))?;
    if let Some(ref snap) = frame.snapshot {
        if snap.schema_version > SCHEMA_VERSION {
            return Err(CodecError::UnsupportedSchema(snap.schema_version));
        }
    }
    Ok(frame)
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

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    /// Drop the most recent undo entry without moving current state to redo.
    pub fn cancel_last_undo(&mut self) -> Option<Vec<u8>> {
        self.undo.pop()
    }
}
