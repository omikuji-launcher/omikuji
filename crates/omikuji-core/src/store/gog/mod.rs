// little note: FUCK YOU GOG. we love you really but what the fuck
pub mod source;
pub mod updates;

use crate::store::StoreGame;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::process::Command as AsyncCommand;

const STORE: &str = "gog";

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
        let (name, id) = read_user_data().unwrap_or_default();
        Self {
            display_name: name,
            user_id: id,
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
        let bin = gogdl_bin()?;
        let auth = gog_auth_path();
        if let Some(parent) = auth.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let gogdl_cfg = gogdl_config_dir();
        let _ = std::fs::create_dir_all(&gogdl_cfg);
        let output = AsyncCommand::new(&bin)
            .env("GOGDL_CONFIG_PATH", &gogdl_cfg)
            .arg("--auth-config-path")
            .arg(&auth)
            .arg("auth")
            .arg("--code")
            .arg(code.trim())
            .output()
            .await?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gogdl auth failed: {}", err.trim());
        }

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
        let token_user_id = creds
            .user_id
            .as_ref()
            .filter(|s| !s.is_empty())
            .cloned()
            .unwrap_or_default();
        if !token_user_id.is_empty() {
            self.user_id = token_user_id.clone();
        }

        let resp = crate::http::client()
            .get("https://embed.gog.com/userData.json")
            .bearer_auth(&creds.access_token)
            .header(
                "User-Agent",
                "omikuji/0.1 (+https://github.com/reakjra/omikuji)",
            )
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
        save_user_data(&self.display_name, &self.user_id);
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
            anyhow::bail!("user id unresolved — try logging in again (userData.json call failed)");
        }

        let creds = read_credentials().await?;
        tracing::debug!(
            "hitting galaxy-library for user_id={} (token len {})",
            self.user_id,
            creds.access_token.len()
        );
        let client = crate::http::client();
        let mut games = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let url = match &page_token {
                Some(tok) => format!(
                    "https://galaxy-library.gog.com/users/{}/releases?page_token={}",
                    self.user_id,
                    urlencoding_simple(tok)
                ),
                None => format!(
                    "https://galaxy-library.gog.com/users/{}/releases",
                    self.user_id
                ),
            };
            let resp = client
                .get(&url)
                .bearer_auth(&creds.access_token)
                .send()
                .await?;
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
            if let Some(items) = v.get("items").and_then(|i| i.as_array()) {
                for item in items {
                    let platform = item
                        .get("platform_id")
                        .and_then(|p| p.as_str())
                        .unwrap_or("");
                    if platform != "gog" {
                        continue;
                    }
                    let external_id = item
                        .get("external_id")
                        .and_then(|e| e.as_str())
                        .unwrap_or("")
                        .to_string();
                    if external_id.is_empty() {
                        continue;
                    }
                    match fetch_game_metadata(client, &external_id).await {
                        Ok((title, banner, coverart, icon)) => {
                            let banner_r =
                                resolve_gog_image(&external_id, "banner", banner.as_deref());
                            let coverart_r =
                                resolve_gog_image(&external_id, "coverart", coverart.as_deref());
                            let icon_r = resolve_gog_image(&external_id, "icon", icon.as_deref());
                            games.push(StoreGame {
                                app_name: external_id,
                                title,
                                banner: banner_r,
                                coverart: coverart_r,
                                icon: icon_r,
                                is_installed: false,
                                install_path: None,
                            });
                        }
                        Err(e) => {
                            tracing::warn!("skipping {}: {}", external_id, e);
                        }
                    }
                }
            }
            match v
                .get("next_page_token")
                .and_then(|t| t.as_str())
                .filter(|s| !s.is_empty())
            {
                Some(tok) => page_token = Some(tok.to_string()),
                None => break,
            }
        }

        // gogdl writes goggame-*.info only on success, so without this marker a stale registry entry reads as installed over an empty dir
        let installed = list_installed_map().unwrap_or_default();
        for g in &mut games {
            if let Some(p) = installed.get(&g.app_name) {
                let really_installed = p.exists() && has_install_marker(p);
                g.is_installed = really_installed;
                g.install_path = if really_installed {
                    Some(p.clone())
                } else {
                    None
                };
            }
        }

        games.sort_by_key(|a| a.title.to_lowercase());
        tracing::info!("got {} games from library", games.len());
        save_cached_library(&games);
        Ok(games)
    }

    pub fn logout(&mut self) {
        let auth = gog_auth_path();
        if auth.exists() {
            let _ = std::fs::remove_file(&auth);
        }
        let user = user_data_path();
        if user.exists() {
            let _ = std::fs::remove_file(&user);
        }
        let _ = std::fs::remove_file(crate::store::cache::library_path(STORE));
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

fn has_install_marker(dir: &Path) -> bool {
    let scan = |d: &Path| -> bool {
        if let Ok(entries) = std::fs::read_dir(d) {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with("goggame-") && name.ends_with(".info") {
                    return true;
                }
            }
        }
        false
    };
    if scan(dir) {
        return true;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            if e.file_type().ok().map(|t| t.is_dir()).unwrap_or(false) && scan(&e.path()) {
                return true;
            }
        }
    }
    false
}

fn list_installed_map() -> Result<HashMap<String, PathBuf>> {
    Ok(crate::store::registry::read(&registry_path())
        .into_iter()
        .map(|(app_name, e)| (app_name, e.install_path))
        .collect())
}

pub use crate::store::registry::InstalledInfo;

pub fn find_installed_info(app_name: &str) -> Option<InstalledInfo> {
    let entry = crate::store::registry::entry(&registry_path(), app_name)?;
    // gogdl leaves executable blank on some titles, so go looking inside the install
    let exe_rel = if entry.has_executable() {
        Some(entry.executable.clone())
    } else {
        source::find_game_exe_pub(&entry.install_path, app_name)
    };
    Some(entry.resolved(exe_rel))
}

pub fn record_install(
    app_name: &str,
    install_path: &Path,
    executable: &str,
    title: &str,
) -> Result<()> {
    let registry = registry_path();
    let mut v: serde_json::Value = if registry.exists() {
        serde_json::from_str(&std::fs::read_to_string(&registry)?).unwrap_or_default()
    } else {
        serde_json::json!({})
    };
    let obj = v
        .as_object_mut()
        .ok_or_else(|| anyhow!("registry corrupt"))?;
    obj.insert(
        app_name.to_string(),
        serde_json::json!({
            "install_path": install_path.to_string_lossy().to_string(),
            "executable": executable,
            "title": title,
        }),
    );
    crate::fs_util::write_atomic(&registry, serde_json::to_string_pretty(&v)?)?;
    Ok(())
}

pub fn remove_install(app_name: &str) -> Result<()> {
    let registry = registry_path();
    if !registry.exists() {
        return Ok(());
    }
    let mut v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&registry)?)?;
    if let Some(obj) = v.as_object_mut() {
        obj.remove(app_name);
    }
    crate::fs_util::write_atomic(&registry, serde_json::to_string_pretty(&v)?)?;
    Ok(())
}

// must stay in sync with the folder-name sanitize in GogInstallDialog.qml
pub fn install_wrapper_dir_name(title: &str) -> String {
    title
        .chars()
        .filter(|c| !"\\/:*?\"<>|".contains(*c))
        .collect::<String>()
        .trim()
        .to_string()
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GogDlc {
    pub id: String,
    pub title: String,
    pub download_bytes: u64,
    pub install_bytes: u64,
    pub image: String,
}

async fn fetch_dlc_art(app_name: &str) -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    let Ok(resp) = crate::http::client()
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
    let bin = gogdl_bin()?;
    let auth = gog_auth_path();
    let gogdl_cfg = gogdl_config_dir();
    let _ = std::fs::create_dir_all(&gogdl_cfg);
    let output = AsyncCommand::new(&bin)
        .env("GOGDL_CONFIG_PATH", &gogdl_cfg)
        .arg("--auth-config-path")
        .arg(&auth)
        .arg("info")
        .arg(app_name)
        .arg("--os")
        .arg("windows")
        .output()
        .await?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gogdl info failed: {}", err.trim());
    }

    let v: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let (install_bytes, download_bytes) = extract_sizes(&v);

    if install_bytes == 0 && download_bytes == 0 {
        let latest_build = v
            .pointer("/builds/items/0/build_id")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string());
        if let Some(build_id) = latest_build {
            tracing::debug!(
                "no manifest in default response - retrying with --build {}",
                build_id
            );
            match fetch_install_size_pinned(app_name, &build_id).await {
                Ok(s) => return Ok(s),
                Err(e) => {
                    tracing::error!("--build retry also failed: {}", e);
                }
            }
        }

        let dump = serde_json::to_string_pretty(&v).unwrap_or_default();
        tracing::warn!(
            "no manifest sizes for {} - full gogdl info response:\n{}",
            app_name,
            dump
        );
    }
    let mut dlcs = extract_dlcs(&v);
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

async fn fetch_install_size_pinned(app_name: &str, build_id: &str) -> Result<InstallSize> {
    let bin = gogdl_bin()?;
    let auth = gog_auth_path();
    let gogdl_cfg = gogdl_config_dir();
    let _ = std::fs::create_dir_all(&gogdl_cfg);
    let output = AsyncCommand::new(&bin)
        .env("GOGDL_CONFIG_PATH", &gogdl_cfg)
        .arg("--auth-config-path")
        .arg(&auth)
        .arg("info")
        .arg(app_name)
        .arg("--os")
        .arg("windows")
        .arg("--build")
        .arg(build_id)
        .output()
        .await?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("gogdl info --build failed: {}", err.trim());
    }
    let v: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let (install_bytes, download_bytes) = extract_sizes(&v);
    if install_bytes == 0 && download_bytes == 0 {
        anyhow::bail!("--build retry still returned no sizes");
    }
    Ok(InstallSize {
        download_bytes,
        install_bytes,
        dlcs: extract_dlcs(&v),
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

// gogdl drops `.gogdl-resume` at the install root during an interrupted dowload
pub fn inspect_existing_install(_app_name: &str, install_path: &Path) -> (u64, bool) {
    if !install_path.exists() {
        return (0, false);
    }
    let has_resume =
        install_path.join(".gogdl-resume").exists() || install_path.join(".gogdl-temp").exists();
    (crate::fs_util::dir_size(install_path), has_resume)
}

#[derive(Debug, Clone)]
pub struct GogCredentials {
    pub access_token: String,
    #[allow(dead_code)]
    pub refresh_token: Option<String>,
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
    let refresh_token = value
        .get("refresh_token")
        .and_then(|s| s.as_str())
        .map(String::from);
    let user_id = value.get("user_id").and_then(|x| match x {
        serde_json::Value::String(s) if !s.is_empty() => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        _ => None,
    });
    Some(GogCredentials {
        access_token,
        refresh_token,
        user_id,
    })
}

pub async fn read_credentials() -> Result<GogCredentials> {
    let bin = gogdl_bin()?;
    let auth = gog_auth_path();
    let gogdl_cfg = gogdl_config_dir();
    let _ = std::fs::create_dir_all(&gogdl_cfg);
    let output = AsyncCommand::new(&bin)
        .env("GOGDL_CONFIG_PATH", &gogdl_cfg)
        .arg("--auth-config-path")
        .arg(&auth)
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
        let body = std::fs::read_to_string(&auth)?;
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
            "couldn't parse auth file at {} (first 200 chars): {}",
            auth.display(),
            body.chars().take(200).collect::<String>()
        );
    }

    tracing::error!(
        "gogdl auth stdout (first 500 chars): {}",
        trimmed.chars().take(500).collect::<String>()
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
            "gogdl not found — install via first-run components or place the binary at {}",
            crate::runtime_dir().join("gogdl").display()
        )
    })
}

pub fn find_gogdl() -> Option<PathBuf> {
    let bundled = crate::runtime_dir().join("gogdl");
    if bundled.exists() {
        return Some(bundled);
    }
    if let Ok(p) = which::which("gogdl") {
        return Some(p);
    }
    None
}

pub fn gog_auth_path() -> PathBuf {
    gog_dir().join("auth.json")
}

// our own dir via GOGDL_CONFIG_PATH so we can wipe stale manifests (which turn downloads into "Nothing to do") without touching the user's env
pub fn gogdl_config_dir() -> PathBuf {
    gog_dir().join("gogdl_state")
}

pub fn installed_dlcs(app_name: &str) -> Vec<GogDlc> {
    let root = gogdl_config_dir();
    if !root.exists() {
        return Vec::new();
    }
    let mut found = None;
    for dir in walkdir_manifests(&root) {
        let candidate = dir.join(app_name);
        if candidate.is_file() {
            found = Some(candidate);
            break;
        }
    }
    let Some(path) = found else {
        return Vec::new();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
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

fn walkdir_manifests(root: &Path) -> Vec<PathBuf> {
    let mut out = vec![root.to_path_buf()];
    let mut i = 0;
    while i < out.len() {
        if let Ok(entries) = std::fs::read_dir(&out[i]) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    out.push(p);
                }
            }
        }
        i += 1;
    }
    out
}

// gogdl deltas against ghost state otherwise, so wipe before a fresh install. heroic does the same on buildid change
pub fn wipe_gogdl_manifest_for(app_id: &str) {
    let root = gogdl_config_dir();
    if !root.exists() {
        return;
    }
    // layout varies by gogdl version, so match on basename instead of hardcoding a path
    fn walk(dir: &Path, needle: &str) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            let md = match e.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            if name == needle {
                if md.is_dir() {
                    let _ = std::fs::remove_dir_all(&p);
                } else {
                    let _ = std::fs::remove_file(&p);
                }
                tracing::debug!("cleared stale gogdl state: {}", p.display());
                continue;
            }
            if md.is_dir() {
                walk(&p, needle);
            }
        }
    }
    walk(&root, app_id);
}

fn urlencoding_simple(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

fn user_data_path() -> PathBuf {
    gog_dir().join("user.json")
}

fn read_user_data() -> Option<(String, String)> {
    let path = user_data_path();
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    let name = v
        .get("username")
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();
    let id = v
        .get("userId")
        .and_then(|i| i.as_str())
        .unwrap_or("")
        .to_string();
    Some((name, id))
}

fn save_user_data(name: &str, id: &str) {
    let body = serde_json::json!({ "username": name, "userId": id }).to_string();
    let _ = crate::fs_util::write_atomic(&user_data_path(), body);
}

async fn fetch_game_metadata(
    client: &reqwest::Client,
    external_id: &str,
) -> Result<(String, Option<String>, Option<String>, Option<String>)> {
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
    Ok((title, banner, coverart, icon))
}

// gog's image urls carry {formatter} placeholders; the plain url is already fine for our card slot
fn normalize_image_url(raw: &str) -> String {
    raw.replace("{formatter}", "").replace(".{ext}", ".jpg")
}

pub fn load_cached_library() -> Vec<StoreGame> {
    crate::store::cache::load_library(STORE)
}

pub fn save_cached_library(games: &[StoreGame]) {
    crate::store::cache::save_library(STORE, games);
}

fn resolve_gog_image(app_name: &str, kind: &str, cdn_url: Option<&str>) -> Option<String> {
    crate::store::cache::resolve_image(STORE, app_name, kind, cdn_url, str::to_string)
}

pub async fn fetch_game_details(app_name: &str) -> Result<String> {
    let client = crate::http::client();

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
