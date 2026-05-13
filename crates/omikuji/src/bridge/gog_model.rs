use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QByteArray, QModelIndex, QString, QVariant};
use std::collections::HashSet;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use lazy_static::lazy_static;
use omikuji_core::gog::{GogStore, GogGame};
use omikuji_core::downloads::{self, DownloadRequest};

lazy_static! {
    static ref GOG_STORE: Arc<Mutex<GogStore>> = Arc::new(Mutex::new(GogStore::new()));
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
        type QHash_i32_QByteArray =
            cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;
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
        type GogModel = super::GogModelRust;
    }

    unsafe extern "RustQt" {
        #[cxx_name = "rowCount"]
        #[cxx_override]
        fn row_count(self: &GogModel, parent: &QModelIndex) -> i32;

        #[cxx_override]
        fn data(self: &GogModel, index: &QModelIndex, role: i32) -> QVariant;

        #[cxx_name = "roleNames"]
        #[cxx_override]
        fn role_names(self: &GogModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        fn get_login_url(self: &GogModel) -> QString;

        #[qinvokable]
        fn login(self: Pin<&mut GogModel>, code: &QString);

        #[qinvokable]
        fn logout(self: Pin<&mut GogModel>);

        #[qinvokable]
        fn refresh(self: Pin<&mut GogModel>);

        #[qinvokable]
        fn enqueue_install(
            self: Pin<&mut GogModel>,
            index: i32,
            install_path: &QString,
            prefix_path: &QString,
            runner_version: &QString,
            is_import: bool,
        ) -> QString;

        #[qinvokable]
        fn get_game_at(self: &GogModel, index: i32) -> QMap_QString_QVariant;

        #[qinvokable]
        fn is_logged_in_sync(self: &GogModel) -> bool;
    }

    unsafe extern "RustQt" {
        #[cxx_name = "beginResetModel"]
        #[inherit]
        fn begin_reset_model(self: Pin<&mut GogModel>);

        #[cxx_name = "endResetModel"]
        #[inherit]
        fn end_reset_model(self: Pin<&mut GogModel>);
    }

    impl cxx_qt::Threading for GogModel {}
}

pub struct GogModelRust {
    pub games: Vec<GogGame>,
    pub imported: HashSet<String>,
    pub is_logged_in: bool,
    pub is_refreshing: bool,
    pub display_name: QString,
}

impl Default for GogModelRust {
    fn default() -> Self {
        let (is_logged_in, display_name) = match GOG_STORE.try_lock() {
            Ok(store) => (store.is_logged_in(), QString::from(&store.display_name)),
            Err(_) => (false, QString::default()),
        };

        Self {
            games: Vec::new(),
            imported: HashSet::new(),
            is_logged_in,
            is_refreshing: false,
            display_name,
        }
    }
}

enum GogRoles {
    AppName = 0,
    Title = 1,
    Banner = 2,
    Coverart = 3,
    Icon = 4,
    IsInstalled = 5,
    HasLibraryEntry = 6,
    InstallPath = 7,
}

impl qobject::GogModel {
    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.rust().games.len() as i32
    }

    pub fn role_names(&self) -> qobject::QHash_i32_QByteArray {
        let mut roles = qobject::QHash_i32_QByteArray::default();
        roles.insert_clone(&(GogRoles::AppName as i32), &QByteArray::from("appName"));
        roles.insert_clone(&(GogRoles::Title as i32), &QByteArray::from("title"));
        roles.insert_clone(&(GogRoles::Banner as i32), &QByteArray::from("banner"));
        roles.insert_clone(&(GogRoles::Coverart as i32), &QByteArray::from("coverart"));
        roles.insert_clone(&(GogRoles::Icon as i32), &QByteArray::from("icon"));
        roles.insert_clone(&(GogRoles::IsInstalled as i32), &QByteArray::from("isInstalled"));
        roles.insert_clone(&(GogRoles::HasLibraryEntry as i32), &QByteArray::from("hasLibraryEntry"));
        roles.insert_clone(&(GogRoles::InstallPath as i32), &QByteArray::from("installPath"));
        roles
    }

    pub fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let i = index.row() as usize;
        if i >= self.rust().games.len() {
            return QVariant::default();
        }

        let game = &self.rust().games[i];

        match role {
            r if r == GogRoles::AppName as i32 => QVariant::from(&QString::from(&game.app_name)),
            r if r == GogRoles::Title as i32 => QVariant::from(&QString::from(&game.title)),
            r if r == GogRoles::Banner as i32 => QVariant::from(&QString::from(game.banner.as_deref().unwrap_or(""))),
            r if r == GogRoles::Coverart as i32 => QVariant::from(&QString::from(game.coverart.as_deref().unwrap_or(""))),
            r if r == GogRoles::Icon as i32 => QVariant::from(&QString::from(game.icon.as_deref().unwrap_or(""))),
            r if r == GogRoles::IsInstalled as i32 => QVariant::from(&game.is_installed),
            r if r == GogRoles::HasLibraryEntry as i32 => {
                QVariant::from(&self.rust().imported.contains(&game.app_name))
            }
            r if r == GogRoles::InstallPath as i32 => {
                let s = game
                    .install_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                QVariant::from(&QString::from(&s))
            }
            _ => QVariant::default(),
        }
    }

    pub fn get_login_url(&self) -> QString {
        QString::from(&GogStore::get_login_url())
    }

    pub fn is_logged_in_sync(&self) -> bool {
        GOG_STORE.blocking_lock().is_logged_in()
    }

    pub fn login(mut self: Pin<&mut Self>, code: &QString) {
        let code_str = code.to_string();
        let qt_thread = self.as_mut().qt_thread();

        tokio::spawn(async move {
            let result = {
                let mut store = GOG_STORE.lock().await;
                store.login(&code_str).await
            };

            match result {
                Ok(name) => {
                    let display_name = QString::from(&name);
                    let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::GogModel>| {
                        obj.as_mut().set_is_logged_in(true);
                        obj.as_mut().set_display_name(display_name);
                        obj.as_mut().refresh();
                    });
                }
                Err(e) => {
                    eprintln!("[GOG] Login failed: {}", e);
                }
            }
        });
    }

    pub fn logout(mut self: Pin<&mut Self>) {
        let qt_thread = self.as_mut().qt_thread();
        tokio::spawn(async move {
            {
                let mut store = GOG_STORE.lock().await;
                store.logout();
            }
            let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::GogModel>| {
                obj.as_mut().set_is_logged_in(false);
                obj.as_mut().set_display_name(QString::default());
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
                let games = omikuji_core::gog::load_cached_library();
                let imported: HashSet<String> =
                    omikuji_core::library::Library::app_ids_for_source("gog")
                        .into_iter()
                        .collect();
                (games, imported)
            })
            .await
            .unwrap_or_default();

            if !cached.is_empty() {
                let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::GogModel>| {
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
                let mut store = GOG_STORE.lock().await;
                store.list_games().await
            };

            match result {
                Ok(games) => {
                    let imported: HashSet<String> = tokio::task::spawn_blocking(|| {
                        omikuji_core::library::Library::app_ids_for_source("gog")
                            .into_iter()
                            .collect()
                    })
                    .await
                    .unwrap_or_default();

                    let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::GogModel>| {
                        let unchanged = obj.as_ref().games == games
                            && obj.as_ref().imported == imported;
                        if !unchanged {
                            obj.as_mut().begin_reset_model();
                            let rust = obj.as_mut().rust_mut().get_mut();
                            rust.games = games;
                            rust.imported = imported;
                            obj.as_mut().end_reset_model();
                        } else {
                        }
                        obj.as_mut().set_is_refreshing(false);
                    });
                }
                Err(e) => {
                    eprintln!("[GOG] Refresh failed: {}", e);
                    let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::GogModel>| {
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
    ) -> QString {
        let i = index as usize;
        let Some(game) = self.rust().games.get(i).cloned() else {
            eprintln!("[gog] enqueue_install: bad index {}", index);
            return QString::default();
        };

        let banner_url = game
            .coverart
            .clone()
            .or(game.banner.clone());

        let prefix = prefix_path.to_string();

        let req = DownloadRequest {
            source: "gog".to_string(),
            app_id: game.app_name.clone(),
            display_name: game.title.clone(),
            banner_url,
            install_path: PathBuf::from(install_path.to_string()),
            prefix_path: if prefix.is_empty() { None } else { Some(PathBuf::from(prefix)) },
            runner_version: runner_version.to_string(),
            temp_dir: None,
            kind: omikuji_core::downloads::DownloadKind::Install,
            destructive_cleanup: !is_import,
            start_paused: false,
        };

        let id = downloads::manager().enqueue(req);
        QString::from(&id)
    }

    pub fn get_game_at(&self, index: i32) -> cxx_qt_lib::QMap<cxx_qt_lib::QMapPair_QString_QVariant> {
        let mut m = cxx_qt_lib::QMap::<cxx_qt_lib::QMapPair_QString_QVariant>::default();
        let i = index as usize;
        let Some(g) = self.rust().games.get(i) else { return m };

        m.insert(QString::from("appName"), QVariant::from(&QString::from(&g.app_name)));
        m.insert(QString::from("title"), QVariant::from(&QString::from(&g.title)));
        m.insert(QString::from("banner"), QVariant::from(&QString::from(g.banner.as_deref().unwrap_or(""))));
        m.insert(QString::from("coverart"), QVariant::from(&QString::from(g.coverart.as_deref().unwrap_or(""))));
        m.insert(QString::from("icon"), QVariant::from(&QString::from(g.icon.as_deref().unwrap_or(""))));
        m.insert(QString::from("isInstalled"), QVariant::from(&g.is_installed));
        m.insert(
            QString::from("hasLibraryEntry"),
            QVariant::from(&self.rust().imported.contains(&g.app_name)),
        );
        let install_path = g
            .install_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        m.insert(QString::from("installPath"), QVariant::from(&QString::from(&install_path)));
        m
    }
}
