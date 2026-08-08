use reciplexa_gui_runtime::dragdrop::*;
use reciplexa_identity::document::StableNodeId;
use reciplexa_identity::gui::WidgetKeyPath;

#[test]
fn drop_consumes_payload() {
    let mut d = DragSession::default();
    let src = WidgetKeyPath::new().push_named("layer-1");
    d.begin(DragPayload {
        source_key: src.clone(),
        node_id: Some(StableNodeId::new(1)),
        label: "Rect".into(),
    });
    let tgt = WidgetKeyPath::new().push_named("layer-2");
    let payload = d.drop_on(&tgt).unwrap();
    assert_eq!(payload.source_key, src);
    assert!(d.active.is_none());
}

#[test]
fn drop_on_inactive_returns_none() {
    let mut d = DragSession::default();
    let tgt = WidgetKeyPath::new().push_named("target");
    assert!(d.drop_on(&tgt).is_none());
}

#[test]
fn cancel_clears_active_and_hover() {
    let mut d = DragSession::default();
    d.begin(DragPayload {
        source_key: WidgetKeyPath::new().push_named("src"),
        node_id: None,
        label: "x".into(),
    });
    d.set_hover(Some(WidgetKeyPath::new().push_named("hover")));
    d.cancel();
    assert!(d.active.is_none());
    assert!(d.hover_target.is_none());
}

#[test]
fn set_hover_updates_target() {
    let mut d = DragSession::default();
    let hover = WidgetKeyPath::new().push_named("drop-zone");
    d.set_hover(Some(hover.clone()));
    assert_eq!(d.hover_target.as_ref().unwrap(), &hover);
    d.set_hover(None);
    assert!(d.hover_target.is_none());
}
