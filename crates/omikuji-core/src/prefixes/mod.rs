pub mod registry;

use crate::launch::prefix_path_for;
use crate::library::Library;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

const WINE_DLL_STAMPS: [&[u8]; 2] = [b"Wine builtin DLL\0", b"Wine placeholder DLL\0"];
const DOS_HEADER_LEN: usize = 0x40;
const STAMPED_HEADER_LEN: usize = DOS_HEADER_LEN + 32;

pub struct PrefixInfo {
    pub path: PathBuf,
    pub name: String,
    pub games: Vec<String>,
    pub runner: String,
}

struct Acc {
    display: PathBuf,
    games: Vec<String>,
    runner: String,
}

fn canonical(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

pub fn windows_dir(prefix: &Path) -> PathBuf {
    prefix.join("drive_c").join("windows")
}

pub fn system32_dir(prefix: &Path) -> PathBuf {
    windows_dir(prefix).join("system32")
}

pub fn syswow64_dir(prefix: &Path) -> PathBuf {
    windows_dir(prefix).join("syswow64")
}

// same test as is_fake_dll in wine's dlls/setupapi/fakedll.c, winebuild stamps every dll wine ships so anything unstamped came from somewhere else
/*

Builtin:
000000 4d 5a 90 00 03 00 00 00 04 00 00 00 ff ff 00 00  >MZ..............<
000010 b8 00 00 00 00 00 00 00 40 00 00 00 00 00 00 00  >........@.......<
000020 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  >................<
000030 00 00 00 00 00 00 00 00 00 00 00 00 80 00 00 00  >................<
000040 57 69 6e 65 20 62 75 69 6c 74 69 6e 20 44 4c 4c  >Wine builtin DLL<
000050 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  >................<
000060 74 20 62 65 20 72 75 6e 20 69 6e 20 44 4f 53 20  >t be run in DOS <
000070 6d 6f 64 65 2e 0d 0d 0a 24 00 00 00 00 00 00 00  >mode....$.......<

Native:
000000 4d 5a 90 00 03 00 00 00 04 00 00 00 ff ff 00 00  >MZ..............<
000010 b8 00 00 00 00 00 00 00 40 00 00 00 00 00 00 00  >........@.......<
000020 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  >................<
000030 00 00 00 00 00 00 00 00 00 00 00 00 80 00 00 00  >................<
000040 0e 1f ba 0e 00 b4 09 cd 21 b8 01 4c cd 21 54 68  >........!..L.!Th<
000050 69 73 20 70 72 6f 67 72 61 6d 20 63 61 6e 6e 6f  >is program canno<
000060 74 20 62 65 20 72 75 6e 20 69 6e 20 44 4f 53 20  >t be run in DOS <
000070 6d 6f 64 65 2e 0d 0d 0a 24 00 00 00 00 00 00 00  >mode....$.......<

*/
pub fn native_dll_present(dir: &Path, name: &str) -> bool {
    let Ok(mut file) = File::open(dir.join(name)) else {
        return false;
    };
    let mut header = [0u8; STAMPED_HEADER_LEN];
    if file.read_exact(&mut header).is_err() {
        return true;
    }
    let e_lfanew = u32::from_le_bytes([header[0x3c], header[0x3d], header[0x3e], header[0x3f]]);
    if &header[..2] != b"MZ" || (e_lfanew as usize) < STAMPED_HEADER_LEN {
        return true;
    }
    let stamp = &header[DOS_HEADER_LEN..];
    !WINE_DLL_STAMPS.iter().any(|s| stamp.starts_with(s))
}

pub fn list_prefixes() -> Vec<PrefixInfo> {
    let games = Library::load().map(|l| l.game).unwrap_or_default();
    let mut acc: BTreeMap<PathBuf, Acc> = BTreeMap::new();

    if let Ok(entries) = std::fs::read_dir(crate::prefixes_dir()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            acc.entry(canonical(&path)).or_insert_with(|| Acc {
                display: path,
                games: Vec::new(),
                runner: String::new(),
            });
        }
    }

    for game in &games {
        if !game.uses_wine_prefix() {
            continue;
        }
        let raw = prefix_path_for(game);
        if !raw.is_dir() {
            continue;
        }
        let entry = acc.entry(canonical(&raw)).or_insert_with(|| Acc {
            display: raw,
            games: Vec::new(),
            runner: String::new(),
        });
        entry.games.push(game.metadata.name.clone());
        if entry.runner.is_empty() && !game.wine.version.is_empty() {
            entry.runner = game.wine.version.clone();
        }
    }

    let default_runner = crate::defaults::Defaults::load()
        .wine
        .version
        .unwrap_or_default();

    acc.into_values()
        .map(|a| {
            let name = a
                .display
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| a.display.to_string_lossy().into_owned());
            let runner = if a.runner.is_empty() {
                default_runner.clone()
            } else {
                a.runner
            };
            PrefixInfo {
                path: a.display,
                name,
                games: a.games,
                runner,
            }
        })
        .collect()
}

pub fn list_steam_prefixes() -> Vec<PrefixInfo> {
    let games = Library::load().map(|l| l.game).unwrap_or_default();
    let mut acc: BTreeMap<PathBuf, Acc> = BTreeMap::new();

    for game in &games {
        if game.source.kind != "steam" || game.source.app_id.is_empty() {
            continue;
        }
        let Some(pfx) = crate::store::steam::local::find_steam_prefix(&game.source.app_id) else {
            continue;
        };
        let entry = acc.entry(canonical(&pfx)).or_insert_with(|| Acc {
            display: pfx,
            games: Vec::new(),
            runner: steam_runner(&game.source.app_id),
        });
        entry.games.push(game.metadata.name.clone());
    }

    acc.into_values()
        .map(|a| PrefixInfo {
            name: a.games.first().cloned().unwrap_or_default(),
            path: a.display,
            games: a.games,
            runner: a.runner,
        })
        .collect()
}

fn steam_runner(app_id: &str) -> String {
    let stamped = crate::store::steam::local::find_steam_proton_version(app_id);
    crate::store::steam::local::resolve_or_default_proton(stamped.as_deref())
        .and_then(|p| p.file_name().map(|s| s.to_string_lossy().into_owned()))
        .or(stamped)
        .map(|name| format!("steam:{name}"))
        .unwrap_or_default()
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PrefixPreset {
    #[default]
    Base,
    Game,
    Application,
}

impl PrefixPreset {
    pub fn from_id(id: &str) -> Self {
        match id {
            "game" => Self::Game,
            "app" => Self::Application,
            _ => Self::Base,
        }
    }

    fn verbs(self) -> &'static [&'static str] {
        match self {
            Self::Base => &[],
            Self::Game => &[
                "d3dx9",
                "d3dcompiler_43",
                "d3dcompiler_47",
                "corefonts",
                "msls31",
            ],
            Self::Application => &["corefonts"],
        }
    }

    fn tool(self) -> crate::wine_tools::WineTool {
        let verbs = self.verbs();
        if verbs.is_empty() {
            return crate::wine_tools::WineTool::Wineboot;
        }
        crate::wine_tools::WineTool::WinetricksVerbs(verbs.iter().map(|s| s.to_string()).collect())
    }
}

pub fn create_prefix<F: FnMut(&str)>(
    name: &str,
    runner: &str,
    preset: &str,
    on_line: F,
) -> anyhow::Result<()> {
    let folder = crate::media::slugify(name);
    if folder.is_empty() {
        anyhow::bail!("prefix name is empty");
    }
    let dir = crate::prefixes_dir().join(&folder);
    std::fs::create_dir_all(&dir)?;

    let game = crate::library::Game::with_options(
        "Ofuda".to_string(),
        PathBuf::new(),
        Some(dir.to_string_lossy().into_owned()),
        Some(crate::library::RunnerType::Wine),
        (!runner.is_empty()).then(|| runner.to_string()),
    );

    crate::wine_tools::run_streamed(&game, PrefixPreset::from_id(preset).tool(), on_line)?;
    crate::dll_packs::install_prefix_defaults(&dir)
}

pub fn prefix_needs_bootstrap(game: &crate::library::Game) -> bool {
    if !game.uses_wine_prefix() {
        return false;
    }
    !system32_dir(&prefix_path_for(game)).is_dir()
}

pub fn bootstrap_prefix<F: FnMut(&str)>(
    game: &crate::library::Game,
    on_line: F,
) -> anyhow::Result<()> {
    let prefix = crate::launch::resolve_prefix(game);
    if system32_dir(&prefix).is_dir() {
        return Ok(());
    }

    crate::wine_tools::run_streamed(game, crate::wine_tools::WineTool::Wineboot, on_line)?;
    crate::dll_packs::install_prefix_defaults(&prefix)
}

pub fn wine_path_to_host(prefix: &Path, win_path: &str) -> Option<PathBuf> {
    let mut chars = win_path.chars();
    let letter = chars.next()?.to_ascii_lowercase();
    if !letter.is_ascii_alphabetic() || chars.next()? != ':' {
        return None;
    }
    let link = prefix.join("dosdevices").join(format!("{letter}:"));
    let root = std::fs::canonicalize(&link)
        .ok()
        .or_else(|| std::fs::read_link(&link).ok())?;
    let rest = win_path[2..].replace('\\', "/");
    let mut out = root;
    for part in rest.split('/').filter(|p| !p.is_empty() && *p != ".") {
        out = match resolve_component(&out, part) {
            Some(found) => found,
            None => out.join(part),
        };
    }
    Some(out)
}

fn resolve_component(dir: &Path, name: &str) -> Option<PathBuf> {
    let exact = dir.join(name);
    if exact.exists() {
        return Some(exact);
    }
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .find(|e| e.file_name().to_string_lossy().eq_ignore_ascii_case(name))
        .map(|e| e.path())
}

pub fn delete_prefix(target: &Path) -> bool {
    if !target.is_dir() {
        tracing::warn!("delete_prefix: not a directory: {}", target.display());
        return false;
    }
    if !list_prefixes().iter().any(|p| p.path == target) {
        tracing::warn!(
            "delete_prefix refused, not a known prefix: {}",
            target.display()
        );
        return false;
    }
    match std::fs::remove_dir_all(target) {
        Ok(_) => true,
        Err(e) => {
            tracing::error!("delete_prefix failed: {e}");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dll_with_stamp(stamp: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0u8; STAMPED_HEADER_LEN + 16];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[0x3c..0x40].copy_from_slice(&(STAMPED_HEADER_LEN as u32).to_le_bytes());
        bytes[DOS_HEADER_LEN..DOS_HEADER_LEN + stamp.len()].copy_from_slice(stamp);
        bytes
    }

    #[test]
    fn native_dll_present_reads_the_wine_stamp() {
        let dir = tempfile::tempdir().unwrap();
        let write =
            |name: &str, bytes: &[u8]| std::fs::write(dir.path().join(name), bytes).unwrap();
        write("builtin.dll", &dll_with_stamp(b"Wine builtin DLL\0"));
        write(
            "placeholder.dll",
            &dll_with_stamp(b"Wine placeholder DLL\0"),
        );
        write("dxvk.dll", &dll_with_stamp(b""));

        assert!(!native_dll_present(dir.path(), "builtin.dll"));
        assert!(!native_dll_present(dir.path(), "placeholder.dll"));
        assert!(native_dll_present(dir.path(), "dxvk.dll"));
        assert!(!native_dll_present(dir.path(), "missing.dll"));
    }
}
