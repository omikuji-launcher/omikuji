use std::pin::Pin;

use cxx_qt::Threading;
use cxx_qt_lib::QString;
use omikuji_core::scripts as core_scripts;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, running)]
        type ScriptsBridge = super::ScriptsRust;
    }

    unsafe extern "RustQt" {
        #[qsignal]
        #[cxx_name = "runOutput"]
        fn run_output(self: Pin<&mut ScriptsBridge>, line: QString);

        #[qsignal]
        #[cxx_name = "runFinished"]
        fn run_finished(
            self: Pin<&mut ScriptsBridge>,
            ok: bool,
            error: QString,
            game_json: QString,
            exe_missing: bool,
        );

        #[qinvokable]
        #[cxx_name = "listJson"]
        fn list_json(self: &ScriptsBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "loadJson"]
        fn load_json(self: &ScriptsBridge, toml_path: &QString) -> QString;

        #[qinvokable]
        fn run(self: Pin<&mut ScriptsBridge>, toml_path: &QString, values_json: &QString);

        #[qinvokable]
        #[cxx_name = "removeScript"]
        fn remove_script(self: &ScriptsBridge, dir: &QString) -> bool;

        #[qsignal]
        #[cxx_name = "remoteListed"]
        fn remote_listed(self: Pin<&mut ScriptsBridge>, ok: bool, json: QString, error: QString);

        #[qsignal]
        #[cxx_name = "remoteInstalled"]
        fn remote_installed(
            self: Pin<&mut ScriptsBridge>,
            ok: bool,
            toml_path: QString,
            error: QString,
        );

        #[qinvokable]
        #[cxx_name = "refreshRemote"]
        fn refresh_remote(self: Pin<&mut ScriptsBridge>);

        #[qinvokable]
        #[cxx_name = "installRemote"]
        fn install_remote(self: Pin<&mut ScriptsBridge>, entry_json: &QString);
    }

    impl cxx_qt::Threading for ScriptsBridge {}
}

#[derive(Default)]
pub struct ScriptsRust {
    running: bool,
}

fn kind_str(kind: core_scripts::InputKind) -> &'static str {
    use core_scripts::InputKind::*;
    match kind {
        Prefix => "prefix",
        Runner => "runner",
        File => "file",
        Directory => "directory",
        Text => "text",
        Choice => "choice",
        Bool => "bool",
    }
}

fn detail_json(script: &core_scripts::Script, toml_text: &str) -> serde_json::Value {
    let inputs: Vec<serde_json::Value> = script
        .inputs
        .iter()
        .map(|i| {
            serde_json::json!({
                "id": i.id,
                "kind": kind_str(i.kind),
                "label": i.label,
                "picker": i.picker,
                "filter": i.filter,
                "options": i.options,
                "default": i.default,
            })
        })
        .collect();
    serde_json::json!({
        "name": script.script.name,
        "description": script.script.description,
        "author": script.script.author,
        "note": script.script.note,
        "gameName": script.games.first().map(|g| g.name.as_str()).unwrap_or_default(),
        "isUtility": script.games.is_empty(),
        "hasShell": script.has_shell(),
        "inputs": inputs,
        "steps": script.steps.iter().map(|s| s.describe()).collect::<Vec<_>>(),
        "toml": toml_text,
    })
}

impl qobject::ScriptsBridge {
    fn list_json(&self) -> QString {
        let list: Vec<serde_json::Value> = core_scripts::list_installed()
            .into_iter()
            .map(|e| {
                let icon = e
                    .icon_path()
                    .filter(|p| p.is_file())
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default();
                serde_json::json!({
                    "toml": e.toml_path.to_string_lossy(),
                    "dir": e.dir.to_string_lossy(),
                    "author": e.author,
                    "name": e.script.script.name,
                    "description": e.script.script.description,
                    "gameName": e.script.games.first().map(|g| g.name.as_str()).unwrap_or_default(),
                    "hasShell": e.script.has_shell(),
                    "modified": e.modified,
                    "icon": icon,
                })
            })
            .collect();
        QString::from(&serde_json::Value::Array(list).to_string())
    }

    fn load_json(&self, toml_path: &QString) -> QString {
        let path = std::path::PathBuf::from(toml_path.to_string());
        let loaded = std::fs::read_to_string(&path)
            .map_err(omikuji_core::anyhow::Error::from)
            .and_then(|text| core_scripts::Script::parse(&text).map(|s| (s, text)));
        let json = match loaded {
            Ok((script, text)) => detail_json(&script, &text),
            Err(e) => serde_json::json!({ "error": format!("{e:#}") }),
        };
        QString::from(&json.to_string())
    }

    fn run(mut self: Pin<&mut Self>, toml_path: &QString, values_json: &QString) {
        if self.running {
            return;
        }
        let path = toml_path.to_string();
        let values: std::collections::HashMap<String, String> =
            serde_json::from_str(&values_json.to_string()).unwrap_or_default();

        self.as_mut().set_running(true);
        let qt = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            let line_qt = qt.clone();
            let res = std::fs::read_to_string(&path)
                .map_err(omikuji_core::anyhow::Error::from)
                .and_then(|text| core_scripts::Script::parse(&text))
                .and_then(|script| {
                    core_scripts::execute(&script, &values, |line| {
                        let l = line.to_string();
                        let _ = line_qt.queue(move |mut obj: Pin<&mut qobject::ScriptsBridge>| {
                            obj.as_mut().run_output(QString::from(&l));
                        });
                    })
                });
            let (ok, err, game_json, exe_missing) = match res {
                Ok(outcome) => match &outcome.game {
                    None => (true, String::new(), String::new(), false),
                    Some(game) => match serde_json::to_string(game) {
                        Ok(json) if outcome.exe_found => (true, String::new(), json, false),
                        Ok(json) => (
                            false,
                            format!(
                                "The game exe wasn't found where the script expected it ({}).",
                                game.metadata.exe.display()
                            ),
                            json,
                            true,
                        ),
                        Err(e) => (false, e.to_string(), String::new(), false),
                    },
                },
                Err(e) => (false, format!("{e:#}"), String::new(), false),
            };
            let _ = qt.queue(move |mut obj: Pin<&mut qobject::ScriptsBridge>| {
                obj.as_mut().set_running(false);
                obj.as_mut().run_finished(
                    ok,
                    QString::from(&err),
                    QString::from(&game_json),
                    exe_missing,
                );
            });
        });
    }

    fn refresh_remote(mut self: Pin<&mut Self>) {
        let qt = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            let res = core_scripts::fetch_index();
            let (ok, json, err) = match res {
                Ok(list) => {
                    let base = core_scripts::fetch_base();
                    let arr: Vec<serde_json::Value> = list
                        .iter()
                        .map(|e| {
                            serde_json::json!({
                                "author": e.author,
                                "slug": e.slug,
                                "name": e.name,
                                "description": e.description,
                                "has_shell": e.has_shell,
                                "modified": e.modified,
                                "toml": e.toml,
                                "icon": e.icon,
                                "iconUrl": if e.icon.is_empty() { String::new() } else { format!("{base}/{}", e.icon) },
                            })
                        })
                        .collect();
                    (
                        true,
                        serde_json::Value::Array(arr).to_string(),
                        String::new(),
                    )
                }
                Err(e) => (false, "[]".to_string(), format!("{e:#}")),
            };
            let _ = qt.queue(move |mut obj: Pin<&mut qobject::ScriptsBridge>| {
                obj.as_mut()
                    .remote_listed(ok, QString::from(&json), QString::from(&err));
            });
        });
    }

    fn install_remote(mut self: Pin<&mut Self>, entry_json: &QString) {
        let entry: core_scripts::RemoteScript = match serde_json::from_str(&entry_json.to_string())
        {
            Ok(e) => e,
            Err(e) => {
                tracing::error!("install_remote: invalid entry json: {e}");
                return;
            }
        };
        let qt = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            let (ok, path, err) = match core_scripts::install_remote(&entry) {
                Ok(p) => (true, p.to_string_lossy().into_owned(), String::new()),
                Err(e) => (false, String::new(), format!("{e:#}")),
            };
            let _ = qt.queue(move |mut obj: Pin<&mut qobject::ScriptsBridge>| {
                obj.as_mut()
                    .remote_installed(ok, QString::from(&path), QString::from(&err));
            });
        });
    }

    fn remove_script(&self, dir: &QString) -> bool {
        match core_scripts::remove_script(std::path::Path::new(&dir.to_string())) {
            Ok(_) => true,
            Err(e) => {
                tracing::error!("failed to remove script: {e:#}");
                false
            }
        }
    }
}
