use anyhow::{Context, Result};
use serde::Deserialize;
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
}

pub const ALL_TYPES: [MediaType; 3] = [MediaType::Banner, MediaType::Coverart, MediaType::Icon];

const SGDB_BASE: &str = "https://www.steamgriddb.com/api/v2";
const SGDB_API_KEY: &str = "b0e57477a2e9665d6e1789d72cf0f334";

#[derive(Debug, Deserialize)]
struct SgdbResponse<T> {
    success: bool,
    data: Option<T>,
}

#[derive(Debug, Deserialize)]
struct SgdbGame {
    id: u64,
    name: String,
    #[serde(default)]
    verified: bool,
}

#[derive(Debug, Deserialize)]
struct SgdbAsset {
    url: String,
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

pub fn media_path(game_id: &str, media_type: &MediaType) -> PathBuf {
    cache_dir().join(format!(
        "{}_{}.{}",
        game_id,
        media_type.suffix(),
        media_type.extension()
    ))
}

pub fn resolve_image(game_id: &str, manual_override: &str, media_type: &MediaType) -> String {
    if !manual_override.is_empty() {
        return to_qml_url(manual_override);
    }

    let path = media_path(game_id, media_type);
    if path.exists() {
        return format!("file://{}", path.to_string_lossy());
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
pub fn fetch_media_blocking(game_id: &str, game_name: &str) -> FetchResult {
    fetch_media_blocking_with(game_id, game_name, |_| {})
}

pub fn fetch_media_blocking_with<F>(game_id: &str, game_name: &str, mut on_asset: F) -> FetchResult
where
    F: FnMut(&MediaType),
{
    let mut result = FetchResult::default();

    let sgdb_id = match sgdb_search(game_name) {
        Ok(Some(id)) => id,
        Ok(None) => {
            eprintln!("sgdb: no match for '{}'", game_name);
            return result;
        }
        Err(e) => {
            eprintln!("sgdb search failed for '{}': {}", game_name, e);
            return result;
        }
    };

    let dir = cache_dir();
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!("failed to create cache dir: {}", e);
        return result;
    }

    // coverart first, card binds to this and it's ~5x smaller than hero
    let tasks: Vec<(MediaType, &str, Vec<(&str, &str)>)> = vec![
        (MediaType::Coverart, "grids", vec![("dimensions", "600x900")]),
        (MediaType::Banner, "heroes", vec![]),
        (MediaType::Icon, "icons", vec![]),
    ];

    for (media_type, endpoint, query) in tasks {
        let url = match sgdb_first_asset(endpoint, sgdb_id, &query) {
            Ok(Some(u)) => u,
            Ok(None) => {
                eprintln!("sgdb {} no data for game {}", endpoint, sgdb_id);
                continue;
            }
            Err(e) => {
                eprintln!("sgdb {} lookup failed: {}", endpoint, e);
                continue;
            }
        };
        eprintln!("sgdb {} -> {}", endpoint, url);
        let dest = media_path(game_id, &media_type);
        match download_blocking(&url, &dest) {
            Ok(n) => {
                eprintln!("sgdb {} wrote {} bytes -> {}", endpoint, n, dest.display());
                match media_type {
                    MediaType::Banner => result.banner = Some(dest.clone()),
                    MediaType::Coverart => result.coverart = Some(dest.clone()),
                    MediaType::Icon => result.icon = Some(dest.clone()),
                }
                on_asset(&media_type);
            }
            Err(e) => eprintln!("sgdb {} download failed: {}", endpoint, e),
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
    let Some(id) = sgdb_search(name)? else { return Ok(None); };
    let mut url = reqwest::Url::parse(SGDB_BASE).unwrap();
    url.path_segments_mut().unwrap().extend(["icons", "game", &id.to_string()]);
    let resp: SgdbResponse<Vec<SgdbAsset>> = sgdb_get(url)?;
    if !resp.success {
        anyhow::bail!("sgdb icons api reported failure for game {}", id);
    }
    let Some(assets) = resp.data else { return Ok(None) };
    let pick = assets.into_iter().find(|a| {
        let lower = a.url.to_lowercase();
        lower.ends_with(".png")
            || lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
            || lower.ends_with(".webp")
            || lower.ends_with(".gif")
    });
    Ok(pick.map(|a| a.url))
}

fn sgdb_search(name: &str) -> Result<Option<u64>> {
    let mut url = reqwest::Url::parse(SGDB_BASE).unwrap();
    url.path_segments_mut()
        .unwrap()
        .extend(["search", "autocomplete", name]);
    let resp: SgdbResponse<Vec<SgdbGame>> = sgdb_get(url)?;
    if !resp.success {
        anyhow::bail!("sgdb search api reported failure for '{}'", name);
    }
    let Some(games) = resp.data else { return Ok(None) };
    if games.is_empty() {
        return Ok(None);
    }

    let needle = name.to_lowercase();
    let pick = games
        .iter()
        .find(|g| g.name.to_lowercase() == needle && g.verified)
        .or_else(|| games.iter().find(|g| g.name.to_lowercase() == needle))
        .or_else(|| games.iter().find(|g| g.verified))
        .unwrap_or(&games[0]);

    eprintln!("sgdb match for '{}' -> '{}' (id {}, verified {})", name, pick.name, pick.id, pick.verified);
    Ok(Some(pick.id))
}

fn sgdb_first_asset(endpoint: &str, game_id: u64, query: &[(&str, &str)]) -> Result<Option<String>> {
    let mut url = reqwest::Url::parse(SGDB_BASE).unwrap();
    url.path_segments_mut()
        .unwrap()
        .extend([endpoint, "game", &game_id.to_string()]);
    if !query.is_empty() {
        url.query_pairs_mut().extend_pairs(query.iter().copied());
    }
    let resp: SgdbResponse<Vec<SgdbAsset>> = sgdb_get(url)?;
    if !resp.success {
        anyhow::bail!("sgdb {} api reported failure for game {}", endpoint, game_id);
    }
    Ok(resp.data.and_then(|v| v.into_iter().next().map(|a| a.url)))
}

fn download_blocking(url: &str, dest: &PathBuf) -> Result<usize> {
    let resp = reqwest::blocking::get(url)
        .with_context(|| format!("downloading {}", url))?;

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

#[derive(Debug, Default)]
pub struct FetchResult {
    pub banner: Option<PathBuf>,
    pub coverart: Option<PathBuf>,
    pub icon: Option<PathBuf>,
}

pub fn fetch_steam_media_blocking(appid: &str) -> FetchResult {
    let mut result = FetchResult::default();
    
    let dir = cache_dir();
    if let Err(e) = fs::create_dir_all(&dir) {
        eprintln!("failed to create cache dir: {}", e);
        return result;
    }
    
    let tasks = vec![
        (MediaType::Coverart, format!("https://cdn.akamai.steamstatic.com/steam/apps/{}/library_600x900.jpg", appid)),
        (MediaType::Banner, format!("https://cdn.akamai.steamstatic.com/steam/apps/{}/header.jpg", appid)),
    ];
    
    for (media_type, url) in tasks {
        let dest = media_path(appid, &media_type);
        
        match download_blocking(&url, &dest) {
            Ok(_) => match media_type {
                MediaType::Banner => result.banner = Some(dest),
                MediaType::Coverart => result.coverart = Some(dest),
                MediaType::Icon => result.icon = Some(dest),
            },
            Err(e) => eprintln!("steam {} download failed: {}", media_type.suffix(), e),
        }
    }
    
    result
}

pub fn remove_cached_media(game_id: &str) {
    for media_type in ALL_TYPES {
        let path = media_path(game_id, &media_type);
        if path.exists() {
            if let Err(e) = fs::remove_file(&path) {
                eprintln!("failed to remove cached {}: {}", media_type.suffix(), e);
            } else {
                eprintln!("removed cached {} for game {}", media_type.suffix(), game_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Honkai: Star Rail"), "honkai-star-rail");
        assert_eq!(slugify("The Witcher 3: Wild Hunt"), "the-witcher-3-wild-hunt");
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
