use std::path::{Path, PathBuf};

pub fn game_state_dir(publisher_slug: &str, game_slug: &str) -> PathBuf {
    crate::gachas_dir().join(publisher_slug).join(game_slug)
}

pub fn version_file(publisher_slug: &str, game_slug: &str, edition_id: &str) -> PathBuf {
    game_state_dir(publisher_slug, game_slug).join(format!("{}.version", edition_id))
}

pub fn read_installed_version(
    publisher_slug: &str,
    game_slug: &str,
    edition_id: &str,
) -> Option<String> {
    let path = version_file(publisher_slug, game_slug, edition_id);
    std::fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

// errors are logged not returned; caller cant do anything useful with a write failure here
pub fn write_installed_version(
    publisher_slug: &str,
    game_slug: &str,
    edition_id: &str,
    version: &str,
) {
    let path = version_file(publisher_slug, game_slug, edition_id);
    if let Some(parent) = path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        tracing::error!("create_dir_all({}) failed: {}", parent.display(), e);
        return;
    }
    if let Err(e) = std::fs::write(&path, version) {
        tracing::error!("write({}) failed: {}", path.display(), e);
    }
}

pub fn state_path_for(publisher_slug: &str, game_slug: &str) -> impl AsRef<Path> {
    game_state_dir(publisher_slug, game_slug)
}

pub fn read_install_dotversion(install_path: &Path) -> Option<String> {
    let bytes = std::fs::read(install_path.join(".version")).ok()?;
    if bytes.len() == 3 {
        return Some(format!("{}.{}.{}", bytes[0], bytes[1], bytes[2]));
    }
    if bytes.len() < 4 {
        return None;
    }
    let s = String::from_utf8(bytes).ok()?;
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn scan_globalgamemanagers(
    install_path: &Path,
    data_folder: &str,
    terminator: u8,
) -> Option<String> {
    if data_folder.is_empty() {
        return None;
    }
    let path = install_path.join(data_folder).join("globalgamemanagers");
    scan_unity_file(&path, 4000, 524288, terminator)
}

pub fn scan_unity_file(file_path: &Path, skip: u64, take: usize, terminator: u8) -> Option<String> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(file_path).ok()?;
    file.seek(SeekFrom::Start(skip)).ok()?;
    let mut buf = vec![0u8; take];
    let n = file.read(&mut buf).ok()?;
    let window = &buf[..n];

    for pos in 0..window.len() {
        if pos > 0 && window[pos - 1].is_ascii_digit() {
            continue;
        }
        if let Some(v) = try_parse_version_at(window, pos, terminator) {
            return Some(v);
        }
    }

    None
}

fn try_parse_version_at(bytes: &[u8], pos: usize, terminator: u8) -> Option<String> {
    let mut p = pos;
    let mut parts: [Vec<u8>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for (i, part) in parts.iter_mut().enumerate() {
        while p < bytes.len() && bytes[p].is_ascii_digit() && part.len() < 3 {
            part.push(bytes[p]);
            p += 1;
        }
        if part.is_empty() {
            return None;
        }
        if i < 2 {
            if p >= bytes.len() || bytes[p] != b'.' {
                return None;
            }
            p += 1;
        }
    }
    if parts[0].len() > 2 {
        return None;
    }
    if p >= bytes.len() || bytes[p] != terminator {
        return None;
    }
    let major = String::from_utf8(parts[0].clone()).ok()?;
    let minor = String::from_utf8(parts[1].clone()).ok()?;
    let patch = String::from_utf8(parts[2].clone()).ok()?;
    Some(format!("{}.{}.{}", major, minor, patch))
}
