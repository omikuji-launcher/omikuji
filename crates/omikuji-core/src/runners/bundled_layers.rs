use crate::dll_packs::{self, DllKind, Layer};
use crate::library::Game;
use anyhow::Result;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

// runners used to get their bundled dlls swapped in place, this only undoes what older builds left behind
// TODO: remove after a few releases, along with restore_legacy_swap and everything it uses
const SIDECAR: &str = ".omikuji-dll-override.json";
const BAK_SUFFIX: &str = ".omikuji-bak";

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum RestoreAction {
    Backup,
    Delete,
}

#[derive(Deserialize)]
struct OverrideFile {
    path: String,
    restore: RestoreAction,
}

#[derive(Deserialize)]
struct KindOverride {
    files: Vec<OverrideFile>,
}

fn bundle_dirs(runner_dir: &Path, kind: DllKind) -> Option<(PathBuf, Option<PathBuf>)> {
    let bundle = runner_dir.join("files/lib/wine").join(kind.bundle_subdir());
    if !bundle.is_dir() {
        return None;
    }
    let pe64 = bundle.join("x86_64-windows");
    if pe64.is_dir() {
        let pe32 = bundle.join("i386-windows");
        return Some((pe64, pe32.is_dir().then_some(pe32)));
    }
    let classic64 = runner_dir
        .join("files/lib64/wine")
        .join(kind.bundle_subdir());
    let dir64 = if classic64.is_dir() {
        classic64
    } else {
        bundle.clone()
    };
    Some((dir64, Some(bundle)))
}

pub fn apply_for_launch(
    runner_dir: &Path,
    game: &Game,
    env: &HashMap<String, String>,
) -> Result<()> {
    restore_legacy_swap(runner_dir);
    let Some(prefix) = env.get("WINEPREFIX").map(PathBuf::from) else {
        return Ok(());
    };
    let system32 = prefix.join("drive_c/windows/system32");
    if !system32.is_dir() {
        return Ok(());
    }
    let syswow64 = prefix.join("drive_c/windows/syswow64");
    for k in DllKind::ALL {
        if matches!(dll_packs::resolved_layer(game, k), Layer::Pack(_)) {
            continue;
        }
        let Some((src64, src32)) = bundle_dirs(runner_dir, k) else {
            continue;
        };
        dll_packs::copy_dll_dir(&src64, &system32)?;
        if let (Some(src32), true) = (src32, syswow64.is_dir()) {
            dll_packs::copy_dll_dir(&src32, &syswow64)?;
        }
    }
    Ok(())
}

fn restore_legacy_swap(runner_dir: &Path) {
    let sidecar = runner_dir.join(SIDECAR);
    let Ok(raw) = std::fs::read_to_string(&sidecar) else {
        return;
    };
    let state: BTreeMap<String, KindOverride> = serde_json::from_str(&raw).unwrap_or_default();
    for f in state.into_values().flat_map(|o| o.files) {
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
    for (dir64, dir32) in DllKind::ALL
        .into_iter()
        .filter_map(|k| bundle_dirs(runner_dir, k))
    {
        for dir in [Some(dir64), dir32].into_iter().flatten() {
            restore_baks_in(&dir);
        }
    }
    let _ = std::fs::remove_file(&sidecar);
    tracing::info!("restored the stock dll bundle of {}", runner_dir.display());
}

fn bak_of(target: &Path) -> PathBuf {
    let mut name = target.file_name().unwrap_or_default().to_os_string();
    name.push(BAK_SUFFIX);
    target.with_file_name(name)
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
