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
        #[qproperty(i32, pending_count, cxx_name = "pendingCount")]
        #[qproperty(i32, total_count, cxx_name = "totalCount")]
        #[qproperty(bool, in_progress, cxx_name = "inProgress")]
        #[qproperty(bool, all_done, cxx_name = "allDone")]
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
        #[cxx_name = "installEager"]
        fn install_eager(self: Pin<&mut ComponentsBridge>);

        #[qinvokable]
        #[cxx_name = "reinstallComponent"]
        fn reinstall_component(self: Pin<&mut ComponentsBridge>, name: QString);

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
    pub pending_count: i32,
    pub total_count: i32,
    pub in_progress: bool,
    pub all_done: bool,
    pub checking: bool,

    statuses: HashMap<String, ComponentStatusEntry>,
    checks_pending: i32,
}

#[derive(Clone, Default)]
struct ComponentStatusEntry {
    status: String,
    percent: f64,
    version: String,
    latest: String,
    error: String,
}

impl Default for ComponentsRust {
    fn default() -> Self {
        let specs = core_components::specs::all();
        let pending = core_components::eager_pending();

        let mut statuses = HashMap::new();
        for spec in specs {
            let (status, version) = match core_components::status_for(spec) {
                core_components::ComponentStatus::Installed { version } => {
                    ("completed".to_string(), version)
                }
                core_components::ComponentStatus::Missing => ("missing".to_string(), String::new()),
            };
            statuses.insert(
                spec.name.to_string(),
                ComponentStatusEntry {
                    status,
                    version,
                    ..Default::default()
                },
            );
        }

        Self {
            pending_count: pending.len() as i32,
            total_count: specs.len() as i32,
            in_progress: false,
            all_done: pending.is_empty(),
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
                        "latest": v.latest,
                        "error": v.error,
                    }),
                )
            })
            .collect();
        QString::from(&serde_json::Value::Object(map).to_string())
    }

    fn refresh(mut self: Pin<&mut Self>) {
        let specs = core_components::specs::all();
        let pending = core_components::eager_pending();

        for spec in specs {
            let current = self.statuses.get(spec.name).cloned();
            let latest = current
                .as_ref()
                .map(|c| c.latest.clone())
                .unwrap_or_default();
            let entry = match core_components::status_for(spec) {
                core_components::ComponentStatus::Installed { version } => ComponentStatusEntry {
                    status: "completed".into(),
                    percent: 100.0,
                    version,
                    latest,
                    ..Default::default()
                },
                // don't overwrite a live "installing" status with "missing" becuase the file check races with the install thread
                core_components::ComponentStatus::Missing => match current {
                    Some(c)
                        if matches!(
                            c.status.as_str(),
                            "installing" | "downloading" | "extracting"
                        ) =>
                    {
                        c
                    }
                    _ => ComponentStatusEntry {
                        status: "missing".into(),
                        latest,
                        ..Default::default()
                    },
                },
            };
            self.as_mut()
                .rust_mut()
                .get_mut()
                .statuses
                .insert(spec.name.into(), entry);
        }

        let pc = pending.len() as i32;
        self.as_mut().set_pending_count(pc);
        self.as_mut().set_all_done(pc == 0);
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

    fn install_eager(self: Pin<&mut Self>) {
        self.spawn_install(core_components::eager_pending());
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

    fn reinstall_component(mut self: Pin<&mut Self>, name: QString) {
        let target = name.to_string();
        let Some(spec) = core_components::specs::all()
            .iter()
            .find(|s| s.name == target)
        else {
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
            let has_active = self.statuses.values().any(|s| {
                matches!(
                    s.status.as_str(),
                    "installing" | "downloading" | "extracting"
                )
            });
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
