use std::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use omikuji_core::fs_watcher::DirWatcher;
use omikuji_core::prefixes as core_prefixes;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, creating)]
        #[qproperty(bool, command_running, cxx_name = "commandRunning")]
        type OfudaBridge = super::OfudaRust;
    }

    unsafe extern "RustQt" {
        #[qsignal]
        fn changed(self: Pin<&mut OfudaBridge>);

        #[qsignal]
        #[cxx_name = "createFinished"]
        fn create_finished(self: Pin<&mut OfudaBridge>, ok: bool, error: QString);

        #[qsignal]
        #[cxx_name = "createOutput"]
        fn create_output(self: Pin<&mut OfudaBridge>, line: QString);

        #[qsignal]
        #[cxx_name = "commandOutput"]
        fn command_output(self: Pin<&mut OfudaBridge>, line: QString);

        #[qsignal]
        #[cxx_name = "commandFinished"]
        fn command_finished(self: Pin<&mut OfudaBridge>, ok: bool, error: QString);

        #[qinvokable]
        #[cxx_name = "listJson"]
        fn list_json(self: &OfudaBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "listSteamJson"]
        fn list_steam_json(self: &OfudaBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "runTool"]
        fn run_tool(self: &OfudaBridge, path: &QString, tool: &QString, runner: &QString) -> bool;

        #[qinvokable]
        #[cxx_name = "runCommand"]
        fn run_command(
            self: Pin<&mut OfudaBridge>,
            path: &QString,
            runner: &QString,
            command: &QString,
        );

        #[qinvokable]
        #[cxx_name = "openFolder"]
        fn open_folder(self: &OfudaBridge, path: &QString) -> bool;

        #[qinvokable]
        #[cxx_name = "deletePrefix"]
        fn delete_prefix(self: Pin<&mut OfudaBridge>, path: &QString) -> bool;

        #[qinvokable]
        #[cxx_name = "createPrefix"]
        fn create_prefix(
            self: Pin<&mut OfudaBridge>,
            name: &QString,
            runner: &QString,
            preset: &QString,
        );

        #[qinvokable]
        fn watch(self: Pin<&mut OfudaBridge>);
    }

    impl cxx_qt::Threading for OfudaBridge {}
}

#[derive(Default)]
pub struct OfudaRust {
    watcher: Option<DirWatcher>,
    creating: bool,
    command_running: bool,
}

fn prefix_game(path: &QString, runner: &QString) -> omikuji_core::library::Game {
    let prefix = path.to_string();
    let runner = runner.to_string();
    omikuji_core::library::Game::with_options(
        "Ofuda".to_string(),
        std::path::PathBuf::new(),
        (!prefix.is_empty()).then_some(prefix),
        Some("wine".to_string()),
        (!runner.is_empty()).then_some(runner),
    )
}

fn prefixes_json(list: Vec<core_prefixes::PrefixInfo>, kind: &str) -> QString {
    let list: Vec<serde_json::Value> = list
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "path": p.path.to_string_lossy(),
                "name": p.name,
                "gameCount": p.games.len(),
                "games": p.games,
                "runner": p.runner,
                "kind": kind,
            })
        })
        .collect();
    QString::from(&serde_json::Value::Array(list).to_string())
}

impl qobject::OfudaBridge {
    fn list_json(&self) -> QString {
        prefixes_json(core_prefixes::list_prefixes(), "omikuji")
    }

    fn list_steam_json(&self) -> QString {
        prefixes_json(core_prefixes::list_steam_prefixes(), "steam")
    }

    fn run_tool(&self, path: &QString, tool: &QString, runner: &QString) -> bool {
        use omikuji_core::wine_tools::WineTool;
        let name = tool.to_string();
        let Some(tool) = WineTool::from_name(&name) else {
            tracing::warn!("unknown ofuda tool: {name}");
            return false;
        };
        match omikuji_core::wine_tools::run(&prefix_game(path, runner), tool) {
            Ok(_) => true,
            Err(e) => {
                tracing::error!("ofuda run_tool failed: {e}");
                false
            }
        }
    }

    fn run_command(mut self: Pin<&mut Self>, path: &QString, runner: &QString, command: &QString) {
        use omikuji_core::wine_tools::{self, WineTool};
        if self.command_running {
            return;
        }
        let Some(tool) = WineTool::from_command_line(&command.to_string()) else {
            return;
        };
        self.as_mut().set_command_running(true);
        let qt = self.as_mut().qt_thread();
        let line_qt = qt.clone();
        wine_tools::run_detached(
            prefix_game(path, runner),
            tool,
            move |line| {
                let l = line.to_string();
                let _ = line_qt.queue(move |mut obj: Pin<&mut qobject::OfudaBridge>| {
                    obj.as_mut().command_output(QString::from(&l));
                });
            },
            move |ok, err| {
                let _ = qt.queue(move |mut obj: Pin<&mut qobject::OfudaBridge>| {
                    obj.as_mut().set_command_running(false);
                    obj.as_mut().command_finished(ok, QString::from(&err));
                });
            },
        );
    }

    fn open_folder(&self, path: &QString) -> bool {
        match omikuji_core::desktop::browse_files(std::path::Path::new(&path.to_string())) {
            Ok(_) => true,
            Err(e) => {
                tracing::error!("ofuda open_folder failed: {e}");
                false
            }
        }
    }

    fn delete_prefix(mut self: Pin<&mut Self>, path: &QString) -> bool {
        let ok = core_prefixes::delete_prefix(std::path::Path::new(&path.to_string()));
        if ok {
            self.as_mut().changed();
        }
        ok
    }

    fn create_prefix(mut self: Pin<&mut Self>, name: &QString, runner: &QString, preset: &QString) {
        if self.creating {
            return;
        }
        self.as_mut().set_creating(true);
        let qt = self.as_mut().qt_thread();
        let name = name.to_string();
        let runner = runner.to_string();
        let preset = preset.to_string();
        std::thread::spawn(move || {
            let line_qt = qt.clone();
            let res = core_prefixes::create_prefix(&name, &runner, &preset, |line| {
                let l = line.to_string();
                let _ = line_qt.queue(move |mut obj: Pin<&mut qobject::OfudaBridge>| {
                    obj.as_mut().create_output(QString::from(&l));
                });
            });
            let (ok, err) = match res {
                Ok(_) => (true, String::new()),
                Err(e) => (false, e.to_string()),
            };
            let _ = qt.queue(move |mut obj: Pin<&mut qobject::OfudaBridge>| {
                obj.as_mut().set_creating(false);
                obj.as_mut().create_finished(ok, QString::from(&err));
                obj.as_mut().changed();
            });
        });
    }

    fn watch(mut self: Pin<&mut Self>) {
        if self.watcher.is_some() {
            return;
        }
        let qt_thread = self.as_mut().qt_thread();
        let watcher = DirWatcher::watch(
            omikuji_core::prefixes_dir(),
            |_| true,
            move || {
                let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::OfudaBridge>| {
                    obj.as_mut().changed();
                });
            },
        );
        match watcher {
            Ok(w) => self.as_mut().rust_mut().get_mut().watcher = Some(w),
            Err(e) => tracing::error!("failed to watch prefixes dir: {e}"),
        }
    }
}
