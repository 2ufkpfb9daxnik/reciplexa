#[test]
fn scan_repo_examples_preview() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());
    let mut failed = Vec::new();
    for e in entries {
        let path = e.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rpx") {
            continue;
        }
        let src = std::fs::read_to_string(&path).unwrap();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        match reciplexa::pipeline::document_from_source(&src) {
            Ok(_) => println!("OK  {name}"),
            Err(err) => {
                let msg = err.display();
                println!("NG  {name}  {msg}");
                failed.push(format!("{name}: {msg}"));
            }
        }
    }
    assert!(failed.is_empty(), "preview failed:\n{}", failed.join("\n"));
}
