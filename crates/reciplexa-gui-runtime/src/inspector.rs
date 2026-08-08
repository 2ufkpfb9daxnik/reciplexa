//! Inspector and layer-tree description helpers (Phase 8).

use reciplexa_identity::document::StableNodeId;
use reciplexa_identity::gui::WidgetKeyPath;

use crate::description::{GuiDescription, GuiNode, GuiNodeKind};
use crate::selection::NodeSelection;

/// One row in the layer / tree view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerRow {
    pub key: WidgetKeyPath,
    pub node_id: StableNodeId,
    pub label: String,
    pub selected: bool,
}

/// Build a layer-list GUI description from drawable nodes (paint order).
pub fn layer_tree_description(
    layers: &[(StableNodeId, String)],
    selection: &NodeSelection,
) -> GuiDescription {
    let roots = layers
        .iter()
        .enumerate()
        .map(|(i, (id, _label))| GuiNode {
            key: WidgetKeyPath::new()
                .push_named("layer")
                .push_index(i as u32),
            stable_id: Some(*id),
            kind: GuiNodeKind::LayerList,
            children: vec![GuiNode {
                key: WidgetKeyPath::new()
                    .push_named("layer")
                    .push_index(i as u32)
                    .push_named("label"),
                stable_id: Some(*id),
                kind: GuiNodeKind::Button,
                children: Vec::new(),
            }],
        })
        .collect();
    let _ = (selection, layers);
    GuiDescription { roots }
}

/// Flatten layer rows for virtualization / accessibility.
pub fn layer_rows(
    layers: &[(StableNodeId, String)],
    selection: &NodeSelection,
) -> Vec<LayerRow> {
    layers
        .iter()
        .enumerate()
        .map(|(i, (id, label))| LayerRow {
            key: WidgetKeyPath::new()
                .push_named("layer")
                .push_index(i as u32),
            node_id: *id,
            label: label.clone(),
            selected: selection.contains(*id),
        })
        .collect()
}

/// Inspector property sheet keyed by selected node (ephemeral UI only).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InspectorModel {
    pub target: Option<StableNodeId>,
    pub fields: Vec<(String, String)>,
}

impl InspectorModel {
    pub fn for_selection(selection: &NodeSelection, fields: Vec<(String, String)>) -> Self {
        Self {
            target: selection.primary(),
            fields,
        }
    }

    pub fn description(&self) -> GuiDescription {
        let mut roots = vec![GuiNode {
            key: WidgetKeyPath::new().push_named("inspector"),
            stable_id: self.target,
            kind: GuiNodeKind::Properties,
            children: Vec::new(),
        }];
        for (i, (name, _)) in self.fields.iter().enumerate() {
            roots.push(GuiNode {
                key: WidgetKeyPath::new()
                    .push_named("inspector")
                    .push_named(name)
                    .push_index(i as u32),
                stable_id: self.target,
                kind: GuiNodeKind::TextField,
                children: Vec::new(),
            });
        }
        GuiDescription { roots }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_rows_mark_selection() {
        let mut sel = NodeSelection::default();
        sel.select_only(StableNodeId::new(2));
        let rows = layer_rows(
            &[
                (StableNodeId::new(1), "a".into()),
                (StableNodeId::new(2), "b".into()),
            ],
            &sel,
        );
        assert!(!rows[0].selected);
        assert!(rows[1].selected);
    }
}
