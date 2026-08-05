use std::path::PathBuf;
use std::sync::Mutex;

use super::manifest::GachaManifest;

const EXTENSIONS: &[&str] = &["png", "jpg", "ico"];

fn in_flight() -> &'static Mutex<std::collections::HashSet<String>> {
    use std::sync::OnceLock;
    static SET: OnceLock<Mutex<std::collections::HashSet<String>>> = OnceLock::new();
    SET.get_or_init(|| Mutex::new(std::collections::HashSet::new()))
}

fn cache_dir_for(publisher_slug: &str, game_slug: &str) -> PathBuf {
    crate::cache_dir()
        .join("images")
        .join("gachas")
        .join(publisher_slug)
        .join(game_slug)
}

fn cached_url(publisher_slug: &str, game_slug: &str, kind: &str) -> Option<String> {
    let dir = cache_dir_for(publisher_slug, game_slug);
    for ext in EXTENSIONS {
        let p = dir.join(format!("{}.{}", kind, ext));
        if p.exists() {
            return Some(format!("file://{}", p.display()));
        }
    }
    None
}

pub fn resolve_art(manifest: &GachaManifest, kind: &str) -> String {
    if let Some(url) = cached_url(&manifest.publisher_slug, &manifest.game_slug, kind) {
        return url;
    }

    let key = format!(
        "{}:{}:{}",
        manifest.publisher_slug, manifest.game_slug, kind
    );
    {
        let mut set = in_flight().lock().unwrap();
        if set.contains(&key) {
            return String::new();
        }
        set.insert(key.clone());
    }

    let pub_s = manifest.publisher_slug.clone();
    let game_s = manifest.game_slug.clone();
    let kind_s = kind.to_string();
    std::thread::spawn(move || {
        let result = fetch_first_match(&pub_s, &game_s, &kind_s);
        if let Err(e) = result {
            tracing::error!("failed to fetch {}/{} {}: {}", pub_s, game_s, kind_s, e);
        }
        in_flight().lock().unwrap().remove(&key);
    });

    String::new()
}

pub fn fetch_into_library_cache(
    manifest: &GachaManifest,
    game_id: &str,
    mut on_asset: impl FnMut(&crate::media::MediaType),
) {
    let library_dir = crate::cache_dir().join("images");
    if let Err(e) = std::fs::create_dir_all(&library_dir) {
        tracing::error!("failed to create library cache dir: {}", e);
        return;
    }

    let pairs: &[(&str, crate::media::MediaType)] = &[
        ("hero", crate::media::MediaType::Banner),
        ("grid", crate::media::MediaType::Coverart),
        ("icon", crate::media::MediaType::Icon),
    ];

    for (gacha_kind, lib_type) in pairs {
        if cached_url(&manifest.publisher_slug, &manifest.game_slug, gacha_kind).is_none()
            && let Err(e) =
                fetch_first_match(&manifest.publisher_slug, &manifest.game_slug, gacha_kind)
        {
            tracing::error!("{} fetch failed: {}", gacha_kind, e);
            continue;
        }

        let gacha_dir = cache_dir_for(&manifest.publisher_slug, &manifest.game_slug);
        for ext in EXTENSIONS {
            let src = gacha_dir.join(format!("{}.{}", gacha_kind, ext));
            if !src.exists() {
                continue;
            }
            let dst = library_dir.join(format!(
                "{}_{}.{}",
                game_id,
                lib_type.suffix(),
                lib_type.extension()
            ));
            match std::fs::copy(&src, &dst) {
                Ok(_) => on_asset(lib_type),
                Err(e) => {
                    tracing::error!("copy {} -> {} failed: {}", src.display(), dst.display(), e)
                }
            }
            break;
        }
    }
}

fn fetch_first_match(publisher_slug: &str, game_slug: &str, kind: &str) -> anyhow::Result<()> {
    let base = crate::settings::get().assets.fetch_url.trim().to_string();
    if base.is_empty() {
        return Err(anyhow::anyhow!("assets.fetch_url is empty"));
    }
    let base = base.trim_end_matches('/');

    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!("omikuji/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let dir = cache_dir_for(publisher_slug, game_slug);
    std::fs::create_dir_all(&dir)?;

    let mut last_status = None;
    for ext in EXTENSIONS {
        let url = format!(
            "{}/gacha/{}/{}/{}.{}",
            base, publisher_slug, game_slug, kind, ext
        );
        let resp = match client.get(&url).send() {
            Ok(r) => r,
            Err(e) => {
                last_status = Some(format!("network: {}", e));
                continue;
            }
        };
        if !resp.status().is_success() {
            last_status = Some(format!("http {} on {}", resp.status(), url));
            continue;
        }
        let bytes = resp.bytes()?;
        let path = dir.join(format!("{}.{}", kind, ext));
        let tmp = path.with_extension(format!("{}.tmp", ext));
        std::fs::write(&tmp, &bytes)?;
        std::fs::rename(&tmp, &path)?;
        return Ok(());
    }
    Err(anyhow::anyhow!(
        "no matching extension found; last: {}",
        last_status.unwrap_or_else(|| "unknown".into())
    ))
}
