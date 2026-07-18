// dll pack cache; downloaded dxvk/vkd3d/dxvk-nvapi/d3d-extras archives sit under
// components/layers/{source.name}/{archive name}/. per-source subfolder prevents tag collisions
// between sources (e.g. DXVK and DXVK-NVAPI both shipping "v2.4"). per-prefix apply copies dlls
// into {prefix}/drive_c/windows/system32/ and syswow64/ and tracks applied versions in
// {prefix}/.omikuji/dll_versions.toml so we skip redndant copies on every launch.

use crate::archive_source;
use crate::components_config::{self, ArchiveSource};
use crate::launch::WineVariant;
use crate::library::Game;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn list_sources() -> Vec<ArchiveSource> {
    components_config::get().layers
}

pub fn source_by_name(name: &str) -> Option<ArchiveSource> {
    list_sources().into_iter().find(|s| s.name == name)
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

#[derive(Debug, Default, Deserialize, Serialize)]
struct InjectedVersions {
    #[serde(default)]
    dll_packs: HashMap<String, String>,
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

    let marker_dir = prefix.join(".omikuji");
    let marker_path = marker_dir.join("dll_versions.toml");
    let mut applied: InjectedVersions = if marker_path.exists() {
        std::fs::read_to_string(&marker_path)
            .ok()
            .and_then(|b| toml::from_str(&b).ok())
            .unwrap_or_default()
    } else {
        InjectedVersions::default()
    };

    let state = components_config::get();
    let mut changed = false;

    for (name, tag) in &state.active {
        // legacy state files used the literal "disabled" string; the ui writes "" which set_active_version turns into a removed key, but be defensive about both
        if tag.is_empty() || tag == "disabled" {
            continue;
        }

        let root = crate::layers_dir().join(name);
        let Some(pack_root) = archive_source::installed_dir(name, &root, tag)
            .or_else(|| root.join(tag).exists().then(|| root.join(tag)))
        else {
            tracing::warn!("active pack {}/{} not installed, skipping", name, tag);
            continue;
        };

        if applied.dll_packs.get(name).map(|v| v.as_str()) == Some(tag.as_str()) {
            continue;
        }

        // x64 dlls always land in system32. 32-bit dlls go to syswow64 on a 64-bit prefix
        // or system32 on a 32-bit prefix. some packs ship "x86" instead of "x32" (vkd3d upstream), try both. iguess
        let x64_src = pack_root.join("x64");
        let x32_src = ["x32", "x86"]
            .iter()
            .map(|d| pack_root.join(d))
            .find(|p| p.exists());

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

        tracing::info!("injected {} {} -> {}", name, tag, prefix.display());
        applied.dll_packs.insert(name.clone(), tag.clone());
        changed = true;
    }

    // when dxvk-nvapi is active, locate nvngx.dll and _nvngx.dll from the host nvidia driver
    // install and copy into system32. the ngx registry key points dlss at that location.
    // no-op on systems without nvidia drivers. dont own a nvidia gpu so we just hope this works
    let nvapi_active = state.active.iter().any(|(n, t)| {
        !t.is_empty()
            && t != "disabled"
            && state
                .layers
                .iter()
                .any(|s| s.name == *n && s.kind == "dxvk_nvapi")
    });
    if nvapi_active && is_64bit {
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

    if changed {
        let body = toml::to_string_pretty(&applied)?;
        crate::fs_util::write_atomic(&marker_path, body)?;
    }

    Ok(())
}

fn copy_dll_dir(from: &Path, to: &Path) -> Result<()> {
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
            std::fs::copy(&path, &dest)?;
        }
    }
    Ok(())
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
    let mut cmd = Command::new(wine_exe);
    cmd.arg("wineboot").arg("-u");
    cmd.env_clear();
    cmd.envs(env);
    if variant == WineVariant::Proton {
        // umu-run synchronous verb, waits for the wineboot child to exit before tearing down
        cmd.env("PROTON_VERB", "waitforexitandrun");
    }
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
    let mut cmd = Command::new(wine_exe);
    cmd.args([
        "reg",
        "add",
        r"HKEY_LOCAL_MACHINE\SOFTWARE\NVIDIA Corporation\Global\NGXCore",
        "/v",
        "FullPath",
        "/d",
        r"C:\windows\system32",
        "/f",
    ]);
    cmd.env_clear();
    cmd.envs(env);
    if variant == WineVariant::Proton {
        cmd.env("PROTON_VERB", "waitforexitandrun");
    }
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("reg add NGXCore exited with {}", status);
    }
    Ok(())
}
