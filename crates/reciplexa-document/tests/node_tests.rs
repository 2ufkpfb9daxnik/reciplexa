use reciplexa_document::*;
#[test]
fn remove_missing_and_root_and_link_missing_child() {
    let mut store = NodeStore::new();
    let missing = reciplexa_identity::document::StableNodeId::new(404);
    store.remove_subtree(missing);
    let doc = store.allocate(DocumentNodeKind::Document);
    store.link_child(doc, missing); // child absent
    store.remove_subtree(doc); // root has no parent
    assert!(store.get(doc).is_none());
}

#[test]
fn duplicate_fails_when_parent_is_dangling() {
    let mut store = NodeStore::new();
    let doc = store.allocate(DocumentNodeKind::Document);
    let page = store.insert_child(doc, DocumentNodeKind::Page).unwrap();
    let rect = store
        .insert_child(page, DocumentNodeKind::Rectangle)
        .unwrap();
    store.drop_node_keep_links(page);
    assert!(store.duplicate_subtree(rect).is_none());
}

#[test]
fn duplicate_gets_new_ids() {
    let mut store = NodeStore::new();
    let doc = store.allocate(DocumentNodeKind::Document);
    let rect = store
        .insert_child(doc, DocumentNodeKind::Rectangle)
        .unwrap();
    let dup = store.duplicate_subtree(rect).unwrap();
    assert_ne!(rect, dup);
}

#[test]
fn remove_subtree_deletes_descendants() {
    let mut store = NodeStore::new();
    let doc = store.allocate(DocumentNodeKind::Document);
    let page = store.insert_child(doc, DocumentNodeKind::Page).unwrap();
    let rect = store
        .insert_child(page, DocumentNodeKind::Rectangle)
        .unwrap();
    store.remove_subtree(page);
    assert!(store.get(page).is_none());
    assert!(store.get(rect).is_none());
}

#[test]
fn root_id_is_document() {
    let mut store = NodeStore::new();
    let doc = store.allocate(DocumentNodeKind::Document);
    let root = store.root_id().unwrap();
    assert_eq!(root, doc);
    assert!(matches!(
        store.get(root).unwrap().kind,
        DocumentNodeKind::Document
    ));
}

#[test]
fn layout_and_text_accessors() {
    let mut node = DocumentNode::new(
        reciplexa_identity::document::StableNodeId::new(1),
        DocumentNodeKind::Text,
    );
    assert!(node.layout().is_none());
    let layout = LayoutBox::new(1.0, 2.0, 3.0, 4.0);
    node.set_layout(layout);
    assert_eq!(node.layout(), Some(layout));
    node.properties
        .push(NodeProperty::Text(TextContent { text: "hi".into() }));
    assert_eq!(node.text().unwrap().text, "hi");
    node.set_layout(LayoutBox::new(5.0, 6.0, 7.0, 8.0));
    assert_eq!(node.layout().unwrap().x, 5.0);
}

#[test]
fn insert_preserved_and_link_child() {
    let mut store = NodeStore::new();
    let doc = store.allocate(DocumentNodeKind::Document);
    let id = reciplexa_identity::document::StableNodeId::new(99);
    store.insert_preserved(DocumentNode::new(id, DocumentNodeKind::Page));
    assert_eq!(store.get(id).unwrap().kind, DocumentNodeKind::Page);
    store.link_child(doc, id);
    assert!(store.get(doc).unwrap().children.contains(&id));
}

#[test]
fn insert_child_missing_parent_returns_none() {
    let mut store = NodeStore::new();
    let missing = reciplexa_identity::document::StableNodeId::new(42);
    assert!(store
        .insert_child(missing, DocumentNodeKind::Page)
        .is_none());
}

#[test]
fn duplicate_subtree_copies_children() {
    let mut store = NodeStore::new();
    let doc = store.allocate(DocumentNodeKind::Document);
    let page = store.insert_child(doc, DocumentNodeKind::Page).unwrap();
    let rect = store
        .insert_child(page, DocumentNodeKind::Rectangle)
        .unwrap();
    let dup = store.duplicate_subtree(page).unwrap();
    assert_ne!(page, dup);
    assert_eq!(store.get(dup).unwrap().children.len(), 1);
    assert!(store.get(rect).is_some());
}

#[test]
fn len_and_is_empty() {
    let mut store = NodeStore::new();
    assert!(store.is_empty());
    store.allocate(DocumentNodeKind::Document);
    assert_eq!(store.len(), 1);
    assert!(!store.is_empty());
}

#[test]
fn layout_text_skip_non_matching_properties() {
    let mut node = DocumentNode::new(
        reciplexa_identity::document::StableNodeId::new(1),
        DocumentNodeKind::Text,
    );
    node.properties
        .push(NodeProperty::Fill(FillColor(
            reciplexa_scene::Color::BLACK,
        )));
    assert!(node.layout().is_none());
    assert!(node.text().is_none());
    node.properties.push(NodeProperty::Text(TextContent {
        text: "x".into(),
    }));
    node.set_layout(LayoutBox::new(0.0, 0.0, 1.0, 1.0));
    assert!(node.layout().is_some());
    assert_eq!(node.text().unwrap().text, "x");
}

#[test]
fn insert_preserved_document_sets_root_and_relink_is_idempotent() {
    let mut store = NodeStore::new();
    let id = reciplexa_identity::document::StableNodeId::new(7);
    store.insert_preserved(DocumentNode::new(id, DocumentNodeKind::Document));
    assert_eq!(store.root_id(), Some(id));
    let child = reciplexa_identity::document::StableNodeId::new(8);
    store.insert_preserved(DocumentNode::new(child, DocumentNodeKind::Page));
    store.link_child(id, child);
    store.link_child(id, child);
    assert_eq!(store.get(id).unwrap().children, vec![child]);
}

#[test]
fn remove_subtree_unlinks_from_parent() {
    let mut store = NodeStore::new();
    let doc = store.allocate(DocumentNodeKind::Document);
    let page = store.insert_child(doc, DocumentNodeKind::Page).unwrap();
    store.remove_subtree(page);
    assert!(!store.get(doc).unwrap().children.contains(&page));
    assert!(store.get(page).is_none());
}

#[test]
fn dangling_child_id_is_skipped_by_iterators() {
    let mut store = NodeStore::new();
    let doc = store.allocate(DocumentNodeKind::Document);
    let ghost = reciplexa_identity::document::StableNodeId::new(999);
    store.get_mut(doc).unwrap().children.push(ghost);
    let ids = drawable_node_ids(&store);
    assert!(!ids.contains(&ghost));
    let _ = scene_shapes_from_document(&store);
}

#[test]
fn link_child_and_remove_tolerate_missing_parent() {
    let mut store = NodeStore::new();
    let doc = store.allocate(DocumentNodeKind::Document);
    let page = store.insert_child(doc, DocumentNodeKind::Page).unwrap();
    let missing = reciplexa_identity::document::StableNodeId::new(42);
    store.link_child(missing, page); // parent absent: child.parent still updated
    assert_eq!(store.get(page).unwrap().parent, Some(missing));
    store.get_mut(page).unwrap().parent = Some(missing);
    store.remove_subtree(page); // parent slot missing — still deletes page
    assert!(store.get(page).is_none());
}

#[test]
fn duplicate_subtree_error_partitions() {
    let mut store = NodeStore::new();
    let doc = store.allocate(DocumentNodeKind::Document);
    let missing = reciplexa_identity::document::StableNodeId::new(99);
    assert!(store.duplicate_subtree(missing).is_none());
    assert!(store.duplicate_subtree(doc).is_none());

    let page = store.insert_child(doc, DocumentNodeKind::Page).unwrap();
    let rect = store
        .insert_child(page, DocumentNodeKind::Rectangle)
        .unwrap();
    // Dangling child id makes recursive duplicate fail.
    store.drop_node_keep_links(rect);
    assert!(store.duplicate_subtree(page).is_none());
}
