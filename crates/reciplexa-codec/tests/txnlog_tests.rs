use reciplexa_codec::txnlog::*;

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
    let bytes = encode_segment(&seg);
    let decoded = decode_segment(&bytes).unwrap();
    assert_eq!(decoded, seg);
}

#[test]
fn decode_segment_garbage_fails() {
    let err = decode_segment(b"not-json").unwrap_err();
    assert!(!err.is_empty());
}
