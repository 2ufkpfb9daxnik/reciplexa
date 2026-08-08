//! Drag-and-drop session state (Phase 8).

use reciplexa_identity::document::StableNodeId;
use reciplexa_identity::gui::WidgetKeyPath;

#[derive(Debug, Clone, PartialEq)]
pub struct DragPayload {
    pub source_key: WidgetKeyPath,
    pub node_id: Option<StableNodeId>,
    pub label: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DragSession {
    pub active: Option<DragPayload>,
    pub hover_target: Option<WidgetKeyPath>,
}

impl DragSession {
    pub fn begin(&mut self, payload: DragPayload) {
        self.active = Some(payload);
        self.hover_target = None;
    }

    pub fn set_hover(&mut self, target: Option<WidgetKeyPath>) {
        self.hover_target = target;
    }

    pub fn drop_on(&mut self, target: &WidgetKeyPath) -> Option<DragPayload> {
        if self.active.is_some() {
            self.hover_target = Some(target.clone());
            self.active.take()
        } else {
            None
        }
    }

    pub fn cancel(&mut self) {
        self.active = None;
        self.hover_target = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
