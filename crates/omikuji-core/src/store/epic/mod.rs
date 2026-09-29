pub mod eos_overlay;
pub mod source;
pub mod updates;

use crate::components::{self, SettingsKey};
use crate::fs_util::find_executable_in_paths;
use crate::launch::{ComponentMissing, build_launch};
use crate::library::Game;
use crate::store::{self, StoreGame};
use crate::{downloads, fs_util, http};
use anyhow::{Result, anyhow};
use serde::Deserialize;
use serde::de::IgnoredAny;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tokio::process::Command as AsyncCommand;

const STORE: &str = "epic";

pub fn find_legendary() -> Option<PathBuf> {
    components::path_for(SettingsKey::Legendary)
}

pub fn legendary_system_path() -> Option<PathBuf> {
    find_executable_in_paths(
        &["legendary"],
        &[
            "~/.local/bin/legendary",
            "/usr/local/bin/legendary",
            "/usr/bin/legendary",
            "~/.local/share/pipx/venvs/legendary-gl/bin/legendary",
        ],
    )
}

pub fn require_legendary() -> Result<PathBuf> {
    find_legendary().ok_or_else(|| {
        anyhow::Error::new(ComponentMissing {
            name: "Legendary".to_string(),
        })
    })
}

fn legendary_command() -> Result<Command> {
    Ok(Command::new(require_legendary()?))
}

fn legendary_async() -> Result<AsyncCommand> {
    Ok(AsyncCommand::from(legendary_command()?))
}

fn check_output(cmd: &Command, output: Output) -> Result<Output> {
    if output.status.success() {
        return Ok(output);
    }
    let subcommand = cmd
        .get_args()
        .map(|a| a.to_string_lossy())
        .find(|a| !a.starts_with('-'))
        .unwrap_or_default();
    let err = String::from_utf8_lossy(&output.stderr);
    anyhow::bail!("legendary {} failed: {}", subcommand, err.trim())
}

fn legendary_output(cmd: &mut Command) -> Result<Output> {
    let output = cmd.output()?;
    check_output(cmd, output)
}

async fn legendary_output_async(cmd: &mut AsyncCommand) -> Result<Output> {
    let output = cmd.output().await?;
    check_output(cmd.as_std(), output)
}

pub fn legendary_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("legendary")
}

fn user_json() -> PathBuf {
    legendary_dir().join("user.json")
}

fn installed_json() -> PathBuf {
    legendary_dir().join("installed.json")
}

fn metadata_json(app_name: &str) -> PathBuf {
    legendary_dir()
        .join("metadata")
        .join(format!("{app_name}.json"))
}

fn resume_file(app_name: &str) -> PathBuf {
    legendary_dir()
        .join("tmp")
        .join(format!("{app_name}.resume"))
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
        logged_in()
    }

    pub fn get_login_url() -> String {
        "https://legendary.gl/epiclogin".to_string()
    }

    pub async fn login(&mut self, code: &str) -> Result<String> {
        legendary_output_async(legendary_async()?.args(["auth", "--code", code.trim()])).await?;

        self.refresh_display_name();
        if self.display_name.is_empty() {
            self.display_name = "Epic User".to_string();
        }
        Ok(self.display_name.clone())
    }

    pub async fn logout(&mut self) -> Result<()> {
        if let Ok(mut cmd) = legendary_async() {
            let _ = cmd.args(["auth", "--delete"]).output().await;
        }
        let _ = std::fs::remove_file(user_json());
        // drop cache so next login starts with an empty library, not the previous user's
        let _ = std::fs::remove_file(store::cache::library_path(STORE));
        self.display_name.clear();
        Ok(())
    }

    pub async fn list_games(&mut self) -> Result<Vec<StoreGame>> {
        tracing::info!("fetching library via legendary list --json ...");
        let output = legendary_output_async(legendary_async()?.args(["list", "--json"])).await?;

        let raw: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let arr = raw
            .as_array()
            .ok_or_else(|| anyhow!("expected array from legendary list"))?;

        let installed = list_installed_map();

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

            games.push(StoreGame {
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

pub fn logged_in() -> bool {
    user_json().exists()
}

fn read_display_name() -> Option<String> {
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(user_json()).ok()?).ok()?;
    v.get("displayName")?.as_str().map(String::from)
}

// only entries with BOTH install_path AND executable; partial installs (killed mid-download) would otherwise show up as "installed" in the ui
fn list_installed_map() -> HashMap<String, PathBuf> {
    store::registry::read(&installed_json())
        .into_iter()
        .filter(|(_, e)| e.has_executable())
        .map(|(app_name, e)| (app_name, e.install_path))
        .collect()
}

pub use crate::store::registry::InstalledInfo;

#[derive(Deserialize)]
struct InstalledRecord {
    version: String,
    platform: String,
    #[serde(default)]
    save_path: Option<String>,
}

fn installed_record(app_name: &str) -> Option<InstalledRecord> {
    store::registry::read_as(&installed_json()).remove(app_name)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EpicDlc {
    pub id: String,
    pub title: String,
    pub image: String,
}

fn read_metadata(app_name: &str) -> Result<serde_json::Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(
        metadata_json(app_name),
    )?)?)
}

fn dlc_items(meta: &serde_json::Value) -> &[serde_json::Value] {
    meta.pointer("/metadata/dlcItemList")
        .and_then(|d| d.as_array())
        .map_or(&[], Vec::as_slice)
}

fn dlc_id(item: &serde_json::Value) -> Option<&str> {
    item.pointer("/releaseInfo/0/appId")?.as_str()
}

fn dlc_image(item: &serde_json::Value) -> Option<&str> {
    let images = item.get("keyImages")?.as_array()?;
    let pick = |want: &str| {
        images
            .iter()
            .find(|i| i.get("type").and_then(|t| t.as_str()) == Some(want))
            .and_then(|i| i.get("url"))
            .and_then(|u| u.as_str())
    };
    pick("DieselGameBox")
        .or_else(|| pick("OfferImageWide"))
        .or_else(|| pick("DieselGameBoxTall"))
        .or_else(|| pick("OfferImageTall"))
}

fn dlc_art(meta: &serde_json::Value) -> HashMap<String, String> {
    dlc_items(meta)
        .iter()
        .filter_map(|e| Some((dlc_id(e)?.to_string(), dlc_image(e)?.to_string())))
        .collect()
}

#[derive(Debug, Clone)]
pub struct InstallSize {
    pub download_bytes: u64,
    pub install_bytes: u64,
    pub launch_exe: String,
    pub dlcs: Vec<EpicDlc>,
}

fn extract_dlcs(v: &serde_json::Value, art: &HashMap<String, String>) -> Vec<EpicDlc> {
    let Some(list) = v.pointer("/game/owned_dlc").and_then(|d| d.as_array()) else {
        return Vec::new();
    };
    list.iter()
        .filter(|d| {
            d.get("installable")
                .and_then(|i| i.as_array())
                .is_some_and(|a| !a.is_empty())
        })
        .filter_map(|d| {
            let id = d.get("app_name")?.as_str()?.to_string();
            let title = d
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or_default()
                .to_string();
            let image = art.get(&id).cloned().unwrap_or_default();
            Some(EpicDlc { id, title, image })
        })
        .collect()
}

pub async fn fetch_install_size(app_name: &str) -> Result<InstallSize> {
    let output = legendary_output_async(legendary_async()?.args([
        "info",
        app_name,
        "--json",
        "--platform",
        "Windows",
    ]))
    .await?;
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

    let launch_exe = v
        .pointer("/manifest/launch_exe")
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_string();

    Ok(InstallSize {
        download_bytes,
        install_bytes,
        launch_exe,
        dlcs: extract_dlcs(
            &v,
            &read_metadata(app_name)
                .map(|m| dlc_art(&m))
                .unwrap_or_default(),
        ),
    })
}

pub fn inspect_existing_install(app_name: &str, install_path: &Path) -> (u64, bool) {
    if !install_path.exists() {
        return (0, false);
    }
    (
        fs_util::dir_size(install_path),
        resume_file(app_name).exists(),
    )
}

pub fn installed_dlcs(app_name: &str) -> Vec<EpicDlc> {
    let Ok(meta) = read_metadata(app_name) else {
        return Vec::new();
    };
    let installed = store::registry::read_as::<IgnoredAny>(&installed_json());
    dlc_items(&meta)
        .iter()
        .filter_map(|e| {
            let id = dlc_id(e)?;
            installed.get(id)?;
            let title = e.get("title").and_then(|t| t.as_str()).unwrap_or(id);
            Some(EpicDlc {
                id: id.to_string(),
                title: title.to_string(),
                image: dlc_image(e).unwrap_or_default().to_string(),
            })
        })
        .collect()
}

pub fn find_installed_info(app_name: &str) -> Option<InstalledInfo> {
    let entry = store::registry::entry(&installed_json(), app_name)?;
    let exe_rel = Some(entry.executable.clone());
    Some(entry.resolved(exe_rel))
}

pub fn uninstall(app_name: &str) -> Result<()> {
    let install_path = find_installed_info(app_name).map(|i| i.install_path);
    legendary_output(legendary_command()?.args(["-y", "uninstall", app_name]))?;
    if let Some(path) = install_path
        && path.exists()
    {
        tracing::warn!(
            "legendary exited 0 but {} still exists, forcing cleanup",
            path.display()
        );
        downloads::cleanup_install_dir_blocking(&path);
    }
    Ok(())
}

// not uninstall(): a dlc's install_path is the base game's dir, its leftover cleanup would wipe the game
pub fn uninstall_dlc(dlc_id: &str) -> Result<()> {
    legendary_output(legendary_command()?.args(["-y", "uninstall", dlc_id]))?;
    Ok(())
}

pub fn discover_save_path(game: &Game) -> Result<String> {
    let app_name = game.effective_app_id();
    let config = build_launch(game)?;

    tracing::info!("discovering save path for '{}'", app_name);

    let status = legendary_command()?
        .args([
            "sync-saves",
            app_name,
            "--skip-upload",
            "--skip-download",
            "--accept-path",
        ])
        .envs(&config.env)
        .status()?;

    if !status.success() {
        tracing::warn!("sync-saves path discovery exited with {}", status);
    }

    Ok(installed_record(app_name)
        .and_then(|r| r.save_path)
        .unwrap_or_default())
}

pub fn sync_saves_download(app_name: &str, save_path: &str) -> Result<()> {
    if save_path.is_empty() {
        return Ok(());
    }
    tracing::info!("downloading saves for '{}' to '{}'", app_name, save_path);
    legendary_output(legendary_command()?.args([
        "sync-saves",
        app_name,
        "--skip-upload",
        "--save-path",
        save_path,
        "-y",
    ]))?;
    Ok(())
}

pub fn sync_saves_upload(app_name: &str, save_path: &str) -> Result<()> {
    if save_path.is_empty() {
        return Ok(());
    }
    tracing::info!("uploading saves for '{}' from '{}'", app_name, save_path);
    legendary_output(legendary_command()?.args([
        "sync-saves",
        app_name,
        "--skip-download",
        "--save-path",
        save_path,
        "-y",
    ]))?;
    Ok(())
}

fn thumbnail_url(url: &str) -> String {
    let sep = if url.contains('?') { '&' } else { '?' };
    format!("{}{}h=480&w=360&resize=1&quality=medium", url, sep)
}

pub fn load_cached_library() -> Vec<StoreGame> {
    store::cache::load_library(STORE)
}

pub fn save_cached_library(games: &[StoreGame]) {
    store::cache::save_library(STORE, games);
}

fn resolve_epic_image(app_name: &str, kind: &str, cdn_url: Option<&str>) -> Option<String> {
    store::cache::resolve_image(STORE, app_name, kind, cdn_url, thumbnail_url)
}

pub async fn fetch_game_details(app_name: &str) -> Result<String> {
    let meta = read_metadata(app_name)?;
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
        let client = http::client();
        let slug = product_slug(client, &namespace, &title).await;
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
    // i just wanted to say that seeing this again made me angry
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
