pub mod spec;
pub mod specs;

pub use spec::{ComponentSpec, ComponentStatus, ExtractStrategy, SettingsKey, Source};

use anyhow::{Result, anyhow};
use std::collections::VecDeque;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

fn version_marker(name: &str) -> PathBuf {
    crate::runtime_dir().join(format!("{}.version", name))
}

fn read_version(name: &str) -> Option<String> {
    fs::read_to_string(version_marker(name))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn write_version(name: &str, tag: &str) -> std::io::Result<()> {
    let path = version_marker(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, tag)
}

pub fn status_for(spec: &ComponentSpec) -> ComponentStatus {
    let canary = crate::runtime_dir().join(spec.dest);
    match (canary.exists(), read_version(spec.name)) {
        (true, Some(ver)) => ComponentStatus::Installed {
            version: ver,
            path: canary,
        },
        _ => match spec.system_probe.and_then(|probe| probe()) {
            Some(path) => ComponentStatus::System { path },
            None => ComponentStatus::Missing,
        },
    }
}

pub fn check_all() -> Vec<&'static ComponentSpec> {
    specs::all()
        .iter()
        .filter(|s| matches!(status_for(s), ComponentStatus::Missing))
        .collect()
}

pub fn umu() -> Vec<&'static ComponentSpec> {
    specs::all()
        .iter()
        .filter(|s| matches!(s.settings_key, SettingsKey::UmuRun))
        .collect()
}

pub fn epic_tools() -> Vec<&'static ComponentSpec> {
    specs::all()
        .iter()
        .filter(|s| {
            matches!(
                s.settings_key,
                SettingsKey::Legendary | SettingsKey::EglDummy
            )
        })
        .collect()
}

pub fn gog_tools() -> Vec<&'static ComponentSpec> {
    specs::all()
        .iter()
        .filter(|s| matches!(s.settings_key, SettingsKey::Gogdl))
        .collect()
}

pub fn gacha_tools(publisher_slug: &str) -> Vec<&'static ComponentSpec> {
    let needs_hpatchz = matches!(publisher_slug, "hoyoverse" | "hypergryph");
    specs::all()
        .iter()
        .filter(|s| match s.settings_key {
            SettingsKey::Hpatchz => needs_hpatchz,
            _ => false,
        })
        .collect()
}

pub fn updatable() -> Vec<&'static ComponentSpec> {
    specs::all()
        .iter()
        .filter(|s| matches!(s.source, Source::GithubRelease { .. }))
        .collect()
}

pub fn ready(specs: &[&'static ComponentSpec]) -> bool {
    specs
        .iter()
        .all(|s| !matches!(status_for(s), ComponentStatus::Missing))
}

pub fn remove(spec: &ComponentSpec) -> Result<()> {
    for path in [
        crate::runtime_dir().join(spec.dest),
        version_marker(spec.name),
    ] {
        if path.exists() {
            fs::remove_file(&path).map_err(|e| anyhow!("remove {}: {}", path.display(), e))?;
        }
    }
    Ok(())
}

pub async fn ensure(specs: &[&'static ComponentSpec]) -> Result<()> {
    for &spec in specs {
        if matches!(status_for(spec), ComponentStatus::Missing) {
            install_one(spec).await?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub enum ComponentEvent {
    Started {
        name: String,
    },
    Progress {
        name: String,
        phase: String,
        percent: f64,
    },
    Completed {
        name: String,
        version: String,
    },
    Failed {
        name: String,
        error: String,
    },
    UpdateChecked {
        name: String,
        latest: String,
    },
}

static EVENTS: OnceLock<Mutex<VecDeque<ComponentEvent>>> = OnceLock::new();

fn queue() -> &'static Mutex<VecDeque<ComponentEvent>> {
    EVENTS.get_or_init(|| Mutex::new(VecDeque::new()))
}

pub fn drain_events() -> Vec<ComponentEvent> {
    queue().lock().unwrap().drain(..).collect()
}

fn push(ev: ComponentEvent) {
    queue().lock().unwrap().push_back(ev);
}

pub fn push_fail_event(name: &str, error: &str) {
    push(ComponentEvent::Failed {
        name: name.to_string(),
        error: error.to_string(),
    });
}

#[derive(Debug, serde::Deserialize)]
struct GhRelease {
    tag_name: String,
    assets: Vec<GhAsset>,
}

#[derive(Debug, serde::Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
}

async fn fetch_latest_release(api_url: &str) -> Result<GhRelease> {
    let resp = crate::http::client()
        .get(api_url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()?;
    Ok(resp.json().await?)
}

pub async fn install_one(spec: &'static ComponentSpec) -> Result<()> {
    push(ComponentEvent::Started {
        name: spec.name.to_string(),
    });

    match install_one_inner(spec).await {
        Ok(tag) => {
            push(ComponentEvent::Completed {
                name: spec.name.to_string(),
                version: tag,
            });
            Ok(())
        }
        Err(e) => {
            let msg = format!("{:#}", e);
            push(ComponentEvent::Failed {
                name: spec.name.to_string(),
                error: msg.clone(),
            });
            Err(anyhow!(msg))
        }
    }
}

pub async fn check_update(spec: &'static ComponentSpec) {
    let latest = match url_for(spec.settings_key) {
        Ok(url) => match fetch_latest_release(&url).await {
            Ok(release) => release.tag_name,
            Err(e) => {
                tracing::warn!("{} update check ({}): {}", spec.name, url, e);
                String::new()
            }
        },
        Err(e) => {
            tracing::warn!("{} update check: {}", spec.name, e);
            String::new()
        }
    };
    push(ComponentEvent::UpdateChecked {
        name: spec.name.to_string(),
        latest,
    });
}

fn url_for(key: SettingsKey) -> Result<String> {
    let s = &crate::settings::get().components;
    let value = match key {
        SettingsKey::UmuRun => &s.umu_run,
        SettingsKey::Hpatchz => &s.hpatchz,
        SettingsKey::Legendary => &s.legendary,
        SettingsKey::Gogdl => &s.gogdl,
        SettingsKey::EglDummy => &s.egl_dummy,
    };
    if value.trim().is_empty() {
        return Err(anyhow!(
            "component URL is empty in settings.toml — check [components]"
        ));
    }
    Ok(value.clone())
}

async fn install_one_inner(spec: &ComponentSpec) -> Result<String> {
    push(ComponentEvent::Progress {
        name: spec.name.to_string(),
        phase: "resolving".into(),
        percent: 0.0,
    });

    let settings_url = url_for(spec.settings_key)?;

    let (url, display_name, tag): (String, String, String) = match &spec.source {
        Source::GithubRelease { asset_matcher } => {
            let release = fetch_latest_release(&settings_url)
                .await
                .map_err(|e| anyhow!("release api ({}): {}", settings_url, e))?;

            tracing::debug!(
                "{} latest release {} has assets:\n  - {}",
                spec.name,
                release.tag_name,
                release
                    .assets
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join("\n  - ")
            );

            let asset = release
                .assets
                .iter()
                .find(|a| asset_matcher(&a.name))
                .ok_or_else(|| {
                    anyhow!(
                        "no asset in release {} matched the filter (found: {})",
                        release.tag_name,
                        release
                            .assets
                            .iter()
                            .map(|a| a.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })?;

            (
                asset.browser_download_url.clone(),
                asset.name.clone(),
                release.tag_name.clone(),
            )
        }
        Source::DirectUrl { marker } => (
            settings_url.clone(),
            settings_url
                .rsplit('/')
                .next()
                .unwrap_or("download")
                .to_string(),
            marker.to_string(),
        ),
    };

    let bytes = download_bytes(&url, spec.name)
        .await
        .map_err(|e| anyhow!("download {}: {}", display_name, e))?;

    push(ComponentEvent::Progress {
        name: spec.name.to_string(),
        phase: "extracting".into(),
        percent: 0.0,
    });

    install_bytes(spec, &bytes).map_err(|e| anyhow!("extract/install: {}", e))?;

    write_version(spec.name, &tag).map_err(|e| anyhow!("write version marker: {}", e))?;

    Ok(tag)
}

async fn download_bytes(url: &str, name: &str) -> Result<Vec<u8>> {
    crate::http::download_with_progress(url, 0, |pct| {
        push(ComponentEvent::Progress {
            name: name.to_string(),
            phase: "downloading".into(),
            percent: pct,
        });
    })
    .await
}

fn install_bytes(spec: &ComponentSpec, bytes: &[u8]) -> Result<()> {
    let runtime = crate::runtime_dir();
    fs::create_dir_all(&runtime)?;
    let dest = runtime.join(spec.dest);
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    match &spec.extract {
        ExtractStrategy::Raw => {
            let tmp = dest.with_extension("dl-tmp");
            fs::write(&tmp, bytes)?;
            chmod_exec(&tmp)?;
            fs::rename(&tmp, &dest)?;
        }
        ExtractStrategy::Tar { inner_path } => {
            promote_from_tar(spec, bytes, None, inner_path, &dest)?;
        }
        ExtractStrategy::TarGz { inner_path } => {
            promote_from_tar(spec, bytes, Some(()), inner_path, &dest)?;
        }
        ExtractStrategy::Zip { inner_path } => {
            let reader = std::io::Cursor::new(bytes);
            let mut archive = zip::ZipArchive::new(reader)?;

            let idx = (0..archive.len())
                .find(|&i| {
                    let Ok(e) = archive.by_index(i) else {
                        return false;
                    };
                    let name = e.name();
                    name == *inner_path
                        || name.ends_with(&format!("/{}", inner_path))
                        || name.rsplit('/').next() == Some(*inner_path)
                })
                .ok_or_else(|| anyhow!("{} not found in zip archive", inner_path))?;

            let mut zfile = archive.by_index(idx)?;
            let tmp = dest.with_extension("dl-tmp");
            let mut out = fs::File::create(&tmp)?;
            std::io::copy(&mut zfile, &mut out)?;
            out.flush()?;
            drop(out);
            chmod_exec(&tmp)?;
            fs::rename(&tmp, &dest)?;
        }
    }
    Ok(())
}

fn promote_from_tar(
    spec: &ComponentSpec,
    bytes: &[u8],
    gzipped: Option<()>,
    inner_path: &str,
    dest: &Path,
) -> Result<()> {
    let runtime = crate::runtime_dir();
    let staging = runtime.join(format!(".staging-{}", spec.name));
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging)?;

    if gzipped.is_some() {
        let gz = flate2::read::GzDecoder::new(bytes);
        tar::Archive::new(gz).unpack(&staging)?;
    } else {
        tar::Archive::new(std::io::Cursor::new(bytes)).unpack(&staging)?;
    }

    let src = find_by_filename(&staging, inner_path).ok_or_else(|| {
        anyhow!(
            "{} not found after extracting tarball. contents:\n  - {}",
            inner_path,
            list_tree(&staging)
        )
    })?;
    let tmp = dest.with_extension("dl-tmp");
    fs::copy(&src, &tmp)?;
    chmod_exec(&tmp)?;
    fs::rename(&tmp, dest)?;
    let _ = fs::remove_dir_all(&staging);
    Ok(())
}

fn chmod_exec(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)
}

fn find_by_filename(root: &Path, target: &str) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.file_name().and_then(|n| n.to_str()) == Some(target) {
                return Some(p);
            }
        }
    }
    None
}

fn list_tree(root: &Path) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p.clone());
            }
            if let Ok(rel) = p.strip_prefix(root) {
                out.push(rel.display().to_string());
            }
        }
    }
    out.sort();
    out.join("\n  - ")
}
