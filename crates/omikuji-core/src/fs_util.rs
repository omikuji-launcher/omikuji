use fs_err::{DirEntry, ReadDir};
use std::iter;
use std::path::{Path, PathBuf};

#[cfg(unix)]
pub fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = fs_err::metadata(path) {
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

pub fn set_executable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs_err::set_permissions(path, std::fs::Permissions::from_mode(0o755))
}

fn replace_via_tmp(
    path: &Path,
    write: impl FnOnce(&Path) -> std::io::Result<()>,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs_err::create_dir_all(parent)?;
    }
    let tmp = match path.extension() {
        Some(ext) => path.with_extension(format!("{}.tmp", ext.to_string_lossy())),
        None => path.with_extension("tmp"),
    };
    write(&tmp)?;
    fs_err::rename(&tmp, path)
}

pub fn write_atomic(path: &Path, body: impl AsRef<[u8]>) -> std::io::Result<()> {
    replace_via_tmp(path, |tmp| fs_err::write(tmp, body))
}

pub fn write_executable_atomic(path: &Path, body: impl AsRef<[u8]>) -> std::io::Result<()> {
    replace_via_tmp(path, |tmp| {
        fs_err::write(tmp, body)?;
        set_executable(tmp)
    })
}

pub fn walk(root: &Path) -> impl Iterator<Item = DirEntry> {
    let mut pending = vec![root.to_path_buf()];
    let mut current: Option<ReadDir> = None;
    iter::from_fn(move || {
        loop {
            match current.as_mut().and_then(Iterator::next) {
                Some(Ok(entry)) => {
                    if entry.file_type().is_ok_and(|t| t.is_dir()) {
                        pending.push(entry.path());
                    }
                    return Some(entry);
                }
                Some(Err(_)) => continue,
                None => current = fs_err::read_dir(pending.pop()?).ok(),
            }
        }
    })
}

pub fn find_file_named(root: &Path, name: &str) -> Option<PathBuf> {
    walk(root)
        .find(|e| e.file_name() == name && e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| e.path())
}

pub fn dir_size(path: &Path) -> u64 {
    walk(path)
        .filter_map(|e| e.metadata().ok())
        .filter(|meta| meta.is_file())
        .map(|meta| meta.len())
        .sum()
}

pub fn move_file(src: &Path, dst: &Path) -> std::io::Result<()> {
    if let Some(parent) = dst.parent() {
        fs_err::create_dir_all(parent)?;
    }
    if fs_err::rename(src, dst).is_ok() {
        return Ok(());
    }
    fs_err::copy(src, dst)?;
    fs_err::remove_file(src)
}

pub fn move_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs_err::create_dir_all(dst)?;
    for entry in fs_err::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            move_dir_all(&from, &to)?;
            let _ = fs_err::remove_dir(&from);
        } else {
            move_file(&from, &to)?;
        }
    }
    Ok(())
}

pub fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs_err::create_dir_all(dst)?;
    for entry in fs_err::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to = dst.join(entry.file_name());
        if ty.is_symlink() {
            let target = fs_err::read_link(entry.path())?;
            fs_err::os::unix::fs::symlink(target, &to)?;
        } else if ty.is_dir() {
            copy_dir_all(&entry.path(), &to)?;
        } else {
            fs_err::copy(entry.path(), &to)?;
        }
    }
    Ok(())
}
