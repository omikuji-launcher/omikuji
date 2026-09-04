use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum MediaType {
    Banner,
    Coverart,
    Icon,
}

impl MediaType {
    pub fn suffix(&self) -> &str {
        match self {
            MediaType::Banner => "banner",
            MediaType::Coverart => "coverart",
            MediaType::Icon => "icon",
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            MediaType::Icon => "png",
            _ => "jpg",
        }
    }

    pub fn from_suffix(suffix: &str) -> Option<Self> {
        ALL_TYPES.into_iter().find(|t| t.suffix() == suffix)
    }

    fn sgdb_endpoint(&self) -> &'static str {
        match self {
            MediaType::Banner => HERO_ENDPOINT,
            MediaType::Coverart => GRID_ENDPOINT,
            MediaType::Icon => ICON_ENDPOINT,
        }
    }

    fn sgdb_query(&self) -> &'static [(&'static str, &'static str)] {
        match self {
            MediaType::Coverart => &GRID_QUERY,
            MediaType::Banner => &[],
            MediaType::Icon => &ICON_QUERY,
        }
    }
}

pub const ALL_TYPES: [MediaType; 3] = [MediaType::Banner, MediaType::Coverart, MediaType::Icon];

#[derive(Debug, Clone, Copy)]
pub enum MediaSlot {
    Live,
    Pending,
}

impl MediaSlot {
    fn infix(&self) -> &str {
        match self {
            MediaSlot::Live => "",
            MediaSlot::Pending => ".pending",
        }
    }
}

const SGDB_BASE: &str = "https://www.steamgriddb.com/api/v2";
const ICON_ENDPOINT: &str = "icons";
const GRID_ENDPOINT: &str = "grids";
const HERO_ENDPOINT: &str = "heroes";
const ICON_DIMENSION: &str = "512";
const ICON_QUERY: [(&str, &str); 1] = [("dimensions", ICON_DIMENSION)];
const GRID_QUERY: [(&str, &str); 1] = [("dimensions", "600x900")];
const ICO_MIME: &str = "image/vnd.microsoft.icon";
const SGDB_API_KEY: &str = "b0e57477a2e9665d6e1789d72cf0f334";

#[derive(Debug, Deserialize)]
struct SgdbResponse<T> {
    success: bool,
    data: Option<T>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SgdbGame {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub verified: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SgdbAsset {
    pub url: String,
    #[serde(default)]
    pub thumb: String,
    #[serde(default)]
    pub mime: String,
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
}

impl SgdbAsset {
    fn is_ico(&self) -> bool {
        self.mime == ICO_MIME
    }

    fn rank(&self) -> (bool, u64) {
        (!self.is_ico(), self.width as u64 * self.height as u64)
    }
}

fn best_icon(assets: impl IntoIterator<Item = SgdbAsset>) -> Option<String> {
    assets
        .into_iter()
        .reduce(|best, a| if a.rank() > best.rank() { a } else { best })
        .map(|a| a.url)
}

pub fn slugify(name: &str) -> String {
    use unicode_normalization::UnicodeNormalization;

    let nfd: String = name.nfd().collect();
    let ascii: String = nfd.chars().filter(|c| c.is_ascii()).collect();
    let lower = ascii.to_lowercase();

    let cleaned: String = lower
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-')
        .collect();

    // collapse whitespace/dashes into single dashes
    let mut slug = String::new();
    let mut prev_dash = true;
    for c in cleaned.chars() {
        if c == ' ' || c == '-' {
            if !prev_dash {
                slug.push('-');
                prev_dash = true;
            }
        } else {
            slug.push(c);
            prev_dash = false;
        }
    }

    if slug.ends_with('-') {
        slug.pop();
    }

    slug
}

fn cache_dir() -> PathBuf {
    crate::cache_dir().join("images")
}

pub fn media_path_in(slot: MediaSlot, game_id: &str, media_type: &MediaType) -> PathBuf {
    cache_dir().join(format!(
        "{}_{}{}.{}",
        game_id,
        media_type.suffix(),
        slot.infix(),
        media_type.extension()
    ))
}

pub fn media_path(game_id: &str, media_type: &MediaType) -> PathBuf {
    media_path_in(MediaSlot::Live, game_id, media_type)
}

pub fn commit_pending(game_id: &str) {
    for media_type in ALL_TYPES {
        let pending = media_path_in(MediaSlot::Pending, game_id, &media_type);
        if pending.is_file() {
            let live = media_path_in(MediaSlot::Live, game_id, &media_type);
            if let Err(e) = fs::rename(&pending, &live) {
                tracing::error!("committing {}: {}", pending.display(), e);
            }
        }
    }
}

pub fn discard_pending(game_id: &str) {
    for media_type in ALL_TYPES {
        let _ = fs::remove_file(media_path_in(MediaSlot::Pending, game_id, &media_type));
    }
}

pub fn resolve_image(game_id: &str, manual_override: &str, media_type: &MediaType) -> String {
    if !manual_override.is_empty() {
        return to_qml_url(&crate::template_vars::TemplateVars::global().expand(manual_override));
    }

    for slot in [MediaSlot::Pending, MediaSlot::Live] {
        let path = media_path_in(slot, game_id, media_type);
        if path.exists() {
            return format!("file://{}", path.to_string_lossy());
        }
    }

    String::new()
}

fn to_qml_url(s: &str) -> String {
    if s.starts_with("file://") || s.starts_with("http://") || s.starts_with("https://") {
        s.to_string()
    } else if s.starts_with('/') {
        format!("file://{}", s)
    } else {
        s.to_string()
    }
}
pub fn fetch_media_blocking_with<F>(
    slot: MediaSlot,
    game_id: &str,
    game_name: &str,
    mut on_asset: F,
) -> FetchResult
where
    F: FnMut(&MediaType),
{
    let mut result = FetchResult::default();

    let sgdb_id = match sgdb_search(game_name) {
        Ok(Some(id)) => id,
        Ok(None) => {
            tracing::warn!("sgdb: no match for '{}'", game_name);
            return result;
        }
        Err(e) => {
            tracing::error!("sgdb search failed for '{}': {}", game_name, e);
            return result;
        }
    };

    let dir = cache_dir();
    if let Err(e) = fs::create_dir_all(&dir) {
        tracing::error!("failed to create cache dir: {}", e);
        return result;
    }

    // coverart first, card binds to this and it's ~5x smaller than hero
    for media_type in [MediaType::Coverart, MediaType::Banner, MediaType::Icon] {
        let endpoint = media_type.sgdb_endpoint();
        let url = match sgdb_best_asset(&media_type, sgdb_id) {
            Ok(Some(u)) => u,
            Ok(None) => {
                tracing::debug!("sgdb {} no data for game {}", endpoint, sgdb_id);
                continue;
            }
            Err(e) => {
                tracing::error!("sgdb {} lookup failed: {}", endpoint, e);
                continue;
            }
        };
        tracing::debug!("sgdb {} -> {}", endpoint, url);
        let dest = media_path_in(slot, game_id, &media_type);
        match download_blocking(&url, &dest) {
            Ok(n) => {
                tracing::debug!("sgdb {} wrote {} bytes -> {}", endpoint, n, dest.display());
                match media_type {
                    MediaType::Banner => result.banner = Some(dest.clone()),
                    MediaType::Coverart => result.coverart = Some(dest.clone()),
                    MediaType::Icon => result.icon = Some(dest.clone()),
                }
                on_asset(&media_type);
            }
            Err(e) => tracing::error!("sgdb {} download failed: {}", endpoint, e),
        }
    }

    result
}

fn sgdb_get<T: serde::de::DeserializeOwned>(url: reqwest::Url) -> Result<T> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("omikuji")
        .build()
        .context("building sgdb client")?;

    let resp = client
        .get(url.clone())
        .bearer_auth(SGDB_API_KEY)
        .send()
        .with_context(|| format!("requesting {}", url))?;

    if !resp.status().is_success() {
        anyhow::bail!("sgdb returned {} for {}", resp.status(), url);
    }

    resp.json::<T>()
        .with_context(|| format!("parsing sgdb response from {}", url))
}

pub fn sgdb_icon_url(name: &str) -> Result<Option<String>> {
    let Some(id) = sgdb_search(name)? else {
        return Ok(None);
    };
    let web_safe = sgdb_icon_assets(id)?.into_iter().filter(|a| !a.is_ico());
    Ok(best_icon(web_safe))
}

pub fn sgdb_search_games(name: &str) -> Result<Vec<SgdbGame>> {
    let mut url = reqwest::Url::parse(SGDB_BASE).unwrap();
    url.path_segments_mut()
        .unwrap()
        .extend(["search", "autocomplete", name]);
    let resp: SgdbResponse<Vec<SgdbGame>> = sgdb_get(url)?;
    if !resp.success {
        anyhow::bail!("sgdb search api reported failure for '{}'", name);
    }

    let needle = name.to_lowercase();
    let mut games = resp.data.unwrap_or_default();
    games.sort_by_key(|g| std::cmp::Reverse((g.name.to_lowercase() == needle, g.verified)));
    Ok(games)
}

fn sgdb_search(name: &str) -> Result<Option<u64>> {
    let games = sgdb_search_games(name)?;
    let Some(pick) = games.first() else {
        return Ok(None);
    };

    tracing::debug!(
        "sgdb match for '{}' -> '{}' (id {}, verified {})",
        name,
        pick.name,
        pick.id,
        pick.verified
    );
    Ok(Some(pick.id))
}

fn sgdb_assets(endpoint: &str, game_id: u64, query: &[(&str, &str)]) -> Result<Vec<SgdbAsset>> {
    let mut url = reqwest::Url::parse(SGDB_BASE).unwrap();
    url.path_segments_mut()
        .unwrap()
        .extend([endpoint, "game", &game_id.to_string()]);
    if !query.is_empty() {
        url.query_pairs_mut().extend_pairs(query.iter().copied());
    }
    let resp: SgdbResponse<Vec<SgdbAsset>> = sgdb_get(url)?;
    if !resp.success {
        anyhow::bail!(
            "sgdb {} api reported failure for game {}",
            endpoint,
            game_id
        );
    }
    Ok(resp.data.unwrap_or_default())
}

pub fn sgdb_assets_for(media_type: &MediaType, game_id: u64) -> Result<Vec<SgdbAsset>> {
    match media_type {
        MediaType::Icon => sgdb_icon_assets(game_id),
        _ => sgdb_assets(media_type.sgdb_endpoint(), game_id, media_type.sgdb_query()),
    }
}

fn sgdb_best_asset(media_type: &MediaType, game_id: u64) -> Result<Option<String>> {
    let assets = sgdb_assets_for(media_type, game_id)?;
    Ok(match media_type {
        MediaType::Icon => best_icon(assets),
        _ => assets.into_iter().next().map(|a| a.url),
    })
}

fn sgdb_icon_assets(game_id: u64) -> Result<Vec<SgdbAsset>> {
    let preferred = sgdb_assets(ICON_ENDPOINT, game_id, &ICON_QUERY)?;
    if !preferred.is_empty() {
        return Ok(preferred);
    }
    tracing::debug!(
        "sgdb icons: no {}px asset for game {}, retrying unfiltered",
        ICON_DIMENSION,
        game_id
    );
    sgdb_assets(ICON_ENDPOINT, game_id, &[])
}

fn download_blocking(url: &str, dest: &PathBuf) -> Result<usize> {
    let resp = reqwest::blocking::get(url).with_context(|| format!("downloading {}", url))?;

    if !resp.status().is_success() {
        anyhow::bail!("image download failed: {} for {}", resp.status(), url);
    }

    let bytes = resp.bytes()?;
    if bytes.is_empty() {
        anyhow::bail!("empty response from {}", url);
    }

    fs::write(dest, &bytes)?;
    Ok(bytes.len())
}

pub fn download_into(
    slot: MediaSlot,
    game_id: &str,
    media_type: &MediaType,
    url: &str,
) -> Result<usize> {
    fs::create_dir_all(cache_dir()).context("creating media cache dir")?;
    download_blocking(url, &media_path_in(slot, game_id, media_type))
}

#[derive(Debug, Default)]
pub struct FetchResult {
    pub banner: Option<PathBuf>,
    pub coverart: Option<PathBuf>,
    pub icon: Option<PathBuf>,
}

pub fn fetch_steam_media_blocking_with<F>(
    slot: MediaSlot,
    appid: &str,
    mut on_asset: F,
) -> FetchResult
where
    F: FnMut(&MediaType),
{
    let mut result = FetchResult::default();

    let dir = cache_dir();
    if let Err(e) = fs::create_dir_all(&dir) {
        tracing::error!("failed to create cache dir: {}", e);
        return result;
    }

    let tasks = vec![
        (
            MediaType::Coverart,
            format!(
                "https://cdn.akamai.steamstatic.com/steam/apps/{}/library_600x900.jpg",
                appid
            ),
        ),
        (
            MediaType::Banner,
            format!(
                "https://cdn.akamai.steamstatic.com/steam/apps/{}/header.jpg",
                appid
            ),
        ),
    ];

    for (media_type, url) in tasks {
        let dest = media_path_in(slot, appid, &media_type);

        match download_blocking(&url, &dest) {
            Ok(_) => {
                match media_type {
                    MediaType::Banner => result.banner = Some(dest),
                    MediaType::Coverart => result.coverart = Some(dest),
                    MediaType::Icon => result.icon = Some(dest),
                }
                on_asset(&media_type);
            }
            Err(e) => tracing::error!("steam {} download failed: {}", media_type.suffix(), e),
        }
    }

    result
}

pub fn remove_cached_media(game_id: &str) {
    for media_type in ALL_TYPES {
        let path = media_path(game_id, &media_type);
        if path.exists() {
            if let Err(e) = fs::remove_file(&path) {
                tracing::warn!("failed to remove cached {}: {}", media_type.suffix(), e);
            } else {
                tracing::debug!(
                    "removed cached {} for game {}",
                    media_type.suffix(),
                    game_id
                );
            }
        }
    }
}

// in-flight guard: store apis can hand the same image to multiple siblings
// during a refresh, so only one outstanding fetch per key
pub fn fetch_cached_image(cache_path: &std::path::Path, url: &str, key: String) -> Option<String> {
    static INFLIGHT: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

    if cache_path.exists() {
        return Some(format!("file://{}", cache_path.display()));
    }
    {
        let mut guard = INFLIGHT.lock().unwrap();
        if guard.iter().any(|k| k == &key) {
            return Some(url.to_string());
        }
        guard.push(key.clone());
    }

    let path = cache_path.to_path_buf();
    let fetch_url = url.to_string();
    tokio::spawn(async move {
        match crate::http::client().get(&fetch_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(bytes) = resp.bytes().await
                    && let Err(e) = crate::fs_util::write_atomic(&path, &bytes)
                {
                    tracing::error!("image cache write failed {}: {}", path.display(), e);
                }
            }
            Ok(resp) => tracing::warn!("image fetch {} returned {}", fetch_url, resp.status()),
            Err(e) => tracing::error!("image fetch failed {}: {}", fetch_url, e),
        }
        let mut guard = INFLIGHT.lock().unwrap();
        guard.retain(|k| k != &key);
    });

    Some(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Honkai: Star Rail"), "honkai-star-rail");
        assert_eq!(
            slugify("The Witcher 3: Wild Hunt"),
            "the-witcher-3-wild-hunt"
        );
        assert_eq!(slugify("DOOM"), "doom");
        assert_eq!(slugify("Half-Life 2"), "half-life-2");
        assert_eq!(slugify("  spaces  everywhere  "), "spaces-everywhere");
        assert_eq!(slugify("Nier: Automata™"), "nier-automata");
    }

    #[test]
    fn test_media_path() {
        let path = media_path("abc123", &MediaType::Coverart);
        assert!(path.to_string_lossy().contains("abc123_coverart.jpg"));

        let path = media_path("abc123", &MediaType::Icon);
        assert!(path.to_string_lossy().contains("abc123_icon.png"));
    }

    #[test]
    fn test_resolve_image_manual_override() {
        let result = resolve_image("abc123", "/custom/path.jpg", &MediaType::Coverart);
        assert_eq!(result, "file:///custom/path.jpg");

        let result = resolve_image("abc123", "file:///already/url.jpg", &MediaType::Coverart);
        assert_eq!(result, "file:///already/url.jpg");

        let result = resolve_image("abc123", "https://cdn.example/x.jpg", &MediaType::Coverart);
        assert_eq!(result, "https://cdn.example/x.jpg");
    }

    #[test]
    fn test_resolve_image_empty() {
        let result = resolve_image("nonexistent_id", "", &MediaType::Coverart);
        assert_eq!(result, "");
    }
}
