use cxx_qt_lib::{
    QByteArray, QHash, QHashPair_i32_QByteArray, QMap, QMapPair_QString_QVariant, QString, QVariant,
};
use omikuji_core::downloads::{self, DownloadKind, DownloadRequest};
use omikuji_core::store::StoreGame;
use std::collections::HashSet;
use std::path::PathBuf;

enum Role {
    AppName = 0,
    Title = 1,
    Banner = 2,
    Coverart = 3,
    Icon = 4,
    IsInstalled = 5,
    HasLibraryEntry = 6,
    InstallPath = 7,
}

fn install_path_string(game: &StoreGame) -> String {
    game.install_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn role_names() -> QHash<QHashPair_i32_QByteArray> {
    let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
    for (role, name) in [
        (Role::AppName, "appName"),
        (Role::Title, "title"),
        (Role::Banner, "banner"),
        (Role::Coverart, "coverart"),
        (Role::Icon, "icon"),
        (Role::IsInstalled, "isInstalled"),
        (Role::HasLibraryEntry, "hasLibraryEntry"),
        (Role::InstallPath, "installPath"),
    ] {
        roles.insert_clone(&(role as i32), &QByteArray::from(name));
    }
    roles
}

pub fn role_data(games: &[StoreGame], imported: &HashSet<String>, row: i32, role: i32) -> QVariant {
    let Some(game) = usize::try_from(row).ok().and_then(|i| games.get(i)) else {
        return QVariant::default();
    };

    match role {
        r if r == Role::AppName as i32 => QVariant::from(&QString::from(&game.app_name)),
        r if r == Role::Title as i32 => QVariant::from(&QString::from(&game.title)),
        r if r == Role::Banner as i32 => {
            QVariant::from(&QString::from(game.banner.as_deref().unwrap_or("")))
        }
        r if r == Role::Coverart as i32 => {
            QVariant::from(&QString::from(game.coverart.as_deref().unwrap_or("")))
        }
        r if r == Role::Icon as i32 => {
            QVariant::from(&QString::from(game.icon.as_deref().unwrap_or("")))
        }
        r if r == Role::IsInstalled as i32 => QVariant::from(&game.is_installed),
        r if r == Role::HasLibraryEntry as i32 => {
            QVariant::from(&imported.contains(&game.app_name))
        }
        r if r == Role::InstallPath as i32 => {
            QVariant::from(&QString::from(&install_path_string(game)))
        }
        _ => QVariant::default(),
    }
}

pub fn game_map(
    games: &[StoreGame],
    imported: &HashSet<String>,
    index: i32,
) -> QMap<QMapPair_QString_QVariant> {
    let mut m = QMap::<QMapPair_QString_QVariant>::default();
    let Some(g) = usize::try_from(index).ok().and_then(|i| games.get(i)) else {
        return m;
    };

    for (key, value) in [
        ("appName", QVariant::from(&QString::from(&g.app_name))),
        ("title", QVariant::from(&QString::from(&g.title))),
        (
            "banner",
            QVariant::from(&QString::from(g.banner.as_deref().unwrap_or(""))),
        ),
        (
            "coverart",
            QVariant::from(&QString::from(g.coverart.as_deref().unwrap_or(""))),
        ),
        (
            "icon",
            QVariant::from(&QString::from(g.icon.as_deref().unwrap_or(""))),
        ),
        ("isInstalled", QVariant::from(&g.is_installed)),
        (
            "hasLibraryEntry",
            QVariant::from(&imported.contains(&g.app_name)),
        ),
        (
            "installPath",
            QVariant::from(&QString::from(&install_path_string(g))),
        ),
    ] {
        m.insert(QString::from(key), value);
    }
    m
}

pub struct InstallOptions<'a> {
    pub install_path: &'a QString,
    pub prefix_path: &'a QString,
    pub runner_version: &'a QString,
    pub is_import: bool,
    pub import_existing: bool,
    pub dlcs: &'a QString,
}

pub fn enqueue_install(source: &str, game: &StoreGame, opts: &InstallOptions) -> QString {
    let prefix = opts.prefix_path.to_string();

    let req = DownloadRequest {
        source: source.to_string(),
        app_id: game.app_name.clone(),
        game_id: String::new(),
        display_name: game.title.clone(),
        banner_url: game.coverart.clone().or(game.banner.clone()),
        install_path: PathBuf::from(opts.install_path.to_string()),
        prefix_path: if prefix.is_empty() {
            None
        } else {
            Some(PathBuf::from(prefix))
        },
        runner_version: opts.runner_version.to_string(),
        temp_dir: None,
        kind: if opts.import_existing {
            DownloadKind::ImportExisting
        } else {
            DownloadKind::Install
        },
        destructive_cleanup: !opts.is_import && !opts.import_existing,
        start_paused: false,
        dlcs: serde_json::from_str(&opts.dlcs.to_string()).unwrap_or_default(),
        alongside: false,
    };

    QString::from(&downloads::manager().enqueue(req))
}
