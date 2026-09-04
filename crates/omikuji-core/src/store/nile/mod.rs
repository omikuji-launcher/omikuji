pub mod fuel;
pub mod source;
pub mod updates;

use crate::store::StoreGame;
use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::process::Command as AsyncCommand;

const STORE: &str = "nile";

// nile appends its own nile segment to NILE_CONFIG_PATH
pub fn config_root() -> PathBuf {
    crate::runtime_dir().join("nile_config")
}

pub fn config_dir() -> PathBuf {
    config_root().join("nile")
}

pub fn sdk_dir() -> PathBuf {
    config_dir().join("SDK")
}

fn library_json() -> PathBuf {
    config_dir().join("library.json")
}

fn installed_json() -> PathBuf {
    config_dir().join("installed.json")
}

fn current_user_json() -> PathBuf {
    config_dir().join("current_user.json")
}

pub fn find_nile() -> Option<PathBuf> {
    let bundled = crate::runtime_dir().join("nile");
    if bundled.exists() {
        return Some(bundled);
    }
    which::which("nile").ok()
}

pub fn require_nile() -> Result<PathBuf> {
    find_nile().ok_or_else(|| {
        anyhow!(
            "nile not found, install via Settings > Components or place the binary at {}",
            crate::runtime_dir().join("nile").display()
        )
    })
}

pub fn blocking_command() -> Result<std::process::Command> {
    let mut cmd = std::process::Command::new(require_nile()?);
    cmd.env("NILE_CONFIG_PATH", config_root());
    Ok(cmd)
}

pub fn command() -> Result<AsyncCommand> {
    Ok(AsyncCommand::from(blocking_command()?))
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginData {
    pub url: String,
    pub code_verifier: String,
    pub serial: String,
    pub client_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct CurrentUser {
    #[serde(default)]
    name: String,
}

pub fn read_display_name() -> Option<String> {
    let raw = std::fs::read_to_string(current_user_json()).ok()?;
    let user: CurrentUser = serde_json::from_str(&raw).ok()?;
    (!user.name.is_empty()).then_some(user.name)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Entitlement {
    id: String,
    product: Product,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Product {
    // usually amzn1.adg.product.<uuid>, but some are bare uuid
    id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    sku: String,
    #[serde(default)]
    product_detail: ProductDetail,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProductDetail {
    // amazon uses iconUrl for 3:4 box art, logoUrl is the icon
    #[serde(default)]
    icon_url: String,
    #[serde(default)]
    details: ProductDetails,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProductDetails {
    #[serde(default)]
    logo_url: String,
    #[serde(default)]
    background_url1: String,
}

fn read_entitlements() -> Vec<Entitlement> {
    let Ok(raw) = std::fs::read_to_string(library_json()) else {
        return Vec::new();
    };
    match serde_json::from_str::<Vec<Entitlement>>(&raw) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("nile library.json parse failed: {}", e);
            Vec::new()
        }
    }
}

pub struct ProductIds {
    pub entitlement_id: String,
    pub sku: String,
}

pub fn product_ids(app_id: &str) -> Option<ProductIds> {
    read_entitlements()
        .into_iter()
        .find(|e| e.product.id == app_id)
        .map(|e| ProductIds {
            entitlement_id: e.id,
            sku: e.product.sku,
        })
}

#[derive(Debug, Clone, Deserialize)]
pub struct InstalledEntry {
    pub id: String,
    #[serde(default)]
    pub version: String,
    pub path: PathBuf,
}

pub fn read_installed() -> HashMap<String, InstalledEntry> {
    let Ok(raw) = std::fs::read_to_string(installed_json()) else {
        return HashMap::new();
    };
    match serde_json::from_str::<Vec<InstalledEntry>>(&raw) {
        Ok(entries) => entries.into_iter().map(|e| (e.id.clone(), e)).collect(),
        Err(e) => {
            tracing::warn!("nile installed.json parse failed: {}", e);
            HashMap::new()
        }
    }
}

pub use crate::store::registry::InstalledInfo;

pub fn find_installed_info(app_id: &str) -> Option<InstalledInfo> {
    let install_path = read_installed().remove(app_id)?.path;
    let executable = fuel::load(&install_path)
        .map(|f| f.exe(&install_path))
        .unwrap_or_default();
    Some(InstalledInfo {
        install_path,
        executable,
        title: None,
    })
}

// nile has no resume marker, it hash-checks at run time
pub fn inspect_existing_install(_app_id: &str, install_path: &Path) -> (u64, bool) {
    (crate::fs_util::dir_size(install_path), false)
}

// nile writes each file to <name>.patch and deletes it on the next run instead of appending
pub fn inflight_bytes(install_path: &Path) -> u64 {
    fn walk(dir: &Path, total: &mut u64) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, total);
            } else if path.extension().is_some_and(|e| e == "patch")
                && let Ok(meta) = entry.metadata()
            {
                *total += meta.len();
            }
        }
    }
    let mut total = 0;
    walk(install_path, &mut total);
    total
}

pub fn uninstall(app_id: &str) -> Result<()> {
    let output = blocking_command()?.arg("uninstall").arg(app_id).output()?;
    if !output.status.success() {
        anyhow::bail!(
            "nile uninstall failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

pub fn post_install_summary(install_root: &Path) -> Option<String> {
    let names: Vec<String> = fuel::load(install_root)
        .ok()?
        .post_install
        .iter()
        .map(|p| {
            p.command
                .rsplit(['\\', '/'])
                .next()
                .unwrap_or(&p.command)
                .to_string()
        })
        .collect();
    (!names.is_empty()).then(|| names.join(", "))
}

pub struct InstallSize {
    pub download_bytes: u64,
}

#[derive(Deserialize)]
struct InfoOutput {
    download_size: u64,
}

pub async fn fetch_install_size(app_id: &str) -> Result<InstallSize> {
    let output = command()?
        .arg("install")
        .arg("--info")
        .arg("--json")
        .arg(app_id)
        .output()
        .await?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("nile install --info failed: {}", err.trim());
    }

    let info: InfoOutput = serde_json::from_slice(&output.stdout)?;
    Ok(InstallSize {
        download_bytes: info.download_size,
    })
}

pub fn load_cached_library() -> Vec<StoreGame> {
    crate::store::cache::load_library(STORE)
}

pub fn save_cached_library(games: &[StoreGame]) {
    crate::store::cache::save_library(STORE, games);
}

fn resolve_nile_image(app_id: &str, kind: &str, cdn_url: &str) -> Option<String> {
    let url = (!cdn_url.is_empty()).then_some(cdn_url);
    crate::store::cache::resolve_image(STORE, app_id, kind, url, str::to_string)
}

pub struct NileStore {
    pub display_name: String,
    pending_login: Option<LoginData>,
}

impl Default for NileStore {
    fn default() -> Self {
        Self::new()
    }
}

impl NileStore {
    pub fn new() -> Self {
        Self {
            display_name: read_display_name().unwrap_or_default(),
            pending_login: None,
        }
    }

    pub fn is_logged_in(&self) -> bool {
        read_display_name().is_some()
    }

    pub async fn begin_login(&mut self) -> Result<String> {
        let output = command()?
            .arg("auth")
            .arg("--login")
            .arg("--non-interactive")
            .output()
            .await?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("nile auth --login failed: {}", err.trim());
        }

        let data: LoginData = serde_json::from_slice(&output.stdout)?;
        let url = data.url.clone();
        self.pending_login = Some(data);
        Ok(url)
    }

    fn auth_code_from(input: &str) -> &str {
        let trimmed = input.trim();
        match trimmed.split("openid.oa2.authorization_code=").nth(1) {
            Some(rest) => rest.split('&').next().unwrap_or(rest),
            None => trimmed,
        }
    }

    pub async fn login(&mut self, code: &str) -> Result<String> {
        let pending = self
            .pending_login
            .clone()
            .ok_or_else(|| anyhow!("no login in progress, open the Amazon login page first"))?;

        let output = command()?
            .arg("register")
            .arg("--code")
            .arg(Self::auth_code_from(code))
            .arg("--client-id")
            .arg(&pending.client_id)
            .arg("--code-verifier")
            .arg(&pending.code_verifier)
            .arg("--serial")
            .arg(&pending.serial)
            .output()
            .await?;

        let Some(name) = read_display_name() else {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("nile register failed: {}", err.trim());
        };

        self.pending_login = None;
        self.display_name = name.clone();
        Ok(name)
    }

    pub async fn logout(&mut self) -> Result<()> {
        let output = command()?.arg("auth").arg("--logout").output().await?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            tracing::error!("nile auth --logout failed: {}", err.trim());
        }
        let _ = std::fs::remove_file(crate::store::cache::library_path(STORE));
        self.pending_login = None;
        self.display_name.clear();
        Ok(())
    }

    pub async fn list_games(&mut self) -> Result<Vec<StoreGame>> {
        if !self.is_logged_in() {
            return Ok(Vec::new());
        }

        let output = command()?.arg("library").arg("sync").output().await?;
        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("nile library sync failed: {}", err.trim());
        }

        let installed = read_installed();
        let mut games = Vec::new();
        for entry in read_entitlements() {
            let app_id = entry.product.id;
            if app_id.is_empty() {
                continue;
            }
            let title = entry.product.title.unwrap_or_else(|| app_id.clone());
            let detail = entry.product.product_detail;

            let install_path = installed.get(&app_id).map(|e| e.path.clone());
            let really_installed = install_path.as_ref().map(|p| p.exists()).unwrap_or(false);

            games.push(StoreGame {
                banner: resolve_nile_image(&app_id, "banner", &detail.details.background_url1),
                coverart: resolve_nile_image(&app_id, "coverart", &detail.icon_url),
                icon: resolve_nile_image(&app_id, "icon", &detail.details.logo_url),
                title,
                is_installed: really_installed,
                install_path: really_installed.then_some(install_path).flatten(),
                app_name: app_id,
            });
        }

        games.sort_by_key(|g| g.title.to_lowercase());
        tracing::info!("got {} games from nile", games.len());
        save_cached_library(&games);
        Ok(games)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_the_code_out_of_a_redirect_url() {
        assert_eq!(
            NileStore::auth_code_from(
                "https://www.amazon.com/?openid.oa2.authorization_code=ANmBLdtqoBFbtsrK&openid.mode=id_res"
            ),
            "ANmBLdtqoBFbtsrK"
        );
    }

    #[test]
    fn accepts_a_bare_code() {
        assert_eq!(
            NileStore::auth_code_from("  ANmBLdtqoBFbtsrK \n"),
            "ANmBLdtqoBFbtsrK"
        );
    }

    #[test]
    fn handles_the_code_being_the_last_param() {
        assert_eq!(
            NileStore::auth_code_from("https://x/?a=1&openid.oa2.authorization_code=ZZZ"),
            "ZZZ"
        );
    }
}
