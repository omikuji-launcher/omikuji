use anyhow::{Context, Result};
use serde::Deserialize;

const API_BASE: &str = "https://api.steampowered.com";

pub struct SteamApi {
    api_key: String,
}

impl SteamApi {
    pub fn with_key(api_key: String) -> Self {
        Self { api_key }
    }

    pub fn get_owned_games(&self, steamid: &str) -> Result<Vec<SteamGame>> {
        let url = format!(
            "{}/IPlayerService/GetOwnedGames/v0001/?key={}&steamid={}&format=json&include_appinfo=1&include_played_free_games=1",
            API_BASE, self.api_key, steamid
        );

        let resp =
            reqwest::blocking::get(&url).with_context(|| "requesting steam api".to_string())?;

        if !resp.status().is_success() {
            anyhow::bail!(
                "steam api returned {}: {}",
                resp.status(),
                resp.text().unwrap_or_default()
            );
        }

        let data: ApiResponse = resp.json().with_context(|| "parsing steam api response")?;

        Ok(data.response.games.unwrap_or_default())
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    response: ResponseData,
}

#[derive(Debug, Deserialize)]
struct ResponseData {
    games: Option<Vec<SteamGame>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SteamGame {
    pub appid: u64,
    pub name: Option<String>,
    pub playtime_forever: Option<u32>,
    pub playtime_windows_forever: Option<u32>,
    pub playtime_mac_forever: Option<u32>,
    pub playtime_linux_forever: Option<u32>,
    pub rtime_last_played: Option<u64>,
    pub img_icon_url: Option<String>,
    pub img_logo_url: Option<String>,
    pub has_community_visible_stats: Option<bool>,
    pub content_descriptorids: Option<Vec<u32>>,
    pub has_leaderboards: Option<bool>,
}

impl SteamGame {
    pub fn store_url(&self) -> String {
        format!("https://store.steampowered.com/app/{}", self.appid)
    }

    pub fn capsule_image_url(&self) -> String {
        format!(
            "https://cdn.akamai.steamstatic.com/steam/apps/{}/capsule_184x69.jpg",
            self.appid
        )
    }

    pub fn library_image_url(&self) -> String {
        format!(
            "https://cdn.steamstatic.com/steam/apps/{}/library_600x900.jpg",
            self.appid
        )
    }

    pub fn header_image_url(&self) -> String {
        format!(
            "https://cdn.cloudflare.steamstatic.com/steam/apps/{}/header.jpg",
            self.appid
        )
    }

    pub fn icon_url(&self) -> Option<String> {
        self.img_icon_url.as_ref().map(|hash| {
            format!(
                "http://media.steampowered.com/steamcommunity/public/images/apps/{}/{hash}.jpg",
                self.appid
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_urls() {
        let game = SteamGame {
            appid: 730, // cs2
            name: Some("Counter-Strike 2".to_string()),
            playtime_forever: Some(1000),
            playtime_windows_forever: None,
            playtime_mac_forever: None,
            playtime_linux_forever: None,
            rtime_last_played: None,
            img_icon_url: Some("hash123".to_string()),
            img_logo_url: None,
            has_community_visible_stats: None,
            content_descriptorids: None,
            has_leaderboards: None,
        };

        assert!(game.store_url().contains("730"));
        assert!(game.capsule_image_url().contains("730"));
        assert!(game.library_image_url().contains("730"));
    }
}
