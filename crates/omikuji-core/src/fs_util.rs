use std::path::{Path, PathBuf};

#[cfg(unix)]
pub fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = std::fs::metadata(path) {
        let mode = metadata.permissions().mode();
        mode & 0o111 != 0
    } else {
        false
    }
}

#[cfg(not(unix))]
pub fn is_executable(_path: &Path) -> bool {
    true
}

pub fn find_executable_in_paths(names: &[&str], extra_paths: &[&str]) -> Option<PathBuf> {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            for name in names {
                let full_path = Path::new(dir).join(name);
                if full_path.exists() && is_executable(&full_path) {
                    return Some(full_path);
                }
            }
        }
    }
    for path in extra_paths {
        let expanded = shellexpand::tilde(path);
        let p = Path::new(expanded.as_ref());
        if p.exists() && is_executable(p) {
            return Some(p.to_path_buf());
        }
    }
    None
}

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

pub fn dir_size(path: &Path) -> u64 {
    let mut total = 0;
    let mut stack = vec![path.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if meta.is_dir() {
                stack.push(entry.path());
            } else if meta.is_file() {
                total += meta.len();
            }
        }
    }
    total
}

pub fn move_file(src: &Path, dst: &Path) -> std::io::Result<()> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if std::fs::rename(src, dst).is_ok() {
        return Ok(());
    }
    std::fs::copy(src, dst)?;
    std::fs::remove_file(src)
}

pub fn move_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            move_dir_all(&from, &to)?;
            let _ = std::fs::remove_dir(&from);
        } else {
            move_file(&from, &to)?;
        }
    }
    Ok(())
}

pub fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to = dst.join(entry.file_name());
        if ty.is_symlink() {
            let target = std::fs::read_link(entry.path())?;
            std::os::unix::fs::symlink(target, &to)?;
        } else if ty.is_dir() {
            copy_dir_all(&entry.path(), &to)?;
        } else {
            std::fs::copy(entry.path(), &to)?;
        }
    }
    Ok(())
}
