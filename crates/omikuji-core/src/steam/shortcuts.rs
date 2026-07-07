use crate::library::Game;
use crate::media::{media_path, MediaType};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

const TYPE_OBJ: u8 = 0x00;
const TYPE_STR: u8 = 0x01;
const TYPE_INT: u8 = 0x02;
const TYPE_END: u8 = 0x08;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Int(u32),
    Obj(Vec<(String, Value)>),
}

pub type Entry = Vec<(String, Value)>;

fn get<'a>(entry: &'a Entry, key: &str) -> Option<&'a Value> {
    entry
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(key))
        .map(|(_, v)| v)
}

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl Reader<'_> {
    fn byte(&mut self) -> Result<u8> {
        let b = *self.buf.get(self.pos).context("unexpected end of vdf")?;
        self.pos += 1;
        Ok(b)
    }

    fn cstring(&mut self) -> Result<String> {
        let start = self.pos;
        let end = self.buf[start..]
            .iter()
            .position(|&b| b == 0)
            .context("unterminated string in vdf")?
            + start;
        self.pos = end + 1;
        Ok(String::from_utf8_lossy(&self.buf[start..end]).into_owned())
    }

    fn u32(&mut self) -> Result<u32> {
        let bytes: [u8; 4] = self
            .buf
            .get(self.pos..self.pos + 4)
            .context("unexpected end of vdf")?
            .try_into()?;
        self.pos += 4;
        Ok(u32::from_le_bytes(bytes))
    }

    fn object(&mut self) -> Result<Entry> {
        let mut entry = vec![];
        loop {
            let ty = self.byte()?;
            if ty == TYPE_END {
                return Ok(entry);
            }
            let key = self.cstring()?;
            let value = match ty {
                TYPE_OBJ => Value::Obj(self.object()?),
                TYPE_STR => Value::Str(self.cstring()?),
                TYPE_INT => Value::Int(self.u32()?),
                other => anyhow::bail!("unknown vdf field type {:#04x}", other),
            };
            entry.push((key, value));
        }
    }
}

fn write_object(out: &mut Vec<u8>, entry: &Entry) {
    for (key, value) in entry {
        match value {
            Value::Obj(obj) => {
                out.push(TYPE_OBJ);
                out.extend_from_slice(key.as_bytes());
                out.push(0);
                write_object(out, obj);
            }
            Value::Str(s) => {
                out.push(TYPE_STR);
                out.extend_from_slice(key.as_bytes());
                out.push(0);
                out.extend_from_slice(s.as_bytes());
                out.push(0);
            }
            Value::Int(n) => {
                out.push(TYPE_INT);
                out.extend_from_slice(key.as_bytes());
                out.push(0);
                out.extend_from_slice(&n.to_le_bytes());
            }
        }
    }
    out.push(TYPE_END);
}

fn parse_entries(buf: &[u8]) -> Result<Vec<Entry>> {
    let mut reader = Reader { buf, pos: 0 };
    let root = reader.object()?;

    let mut entries = vec![];
    if let Some(Value::Obj(list)) = get(&root, "shortcuts") {
        for (_, v) in list {
            if let Value::Obj(e) = v {
                entries.push(e.clone());
            }
        }
    }
    Ok(entries)
}

fn serialize_entries(entries: &[Entry]) -> Vec<u8> {
    let list: Entry = entries
        .iter()
        .enumerate()
        .map(|(i, e)| (i.to_string(), Value::Obj(e.clone())))
        .collect();
    let root: Entry = vec![("shortcuts".to_string(), Value::Obj(list))];

    let mut out = vec![];
    write_object(&mut out, &root);
    out
}

fn read_entries(path: &Path) -> Result<Vec<Entry>> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let buf = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    parse_entries(&buf)
}

fn write_entries(path: &Path, entries: &[Entry]) -> Result<()> {
    crate::fs_util::write_atomic(path, serialize_entries(entries))
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

fn user_config_dir() -> Option<PathBuf> {
    let userdata = super::local::find_steam_dir()?.join("userdata");
    let ids: Vec<String> = fs::read_dir(&userdata)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n != "0" && n.chars().all(|c| c.is_ascii_digit()))
        .collect();

    let active = super::local::get_active_steamid64()
        .and_then(|id| super::local::steamid64_to_steamid32(&id));

    let id = match active {
        Some(id) if ids.contains(&id) => id,
        _ => ids.into_iter().next()?,
    };

    Some(userdata.join(id).join("config"))
}

pub fn available() -> bool {
    user_config_dir().is_some()
}

fn launch_spec(game: &Game) -> (String, String) {
    let target = crate::desktop::launch_target(game);
    if let Ok(app_id) = std::env::var("FLATPAK_ID") {
        (
            "/usr/bin/flatpak".to_string(),
            format!("run {} run {}", app_id, target),
        )
    } else {
        let exe = std::env::current_exe()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "omikuji".to_string());
        (exe, format!("run {}", target))
    }
}

fn matches_game(entry: &Entry, game: &Game) -> bool {
    let Some(Value::Str(options)) = get(entry, "LaunchOptions") else {
        return false;
    };
    let suffix = format!("_{}", game.metadata.id);
    options
        .split_whitespace()
        .last()
        .is_some_and(|t| t.ends_with(&suffix))
}

pub fn shortcut_exists(game: &Game) -> bool {
    let Some(config) = user_config_dir() else {
        return false;
    };
    read_entries(&config.join("shortcuts.vdf"))
        .map(|entries| entries.iter().any(|e| matches_game(e, game)))
        .unwrap_or(false)
}

pub fn create_shortcut(game: &Game) -> Result<PathBuf> {
    let config = user_config_dir().context("no steam user data found")?;
    let path = config.join("shortcuts.vdf");
    let mut entries = read_entries(&path)?;
    entries.retain(|e| !matches_game(e, game));

    let (exe, options) = launch_spec(game);
    let quoted_exe = format!("\"{}\"", exe);
    let appid =
        crc32fast::hash(format!("{}{}", quoted_exe, game.metadata.name).as_bytes()) | 0x8000_0000;

    let start_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/"))
        .display()
        .to_string();

    let icon = [MediaType::Icon, MediaType::Coverart]
        .iter()
        .map(|t| media_path(&game.metadata.id, t))
        .find(|p| p.exists())
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    entries.push(vec![
        ("appid".to_string(), Value::Int(appid)),
        ("AppName".to_string(), Value::Str(game.metadata.name.clone())),
        ("Exe".to_string(), Value::Str(quoted_exe)),
        ("StartDir".to_string(), Value::Str(format!("\"{}\"", start_dir))),
        ("icon".to_string(), Value::Str(icon)),
        ("LaunchOptions".to_string(), Value::Str(options)),
        ("IsHidden".to_string(), Value::Int(0)),
        ("AllowDesktopConfig".to_string(), Value::Int(1)),
        ("AllowOverlay".to_string(), Value::Int(1)),
        ("OpenVR".to_string(), Value::Int(0)),
        ("Devkit".to_string(), Value::Int(0)),
        ("DevkitOverrideAppID".to_string(), Value::Int(0)),
        ("LastPlayTime".to_string(), Value::Int(0)),
        ("tags".to_string(), Value::Obj(vec![])),
    ]);

    write_entries(&path, &entries)?;
    set_artwork(&config, appid, game);
    Ok(path)
}

pub fn remove_shortcut(game: &Game) -> Result<()> {
    let config = user_config_dir().context("no steam user data found")?;
    let path = config.join("shortcuts.vdf");
    let mut entries = read_entries(&path)?;
    let before = entries.len();
    entries.retain(|e| !matches_game(e, game));
    if entries.len() == before {
        return Ok(());
    }
    write_entries(&path, &entries)
}

fn set_artwork(config: &Path, appid: u32, game: &Game) {
    let grid = config.join("grid");
    if let Err(e) = fs::create_dir_all(&grid) {
        tracing::warn!("creating {} failed: {}", grid.display(), e);
        return;
    }

    let banner = media_path(&game.metadata.id, &MediaType::Banner);
    let cover = media_path(&game.metadata.id, &MediaType::Coverart);
    let icon = media_path(&game.metadata.id, &MediaType::Icon);

    let assets = [
        (&banner, grid.join(format!("{}.jpg", appid))),
        (&banner, grid.join(format!("{}_hero.jpg", appid))),
        (&cover, grid.join(format!("{}p.jpg", appid))),
        (&icon, grid.join(format!("{}_icon.png", appid))),
    ];

    for (source, target) in assets {
        if source.exists() && !target.exists()
            && let Err(e) = fs::copy(source, &target) {
                tracing::warn!("steam artwork copy to {} failed: {}", target.display(), e);
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(name: &str, options: &str) -> Entry {
        vec![
            ("appid".to_string(), Value::Int(0x8000_0001)),
            ("AppName".to_string(), Value::Str(name.to_string())),
            ("Exe".to_string(), Value::Str("\"/usr/bin/omikuji\"".to_string())),
            ("LaunchOptions".to_string(), Value::Str(options.to_string())),
            ("LastPlayTime".to_string(), Value::Int(0)),
            (
                "tags".to_string(),
                Value::Obj(vec![("0".to_string(), Value::Str("favorite".to_string()))]),
            ),
        ]
    }

    #[test]
    fn test_vdf_roundtrip() {
        let entries = vec![
            sample_entry("Elden Ring", "run elden-ring_abc123"),
            sample_entry("Sekiro", "run sekiro_xyz789"),
        ];
        let bytes = serialize_entries(&entries);
        let parsed = parse_entries(&bytes).unwrap();
        assert_eq!(parsed, entries);
    }

    #[test]
    fn test_vdf_empty() {
        let bytes = serialize_entries(&[]);
        assert_eq!(parse_entries(&bytes).unwrap(), Vec::<Entry>::new());
    }

    #[test]
    fn test_matches_game() {
        let mut game = Game::new("Test".to_string(), std::path::PathBuf::from("/g/t.exe"));
        game.metadata.id = "abc123".to_string();

        assert!(matches_game(&sample_entry("Test", "run test_abc123"), &game));
        assert!(matches_game(
            &sample_entry("Test", "run io.github.omikuji run test_abc123"),
            &game
        ));
        assert!(!matches_game(&sample_entry("Other", "run other_zzz999"), &game));
        assert!(!matches_game(&vec![], &game));
    }

    #[test]
    fn test_get_case_insensitive() {
        let entry = sample_entry("Test", "run test_abc123");
        assert!(matches!(get(&entry, "appname"), Some(Value::Str(_))));
        assert!(matches!(get(&entry, "LASTPLAYTIME"), Some(Value::Int(0))));
        assert!(get(&entry, "missing").is_none());
    }
}
