//! Transaction log segments — never partially apply a torn transaction.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentStatus {
    Open,
    Committed,
    Aborted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogRecord {
    pub seq: u64,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionLogSegment {
    pub segment_id: u64,
    pub status: SegmentStatus,
    pub records: Vec<LogRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogError {
    TornSegment,
    AlreadyClosed,
    EmptyCommit,
}

impl TransactionLogSegment {
    pub fn open(segment_id: u64) -> Self {
        Self {
            segment_id,
            status: SegmentStatus::Open,
            records: Vec::new(),
        }
    }

    pub fn append(&mut self, payload: Vec<u8>) -> Result<(), LogError> {
        if self.status != SegmentStatus::Open {
            return Err(LogError::AlreadyClosed);
        }
        let seq = self.records.len() as u64;
        self.records.push(LogRecord { seq, payload });
        Ok(())
    }

    pub fn commit(&mut self) -> Result<(), LogError> {
        if self.status != SegmentStatus::Open {
            return Err(LogError::AlreadyClosed);
        }
        if self.records.is_empty() {
            return Err(LogError::EmptyCommit);
        }
        self.status = SegmentStatus::Committed;
        Ok(())
    }

    pub fn abort(&mut self) {
        self.status = SegmentStatus::Aborted;
        self.records.clear();
    }

    /// Only committed segments may be applied. Torn/open segments are rejected.
    pub fn applyable_records(&self) -> Result<&[LogRecord], LogError> {
        match self.status {
            SegmentStatus::Committed => Ok(&self.records),
            SegmentStatus::Open | SegmentStatus::Aborted => Err(LogError::TornSegment),
        }
    }
}

/// Encode/decode segment for durable storage.
pub fn encode_segment(seg: &TransactionLogSegment) -> Result<Vec<u8>, String> {
    serde_json::to_vec(seg).map_err(|e| e.to_string())
}

pub fn decode_segment(bytes: &[u8]) -> Result<TransactionLogSegment, String> {
    serde_json::from_slice(bytes).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_torn_segment() {
        let mut seg = TransactionLogSegment::open(1);
        seg.append(b"a".to_vec()).unwrap();
        assert!(seg.applyable_records().is_err());
        seg.commit().unwrap();
        assert_eq!(seg.applyable_records().unwrap().len(), 1);
    }

    #[test]
    fn append_after_close_returns_already_closed() {
        let mut seg = TransactionLogSegment::open(1);
        seg.append(b"x".to_vec()).unwrap();
        seg.commit().unwrap();
        let err = seg.append(b"y".to_vec()).unwrap_err();
        assert!(matches!(err, LogError::AlreadyClosed));
    }

    #[test]
    fn commit_empty_returns_empty_commit() {
        let mut seg = TransactionLogSegment::open(2);
        let err = seg.commit().unwrap_err();
        assert!(matches!(err, LogError::EmptyCommit));
    }

    #[test]
    fn commit_twice_returns_already_closed() {
        let mut seg = TransactionLogSegment::open(3);
        seg.append(b"a".to_vec()).unwrap();
        seg.commit().unwrap();
        let err = seg.commit().unwrap_err();
        assert!(matches!(err, LogError::AlreadyClosed));
    }

    #[test]
    fn abort_makes_segment_torn() {
        let mut seg = TransactionLogSegment::open(4);
        seg.append(b"a".to_vec()).unwrap();
        seg.abort();
        assert!(matches!(seg.status, SegmentStatus::Aborted));
        assert!(seg.records.is_empty());
        assert!(matches!(
            seg.applyable_records().unwrap_err(),
            LogError::TornSegment
        ));
    }

    #[test]
    fn encode_decode_roundtrip() {
        let mut seg = TransactionLogSegment::open(5);
        seg.append(b"one".to_vec()).unwrap();
        seg.append(b"two".to_vec()).unwrap();
        seg.commit().unwrap();
        let bytes = encode_segment(&seg).unwrap();
        let decoded = decode_segment(&bytes).unwrap();
        assert_eq!(decoded, seg);
    }

    #[test]
    fn decode_segment_garbage_fails() {
        let err = decode_segment(b"not-json").unwrap_err();
        assert!(!err.is_empty());
    }
}
