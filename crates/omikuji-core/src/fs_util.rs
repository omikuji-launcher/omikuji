use std::path::Path;

pub fn write_atomic(path: &Path, body: impl AsRef<[u8]>) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = match path.extension() {
        Some(ext) => path.with_extension(format!("{}.tmp", ext.to_string_lossy())),
        None => path.with_extension("tmp"),
    };
    std::fs::write(&tmp, body)?;
    std::fs::rename(&tmp, path)
}
