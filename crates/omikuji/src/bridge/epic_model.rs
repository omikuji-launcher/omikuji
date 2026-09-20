#![allow(clippy::too_many_arguments)]

use super::store_model;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QModelIndex, QString, QVariant};
use lazy_static::lazy_static;
use omikuji_core::store::StoreGame;
use omikuji_core::store::epic::EpicStore;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

lazy_static! {
    static ref EPIC_STORE: Arc<Mutex<EpicStore>> = Arc::new(Mutex::new(EpicStore::new()));
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
        #[qproperty(bool, tool_ready, cxx_name = "toolReady")]
        #[qproperty(bool, tool_installing, cxx_name = "toolInstalling")]
        type EpicModel = super::EpicModelRust;
    }

    unsafe extern "RustQt" {
        #[cxx_name = "rowCount"]
        #[cxx_override]
        fn row_count(self: &EpicModel, parent: &QModelIndex) -> i32;

        #[cxx_override]
        fn data(self: &EpicModel, index: &QModelIndex, role: i32) -> QVariant;

        #[cxx_name = "roleNames"]
        #[cxx_override]
        fn role_names(self: &EpicModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        fn get_login_url(self: &EpicModel) -> QString;

        #[qinvokable]
        fn login(self: Pin<&mut EpicModel>, code: &QString);

        #[qinvokable]
        fn logout(self: Pin<&mut EpicModel>);

        #[qinvokable]
        fn refresh(self: Pin<&mut EpicModel>);

        #[qinvokable]
        fn enqueue_install(
            self: Pin<&mut EpicModel>,
            index: i32,
            install_path: &QString,
            prefix_path: &QString,
            runner_version: &QString,
            is_import: bool,
            import_existing: bool,
            dlcs: &QString,
        ) -> QString;

        #[qinvokable]
        fn get_game_at(self: &EpicModel, index: i32) -> QMap_QString_QVariant;

        #[qinvokable]
        fn install_tools(self: Pin<&mut EpicModel>);

        #[qinvokable]
        fn refresh_tools(self: Pin<&mut EpicModel>);
    }

    unsafe extern "RustQt" {
        #[cxx_name = "beginResetModel"]
        #[inherit]
        fn begin_reset_model(self: Pin<&mut EpicModel>);

        #[cxx_name = "endResetModel"]
        #[inherit]
        fn end_reset_model(self: Pin<&mut EpicModel>);
    }

    impl cxx_qt::Threading for EpicModel {}
}

pub struct EpicModelRust {
    pub games: Vec<StoreGame>,
    // app_name -> library game id, drives the three-state card ui and attributes downloads
    pub library_ids: HashMap<String, String>,
    pub is_logged_in: bool,
    pub is_refreshing: bool,
    pub display_name: QString,
    pub tool_ready: bool,
    pub tool_installing: bool,
}

impl Default for EpicModelRust {
    fn default() -> Self {
        // blocking_lock would panic inside the tokio runtime; try_lock is safe at startup
        let (is_logged_in, display_name) = match EPIC_STORE.try_lock() {
            Ok(store) => (store.is_logged_in(), QString::from(&store.display_name)),
            Err(_) => (false, QString::default()),
        };

        Self {
            games: Vec::new(),
            library_ids: HashMap::new(),
            is_logged_in,
            is_refreshing: false,
            display_name,
            tool_ready: omikuji_core::components::ready(&omikuji_core::components::epic_tools()),
            tool_installing: false,
        }
    }
}

impl qobject::EpicModel {
    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.rust().games.len() as i32
    }

    pub fn role_names(&self) -> qobject::QHash_i32_QByteArray {
        store_model::role_names()
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        store_model::role_data(
            &self.rust().games,
            &self.rust().library_ids,
            index.row(),
            role,
        )
    }

    pub fn get_login_url(&self) -> QString {
        QString::from(&EpicStore::get_login_url())
    }

    pub fn install_tools(mut self: Pin<&mut Self>) {
        if self.rust().tool_installing {
            return;
        }
        self.as_mut().set_tool_installing(true);
        let qt_thread = self.as_mut().qt_thread();
        tokio::spawn(async move {
            let ok = omikuji_core::components::ensure(&omikuji_core::components::epic_tools())
                .await
                .map_err(|e| tracing::error!("epic tools install failed: {}", e))
                .is_ok();
            let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::EpicModel>| {
                obj.as_mut().set_tool_installing(false);
                obj.as_mut().set_tool_ready(ok);
            });
        });
    }

    pub fn refresh_tools(mut self: Pin<&mut Self>) {
        let ready = omikuji_core::components::ready(&omikuji_core::components::epic_tools());
        self.as_mut().set_tool_ready(ready);
    }

    pub fn login(mut self: Pin<&mut Self>, code: &QString) {
        let code_str = code.to_string();
        let qt_thread = self.as_mut().qt_thread();

        tokio::spawn(async move {
            let result = {
                let mut store = EPIC_STORE.lock().await;
                store.login(&code_str).await
            };

            match result {
                Ok(name) => {
                    let display_name = QString::from(&name);
                    let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::EpicModel>| {
                        obj.as_mut().set_is_logged_in(true);
                        obj.as_mut().set_display_name(display_name);
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
                let mut store = EPIC_STORE.lock().await;
                if let Err(e) = store.logout().await {
                    tracing::error!("logout failed: {}", e);
                }
            }
            let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::EpicModel>| {
                obj.as_mut().set_is_logged_in(false);
                obj.as_mut().set_display_name(QString::default());
                obj.as_mut().begin_reset_model();
                let rust = obj.as_mut().rust_mut().get_mut();
                rust.games.clear();
                rust.library_ids.clear();
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
            let (cached, ids_pre) = tokio::task::spawn_blocking(|| {
                let games = omikuji_core::store::epic::load_cached_library();
                let ids = omikuji_core::library::Library::game_ids_by_app_id("epic");
                (games, ids)
            })
            .await
            .unwrap_or_default();

            if !cached.is_empty() {
                let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::EpicModel>| {
                    if !obj.as_ref().games.is_empty() {
                        return;
                    }
                    obj.as_mut().begin_reset_model();
                    let rust = obj.as_mut().rust_mut().get_mut();
                    rust.games = cached;
                    rust.library_ids = ids_pre;
                    obj.as_mut().end_reset_model();
                });
            }

            let result = {
                let mut store = EPIC_STORE.lock().await;
                store.list_games().await
            };

            match result {
                Ok(games) => {
                    let ids = tokio::task::spawn_blocking(|| {
                        omikuji_core::library::Library::game_ids_by_app_id("epic")
                    })
                    .await
                    .unwrap_or_default();

                    let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::EpicModel>| {
                        let unchanged =
                            obj.as_ref().games == games && obj.as_ref().library_ids == ids;
                        if !unchanged {
                            obj.as_mut().begin_reset_model();
                            let rust = obj.as_mut().rust_mut().get_mut();
                            rust.games = games;
                            rust.library_ids = ids;
                            obj.as_mut().end_reset_model();
                        }
                        obj.as_mut().set_is_refreshing(false);
                    });
                }
                Err(e) => {
                    tracing::error!("refresh failed: {}", e);
                    let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::EpicModel>| {
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

        let game_id = self
            .rust()
            .library_ids
            .get(&game.app_name)
            .cloned()
            .unwrap_or_default();

        store_model::enqueue_install(
            "epic",
            game,
            &store_model::InstallOptions {
                install_path,
                prefix_path,
                runner_version,
                is_import,
                import_existing,
                dlcs,
                game_id: &game_id,
            },
        )
    }

    pub fn get_game_at(
        &self,
        index: i32,
    ) -> cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant> {
        store_model::game_map(&self.rust().games, &self.rust().library_ids, index)
    }
}
