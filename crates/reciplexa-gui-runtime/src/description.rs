//! GUI description tree (immutable per frame).

use reciplexa_identity::document::StableNodeId;
use reciplexa_identity::gui::WidgetKeyPath;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GuiDescription {
    pub roots: Vec<GuiNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuiNode {
    pub key: WidgetKeyPath,
    pub stable_id: Option<StableNodeId>,
    pub kind: GuiNodeKind,
    pub children: Vec<GuiNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiNodeKind {
    Canvas,
    LayerList,
    Properties,
    TextField,
    Button,
    Group,
}

impl GuiDescription {
    pub fn from_stable_nodes(nodes: &[(WidgetKeyPath, StableNodeId, GuiNodeKind)]) -> Self {
        Self {
            roots: nodes
                .iter()
                .map(|(key, id, kind)| GuiNode {
                    key: key.clone(),
                    stable_id: Some(*id),
                    kind: *kind,
                    children: Vec::new(),
                })
                .collect(),
        }
    }

    pub fn find_by_key(&self, key: &WidgetKeyPath) -> Option<&GuiNode> {
        self.roots.iter().find(|n| &n.key == key)
    }
}
