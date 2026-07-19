//!update tries in order:
// 1. per-file resource diffs (preferred): patch.json manifests with HDiffPatch
//      variants keyed by base_md5. a 1.2.3 -> 1.2.4 update is ~160MB this way vs the 46GB full reinstall the top-level patch field would claim.
// 2. overlay archive bundle (patch.patches[]), used when patch.json has no applicable variants for the installed version.
// 3. error "full reinstall required" if server has no diff path.

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use md5::{Digest, Md5};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use super::api;
use crate::downloads::{
    ControlSignal, DownloadEntry, DownloadKind, DownloadSource, DownloadStatus, check_control,
    report_progress, set_status,
};
use crate::gachas::manifest::GachaManifest;

pub struct EndfieldSource;

struct ParsedEndfieldApp {
    manifest: GachaManifest,
    edition_id: String,
    edition_label: String,
    cfg: api::EditionConfig,
}

fn parse_endfield_app(app_id: &str) -> Result<ParsedEndfieldApp> {
    let (manifest, edition_id, _) = crate::gachas::strategies::find_for_app_id(app_id)
        .ok_or_else(|| anyhow!("no manifest found for app_id: {}", app_id))?;
    let edition_label = manifest
        .editions
        .iter()
        .find(|e| e.id == edition_id)
        .map(|e| e.label.clone())
        .unwrap_or_else(|| edition_id.clone());
    let cfg = api::EditionConfig::from_manifest(&manifest, &edition_id)?;
    Ok(ParsedEndfieldApp {
        manifest,
        edition_id,
        edition_label,
        cfg,
    })
}

#[async_trait]
impl DownloadSource for EndfieldSource {
    async fn install(&self, entry: &DownloadEntry) -> Result<()> {
        let parsed = parse_endfield_app(&entry.app_id)?;
        tracing::info!("install: {} ({})", entry.display_name, parsed.edition_label);

        let resp = api::fetch_latest(&parsed.cfg, "").await?;
        let target_version = resp.version.clone();
        if target_version.is_empty() {
            return Err(anyhow!("get_latest returned no version"));
        }

        let packs = api::packs_from(&resp);
        if packs.is_empty() {
            return Err(anyhow!(
                "get_latest returned no packs for {} — server may require a different channel",
                parsed.edition_label
            ));
        }

        download_and_extract(entry, packs, "pack").await?;

        if check_control(&entry.id) != ControlSignal::None {
            return Ok(());
        }

        super::set_installed_version(
            &parsed.manifest.game_slug,
            &parsed.edition_id,
            &target_version,
        );
        tracing::info!(
            "install complete: {} {}",
            parsed.edition_label,
            target_version
        );
        Ok(())
    }

    async fn update(&self, entry: &DownloadEntry) -> Result<()> {
        let from_version = match &entry.kind {
            DownloadKind::Update { from_version } => from_version.clone(),
            _ => return Err(anyhow!("update() called on a non-update entry")),
        };

        let parsed = parse_endfield_app(&entry.app_id)?;
        tracing::info!(
            "update: {} {} -> latest",
            parsed.edition_label,
            from_version
        );

        let resp = api::fetch_latest(&parsed.cfg, &from_version).await?;
        let target_version = resp.version.clone();

        if target_version == from_version || target_version.is_empty() {
            return Ok(());
        }

        // rand_str comes from teh get_latest pkg.file_path and is needed for the resource patch end(field)point
        let rand = api::rand_str_from(&resp);
        let game_version = major_minor(&target_version);
        let mut tried_v2 = false;

        if !rand.is_empty() && !game_version.is_empty() {
            tried_v2 = true;
            match apply_resource_patches(entry, &parsed.cfg, &game_version, &target_version, &rand)
                .await
            {
                Ok(true) => {
                    super::set_installed_version(
                        &parsed.manifest.game_slug,
                        &parsed.edition_id,
                        &target_version,
                    );
                    tracing::info!(
                        "update complete via resource patches: {} -> {}",
                        from_version,
                        target_version
                    );
                    return Ok(());
                }
                Ok(false) => {
                    tracing::warn!(
                        "resource patch.json had no variants applicable to installed state, falling back"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "resource patch path failed: {} - falling back to overlay bundle",
                        e
                    );
                }
            }
        }

        if check_control(&entry.id) != ControlSignal::None {
            return Ok(());
        }

        let patches = api::patches_from(&resp);
        if patches.is_empty() {
            if tried_v2 {
                return Err(anyhow!(
                    "update failed: no applicable resource-level patches and no overlay bundle offered (full reinstall required)"
                ));
            }
            return Err(anyhow!(
                "no overlay patch path from {} to {} — full reinstall required",
                from_version,
                target_version
            ));
        }

        download_and_extract(entry, patches, "patch").await?;

        if check_control(&entry.id) != ControlSignal::None {
            return Ok(());
        }

        super::set_installed_version(
            &parsed.manifest.game_slug,
            &parsed.edition_id,
            &target_version,
        );
        tracing::info!(
            "update complete via overlay: {} -> {}",
            from_version,
            target_version
        );
        Ok(())
    }
}

async fn apply_resource_patches(
    entry: &DownloadEntry,
    cfg: &api::EditionConfig,
    game_version: &str,
    target_version: &str,
    rand_str: &str,
) -> Result<bool> {
    let resources = api::fetch_resources(cfg, game_version, target_version, rand_str).await?;
    if resources.resources.is_empty() {
        return Ok(false);
    }

    let safe_id = entry.app_id.replace(':', "-");
    let scratch_parent = match entry.temp_dir.as_deref() {
        Some(p) => p.to_path_buf(),
        None => entry
            .install_path
            .parent()
            .unwrap_or(&entry.install_path)
            .to_path_buf(),
    };
    let scratch_root = scratch_parent.join(format!(".omikuji-update-{}", safe_id));
    let _ = std::fs::create_dir_all(&scratch_root);
    let mut manifests: Vec<(api::ResourceRef, api::ResourcePatchManifest)> = Vec::new();
    for resource in &resources.resources {
        if check_control(&entry.id) != ControlSignal::None {
            return Ok(false);
        }
        tracing::debug!("fetching patch.json for resource {}", resource.name);
        match api::fetch_resource_patch(&resource.path).await {
            Ok(m) => {
                tracing::debug!(
                    "resource {} has {} file entries (target build {})",
                    resource.name,
                    m.files.len(),
                    m.version
                );
                manifests.push((resource.clone(), m));
            }
            Err(e) => {
                tracing::warn!("skip {}: {}", resource.name, e);
            }
        }
    }
    if manifests.is_empty() {
        return Ok(false);
    }

    let total_files: u64 = manifests.iter().map(|(_, m)| m.files.len() as u64).sum();
    let mut done: u64 = 0;
    let mut applied: u64 = 0;
    let mut already: u64 = 0;
    let mut not_materialized: u64 = 0;
    let mut unpatchable: u64 = 0;

    set_status(&entry.id, DownloadStatus::Patching);

    for (resource, manifest) in &manifests {
        let resource_scratch = scratch_root.join(&resource.name);
        let _ = std::fs::create_dir_all(&resource_scratch);

        for file in &manifest.files {
            if check_control(&entry.id) != ControlSignal::None {
                return Ok(unpatchable == 0);
            }

            let outcome = apply_one_file(entry, &resource.path, &resource_scratch, file).await;
            match outcome {
                Ok(FileOutcome::Applied) => applied += 1,
                Ok(FileOutcome::AlreadyCurrent) => already += 1,
                Ok(FileOutcome::NotMaterialized) => not_materialized += 1,
                Ok(FileOutcome::Unpatchable) => unpatchable += 1,
                Err(e) => {
                    tracing::warn!(
                        "hpatchz failed for {}: {} - counting as unpatchable",
                        file.name,
                        e
                    );
                    unpatchable += 1;
                }
            }
            done += 1;
            let pct = (done as f64 / total_files.max(1) as f64) * 100.0;
            report_progress(&entry.id, pct, 0, 0, 0);
        }
    }

    tracing::info!(
        "resource patch pass: {} applied, {} already at target, {} lazy-loaded (skipped ok), {} unpatchable (of {} total)",
        applied,
        already,
        not_materialized,
        unpatchable,
        total_files
    );

    // lazy-loaded bundle assets are fine; the game's own resource manager
    // fetches them on first access, they don't represent a broken update
    let success = unpatchable == 0;
    if success {
        let _ = std::fs::remove_dir_all(&scratch_root);
    }
    Ok(success)
}

#[derive(Debug)]
enum FileOutcome {
    Applied,
    AlreadyCurrent,
    NotMaterialized,
    Unpatchable,
}

const CANDIDATE_ROOTS: &[&str] = &[
    "Endfield_Data/Persistent/VFS",
    "",
    "Endfield_Data",
    "Endfield_Data/StreamingAssets",
    "Endfield_Data/StreamingAssets/aa",
    "Endfield_Data/StreamingAssets/aa/StandaloneWindows64",
];

async fn apply_one_file(
    entry: &DownloadEntry,
    resource_path_base: &str,
    scratch: &Path,
    file: &api::ResourcePatchFile,
) -> Result<FileOutcome> {
    if file.diff_type != 1 {
        tracing::warn!(
            "{} has diffType={}, skipping (unsupported)",
            file.name,
            file.diff_type
        );
        return Ok(FileOutcome::Unpatchable);
    }

    if let Some(target_abs) = resolve_on_disk(&entry.install_path, &file.name, &file.md5)? {
        tracing::debug!("{} already at target md5, skip", target_abs.display());
        return Ok(FileOutcome::AlreadyCurrent);
    }

    for variant in &file.patch {
        if let Some(base_abs) =
            resolve_on_disk(&entry.install_path, &variant.base_file, &variant.base_md5)?
        {
            return apply_variant(entry, resource_path_base, scratch, file, variant, &base_abs)
                .await;
        }
    }

    if !any_variant_base_exists_on_disk(&entry.install_path, file)
        && !file_exists_anywhere(&entry.install_path, &file.name)
    {
        tracing::debug!(
            "{} not materialized on disk (likely inside a .blc bundle, lazy-loaded) - skip, game will reconcile",
            file.name
        );
        return Ok(FileOutcome::NotMaterialized);
    }

    tracing::warn!(
        "UNPATCHABLE {} - target_md5={}, variants={}",
        file.name,
        file.md5,
        file.patch.len()
    );
    for (i, v) in file.patch.iter().enumerate() {
        tracing::debug!(
            "  variant[{}]: base_file={} base_md5={}",
            i,
            v.base_file,
            v.base_md5
        );
    }
    Ok(FileOutcome::Unpatchable)
}

fn file_exists_anywhere(install_path: &Path, rel_path: &str) -> bool {
    for root in CANDIDATE_ROOTS {
        let abs = if root.is_empty() {
            install_path.join(rel_path)
        } else {
            install_path.join(root).join(rel_path)
        };
        if abs.is_file() {
            return true;
        }
    }
    false
}

fn any_variant_base_exists_on_disk(install_path: &Path, file: &api::ResourcePatchFile) -> bool {
    file.patch
        .iter()
        .any(|v| file_exists_anywhere(install_path, &v.base_file))
}

async fn apply_variant(
    entry: &DownloadEntry,
    resource_path_base: &str,
    scratch: &Path,
    file: &api::ResourcePatchFile,
    variant: &api::ResourcePatchVariant,
    base_abs: &Path,
) -> Result<FileOutcome> {
    let url = format!(
        "{}/Patch/{}",
        resource_path_base.trim_end_matches('/'),
        variant.patch_path
    );
    let blob_name = variant
        .patch_path
        .rsplit('/')
        .next()
        .unwrap_or("patch.hdiff");
    let blob_path = scratch.join(blob_name);

    let need_dl = !matches!(std::fs::metadata(&blob_path), Ok(m) if m.len() == variant.patch_size);
    if need_dl {
        crate::hoyo::source::download_file(&url, &blob_path, &entry.id, 0, variant.patch_size)
            .await?;
    }

    let target_abs = install_path_for(
        &entry.install_path,
        base_abs,
        &variant.base_file,
        &file.name,
    )
    .ok_or_else(|| anyhow!("couldn't locate install root for {}", file.name))?;

    let tmp_out = scratch.join(format!("{}.out", blob_name));
    crate::external::hpatchz::patch(base_abs, &blob_path, &tmp_out)
        .map_err(|e| anyhow!("hpatchz({}): {}", file.name, e))?;

    let got = md5_of_file(&tmp_out)?;
    if got != file.md5.to_ascii_lowercase() {
        let _ = std::fs::remove_file(&tmp_out);
        return Err(anyhow!(
            "md5 mismatch after hpatchz for {}: expected {} got {}",
            file.name,
            file.md5,
            got
        ));
    }

    if let Some(parent) = target_abs.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::rename(&tmp_out, &target_abs).map_err(|e| {
        anyhow!(
            "rename {} -> {}: {}",
            tmp_out.display(),
            target_abs.display(),
            e
        )
    })?;

    let _ = std::fs::remove_file(&blob_path);
    Ok(FileOutcome::Applied)
}

fn resolve_on_disk(
    install_path: &Path,
    relative: &str,
    expected_md5: &str,
) -> Result<Option<PathBuf>> {
    for root in CANDIDATE_ROOTS {
        let abs = if root.is_empty() {
            install_path.join(relative)
        } else {
            install_path.join(root).join(relative)
        };
        if !abs.is_file() {
            continue;
        }
        let actual = md5_of_file(&abs).unwrap_or_default();
        if actual == expected_md5.to_ascii_lowercase() {
            return Ok(Some(abs));
        }
    }
    Ok(None)
}

fn install_path_for(
    install_path: &Path,
    base_abs: &Path,
    base_rel: &str,
    target_rel: &str,
) -> Option<PathBuf> {
    let rel_to_install = base_abs.strip_prefix(install_path).ok()?;
    let rel_str = rel_to_install.to_string_lossy().replace('\\', "/");
    let base_rel_norm = base_rel.replace('\\', "/");
    if !rel_str.ends_with(&base_rel_norm) {
        return Some(install_path.join(target_rel));
    }
    let prefix_len = rel_str.len() - base_rel_norm.len();
    let root = &rel_str[..prefix_len.saturating_sub(1)]; // drop trailing /
    if root.is_empty() {
        Some(install_path.join(target_rel))
    } else {
        Some(install_path.join(root).join(target_rel))
    }
}

fn md5_of_file(path: &Path) -> Result<String> {
    let mut f = File::open(path).map_err(|e| anyhow!("open {}: {}", path.display(), e))?;
    let mut hasher = Md5::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf).map_err(|e| anyhow!("read: {}", e))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn major_minor(semver: &str) -> String {
    let parts: Vec<&str> = semver.split('.').collect();
    if parts.len() >= 2 {
        format!("{}.{}", parts[0], parts[1])
    } else {
        semver.to_string()
    }
}

async fn download_and_extract(
    entry: &DownloadEntry,
    files: &[api::PackFile],
    label: &str,
) -> Result<()> {
    let safe_id = entry.app_id.replace(':', "-");
    let scratch_parent = match entry.temp_dir.as_deref() {
        Some(p) => p.to_path_buf(),
        None => entry
            .install_path
            .parent()
            .unwrap_or(&entry.install_path)
            .to_path_buf(),
    };
    let temp_dir = scratch_parent.join(format!(".omikuji-dl-{}", safe_id));
    let _ = std::fs::create_dir_all(&temp_dir);

    let total_bytes: u64 = files.iter().map(|f| f.package_size).sum();
    let mut so_far: u64 = 0;

    let mut first_segment: Option<std::path::PathBuf> = None;
    for f in files {
        if check_control(&entry.id) != ControlSignal::None {
            return Ok(());
        }
        let filename = f
            .url
            .rsplit('/')
            .next()
            .unwrap_or("endfield.zip")
            .to_string();
        let temp_path = temp_dir.join(&filename);
        if first_segment.is_none() {
            first_segment = Some(temp_path.clone());
        }

        tracing::debug!(
            "{} segment: {} ({})",
            label,
            filename,
            format_bytes(f.package_size)
        );

        crate::hoyo::source::download_file(&f.url, &temp_path, &entry.id, so_far, total_bytes)
            .await?;

        so_far += f.package_size;
    }

    if check_control(&entry.id) != ControlSignal::None {
        return Ok(());
    }

    if let Some(first) = &first_segment {
        tracing::info!("extracting {} to {}", label, entry.install_path.display());
        crate::notifications::info(
            &entry.display_name,
            if label == "patch" {
                "Applying update…"
            } else {
                "Extracting game files…"
            },
        );
        set_status(&entry.id, DownloadStatus::Extracting);
        crate::hoyo::source::extract_archive(first, &entry.install_path, Some(&entry.id))?;

        for f in files {
            let fname = f.url.rsplit('/').next().unwrap_or("endfield.zip");
            let _ = std::fs::remove_file(temp_dir.join(fname));
        }
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

pub fn cleanup_endfield_state(app_id: &str, install_path: &Path, temp_dir: Option<&Path>) {
    let safe_id = app_id.replace(':', "-");
    let dirname = format!(".omikuji-dl-{}", safe_id);
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Some(t) = temp_dir {
        candidates.push(t.join(&dirname));
    }
    candidates.push(install_path.parent().unwrap_or(install_path).join(&dirname));
    for dir in candidates {
        if dir.exists()
            && let Err(e) = std::fs::remove_dir_all(&dir)
        {
            tracing::warn!("clean {} failed: {}", dir.display(), e);
        }
    }
}

// returns total bytes across files, number of zip-segment files
pub fn inspect_endfield_temp(
    app_id: &str,
    install_path: &Path,
    temp_dir: Option<&Path>,
) -> (u64, u32) {
    let safe_id = app_id.replace(':', "-");
    let dirname = format!(".omikuji-dl-{}", safe_id);
    let candidates: Vec<std::path::PathBuf> = {
        let mut v = Vec::new();
        if let Some(t) = temp_dir {
            v.push(t.join(&dirname));
        }
        v.push(install_path.parent().unwrap_or(install_path).join(&dirname));
        v
    };

    let mut total_bytes: u64 = 0;
    let mut segments: u32 = 0;
    for dir in &candidates {
        if !dir.is_dir() {
            continue;
        }
        let Ok(rd) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in rd.flatten() {
            if let Ok(meta) = entry.metadata()
                && meta.is_file()
            {
                total_bytes += meta.len();
                let name = entry.file_name().to_string_lossy().to_string();
                if name.contains(".zip.") {
                    segments += 1;
                }
            }
        }
    }
    (total_bytes, segments)
}

fn format_bytes(n: u64) -> String {
    if n >= 1_073_741_824 {
        format!("{:.1} GiB", n as f64 / 1_073_741_824.0)
    } else if n >= 1_048_576 {
        format!("{:.1} MiB", n as f64 / 1_048_576.0)
    } else {
        format!("{:.0} KiB", n as f64 / 1024.0)
    }
}
