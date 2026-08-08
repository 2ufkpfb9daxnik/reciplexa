//! Resource manifest — durable refs only (no secrets/capabilities/native pointers).

use serde::{Deserialize, Serialize};

/// Portable resource entry (path or content hash). Never stores secrets or native pointers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceEntry {
    pub id: String,
    pub kind: String,
    pub path: Option<String>,
    pub content_sha256: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceManifest {
    pub entries: Vec<ResourceEntry>,
}

impl ResourceManifest {
    pub fn insert(&mut self, entry: ResourceEntry) {
        if let Some(slot) = self.entries.iter_mut().find(|e| e.id == entry.id) {
            *slot = entry;
        } else {
            self.entries.push(entry);
        }
        self.entries.sort_by(|a, b| a.id.cmp(&b.id));
    }

    pub fn get(&self, id: &str) -> Option<&ResourceEntry> {
        self.entries.iter().find(|e| e.id == id)
    }
}
