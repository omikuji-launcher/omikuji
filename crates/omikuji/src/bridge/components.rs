use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use omikuji_core::components as core_components;
use std::collections::HashMap;
use std::pin::Pin;
use std::thread;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, in_progress, cxx_name = "inProgress")]
        #[qproperty(bool, checking)]
        type ComponentsBridge = super::ComponentsRust;
    }

    unsafe extern "RustQt" {
        #[qsignal]
        #[cxx_name = "componentStarted"]
        fn component_started(self: Pin<&mut ComponentsBridge>, name: QString);

        #[qsignal]
        #[cxx_name = "componentProgress"]
        fn component_progress(
            self: Pin<&mut ComponentsBridge>,
            name: QString,
            phase: QString,
            percent: f64,
        );

        #[qsignal]
        #[cxx_name = "componentCompleted"]
        fn component_completed(self: Pin<&mut ComponentsBridge>, name: QString, version: QString);

        #[qsignal]
        #[cxx_name = "componentFailed"]
        fn component_failed(self: Pin<&mut ComponentsBridge>, name: QString, error: QString);

        #[qinvokable]
        #[cxx_name = "statusJson"]
        fn status_json(self: &ComponentsBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "installAll"]
        fn install_all(self: Pin<&mut ComponentsBridge>);

        #[qinvokable]
        #[cxx_name = "installComponent"]
        fn install_component(self: Pin<&mut ComponentsBridge>, name: QString);

        #[qinvokable]
        #[cxx_name = "removeComponent"]
        fn remove_component(self: Pin<&mut ComponentsBridge>, name: QString);

        #[qinvokable]
        #[cxx_name = "componentReady"]
        fn component_ready(self: &ComponentsBridge, name: QString) -> bool;

        #[qinvokable]
        #[cxx_name = "checkUpdates"]
        fn check_updates(self: Pin<&mut ComponentsBridge>);

        #[qinvokable]
        fn refresh(self: Pin<&mut ComponentsBridge>);

        #[qinvokable]
        #[cxx_name = "drainEvents"]
        fn drain_events(self: Pin<&mut ComponentsBridge>);
    }
}

pub struct ComponentsRust {
    pub in_progress: bool,
    pub checking: bool,

    statuses: HashMap<String, ComponentStatusEntry>,
    checks_pending: i32,
}

#[derive(Clone, Default)]
struct ComponentStatusEntry {
    status: String,
    percent: f64,
    version: String,
    path: String,
    latest: String,
    error: String,
}

fn spec_by_name(name: &str) -> Option<&'static core_components::ComponentSpec> {
    core_components::specs::all()
        .iter()
        .find(|s| s.name == name)
}

fn is_busy(status: &str) -> bool {
    matches!(
        status,
        "resolving" | "installing" | "downloading" | "extracting"
    )
}

fn fresh_entry(spec: &core_components::ComponentSpec, latest: String) -> ComponentStatusEntry {
    match core_components::status_for(spec) {
        core_components::ComponentStatus::Installed { version, path } => ComponentStatusEntry {
            status: "completed".into(),
            percent: 100.0,
            version,
            path: path.display().to_string(),
            latest,
            ..Default::default()
        },
        core_components::ComponentStatus::System { path } => ComponentStatusEntry {
            status: "system".into(),
            percent: 100.0,
            path: path.display().to_string(),
            latest,
            ..Default::default()
        },
        core_components::ComponentStatus::Missing => ComponentStatusEntry {
            status: "missing".into(),
            latest,
            ..Default::default()
        },
    }
}

impl Default for ComponentsRust {
    fn default() -> Self {
        let statuses = core_components::specs::all()
            .iter()
            .map(|spec| (spec.name.to_string(), fresh_entry(spec, String::new())))
            .collect();

        Self {
            in_progress: false,
            checking: false,
            statuses,
            checks_pending: 0,
        }
    }
}

impl qobject::ComponentsBridge {
    fn status_json(&self) -> QString {
        let map: serde_json::Map<String, serde_json::Value> = self
            .statuses
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    serde_json::json!({
                        "status": v.status,
                        "percent": v.percent,
                        "version": v.version,
                        "path": v.path,
                        "latest": v.latest,
                        "error": v.error,
                    }),
                )
            })
            .collect();
        QString::from(&serde_json::Value::Object(map).to_string())
    }

    fn refresh(mut self: Pin<&mut Self>) {
        for spec in core_components::specs::all() {
            let current = self.statuses.get(spec.name).cloned();
            let latest = current
                .as_ref()
                .map(|c| c.latest.clone())
                .unwrap_or_default();
            let entry = match current {
                Some(c) if is_busy(&c.status) => c,
                _ => fresh_entry(spec, latest),
            };
            self.as_mut()
                .rust_mut()
                .get_mut()
                .statuses
                .insert(spec.name.into(), entry);
        }
    }

    // spawn an OS thread then block_on; we're inside #[tokio::main], so building a runtime directly would panic.
    fn spawn_install(
        mut self: Pin<&mut Self>,
        specs: Vec<&'static core_components::ComponentSpec>,
    ) {
        if self.in_progress || specs.is_empty() {
            return;
        }
        self.as_mut().set_in_progress(true);

        for spec in &specs {
            if let Some(entry) = self
                .as_mut()
                .rust_mut()
                .get_mut()
                .statuses
                .get_mut(spec.name)
            {
                entry.status = "installing".into();
                entry.percent = 0.0;
                entry.error.clear();
            }
        }

        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let Ok(rt) = rt else {
                omikuji_core::components::push_fail_event("setup", "couldn't build tokio runtime");
                return;
            };
            rt.block_on(async {
                for spec in specs {
                    let _ = core_components::install_one(spec).await;
                }
            });
        });
    }

    fn install_all(self: Pin<&mut Self>) {
        self.spawn_install(core_components::check_all());
    }

    fn component_ready(&self, name: QString) -> bool {
        spec_by_name(&name.to_string()).is_some_and(|s| core_components::ready(&[s]))
    }

    fn remove_component(mut self: Pin<&mut Self>, name: QString) {
        let target = name.to_string();
        let Some(spec) = spec_by_name(&target) else {
            return;
        };
        if let Err(e) = core_components::remove(spec) {
            omikuji_core::notifications::error(
                format!("Couldn't remove {}", target),
                e.to_string(),
            );
            return;
        }
        self.as_mut().refresh();
    }

    fn check_updates(mut self: Pin<&mut Self>) {
        let specs = core_components::updatable();
        if self.checking || specs.is_empty() {
            return;
        }
        self.as_mut().set_checking(true);
        self.as_mut().rust_mut().get_mut().checks_pending = specs.len() as i32;

        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let Ok(rt) = rt else {
                omikuji_core::components::push_fail_event("setup", "couldn't build tokio runtime");
                return;
            };
            rt.block_on(async {
                for spec in specs {
                    core_components::check_update(spec).await;
                }
            });
        });
    }

    fn install_component(mut self: Pin<&mut Self>, name: QString) {
        let target = name.to_string();
        let Some(spec) = spec_by_name(&target) else {
            omikuji_core::components::push_fail_event(
                &target,
                "unknown component (not in specs::all())",
            );
            return;
        };
        self.as_mut().spawn_install(vec![spec]);
    }

    fn drain_events(mut self: Pin<&mut Self>) {
        let events = core_components::drain_events();
        if events.is_empty() {
            let has_active = self.statuses.values().any(|s| is_busy(&s.status));
            if !has_active && self.in_progress {
                self.as_mut().set_in_progress(false);
                self.as_mut().refresh();
            }
            return;
        }

        for ev in events {
            match ev {
                core_components::ComponentEvent::Started { name } => {
                    self.as_mut().rust_mut().get_mut().statuses.insert(
                        name.clone(),
                        ComponentStatusEntry {
                            status: "installing".into(),
                            ..Default::default()
                        },
                    );
                    self.as_mut().component_started(QString::from(&name));
                }
                core_components::ComponentEvent::Progress {
                    name,
                    phase,
                    percent,
                } => {
                    if let Some(entry) = self.as_mut().rust_mut().get_mut().statuses.get_mut(&name)
                    {
                        entry.status = phase.clone();
                        entry.percent = percent;
                    }
                    self.as_mut().component_progress(
                        QString::from(&name),
                        QString::from(&phase),
                        percent,
                    );
                }
                core_components::ComponentEvent::Completed { name, version } => {
                    if let Some(entry) = self.as_mut().rust_mut().get_mut().statuses.get_mut(&name)
                    {
                        entry.status = "completed".into();
                        entry.percent = 100.0;
                        entry.version = version.clone();
                        entry.error.clear();
                    }
                    self.as_mut()
                        .component_completed(QString::from(&name), QString::from(&version));
                    self.as_mut().refresh();
                }
                core_components::ComponentEvent::Failed { name, error } => {
                    if let Some(entry) = self.as_mut().rust_mut().get_mut().statuses.get_mut(&name)
                    {
                        entry.status = "failed".into();
                        entry.error = error.clone();
                    }
                    self.as_mut()
                        .component_failed(QString::from(&name), QString::from(&error));
                }
                core_components::ComponentEvent::UpdateChecked { name, latest } => {
                    if let Some(entry) = self.as_mut().rust_mut().get_mut().statuses.get_mut(&name)
                    {
                        entry.latest = latest;
                    }
                    let left = (self.checks_pending - 1).max(0);
                    self.as_mut().rust_mut().get_mut().checks_pending = left;
                    if left == 0 {
                        self.as_mut().set_checking(false);
                    }
                }
            }
        }
    }
}
