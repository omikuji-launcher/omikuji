// little note: FUCK YOU GOG. we love you really but what the fuck
pub mod source;
pub mod updates;

use crate::components::{self, SettingsKey};
use crate::store::{self, StoreGame};
use crate::{fs_util, http};
use anyhow::{Result, anyhow};
use futures_util::{StreamExt, future, stream};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tokio::process::Command as AsyncCommand;

const STORE: &str = "gog";
const METADATA_FETCHES: usize = 8;

pub struct GogStore {
    pub display_name: String,
    pub user_id: String,
}

impl Default for GogStore {
    fn default() -> Self {
        Self::new()
    }
}

impl GogStore {
    pub fn new() -> Self {
        let user = read_user_data().unwrap_or_default();
        Self {
            display_name: user.username,
            user_id: user.user_id,
        }
    }

    pub fn is_logged_in(&self) -> bool {
        gog_auth_path().exists()
    }

    // redirect lands on embed.gog.com/on_login_success?code=... and the user pastes that code back
    pub fn get_login_url() -> String {
        "https://auth.gog.com/auth?client_id=46899977096215655&redirect_uri=https%3A%2F%2Fembed.gog.com%2Fon_login_success%3Forigin%3Dclient&response_type=code&layout=galaxy".to_string()
    }

    pub async fn login(&mut self, code: &str) -> Result<String> {
        gogdl_output(&["auth", "--code", code.trim()]).await?;

        if let Err(e) = self.refresh_user_data().await {
            tracing::error!("refresh_user_data after login failed: {}", e);
        }
        if self.display_name.is_empty() {
            self.display_name = "GOG User".to_string();
        }
        Ok(self.display_name.clone())
    }

    // user_id comes from the gogdl token, never userData.json: that ones the gog.com account id, not the galaxy id the token binds to, and galaxy-library 403s "Wrong user" on a mismatch (ahhaahahah?!!??!!?)
    pub async fn refresh_user_data(&mut self) -> Result<()> {
        if !self.is_logged_in() {
            tracing::warn!("refresh_user_data: not logged in (no auth file)");
            return Ok(());
        }
        let creds = read_credentials().await?;
        if let Some(id) = &creds.user_id {
            self.user_id = id.clone();
        }

        let resp = http::client()
            .get("https://embed.gog.com/userData.json")
            .bearer_auth(&creds.access_token)
            .send()
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            tracing::error!(
                "userData.json returned {} - body (first 300 chars): {}",
                status,
                body.chars().take(300).collect::<String>()
            );
            anyhow::bail!("userData.json returned {}", status);
        }
        let v: serde_json::Value = resp.json().await?;
        let name = v
            .get("username")
            .and_then(|n| n.as_str())
            .or_else(|| v.pointer("/userName").and_then(|n| n.as_str()))
            .unwrap_or("")
            .to_string();
        let userdata_id = v
            .get("userId")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        if !name.is_empty() {
            self.display_name = name.clone();
        }
        // deliberately do NOT overwrite self.user_id with userdata_id
        if !userdata_id.is_empty() && userdata_id != self.user_id {
            tracing::warn!(
                "userData.json userId={} differs from token user_id={} - using token id for galaxy-library",
                userdata_id,
                self.user_id
            );
        }
        tracing::info!(
            "refresh_user_data ok - username='{}' token_user_id='{}'",
            self.display_name,
            self.user_id
        );
        save_user_data(&UserData {
            username: self.display_name.clone(),
            user_id: self.user_id.clone(),
        });
        Ok(())
    }

    pub async fn list_games(&mut self) -> Result<Vec<StoreGame>> {
        if !self.is_logged_in() {
            return Ok(Vec::new());
        }
        if let Err(e) = self.refresh_user_data().await {
            tracing::error!("list_games: refresh_user_data failed: {}", e);
        }
        if self.user_id.is_empty() {
            anyhow::bail!("couldn't get the GOG user id, try logging in again");
        }

        let creds = read_credentials().await?;
        tracing::debug!(
            "hitting galaxy-library for user_id={} (token len {})",
            self.user_id,
            creds.access_token.len()
        );
        let client = http::client();
        let mut ids = Vec::new();
        let mut page_token: Option<String> = None;

        let url = format!(
            "https://galaxy-library.gog.com/users/{}/releases",
            self.user_id
        );
        loop {
            let mut request = client.get(&url).bearer_auth(&creds.access_token);
            if let Some(tok) = &page_token {
                request = request.query(&[("page_token", tok)]);
            }
            let resp = request.send().await?;
            let status = resp.status();
            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                tracing::error!(
                    "galaxy-library returned {} - body (first 300): {}",
                    status,
                    body.chars().take(300).collect::<String>()
                );
                anyhow::bail!("galaxy-library returned {}", status);
            }
            let v: serde_json::Value = resp.json().await?;
            let items = v.get("items").and_then(|i| i.as_array());
            ids.extend(items.into_iter().flatten().filter_map(|item| {
                if item.get("platform_id").and_then(|p| p.as_str()) != Some("gog") {
                    return None;
                }
                item.get("external_id")
                    .and_then(|e| e.as_str())
                    .filter(|id| !id.is_empty())
                    .map(String::from)
            }));
            match v
                .get("next_page_token")
                .and_then(|t| t.as_str())
                .filter(|s| !s.is_empty())
            {
                Some(tok) => page_token = Some(tok.to_string()),
                None => break,
            }
        }

        let mut games: Vec<StoreGame> = stream::iter(ids)
            .map(|external_id| store_game(client, external_id))
            .buffer_unordered(METADATA_FETCHES)
            .filter_map(future::ready)
            .collect()
            .await;

        // gogdl writes goggame-*.info only on success, so without this marker a stale registry entry reads as installed over an empty dir
        let installed = list_installed_map();
        for g in &mut games {
            if let Some(p) = installed.get(&g.app_name) {
                g.is_installed = source::dir_has_info_marker(p, &g.app_name);
                g.install_path = g.is_installed.then(|| p.clone());
            }
        }

        games.sort_by_key(|a| a.title.to_lowercase());
        tracing::info!("got {} games from library", games.len());
        save_cached_library(&games);
        Ok(games)
    }

    pub fn logout(&mut self) {
        let _ = fs_err::remove_file(gog_auth_path());
        let _ = fs_err::remove_file(user_data_path());
        let _ = fs_err::remove_file(store::cache::library_path(STORE));
        self.display_name.clear();
        self.user_id.clear();
    }
}

pub fn gog_dir() -> PathBuf {
    crate::runtime_dir().join("gog")
}

fn registry_path() -> PathBuf {
    gog_dir().join("installed.json")
}

fn list_installed_map() -> HashMap<String, PathBuf> {
    store::registry::read(&registry_path())
        .into_iter()
        .map(|(app_name, e)| (app_name, e.install_path))
        .collect()
}

pub use crate::store::registry::InstalledInfo;

pub fn find_installed_info(app_name: &str) -> Option<InstalledInfo> {
    let entry = store::registry::entry(&registry_path(), app_name)?;
    // gogdl leaves executable blank on some titles, so go looking inside the install
    let exe_rel = if entry.has_executable() {
        Some(entry.executable.clone())
    } else {
        source::find_game_exe(&entry.install_path, app_name)
    };
    Some(entry.resolved(exe_rel))
}

pub fn record_install(
    app_name: &str,
    install_path: &Path,
    executable: &str,
    title: &str,
) -> Result<()> {
    let entry = store::registry::Entry {
        install_path: install_path.to_path_buf(),
        executable: executable.to_string(),
        title: Some(title.to_string()),
    };
    store::registry::insert(&registry_path(), app_name, entry)
}

pub fn remove_install(app_name: &str) -> Result<()> {
    store::registry::remove(&registry_path(), app_name)
}

pub fn uninstall(app_name: &str, fallback_title: &str) -> Result<()> {
    if let Some(installed) = find_installed_info(app_name)
        && installed.install_path.exists()
    {
        let path = &installed.install_path;
        fs_err::remove_dir_all(path).map_err(|e| anyhow!("Failed to remove install dir: {e}"))?;
        let wrapper_name =
            install_wrapper_dir_name(installed.title.as_deref().unwrap_or(fallback_title));
        if !wrapper_name.is_empty()
            && let Some(parent) = path.parent()
            && parent
                .file_name()
                .is_some_and(|n| n.to_string_lossy() == wrapper_name)
        {
            let _ = fs_err::remove_dir(parent);
        }
    }
    if let Err(e) = remove_install(app_name) {
        tracing::error!("registry remove failed: {}", e);
    }
    Ok(())
}

// must stay in sync with folderName in qml/components/lib/Paths.js
pub fn install_wrapper_dir_name(title: &str) -> String {
    let name: String = title
        .chars()
        .filter(|c| !"\\/:*?\"<>|".contains(*c))
        .collect();
    match name.trim() {
        "" => "Game".to_string(),
        trimmed => trimmed.to_string(),
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GogDlc {
    pub id: String,
    pub title: String,
    pub download_bytes: u64,
    pub install_bytes: u64,
    pub image: String,
}

async fn fetch_dlc_art(app_name: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let Ok(resp) = http::client()
        .get(format!(
            "https://api.gog.com/products/{app_name}?expand=expanded_dlcs"
        ))
        .send()
        .await
    else {
        return out;
    };
    let Ok(v) = resp.json::<serde_json::Value>().await else {
        return out;
    };
    let Some(list) = v.get("expanded_dlcs").and_then(|d| d.as_array()) else {
        return out;
    };
    for e in list {
        let Some(id) = e.get("id").and_then(|i| i.as_i64()) else {
            continue;
        };
        let Some(url) = ["logo2x", "logo", "icon"]
            .iter()
            .filter_map(|k| e.pointer(&format!("/images/{k}")).and_then(|u| u.as_str()))
            .find(|u| !u.is_empty())
        else {
            continue;
        };
        let url = if let Some(rest) = url.strip_prefix("//") {
            format!("https://{rest}")
        } else {
            url.to_string()
        };
        out.insert(id.to_string(), url);
    }
    out
}

#[derive(Debug, Clone, Default)]
pub struct InstallSize {
    pub download_bytes: u64,
    pub install_bytes: u64,
    pub dlcs: Vec<GogDlc>,
}

fn extract_dlcs(v: &serde_json::Value) -> Vec<GogDlc> {
    let Some(list) = v.get("dlcs").and_then(|d| d.as_array()) else {
        return Vec::new();
    };
    list.iter()
        .filter_map(|d| {
            let id = d.get("id")?.as_str()?.to_string();
            let title = d
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or_default()
                .to_string();
            let (install_bytes, download_bytes) = extract_sizes(d);
            Some(GogDlc {
                id,
                title,
                download_bytes,
                install_bytes,
                image: String::new(),
            })
        })
        .collect()
}

// linux-native games return 0/0 sizes from gogdl and would need their own path, deferred
pub async fn fetch_install_size(app_name: &str) -> Result<InstallSize> {
    let mut info = gogdl_info(app_name, None).await?;

    if extract_sizes(&info) == (0, 0)
        && let Some(build_id) = latest_build_id(&info)
    {
        tracing::debug!(
            "no manifest in default response - retrying with --build {}",
            build_id
        );
        match gogdl_info(app_name, Some(&build_id)).await {
            Ok(pinned) if extract_sizes(&pinned) != (0, 0) => info = pinned,
            Ok(_) => tracing::error!("--build retry still returned no sizes"),
            Err(e) => tracing::error!("--build retry also failed: {}", e),
        }
    }

    let (install_bytes, download_bytes) = extract_sizes(&info);
    if install_bytes == 0 && download_bytes == 0 {
        let dump = serde_json::to_string_pretty(&info).unwrap_or_default();
        tracing::warn!(
            "no manifest sizes for {} - full gogdl info response:\n{}",
            app_name,
            dump
        );
    }
    let mut dlcs = extract_dlcs(&info);
    if !dlcs.is_empty() {
        let art = fetch_dlc_art(app_name).await;
        for d in &mut dlcs {
            if let Some(url) = art.get(&d.id) {
                d.image = url.clone();
            }
        }
    }

    Ok(InstallSize {
        download_bytes,
        install_bytes,
        dlcs,
    })
}

// prefer /size/en-US plus the "*" common payload, then any non-star locale, then the legacy manifest.disk_size shape
fn extract_sizes(v: &serde_json::Value) -> (u64, u64) {
    if let Some(size) = v.get("size").and_then(|s| s.as_object()) {
        let common_install = size
            .get("*")
            .and_then(|c| c.get("disk_size"))
            .and_then(|x| x.as_u64())
            .unwrap_or(0);
        let common_download = size
            .get("*")
            .and_then(|c| c.get("download_size"))
            .and_then(|x| x.as_u64())
            .unwrap_or(0);

        let pick = size
            .get("en-US")
            .or_else(|| size.get("en-us"))
            .or_else(|| size.iter().find(|(k, _)| k.as_str() != "*").map(|(_, v)| v));

        if let Some(locale) = pick {
            let install = locale
                .get("disk_size")
                .and_then(|x| x.as_u64())
                .unwrap_or(0);
            let download = locale
                .get("download_size")
                .and_then(|x| x.as_u64())
                .unwrap_or(0);
            if install > 0 || download > 0 {
                return (install + common_install, download + common_download);
            }
        }
    }

    let install_bytes = v
        .pointer("/manifest/disk_size")
        .and_then(|x| x.as_u64())
        .or_else(|| v.pointer("/manifest/size").and_then(|x| x.as_u64()))
        .or_else(|| v.get("disk_size").and_then(|x| x.as_u64()))
        .or_else(|| {
            v.pointer("/manifest/perLangSize")
                .and_then(|m| m.as_object())
                .map(|obj| {
                    obj.values()
                        .filter_map(|v| v.get("disk_size").and_then(|x| x.as_u64()))
                        .sum()
                })
                .filter(|n: &u64| *n > 0)
        })
        .unwrap_or(0);

    let download_bytes = v
        .pointer("/manifest/download_size")
        .and_then(|x| x.as_u64())
        .or_else(|| v.get("download_size").and_then(|x| x.as_u64()))
        .or_else(|| {
            v.pointer("/manifest/perLangSize")
                .and_then(|m| m.as_object())
                .map(|obj| {
                    obj.values()
                        .filter_map(|v| v.get("download_size").and_then(|x| x.as_u64()))
                        .sum()
                })
                .filter(|n: &u64| *n > 0)
        })
        .unwrap_or(0);

    (install_bytes, download_bytes)
}

// gogdl drops `.gogdl-resume` at the install root during an interrupted dowload
pub fn inspect_existing_install(_app_name: &str, install_path: &Path) -> (u64, bool) {
    if !install_path.exists() {
        return (0, false);
    }
    let has_resume =
        install_path.join(".gogdl-resume").exists() || install_path.join(".gogdl-temp").exists();
    (fs_util::dir_size(install_path), has_resume)
}

#[derive(Clone)]
pub struct GogCredentials {
    pub access_token: String,
    pub user_id: Option<String>,
}

fn parse_credentials(value: &serde_json::Value) -> Option<GogCredentials> {
    let access_token = value
        .get("access_token")
        .and_then(|s| s.as_str())?
        .to_string();
    if access_token.is_empty() {
        return None;
    }
    let user_id = value.get("user_id").and_then(|x| match x {
        serde_json::Value::String(s) if !s.is_empty() => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        _ => None,
    });
    Some(GogCredentials {
        access_token,
        user_id,
    })
}

pub async fn read_credentials() -> Result<GogCredentials> {
    let auth = gog_auth_path();
    let output = AsyncCommand::from(gogdl_command()?)
        .arg("auth")
        .output()
        .await?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        tracing::warn!("gogdl auth stderr: {}", err.trim());
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    let trimmed = raw.trim();

    if !trimmed.is_empty() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed)
            && let Some(creds) = parse_credentials(&v)
        {
            return Ok(creds);
        }
        if let Some(creds) = find_json_blob(trimmed) {
            return Ok(creds);
        }
    }

    // auth.json is keyed by user_id at teh top level: { "<user_id>": { access_token, .... } }
    if auth.exists() {
        let body = fs_err::read_to_string(&auth)?;
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
            let creds_val = if v.get("access_token").is_some() {
                &v
            } else if let Some(obj) = v.as_object() {
                obj.values().next().unwrap_or(&v)
            } else {
                &v
            };
            if let Some(creds) = parse_credentials(creds_val) {
                return Ok(creds);
            }
        }
        tracing::error!(
            "couldn't parse auth file at {} ({} bytes)",
            auth.display(),
            body.len()
        );
    }

    tracing::error!(
        "gogdl auth stdout had no usable credentials ({} bytes)",
        trimmed.len()
    );
    anyhow::bail!(
        "couldn't read gogdl credentials from stdout or {}",
        auth.display()
    )
}

fn find_json_blob(s: &str) -> Option<GogCredentials> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut start: Option<usize> = None;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'{' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(s_idx) = start {
                        let chunk = &s[s_idx..=i];
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(chunk)
                            && let Some(creds) = parse_credentials(&v)
                        {
                            return Some(creds);
                        }
                    }
                    start = None;
                }
            }
            _ => {}
        }
    }
    None
}

fn gogdl_bin() -> Result<PathBuf> {
    find_gogdl().ok_or_else(|| {
        anyhow!(
            "gogdl not found, install via Settings > Components or place the binary at {}",
            crate::runtime_dir().join("gogdl").display()
        )
    })
}

fn gogdl_command() -> Result<Command> {
    let config = gogdl_config_dir();
    let _ = fs_err::create_dir_all(&config);
    let mut cmd = Command::new(gogdl_bin()?);
    cmd.env("GOGDL_CONFIG_PATH", &config)
        .arg("--auth-config-path")
        .arg(gog_auth_path());
    Ok(cmd)
}

async fn gogdl_output(args: &[&str]) -> Result<Output> {
    let output = AsyncCommand::from(gogdl_command()?)
        .args(args)
        .output()
        .await?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let subcommand = args.first().copied().unwrap_or_default();
        anyhow::bail!("gogdl {} failed: {}", subcommand, err.trim());
    }
    Ok(output)
}

async fn gogdl_info(app_name: &str, build: Option<&str>) -> Result<serde_json::Value> {
    let mut args = vec!["info", app_name, "--os", "windows"];
    args.extend(build.map(|build| ["--build", build]).into_iter().flatten());
    Ok(serde_json::from_slice(&gogdl_output(&args).await?.stdout)?)
}

fn latest_build_id(info: &serde_json::Value) -> Option<String> {
    info.pointer("/builds/items/0/build_id")
        .and_then(|b| b.as_str())
        .map(String::from)
}

pub fn find_gogdl() -> Option<PathBuf> {
    components::path_for(SettingsKey::Gogdl)
}

pub fn gog_auth_path() -> PathBuf {
    gog_dir().join("auth.json")
}

// our own dir via GOGDL_CONFIG_PATH so we can wipe stale manifests (which turn downloads into "Nothing to do") without touching the user's env
pub fn gogdl_config_dir() -> PathBuf {
    gog_dir().join("gogdl_state")
}

pub fn installed_dlcs(app_name: &str) -> Vec<GogDlc> {
    let Some(path) = fs_util::find_file_named(&gogdl_config_dir(), app_name) else {
        return Vec::new();
    };
    let Ok(text) = fs_err::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    let Some(installed) = v.get("HGLdlcs").and_then(|p| p.as_array()) else {
        return Vec::new();
    };
    installed
        .iter()
        .filter_map(|p| {
            let id = p.get("id")?.as_str()?.to_string();
            let title = p
                .get("title")
                .and_then(|n| n.as_str())
                .unwrap_or(&id)
                .to_string();
            Some(GogDlc {
                id,
                title,
                download_bytes: 0,
                install_bytes: 0,
                image: String::new(),
            })
        })
        .collect()
}

// gogdl deltas against ghost state otherwise, so wipe before a fresh install. heroic does the same on buildid change
pub fn wipe_gogdl_manifest_for(app_id: &str) {
    // layout varies by gogdl version, so match on basename instead of hardcoding a path
    let stale: Vec<PathBuf> = fs_util::walk(&gogdl_config_dir())
        .filter(|e| e.file_name() == app_id)
        .map(|e| e.path())
        .collect();
    for path in stale {
        let removed = if path.is_dir() {
            fs_err::remove_dir_all(&path)
        } else {
            fs_err::remove_file(&path)
        };
        if removed.is_ok() {
            tracing::debug!("cleared stale gogdl state: {}", path.display());
        }
    }
}

fn user_data_path() -> PathBuf {
    gog_dir().join("user.json")
}

#[derive(Default, Serialize, Deserialize)]
struct UserData {
    #[serde(default)]
    username: String,
    #[serde(default, rename = "userId")]
    user_id: String,
}

fn read_user_data() -> Option<UserData> {
    serde_json::from_str(&fs_err::read_to_string(user_data_path()).ok()?).ok()
}

fn save_user_data(user: &UserData) {
    if let Ok(body) = serde_json::to_string(user) {
        let _ = fs_util::write_atomic(&user_data_path(), body);
    }
}

async fn store_game(client: &reqwest::Client, external_id: String) -> Option<StoreGame> {
    let meta = match fetch_game_metadata(client, &external_id).await {
        Ok(meta) => meta,
        Err(e) => {
            tracing::warn!("skipping {}: {}", external_id, e);
            return None;
        }
    };
    if meta.is_dlc {
        tracing::debug!("skipping dlc {} ({})", meta.title, external_id);
        return None;
    }
    Some(StoreGame {
        banner: resolve_gog_image(&external_id, "banner", meta.banner.as_deref()),
        coverart: resolve_gog_image(&external_id, "coverart", meta.coverart.as_deref()),
        icon: resolve_gog_image(&external_id, "icon", meta.icon.as_deref()),
        app_name: external_id,
        title: meta.title,
        is_installed: false,
        install_path: None,
    })
}

struct ProductMeta {
    title: String,
    banner: Option<String>,
    coverart: Option<String>,
    icon: Option<String>,
    is_dlc: bool,
}

async fn fetch_game_metadata(client: &reqwest::Client, external_id: &str) -> Result<ProductMeta> {
    let url = format!("https://api.gog.com/v2/games/{}?locale=en-US", external_id);
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("api.gog.com v2 returned {}", resp.status());
    }
    let v: serde_json::Value = resp.json().await?;
    let title = v
        .pointer("/_embedded/product/title")
        .and_then(|t| t.as_str())
        .or_else(|| v.get("title").and_then(|t| t.as_str()))
        .unwrap_or(external_id)
        .to_string();
    let coverart = v
        .pointer("/_links/boxArtImage/href")
        .and_then(|u| u.as_str())
        .map(normalize_image_url);
    let banner = v
        .pointer("/_links/backgroundImage/href")
        .and_then(|u| u.as_str())
        .map(normalize_image_url)
        .or_else(|| coverart.clone());
    let icon = v
        .pointer("/_links/icon/href")
        .and_then(|u| u.as_str())
        .map(normalize_image_url)
        .or_else(|| coverart.clone());
    // PACK is playable (bundles, goty editions), only DLC is an add-on. ok lol
    let is_dlc = v
        .pointer("/_embedded/productType")
        .and_then(|t| t.as_str())
        .is_some_and(|t| t.eq_ignore_ascii_case("dlc"));
    Ok(ProductMeta {
        title,
        banner,
        coverart,
        icon,
        is_dlc,
    })
}

// gog's image urls carry {formatter} placeholders; the plain url is already fine for our card slot
fn normalize_image_url(raw: &str) -> String {
    raw.replace("{formatter}", "").replace(".{ext}", ".jpg")
}

pub fn load_cached_library() -> Vec<StoreGame> {
    store::cache::load_library(STORE)
}

pub fn save_cached_library(games: &[StoreGame]) {
    store::cache::save_library(STORE, games);
}

fn resolve_gog_image(app_name: &str, kind: &str, cdn_url: Option<&str>) -> Option<String> {
    store::cache::resolve_image(STORE, app_name, kind, cdn_url, str::to_string)
}

pub async fn fetch_game_details(app_name: &str) -> Result<String> {
    let client = http::client();

    // summary comes from gamesdb because v2's own description field is promo html with inline css. genuinely why
    let mut description = String::new();
    if let Ok(resp) = client
        .get(format!(
            "https://gamesdb.gog.com/platforms/gog/external_releases/{app_name}"
        ))
        .send()
        .await
        && let Ok(v) = resp.json::<serde_json::Value>().await
        && let Some(s) = v.pointer("/summary/*").and_then(|s| s.as_str())
    {
        description = s.trim().to_string();
    }

    let mut reqs = Vec::new();
    if let Ok(resp) = client
        .get(format!(
            "https://api.gog.com/v2/games/{app_name}?locale=en-US"
        ))
        .send()
        .await
        && let Ok(v) = resp.json::<serde_json::Value>().await
    {
        reqs = extract_windows_reqs(&v);
    }

    if description.is_empty() && reqs.is_empty() {
        anyhow::bail!("no details available for {app_name}");
    }
    Ok(serde_json::json!({ "description": description, "reqs": reqs }).to_string())
}

fn extract_windows_reqs(v: &serde_json::Value) -> Vec<serde_json::Value> {
    let groups = v
        .pointer("/_embedded/supportedOperatingSystems")
        .and_then(|o| o.as_array())
        .and_then(|oss| {
            oss.iter().find(|o| {
                o.pointer("/operatingSystem/name").and_then(|n| n.as_str()) == Some("windows")
            })
        })
        .and_then(|win| win.get("systemRequirements"))
        .and_then(|r| r.as_array());
    let Some(groups) = groups else {
        return Vec::new();
    };

    let group = |ty: &str| {
        groups
            .iter()
            .find(|g| g.get("type").and_then(|t| t.as_str()) == Some(ty))
            .and_then(|g| g.get("requirements"))
            .and_then(|r| r.as_array())
    };
    let Some(minimum) = group("minimum") else {
        return Vec::new();
    };
    let recommended = group("recommended");

    minimum
        .iter()
        .filter_map(|r| {
            let name = r.get("name")?.as_str()?.trim_end_matches(':').to_string();
            let desc = r.get("description")?.as_str()?.trim();
            if desc.is_empty() {
                return None;
            }
            let id = r.get("id").and_then(|i| i.as_str()).unwrap_or_default();
            let rec = recommended
                .and_then(|rs| {
                    rs.iter()
                        .find(|c| c.get("id").and_then(|i| i.as_str()) == Some(id))
                })
                .and_then(|c| c.get("description"))
                .and_then(|d| d.as_str())
                .unwrap_or_default();
            Some(serde_json::json!({ "title": name, "minimum": desc, "recommended": rec }))
        })
        .collect()
}

// TODO hide dlcs from the store cards, they're literally useless but i will do this in a year or something

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_sizes_from_new_shape() {
        let body = r#"{
            "size": {
                "*": { "disk_size": 755, "download_size": 202 },
                "en-US": { "disk_size": 22258049719, "download_size": 15862889094 },
                "de-DE": { "disk_size": 22258049718, "download_size": 15862889094 }
            }
        }"#;
        let v: serde_json::Value = serde_json::from_str(body).unwrap();
        let (install, download) = extract_sizes(&v);
        assert_eq!(install, 22258049719 + 755);
        assert_eq!(download, 15862889094 + 202);
    }

    #[test]
    fn extract_sizes_from_manifest_legacy() {
        let body = r#"{ "manifest": { "disk_size": 1000, "download_size": 500 } }"#;
        let v: serde_json::Value = serde_json::from_str(body).unwrap();
        let (install, download) = extract_sizes(&v);
        assert_eq!(install, 1000);
        assert_eq!(download, 500);
    }

    #[test]
    fn extract_sizes_no_data() {
        let body = r#"{ "buildId": "x", "builds": { "items": [] } }"#;
        let v: serde_json::Value = serde_json::from_str(body).unwrap();
        let (install, download) = extract_sizes(&v);
        assert_eq!(install, 0);
        assert_eq!(download, 0);
    }
}
