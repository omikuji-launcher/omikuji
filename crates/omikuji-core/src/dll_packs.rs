use crate::archive_source;
use crate::components_config::{self, ArchiveSource};
use crate::launch::{ProtonVerb, WineVariant, wine_command};
use crate::library::Game;
use anyhow::Result;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Stdio;

pub fn list_sources() -> Vec<ArchiveSource> {
    components_config::get().layers
}

// per-source root: components/layers/{source.name}/. versions land inside as {tag}/.
pub fn source_root(source: &ArchiveSource) -> PathBuf {
    crate::layers_dir().join(&source.name)
}

pub async fn fetch_versions(source: &ArchiveSource) -> Result<Vec<archive_source::ReleaseInfo>> {
    archive_source::fetch_versions(source).await
}

pub async fn install_version(
    source: &ArchiveSource,
    release: &archive_source::ReleaseInfo,
) -> Result<PathBuf> {
    archive_source::install_version("dll_packs", source, release, &source_root(source)).await
}

pub fn list_installed(source: &ArchiveSource) -> Vec<String> {
    archive_source::list_installed(source, &source_root(source))
}

pub fn delete_version(source: &ArchiveSource, tag: &str) -> Result<()> {
    archive_source::delete_version(source, &source_root(source), tag)
}

fn resolve_pack(
    cfg: &components_config::ComponentsConfig,
    kind: &str,
    pinned: &str,
) -> Option<(ArchiveSource, String)> {
    let sources: Vec<ArchiveSource> = cfg
        .layers
        .iter()
        .filter(|s| s.kind == kind)
        .cloned()
        .collect();
    let tag = if !pinned.is_empty() && pinned != "disabled" {
        pinned.to_string()
    } else {
        sources
            .iter()
            .map(|s| components_config::active_version(&s.name))
            .find(|t| !t.is_empty() && t != "disabled")?
    };
    let source = sources
        .into_iter()
        .find(|s| list_installed(s).iter().any(|v| v == &tag))?;
    Some((source, tag))
}

fn game_layer<'a>(game: &'a Game, kind: &str) -> Option<(bool, &'a str)> {
    match kind {
        "dxvk" => Some((game.wine.dxvk, game.wine.dxvk_version.as_str())),
        "vkd3d" => Some((game.wine.vkd3d, game.wine.vkd3d_version.as_str())),
        "dxvk_nvapi" => Some((game.wine.dxvk_nvapi, game.wine.dxvk_nvapi_version.as_str())),
        _ => None,
    }
}

pub fn resolved_layer(game: &Game, kind: &str) -> Option<String> {
    let (enabled, pinned) = game_layer(game, kind)?;
    if !enabled {
        return None;
    }
    resolve_pack(&components_config::get(), kind, pinned).map(|(_, tag)| tag)
}

pub fn installed_versions_for_kind(kind: &str) -> Vec<String> {
    let mut out = Vec::new();
    for source in components_config::get()
        .layers
        .iter()
        .filter(|s| s.kind == kind)
    {
        for v in list_installed(source) {
            if !out.contains(&v) {
                out.push(v);
            }
        }
    }
    out
}

pub fn pack_dir(kind: &str, tag: &str) -> Option<PathBuf> {
    let source = components_config::get()
        .layers
        .into_iter()
        .filter(|s| s.kind == kind)
        .find(|s| list_installed(s).iter().any(|v| v == tag))?;
    let root = source_root(&source);
    archive_source::installed_dir(&source.name, &root, tag)
        .or_else(|| root.join(tag).exists().then(|| root.join(tag)))
}

pub fn pack_arch_dirs(pack_root: &Path) -> (PathBuf, Option<PathBuf>) {
    let x64 = pack_root.join("x64");
    let x32 = ["x32", "x86"]
        .iter()
        .map(|d| pack_root.join(d))
        .find(|p| p.exists());
    (x64, x32)
}

pub fn inject_all(game: &Game, env: &HashMap<String, String>) -> Result<()> {
    let Some(prefix_str) = env.get("WINEPREFIX") else {
        return Ok(());
    };
    let prefix = PathBuf::from(prefix_str);
    let wine_exe = env
        .get("WINE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("wine"));
    let variant = WineVariant::from_version(&game.wine.version);

    let system32 = prefix.join("drive_c").join("windows").join("system32");
    if !system32.exists() {
        ensure_prefix_bootstrapped(&prefix, &wine_exe, variant, env)?;
    }
    if !system32.exists() {
        tracing::warn!(
            "prefix bootstrap left no system32, skipping injection for {}",
            prefix.display()
        );
        return Ok(());
    }
    let syswow64 = prefix.join("drive_c").join("windows").join("syswow64");
    let is_64bit = syswow64.exists();

    if variant != WineVariant::Proton {
        for kind in ["dxvk", "vkd3d", "dxvk_nvapi"] {
            let Some(tag) = resolved_layer(game, kind) else {
                continue;
            };
            let Some(pack_root) = pack_dir(kind, &tag) else {
                tracing::warn!("{} {} resolved but its install dir is gone", kind, tag);
                continue;
            };

            let (x64_src, x32_src) = pack_arch_dirs(&pack_root);

            if is_64bit {
                if x64_src.exists() {
                    copy_dll_dir(&x64_src, &system32)?;
                }
                if let Some(ref x32) = x32_src {
                    copy_dll_dir(x32, &syswow64)?;
                }
            } else if let Some(ref x32) = x32_src {
                copy_dll_dir(x32, &system32)?;
            }

            tracing::info!("injected {} {} -> {}", kind, tag, prefix.display());
        }
    }

    if game.wine.dxvk_nvapi && is_64bit {
        if let Some(nvidia_wine_dir) = find_nvidia_wine_dir() {
            let mut copied = false;
            for name in ["nvngx.dll", "_nvngx.dll"] {
                let src = nvidia_wine_dir.join(name);
                if src.exists() {
                    let dest = system32.join(name);
                    if let Err(e) = std::fs::copy(&src, &dest) {
                        tracing::error!("failed to copy {}: {}", name, e);
                    } else {
                        copied = true;
                    }
                }
            }
            if copied {
                tracing::info!(
                    "copied nvngx from {} -> {}",
                    nvidia_wine_dir.display(),
                    system32.display()
                );
                if let Err(e) = set_ngx_registry(&wine_exe, variant, env) {
                    tracing::error!("ngx registry set failed: {}", e);
                }
            }
        } else {
            tracing::warn!("dxvk-nvapi active but nvidia wine dir not found - dlss disabled");
        }
    }

    Ok(())
}

pub fn copy_dll_dir(from: &Path, to: &Path) -> Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .extension()
            .map(|e| e.eq_ignore_ascii_case(OsStr::new("dll")))
            .unwrap_or(false)
            && let Some(file_name) = path.file_name()
        {
            let dest = to.join(file_name);
            if same_size(&path, &dest) {
                continue;
            }
            std::fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

fn same_size(src: &Path, dest: &Path) -> bool {
    match (std::fs::metadata(src), std::fs::metadata(dest)) {
        (Ok(a), Ok(b)) => a.len() == b.len(),
        _ => false,
    }
}

// wineboot -u the prefix and wait for it. needed when the prefix dir exists but wine has
// never populated it, so no system32 yet and injecting would have nowhere to land.
// idempotent at the wine leel but we still gate on system32 missing to avoid ~5s on every launch.
fn ensure_prefix_bootstrapped(
    prefix: &Path,
    wine_exe: &Path,
    variant: WineVariant,
    env: &HashMap<String, String>,
) -> Result<()> {
    tracing::info!("bootstrapping prefix via wineboot: {}", prefix.display());
    // waitforexitandrun so umu-run waits for the wineboot child before tearing the session down
    let mut cmd = wine_command(
        wine_exe,
        env,
        variant,
        Some(ProtonVerb::WaitForExitAndRun),
        ["wineboot", "-u"],
    );
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    let status = cmd
        .status()
        .map_err(|e| anyhow::anyhow!("failed to spawn wineboot: {}", e))?;
    if !status.success() {
        anyhow::bail!("wineboot -u exited with {}", status);
    }
    Ok(())
}

// search common nvidia driver install locations for teh wine nvngx bridge dlls. first hit wins.
fn find_nvidia_wine_dir() -> Option<PathBuf> {
    const CANDIDATES: &[&str] = &[
        "/usr/lib/nvidia/wine",
        "/usr/lib/x86_64-linux-gnu/nvidia/wine",
        "/usr/lib64/nvidia/wine",
        "/opt/nvidia/wine",
    ];
    for c in CANDIDATES {
        let p = Path::new(c);
        if p.join("nvngx.dll").exists() {
            return Some(p.to_path_buf());
        }
    }
    None
}

// without this registry key dlss silently falls back to whatever the engine ships,which on linux is nothing
fn set_ngx_registry(
    wine_exe: &Path,
    variant: WineVariant,
    env: &HashMap<String, String>,
) -> Result<()> {
    let mut cmd = wine_command(
        wine_exe,
        env,
        variant,
        Some(ProtonVerb::WaitForExitAndRun),
        [
            "reg",
            "add",
            r"HKEY_LOCAL_MACHINE\SOFTWARE\NVIDIA Corporation\Global\NGXCore",
            "/v",
            "FullPath",
            "/d",
            r"C:\windows\system32",
            "/f",
        ],
    );
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("reg add NGXCore exited with {}", status);
    }
    Ok(())
}
