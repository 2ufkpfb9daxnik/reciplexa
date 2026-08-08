use reciplexa_document::*;
use reciplexa_identity::document::DocumentIdentity;



#[test]
fn new_snapshot_has_document_root() {
    let snap = DocumentSnapshot::new(DocumentIdentity::new(1));
    assert!(snap.nodes.root_id().is_some());
}
