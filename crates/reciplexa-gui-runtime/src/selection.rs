//! Node selection (document identity, not array index).

use reciplexa_identity::document::StableNodeId;

/// Current document-node selection owned by the GUI runtime (not the document).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NodeSelection {
    primary: Option<StableNodeId>,
    additional: Vec<StableNodeId>,
}

impl NodeSelection {
    pub fn primary(&self) -> Option<StableNodeId> {
        self.primary
    }

    pub fn ids(&self) -> impl Iterator<Item = StableNodeId> + '_ {
        self.primary
            .into_iter()
            .chain(self.additional.iter().copied())
    }

    pub fn clear(&mut self) {
        self.primary = None;
        self.additional.clear();
    }

    pub fn select_only(&mut self, id: StableNodeId) {
        self.primary = Some(id);
        self.additional.clear();
    }

    pub fn add(&mut self, id: StableNodeId) {
        if self.primary == Some(id) || self.additional.contains(&id) {
            return;
        }
        if self.primary.is_none() {
            self.primary = Some(id);
        } else {
            self.additional.push(id);
        }
    }

    pub fn contains(&self, id: StableNodeId) -> bool {
        self.primary == Some(id) || self.additional.contains(&id)
    }

    /// Drop ids that no longer exist in the document (owner deletion).
    pub fn retain_existing<F>(&mut self, mut exists: F)
    where
        F: FnMut(StableNodeId) -> bool,
    {
        if let Some(p) = self.primary {
            if !exists(p) {
                self.primary = None;
            }
        }
        self.additional.retain(|id| exists(*id));
        if self.primary.is_none() {
            if let Some(next) = self.additional.first().copied() {
                self.primary = Some(next);
                self.additional.remove(0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retain_existing_clears_deleted_primary() {
        let mut sel = NodeSelection::default();
        sel.select_only(StableNodeId::new(1));
        sel.add(StableNodeId::new(2));
        sel.retain_existing(|id| id.get() == 2);
        assert_eq!(sel.primary(), Some(StableNodeId::new(2)));
    }

    #[test]
    fn add_dedupes_primary_and_additional() {
        let mut sel = NodeSelection::default();
        let id = StableNodeId::new(1);
        sel.select_only(id);
        sel.add(id);
        sel.add(StableNodeId::new(2));
        sel.add(StableNodeId::new(2));
        assert_eq!(sel.primary(), Some(id));
        assert_eq!(sel.additional.len(), 1);
        assert_eq!(sel.additional[0], StableNodeId::new(2));
    }

    #[test]
    fn ids_iterates_primary_then_additional() {
        let mut sel = NodeSelection::default();
        sel.select_only(StableNodeId::new(10));
        sel.add(StableNodeId::new(20));
        sel.add(StableNodeId::new(30));
        let ids: Vec<_> = sel.ids().collect();
        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0], StableNodeId::new(10));
        assert_eq!(ids[1], StableNodeId::new(20));
        assert_eq!(ids[2], StableNodeId::new(30));
    }

    #[test]
    fn retain_promotes_additional_when_primary_deleted() {
        let mut sel = NodeSelection::default();
        sel.select_only(StableNodeId::new(1));
        sel.add(StableNodeId::new(2));
        sel.add(StableNodeId::new(3));
        sel.retain_existing(|id| id.get() != 1);
        assert_eq!(sel.primary(), Some(StableNodeId::new(2)));
        assert_eq!(sel.additional, vec![StableNodeId::new(3)]);
    }

    #[test]
    fn clear_removes_all() {
        let mut sel = NodeSelection::default();
        sel.select_only(StableNodeId::new(1));
        sel.add(StableNodeId::new(2));
        sel.clear();
        assert!(sel.primary().is_none());
        assert!(sel.ids().next().is_none());
        assert!(!sel.contains(StableNodeId::new(1)));
    }

    #[test]
    fn contains_checks_primary_and_additional() {
        let mut sel = NodeSelection::default();
        assert!(!sel.contains(StableNodeId::new(1)));
        sel.select_only(StableNodeId::new(1));
        assert!(sel.contains(StableNodeId::new(1)));
        sel.add(StableNodeId::new(2));
        assert!(sel.contains(StableNodeId::new(2)));
    }

    #[test]
    fn add_to_empty_sets_primary() {
        let mut sel = NodeSelection::default();
        sel.add(StableNodeId::new(7));
        assert_eq!(sel.primary(), Some(StableNodeId::new(7)));
    }
}
