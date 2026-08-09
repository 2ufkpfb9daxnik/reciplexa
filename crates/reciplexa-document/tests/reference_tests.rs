use reciplexa_document::*;
use reciplexa_identity::document::StableNodeId;

#[test]
fn links_parent_to_child() {
    let mut g = ReferenceGraph::new();
    let p = StableNodeId::new(1);
    let c = StableNodeId::new(2);
    g.link(p, c);
    assert_eq!(g.children_of(p), &[c]);
}
