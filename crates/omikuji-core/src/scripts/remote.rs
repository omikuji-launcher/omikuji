use super::Script;
use crate::http::{ResponseExt, USER_AGENT};
use crate::settings;
use anyhow::{Context, Result, bail};
use reqwest::blocking;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct RemoteScript {
    pub author: String,
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub has_shell: bool,
    #[serde(default)]
    pub modified: String,
    pub toml: String,
    #[serde(default)]
    pub icon: String,
}

pub fn fetch_base() -> String {
    settings::get()
        .scripts
        .fetch_url
        .trim_end_matches('/')
        .to_string()
}

fn get(url: &str) -> Result<blocking::Response> {
    let resp = blocking::Client::builder()
        .user_agent(USER_AGENT)
        .build()?
        .get(url)
        .send()
        .with_context(|| format!("requesting {url}"))?;
    Ok(resp.check()?)
}

pub fn fetch_index() -> Result<Vec<RemoteScript>> {
    let base = fetch_base();
    if base.is_empty() {
        return Ok(Vec::new());
    }
    let url = format!("{base}/index.json");
    let list = get(&url)?
        .json::<Vec<RemoteScript>>()
        .context("invalid index.json")?;
    Ok(list)
}

fn plain_name(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('.') && !name.contains(['/', '\\'])
}

pub fn install_remote(entry: &RemoteScript) -> Result<PathBuf> {
    let base = fetch_base();
    if base.is_empty() {
        bail!("scripts fetch url is not configured");
    }
    if !plain_name(&entry.author) || !plain_name(&entry.slug) {
        bail!(
            "refusing suspicious script path {}/{}",
            entry.author,
            entry.slug
        );
    }
    let toml_name = entry
        .toml
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_string();
    if !plain_name(&toml_name) {
        bail!("refusing suspicious script file name {toml_name}");
    }

    let url = format!("{base}/{}", entry.toml);
    let text = get(&url)?.text()?;
    Script::parse(&text)?;

    let dir = crate::scripts_dir().join(&entry.author).join(&entry.slug);
    fs_err::create_dir_all(&dir)?;
    let toml_path = dir.join(toml_name);
    fs_err::write(&toml_path, &text)?;

    if !entry.icon.is_empty() {
        let icon_name = entry
            .icon
            .rsplit('/')
            .next()
            .unwrap_or_default()
            .to_string();
        if plain_name(&icon_name) {
            let fetched = get(&format!("{base}/{}", entry.icon)).and_then(|r| Ok(r.bytes()?));
            match fetched {
                Ok(bytes) => {
                    let _ = fs_err::write(dir.join(icon_name), &bytes);
                }
                Err(e) => tracing::warn!("script icon fetch failed: {e}"),
            }
        }
    }
    Ok(toml_path)
}
