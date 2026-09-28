use std::path::Path;

use reqwest::blocking;

use super::manifest::GachaManifest;
use crate::fs_util::write_atomic;
use crate::media::{MediaSlot, MediaType, media_path_in};
use crate::settings;

const KINDS: &[(&str, MediaType)] = &[
    ("hero", MediaType::Banner),
    ("grid", MediaType::Coverart),
    ("icon", MediaType::Icon),
];

fn asset_url(game_slug: &str, kind: &str) -> String {
    let base = settings::get().assets.fetch_url.trim().to_string();
    if base.is_empty() {
        return String::new();
    }
    format!(
        "{}/gacha/{}/{}.png",
        base.trim_end_matches('/'),
        game_slug,
        kind
    )
}

pub fn resolve_art(manifest: &GachaManifest, kind: &str) -> String {
    asset_url(&manifest.game_slug, kind)
}

pub fn fetch_into_library_cache(
    slot: MediaSlot,
    manifest: &GachaManifest,
    game_id: &str,
    mut on_asset: impl FnMut(&MediaType),
) {
    let client = match blocking::Client::builder()
        .user_agent(concat!("omikuji/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(30))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("art client build failed: {}", e);
            return;
        }
    };

    for (kind, lib_type) in KINDS {
        let url = resolve_art(manifest, kind);
        if url.is_empty() {
            tracing::error!("assets.fetch_url is empty");
            return;
        }
        match download_to(&client, &url, &media_path_in(slot, game_id, lib_type)) {
            Ok(()) => on_asset(lib_type),
            Err(e) => tracing::error!("{} fetch failed: {}", kind, e),
        }
    }
}

fn download_to(client: &blocking::Client, url: &str, dest: &Path) -> anyhow::Result<()> {
    let resp = client.get(url).send()?;
    if !resp.status().is_success() {
        anyhow::bail!("http {} on {}", resp.status(), url);
    }
    write_atomic(dest, &resp.bytes()?)?;
    Ok(())
}
