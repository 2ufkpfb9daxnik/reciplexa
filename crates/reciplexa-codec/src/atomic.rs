//! Atomic file write helper.

use std::fs;
use std::io;
use std::path::Path;

/// Write bytes atomically via temp file + rename.
pub fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    let parent = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("reciplexa");
    let tmp = parent.join(format!(".{name}.tmp"));
    fs::write(&tmp, data)?;
    fs::rename(&tmp, path)?;
    Ok(())
}
