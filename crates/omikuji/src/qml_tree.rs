use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn walk_files(dir: &Path, exts: &[&str]) -> io::Result<Vec<PathBuf>> {
    let mut out = vec![];
    for entry in fs::read_dir(dir)? {
        let p = entry?.path();
        if p.is_dir() {
            out.extend(walk_files(&p, exts)?);
        } else if p
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|ext| exts.contains(&ext))
        {
            out.push(p);
        }
    }
    out.sort();
    Ok(out)
}

pub fn is_singleton(qml: &Path) -> bool {
    fs::read_to_string(qml).is_ok_and(|src| src.lines().any(|l| l.trim() == "pragma Singleton"))
}
