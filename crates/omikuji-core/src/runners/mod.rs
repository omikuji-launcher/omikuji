use crate::archive_source;
use crate::components_config::{self, ArchiveSource};
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn runners_dir() -> PathBuf {
    crate::runners_dir()
}

pub fn list_sources() -> Vec<ArchiveSource> {
    components_config::get().runners
}

pub fn source_by_name(name: &str) -> Option<ArchiveSource> {
    list_sources().into_iter().find(|s| s.name == name)
}

pub fn source_root(source: &ArchiveSource) -> PathBuf {
    runners_dir().join(&source.name)
}

pub async fn fetch_versions(source: &ArchiveSource) -> Result<Vec<archive_source::ReleaseInfo>> {
    archive_source::fetch_versions(source).await
}

pub async fn install_version(
    source: &ArchiveSource,
    release: &archive_source::ReleaseInfo,
) -> Result<PathBuf> {
    archive_source::install_version("runners", source, release, &source_root(source)).await
}

pub fn list_installed(source: &ArchiveSource) -> Vec<String> {
    archive_source::list_installed(source, &source_root(source))
}

pub fn delete_version(source: &ArchiveSource, tag: &str) -> Result<()> {
    archive_source::delete_version(source, &source_root(source), tag)
}

fn is_runner_dir(path: &Path) -> bool {
    path.join("bin/wine").exists()
        || path.join("files/bin/wine64").exists()
        || path.join("proton").exists()
}

pub fn installed_runner_dir(version: &str) -> Option<PathBuf> {
    let root = runners_dir();
    let direct = root.join(version);
    if direct.is_dir() {
        return Some(direct);
    }
    std::fs::read_dir(&root)
        .ok()?
        .flatten()
        .map(|e| e.path().join(version))
        .find(|p| p.is_dir())
}

pub fn list_installed_runners() -> Vec<(String, String)> {
    let mut runners = vec![];

    let push_runner = |list: &mut Vec<(String, String)>, path: &Path| {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            list.push((name.to_string(), String::new()));
        }
    };

    if let Ok(entries) = std::fs::read_dir(runners_dir()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if is_runner_dir(&path) {
                push_runner(&mut runners, &path);
                continue;
            }
            if let Ok(children) = std::fs::read_dir(&path) {
                for child in children.flatten() {
                    let child_path = child.path();
                    if child_path.is_dir() && is_runner_dir(&child_path) {
                        push_runner(&mut runners, &child_path);
                    }
                }
            }
        }
    }

    for (name, path) in crate::steam::local::iter_steam_protons() {
        let label = crate::steam::local::proton_display_name(&path).unwrap_or_default();
        runners.push((format!("steam:{name}"), label));
    }

    for name in system_wine_paths().keys() {
        runners.push((format!("system:{name}"), String::new()));
    }

    if which::which("wine").is_ok() {
        runners.push(("system".to_string(), String::new()));
    }

    runners.sort();
    runners.dedup();
    runners
}

pub fn system_wine_paths() -> HashMap<String, PathBuf> {
    let mut paths = HashMap::new();

    let hardcoded: &[(&str, &str)] = &[
        ("winehq-devel", "/opt/wine-devel/bin/wine"),
        ("winehq-staging", "/opt/wine-staging/bin/wine"),
        ("wine-development", "/usr/lib/wine-development/wine"),
    ];
    for (name, path) in hardcoded {
        let p = PathBuf::from(path);
        if p.is_file() {
            paths.insert((*name).to_string(), p);
        }
    }

    if let Ok(entries) = std::fs::read_dir("/usr/lib") {
        for entry in entries.flatten() {
            let dir = entry.path();
            let Some(name) = dir.file_name().and_then(|n| n.to_str()) else { continue };
            if name.starts_with("wine-") && !paths.contains_key(name) {
                let wine_bin = dir.join("bin/wine");
                if wine_bin.is_file() {
                    paths.insert(name.to_string(), wine_bin);
                }
            }
        }
    }

    paths
}

fn clean_lspci(name: &str) -> String {
    name.replace("Advanced Micro Devices, Inc.", "AMD")
        .replace("NVIDIA Corporation", "NVIDIA")
        .replace("Intel Corporation", "Intel")
        .replace("Corp.", "")
}

pub fn list_gpus() -> Vec<(String, String)> {
    let mut gpus = vec![("Default".to_string(), "".to_string())];

    let vk = crate::system_info::gpu_select_list();
    if !vk.is_empty() {
        gpus.extend(vk);
        return gpus;
    }

    if let Ok(output) = Command::new("lspci").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("VGA") || line.contains("3D controller") || line.contains("Display controller") {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() >= 2 {
                    let desc = parts[1].trim();
                    if let Some(idx) = desc.find(':') {
                        gpus.push((clean_lspci(desc[idx + 1..].trim()), String::new()));
                    }
                }
            }
        }
    }

    gpus
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_runners_dir() {
        let dir = runners_dir();
        assert!(dir.to_string_lossy().contains("omikuji"));
    }
    
    #[test]
    fn test_list_gpus() {
        let gpus = list_gpus();
        assert!(!gpus.is_empty());
        assert_eq!(gpus[0].0, "Default");
    }
}
