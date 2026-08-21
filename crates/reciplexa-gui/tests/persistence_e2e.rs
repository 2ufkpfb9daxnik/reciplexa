//! Step 10: editor persistence wiring (atomic save, recovery).

use reciplexa_codec::{atomic_write, write_journal, RecoveryPaths};
use reciplexa_document::snapshot::DocumentSnapshot;
use reciplexa_gui::persistence::recover_snapshot_sidecar;
use reciplexa_identity::document::DocumentIdentity;

const TEXT_LINE: &str = include_str!("../../../examples/text_line.rpx");

#[test]
fn rpx_atomic_save_roundtrip_and_no_temp_sidecar() {
    let dir = std::env::temp_dir().join(format!("reciplexa-atomic-save-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("text_line.rpx");
    atomic_write(&path, TEXT_LINE.as_bytes()).expect("atomic save");
    let tmp = dir.join(".text_line.rpx.tmp");
    assert!(!tmp.exists(), "temp sidecar must be removed after rename");
    let reloaded = std::fs::read_to_string(&path).expect("reload");
    assert_eq!(reloaded, TEXT_LINE);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn rpx_atomic_save_overwrites_prior_content() {
    let dir =
        std::env::temp_dir().join(format!("reciplexa-atomic-overwrite-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("doc.rpx");
    atomic_write(&path, b"v1").expect("first save");
    atomic_write(&path, b"v2-longer").expect("second save");
    let bytes = std::fs::read(&path).expect("read");
    assert_eq!(bytes, b"v2-longer");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn snapshot_sidecar_recovery_writes_recovered_without_touching_rpx() {
    let dir = std::env::temp_dir().join(format!("reciplexa-snap-recovery-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let rpx = dir.join("doc.rpx");
    atomic_write(&rpx, TEXT_LINE.as_bytes()).expect("write rpx");
    let paths = RecoveryPaths::for_rpx_source(&rpx);
    let snap = DocumentSnapshot::new(DocumentIdentity::new(42));
    write_journal(&paths, &snap).expect("journal");
    assert!(!paths.primary.exists());

    let note = recover_snapshot_sidecar(&rpx).expect("recover");
    assert!(note.is_some());
    assert!(paths.primary.exists());
    assert!(!paths.journal.exists());
    assert!(!paths.recovered.exists());
    assert_eq!(
        std::fs::read_to_string(&rpx).expect("rpx unchanged"),
        TEXT_LINE
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn authoring_undo_roundtrip_via_revision_log() {
    use reciplexa_gui::persistence::AuthoringUndo;

    let mut undo = AuthoringUndo::new(10);
    let v1 = TEXT_LINE;
    let v2 = TEXT_LINE.replace("circle", "ellipse");
    undo.push(v1);
    assert_eq!(undo.undo(&v2).as_deref(), Some(v1));
    assert_eq!(undo.redo(v1).as_deref(), Some(v2.as_str()));
}

#[test]
fn layer_selection_rebase_survives_undo_source_swap() {
    use reciplexa_gui::rebase::{capture_layer_anchors, resolve_layer_indices};

    let v1 = TEXT_LINE;
    let v2 = TEXT_LINE.replace("circle", "ellipse");
    let anchors = capture_layer_anchors(v1, 0, &[0]);
    assert!(!anchors.is_empty());
    assert_eq!(resolve_layer_indices(&v2, 0, &anchors), vec![0]);
    assert_eq!(resolve_layer_indices(v1, 0, &anchors), vec![0]);
}

#[test]
fn migrate_existing_sidecar_drops_unknown_extensions() {
    use reciplexa_codec::{
        load_sidecar_bytes, snapshot_to_portable_public, ExtensibleEnvelope, ExtensionBlock,
        RecoveryPaths,
    };
    use reciplexa_document::snapshot::DocumentSnapshot;
    use reciplexa_gui::persistence::migrate_existing_sidecar;
    use reciplexa_identity::document::DocumentIdentity;
    use serde_json::json;

    let dir =
        std::env::temp_dir().join(format!("reciplexa-migrate-sidecar-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let rpx = dir.join("doc.rpx");
    let paths = RecoveryPaths::for_rpx_source(&rpx);
    let snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    let env = ExtensibleEnvelope {
        snapshot: snapshot_to_portable_public(&snap),
        extensions: vec![ExtensionBlock {
            name: "vendor.unknown".into(),
            version: 99,
            payload: json!(null),
        }],
    };
    let bytes = serde_json::to_vec(&env).expect("encode envelope");
    std::fs::write(&paths.primary, bytes).expect("write sidecar");
    let notice = migrate_existing_sidecar(&rpx)
        .expect("migrate")
        .expect("notice");
    assert!(notice.contains("vendor.unknown@v99"));
    let reloaded = load_sidecar_bytes(&std::fs::read(&paths.primary).expect("read")).expect("load");
    assert!(reloaded.dropped_extensions.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}
