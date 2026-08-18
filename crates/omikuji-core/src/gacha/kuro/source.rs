use anyhow::{Result, anyhow};
use async_trait::async_trait;

use super::api;
use crate::downloads::{DownloadEntry, DownloadKind, DownloadSource};
use crate::gacha::file_sync::{self, SyncFile, SyncProgress};

pub struct KuroSource;

#[async_trait]
impl DownloadSource for KuroSource {
    async fn install(&self, entry: &DownloadEntry) -> Result<()> {
        run_install_or_update(entry).await
    }

    async fn update(&self, entry: &DownloadEntry) -> Result<()> {
        run_install_or_update(entry).await
    }
}

// pgr ships leading-slash paths and wuwa doesnt; only the url half needs them stripped
pub(super) fn sync_file(file: &api::ResourceFile, base_url: &str) -> SyncFile {
    let clean = file.dest.trim_start_matches(['/', '\\']);
    SyncFile {
        rel_path: file.dest.clone(),
        size: file.size,
        url: format!("{}{}", base_url, clean.replace(' ', "%20")),
    }
}

async fn run_install_or_update(entry: &DownloadEntry) -> Result<()> {
    if !matches!(
        entry.kind,
        DownloadKind::Install | DownloadKind::Update { .. }
    ) {
        return Err(anyhow!("KuroSource: unexpected DownloadKind"));
    }

    let (manifest, edition_id, _) = crate::gacha::strategies::find_for_app_id(&entry.app_id)
        .ok_or_else(|| anyhow!("no manifest for app_id {}", entry.app_id))?;
    let game_slug = manifest.game_slug.clone();

    let info = api::fetch_resource_info(&manifest, &edition_id).await?;

    if let DownloadKind::Update { from_version } = &entry.kind
        && let Some(pc) = info.matching_patch(from_version)
    {
        match super::patcher::run_patch_update(entry, &info, pc).await {
            Ok(true) => {
                super::set_installed_version(&game_slug, &edition_id, &info.version);
                return Ok(());
            }
            Ok(false) => return Ok(()),
            Err(e) => tracing::warn!("kuro delta update failed, falling back to full sync: {}", e),
        }
    }

    let index = api::fetch_index_file(&info.index_file_url).await?;
    if index.resource.is_empty() {
        return Err(anyhow!("indexFile returned zero resources"));
    }

    let install_root = entry.install_path.clone();
    std::fs::create_dir_all(&install_root)?;

    let total: u64 = index.resource.iter().map(|r| r.size).sum();
    let files: Vec<SyncFile> = index
        .resource
        .iter()
        .map(|f| sync_file(f, &info.base_url))
        .collect();

    let progress = SyncProgress::new(total);
    if !file_sync::sync_all(&entry.id, files, &install_root, progress).await? {
        return Ok(());
    }

    for stale in &index.delete_files {
        let p = install_root.join(file_sync::sanitize_rel(stale));
        if p.exists() {
            let _ = std::fs::remove_file(&p);
        }
    }
    super::set_installed_version(&game_slug, &edition_id, &info.version);
    Ok(())
}
