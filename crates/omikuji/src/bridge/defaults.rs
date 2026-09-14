use cxx_qt::{CxxQtType, Threading};
use omikuji_core::defaults::{ConfigSections, Defaults, FIELDS, defaults_path};
use omikuji_core::fs_watcher::FileWatcher;
use std::pin::Pin;
use std::time::{Duration, Instant};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qmap.h");
        type QMap_QString_QVariant = cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant>;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type DefaultsBridge = super::DefaultsRust;
    }

    unsafe extern "RustQt" {
        #[qsignal]
        #[cxx_name = "changed"]
        fn changed(self: Pin<&mut DefaultsBridge>);
    }

    impl cxx_qt::Threading for DefaultsBridge {}

    unsafe extern "RustQt" {
        #[qinvokable]
        #[cxx_name = "getConfig"]
        fn get_config(self: &DefaultsBridge) -> QMap_QString_QVariant;

        #[qinvokable]
        #[cxx_name = "setKeysJson"]
        fn set_keys_json(self: &DefaultsBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "fieldsJson"]
        fn fields_json(self: &DefaultsBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "updateField"]
        fn update_field(self: Pin<&mut DefaultsBridge>, key: &QString, value: &QString) -> bool;

        #[qinvokable]
        #[cxx_name = "resetField"]
        fn reset_field(self: Pin<&mut DefaultsBridge>, key: &QString) -> bool;

        #[qinvokable]
        #[cxx_name = "initWatcher"]
        fn init_watcher(self: Pin<&mut DefaultsBridge>);
    }
}

pub struct DefaultsRust {
    pub data: Defaults,
    pub watcher: Option<FileWatcher>,
    pub suppress_reload_until: Option<Instant>,
}

impl Default for DefaultsRust {
    fn default() -> Self {
        Self {
            data: Defaults::load(),
            watcher: None,
            suppress_reload_until: None,
        }
    }
}

impl qobject::DefaultsBridge {
    fn get_config(&self) -> cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant> {
        build_defaults_map(&self.data)
    }

    fn set_keys_json(&self) -> cxx_qt_lib::QString {
        let keys = collect_set_keys(&self.data);
        let json = serde_json::to_string(&keys).unwrap_or_else(|_| "[]".to_string());
        cxx_qt_lib::QString::from(&json)
    }

    fn fields_json(&self) -> cxx_qt_lib::QString {
        let json = serde_json::to_string(FIELDS).unwrap_or_else(|_| "[]".to_string());
        cxx_qt_lib::QString::from(&json)
    }

    fn update_field(
        mut self: Pin<&mut Self>,
        key: &cxx_qt_lib::QString,
        value: &cxx_qt_lib::QString,
    ) -> bool {
        let k = key.to_string();
        let v = value.to_string();
        let d = &mut self.as_mut().rust_mut().get_mut().data;
        let ok = apply_to_defaults(d, &k, &v);
        if ok {
            self.as_mut().persist();
            self.as_mut().changed();
        }
        ok
    }

    fn reset_field(mut self: Pin<&mut Self>, key: &cxx_qt_lib::QString) -> bool {
        let k = key.to_string();
        let d = &mut self.as_mut().rust_mut().get_mut().data;
        let ok = clear_in_defaults(d, &k);
        if ok {
            self.as_mut().persist();
            self.as_mut().changed();
        }
        ok
    }

    fn persist(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().get_mut().suppress_reload_until =
            Some(Instant::now() + Duration::from_millis(600));
        if let Err(e) = self.as_ref().rust().data.save() {
            tracing::error!("save failed: {}", e);
        }
    }

    fn init_watcher(mut self: Pin<&mut Self>) {
        if self.as_ref().rust().watcher.is_some() {
            return;
        }
        let path = defaults_path();
        let qt_thread = self.as_mut().qt_thread();
        let watcher = FileWatcher::watch(path, move || {
            let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::DefaultsBridge>| {
                let within_window = obj
                    .as_ref()
                    .rust()
                    .suppress_reload_until
                    .map(|until| Instant::now() < until)
                    .unwrap_or(false);
                if within_window {
                    return;
                }
                obj.as_mut().rust_mut().get_mut().data = Defaults::load();
                obj.as_mut().changed();
            });
        });
        match watcher {
            Ok(w) => {
                self.as_mut().rust_mut().get_mut().watcher = Some(w);
                tracing::debug!("watching {} via notify", defaults_path().display());
            }
            Err(e) => tracing::error!("failed to start watcher: {e}"),
        }
    }
}

macro_rules! defaults_get {
    (str, $m:ident, $key:literal, $v:expr, $base:expr) => {
        $m.insert(
            QString::from($key),
            QVariant::from(&QString::from(&*$v.clone().unwrap_or($base))),
        );
    };
    (bool, $m:ident, $key:literal, $v:expr, $base:expr) => {
        $m.insert(QString::from($key), QVariant::from(&$v.unwrap_or($base)));
    };
    (int, $m:ident, $key:literal, $v:expr, $base:expr) => {
        $m.insert(
            QString::from($key),
            QVariant::from(&($v.unwrap_or($base) as i32)),
        );
    };
    (map, $m:ident, $key:literal, $v:expr, $base:expr) => {
        if let Ok(json) = serde_json::to_string(&$v) {
            $m.insert(QString::from($key), QVariant::from(&QString::from(&*json)));
        }
    };
}

macro_rules! defaults_set {
    (str, $d:ident, $key:ident, $value:ident, $lit:literal, $($path:ident).+) => {
        if $key == $lit {
            $d.$($path).+ = Some($value.to_string());
            return true;
        }
    };
    (bool, $d:ident, $key:ident, $value:ident, $lit:literal, $($path:ident).+) => {
        if $key == $lit {
            $d.$($path).+ = Some($value == "true");
            return true;
        }
    };
    (int, $d:ident, $key:ident, $value:ident, $lit:literal, $($path:ident).+) => {
        if $key == $lit {
            $d.$($path).+ = Some($value.parse().unwrap_or(0));
            return true;
        }
    };
    (map, $d:ident, $key:ident, $value:ident, $lit:literal, $($path:ident).+) => {
        if $key == $lit {
            if let Ok(parsed) = serde_json::from_str($value) {
                $d.$($path).+ = parsed;
                return true;
            }
            return false;
        }
    };
}

macro_rules! defaults_clear {
    (map, $d:ident, $key:ident, $lit:literal, $($path:ident).+) => {
        if $key == $lit {
            $d.$($path).+.clear();
            return true;
        }
    };
    ($kind:ident, $d:ident, $key:ident, $lit:literal, $($path:ident).+) => {
        if $key == $lit {
            $d.$($path).+ = None;
            return true;
        }
    };
}

macro_rules! defaults_diff {
    (str, $k:ident, $d:ident, $key:literal, $base:expr, $($path:ident).+) => {
        if $d.$($path).+.as_ref().is_some_and(|v| v != &$base) {
            $k.push($key.into());
        }
    };
    (map, $k:ident, $d:ident, $key:literal, $base:expr, $($path:ident).+) => {
        if !$d.$($path).+.is_empty() {
            $k.push($key.into());
        }
    };
    ($kind:ident, $k:ident, $d:ident, $key:literal, $base:expr, $($path:ident).+) => {
        if $d.$($path).+.is_some_and(|v| v != $base) {
            $k.push($key.into());
        }
    };
}

// equality with the baseline default = nothing to undo, no reset badge
macro_rules! defaults_fields {
    ($( $key:literal => $kind:ident, $group:ident, $($path:ident).+ ),* $(,)?) => {
        fn build_defaults_map(d: &Defaults) -> cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant> {
            use cxx_qt_lib::{QMap, QMapPair_QString_QVariant, QString, QVariant};

            let mut m = QMap::<QMapPair_QString_QVariant>::default();
            let stock = ConfigSections::default();
            $( defaults_get!($kind, m, $key, d.$($path).+, stock.$($path).+); )*
            m
        }

        fn apply_to_defaults(d: &mut Defaults, key: &str, value: &str) -> bool {
            $( defaults_set!($kind, d, key, value, $key, $($path).+); )*
            tracing::warn!("unknown key: {}", key);
            false
        }

        fn clear_in_defaults(d: &mut Defaults, key: &str) -> bool {
            $( defaults_clear!($kind, d, key, $key, $($path).+); )*
            tracing::warn!("unknown key to reset: {}", key);
            false
        }

        fn collect_set_keys(d: &Defaults) -> Vec<String> {
            let mut k = Vec::new();
            let stock = ConfigSections::default();
            $( defaults_diff!($kind, k, d, $key, stock.$($path).+, $($path).+); )*
            k
        }
    };
}

omikuji_core::with_default_fields!(defaults_fields);
