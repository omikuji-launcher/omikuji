pub mod api;
pub mod sophon;
pub mod source;
pub mod update;

use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::gacha::file_sync::sanitize_rel;
use crate::gacha::manifest::{GachaManifest, ManifestEdition, ManifestVoice};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HoyoEdition {
    Global,
    China,
}

impl HoyoEdition {
    pub fn from_id(id: &str) -> Result<Self> {
        match id {
            "global" => Ok(Self::Global),
            "china" => Ok(Self::China),
            other => bail!("unknown hoyo edition: {}", other),
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::China => "china",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Global => "Global",
            Self::China => "China",
        }
    }

    pub fn launcher_id(&self) -> &'static str {
        match self {
            Self::Global => "VYTpXlbWo8",
            Self::China => "jGHBHlcOq1",
        }
    }
}

pub fn voice_installed(
    manifest: &GachaManifest,
    edition: &ManifestEdition,
    voice: &ManifestVoice,
    root: &Path,
) -> bool {
    root.join(manifest.voice_folder(edition, voice)).is_dir()
}

pub fn installed_voice_ids(
    manifest: &GachaManifest,
    edition: &ManifestEdition,
    root: &Path,
) -> Vec<String> {
    manifest
        .voice_locales
        .iter()
        .filter(|voice| voice_installed(manifest, edition, voice, root))
        .map(|voice| voice.id.clone())
        .collect()
}

// the pack's own Audio_*_pkg_version lists its files so removal needs no network. hsr ships none and falls back to the folder
pub fn remove_voice_pack(
    manifest: &GachaManifest,
    edition: &ManifestEdition,
    root: &Path,
    pack: &str,
) -> Result<()> {
    let voice = manifest
        .voice_locales
        .iter()
        .find(|v| v.id == pack)
        .ok_or_else(|| anyhow!("unknown voice pack: {pack}"))?;
    remove_pack_files(root, &manifest.voice_folder(edition, voice))
}

fn remove_pack_files(root: &Path, folder: &Path) -> Result<()> {
    if let Some((marker, files)) = pack_marker(root, folder) {
        for rel in files {
            match fs_err::remove_file(root.join(sanitize_rel(&rel))) {
                Err(e) if e.kind() != ErrorKind::NotFound => return Err(e.into()),
                _ => {}
            }
        }
        fs_err::remove_file(marker)?;
    }
    let dir = root.join(folder);
    if dir.exists() {
        fs_err::remove_dir_all(dir)?;
    }
    Ok(())
}

#[derive(Deserialize)]
struct PkgVersionLine {
    #[serde(rename = "remoteName")]
    remote_name: String,
}

fn pack_marker(root: &Path, folder: &Path) -> Option<(PathBuf, Vec<String>)> {
    let prefix = format!("{}/", folder.to_string_lossy());
    fs_err::read_dir(root)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("Audio_") && name.ends_with("_pkg_version"))
        })
        .find_map(|path| {
            let files: Vec<String> = fs_err::read_to_string(&path)
                .ok()?
                .lines()
                .filter_map(|line| serde_json::from_str::<PkgVersionLine>(line).ok())
                .map(|line| line.remote_name)
                .collect();
            files
                .iter()
                .any(|file| file.starts_with(&prefix))
                .then_some((path, files))
        })
}

#[derive(Deserialize)]
struct HoyoConfig {
    biz_id: String,
}

pub fn biz_id(manifest: &GachaManifest, edition_id: &str) -> Result<String> {
    let config: HoyoConfig = manifest.strategy_config(manifest.require_edition(edition_id)?)?;
    Ok(config.biz_id)
}

pub fn read_install_version(install_path: &std::path::Path, data_folder: &str) -> Option<String> {
    use crate::gacha::state;
    if let Some(v) = state::read_install_dotversion(install_path) {
        return Some(v);
    }
    if let Some(v) = state::scan_globalgamemanagers(install_path, data_folder, b'_') {
        return Some(v);
    }
    if let Some(v) = state::scan_globalgamemanagers(install_path, data_folder, 0) {
        return Some(v);
    }
    if data_folder.is_empty() {
        return None;
    }
    state::scan_unity_file(
        &install_path.join(data_folder).join("data.unity3d"),
        2000,
        524288,
        0,
    )
}

#[cfg(test)]
mod tests {
    use super::remove_pack_files;
    use std::path::Path;
    use tempfile::tempdir;

    const FULL: &str = "Game_Data/StreamingAssets/Audio/Windows/Full";

    fn touch(root: &Path, rel: &str) {
        let path = root.join(rel);
        fs_err::create_dir_all(path.parent().unwrap()).unwrap();
        fs_err::write(path, b"x").unwrap();
    }

    fn marker(root: &Path, name: &str, files: &[&str]) {
        let body: Vec<String> = files
            .iter()
            .map(|f| format!(r#"{{"remoteName": "{f}", "md5": "0", "fileSize": 1}}"#))
            .collect();
        fs_err::write(root.join(name), body.join("\n")).unwrap();
    }

    #[test]
    fn removes_only_the_marked_pack() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        let jp = [format!("{FULL}/Jp/1.pck"), format!("{FULL}/Jp/2.pck")];
        let en = format!("{FULL}/En/1.pck");
        for f in jp.iter().chain([&en]) {
            touch(root, f);
        }
        touch(root, &format!("{FULL}/Hotfix.pck"));
        marker(root, "Audio_Japanese_pkg_version", &[&jp[0], &jp[1]]);
        marker(root, "Audio_English(US)_pkg_version", &[&en]);

        remove_pack_files(root, &Path::new(FULL).join("Jp")).unwrap();

        assert!(!root.join(FULL).join("Jp").exists());
        assert!(!root.join("Audio_Japanese_pkg_version").exists());
        assert!(root.join(&en).exists());
        assert!(root.join("Audio_English(US)_pkg_version").exists());
        assert!(root.join(FULL).join("Hotfix.pck").exists());
    }

    #[test]
    fn falls_back_to_the_folder_without_a_marker() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        touch(
            root,
            "Game_Data/Persistent/Audio/AudioPackage/Windows/English/External0.pck",
        );
        touch(
            root,
            "Game_Data/Persistent/Audio/AudioPackage/Windows/Japanese/External0.pck",
        );

        remove_pack_files(
            root,
            Path::new("Game_Data/Persistent/Audio/AudioPackage/Windows/English"),
        )
        .unwrap();

        assert!(
            !root
                .join("Game_Data/Persistent/Audio/AudioPackage/Windows/English")
                .exists()
        );
        assert!(
            root.join("Game_Data/Persistent/Audio/AudioPackage/Windows/Japanese/External0.pck")
                .exists()
        );
    }
}
