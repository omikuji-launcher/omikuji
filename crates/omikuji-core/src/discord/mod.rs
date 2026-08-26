use anyhow::Result;
use discord_rich_presence::{
    DiscordIpc, DiscordIpcClient,
    activity::{Activity, ActivityType, Assets, StatusDisplayType, Timestamps},
};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::library::Game;

const APP_ID: &str = "1503896994018623709";
const LAUNCHER_LOGO: &str =
    "https://raw.githubusercontent.com/reakjra/omikuji/master/crates/omikuji/qml/icons/app.png";

fn client_cell() -> &'static Mutex<Option<DiscordIpcClient>> {
    static CELL: OnceLock<Mutex<Option<DiscordIpcClient>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(None))
}

fn url_cache() -> &'static Mutex<HashMap<String, Option<String>>> {
    static CELL: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn set_playing(game: &Game) {
    if !game.system.discord_rpc {
        return;
    }

    let game = game.clone();
    std::thread::spawn(move || {
        let image = image_url_for(&game).unwrap_or_else(|| LAUNCHER_LOGO.to_string());
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let mut activity = Activity::new()
            .name(&game.metadata.name)
            .activity_type(ActivityType::Playing)
            .status_display_type(StatusDisplayType::Name)
            .timestamps(Timestamps::new().start(now))
            .assets(
                Assets::new()
                    .large_image(&image)
                    .large_text(&game.metadata.name),
            );

        if crate::app_settings::AppSettings::load()
            .behavior
            .discord_show_launcher
        {
            activity = activity.state("Playing on Omikuji");
        }

        if let Err(e) = send_activity(activity) {
            tracing::error!("set_playing failed: {}", e);
        }
    });
}

pub fn clear() {
    std::thread::spawn(|| {
        if let Err(e) = clear_activity_inner() {
            tracing::error!("clear failed: {}", e);
        }
    });
}

fn send_activity(activity: Activity) -> Result<()> {
    let mut guard = client_cell().lock().unwrap();
    if guard.is_none() {
        let mut c = DiscordIpcClient::new(APP_ID);
        c.connect().map_err(|e| anyhow::anyhow!("{}", e))?;
        *guard = Some(c);
    }
    if let Some(c) = guard.as_mut() {
        c.set_activity(activity)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
    }
    Ok(())
}

fn clear_activity_inner() -> Result<()> {
    let mut guard = client_cell().lock().unwrap();
    if let Some(c) = guard.as_mut() {
        c.clear_activity().map_err(|e| anyhow::anyhow!("{}", e))?;
    }
    Ok(())
}

fn image_url_for(game: &Game) -> Option<String> {
    let key = if !game.metadata.slug.is_empty() {
        game.metadata.slug.clone()
    } else {
        crate::media::slugify(&game.metadata.name)
    };

    {
        let cache = url_cache().lock().unwrap();
        if let Some(cached) = cache.get(&key) {
            return cached.clone();
        }
    }

    let result = crate::media::sgdb_icon_url(&game.metadata.name)
        .ok()
        .flatten();
    url_cache().lock().unwrap().insert(key, result.clone());
    result
}
