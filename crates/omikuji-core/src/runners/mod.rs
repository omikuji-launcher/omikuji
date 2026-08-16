use crate::archive_source;
use crate::components_config::{self, ArchiveSource};
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod dll_override;

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

pub struct AdvisedRunner {
    pub source: ArchiveSource,
    pub tag: String,
    pub registered: bool,
}

impl AdvisedRunner {
    pub fn dest_root(&self) -> PathBuf {
        if self.registered {
            source_root(&self.source)
        } else {
            runners_dir()
        }
    }
}

pub fn resolve_advised(link: &str) -> Option<AdvisedRunner> {
    let rest = link.split_once("://").map(|(_, r)| r).unwrap_or(link);
    let mut parts = rest.trim_end_matches('/').split('/');
    let host = parts.next()?;
    let owner = parts.next()?;
    let repo = parts.next()?;
    let tag = match (parts.next(), parts.next()) {
        (Some("releases"), Some("tag")) => parts.next()?.to_string(),
        _ => return None,
    };
    if host.is_empty() || owner.is_empty() || repo.is_empty() || tag.is_empty() {
        return None;
    }

    let needle = format!("{}/{}", owner, repo);
    if let Some(source) = list_sources()
        .into_iter()
        .find(|s| s.api_url.contains(&needle))
    {
        return Some(AdvisedRunner {
            source,
            tag,
            registered: true,
        });
    }

    let api_url = if host == "github.com" {
        format!("https://api.github.com/repos/{}/releases", needle)
    } else {
        format!("https://{}/api/v1/repos/{}/releases", host, needle)
    };
    Some(AdvisedRunner {
        source: ArchiveSource {
            name: repo.to_string(),
            kind: "proton".to_string(),
            api_url,
            desc: String::new(),
        },
        tag,
        registered: false,
    })
}

pub fn delete_version(source: &ArchiveSource, tag: &str) -> Result<()> {
    archive_source::delete_version(source, &source_root(source), tag)
}

pub fn is_proton_dir(path: &Path) -> bool {
    path.join("proton").exists()
}

pub fn move_to_steam_dir(src: &Path, roots: &[PathBuf]) -> Result<()> {
    use anyhow::bail;
    if !src.is_dir() {
        bail!("not a runner directory: {}", src.display());
    }
    let name = src
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("bad runner path: {}", src.display()))?;
    if !src.join("compatibilitytool.vdf").exists() {
        bail!("{name} ships no compatibilitytool.vdf, Steam would not list it");
    }
    let mut targets = vec![];
    for root in roots {
        let ctd = root.join("compatibilitytools.d");
        std::fs::create_dir_all(&ctd)?;
        targets.push(ctd.join(name));
    }
    for dest in &targets {
        let _ = std::fs::remove_dir_all(dest);
    }
    if let [dest] = targets.as_slice()
        && std::fs::rename(src, dest).is_ok()
    {
        return Ok(());
    }
    for dest in &targets {
        crate::fs_util::copy_dir_all(src, dest)?;
    }
    std::fs::remove_dir_all(src)?;
    Ok(())
}

fn is_runner_dir(path: &Path) -> bool {
    path.join("bin/wine").exists() || path.join("files/bin/wine64").exists() || is_proton_dir(path)
}

pub fn delete_found_runner(path: &Path) -> Result<()> {
    if !is_runner_dir(path) {
        anyhow::bail!("not a runner directory: {}", path.display());
    }
    std::fs::remove_dir_all(path)?;
    Ok(())
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

pub fn runner_dir(version: &str) -> Option<PathBuf> {
    if version.is_empty() || version == "system" {
        return None;
    }
    match version.strip_prefix("steam:") {
        Some(rest) => crate::store::steam::local::find_proton_install(rest),
        None => installed_runner_dir(version),
    }
}

fn iter_local_runner_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![];
    if let Ok(entries) = std::fs::read_dir(runners_dir()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if is_runner_dir(&path) {
                dirs.push(path);
                continue;
            }
            if let Ok(children) = std::fs::read_dir(&path) {
                for child in children.flatten() {
                    let child_path = child.path();
                    if child_path.is_dir() && is_runner_dir(&child_path) {
                        dirs.push(child_path);
                    }
                }
            }
        }
    }
    dirs
}

fn runner_kind(path: &Path) -> &'static str {
    if is_proton_dir(path) {
        "proton"
    } else {
        "wine"
    }
}

#[derive(Serialize)]
pub struct FoundRunner {
    pub name: String,
    pub kind: String,
    pub origin: String,
    pub path: String,
}

pub fn found_runners() -> Vec<FoundRunner> {
    let mut out = vec![];
    let mut push = |path: &Path, origin: &str| {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            out.push(FoundRunner {
                name: name.to_string(),
                kind: runner_kind(path).to_string(),
                origin: origin.to_string(),
                path: path.to_string_lossy().into_owned(),
            });
        }
    };
    for path in iter_local_runner_dirs() {
        push(&path, "Omikuji");
    }
    for (_name, path) in crate::store::steam::local::iter_steam_protons() {
        push(&path, "Steam");
    }
    out
}

pub fn list_installed_runners() -> Vec<(String, String, String)> {
    let mut runners = vec![];

    for path in iter_local_runner_dirs() {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            runners.push((name.to_string(), String::new(), runner_kind(&path).to_string()));
        }
    }

    for (name, path) in crate::store::steam::local::iter_steam_protons() {
        let label = crate::store::steam::local::proton_display_name(&path).unwrap_or_default();
        runners.push((format!("steam:{name}"), label, "proton".to_string()));
    }

    for name in system_wine_paths().keys() {
        runners.push((format!("system:{name}"), String::new(), "wine".to_string()));
    }

    if which::which("wine").is_ok() {
        runners.push(("system".to_string(), String::new(), "wine".to_string()));
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
            let Some(name) = dir.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
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
            if line.contains("VGA")
                || line.contains("3D controller")
                || line.contains("Display controller")
            {
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
