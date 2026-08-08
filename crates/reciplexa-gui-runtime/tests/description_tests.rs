use reciplexa_gui_runtime::description::*;
use reciplexa_identity::document::StableNodeId;
use reciplexa_identity::gui::WidgetKeyPath;

#[test]
fn find_by_key_miss_returns_none() {
    let desc = GuiDescription::from_stable_nodes(&[(
        WidgetKeyPath::new().push_named("exists"),
        StableNodeId::new(1),
        GuiNodeKind::Button,
    )]);
    let miss = WidgetKeyPath::new().push_named("missing");
    assert!(desc.find_by_key(&miss).is_none());
    let hit = WidgetKeyPath::new().push_named("exists");
    assert!(desc.find_by_key(&hit).is_some());
}
