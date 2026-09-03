use crate::archive_source;
use crate::components_config::{self, ArchiveSource};
use anyhow::{Result, anyhow};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{LazyLock, Mutex};

pub mod dll_override;

pub fn runners_dir() -> PathBuf {
    crate::runners_dir()
}

pub fn list_sources() -> Vec<ArchiveSource> {
    components_config::get().runners
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

pub const LATEST_SUFFIX: &str = "-Latest";
pub const LATEST_CATEGORY: &str = "runners_latest";

pub fn latest_dir_name(source: &ArchiveSource) -> String {
    format!("{}{}", source.name, LATEST_SUFFIX)
}

pub fn latest_dir(source: &ArchiveSource) -> PathBuf {
    source_root(source).join(latest_dir_name(source))
}

pub fn latest_source(version: &str) -> Option<ArchiveSource> {
    let name = version.strip_suffix(LATEST_SUFFIX)?;
    list_sources().into_iter().find(|s| s.name == name)
}

async fn latest_release(source: &ArchiveSource) -> Result<archive_source::ReleaseInfo> {
    let mut versions = fetch_versions(source).await?.into_iter();
    if !source.require_asset_match || source.asset_priority.is_empty() {
        return versions
            .next()
            .ok_or_else(|| anyhow!("{} has no installable release", source.name));
    }
    versions
        .find(|r| archive_source::matches_priority(&r.asset_name, &source.asset_priority))
        .ok_or_else(|| {
            anyhow!(
                "no {} release has a build matching {}",
                source.name,
                source.asset_priority.join(" ")
            )
        })
}

static UPDATING_LATEST: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(Default::default);

struct UpdatingGuard(String);

impl Drop for UpdatingGuard {
    fn drop(&mut self) {
        if let Ok(mut set) = UPDATING_LATEST.lock() {
            set.remove(&self.0);
        }
    }
}

pub fn is_latest_updating(version: &str) -> bool {
    let Some(name) = version.strip_suffix(LATEST_SUFFIX) else {
        return false;
    };
    UPDATING_LATEST
        .lock()
        .map(|set| set.contains(name))
        .unwrap_or(false)
}

pub async fn install_latest_release(
    source: &ArchiveSource,
    release: &archive_source::ReleaseInfo,
) -> Result<PathBuf> {
    if let Ok(mut set) = UPDATING_LATEST.lock() {
        set.insert(source.name.clone());
    }
    let _guard = UpdatingGuard(source.name.clone());

    let dir = archive_source::install_version_named(
        LATEST_CATEGORY,
        source,
        release,
        &source_root(source),
        &latest_dir_name(source),
    )
    .await?;

    if !steam_links(&dir).is_empty()
        && let Err(e) = stamp_steam_identity(&dir)
    {
        tracing::warn!("{} keeps its upstream Steam name: {:#}", dir.display(), e);
    }
    Ok(dir)
}

fn stamp_steam_identity(dir: &Path) -> Result<()> {
    if !dir.join("compatibilitytool.vdf").exists() {
        return Ok(());
    }
    let name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("bad runner path: {}", dir.display()))?;
    crate::store::steam::local::set_compat_tool_name(dir, name)
}

pub async fn ensure_latest(source: &ArchiveSource) -> Result<PathBuf> {
    let dir = latest_dir(source);
    if dir.exists() {
        return Ok(dir);
    }
    let release = latest_release(source).await?;
    install_latest_release(source, &release).await
}

pub fn latest_sources_for<I, S>(selections: I) -> Vec<ArchiveSource>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut names: Vec<String> = selections
        .into_iter()
        .filter_map(|s| s.as_ref().strip_suffix(LATEST_SUFFIX).map(str::to_string))
        .collect();
    names.sort();
    names.dedup();

    let sources = list_sources();
    names
        .into_iter()
        .filter_map(|n| sources.iter().find(|s| s.name == n).cloned())
        .collect()
}

pub fn ensure_latest_blocking(version: &str) -> Result<()> {
    let Some(source) = latest_source(version) else {
        return Ok(());
    };
    if is_latest_updating(version) {
        anyhow::bail!("{version} is being updated right now, try again once it finishes");
    }
    if latest_dir(&source).exists() {
        return Ok(());
    }

    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(ensure_latest(&source))
    })
    .join()
    .map_err(|_| anyhow!("runner install thread panicked"))?
    .map(|_| ())
}

pub fn latest_options() -> Vec<(String, String, String)> {
    list_sources()
        .into_iter()
        .map(|s| (latest_dir_name(&s), String::new(), s.kind))
        .collect()
}

pub fn list_runner_options() -> Vec<(String, String, String)> {
    let ignore_steam = steam_runners_ignored();
    let mut out: Vec<_> = list_installed_runners()
        .into_iter()
        .filter(|(value, _, _)| !value.ends_with(LATEST_SUFFIX))
        .filter(|(value, _, _)| !(ignore_steam && value.starts_with("steam:")))
        .collect();
    out.extend(latest_options());
    out
}

pub async fn latest_update(source: &ArchiveSource) -> Result<Option<archive_source::ReleaseInfo>> {
    let release = latest_release(source).await?;
    let installed = archive_source::installed_source_tag(&latest_dir(source)).map(|(_, tag)| tag);
    Ok((installed.as_deref() != Some(release.tag.as_str())).then_some(release))
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
    let repo = archive_source::RepoLink::parse(link)?;
    let rest = link.split_once("://").map(|(_, r)| r).unwrap_or(link);
    let mut tail = rest.trim_end_matches('/').split('/').skip(3);
    let tag = match (tail.next(), tail.next()) {
        (Some("releases"), Some("tag")) => tail.next()?.to_string(),
        _ => return None,
    };
    if tag.is_empty() {
        return None;
    }

    let needle = repo.slug();
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

    Some(AdvisedRunner {
        source: ArchiveSource {
            name: repo.repo.clone(),
            kind: "proton".to_string(),
            api_url: repo.releases_api_url(),
            desc: String::new(),
            asset_priority: Vec::new(),
            require_asset_match: false,
        },
        tag,
        registered: false,
    })
}

pub fn delete_version(source: &ArchiveSource, tag: &str) -> Result<()> {
    let root = source_root(source);
    unlink_from_steam(&root.join(tag))?;
    archive_source::delete_version(source, &root, tag)
}

pub fn is_proton_dir(path: &Path) -> bool {
    path.join("proton").exists()
}

pub fn steam_proton_dirs_by_name(names: &[String]) -> BTreeMap<String, String> {
    let ours = runners_dir();
    let mut out = BTreeMap::new();
    for (dir_name, path) in crate::store::steam::local::iter_steam_protons() {
        if path
            .canonicalize()
            .unwrap_or_else(|_| path.clone())
            .starts_with(&ours)
        {
            continue;
        }
        let display = crate::store::steam::local::proton_display_name(&path);
        if let Some(name) = names
            .iter()
            .find(|n| **n == dir_name || Some(n.as_str()) == display.as_deref())
        {
            out.insert(name.clone(), dir_name);
        }
    }
    out
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

fn clear_compat_entry(dest: &Path) -> Result<()> {
    let Ok(meta) = dest.symlink_metadata() else {
        return Ok(());
    };
    if meta.is_symlink() {
        std::fs::remove_file(dest)?;
    } else {
        std::fs::remove_dir_all(dest)?;
    }
    Ok(())
}

pub fn steam_links(src: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = crate::store::steam::local::iter_compat_tools_dirs()
        .into_iter()
        .flat_map(|ctd| std::fs::read_dir(ctd).into_iter().flatten().flatten())
        .map(|e| e.path())
        .filter(|p| std::fs::read_link(p).is_ok_and(|t| t == src))
        .filter_map(|p| {
            let ctd = std::fs::canonicalize(p.parent()?).ok()?;
            Some(ctd.join(p.file_name()?))
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

pub fn steam_link_roots(src: &Path) -> Vec<PathBuf> {
    steam_links(src)
        .iter()
        .filter_map(|p| Some(p.parent()?.parent()?.to_path_buf()))
        .collect()
}

pub fn set_steam_links(src: &Path, roots: &[PathBuf]) -> Result<()> {
    let name = src
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("bad runner path: {}", src.display()))?;
    if !roots.is_empty() && !src.join("compatibilitytool.vdf").exists() {
        anyhow::bail!("{name} ships no compatibilitytool.vdf, Steam would not list it");
    }

    let wanted: Vec<PathBuf> = roots
        .iter()
        .map(|r| r.join("compatibilitytools.d").join(name))
        .collect();
    for stale in steam_links(src).iter().filter(|p| !wanted.contains(p)) {
        clear_compat_entry(stale)?;
    }
    for dest in &wanted {
        if let Some(ctd) = dest.parent() {
            std::fs::create_dir_all(ctd)?;
        }
        clear_compat_entry(dest)?;
        std::os::unix::fs::symlink(src, dest)?;
    }

    if wanted.is_empty() {
        Ok(())
    } else {
        stamp_steam_identity(src)
    }
}

pub fn unlink_from_steam(src: &Path) -> Result<()> {
    set_steam_links(src, &[])
}

fn is_runner_dir(path: &Path) -> bool {
    path.join("bin/wine").exists() || path.join("files/bin/wine64").exists() || is_proton_dir(path)
}

pub fn delete_found_runner(path: &Path) -> Result<()> {
    if !is_runner_dir(path) {
        anyhow::bail!("not a runner directory: {}", path.display());
    }
    unlink_from_steam(path)?;
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

pub fn steam_runners_ignored() -> bool {
    crate::app_settings::AppSettings::load()
        .behavior
        .ignore_steam_runners
}

pub fn runner_dir(version: &str) -> Option<PathBuf> {
    if version.is_empty() || version == "system" {
        return None;
    }
    match version.strip_prefix("steam:") {
        Some(_) if steam_runners_ignored() => None,
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

#[derive(Clone, Copy, PartialEq)]
pub enum SteamAction {
    None,
    Move,
    Link,
}

impl SteamAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Move => "move",
            Self::Link => "link",
        }
    }
}

impl Serialize for SteamAction {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

pub fn steam_action(path: &Path) -> SteamAction {
    if !path.starts_with(runners_dir()) || !path.join("compatibilitytool.vdf").exists() {
        return SteamAction::None;
    }
    match path.file_name().and_then(|n| n.to_str()) {
        Some(name) if name.ends_with(LATEST_SUFFIX) => SteamAction::Link,
        _ => SteamAction::Move,
    }
}

#[derive(Serialize)]
pub struct FoundRunner {
    pub name: String,
    pub kind: String,
    pub origin: String,
    pub path: String,
    pub steam_action: SteamAction,
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
                steam_action: steam_action(path),
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
            runners.push((
                name.to_string(),
                String::new(),
                runner_kind(&path).to_string(),
            ));
        }
    }

    for (name, path) in crate::store::steam::local::iter_steam_protons() {
        let label = crate::store::steam::local::proton_display_name(&path).unwrap_or_default();
        runners.push((format!("steam:{name}"), label, "proton".to_string()));
    }

    for (name, path) in system_wine_paths() {
        let label = wine_version(&path).unwrap_or_default();
        runners.push((format!("system:{name}"), label, "wine".to_string()));
    }

    if let Ok(path) = which::which("wine") {
        let label = wine_version(&path).unwrap_or_default();
        runners.push(("system".to_string(), label, "wine".to_string()));
    }

    runners.sort();
    runners.dedup();
    runners
}

static WINE_VERSIONS: LazyLock<Mutex<HashMap<PathBuf, Option<String>>>> =
    LazyLock::new(Default::default);

pub fn wine_version(exe: &Path) -> Option<String> {
    if let Ok(cache) = WINE_VERSIONS.lock()
        && let Some(hit) = cache.get(exe)
    {
        return hit.clone();
    }
    let probed = probe_wine_version(exe);
    if let Ok(mut cache) = WINE_VERSIONS.lock() {
        cache.insert(exe.to_path_buf(), probed.clone());
    }
    probed
}

fn probe_wine_version(exe: &Path) -> Option<String> {
    let out = Command::new(exe).arg("--version").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let version = text.trim();
    if version.is_empty() {
        return None;
    }
    Some(version.strip_prefix("wine-").unwrap_or(version).to_string())
}

pub fn display_name(version: &str) -> String {
    let named = version.strip_prefix("system:");
    if !(version.is_empty() || version == "system" || named.is_some()) {
        return version.to_string();
    }
    let exe = match named {
        Some(name) => system_wine_paths().get(name).cloned(),
        None => which::which("wine").ok(),
    };
    let probed = exe.as_deref().and_then(wine_version);
    let base = named.unwrap_or("System Wine");
    match (probed, named.is_some()) {
        (Some(v), true) => format!("{base} {v} (System)"),
        (Some(v), false) => format!("{base} {v}"),
        (None, true) => format!("{base} (System)"),
        (None, false) => base.to_string(),
    }
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

    let primary = which::which("wine")
        .ok()
        .and_then(|p| std::fs::canonicalize(p).ok());
    for dir in std::env::var_os("PATH")
        .iter()
        .flat_map(std::env::split_paths)
    {
        let bin = dir.join("wine");
        if !bin.is_file() {
            continue;
        }
        let Ok(real) = std::fs::canonicalize(&bin) else {
            continue;
        };
        if primary.as_ref() == Some(&real) {
            continue;
        }
        if paths
            .values()
            .any(|p| std::fs::canonicalize(p).ok().as_ref() == Some(&real))
        {
            continue;
        }
        if let Some(name) = wine_root_name(&dir) {
            paths.entry(name).or_insert(bin);
        }
    }

    paths
}

fn wine_root_name(dir: &Path) -> Option<String> {
    let root = match dir.file_name() {
        Some(name) if name == "bin" => dir.parent()?,
        _ => dir,
    };
    Some(root.file_name()?.to_str()?.to_string())
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
