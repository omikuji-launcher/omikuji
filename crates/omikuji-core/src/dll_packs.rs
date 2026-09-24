use crate::archive_source;
use crate::components_config::{self, ArchiveSource};
use crate::launch::{ProtonVerb, WineVariant, wine_command};
use crate::library::Game;
use crate::prefixes;
use anyhow::Result;
use serde::Serialize;
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

pub const BUILTIN: &str = "builtin";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DllKind {
    Dxvk,
    Vkd3d,
    Nvapi,
}

impl DllKind {
    pub const ALL: [DllKind; 3] = [DllKind::Dxvk, DllKind::Vkd3d, DllKind::Nvapi];

    pub fn from_pack_kind(kind: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.pack_kind() == kind)
    }

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

    pub fn dlls(self) -> &'static str {
        match self {
            DllKind::Dxvk => "d3d11,d3d10core,d3d9,d3d8,dxgi",
            DllKind::Vkd3d => "d3d12,d3d12core",
            DllKind::Nvapi => "nvapi,nvapi64,nvofapi64",
        }
    }

    pub fn probe_dll(self, is_64bit: bool) -> &'static str {
        match self {
            DllKind::Dxvk => "d3d11.dll",
            DllKind::Vkd3d => "d3d12.dll",
            DllKind::Nvapi if is_64bit => "nvapi64.dll",
            DllKind::Nvapi => "nvapi.dll",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Layer {
    Off,
    Builtin,
    Pack(String),
}

fn game_layer(game: &Game, kind: DllKind) -> (bool, &str) {
    match kind {
        DllKind::Dxvk => (game.wine.dxvk, game.wine.dxvk_version.as_str()),
        DllKind::Vkd3d => (game.wine.vkd3d, game.wine.vkd3d_version.as_str()),
        DllKind::Nvapi => (game.wine.dxvk_nvapi, game.wine.dxvk_nvapi_version.as_str()),
    }
}

pub fn resolved_layer(game: &Game, kind: DllKind) -> Layer {
    let (enabled, pinned) = game_layer(game, kind);
    if !enabled {
        return Layer::Off;
    }
    if pack_dir(kind.pack_kind(), pinned).is_some() {
        Layer::Pack(pinned.to_string())
    } else {
        Layer::Builtin
    }
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

fn install_pack(kind: &str, tag: &str, system32: &Path, syswow64: Option<&PathBuf>) -> Result<()> {
    let Some(pack_root) = pack_dir(kind, tag) else {
        tracing::warn!("{} {} resolved but its install dir is gone", kind, tag);
        return Ok(());
    };
    let (x64_src, x32_src) = pack_arch_dirs(&pack_root);

    match syswow64 {
        Some(syswow64) => {
            if x64_src.exists() {
                copy_dll_dir(&x64_src, system32)?;
            }
            if let Some(ref x32) = x32_src {
                copy_dll_dir(x32, syswow64)?;
            }
        }
        None => {
            if let Some(ref x32) = x32_src {
                copy_dll_dir(x32, system32)?;
            }
        }
    }

    tracing::info!("injected {} {} -> {}", kind, tag, system32.display());
    Ok(())
}

fn prefix_install_tag(source: &ArchiveSource) -> Option<&str> {
    let tag = source.prefix_install_version.as_str();
    (!tag.is_empty() && tag != "disabled").then_some(tag)
}

pub fn install_prefix_defaults(prefix: &Path) -> Result<()> {
    let system32 = prefixes::system32_dir(prefix);
    if !system32.is_dir() {
        return Ok(());
    }
    let syswow64 = prefixes::syswow64_dir(prefix);
    let syswow64 = syswow64.is_dir().then_some(syswow64);

    for source in components_config::get().layers {
        let Some(tag) = prefix_install_tag(&source) else {
            continue;
        };
        install_pack(&source.kind, tag, &system32, syswow64.as_ref())?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrefixLayer {
    Present,
    Absent,
    PresentOnCreate,
    AbsentOnCreate,
}

fn auto_installs_into_new_prefixes(kind: DllKind) -> bool {
    let kind = kind.pack_kind();
    components_config::get().layers.iter().any(|source| {
        source.kind == kind
            && prefix_install_tag(source).is_some_and(|tag| pack_dir(kind, tag).is_some())
    })
}

pub fn prefix_layer(prefix: &Path, kind: DllKind) -> PrefixLayer {
    let system32 = prefixes::system32_dir(prefix);
    if !system32.is_dir() {
        return if auto_installs_into_new_prefixes(kind) {
            PrefixLayer::PresentOnCreate
        } else {
            PrefixLayer::AbsentOnCreate
        };
    }
    let is_64bit = prefixes::syswow64_dir(prefix).is_dir();
    if prefixes::native_dll_present(&system32, kind.probe_dll(is_64bit)) {
        PrefixLayer::Present
    } else {
        PrefixLayer::Absent
    }
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

    let system32 = prefixes::system32_dir(&prefix);
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
    let syswow64 = prefixes::syswow64_dir(&prefix);
    let is_64bit = syswow64.exists();

    for kind in DllKind::ALL {
        let Layer::Pack(tag) = resolved_layer(game, kind) else {
            continue;
        };
        install_pack(
            kind.pack_kind(),
            &tag,
            &system32,
            is_64bit.then_some(&syswow64),
        )?;
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
    install_prefix_defaults(prefix)
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
