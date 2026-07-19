pub mod eos_overlay;
pub mod updates;

use crate::downloads::legendary::require_legendary;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EpicGame {
    pub app_name: String,
    pub title: String,
    pub banner: Option<String>,
    pub coverart: Option<String>,
    pub icon: Option<String>,
    pub is_installed: bool,
    pub install_path: Option<PathBuf>,
}

pub struct EpicStore {
    pub display_name: String,
}

impl Default for EpicStore {
    fn default() -> Self {
        Self::new()
    }
}

impl EpicStore {
    pub fn new() -> Self {
        Self {
            display_name: read_display_name().unwrap_or_default(),
        }
    }

    pub fn refresh_display_name(&mut self) {
        self.display_name = read_display_name().unwrap_or_default();
    }

    pub fn is_logged_in(&self) -> bool {
        legendary_user_json().map(|p| p.exists()).unwrap_or(false)
    }

    pub fn get_login_url() -> String {
        "https://legendary.gl/epiclogin".to_string()
    }

    pub async fn login(&mut self, code: &str) -> Result<String> {
        let bin = require_legendary()?;
        let output = AsyncCommand::new(&bin)
            .arg("auth")
            .arg("--code")
            .arg(code.trim())
            .output()
            .await?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("legendary auth failed: {}", err.trim());
        }

        self.refresh_display_name();
        if self.display_name.is_empty() {
            self.display_name = "Epic User".to_string();
        }
        Ok(self.display_name.clone())
    }

    // legendary refreshes its own tokens on every list/install/info call ,we dont need to do anything here
    pub async fn try_refresh(&mut self) -> bool {
        self.refresh_display_name();
        self.is_logged_in()
    }

    pub async fn logout(&mut self) -> Result<()> {
        if let Ok(bin) = require_legendary() {
            let _ = AsyncCommand::new(&bin)
                .arg("auth")
                .arg("--delete")
                .output()
                .await;
        }
        if let Some(path) = legendary_user_json()
            && path.exists()
        {
            let _ = std::fs::remove_file(&path);
        }
        // drop cache so next login starts with an empty library, not the previous user's
        let _ = std::fs::remove_file(cached_library_path());
        self.display_name.clear();
        Ok(())
    }

    pub async fn list_games(&mut self) -> Result<Vec<EpicGame>> {
        migrate_image_cache_once();
        let bin = require_legendary()?;
        tracing::info!("fetching library via legendary list --json ...");
        let output = AsyncCommand::new(&bin)
            .arg("list")
            .arg("--json")
            .output()
            .await?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("legendary list failed: {}", err.trim());
        }

        let raw: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let arr = raw
            .as_array()
            .ok_or_else(|| anyhow!("expected array from legendary list"))?;

        let installed = list_installed_map().unwrap_or_default();

        let mut games = Vec::new();
        for entry in arr {
            if let Some(cats) = entry
                .pointer("/metadata/categories")
                .and_then(|c| c.as_array())
                && cats.iter().any(|c| {
                    c.get("path")
                        .and_then(|p| p.as_str())
                        .map(|p| p == "assets" || p == "plugins")
                        .unwrap_or(false)
                })
            {
                continue;
            }

            let app_name = entry
                .get("app_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if app_name.is_empty() {
                continue;
            }
            let title = entry
                .get("app_title")
                .and_then(|v| v.as_str())
                .or_else(|| entry.pointer("/metadata/title").and_then(|v| v.as_str()))
                .unwrap_or(&app_name)
                .to_string();

            let mut banner = None;
            let mut coverart = None;
            let mut icon = None;
            if let Some(images) = entry
                .pointer("/metadata/keyImages")
                .and_then(|v| v.as_array())
            {
                for img in images {
                    let url = img.get("url").and_then(|v| v.as_str()).unwrap_or("");
                    let ty = img.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    match ty {
                        "DieselGameBox" | "OfferImageWide" => banner = Some(url.to_string()),
                        "DieselGameBoxTall" | "OfferImageTall" | "DieselStoreFrontTall" => {
                            coverart = Some(url.to_string())
                        }
                        "DieselGameBoxLogo" => icon = Some(url.to_string()),
                        _ => {}
                    }
                }
            }

            let banner = resolve_epic_image(&app_name, "banner", banner.as_deref());
            let coverart = resolve_epic_image(&app_name, "coverart", coverart.as_deref());
            let icon = resolve_epic_image(&app_name, "icon", icon.as_deref());

            let legendary_path = installed.get(&app_name).cloned();
            let really_installed = legendary_path.as_ref().map(|p| p.exists()).unwrap_or(false);

            games.push(EpicGame {
                app_name: app_name.clone(),
                title,
                banner,
                coverart,
                icon,
                is_installed: really_installed,
                install_path: if really_installed {
                    legendary_path
                } else {
                    None
                },
            });
        }

        games.sort_by_key(|a| a.title.to_lowercase());
        tracing::info!("got {} games from legendary", games.len());
        save_cached_library(&games);
        Ok(games)
    }
}

fn legendary_user_json() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("legendary").join("user.json"))
}

fn read_display_name() -> Option<String> {
    let path = legendary_user_json()?;
    if !path.exists() {
        return None;
    }
    let json = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&json).ok()?;
    v.get("displayName")
        .and_then(|n| n.as_str())
        .map(String::from)
}

// only include entreies with BOTH install_path AND executable; partial installs (killed mid-download) would otherwise show up as "installed" in the ui
fn list_installed_map() -> Result<HashMap<String, PathBuf>> {
    let path = dirs::config_dir()
        .unwrap_or_default()
        .join("legendary")
        .join("installed.json");
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = std::fs::read_to_string(path)?;
    let v: serde_json::Value = serde_json::from_str(&content)?;
    let mut map = HashMap::new();
    if let Some(obj) = v.as_object() {
        for (app_name, data) in obj {
            let has_exe = data
                .get("executable")
                .and_then(|p| p.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false);
            if !has_exe {
                continue;
            }
            if let Some(path) = data.get("install_path").and_then(|p| p.as_str()) {
                map.insert(app_name.clone(), PathBuf::from(path));
            }
        }
    }
    Ok(map)
}

#[derive(Debug, Clone)]
pub struct InstalledInfo {
    pub install_path: PathBuf,
    pub executable: PathBuf,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct InstallSize {
    pub download_bytes: u64,
    pub install_bytes: u64,
}

pub async fn fetch_install_size(app_name: &str) -> Result<InstallSize> {
    let bin = require_legendary()?;
    let output = AsyncCommand::new(&bin)
        .arg("info")
        .arg(app_name)
        .arg("--json")
        .arg("--platform")
        .arg("Windows")
        .output()
        .await?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("legendary info failed: {}", err.trim());
    }

    let v: serde_json::Value = serde_json::from_slice(&output.stdout)?;

    let install_bytes = v
        .pointer("/manifest/disk_size")
        .and_then(|x| x.as_u64())
        .unwrap_or(0);
    let download_bytes = v
        .pointer("/manifest/download_size")
        .and_then(|x| x.as_u64())
        .unwrap_or(0);

    if install_bytes == 0 && download_bytes == 0 {
        anyhow::bail!("legendary info returned no size fields");
    }

    Ok(InstallSize {
        download_bytes,
        install_bytes,
    })
}

pub fn inspect_existing_install(app_name: &str, install_path: &Path) -> (u64, bool) {
    if !install_path.exists() {
        return (0, false);
    }

    let has_resume = dirs::config_dir()
        .map(|c| {
            c.join("legendary")
                .join("tmp")
                .join(format!("{}.resume", app_name))
                .exists()
        })
        .unwrap_or(false);

    let bytes = std::process::Command::new("du")
        .args(["-sb"])
        .arg(install_path)
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(o.stdout)
            } else {
                None
            }
        })
        .and_then(|stdout| {
            let s = String::from_utf8_lossy(&stdout);
            s.split_whitespace()
                .next()
                .and_then(|n| n.parse::<u64>().ok())
        })
        .unwrap_or(0);

    (bytes, has_resume)
}

pub fn find_installed_info(app_name: &str) -> Option<InstalledInfo> {
    let installed_json = dirs::config_dir()?.join("legendary").join("installed.json");
    if !installed_json.exists() {
        return None;
    }
    let content = std::fs::read_to_string(installed_json).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    let entry = v.get(app_name)?;
    let install_path = PathBuf::from(entry.get("install_path")?.as_str()?);
    let exe_rel = entry
        .get("executable")
        .and_then(|e| e.as_str())
        .unwrap_or("");
    let executable = if exe_rel.is_empty() {
        PathBuf::new()
    } else {
        install_path.join(exe_rel)
    };
    let title = entry
        .get("title")
        .and_then(|t| t.as_str())
        .map(String::from);
    Some(InstalledInfo {
        install_path,
        executable,
        title,
    })
}

fn installed_save_path(app_name: &str) -> Option<String> {
    let installed_json = dirs::config_dir()?.join("legendary").join("installed.json");
    let content = std::fs::read_to_string(installed_json).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    v.get(app_name)?
        .get("save_path")
        .and_then(|p| p.as_str())
        .map(String::from)
}

pub fn discover_save_path(game: &crate::library::Game) -> Result<String> {
    let bin = require_legendary()?;
    let app_name = if game.source.app_id.is_empty() {
        &game.metadata.id
    } else {
        &game.source.app_id
    };

    let config = crate::launch::build_launch(game)?;

    tracing::info!("discovering save path for '{}'", app_name);

    let status = std::process::Command::new(&bin)
        .arg("sync-saves")
        .arg(app_name)
        .arg("--skip-upload")
        .arg("--skip-download")
        .arg("--accept-path")
        .envs(&config.env)
        .status()?;

    if !status.success() {
        tracing::warn!("sync-saves path discovery exited with {}", status);
    }

    Ok(installed_save_path(app_name).unwrap_or_default())
}

pub fn sync_saves_download(app_name: &str, save_path: &str) -> Result<()> {
    if save_path.is_empty() {
        return Ok(());
    }
    let bin = require_legendary()?;
    tracing::info!("downloading saves for '{}' to '{}'", app_name, save_path);

    let status = std::process::Command::new(&bin)
        .arg("sync-saves")
        .arg(app_name)
        .arg("--skip-upload")
        .arg("--save-path")
        .arg(save_path)
        .arg("-y")
        .status()?;

    if !status.success() {
        anyhow::bail!("sync-saves download failed with {}", status);
    }
    Ok(())
}

pub fn sync_saves_upload(app_name: &str, save_path: &str) -> Result<()> {
    if save_path.is_empty() {
        return Ok(());
    }
    let bin = require_legendary()?;
    tracing::info!("uploading saves for '{}' from '{}'", app_name, save_path);

    let status = std::process::Command::new(&bin)
        .arg("sync-saves")
        .arg(app_name)
        .arg("--skip-download")
        .arg("--save-path")
        .arg(save_path)
        .arg("-y")
        .status()?;

    if !status.success() {
        anyhow::bail!("sync-saves upload failed with {}", status);
    }
    Ok(())
}

fn epic_cache_dir() -> PathBuf {
    crate::cache_dir().join("epic")
}

fn cached_image_path(app_name: &str, kind: &str) -> PathBuf {
    epic_cache_dir().join(format!("{}_{}.img", app_name, kind))
}

fn cached_library_path() -> PathBuf {
    epic_cache_dir().join("library.json")
}

fn thumbnail_url(url: &str) -> String {
    let sep = if url.contains('?') { '&' } else { '?' };
    format!("{}{}h=480&w=360&resize=1&quality=medium", url, sep)
}

fn migrate_image_cache_once() {
    use std::sync::OnceLock;
    static MIGRATED: OnceLock<()> = OnceLock::new();
    MIGRATED.get_or_init(|| {
        let dir = epic_cache_dir();
        let marker = dir.join(".thumb-v1");
        if marker.exists() {
            return;
        }
        tracing::info!("migrating image cache to thumbnailed version");
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.extension().and_then(|s| s.to_str()) == Some("img") {
                    let _ = std::fs::remove_file(&p);
                }
            }
        }
        let _ = std::fs::remove_file(cached_library_path());
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(&marker, "v1");
    });
}

pub fn load_cached_library() -> Vec<EpicGame> {
    migrate_image_cache_once();
    let path = cached_library_path();
    let Ok(data) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    match serde_json::from_str::<Vec<EpicGame>>(&data) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("library cache parse failed: {}", e);
            Vec::new()
        }
    }
}

pub fn save_cached_library(games: &[EpicGame]) {
    let path = cached_library_path();
    let body = match serde_json::to_string(games) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("library cache serialize failed: {}", e);
            return;
        }
    };
    if let Err(e) = crate::fs_util::write_atomic(&path, body) {
        tracing::error!("library cache write failed: {}", e);
    }
}

fn resolve_epic_image(app_name: &str, kind: &str, cdn_url: Option<&str>) -> Option<String> {
    let url = cdn_url?;
    if url.is_empty() {
        return None;
    }
    crate::media::fetch_cached_image(
        &cached_image_path(app_name, kind),
        &thumbnail_url(url),
        format!("epic_{}_{}", app_name, kind),
    )
}

pub async fn fetch_game_details(app_name: &str) -> Result<String> {
    let path = dirs::config_dir()
        .ok_or_else(|| anyhow!("no config dir"))?
        .join("legendary")
        .join("metadata")
        .join(format!("{app_name}.json"));
    let meta: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    let title = meta
        .pointer("/metadata/title")
        .and_then(|t| t.as_str())
        .unwrap_or_default()
        .trim()
        .to_string();
    let namespace = meta
        .pointer("/metadata/namespace")
        .and_then(|n| n.as_str())
        .unwrap_or_default()
        .to_string();

    let mut description = String::new();
    let mut reqs = Vec::new();

    if !namespace.is_empty() {
        let client = reqwest::Client::new();
        let slug = product_slug(&client, &namespace, &title).await;
        if let Ok(resp) = client
            .get(format!(
                "https://store-content.ak.epicgames.com/api/en-US/content/products/{slug}"
            ))
            .send()
            .await
            && let Ok(v) = resp.json::<serde_json::Value>().await
            && let Some(home) = v.get("pages").and_then(|p| p.as_array()).and_then(|ps| {
                ps.iter()
                    .find(|p| p.get("type").and_then(|t| t.as_str()) == Some("productHome"))
            })
        {
            description = home
                .pointer("/data/about/description")
                .and_then(|d| d.as_str())
                .filter(|s| !s.trim().is_empty())
                .or_else(|| {
                    home.pointer("/data/about/shortDescription")
                        .and_then(|d| d.as_str())
                })
                .map(strip_markdown_headers)
                .unwrap_or_default();
            reqs = extract_store_reqs(home);
        }
    }

    if description.is_empty() {
        description = local_description(&meta, &title).unwrap_or_default();
    }
    if description.is_empty() && reqs.is_empty() {
        anyhow::bail!("no details available for {app_name}");
    }
    Ok(serde_json::json!({ "description": description, "reqs": reqs }).to_string())
}

async fn product_slug(client: &reqwest::Client, namespace: &str, title: &str) -> String {
    let query = serde_json::json!({
        "query": format!(
            "{{ Catalog {{ catalogNs(namespace: \"{namespace}\") {{ mappings (pageType: \"productHome\") {{ pageSlug pageType }} }} }} }}"
        )
    });
    if let Ok(resp) = client
        .post("https://launcher.store.epicgames.com/graphql")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) EpicGamesLauncher",
        )
        .json(&query)
        .send()
        .await
        && let Ok(v) = resp.json::<serde_json::Value>().await
        && let Some(slug) = v
            .pointer("/data/Catalog/catalogNs/mappings")
            .and_then(|m| m.as_array())
            .and_then(|ms| {
                ms.iter()
                    .find(|m| m.get("pageType").and_then(|t| t.as_str()) == Some("productHome"))
            })
            .and_then(|m| m.get("pageSlug"))
            .and_then(|s| s.as_str())
    {
        return slug.to_string();
    }
    slug_from_title(title)
}

fn slug_from_title(title: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in title.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    out.trim_end_matches('-').to_string()
}

fn strip_markdown_headers(s: &str) -> String {
    s.lines()
        .map(|l| l.trim_start_matches('#').trim_start())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn extract_store_reqs(home: &serde_json::Value) -> Vec<serde_json::Value> {
    let Some(systems) = home
        .pointer("/data/requirements/systems")
        .and_then(|s| s.as_array())
    else {
        return Vec::new();
    };
    let Some(system) = systems
        .iter()
        .find(|s| s.get("systemType").and_then(|t| t.as_str()) == Some("Windows"))
        .or_else(|| systems.first())
    else {
        return Vec::new();
    };
    system
        .get("details")
        .and_then(|d| d.as_array())
        .map(|ds| {
            ds.iter()
                .filter_map(|d| {
                    let t = d.get("title")?.as_str()?.trim();
                    let min = d
                        .get("minimum")
                        .and_then(|m| m.as_str())
                        .unwrap_or_default()
                        .trim();
                    let rec = d
                        .get("recommended")
                        .and_then(|m| m.as_str())
                        .unwrap_or_default()
                        .trim();
                    if t.is_empty() || min.is_empty() {
                        return None;
                    }
                    let rec = if rec == min { "" } else { rec };
                    Some(serde_json::json!({ "title": t, "minimum": min, "recommended": rec }))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn local_description(meta: &serde_json::Value, title: &str) -> Option<String> {
    [
        "/metadata/longDescription",
        "/metadata/description",
        "/metadata/shortDescription",
    ]
    .iter()
    .find_map(|p| {
        meta.pointer(p)
            .and_then(|d| d.as_str())
            .map(str::trim)
            .filter(|s| s.len() > 40 && *s != title)
    })
    .map(str::to_string)
}
