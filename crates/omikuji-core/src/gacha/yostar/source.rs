use anyhow::{Result, anyhow};
use async_trait::async_trait;

use super::api;
use crate::downloads::{DownloadEntry, DownloadKind, DownloadSource};
use crate::gacha::file_sync::{self, SyncProgress};

pub struct YostarSource;

#[async_trait]
impl DownloadSource for YostarSource {
    async fn install(&self, entry: &DownloadEntry) -> Result<()> {
        run_sync(entry).await
    }

    async fn update(&self, entry: &DownloadEntry) -> Result<()> {
        run_sync(entry).await
    }

    fn supports_import(&self) -> bool {
        true
    }

    async fn import_existing(&self, entry: &DownloadEntry) -> Result<()> {
        run_sync(entry).await
    }
}

// an update is the same walk; no deletion list, so dropped files linger
async fn run_sync(entry: &DownloadEntry) -> Result<()> {
    if !matches!(
        entry.kind,
        DownloadKind::Install | DownloadKind::Update { .. } | DownloadKind::ImportExisting
    ) {
        return Err(anyhow!("YostarSource: unexpected DownloadKind"));
    }

    let (manifest, edition_id, _) = crate::gacha::strategies::find_for_app_id(&entry.app_id)
        .ok_or_else(|| anyhow!("no manifest for app_id {}", entry.app_id))?;

    if matches!(entry.kind, DownloadKind::ImportExisting) {
        super::verify_edition_on_disk(&manifest, &edition_id, &entry.install_path)?;
    }

    let api = super::edition_api(&manifest, &edition_id)?;
    let config = api::fetch_game_config(&api).await?;
    let index = api::fetch_file_index(&api, &config).await?;
    if index.files.is_empty() {
        return Err(anyhow!("file index returned zero files"));
    }

    let install_root = entry.install_path.clone();
    std::fs::create_dir_all(&install_root)?;

    let progress = SyncProgress::new(index.total_size());
    if !file_sync::sync_all(&entry.id, index.sync_files(), &install_root, progress).await? {
        return Ok(());
    }

    super::set_installed_version(&manifest.game_slug, &edition_id, &config.version);
    Ok(())
}
