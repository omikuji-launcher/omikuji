use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const STEAM_DATA_DIRS: &[&str] = &[
    "~/.steam/debian-installation",
    "~/.steam",
    "~/.local/share/steam",
    "~/.local/share/Steam",
    "~/snap/steam/common/.local/share/Steam", // well lutris has it... soooo
    "~/.steam/steam",
    "~/.var/app/com.valvesoftware.Steam/data/steam",
    "~/.var/app/com.valvesoftware.Steam/data/Steam", // flatpak (didnt test so yikes)
    "/usr/share/steam",
    "/usr/local/share/steam",
];

pub fn find_steam_dir() -> Option<PathBuf> {
    for dir in STEAM_DATA_DIRS {
        let expanded = shellexpand::tilde(dir);
        let path = Path::new(expanded.as_ref());
        if path.join("steamapps").exists() {
            return Some(path.to_path_buf());
        }
    }
    None
}

pub fn iter_compat_tools_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![];
    for dir in STEAM_DATA_DIRS {
        let expanded = shellexpand::tilde(dir);
        let ctd = Path::new(expanded.as_ref()).join("compatibilitytools.d");
        if ctd.is_dir() {
            dirs.push(ctd);
        }
    }
    dirs
}

pub fn get_steamapps_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![];

    if let Some(steam_dir) = find_steam_dir() {
        let main = steam_dir.join("steamapps");
        if main.exists() {
            dirs.push(main);
        }

        if let Ok(libraries) = read_library_folders(&steam_dir) {
            for (_key, entry) in libraries {
                if let Some(path_str) = entry.get("path") {
                    let lib_steamapps = Path::new(path_str).join("steamapps");
                    if lib_steamapps.exists() && !dirs.contains(&lib_steamapps) {
                        dirs.push(lib_steamapps);
                    }
                }
            }
        }
    }

    dirs
}

pub fn parse_vdf(content: &str) -> HashMap<String, VdfValue> {
    let mut result = HashMap::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            i += 1;
            continue;
        }

        if let Some(key_start) = trimmed.find('"') {
            let after_first_quote = &trimmed[key_start + 1..];
            if let Some(key_end_rel) = after_first_quote.find('"') {
                let key = after_first_quote[..key_end_rel].to_string();
                let after_key = &after_first_quote[key_end_rel + 1..].trim();

                if after_key.is_empty() || after_key.starts_with("//") {
                    i += 1;
                    while i < lines.len() {
                        let next_trimmed = lines[i].trim();
                        if !next_trimmed.is_empty() {
                            if next_trimmed.starts_with('{') {
                                i += 1;
                                let mut brace_count = 1;
                                let mut nested_content = String::new();

                                while i < lines.len() && brace_count > 0 {
                                    let line = lines[i];
                                    brace_count += line.chars().filter(|&c| c == '{').count();
                                    brace_count -= line.chars().filter(|&c| c == '}').count();
                                    if brace_count > 0 {
                                        nested_content.push_str(line);
                                        nested_content.push('\n');
                                    }
                                    i += 1;
                                }

                                let nested = parse_vdf(&nested_content);
                                result.insert(key, VdfValue::Object(nested));
                            } else if let Some(rest) = next_trimmed.strip_prefix('"') {
                                if let Some(val_end) = rest.find('"') {
                                    let val = rest[..val_end].to_string();
                                    result.insert(key, VdfValue::String(val));
                                }
                                i += 1;
                            }
                            break;
                        }
                        i += 1;
                    }
                } else if after_key.starts_with('{') {
                    i += 1;
                    let mut brace_count = 1;
                    let mut nested_content = String::new();

                    while i < lines.len() && brace_count > 0 {
                        let line = lines[i];
                        brace_count += line.chars().filter(|&c| c == '{').count();
                        brace_count -= line.chars().filter(|&c| c == '}').count();
                        if brace_count > 0 {
                            nested_content.push_str(line);
                            nested_content.push('\n');
                        }
                        i += 1;
                    }

                    let nested = parse_vdf(&nested_content);
                    result.insert(key, VdfValue::Object(nested));
                } else if let Some(rest) = after_key.strip_prefix('"') {
                    if let Some(val_end) = rest.find('"') {
                        let val = rest[..val_end].to_string();
                        result.insert(key, VdfValue::String(val));
                    }
                    i += 1;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    result
}

#[derive(Debug, Clone)]
pub enum VdfValue {
    String(String),
    Object(HashMap<String, VdfValue>),
}

impl VdfValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            VdfValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, VdfValue>> {
        match self {
            VdfValue::Object(o) => Some(o),
            _ => None,
        }
    }
}

fn read_library_folders(steam_dir: &Path) -> Result<HashMap<String, HashMap<String, String>>> {
    let path = steam_dir.join("config/libraryfolders.vdf");
    let content = fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;

    let vdf = parse_vdf(&content);
    let mut result = HashMap::new();

    if let Some(VdfValue::Object(folders)) = vdf.get("libraryfolders") {
        for (key, value) in folders {
            if let VdfValue::Object(entry) = value {
                let mut entry_map = HashMap::new();
                for (k, v) in entry {
                    if let Some(s) = v.as_str() {
                        entry_map.insert(k.clone(), s.to_string());
                    }
                }
                result.insert(key.clone(), entry_map);
            }
        }
    }

    Ok(result)
}

pub fn get_steam_users() -> Vec<SteamUser> {
    let mut users = vec![];

    let Some(steam_dir) = find_steam_dir() else {
        return users;
    };

    let path = steam_dir.join("config/loginusers.vdf");
    let Ok(content) = fs::read_to_string(&path) else {
        return users;
    };

    let vdf = parse_vdf(&content);

    if let Some(VdfValue::Object(users_obj)) = vdf.get("users") {
        for (steamid64, user_data) in users_obj {
            if let VdfValue::Object(user) = user_data {
                let account_name = user.get("AccountName")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let persona_name = user.get("PersonaName")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let most_recent = user.get("mostrecent")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "1")
                    .unwrap_or(false);

                users.push(SteamUser {
                    steamid64: steamid64.clone(),
                    account_name,
                    persona_name,
                    most_recent,
                });
            }
        }
    }

    users.sort_by_key(|u| std::cmp::Reverse(u.most_recent));
    users
}

#[derive(Debug, Clone)]
pub struct SteamUser {
    pub steamid64: String,
    pub account_name: String,
    pub persona_name: String,
    pub most_recent: bool,
}

pub fn get_active_steamid64() -> Option<String> {
    let users = get_steam_users();
    users.first().map(|u| u.steamid64.clone())
}

const STEAMID64_BASE: u64 = 76561197960265728;

pub fn steamid64_to_steamid32(steamid64: &str) -> Option<String> {
    steamid64
        .parse::<u64>()
        .ok()
        .map(|id| (id - STEAMID64_BASE).to_string())
}

#[derive(Debug, Clone)]
pub struct AppManifest {
    pub appid: String,
    pub name: String,
    pub installdir: String,
    pub state_flags: u32,
}

impl AppManifest {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;

        let vdf = parse_vdf(&content);

        let app_state = vdf.get("AppState")
            .and_then(|v| v.as_object())
            .ok_or_else(|| anyhow::anyhow!("no AppState in manifest"))?;

        let appid = app_state.get("appid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let name = app_state.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let installdir = app_state.get("installdir")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let state_flags = app_state.get("StateFlags")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        Ok(AppManifest {
            appid,
            name,
            installdir,
            state_flags,
        })
    }

    pub fn is_installed(&self) -> bool {
        // bit 2 = fully installed per steam's StateFlags enum
        self.state_flags & 4 == 4
    }
}

const NON_GAME_KEYWORDS: &[&str] = &[
    "proton",
    "steam linux runtime",
    "steamworks",
    "steamvr",
    "steam link",
    "steam audio",
    "steam 360",
    "steam input",
    "steam controller",
    "easyanticheat",
    "battleye",
    "openxr",
    "compatibility layer",
];

fn is_non_game(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    NON_GAME_KEYWORDS.iter().any(|kw| name_lower.contains(kw))
}

pub fn get_installed_games() -> Vec<AppManifest> {
    let mut games = vec![];

    for steamapps_dir in get_steamapps_dirs() {
        let Ok(entries) = fs::read_dir(&steamapps_dir) else { continue };

        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if name_str.starts_with("appmanifest_") && name_str.ends_with(".acf")
                && let Ok(manifest) = AppManifest::from_file(&entry.path())
                    && manifest.is_installed()
                        && !manifest.appid.is_empty()
                        && !is_non_game(&manifest.name) {
                        games.push(manifest);
                    }
        }
    }

    games
}

pub fn find_local_library_image(appid: &str) -> Option<PathBuf> {
    let appid_dir = find_steam_dir()?
        .join("appcache")
        .join("librarycache")
        .join(appid);
    if !appid_dir.exists() {
        return None;
    }
    if let Ok(entries) = std::fs::read_dir(&appid_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let candidate = path.join("library_capsule.jpg");
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }
    let flat = appid_dir.join("library_600x900.jpg");
    if flat.exists() {
        return Some(flat);
    }
    None
}

pub fn get_game_install_dir(appid: &str) -> Option<PathBuf> {
    for steamapps_dir in get_steamapps_dirs() {
        let manifest_path = steamapps_dir.join(format!("appmanifest_{}.acf", appid));
        if let Ok(manifest) = AppManifest::from_file(&manifest_path)
            && manifest.is_installed() && !manifest.installdir.is_empty() {
                // installdir is relative to steamapps/common
                let install_path = steamapps_dir.join("common").join(&manifest.installdir);
                if install_path.exists() {
                    return Some(install_path);
                }
            }
    }
    None
}

// compatdata is created on first launch, so None here means the game hasn't
// been run yet (or isnt a steam game at all).
pub fn find_steam_prefix(appid: &str) -> Option<PathBuf> {
    for steamapps_dir in get_steamapps_dirs() {
        let pfx = steamapps_dir
            .join("compatdata")
            .join(appid)
            .join("pfx");
        if pfx.exists() {
            return Some(pfx);
        }
    }
    None
}

pub fn find_steam_proton_version(appid: &str) -> Option<String> {
    for steamapps_dir in get_steamapps_dirs() {
        let f = steamapps_dir.join("compatdata").join(appid).join("version");
        if let Ok(s) = std::fs::read_to_string(&f) {
            let name = s.trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

// validate like lutris, steam makes empty placeholder folders, tsk.
pub fn is_proton_install(path: &Path) -> bool {
    path.join("proton").is_file()
}

pub fn iter_steam_protons() -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    for ctd in iter_compat_tools_dirs() {
        push_protons_from(&ctd, &mut out);
    }
    for dir in get_steamapps_dirs() {
        push_protons_from(&dir.join("common"), &mut out);
    }
    out
}

fn push_protons_from(parent: &Path, out: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(parent) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if is_proton_install(&p)
            && let Some(name) = p.file_name().and_then(|n| n.to_str()) {
            out.push((name.to_string(), p));
        }
    }
}

pub fn proton_display_name(dir: &Path) -> Option<String> {
    let content = fs::read_to_string(dir.join("compatibilitytool.vdf")).ok()?;
    let vdf = parse_vdf(&content);
    let tools = vdf
        .get("compatibilitytools")?
        .as_object()?
        .get("compat_tools")?
        .as_object()?;
    let (_, tool) = tools.iter().next()?;
    tool.as_object()?
        .get("display_name")?
        .as_str()
        .map(str::to_string)
}

pub fn find_proton_install(name: &str) -> Option<PathBuf> {
    let all = iter_steam_protons();
    if let Some((_, p)) = all.iter().find(|(n, _)| n == name) {
        return Some(p.clone());
    }
    if let Some(mm) = proton_version_major_minor(name) {
        let derived = format!("Proton {}", mm);
        if let Some((_, p)) = all.into_iter().find(|(n, _)| n == &derived) {
            return Some(p);
        }
    }
    None
}

// steam's internal build IDs look like "8.0-103" or "10.1000-200".
// minor "1000" is steam's sentinel for "no minor", displayed as "0". ?????
fn proton_version_major_minor(s: &str) -> Option<String> {
    let base = s.split('-').next()?;
    let mut parts = base.split('.');
    let major = parts.next()?;
    let minor = parts.next()?;
    if major.is_empty() || !major.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let minor_display = if minor == "1000" { "0" } else { minor };
    Some(format!("{}.{}", major, minor_display))
}

// GE-Proton first, ge proton = cool
pub fn default_proton_install() -> Option<PathBuf> {
    let mut all = iter_steam_protons();
    all.sort_by(|a, b| {
        let a_ge = a.0.starts_with("GE-Proton");
        let b_ge = b.0.starts_with("GE-Proton");
        b_ge.cmp(&a_ge).then_with(|| b.0.cmp(&a.0))
    });
    all.into_iter().next().map(|(_, p)| p)
}

pub fn resolve_or_default_proton(name: Option<&str>) -> Option<PathBuf> {
    if let Some(n) = name
        && let Some(p) = find_proton_install(n) {
            return Some(p);
        }
    default_proton_install()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vdf_simple() {
        let content = r#"
            "key1" "value1"
            "key2" "value2"
        "#;

        let vdf = parse_vdf(content);
        assert_eq!(vdf.get("key1").and_then(|v| v.as_str()), Some("value1"));
        assert_eq!(vdf.get("key2").and_then(|v| v.as_str()), Some("value2"));
    }

    #[test]
    fn test_parse_vdf_nested() {
        let content = r#"
            "users"
            {
                "12345"
                {
                    "name" "test"
                }
            }
        "#;

        let vdf = parse_vdf(content);
        let users = vdf.get("users").and_then(|v| v.as_object());
        assert!(users.is_some());
    }

    #[test]
    fn test_steamid_conversion() {
        assert_eq!(steamid64_to_steamid32("76561197960287930").as_deref(), Some("22202"));
        assert_eq!(steamid64_to_steamid32("not a number"), None);
    }
}
