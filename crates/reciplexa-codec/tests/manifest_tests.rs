use reciplexa_codec::manifest::*;

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
