use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

mod exec;
mod remote;
pub use exec::{ExecOutcome, execute};
pub use remote::{RemoteScript, fetch_base, fetch_index, install_remote};

pub const BUILTIN_VARS: &[&str] = &["prefix", "cache", "home"];

#[derive(Debug, Clone, Deserialize)]
pub struct Script {
    pub script: ScriptMeta,
    #[serde(default, rename = "input")]
    pub inputs: Vec<Input>,
    #[serde(default, rename = "step")]
    pub steps: Vec<Step>,
    #[serde(default)]
    pub game: Option<GameSpec>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScriptMeta {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    Prefix,
    Runner,
    File,
    Directory,
    Text,
    Choice,
    Bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub id: String,
    pub kind: InputKind,
    pub label: String,
    #[serde(default)]
    pub picker: String,
    #[serde(default)]
    pub filter: String,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub default: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "task", rename_all = "snake_case")]
pub enum Step {
    InitPrefix,
    Winetricks {
        verbs: Vec<String>,
    },
    Download {
        url: String,
        dest: String,
        #[serde(default)]
        sha256: String,
    },
    Extract {
        archive: String,
        dest: String,
    },
    RunExe {
        exe: String,
        #[serde(default)]
        dll_overrides: HashMap<String, String>,
    },
    Shell {
        run: String,
    },
}

impl Step {
    pub fn describe(&self) -> String {
        match self {
            Step::InitPrefix => "initializing prefix".into(),
            Step::Winetricks { verbs } => format!("winetricks {}", verbs.join(" ")),
            Step::Download { url, .. } => format!("downloading {url}"),
            Step::Extract { archive, .. } => format!("extracting {archive}"),
            Step::RunExe { exe, .. } => format!("running {exe}"),
            Step::Shell { .. } => "running shell step".into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GameSpec {
    pub name: String,
    pub exe: String,
    #[serde(default)]
    pub runner: String,
    #[serde(default)]
    pub wine_version: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub dll_overrides: HashMap<String, String>,
}

impl Script {
    pub fn parse(text: &str) -> Result<Self> {
        let script: Script = toml::from_str(text).context("invalid script toml")?;
        script.validate()?;
        Ok(script)
    }

    pub fn has_shell(&self) -> bool {
        self.steps.iter().any(|s| matches!(s, Step::Shell { .. }))
    }

    pub fn prefix_input(&self) -> Option<&Input> {
        self.inputs.iter().find(|i| i.kind == InputKind::Prefix)
    }

    pub fn runner_input(&self) -> Option<&Input> {
        self.inputs.iter().find(|i| i.kind == InputKind::Runner)
    }

    fn validate(&self) -> Result<()> {
        if self.script.name.trim().is_empty() {
            bail!("script.name is empty");
        }
        if let Some(game) = &self.game {
            if game.name.trim().is_empty() {
                bail!("game.name is empty");
            }
            if game.exe.trim().is_empty() {
                bail!("game.exe is empty");
            }
            if !matches!(game.runner.as_str(), "" | "wine" | "native") {
                bail!(
                    "game.runner \"{}\" is not supported (wine, native)",
                    game.runner
                );
            }
        }

        let mut ids = HashSet::new();
        for input in &self.inputs {
            if input.id.trim().is_empty() {
                bail!("input with empty id");
            }
            if !ids.insert(input.id.as_str()) {
                bail!("duplicate input id \"{}\"", input.id);
            }
            let reserved = match input.id.as_str() {
                "cache" | "home" => true,
                "prefix" => input.kind != InputKind::Prefix,
                _ => false,
            };
            if reserved {
                bail!("input id \"{}\" is reserved", input.id);
            }
            if input.kind == InputKind::Choice && input.options.is_empty() {
                bail!("choice input \"{}\" has no options", input.id);
            }
            match input.kind {
                InputKind::Prefix if !matches!(input.picker.as_str(), "" | "list" | "path") => {
                    bail!(
                        "prefix input \"{}\" has unknown picker \"{}\"",
                        input.id,
                        input.picker
                    )
                }
                InputKind::Prefix => {}
                _ if !input.picker.is_empty() => {
                    bail!(
                        "picker is only valid on prefix inputs (input \"{}\")",
                        input.id
                    )
                }
                _ => {}
            }
        }
        if self
            .inputs
            .iter()
            .filter(|i| i.kind == InputKind::Prefix)
            .count()
            > 1
        {
            bail!("more than one prefix input");
        }
        if self
            .inputs
            .iter()
            .filter(|i| i.kind == InputKind::Runner)
            .count()
            > 1
        {
            bail!("more than one runner input");
        }
        if self.runner_input().is_some()
            && self
                .game
                .as_ref()
                .is_some_and(|g| !g.wine_version.is_empty())
        {
            bail!("declare a runner input or game.wine_version, not both");
        }

        let mut known: HashSet<&str> = BUILTIN_VARS.iter().copied().collect();
        known.extend(self.inputs.iter().map(|i| i.id.as_str()));
        for text in self.template_strings() {
            for var in placeholders(text)? {
                if !known.contains(var.as_str()) {
                    bail!("unknown variable ${{{var}}}");
                }
            }
        }
        Ok(())
    }

    fn template_strings(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for step in &self.steps {
            match step {
                Step::InitPrefix | Step::Winetricks { .. } => {}
                Step::Download { url, dest, .. } => {
                    out.push(url);
                    out.push(dest);
                }
                Step::Extract { archive, dest } => {
                    out.push(archive);
                    out.push(dest);
                }
                Step::RunExe { exe, dll_overrides } => {
                    out.push(exe);
                    out.extend(dll_overrides.values().map(String::as_str));
                }
                Step::Shell { run } => out.push(run),
            }
        }
        if let Some(game) = &self.game {
            out.push(&game.exe);
            out.extend(game.env.values().map(String::as_str));
            out.extend(game.dll_overrides.values().map(String::as_str));
        }
        out
    }
}

fn expand_with<F: FnMut(&str) -> Result<String>>(text: &str, mut resolve: F) -> Result<String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after
            .find('}')
            .ok_or_else(|| anyhow::anyhow!("unterminated ${{...}} in \"{text}\""))?;
        out.push_str(&resolve(&after[..end])?);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

pub fn interpolate(text: &str, vars: &HashMap<String, String>) -> Result<String> {
    expand_with(text, |key| {
        vars.get(key)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("unknown variable ${{{key}}}"))
    })
}

fn placeholders(text: &str) -> Result<Vec<String>> {
    let mut keys = Vec::new();
    expand_with(text, |key| {
        keys.push(key.to_string());
        Ok(String::new())
    })?;
    Ok(keys)
}

#[derive(Debug, Clone)]
pub struct ScriptEntry {
    pub toml_path: PathBuf,
    pub dir: PathBuf,
    pub author: String,
    pub script: Script,
    pub modified: String,
}

impl ScriptEntry {
    pub fn icon_path(&self) -> Option<PathBuf> {
        let icon = &self.script.script.icon;
        (!icon.is_empty()).then(|| self.dir.join(icon))
    }
}

pub fn list_installed() -> Vec<ScriptEntry> {
    let mut out = Vec::new();
    let root = crate::scripts_dir();
    let Ok(authors) = std::fs::read_dir(&root) else {
        return out;
    };
    for author in authors.flatten() {
        let author_dir = author.path();
        if !author_dir.is_dir() {
            continue;
        }
        let author_name = author.file_name().to_string_lossy().into_owned();
        let Ok(entries) = std::fs::read_dir(&author_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let Some(toml_path) = first_toml(&dir) else {
                continue;
            };
            let parsed = std::fs::read_to_string(&toml_path)
                .map_err(anyhow::Error::from)
                .and_then(|t| Script::parse(&t));
            match parsed {
                Ok(script) => {
                    let modified = std::fs::metadata(&toml_path)
                        .and_then(|m| m.modified())
                        .ok()
                        .map(|t| {
                            chrono::DateTime::<chrono::Local>::from(t)
                                .format("%Y-%m-%d %H:%M")
                                .to_string()
                        })
                        .unwrap_or_default();
                    out.push(ScriptEntry {
                        toml_path,
                        dir,
                        author: author_name.clone(),
                        script,
                        modified,
                    });
                }
                Err(e) => tracing::warn!("skipping script at {}: {e:#}", dir.display()),
            }
        }
    }
    out.sort_by(|a, b| {
        a.script
            .script
            .name
            .to_lowercase()
            .cmp(&b.script.script.name.to_lowercase())
    });
    out
}

fn first_toml(dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|e| e == "toml"))
}

pub fn remove_script(dir: &Path) -> Result<()> {
    let root = crate::scripts_dir().canonicalize()?;
    let target = dir.canonicalize()?;
    if target.parent().and_then(Path::parent) != Some(root.as_path()) {
        bail!(
            "refusing to remove {}: not an installed script dir",
            dir.display()
        );
    }
    std::fs::remove_dir_all(&target)?;
    if let Some(author_dir) = target.parent() {
        let _ = std::fs::remove_dir(author_dir);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[script]
name = "Are gachas anti-communism"
description = "one of everything"
author = "Karl Maxr"
icon = "./icon.png"

[[input]]
id = "prefix"
kind = "prefix"
label = "Wine prefix"

[[input]]
id = "installer"
kind = "file"
label = "Installer exe"
filter = "*.exe"

[[input]]
id = "quality"
kind = "choice"
label = "Texture quality"
options = ["low", "high"]
default = "high"

[[step]]
task = "init_prefix"

[[step]]
task = "winetricks"
verbs = ["corefonts", "vcrun2019"]

[[step]]
task = "download"
url = "https://example.com/patch.zip"
dest = "${cache}/patch.zip"
sha256 = "abc123"

[[step]]
task = "extract"
archive = "${cache}/patch.zip"
dest = "${prefix}/drive_c/Game"

[[step]]
task = "run_exe"
exe = "${installer}"
dll_overrides = { powershell = "d" }

[[step]]
task = "shell"
run = "echo ${quality}"

[game]
name = "some game"
exe = "${prefix}/drive_c/Game/game.exe"

[game.env]
QUALITY = "${quality}"

[game.dll_overrides]
d3d11 = "n,b"
"#;

    #[test]
    fn kitchen_sink() {
        let s = Script::parse(SAMPLE).unwrap();
        assert_eq!(s.inputs.len(), 3);
        assert_eq!(s.steps.len(), 6);
        assert!(s.has_shell());
        assert_eq!(s.prefix_input().unwrap().id, "prefix");
        assert_eq!(s.game.as_ref().unwrap().dll_overrides["d3d11"], "n,b");
        assert!(
            matches!(&s.steps[4], Step::RunExe { dll_overrides, .. } if dll_overrides["powershell"] == "d")
        );
        assert!(
            Script::parse(SAMPLE.split("[game]").next().unwrap())
                .unwrap()
                .game
                .is_none()
        );
    }

    #[test]
    fn garbage_in() {
        let breakages = [
            ("${installer}", "${nope}", "nope"),
            ("id = \"installer\"", "id = \"quality\"", "duplicate"),
            ("id = \"installer\"", "id = \"cache\"", "reserved"),
            (
                "id = \"prefix\"\nkind = \"prefix\"",
                "id = \"prefix\"\nkind = \"text\"",
                "reserved",
            ),
            (
                "[game]\nname",
                "[game]\nrunner = \"gamecube\"\nname",
                "runner",
            ),
            (
                "kind = \"file\"",
                "kind = \"file\"\npicker = \"path\"",
                "picker",
            ),
            ("options = [\"low\", \"high\"]\n", "", "options"),
            (
                "id = \"installer\"\nkind = \"file\"",
                "id = \"installer\"\nkind = \"prefix\"",
                "prefix input",
            ),
        ];
        for (from, to, expect) in breakages {
            let err = Script::parse(&SAMPLE.replace(from, to))
                .unwrap_err()
                .to_string();
            assert!(err.contains(expect), "{err}");
        }
        let both = SAMPLE
            .replace("kind = \"file\"", "kind = \"runner\"")
            .replace("[game]\nname", "[game]\nwine_version = \"system\"\nname");
        assert!(
            Script::parse(&both)
                .unwrap_err()
                .to_string()
                .contains("not both")
        );
        assert!(interpolate("${broken", &HashMap::new()).is_err());
    }
}
