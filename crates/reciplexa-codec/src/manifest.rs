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

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str) -> ResourceEntry {
        ResourceEntry {
            id: id.into(),
            kind: "image".into(),
            path: Some(format!("{id}.png")),
            content_sha256: None,
        }
    }

    #[test]
    fn insert_sorts_by_id() {
        let mut m = ResourceManifest::default();
        m.insert(entry("z"));
        m.insert(entry("a"));
        m.insert(entry("m"));
        let ids: Vec<_> = m.entries.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["a", "m", "z"]);
    }

    #[test]
    fn upsert_overwrites_existing() {
        let mut m = ResourceManifest::default();
        m.insert(entry("img1"));
        m.insert(ResourceEntry {
            id: "img1".into(),
            kind: "font".into(),
            path: Some("font.ttf".into()),
            content_sha256: Some("deadbeef".into()),
        });
        let e = m.get("img1").unwrap();
        assert_eq!(e.kind, "font");
        assert_eq!(m.entries.len(), 1);
    }

    #[test]
    fn get_miss_returns_none() {
        let m = ResourceManifest::default();
        assert!(m.get("nope").is_none());
    }
}
