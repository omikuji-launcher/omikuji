use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

use crate::library::Game;

const RESERVED: &[&str] = &[
    "exe",
    "game_dir",
    "game_prefix",
    "game_id",
    "game_name",
    "home",
    "data_path",
    "gachas_path",
    "components_path",
    "runners_path",
    "layers_path",
    "prefixes_path",
    "cache_path",
    "logs_path",
    "runtime_path",
    "scripts_path",
];

fn root_paths() -> Vec<(String, String)> {
    let p = |b: PathBuf| b.to_string_lossy().into_owned();
    vec![
        ("home".to_string(), p(dirs::home_dir().unwrap_or_default())),
        ("data_path".to_string(), p(crate::data_dir())),
        ("gachas_path".to_string(), p(crate::gachas_dir())),
        ("components_path".to_string(), p(crate::components_dir())),
        ("runners_path".to_string(), p(crate::runners_dir())),
        ("layers_path".to_string(), p(crate::layers_dir())),
        ("prefixes_path".to_string(), p(crate::prefixes_dir())),
        ("cache_path".to_string(), p(crate::cache_dir())),
        ("logs_path".to_string(), p(crate::logs_dir())),
        ("runtime_path".to_string(), p(crate::runtime_dir())),
        ("scripts_path".to_string(), p(crate::scripts_dir())),
    ]
}

type UserVarsCache = Option<(SystemTime, Vec<(String, String)>)>;

static USER_VARS: Mutex<UserVarsCache> = Mutex::new(None);

fn user_vars() -> Vec<(String, String)> {
    let mtime = std::fs::metadata(crate::ui_settings::ui_settings_path())
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);
    let mut guard = USER_VARS.lock().unwrap();
    if let Some((cached, vars)) = guard.as_ref()
        && *cached == mtime
    {
        return vars.clone();
    }
    let vars: Vec<(String, String)> = crate::ui_settings::UiSettings::load()
        .template_vars
        .into_iter()
        .filter(|(k, _)| !k.is_empty() && !RESERVED.contains(&k.as_str()))
        .collect();
    *guard = Some((mtime, vars.clone()));
    vars
}

pub struct TemplateVars(Vec<(String, String)>);

impl TemplateVars {
    fn finish(vars: Vec<(String, String)>) -> Self {
        let mut this = Self(vars);
        for (key, value) in user_vars() {
            let expanded = this.expand(&value);
            this.0.push((key, expanded));
        }
        this
    }

    pub fn global() -> Self {
        Self::finish(root_paths())
    }

    pub fn base(game: &Game) -> Self {
        let mut vars = Vec::new();
        let exe = &game.metadata.exe;
        if !exe.as_os_str().is_empty() {
            vars.push(("exe".to_string(), exe.to_string_lossy().into_owned()));
            if let Some(dir) = exe.parent() {
                vars.push(("game_dir".to_string(), dir.to_string_lossy().into_owned()));
            }
        }
        vars.push(("game_id".to_string(), game.metadata.id.clone()));
        vars.push(("game_name".to_string(), game.metadata.name.clone()));
        vars.extend(root_paths());
        Self::finish(vars)
    }

    pub fn for_game(game: &Game) -> Self {
        let mut this = Self::base(game);
        if let Some(prefix) = crate::launch::effective_prefix(game) {
            this.0.push((
                "game_prefix".to_string(),
                prefix.to_string_lossy().into_owned(),
            ));
        }
        this
    }

    pub fn expand(&self, input: &str) -> String {
        if !input.contains("${") {
            return input.to_string();
        }
        let mut out = String::with_capacity(input.len());
        let mut rest = input;
        while let Some(start) = rest.find("${") {
            out.push_str(&rest[..start]);
            let after = &rest[start + 2..];
            match after.find('}') {
                Some(end) => {
                    let key = &after[..end];
                    match self.0.iter().find(|(k, _)| k == key) {
                        Some((_, value)) => out.push_str(value),
                        None => out.push_str(&rest[start..start + end + 3]),
                    }
                    rest = &after[end + 1..];
                }
                None => {
                    out.push_str(&rest[start..]);
                    rest = "";
                }
            }
        }
        out.push_str(rest);
        out
    }

    pub fn expand_env(
        &self,
        env: std::collections::HashMap<String, String>,
    ) -> std::collections::HashMap<String, String> {
        env.into_iter().map(|(k, v)| (k, self.expand(&v))).collect()
    }

    pub fn into_map(self) -> std::collections::HashMap<String, String> {
        self.0.into_iter().collect()
    }
}
