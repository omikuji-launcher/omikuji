// gacha manifest; declarative per-game config. the ui renders from editions/voice_locales and the bridge routes against it.
//
// storage: {gachas_dir}/{publisher_slug}/{game_slug}/manifest.json
// schema_version gates forward-compat; unknown versions are logged and skipped rather than parsed as best-effort.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GachaManifest {
    pub schema_version: u32,

    pub id: String,

    pub publisher_slug: String,
    pub game_slug: String,

    pub display_name: String,
    pub publisher: String,

    pub install_strategy: String,

    pub app_id_prefix: String,

    pub editions: Vec<ManifestEdition>,

    #[serde(default)]
    pub voice_locales: Vec<ManifestVoice>,

    pub default_library_template: String,
    pub install_folder_name: String,

    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub launch_patch: String,

    #[serde(default)]
    pub anti_cheat: String,
    #[serde(default)]
    pub runner_preference: Vec<String>,
    #[serde(default)]
    pub telemetry_block: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,

    #[serde(default)]
    pub letter_fallback: String,

    // true: install writes arcihves to scratch dir first, then extracts (hoyo_sophon, gryphline_resource_patch).
    // false: writes directly to install_path (kuro_resource_index). drives the dialog's temp-path field visibility and free-space math
    #[serde(default = "default_true")]
    pub uses_temp_dir: bool,

    // per-strategy data, opaque to the manifest layer. parsed by the strategy code.
    #[serde(default)]
    pub strategy_config: serde_json::Value,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEdition {
    pub id: String,
    pub label: String,
    pub exe_name: String,
    // unity _Data folder name, per-edition for genshin, shared otherwise
    pub data_folder: String,

    // per-strategy data (biz_id, api_base, index_url, etc), opaque to manifest layer
    #[serde(default)]
    pub strategy_config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestVoice {
    pub id: String,
    pub label: String,
    pub folder_name: String,
}

pub fn load_all() -> Vec<GachaManifest> {
    use std::collections::HashMap;
    let mut by_id: HashMap<String, GachaManifest> = HashMap::new();
    for m in walk_manifests(&crate::gachas_dir()) {
        by_id.insert(m.id.clone(), m);
    }
    let mut out: Vec<GachaManifest> = by_id.into_values().collect();
    out.sort_by(|a, b| {
        a.publisher
            .cmp(&b.publisher)
            .then(a.display_name.cmp(&b.display_name))
    });
    out
}

pub fn find(id: &str) -> Option<GachaManifest> {
    load_all().into_iter().find(|m| m.id == id)
}

fn walk_manifests(root: &std::path::Path) -> Vec<GachaManifest> {
    let Ok(publishers) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for pub_entry in publishers.flatten() {
        let pub_path = pub_entry.path();
        if !pub_path.is_dir() {
            continue;
        }
        let Ok(games) = std::fs::read_dir(&pub_path) else {
            continue;
        };
        for game_entry in games.flatten() {
            let game_path = game_entry.path();
            if !game_path.is_dir() {
                continue;
            }
            let manifest_path = game_path.join("manifest.json");
            let Ok(data) = std::fs::read_to_string(&manifest_path) else {
                continue;
            };
            match serde_json::from_str::<GachaManifest>(&data) {
                Ok(m) if m.schema_version == SCHEMA_VERSION => out.push(m),
                Ok(m) => tracing::warn!(
                    "skipping {}: unsupported schema_version {}",
                    manifest_path.display(),
                    m.schema_version
                ),
                Err(e) => tracing::warn!("skipping {}: {}", manifest_path.display(), e),
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn example_manifest() -> GachaManifest {
        GachaManifest {
            schema_version: SCHEMA_VERSION,
            id: "test.game".into(),
            publisher_slug: "test".into(),
            game_slug: "game".into(),
            display_name: "Test Game".into(),
            publisher: "Test Publisher".into(),
            install_strategy: "hoyo_sophon".into(),
            app_id_prefix: "game".into(),
            editions: vec![ManifestEdition {
                id: "global".into(),
                label: "Global".into(),
                exe_name: "Game.exe".into(),
                data_folder: "Game_Data".into(),
                strategy_config: serde_json::Value::Null,
            }],
            voice_locales: vec![],
            default_library_template: "{home}/Games".into(),
            install_folder_name: "Test Game".into(),
            category: "Test".into(),
            launch_patch: String::new(),
            anti_cheat: String::new(),
            runner_preference: vec![],
            telemetry_block: vec![],
            env: HashMap::new(),
            letter_fallback: "T".into(),
            uses_temp_dir: true,
            strategy_config: serde_json::Value::Null,
        }
    }

    #[test]
    fn walks_nested_layout() {
        let tmp = tempdir().unwrap();
        let m = example_manifest();
        let dir = tmp.path().join(&m.publisher_slug).join(&m.game_slug);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("manifest.json"),
            serde_json::to_string_pretty(&m).unwrap(),
        )
        .unwrap();

        let found = walk_manifests(tmp.path());
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, "test.game");
    }

    #[test]
    fn skips_wrong_schema_version() {
        let tmp = tempdir().unwrap();
        let mut m = example_manifest();
        m.schema_version = 99;
        let dir = tmp.path().join(&m.publisher_slug).join(&m.game_slug);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("manifest.json"),
            serde_json::to_string_pretty(&m).unwrap(),
        )
        .unwrap();

        assert!(walk_manifests(tmp.path()).is_empty());
    }

    #[test]
    fn missing_dir_returns_empty() {
        let tmp = tempdir().unwrap();
        let missing = tmp.path().join("does_not_exist");
        assert!(walk_manifests(&missing).is_empty());
    }
}
