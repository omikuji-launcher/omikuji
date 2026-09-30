use anyhow::{Result, anyhow};
use serde::Serialize;

use super::kuro::KuroConfig;
use super::manifest::{GachaManifest, LaunchEffect, ManifestOption};
use super::strategies::{self, InstallStrategy};
use crate::library::{Game, LaunchConfig};

pub const PACKS_CONTROL: &str = "packs";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlKind {
    Toggle,
    Choice,
}

#[derive(Debug, Clone, Serialize)]
pub struct ControlChoice {
    pub id: String,
    pub label: String,
    pub description: String,
    pub available: bool,
    #[serde(skip)]
    pub effect: LaunchEffect,
}

#[derive(Debug, Clone, Serialize)]
pub struct LaunchControl {
    pub id: String,
    pub label: String,
    pub kind: ControlKind,
    pub choices: Vec<ControlChoice>,
    pub selected: Option<String>,
}

impl LaunchControl {
    fn new(id: String, label: String, kind: ControlKind, choices: Vec<ControlChoice>) -> Self {
        Self {
            id,
            label,
            kind,
            choices,
            selected: None,
        }
    }

    fn derive_selected(mut self, launch: &LaunchConfig) -> Self {
        self.selected = self
            .choices
            .iter()
            .find(|c| c.effect.is_applied(launch))
            .map(|c| c.id.clone());
        self
    }

    pub fn select(&self, launch: &mut LaunchConfig, choice: Option<&str>) -> Result<()> {
        let chosen = choice
            .map(|id| {
                self.choices
                    .iter()
                    .find(|c| c.id == id && c.available)
                    .ok_or_else(|| anyhow!("{id} is not an available choice of {}", self.label))
            })
            .transpose()?;
        for c in &self.choices {
            c.effect.strip(launch);
        }
        if let Some(c) = chosen {
            c.effect.apply(launch);
        }
        Ok(())
    }
}

pub fn launch_controls(game: &Game) -> Vec<LaunchControl> {
    let Some((manifest, edition_id)) = strategies::find_for_app_id(&game.source.app_id) else {
        return Vec::new();
    };
    pack_control(game, &manifest, &edition_id)
        .into_iter()
        .chain(
            manifest
                .options
                .iter()
                .filter(|o| o.alongside.is_none() && !o.effect.is_empty())
                .map(option_toggle),
        )
        .map(|c| c.derive_selected(&game.launch))
        .collect()
}

fn pack_control(game: &Game, manifest: &GachaManifest, edition_id: &str) -> Option<LaunchControl> {
    let edition = manifest.edition(edition_id)?;
    if manifest.strategy_for(edition) != InstallStrategy::KuroResourceIndex {
        return None;
    }
    let config = KuroConfig::load(manifest, edition_id).ok()?;
    if config.packs.is_empty() {
        return None;
    }
    let installed: Vec<String> = strategies::packs(game)
        .into_iter()
        .filter(|p| p.installed)
        .map(|p| p.id)
        .collect();
    let choices = config
        .packs
        .into_iter()
        .map(|(id, def)| ControlChoice {
            available: installed.contains(&id),
            id,
            label: def.label,
            description: String::new(),
            effect: def.effect,
        })
        .collect();
    Some(LaunchControl::new(
        PACKS_CONTROL.into(),
        "Active pack".into(),
        ControlKind::Choice,
        choices,
    ))
}

pub fn pack_fallback(game: &Game, removing: &str) -> Option<ControlChoice> {
    let (manifest, edition_id) = strategies::find_for_app_id(&game.source.app_id)?;
    let default = KuroConfig::load(&manifest, &edition_id).ok()?.default_pack;
    let (preferred, rest): (Vec<_>, Vec<_>) = pack_control(game, &manifest, &edition_id)?
        .choices
        .into_iter()
        .filter(|c| c.available && c.id != removing)
        .partition(|c| default.as_deref() == Some(c.id.as_str()));
    preferred.into_iter().chain(rest).next()
}

fn option_toggle(option: &ManifestOption) -> LaunchControl {
    let choice = ControlChoice {
        id: option.id.clone(),
        label: option.label.clone(),
        description: option.description.clone(),
        available: true,
        effect: option.effect.clone(),
    };
    LaunchControl::new(
        option.id.clone(),
        option.label.clone(),
        ControlKind::Toggle,
        vec![choice],
    )
}
