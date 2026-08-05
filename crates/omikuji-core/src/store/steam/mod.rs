pub mod api;
pub mod local;
pub mod shortcuts;

use anyhow::Result;

pub use api::{SteamApi, SteamGame};
pub use local::{
    AppManifest, SteamUser, find_steam_dir, get_active_steamid64, get_installed_games,
    get_steam_users,
};

pub fn is_steam_installed() -> bool {
    find_steam_dir().is_some()
}

pub fn synthetic_appid(game_id: &str) -> u32 {
    let mut h: u32 = 2166136261;
    for b in game_id.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(16777619);
    }
    1_000_000_000 + (h % 1_000_000_000)
}

pub type SteamPlaytimeMap = std::collections::HashMap<String, (f64, u64)>;

// empty api_key means no remote sync, local process-tracking still populates playtime
// blocking http; call from std::thread::spawn, panics inside tokio runtime
pub fn fetch_playtime_data(api_key: &str) -> Result<SteamPlaytimeMap> {
    if api_key.is_empty() {
        return Ok(SteamPlaytimeMap::default());
    }

    let steamid = get_active_steamid64().ok_or_else(|| anyhow::anyhow!("no steam user found"))?;

    let api = SteamApi::with_key(api_key.to_string());
    let steam_games = api.get_owned_games(&steamid)?;

    Ok(steam_games
        .into_iter()
        .filter_map(|g| {
            let appid = g.appid.to_string();
            let playtime = g.playtime_forever.map(|m| m as f64 / 60.0);
            let last_played = g.rtime_last_played.unwrap_or(0);
            playtime.map(|p| (appid, (p, last_played)))
        })
        .collect())
}

pub fn apply_playtime_data(
    library: &mut crate::library::Library,
    steam_data: &SteamPlaytimeMap,
) -> (usize, usize) {
    let mut updated = 0;
    let steam_game_count = library
        .game
        .iter()
        .filter(|g| g.runner.runner_type == "steam")
        .count();

    for game in &mut library.game {
        if game.runner.runner_type != "steam" {
            continue;
        }

        let appid = &game.metadata.id;
        if let Some((playtime_hours, last_played_ts)) = steam_data.get(appid) {
            game.metadata.playtime = *playtime_hours;

            if *last_played_ts > 0 {
                let datetime = chrono::DateTime::from_timestamp(*last_played_ts as i64, 0)
                    .map(|dt| dt.format("%b %d, %Y").to_string())
                    .unwrap_or_default();
                game.metadata.last_played = datetime;
            }

            updated += 1;
        }
    }

    (updated, steam_game_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steam_installed() {
        let installed = is_steam_installed();
        println!("steam installed: {}", installed);
    }

    #[test]
    fn test_get_current_user() {
        if let Some(user) = get_steam_users().into_iter().next() {
            println!("current user: {:?}", user);
        } else {
            println!("no steam user found");
        }
    }
}
