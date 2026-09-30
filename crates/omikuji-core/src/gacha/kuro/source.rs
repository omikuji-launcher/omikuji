use std::iter;
use std::sync::Arc;

use anyhow::{Result, anyhow, bail};
use async_trait::async_trait;

use super::api::{self, KuroIndex, Pack};
use super::{KuroConfig, patcher};
use crate::downloads::{
    ControlSignal, DownloadEntry, DownloadKind, DownloadSource, DownloadStatus, check_control,
    set_status,
};
use crate::gacha::file_sync::{self, Skip, SyncFile, SyncProgress};
use crate::gacha::manifest::GachaManifest;
use crate::gacha::strategies::normalize_version;
use crate::gacha::{state, strategies};

pub struct KuroSource;

#[async_trait]
impl DownloadSource for KuroSource {
    fn cleanup_state(&self, entry: &DownloadEntry) {
        patcher::discard_staging(&entry.install_path);
        if let DownloadKind::AddPack { pack } = &entry.kind {
            discard_partial_pack(entry, pack);
        }
    }

    async fn install(&self, entry: &DownloadEntry) -> Result<()> {
        run_sync(entry).await
    }

    async fn update(&self, entry: &DownloadEntry) -> Result<()> {
        run_sync(entry).await
    }

    fn supports_repair(&self) -> bool {
        true
    }

    async fn repair(&self, entry: &DownloadEntry) -> Result<()> {
        run_sync(entry).await
    }

    async fn add_pack(&self, entry: &DownloadEntry, pack: &str) -> Result<()> {
        let target = Target::resolve(&entry.app_id).await?;
        target.config.pack(pack)?;
        let remote = target.index.pack(pack)?;
        let edition = target.manifest.require_edition(&target.edition_id)?;
        if let Some(installed) =
            super::read_install_version(&entry.install_path, &edition.data_folder)
            && normalize_version(&installed) != normalize_version(&remote.version)
        {
            bail!(
                "update {} to {} before adding packs",
                target.manifest.display_name,
                remote.version
            );
        }
        sync_pack(entry, pack, remote, None).await
    }
}

struct Target {
    manifest: GachaManifest,
    edition_id: String,
    config: KuroConfig,
    index: KuroIndex,
}

impl Target {
    async fn resolve(app_id: &str) -> Result<Self> {
        let (manifest, edition_id) = strategies::find_for_app_id(app_id)
            .ok_or_else(|| anyhow!("no manifest for app_id {app_id}"))?;
        let config = KuroConfig::load(&manifest, &edition_id)?;
        let index = api::fetch_index(&config.index_url).await?;
        Ok(Self {
            manifest,
            edition_id,
            config,
            index,
        })
    }
}

fn discard_partial_pack(entry: &DownloadEntry, pack: &str) {
    let removed = strategies::find_for_app_id(&entry.app_id)
        .ok_or_else(|| anyhow!("no manifest for app_id {}", entry.app_id))
        .and_then(|(manifest, edition_id)| KuroConfig::load(&manifest, &edition_id))
        .and_then(|config| config.remove_pack(&entry.install_path, pack));
    if let Err(e) = removed {
        tracing::warn!("discarding the partial {pack} pack: {e:#}");
    }
}

// pgr ships leading-slash paths and wuwa doesnt; only the url half needs them stripped
pub(super) fn sync_file(file: &api::ResourceFile, base_url: &str) -> SyncFile {
    let clean = file.dest.trim_start_matches(['/', '\\']);
    SyncFile {
        rel_path: file.dest.clone(),
        size: file.size,
        url: format!("{}{}", base_url, clean.replace(' ', "%20")),
        md5: file_sync::expected_md5(&file.md5),
    }
}

async fn run_sync(entry: &DownloadEntry) -> Result<()> {
    if !matches!(
        entry.kind,
        DownloadKind::Install | DownloadKind::Update { .. } | DownloadKind::Repair
    ) {
        return Err(anyhow!("KuroSource: unexpected DownloadKind"));
    }

    let Target {
        manifest,
        edition_id,
        config,
        index,
    } = Target::resolve(&entry.app_id).await?;
    let base_id = index.base_id()?;

    let tiers: Vec<&str> = match entry.kind {
        DownloadKind::Install => config.install_packs(&entry.packs, &manifest.display_name)?,
        _ => config.installed_packs(&entry.install_path).collect(),
    };
    let pack_ids: Vec<&str> = iter::once(base_id).chain(tiers).collect();
    let total = pack_ids
        .iter()
        .map(|id| index.pack(id).map(|p| p.download_bytes))
        .sum::<Result<u64>>()?;
    // one bar across every pack otherwise it restarts at 0% per pack and looks like a second download
    let shared = (matches!(entry.kind, DownloadKind::Install) && total > 0)
        .then(|| SyncProgress::new(total));

    for pack_id in pack_ids {
        let pack = index.pack(pack_id)?;
        let stamp = format!("{} {}", entry.id, pack.version);
        if patcher::is_pack_done(&entry.install_path, pack_id, &stamp) {
            if let Some(progress) = &shared {
                progress.advance(&entry.id, pack.download_bytes);
            }
            continue;
        }
        sync_pack(entry, pack_id, pack, shared.as_ref()).await?;
        if check_control(&entry.id) != ControlSignal::None {
            return Ok(());
        }
        patcher::mark_pack_done(&entry.install_path, pack_id, &stamp)?;
    }

    patcher::discard_staging(&entry.install_path);
    state::write_installed_version(&manifest.game_slug, &edition_id, &index.base()?.version);
    Ok(())
}

async fn sync_pack(
    entry: &DownloadEntry,
    pack_id: &str,
    pack: &Pack,
    shared: Option<&Arc<SyncProgress>>,
) -> Result<()> {
    let mut patch_stale: Vec<String> = Vec::new();
    // NOTE: lets hope this works 😭
    if let DownloadKind::Update { from_version } = &entry.kind
        && let Some(pc) = pack.matching_patch(from_version)
    {
        let index_url = format!("{}{}", pack.cdn_url, pc.index_file_rel);
        match api::fetch_patch_index(&index_url).await {
            Ok(pidx) => match patcher::run_patch_update(entry, pack_id, pack, pc, &pidx).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    tracing::warn!(
                        "kuro {pack_id} delta update failed, falling back to full sync: {e}"
                    );
                    patch_stale = pidx.delete_files.clone();
                }
            },
            Err(e) => {
                tracing::warn!(
                    "kuro {pack_id} patch index fetch failed, falling back to full sync: {e}"
                )
            }
        }
    }

    let index = api::fetch_index_file(&pack.index_file_url).await?;
    if index.resource.is_empty() {
        return Err(anyhow!("{pack_id} indexFile returned zero resources"));
    }

    let install_root = entry.install_path.clone();
    fs_err::create_dir_all(&install_root)?;

    let mut files: Vec<SyncFile> = index
        .resource
        .iter()
        .map(|f| sync_file(f, &pack.base_url))
        .collect();

    let verified = !matches!(
        entry.kind,
        DownloadKind::Install | DownloadKind::AddPack { .. }
    );
    if verified {
        set_status(&entry.id, DownloadStatus::Verifying);
        let Some(stale) = file_sync::select_stale(&entry.id, files, &install_root).await? else {
            return Ok(());
        };
        files = stale;
    }

    set_status(&entry.id, DownloadStatus::Downloading);
    let total: u64 = files.iter().map(|f| f.size).sum();
    let progress = shared.cloned().unwrap_or_else(|| SyncProgress::new(total));
    let skip = if verified {
        Skip::Never
    } else {
        Skip::SameSize
    };
    if !file_sync::sync_all(&entry.id, files, &install_root, progress, skip).await? {
        return Ok(());
    }

    for stale in index.delete_files.iter().chain(patch_stale.iter()) {
        let p = install_root.join(file_sync::sanitize_rel(stale));
        if p.exists() {
            let _ = fs_err::remove_file(&p);
        }
    }
    Ok(())
}
