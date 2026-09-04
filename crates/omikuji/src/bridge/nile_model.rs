#![allow(clippy::too_many_arguments)]

use super::store_model;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QModelIndex, QString, QVariant};
use lazy_static::lazy_static;
use omikuji_core::store::StoreGame;
use omikuji_core::store::nile::NileStore;
use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

lazy_static! {
    static ref NILE_STORE: Arc<Mutex<NileStore>> = Arc::new(Mutex::new(NileStore::new()));
}

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!(<QtCore/QAbstractListModel>);
        type QAbstractListModel;

        include!("cxx-qt-lib/qmodelindex.h");
        type QModelIndex = cxx_qt_lib::QModelIndex;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qbytearray.h");
        type QByteArray = cxx_qt_lib::QByteArray;
        include!("cxx-qt-lib/qhash.h");
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
        include!("cxx-qt-lib/qmap.h");
        type QMap_QString_QVariant = cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant>;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        #[qproperty(bool, is_logged_in, cxx_name = "isLoggedIn")]
        #[qproperty(bool, is_refreshing, cxx_name = "isRefreshing")]
        #[qproperty(QString, display_name, cxx_name = "displayName")]
        #[qproperty(QString, login_url, cxx_name = "loginUrl")]
        #[qproperty(bool, tool_ready, cxx_name = "toolReady")]
        #[qproperty(bool, tool_installing, cxx_name = "toolInstalling")]
        type NileModel = super::NileModelRust;
    }

    unsafe extern "RustQt" {
        #[cxx_name = "rowCount"]
        #[cxx_override]
        fn row_count(self: &NileModel, parent: &QModelIndex) -> i32;

        #[cxx_override]
        fn data(self: &NileModel, index: &QModelIndex, role: i32) -> QVariant;

        #[cxx_name = "roleNames"]
        #[cxx_override]
        fn role_names(self: &NileModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        fn begin_login(self: Pin<&mut NileModel>);

        #[qinvokable]
        fn login(self: Pin<&mut NileModel>, code: &QString);

        #[qinvokable]
        fn logout(self: Pin<&mut NileModel>);

        #[qinvokable]
        fn refresh(self: Pin<&mut NileModel>);

        #[qinvokable]
        fn enqueue_install(
            self: Pin<&mut NileModel>,
            index: i32,
            install_path: &QString,
            prefix_path: &QString,
            runner_version: &QString,
            is_import: bool,
            import_existing: bool,
            dlcs: &QString,
        ) -> QString;

        #[qinvokable]
        fn get_game_at(self: &NileModel, index: i32) -> QMap_QString_QVariant;

        #[qinvokable]
        fn install_tools(self: Pin<&mut NileModel>);

        #[qinvokable]
        fn refresh_tools(self: Pin<&mut NileModel>);
    }

    unsafe extern "RustQt" {
        #[cxx_name = "beginResetModel"]
        #[inherit]
        fn begin_reset_model(self: Pin<&mut NileModel>);

        #[cxx_name = "endResetModel"]
        #[inherit]
        fn end_reset_model(self: Pin<&mut NileModel>);
    }

    impl cxx_qt::Threading for NileModel {}
}

pub struct NileModelRust {
    pub games: Vec<StoreGame>,
    pub imported: HashSet<String>,
    pub is_logged_in: bool,
    pub is_refreshing: bool,
    pub display_name: QString,
    pub login_url: QString,
    pub tool_ready: bool,
    pub tool_installing: bool,
    pub login_fetching: bool,
}

impl Default for NileModelRust {
    fn default() -> Self {
        let (is_logged_in, display_name) = match NILE_STORE.try_lock() {
            Ok(store) => (store.is_logged_in(), QString::from(&store.display_name)),
            Err(_) => (false, QString::default()),
        };

        Self {
            games: Vec::new(),
            imported: HashSet::new(),
            is_logged_in,
            is_refreshing: false,
            display_name,
            login_url: QString::default(),
            tool_ready: omikuji_core::components::ready(&omikuji_core::components::nile_tools()),
            tool_installing: false,
            login_fetching: false,
        }
    }
}

impl qobject::NileModel {
    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.rust().games.len() as i32
    }

    pub fn role_names(&self) -> qobject::QHash_i32_QByteArray {
        store_model::role_names()
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        store_model::role_data(&self.rust().games, &self.rust().imported, index.row(), role)
    }

    pub fn install_tools(mut self: Pin<&mut Self>) {
        if self.rust().tool_installing {
            return;
        }
        self.as_mut().set_tool_installing(true);
        let qt_thread = self.as_mut().qt_thread();
        tokio::spawn(async move {
            let ok = omikuji_core::components::ensure(&omikuji_core::components::nile_tools())
                .await
                .map_err(|e| tracing::error!("nile tools install failed: {}", e))
                .is_ok();
            let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::NileModel>| {
                obj.as_mut().set_tool_installing(false);
                obj.as_mut().set_tool_ready(ok);
                if ok {
                    obj.as_mut().begin_login();
                }
            });
        });
    }

    pub fn refresh_tools(mut self: Pin<&mut Self>) {
        let ready = omikuji_core::components::ready(&omikuji_core::components::nile_tools());
        self.as_mut().set_tool_ready(ready);
        if ready {
            self.as_mut().begin_login();
        }
    }

    pub fn begin_login(mut self: Pin<&mut Self>) {
        if self.rust().is_logged_in
            || self.rust().login_fetching
            || !self.rust().login_url.is_empty()
        {
            return;
        }
        self.as_mut().rust_mut().get_mut().login_fetching = true;
        let qt_thread = self.as_mut().qt_thread();

        tokio::spawn(async move {
            let result = {
                let mut store = NILE_STORE.lock().await;
                store.begin_login().await
            };

            let url = match result {
                Ok(url) => QString::from(&url),
                Err(e) => {
                    tracing::error!("nile begin_login failed: {}", e);
                    QString::default()
                }
            };

            let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::NileModel>| {
                obj.as_mut().set_login_url(url);
                obj.as_mut().rust_mut().get_mut().login_fetching = false;
            });
        });
    }

    pub fn login(mut self: Pin<&mut Self>, code: &QString) {
        let code_str = code.to_string();
        let qt_thread = self.as_mut().qt_thread();

        tokio::spawn(async move {
            let result = {
                let mut store = NILE_STORE.lock().await;
                store.login(&code_str).await
            };

            match result {
                Ok(name) => {
                    let display_name = QString::from(&name);
                    let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::NileModel>| {
                        obj.as_mut().set_is_logged_in(true);
                        obj.as_mut().set_display_name(display_name);
                        obj.as_mut().set_login_url(QString::default());
                        obj.as_mut().refresh();
                    });
                }
                Err(e) => {
                    tracing::error!("login failed: {}", e);
                }
            }
        });
    }

    pub fn logout(mut self: Pin<&mut Self>) {
        let qt_thread = self.as_mut().qt_thread();
        tokio::spawn(async move {
            {
                let mut store = NILE_STORE.lock().await;
                if let Err(e) = store.logout().await {
                    tracing::error!("logout failed: {}", e);
                }
            }
            let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::NileModel>| {
                obj.as_mut().set_is_logged_in(false);
                obj.as_mut().set_display_name(QString::default());
                obj.as_mut().set_login_url(QString::default());
                obj.as_mut().begin_reset_model();
                let rust = obj.as_mut().rust_mut().get_mut();
                rust.games.clear();
                rust.imported.clear();
                obj.as_mut().end_reset_model();
            });
        });
    }

    pub fn refresh(mut self: Pin<&mut Self>) {
        if self.rust().is_refreshing {
            return;
        }
        self.as_mut().set_is_refreshing(true);
        let qt_thread = self.as_mut().qt_thread();

        tokio::spawn(async move {
            let (cached, imported_pre) = tokio::task::spawn_blocking(|| {
                let games = omikuji_core::store::nile::load_cached_library();
                let imported: HashSet<String> =
                    omikuji_core::library::Library::app_ids_for_source("nile")
                        .into_iter()
                        .collect();
                (games, imported)
            })
            .await
            .unwrap_or_default();

            if !cached.is_empty() {
                let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::NileModel>| {
                    if !obj.as_ref().games.is_empty() {
                        return;
                    }
                    obj.as_mut().begin_reset_model();
                    let rust = obj.as_mut().rust_mut().get_mut();
                    rust.games = cached;
                    rust.imported = imported_pre;
                    obj.as_mut().end_reset_model();
                });
            }

            let result = {
                let mut store = NILE_STORE.lock().await;
                store.list_games().await
            };

            match result {
                Ok(games) => {
                    let imported: HashSet<String> = tokio::task::spawn_blocking(|| {
                        omikuji_core::library::Library::app_ids_for_source("nile")
                            .into_iter()
                            .collect()
                    })
                    .await
                    .unwrap_or_default();

                    let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::NileModel>| {
                        let unchanged =
                            obj.as_ref().games == games && obj.as_ref().imported == imported;
                        if !unchanged {
                            obj.as_mut().begin_reset_model();
                            let rust = obj.as_mut().rust_mut().get_mut();
                            rust.games = games;
                            rust.imported = imported;
                            obj.as_mut().end_reset_model();
                        }
                        obj.as_mut().set_is_refreshing(false);
                    });
                }
                Err(e) => {
                    tracing::error!("refresh failed: {}", e);
                    let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::NileModel>| {
                        obj.as_mut().set_is_refreshing(false);
                    });
                }
            }
        });
    }

    pub fn enqueue_install(
        self: Pin<&mut Self>,
        index: i32,
        install_path: &QString,
        prefix_path: &QString,
        runner_version: &QString,
        is_import: bool,
        import_existing: bool,
        dlcs: &QString,
    ) -> QString {
        let Some(game) = usize::try_from(index)
            .ok()
            .and_then(|i| self.rust().games.get(i))
        else {
            tracing::error!("enqueue_install: bad index {}", index);
            return QString::default();
        };

        store_model::enqueue_install(
            "nile",
            game,
            &store_model::InstallOptions {
                install_path,
                prefix_path,
                runner_version,
                is_import,
                import_existing,
                dlcs,
            },
        )
    }

    pub fn get_game_at(
        &self,
        index: i32,
    ) -> cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant> {
        store_model::game_map(&self.rust().games, &self.rust().imported, index)
    }
}
