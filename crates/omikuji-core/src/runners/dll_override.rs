use crate::dll_packs;
use crate::library::Game;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

const SIDECAR: &str = ".omikuji-dll-override.json";
const BAK_SUFFIX: &str = ".omikuji-bak";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DllKind {
    Dxvk,
    Vkd3d,
    Nvapi,
}

impl DllKind {
    pub const ALL: [DllKind; 3] = [DllKind::Dxvk, DllKind::Vkd3d, DllKind::Nvapi];

    pub fn pack_kind(self) -> &'static str {
        match self {
            DllKind::Dxvk => "dxvk",
            DllKind::Vkd3d => "vkd3d",
            DllKind::Nvapi => "dxvk_nvapi",
        }
    }

    pub fn bundle_subdir(self) -> &'static str {
        match self {
            DllKind::Dxvk => "dxvk",
            DllKind::Vkd3d => "vkd3d-proton",
            DllKind::Nvapi => "nvapi",
        }
    }

    pub fn from_pack_kind(s: &str) -> Option<Self> {
        DllKind::ALL.into_iter().find(|k| k.pack_kind() == s)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum RestoreAction {
    Backup,
    Delete,
}

#[derive(Serialize, Deserialize, Clone)]
struct OverrideFile {
    path: String,
    restore: RestoreAction,
}

#[derive(Serialize, Deserialize, Clone)]
struct KindOverride {
    component: String,
    files: Vec<OverrideFile>,
}

type OverrideState = BTreeMap<String, KindOverride>;

fn sidecar_path(runner_dir: &Path) -> PathBuf {
    runner_dir.join(SIDECAR)
}

fn read_state(runner_dir: &Path) -> OverrideState {
    std::fs::read_to_string(sidecar_path(runner_dir))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_state(runner_dir: &Path, state: &OverrideState) -> Result<()> {
    let path = sidecar_path(runner_dir);
    if state.is_empty() {
        let _ = std::fs::remove_file(&path);
        return Ok(());
    }
    crate::fs_util::write_atomic(&path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

fn arch_targets(runner_dir: &Path, kind: DllKind) -> Option<(PathBuf, Option<PathBuf>)> {
    let bundle = runner_dir.join("files/lib/wine").join(kind.bundle_subdir());
    if !bundle.is_dir() {
        return None;
    }
    let pe64 = bundle.join("x86_64-windows");
    if pe64.is_dir() {
        let pe32 = bundle.join("i386-windows");
        return Some((pe64, pe32.is_dir().then_some(pe32)));
    }
    let classic64 = runner_dir.join("files/lib64/wine").join(kind.bundle_subdir());
    let dst64 = if classic64.is_dir() {
        classic64
    } else {
        bundle.clone()
    };
    Some((dst64, Some(bundle)))
}

pub fn supported(runner_dir: &Path) -> BTreeMap<String, bool> {
    DllKind::ALL
        .into_iter()
        .map(|k| {
            (
                k.pack_kind().to_string(),
                arch_targets(runner_dir, k).is_some(),
            )
        })
        .collect()
}

pub fn active(runner_dir: &Path) -> BTreeMap<String, String> {
    let state = read_state(runner_dir);
    DllKind::ALL
        .into_iter()
        .map(|k| {
            let tag = state
                .get(k.pack_kind())
                .map(|o| o.component.clone())
                .unwrap_or_default();
            (k.pack_kind().to_string(), tag)
        })
        .collect()
}

fn sync_from_game(runner_dir: &Path, game: &Game) -> Result<()> {
    let sup = supported(runner_dir);
    let current = active(runner_dir);
    for k in DllKind::ALL {
        let key = k.pack_kind();
        if !sup.get(key).copied().unwrap_or(false) {
            continue;
        }
        let now = current.get(key).cloned().unwrap_or_default();
        match dll_packs::resolved_layer(game, key) {
            Some(tag) if tag != now => apply(runner_dir, k, &tag)?,
            None if !now.is_empty() => restore(runner_dir, k)?,
            _ => {}
        }
    }
    Ok(())
}

pub fn apply_for_launch(runner_dir: &Path, game: &Game, env: &HashMap<String, String>) -> Result<()> {
    sync_from_game(runner_dir, game)?;
    let Some(prefix) = env.get("WINEPREFIX").map(PathBuf::from) else {
        return Ok(());
    };
    let system32 = prefix.join("drive_c/windows/system32");
    if !system32.is_dir() {
        return Ok(());
    }
    let syswow64 = prefix.join("drive_c/windows/syswow64");
    for k in DllKind::ALL {
        let Some((src64, src32)) = arch_targets(runner_dir, k) else {
            continue;
        };
        dll_packs::copy_dll_dir(&src64, &system32)?;
        if let (Some(src32), true) = (src32, syswow64.is_dir()) {
            dll_packs::copy_dll_dir(&src32, &syswow64)?;
        }
    }
    Ok(())
}

fn is_dll(p: &Path) -> bool {
    p.extension()
        .map(|e| e.eq_ignore_ascii_case("dll"))
        .unwrap_or(false)
}

fn bak_of(target: &Path) -> PathBuf {
    let mut name = target.file_name().unwrap_or_default().to_os_string();
    name.push(BAK_SUFFIX);
    target.with_file_name(name)
}

fn rel(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn override_into(
    src: &Path,
    dst: &Path,
    runner_dir: &Path,
    files: &mut Vec<OverrideFile>,
) -> Result<()> {
    for entry in std::fs::read_dir(src)? {
        let from = entry?.path();
        if !is_dll(&from) {
            continue;
        }
        let Some(name) = from.file_name() else {
            continue;
        };
        let target = dst.join(name);
        let bak = bak_of(&target);
        let restore = if target.exists() {
            if !bak.exists() {
                std::fs::rename(&target, &bak)
                    .with_context(|| format!("backing up {}", target.display()))?;
            }
            RestoreAction::Backup
        } else {
            RestoreAction::Delete
        };
        std::fs::copy(&from, &target).with_context(|| format!("writing {}", target.display()))?;
        files.push(OverrideFile {
            path: rel(&target, runner_dir),
            restore,
        });
    }
    Ok(())
}

pub fn apply(runner_dir: &Path, kind: DllKind, tag: &str) -> Result<()> {
    restore(runner_dir, kind)?;

    let Some((dst64, dst32)) = arch_targets(runner_dir, kind) else {
        anyhow::bail!(
            "{} ships no {} to override",
            runner_dir.display(),
            kind.bundle_subdir()
        );
    };
    let pack = dll_packs::pack_dir(kind.pack_kind(), tag)
        .with_context(|| format!("{} {} is not installed", kind.pack_kind(), tag))?;
    let (src64, src32) = dll_packs::pack_arch_dirs(&pack);

    let mut files = Vec::new();
    if src64.is_dir() {
        override_into(&src64, &dst64, runner_dir, &mut files)?;
    }
    if let (Some(src32), Some(dst32)) = (src32, dst32) {
        override_into(&src32, &dst32, runner_dir, &mut files)?;
    }

    let mut state = read_state(runner_dir);
    state.insert(
        kind.pack_kind().to_string(),
        KindOverride {
            component: tag.to_string(),
            files,
        },
    );
    write_state(runner_dir, &state)
}

pub fn restore(runner_dir: &Path, kind: DllKind) -> Result<()> {
    let mut state = read_state(runner_dir);
    if let Some(over) = state.remove(kind.pack_kind()) {
        for f in over.files {
            let target = runner_dir.join(&f.path);
            match f.restore {
                RestoreAction::Backup => {
                    let bak = bak_of(&target);
                    if bak.exists() {
                        let _ = std::fs::rename(&bak, &target);
                    }
                }
                RestoreAction::Delete => {
                    let _ = std::fs::remove_file(&target);
                }
            }
        }
    }
    if let Some((dst64, dst32)) = arch_targets(runner_dir, kind) {
        for dir in [Some(dst64), dst32].into_iter().flatten() {
            restore_baks_in(&dir);
        }
    }
    write_state(runner_dir, &state)
}

fn restore_baks_in(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let bak = entry.path();
        let Some(name) = bak.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if let Some(orig) = name.strip_suffix(BAK_SUFFIX) {
            let _ = std::fs::rename(&bak, dir.join(orig));
        }
    }
}
