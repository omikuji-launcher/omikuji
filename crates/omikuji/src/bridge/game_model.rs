#![allow(clippy::too_many_arguments)]

mod drains;
mod shortcuts;
mod steam;
mod launch;
mod updates;
mod epic;
mod gog;
mod gacha;
mod scripts;

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
        include!("cxx-qt-lib/qlist.h");
        type QList_i32 = cxx_qt_lib::QList<i32>;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = QAbstractListModel]
        #[qproperty(i32, count)]
        #[qproperty(bool, preparing)]
        #[qproperty(bool, wine_command_running, cxx_name = "wineCommandRunning")]
        type GameModel = super::GameModelRust;
    }

    unsafe extern "RustQt" {
        #[qsignal]
        fn game_stopped(self: Pin<&mut GameModel>, game_id: &QString);

        // request_id matches the open_file_dialog call that triggered this
        #[qsignal]
        fn file_dialog_result(
            self: Pin<&mut GameModel>,
            request_id: &QString,
            path: &QString,
        );

        // payload is json: { "download": "123", "install": "456", "error": "" }
        // sizes are stringified u64 becuase js Number loses precision above 2^53
        #[qsignal]
        fn install_size_result(
            self: Pin<&mut GameModel>,
            request_id: &QString,
            payload: &QString,
        );

        #[qsignal]
        fn notification(
            self: Pin<&mut GameModel>,
            level: &QString,
            title: &QString,
            message: &QString,
        );

        // download_size is stringified u64, same js precision reason as install_size_result
        #[qsignal]
        fn update_required(
            self: Pin<&mut GameModel>,
            game_id: &QString,
            app_id: &QString,
            display_name: &QString,
            from_version: &QString,
            to_version: &QString,
            download_size: &QString,
            can_diff: bool,
            delta_supported: bool,
        );

        // cxx_name required: cxx-qt doesn't auto-camelCase signal names for qml handlers
        #[qsignal]
        #[cxx_name = "gachaManifestsReady"]
        fn gacha_manifests_ready(self: Pin<&mut GameModel>, fetched: i32);

        #[qsignal]
        fn updates_queued(self: Pin<&mut GameModel>, epic_count: i32, gog_count: i32);

        #[qsignal]
        #[cxx_name = "gameLogAppended"]
        fn game_log_appended(self: Pin<&mut GameModel>, game_id: &QString);

        #[qsignal]
        #[cxx_name = "prepareOutput"]
        fn prepare_output(self: Pin<&mut GameModel>, line: &QString);

        #[qsignal]
        #[cxx_name = "prepareFinished"]
        fn prepare_finished(self: Pin<&mut GameModel>, ok: bool, error: &QString);

        #[qsignal]
        #[cxx_name = "wineCommandOutput"]
        fn wine_command_output(self: Pin<&mut GameModel>, line: &QString);

        #[qsignal]
        #[cxx_name = "wineCommandFinished"]
        fn wine_command_finished(self: Pin<&mut GameModel>, ok: bool, error: &QString);

        #[qsignal]
        fn error_required(
            self: Pin<&mut GameModel>,
            game_id: &QString,
            display_name: &QString,
            title: &QString,
            message: &QString,
            action: &QString,
        );

        #[cxx_name = "rowCount"]
        #[cxx_override]
        fn row_count(self: &GameModel, parent: &QModelIndex) -> i32;

        #[cxx_override]
        fn data(self: &GameModel, index: &QModelIndex, role: i32) -> QVariant;

        #[cxx_name = "roleNames"]
        #[cxx_override]
        fn role_names(self: &GameModel) -> QHash_i32_QByteArray;

        #[qinvokable]
        fn begin_new_game(self: Pin<&mut GameModel>) -> QMap_QString_QVariant;

        #[qinvokable]
        fn get_draft_config(self: &GameModel) -> QMap_QString_QVariant;

        #[qinvokable]
        fn update_draft_field(self: Pin<&mut GameModel>, key: &QString, value: &QString) -> bool;

        #[qinvokable]
        fn commit_new_game(self: Pin<&mut GameModel>) -> QString;

        #[qinvokable]
        fn discard_draft(self: Pin<&mut GameModel>);

        #[qinvokable]
        #[cxx_name = "applySortMode"]
        fn apply_sort_mode(self: Pin<&mut GameModel>, value: &QString);

        #[qinvokable]
        #[cxx_name = "moveGame"]
        fn move_game(self: Pin<&mut GameModel>, from: i32, to: i32);

        #[qinvokable]
        #[cxx_name = "commitOrder"]
        fn commit_order(self: Pin<&mut GameModel>);

        #[qinvokable]
        fn begin_edit_game(self: Pin<&mut GameModel>, index: i32) -> QMap_QString_QVariant;

        #[qinvokable]
        fn commit_edit_game(self: Pin<&mut GameModel>, game_id: &QString) -> bool;

        #[qinvokable]
        fn remove_game(self: Pin<&mut GameModel>, index: i32);

        #[qinvokable]
        fn remove_game_with_prefix(self: Pin<&mut GameModel>, index: i32);

        #[qinvokable]
        fn game_prefix_info(self: &GameModel, index: i32) -> QString;

        #[qinvokable]
        fn refresh(self: Pin<&mut GameModel>, selected_index: i32) -> QString;

        #[qinvokable]
        fn get_game(self: &GameModel, index: i32) -> QMap_QString_QVariant;

        #[qinvokable]
        fn cache_dir(self: &GameModel) -> QString;

        #[qinvokable]
        fn library_dir(self: &GameModel) -> QString;

        #[qinvokable]
        fn launch_game(self: &GameModel, index: i32) -> bool;

        #[qinvokable]
        fn launch_game_force(self: &GameModel, index: i32) -> bool;

        #[qinvokable]
        fn needs_prefix_prep(self: &GameModel, index: i32) -> bool;

        #[qinvokable]
        fn prepare_prefix(self: Pin<&mut GameModel>, index: i32);

        #[qinvokable]
        fn launch_exe(self: &GameModel, exe: &QString, runner: &QString, prefix: &QString) -> bool;

        #[qinvokable]
        fn run_exe_path(self: &GameModel) -> QString;

        #[qinvokable]
        fn quit_now(self: &GameModel);

        #[qinvokable]
        fn check_epic_update(self: &GameModel, game_id: &QString) -> bool;

        #[qinvokable]
        fn check_gog_update(self: &GameModel, game_id: &QString) -> bool;

        #[qinvokable]
        fn scan_all_for_updates(self: Pin<&mut GameModel>);

        #[qinvokable]
        fn is_running(self: &GameModel, index: i32) -> bool;

        #[qinvokable]
        fn logs_dir(self: &GameModel) -> QString;

        #[qinvokable]
        fn get_game_config(self: &GameModel, index: i32) -> QMap_QString_QVariant;

        #[qinvokable]
        fn update_game_field(self: Pin<&mut GameModel>, index: i32, key: &QString, value: &QString) -> bool;

        // id is safer than index here; index can shift during a concurrent refresh
        #[qinvokable]
        fn save_game(self: Pin<&mut GameModel>, game_id: &QString) -> bool;

        #[qinvokable]
        fn refetch_media(self: Pin<&mut GameModel>, game_id: &QString);

        #[qinvokable]
        #[cxx_name = "applyDefaultsToExistingGames"]
        fn apply_defaults_to_existing_games(
            self: Pin<&mut GameModel>,
            sections_csv: &QString,
            replace_maps: bool,
        ) -> i32;

        #[qinvokable]
        fn list_runners(self: &GameModel) -> QString;

        #[qinvokable]
        fn list_gpus(self: &GameModel) -> QString;

        #[qinvokable]
        fn system_info(self: &GameModel) -> QString;

        #[qinvokable]
        fn app_version(self: &GameModel) -> QString;

        #[qinvokable]
        #[cxx_name = "cpuCoreCount"]
        fn cpu_core_count(self: &GameModel) -> i32;

        #[qinvokable]
        fn stop_game(self: &GameModel, game_id: &QString);

        #[qinvokable]
        fn run_wine_tool(self: &GameModel, game_id: &QString, tool: &QString);

        #[qinvokable]
        fn run_wine_command(self: Pin<&mut GameModel>, game_id: &QString, command: &QString);

        #[qinvokable]
        fn run_wine_exe(self: &GameModel, game_id: &QString, exe_path: &QString);

        #[qinvokable]
        fn check_exited_games(self: Pin<&mut GameModel>);

        #[qinvokable]
        fn drain_game_log_events(self: Pin<&mut GameModel>);

        #[qinvokable]
        fn game_log(self: &GameModel, game_id: &QString) -> QString;

        #[qinvokable]
        fn clear_game_log(self: &GameModel, game_id: &QString);

        #[qinvokable]
        fn save_game_log(self: &GameModel, game_id: &QString) -> QString;

        #[qinvokable]
        fn drain_notifications(self: Pin<&mut GameModel>);

        #[qinvokable]
        fn drain_update_notifications(self: Pin<&mut GameModel>);

        #[qinvokable]
        fn drain_errors(self: Pin<&mut GameModel>);

        #[qinvokable]
        fn index_of_id(self: &GameModel, game_id: &QString) -> i32;

        #[qinvokable]
        fn enqueue_game_update(
            self: Pin<&mut GameModel>,
            game_id: &QString,
            from_version: &QString,
        ) -> QString;

        #[qinvokable]
        fn enqueue_game_repair(self: Pin<&mut GameModel>, game_id: &QString) -> QString;

        #[qinvokable]
        fn game_supports_repair(self: &GameModel, game_id: &QString) -> bool;

        #[qinvokable]
        fn browse_files(self: &GameModel, index: i32) -> bool;

        #[qinvokable]
        fn create_desktop_shortcut(self: &GameModel, index: i32) -> bool;

        #[qinvokable]
        fn create_menu_shortcut(self: &GameModel, index: i32) -> bool;

        #[qinvokable]
        fn remove_desktop_shortcut(self: &GameModel, index: i32) -> bool;

        #[qinvokable]
        fn remove_menu_shortcut(self: &GameModel, index: i32) -> bool;

        #[qinvokable]
        fn has_desktop_shortcut(self: &GameModel, index: i32) -> bool;

        #[qinvokable]
        fn has_menu_shortcut(self: &GameModel, index: i32) -> bool;

        #[qinvokable]
        fn create_steam_shortcut(self: &GameModel, index: i32) -> bool;

        #[qinvokable]
        fn remove_steam_shortcut(self: &GameModel, index: i32) -> bool;

        #[qinvokable]
        fn has_steam_shortcut(self: &GameModel, index: i32) -> bool;

        #[qinvokable]
        fn steam_shortcut_available(self: &GameModel, index: i32) -> bool;

        #[qinvokable]
        fn duplicate_game(self: Pin<&mut GameModel>, index: i32) -> bool;

        #[qinvokable]
        fn steam_get_installed_games(self: &GameModel) -> QString;

        #[qinvokable]
        fn steam_import_game(self: Pin<&mut GameModel>, appid: &QString, name: &QString) -> bool;

        #[qinvokable]
        fn steam_local_library_image(self: &GameModel, appid: &QString) -> QString;

        #[qinvokable]
        fn is_flatpak(self: &GameModel) -> bool;

        // blocking http inside the tokio runtime panics; we escape to an os thread first
        #[qinvokable]
        fn steam_sync_playtime(self: Pin<&mut GameModel>);

        // result arrives async via file_dialog_result signal, not as a return value
        #[qinvokable]
        fn open_file_dialog(self: Pin<&mut GameModel>, request_id: &QString, select_folder: bool, title: &QString, default_path: &QString, filter: &QString);

        #[qinvokable]
        fn disk_free_space(self: &GameModel, path: &QString) -> QString;

        #[qinvokable]
        fn epic_check_existing_install(
            self: &GameModel,
            app_name: &QString,
            install_path: &QString,
        ) -> QString;

        #[qinvokable]
        fn fetch_epic_install_size(
            self: Pin<&mut GameModel>,
            request_id: &QString,
            app_name: &QString,
        );

        #[qinvokable]
        fn drain_install_sizes(self: Pin<&mut GameModel>);

        #[qinvokable]
        fn drain_file_dialog_results(self: Pin<&mut GameModel>);

        // calls glibc malloc_trim to return freed heap to the os
        #[qinvokable]
        fn trim_heap(self: &GameModel);

        #[qinvokable]
        fn launch_console_mode(self: &GameModel);

        #[qinvokable]
        fn launch_desktop_mode(self: &GameModel);


        #[qinvokable]
        fn home_dir(self: &GameModel) -> QString;

        #[qinvokable]
        fn register_game_json(self: Pin<&mut GameModel>, game_json: &QString) -> QString;

        #[qinvokable]
        fn epic_import_after_install(
            self: Pin<&mut GameModel>,
            app_name: &QString,
            display_name: &QString,
            prefix_path: &QString,
            runner_version: &QString,
        ) -> QString;

        #[qinvokable]
        fn gog_check_existing_install(
            self: &GameModel,
            app_name: &QString,
            install_path: &QString,
        ) -> QString;

        #[qinvokable]
        fn fetch_gog_install_size(
            self: Pin<&mut GameModel>,
            request_id: &QString,
            app_name: &QString,
        );

        #[qinvokable]
        fn gog_import_after_install(
            self: Pin<&mut GameModel>,
            app_name: &QString,
            display_name: &QString,
            prefix_path: &QString,
            runner_version: &QString,
        ) -> QString;

        #[qinvokable]
        fn gog_uninstall(self: Pin<&mut GameModel>, game_id: &QString) -> bool;

        #[qinvokable]
        fn list_gachas(self: &GameModel) -> QString;

        #[qinvokable]
        #[cxx_name = "ensureGachaManifests"]
        fn ensure_gacha_manifests(self: Pin<&mut GameModel>);

        #[qinvokable]
        fn get_gacha_manifest(self: &GameModel, manifest_id: &QString) -> QString;

        #[qinvokable]
        fn gacha_manifest_for_app_id(self: &GameModel, app_id: &QString) -> QString;

        #[qinvokable]
        fn gacha_posters(self: &GameModel) -> QString;

        #[qinvokable]
        fn fetch_gacha_install_size(
            self: Pin<&mut GameModel>,
            request_id: &QString,
            manifest_id: &QString,
            edition_id: &QString,
            voices_csv: &QString,
        );

        #[qinvokable]
        fn gacha_check_existing_install(
            self: &GameModel,
            manifest_id: &QString,
            edition_id: &QString,
            install_path: &QString,
            temp_path: &QString,
        ) -> QString;

        #[qinvokable]
        fn gacha_import_after_install(
            self: Pin<&mut GameModel>,
            manifest_id: &QString,
            edition_id: &QString,
            display_name: &QString,
            install_path: &QString,
            runner_version: &QString,
            prefix_path: &QString,
        ) -> QString;

        #[qinvokable]
        fn epic_toggle_overlay(
            self: Pin<&mut GameModel>,
            game_id: &QString,
            enable: bool,
        ) -> bool;

        #[qinvokable]
        fn epic_uninstall(self: Pin<&mut GameModel>, game_id: &QString) -> bool;

        #[qinvokable]
        fn epic_overlay_is_installed(self: &GameModel) -> bool;

        #[qinvokable]
        fn epic_set_cloud_saves(
            self: Pin<&mut GameModel>,
            game_id: &QString,
            enable: bool,
        ) -> bool;
    }

    unsafe extern "RustQt" {
        #[cxx_name = "beginInsertRows"]
        #[inherit]
        fn begin_insert_rows(
            self: Pin<&mut GameModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[cxx_name = "endInsertRows"]
        #[inherit]
        fn end_insert_rows(self: Pin<&mut GameModel>);

        #[cxx_name = "beginRemoveRows"]
        #[inherit]
        fn begin_remove_rows(
            self: Pin<&mut GameModel>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[cxx_name = "endRemoveRows"]
        #[inherit]
        fn end_remove_rows(self: Pin<&mut GameModel>);

        #[cxx_name = "beginMoveRows"]
        #[inherit]
        fn begin_move_rows(
            self: Pin<&mut GameModel>,
            source_parent: &QModelIndex,
            source_first: i32,
            source_last: i32,
            destination_parent: &QModelIndex,
            destination_child: i32,
        ) -> bool;

        #[cxx_name = "endMoveRows"]
        #[inherit]
        fn end_move_rows(self: Pin<&mut GameModel>);

        #[cxx_name = "beginResetModel"]
        #[inherit]
        fn begin_reset_model(self: Pin<&mut GameModel>);

        #[cxx_name = "endResetModel"]
        #[inherit]
        fn end_reset_model(self: Pin<&mut GameModel>);

        #[cxx_name = "index"]
        #[inherit]
        fn model_index(
            self: &GameModel,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;

        #[cxx_name = "dataChanged"]
        #[inherit]
        fn data_changed(
            self: Pin<&mut GameModel>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QList_i32,
        );
    }

    impl cxx_qt::Threading for GameModel {}
}

use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::PathBuf;
use std::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QMap, QMapPair_QString_QVariant, QString, QVariant};

use omikuji_core::library::{rfc3339_now, Game, Library};
use omikuji_core::media::{self, MediaType};
use omikuji_core::ui_settings::UiSettings;

const ROLE_ID: i32 = 0x0100;
const ROLE_NAME: i32 = 0x0101;
const ROLE_BANNER: i32 = 0x0102;
const ROLE_COLOR: i32 = 0x0103;
const ROLE_PLAYTIME: i32 = 0x0104;
const ROLE_LAST_PLAYED: i32 = 0x0105;
const ROLE_RUNNER: i32 = 0x0106;
const ROLE_EXE: i32 = 0x0107;
const ROLE_COVERART: i32 = 0x0108;
const ROLE_ICON: i32 = 0x0109;
const ROLE_FAVOURITE: i32 = 0x010A;
const ROLE_CATEGORIES: i32 = 0x010B;
const ROLE_RUNNER_TYPE: i32 = 0x010C;
const ROLE_HIDDEN: i32 = 0x010D;

#[derive(Clone, Copy, PartialEq, Default)]
enum SortMode {
    #[default]
    Added,
    NameAsc,
    NameDesc,
    Custom,
}

impl SortMode {
    fn parse(v: &str) -> Self {
        match v {
            "a-z" => Self::NameAsc,
            "z-a" => Self::NameDesc,
            "custom" => Self::Custom,
            _ => Self::Added,
        }
    }

    fn cmp(self, a: &Game, b: &Game) -> Ordering {
        let added = a.added_key().cmp(&b.added_key());
        match self {
            Self::Added => added,
            Self::NameAsc => a.display_sort_key().cmp(&b.display_sort_key()).then(added),
            Self::NameDesc => b.display_sort_key().cmp(&a.display_sort_key()).then(added),
            Self::Custom => a.custom_key().cmp(&b.custom_key()),
        }
    }
}

pub struct GameModelRust {
    library: Library,
    count: i32,
    // in-memory staging slot for the add-game page. cleared on commit/discard.
    draft: Option<Game>,
    preparing: bool,
    wine_command_running: bool,
    sort_mode: SortMode,
    dirty_order: HashSet<String>,
}

impl Default for GameModelRust {
    fn default() -> Self {
        let mut library = Library::load().unwrap_or_default();
        let sort_mode = SortMode::parse(&UiSettings::load().display.card_sort);
        library.game.sort_by(|a, b| sort_mode.cmp(a, b));
        let count = library.game.len() as i32;
        Self { library, count, draft: None, preparing: false, wine_command_running: false, sort_mode, dirty_order: Default::default() }
    }
}

fn runner_display(game: &Game) -> String {
    match game.runner.runner_type.as_str() {
        "steam" if !game.source.app_id.is_empty() => format!("steam:{}", game.source.app_id),
        "flatpak" if !game.source.app_id.is_empty() => format!("flatpak:{}", game.source.app_id),
        "native" => "Native".to_string(),
        _ => game.wine.version.clone(),
    }
}

fn args_to_text(args: &[String]) -> String {
    args.iter()
        .map(|a| {
            if a.is_empty() {
                "\"\"".to_string()
            } else if a.chars().any(|c| c.is_whitespace() || matches!(c, '"' | '\'' | '\\')) {
                let escaped = a.replace('\\', "\\\\").replace('"', "\\\"");
                format!("\"{}\"", escaped)
            } else {
                a.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn args_from_text(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;
    let mut started = false;
    for c in s.chars() {
        if escape {
            cur.push(c);
            escape = false;
            started = true;
            continue;
        }
        if c == '\\' && !in_single {
            escape = true;
            continue;
        }
        if c == '\'' && !in_double {
            in_single = !in_single;
            started = true;
            continue;
        }
        if c == '"' && !in_single {
            in_double = !in_double;
            started = true;
            continue;
        }
        if c.is_whitespace() && !in_single && !in_double {
            if started {
                out.push(std::mem::take(&mut cur));
                started = false;
            }
            continue;
        }
        cur.push(c);
        started = true;
    }
    if started {
        out.push(cur);
    }
    out
}

macro_rules! field_get {
    (str, $m:ident, $key:literal, $v:expr) => {
        $m.insert(QString::from($key), QVariant::from(&QString::from(&*$v)));
    };
    (path, $m:ident, $key:literal, $v:expr) => {
        $m.insert(QString::from($key), QVariant::from(&QString::from(&*$v.to_string_lossy())));
    };
    (bool, $m:ident, $key:literal, $v:expr) => {
        $m.insert(QString::from($key), QVariant::from(&$v));
    };
    (int, $m:ident, $key:literal, $v:expr) => {
        $m.insert(QString::from($key), QVariant::from(&($v as i32)));
    };
    (json, $m:ident, $key:literal, $v:expr) => {
        if let Ok(json) = serde_json::to_string(&$v) {
            $m.insert(QString::from($key), QVariant::from(&QString::from(&*json)));
        }
    };
    (args, $m:ident, $key:literal, $v:expr) => {
        $m.insert(QString::from($key), QVariant::from(&QString::from(&*args_to_text(&$v))));
    };
}

macro_rules! field_set {
    ($kind:ident readonly, $game:ident, $key:ident, $value:ident, $lit:literal, $($path:ident).+) => {};
    (str, $game:ident, $key:ident, $value:ident, $lit:literal, $($path:ident).+) => {
        if $key == $lit {
            $game.$($path).+ = $value.to_string();
            return true;
        }
    };
    (path, $game:ident, $key:ident, $value:ident, $lit:literal, $($path:ident).+) => {
        if $key == $lit {
            $game.$($path).+ = PathBuf::from($value);
            return true;
        }
    };
    (bool, $game:ident, $key:ident, $value:ident, $lit:literal, $($path:ident).+) => {
        if $key == $lit {
            $game.$($path).+ = $value == "true";
            return true;
        }
    };
    (int, $game:ident, $key:ident, $value:ident, $lit:literal, $($path:ident).+) => {
        if $key == $lit {
            $game.$($path).+ = $value.parse().unwrap_or(0);
            return true;
        }
    };
    (json, $game:ident, $key:ident, $value:ident, $lit:literal, $($path:ident).+) => {
        if $key == $lit {
            if let Ok(parsed) = serde_json::from_str($value) {
                $game.$($path).+ = parsed;
            }
            return true;
        }
    };
    (args, $game:ident, $key:ident, $value:ident, $lit:literal, $($path:ident).+) => {
        if $key == $lit {
            $game.$($path).+ = args_from_text($value);
            return true;
        }
    };
}

macro_rules! game_fields {
    ($( $key:literal => $kind:ident $($flag:ident)?, $($path:ident).+ ),* $(,)?) => {
        fn populate_config_map(game: &Game, m: &mut QMap<QMapPair_QString_QVariant>) {
            $( field_get!($kind, m, $key, game.$($path).+); )*
        }

        fn apply_field_to_game(game: &mut Game, key: &str, value: &str) -> bool {
            $( field_set!($kind $($flag)?, game, key, value, $key, $($path).+); )*
            tracing::warn!("unknown or read-only config key: {}", key);
            false
        }
    };
}

game_fields! {
    "meta.id" => str readonly, metadata.id,
    "meta.name" => str, metadata.name,
    "meta.sort_name" => str, metadata.sort_name,
    "meta.slug" => str, metadata.slug,
    "meta.exe" => path, metadata.exe,
    "meta.color" => str, metadata.color,
    "meta.banner" => str, metadata.banner,
    "meta.coverart" => str, metadata.coverart,
    "meta.icon" => str, metadata.icon,
    "meta.favourite" => bool, metadata.favourite,
    "meta.hidden" => bool, metadata.hidden,
    "meta.categories" => json, metadata.categories,

    "source.kind" => str readonly, source.kind,
    "source.app_id" => str, source.app_id,
    "source.eos_overlay" => bool readonly, source.eos_overlay,
    "source.cloud_saves" => bool readonly, source.cloud_saves,
    "source.save_path" => str, source.save_path,
    "source.patch" => str readonly, source.patch,

    "runner.type" => str, runner.runner_type,

    "wine.version" => str, wine.version,
    "wine.prefix" => str, wine.prefix,
    "wine.prefix_arch" => str, wine.prefix_arch,
    "wine.esync" => bool, wine.esync,
    "wine.fsync" => bool, wine.fsync,
    "wine.ntsync" => bool, wine.ntsync,
    "wine.dxvk" => bool, wine.dxvk,
    "wine.dxvk_version" => str, wine.dxvk_version,
    "wine.vkd3d" => bool, wine.vkd3d,
    "wine.vkd3d_version" => str, wine.vkd3d_version,
    "wine.d3d_extras" => bool, wine.d3d_extras,
    "wine.d3d_extras_version" => str, wine.d3d_extras_version,
    "wine.dxvk_nvapi" => bool, wine.dxvk_nvapi,
    "wine.dxvk_nvapi_version" => str, wine.dxvk_nvapi_version,
    "wine.fsr" => bool, wine.fsr,
    "wine.battleye" => bool, wine.battleye,
    "wine.easyanticheat" => bool, wine.easyanticheat,
    "wine.dpi_scaling" => bool, wine.dpi_scaling,
    "wine.dpi" => int, wine.dpi,
    "wine.audio_driver" => str, wine.audio_driver,
    "wine.graphics_driver" => str, wine.graphics_driver,
    "wine.dll_overrides" => json, wine.dll_overrides,
    "wine.dll_override_sets" => json, wine.dll_override_sets,

    "launch.args" => args, launch.args,
    "launch.working_dir" => str, launch.working_dir,
    "launch.command_prefix" => str, launch.command_prefix,
    "launch.pre_launch_script" => str, launch.pre_launch_script,
    "launch.post_exit_script" => str, launch.post_exit_script,
    "launch.env" => json, launch.env,
    "launch.env_sets" => json, launch.env_sets,

    "graphics.mangohud" => bool, graphics.mangohud,
    "graphics.gpu" => str, graphics.gpu,

    "graphics.gamescope.enabled" => bool, graphics.gamescope.enabled,
    "graphics.gamescope.width" => int, graphics.gamescope.width,
    "graphics.gamescope.height" => int, graphics.gamescope.height,
    "graphics.gamescope.game_width" => int, graphics.gamescope.game_width,
    "graphics.gamescope.game_height" => int, graphics.gamescope.game_height,
    "graphics.gamescope.fps" => int, graphics.gamescope.fps,
    "graphics.gamescope.refresh_rate" => int, graphics.gamescope.refresh_rate,
    "graphics.gamescope.fullscreen" => bool, graphics.gamescope.fullscreen,
    "graphics.gamescope.borderless" => bool, graphics.gamescope.borderless,
    "graphics.gamescope.integer_scaling" => bool, graphics.gamescope.integer_scaling,
    "graphics.gamescope.hdr" => bool, graphics.gamescope.hdr,
    "graphics.gamescope.filter" => str, graphics.gamescope.filter,
    "graphics.gamescope.fsr_sharpness" => int, graphics.gamescope.fsr_sharpness,

    "system.gamemode" => bool, system.gamemode,
    "system.prevent_sleep" => bool, system.prevent_sleep,
    "system.pulse_latency" => bool, system.pulse_latency,
    "system.cpu_limit" => int, system.cpu_limit,
}

fn config_map(game: &Game) -> QMap<QMapPair_QString_QVariant> {
    let mut m = QMap::<QMapPair_QString_QVariant>::default();
    populate_config_map(game, &mut m);
    if !game.metadata.id.is_empty() {
        let resolved = omikuji_core::launch::prefix_path_for(game);
        m.insert(
            QString::from("wine.prefix.resolved"),
            QVariant::from(&QString::from(&*resolved.to_string_lossy())),
        );
    }
    m
}

fn media_changed_notifier(
    qt_thread: cxx_qt::CxxQtThread<qobject::GameModel>,
    game_id: String,
) -> impl FnMut(&media::MediaType) {
    move |_| {
        let id_inner = game_id.clone();
        let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::GameModel>| {
            let Some(row) = obj.library.game.iter().position(|g| g.metadata.id == id_inner) else {
                return;
            };
            let idx = obj.as_ref().model_index(row as i32, 0, &QModelIndex::default());
            let roles = cxx_qt_lib::QList::<i32>::default();
            obj.as_mut().data_changed(&idx, &idx, &roles);
        });
    }
}

impl qobject::GameModel {
    fn row_count(&self, _parent: &QModelIndex) -> i32 {
        self.library.game.len() as i32
    }

    fn data(&self, index: &QModelIndex, role: i32) -> QVariant {
        let row = index.row() as usize;
        let Some(game) = self.library.game.get(row) else {
            return QVariant::default();
        };

        // debug: log first data() call per game
        if role == ROLE_NAME {
            tracing::debug!("row={} name='{}' coverart='{}'",
                row, game.metadata.name,
                media::resolve_image(&game.metadata.id, &game.metadata.coverart, &MediaType::Coverart));
        }

        match role {
            ROLE_ID => QVariant::from(&QString::from(&*game.metadata.id)),
            ROLE_NAME => QVariant::from(&QString::from(&*game.metadata.name)),
            ROLE_COLOR => QVariant::from(&QString::from(&*game.metadata.color)),
            ROLE_PLAYTIME => QVariant::from(&game.metadata.playtime),
            ROLE_LAST_PLAYED => QVariant::from(&QString::from(&*game.metadata.last_played)),
            ROLE_RUNNER => QVariant::from(&QString::from(&*runner_display(game))),
            ROLE_EXE => QVariant::from(&QString::from(&*game.metadata.exe.to_string_lossy())),

            // resolve order: manual override > cached file > empty string
            ROLE_BANNER => {
                let path = media::resolve_image(&game.metadata.id, &game.metadata.banner, &MediaType::Banner);
                QVariant::from(&QString::from(&*path))
            }
            ROLE_COVERART => {
                let path = media::resolve_image(&game.metadata.id, &game.metadata.coverart, &MediaType::Coverart);
                QVariant::from(&QString::from(&*path))
            }
            ROLE_ICON => {
                let path = media::resolve_image(&game.metadata.id, &game.metadata.icon, &MediaType::Icon);
                QVariant::from(&QString::from(&*path))
            }
            ROLE_FAVOURITE => QVariant::from(&game.metadata.favourite),
            ROLE_HIDDEN => QVariant::from(&game.metadata.hidden),
            ROLE_RUNNER_TYPE => QVariant::from(&QString::from(&*game.runner.runner_type)),
            ROLE_CATEGORIES => {
                match serde_json::to_string(&game.metadata.categories) {
                    Ok(json) => QVariant::from(&QString::from(&json)),
                    Err(_) => QVariant::from(&QString::from("[]")),
                }
            }
            _ => QVariant::default(),
        }
    }

    fn role_names(&self) -> QHash<QHashPair_i32_QByteArray> {
        let mut hash = QHash::<QHashPair_i32_QByteArray>::default();
        hash.insert_clone(&ROLE_ID, &QByteArray::from("gameId"));
        hash.insert_clone(&ROLE_NAME, &QByteArray::from("name"));
        hash.insert_clone(&ROLE_BANNER, &QByteArray::from("banner"));
        hash.insert_clone(&ROLE_COLOR, &QByteArray::from("color"));
        hash.insert_clone(&ROLE_PLAYTIME, &QByteArray::from("playtime"));
        hash.insert_clone(&ROLE_LAST_PLAYED, &QByteArray::from("lastPlayed"));
        hash.insert_clone(&ROLE_RUNNER, &QByteArray::from("runner"));
        hash.insert_clone(&ROLE_EXE, &QByteArray::from("exe"));
        hash.insert_clone(&ROLE_COVERART, &QByteArray::from("coverart"));
        hash.insert_clone(&ROLE_ICON, &QByteArray::from("icon"));
        hash.insert_clone(&ROLE_FAVOURITE, &QByteArray::from("favourite"));
        hash.insert_clone(&ROLE_HIDDEN, &QByteArray::from("hidden"));
        hash.insert_clone(&ROLE_CATEGORIES, &QByteArray::from("categories"));
        hash.insert_clone(&ROLE_RUNNER_TYPE, &QByteArray::from("runnerType"));
        hash
    }

    fn begin_new_game(mut self: Pin<&mut Self>) -> QMap<QMapPair_QString_QVariant> {
        let mut game = Game::new(String::new(), PathBuf::new());
        game.seed_from_defaults(&omikuji_core::defaults::Defaults::load());
        let m = config_map(&game);
        self.as_mut().rust_mut().get_mut().draft = Some(game);
        m
    }

    fn get_draft_config(&self) -> QMap<QMapPair_QString_QVariant> {
        match &self.rust().draft {
            Some(game) => config_map(game),
            None => QMap::<QMapPair_QString_QVariant>::default(),
        }
    }

    fn update_draft_field(mut self: Pin<&mut Self>, key: &QString, value: &QString) -> bool {
        let k = key.to_string();
        let v = value.to_string();
        let Some(game) = self.as_mut().rust_mut().get_mut().draft.as_mut() else {
            return false;
        };
        apply_field_to_game(game, &k, &v)
    }

    pub(crate) fn insert_game_sorted(mut self: Pin<&mut Self>, game: Game) -> i32 {
        let mode = self.sort_mode;
        let row = self
            .library
            .game
            .partition_point(|g| mode.cmp(g, &game) != Ordering::Greater) as i32;
        self.as_mut().begin_insert_rows(&QModelIndex::default(), row, row);
        self.as_mut().rust_mut().get_mut().library.game.insert(row as usize, game);
        let count = self.library.game.len() as i32;
        self.as_mut().set_count(count);
        self.as_mut().end_insert_rows();
        row
    }

    fn resort_reset(mut self: Pin<&mut Self>) {
        let mode = self.sort_mode;
        if self
            .library
            .game
            .is_sorted_by(|a, b| mode.cmp(a, b) != Ordering::Greater)
        {
            return;
        }
        self.as_mut().begin_reset_model();
        self.as_mut().rust_mut().get_mut().library.game.sort_by(|a, b| mode.cmp(a, b));
        self.as_mut().end_reset_model();
    }

    fn apply_sort_mode(mut self: Pin<&mut Self>, value: &QString) {
        let mode = SortMode::parse(&value.to_string());
        if mode == self.sort_mode {
            return;
        }
        if mode == SortMode::Custom {
            self.as_mut().materialize_custom_order();
            self.as_mut().commit_order();
        }
        self.as_mut().rust_mut().get_mut().sort_mode = mode;
        self.resort_reset();
    }

    fn materialize_custom_order(mut self: Pin<&mut Self>) {
        let rust = self.as_mut().rust_mut().get_mut();
        let mut next = rust
            .library
            .game
            .iter()
            .filter_map(|g| g.metadata.custom_pos)
            .max()
            .map_or(0, |m| m + 1);
        for game in rust.library.game.iter_mut() {
            if game.metadata.custom_pos.is_none() {
                game.metadata.custom_pos = Some(next);
                next += 1;
                rust.dirty_order.insert(game.metadata.id.clone());
            }
        }
    }

    fn move_game(mut self: Pin<&mut Self>, from: i32, to: i32) {
        let count = self.library.game.len() as i32;
        if self.sort_mode != SortMode::Custom
            || from == to
            || !(0..count).contains(&from)
            || !(0..count).contains(&to)
        {
            return;
        }
        let dest = if to > from { to + 1 } else { to };
        if !self.as_mut().begin_move_rows(
            &QModelIndex::default(),
            from,
            from,
            &QModelIndex::default(),
            dest,
        ) {
            return;
        }
        let rust = self.as_mut().rust_mut().get_mut();
        let game = rust.library.game.remove(from as usize);
        rust.library.game.insert(to as usize, game);
        for (i, g) in rust.library.game.iter_mut().enumerate() {
            if g.metadata.custom_pos != Some(i as u32) {
                g.metadata.custom_pos = Some(i as u32);
                rust.dirty_order.insert(g.metadata.id.clone());
            }
        }
        self.as_mut().end_move_rows();
    }

    fn commit_order(mut self: Pin<&mut Self>) {
        let rust = self.as_mut().rust_mut().get_mut();
        let dirty = std::mem::take(&mut rust.dirty_order);
        for game in rust.library.game.iter().filter(|g| dirty.contains(g.id())) {
            if let Err(e) = Library::save_game_static(game) {
                tracing::warn!("save custom order for {}: {}", game.id(), e);
            }
        }
    }

    // on failure, draft is preserved so the user can fix fields and retry (a bit useless most of the times but may it be a connection error)
    fn commit_new_game(mut self: Pin<&mut Self>) -> QString {
        let Some(mut game) = self.as_mut().rust_mut().get_mut().draft.take() else {
            tracing::warn!("commit_new_game: no draft");
            return QString::default();
        };

        // exe is allowed empty for non-wine runners (steam, flatpak, etc)
        if game.metadata.name.trim().is_empty() {
            tracing::warn!("commit_new_game: name is required");
            self.as_mut().rust_mut().get_mut().draft = Some(game);
            return QString::default();
        }

        game.metadata.name = game.metadata.name.trim().to_string();
        game.metadata.added = rfc3339_now();

        let game_id = game.metadata.id.clone();
        let game_name = game.metadata.name.clone();

        if let Err(e) = Library::save_game_static(&game) {
            tracing::error!("commit_new_game: failed to save: {}", e);
            self.as_mut().rust_mut().get_mut().draft = Some(game);
            return QString::default();
        }

        self.as_mut().insert_game_sorted(game);

        let new_id = QString::from(&*game_id);
        let qt_thread = self.as_mut().qt_thread();
        let on_asset = media_changed_notifier(qt_thread, game_id.clone());
        std::thread::spawn(move || {
            let result = media::fetch_media_blocking_with(&game_id, &game_name, on_asset);
            let fetched: Vec<&str> = [
                result.banner.as_ref().map(|_| "banner"),
                result.coverart.as_ref().map(|_| "coverart"),
                result.icon.as_ref().map(|_| "icon"),
            ]
            .into_iter()
            .flatten()
            .collect();
            if fetched.is_empty() {
                tracing::warn!("no media found for '{}'", game_name);
            } else {
                tracing::info!("fetched {} for '{}'", fetched.join(", "), game_name);
            }
        });

        new_id
    }

    fn discard_draft(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().get_mut().draft = None;
    }

    fn begin_edit_game(mut self: Pin<&mut Self>, index: i32) -> QMap<QMapPair_QString_QVariant> {
        let idx = index as usize;
        let cloned = self.library.game.get(idx).cloned();
        let m = match &cloned {
            Some(game) => config_map(game),
            None => QMap::<QMapPair_QString_QVariant>::default(),
        };
        self.as_mut().rust_mut().get_mut().draft = cloned;
        m
    }

    fn commit_edit_game(mut self: Pin<&mut Self>, game_id: &QString) -> bool {
        let id = game_id.to_string();
        let Some(draft) = self.as_mut().rust_mut().get_mut().draft.take() else {
            tracing::warn!("commit_edit_game: no draft");
            return false;
        };
        let Some(idx) = self.library.game.iter().position(|g| g.metadata.id == id) else {
            tracing::warn!("commit_edit_game: game id '{}' not found", id);
            self.as_mut().rust_mut().get_mut().draft = Some(draft);
            return false;
        };
        if let Err(e) = Library::save_game_static(&draft) {
            tracing::error!("commit_edit_game: failed to save: {}", e);
            self.as_mut().rust_mut().get_mut().draft = Some(draft);
            return false;
        }
        self.as_mut().rust_mut().get_mut().library.game[idx] = draft;
        let model_idx = self.as_ref().model_index(idx as i32, 0, &QModelIndex::default());
        let roles = cxx_qt_lib::QList::<i32>::default();
        self.as_mut().data_changed(&model_idx, &model_idx, &roles);
        self.as_mut().resort_reset();
        true
    }

    fn remove_game(mut self: Pin<&mut Self>, index: i32) {
        let idx = index as usize;
        if idx >= self.library.game.len() {
            return;
        }

        let game_id = self.library.game[idx].metadata.id.clone();

        if let Err(e) = omikuji_core::library::Library::remove_game_file(&game_id) {
            tracing::error!("failed to remove game file: {}", e);
            omikuji_core::process::notify_error(omikuji_core::process::ErrorNotification {
                game_id,
                title: "Remove failed".to_string(),
                message: format!("Couldn't delete the game's library file: {}", e),
                action: omikuji_core::process::ErrorAction::None,
            });
            return;
        }

        self.as_mut()
            .begin_remove_rows(&QModelIndex::default(), index, index);

        self.as_mut().rust_mut().get_mut().library.game.remove(idx);

        media::remove_cached_media(&game_id);

        let count = self.library.game.len() as i32;
        self.as_mut().set_count(count);
        self.end_remove_rows();
    }

    fn remove_game_with_prefix(mut self: Pin<&mut Self>, index: i32) {
        let idx = index as usize;
        let prefix = match self.library.game.get(idx) {
            Some(game) if game.uses_wine_prefix() => {
                Some(omikuji_core::launch::prefix_path_for(game))
            }
            _ => None,
        };
        if let Some(p) = prefix {
            omikuji_core::prefixes::delete_prefix(&p);
        }
        self.as_mut().remove_game(index);
    }

    fn game_prefix_info(&self, index: i32) -> QString {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            return QString::default();
        };
        let path = omikuji_core::launch::prefix_path_for(game);
        let has_prefix = game.uses_wine_prefix() && path.is_dir();
        let games: Vec<String> = if has_prefix {
            self.library
                .game
                .iter()
                .filter(|g| g.uses_wine_prefix() && omikuji_core::launch::prefix_path_for(g) == path)
                .map(|g| g.metadata.name.clone())
                .collect()
        } else {
            Vec::new()
        };
        let v = serde_json::json!({
            "hasPrefix": has_prefix,
            "path": path.to_string_lossy(),
            "gameCount": games.len(),
            "games": games,
        });
        QString::from(&v.to_string())
    }

    fn needs_prefix_prep(&self, index: i32) -> bool {
        self.library
            .game
            .get(index as usize)
            .map(omikuji_core::prefixes::prefix_needs_bootstrap)
            .unwrap_or(false)
    }

    fn prepare_prefix(mut self: Pin<&mut Self>, index: i32) {
        if self.preparing {
            return;
        }
        let Some(game) = self.library.game.get(index as usize).cloned() else {
            return;
        };
        self.as_mut().set_preparing(true);
        let qt = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            let line_qt = qt.clone();
            let res = omikuji_core::prefixes::bootstrap_prefix(&game, |line| {
                let l = line.to_string();
                let _ = line_qt.queue(move |mut obj: Pin<&mut qobject::GameModel>| {
                    obj.as_mut().prepare_output(&QString::from(&l));
                });
            });
            let (ok, err) = match res {
                Ok(_) => (true, String::new()),
                Err(e) => (false, e.to_string()),
            };
            let _ = qt.queue(move |mut obj: Pin<&mut qobject::GameModel>| {
                obj.as_mut().set_preparing(false);
                obj.as_mut().prepare_finished(ok, &QString::from(&err));
            });
        });
    }

    fn refresh(mut self: Pin<&mut Self>, selected_index: i32) -> QString {
        let selected_id = if selected_index >= 0 {
            self.library.game.get(selected_index as usize)
                .map(|g| g.metadata.id.clone())
                .unwrap_or_default()
        } else {
            String::new()
        };

        match Library::load() {
            Ok(mut new_lib) => {
                let mode = self.sort_mode;
                new_lib.game.sort_by(|a, b| mode.cmp(a, b));
                if new_lib.game == self.library.game {
                    return QString::from(&*selected_id);
                }
                let new_count = new_lib.game.len() as i32;
                self.as_mut().begin_reset_model();
                self.as_mut().rust_mut().get_mut().library = new_lib;
                self.as_mut().set_count(new_count);
                self.as_mut().end_reset_model();
                QString::from(&*selected_id)
            }
            Err(e) => {
                tracing::error!("failed to reload library: {}", e);
                QString::from(&*selected_id)
            }
        }
    }

    fn get_game(&self, index: i32) -> QMap<QMapPair_QString_QVariant> {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            return QMap::<QMapPair_QString_QVariant>::default();
        };

        let mut map = QMap::<QMapPair_QString_QVariant>::default();
        map.insert(
            QString::from("name"),
            QVariant::from(&QString::from(&*game.metadata.name)),
        );
        map.insert(
            QString::from("playtime"),
            QVariant::from(&game.metadata.playtime),
        );
        map.insert(
            QString::from("lastPlayed"),
            QVariant::from(&QString::from(&*game.metadata.last_played)),
        );
        map.insert(
            QString::from("runner"),
            QVariant::from(&QString::from(&*runner_display(game))),
        );
        map.insert(
            QString::from("runnerType"),
            QVariant::from(&QString::from(&*game.runner.runner_type)),
        );
        map.insert(
            QString::from("exe"),
            QVariant::from(&QString::from(&*game.metadata.exe.to_string_lossy())),
        );
        map.insert(
            QString::from("color"),
            QVariant::from(&QString::from(&*game.metadata.color)),
        );
        map.insert(
            QString::from("gameId"),
            QVariant::from(&QString::from(&*game.metadata.id)),
        );
        map.insert(
            QString::from("favourite"),
            QVariant::from(&game.metadata.favourite),
        );
        let prefix_path = if game.source.kind == "steam" && !game.source.app_id.is_empty() {
            omikuji_core::steam::local::find_steam_prefix(&game.source.app_id).unwrap_or_default()
        } else {
            omikuji_core::launch::prefix_path_for(game)
        };
        map.insert(
            QString::from("prefixPath"),
            QVariant::from(&QString::from(&*prefix_path.to_string_lossy())),
        );
        map.insert(
            QString::from("hidden"),
            QVariant::from(&game.metadata.hidden),
        );
        let cats_json = serde_json::to_string(&game.metadata.categories)
            .unwrap_or_else(|_| "[]".to_string());
        map.insert(
            QString::from("categories"),
            QVariant::from(&QString::from(&cats_json)),
        );

        let banner_path = media::resolve_image(&game.metadata.id, &game.metadata.banner, &MediaType::Banner);
        let coverart_path = media::resolve_image(&game.metadata.id, &game.metadata.coverart, &MediaType::Coverart);
        let icon_path = media::resolve_image(&game.metadata.id, &game.metadata.icon, &MediaType::Icon);

        map.insert(
            QString::from("banner"),
            QVariant::from(&QString::from(&*banner_path)),
        );
        map.insert(
            QString::from("coverart"),
            QVariant::from(&QString::from(&*coverart_path)),
        );
        map.insert(
            QString::from("icon"),
            QVariant::from(&QString::from(&*icon_path)),
        );

        // sourceKind and sourceAppId are needed by the context menu to branch behavior
        // (e.g., epic uninstall only shows for epic games)
        map.insert(
            QString::from("sourceKind"),
            QVariant::from(&QString::from(&*game.source.kind)),
        );
        map.insert(
            QString::from("sourceAppId"),
            QVariant::from(&QString::from(&*game.source.app_id)),
        );

        map
    }

    fn cache_dir(&self) -> QString {
        let path = omikuji_core::cache_dir().join("images");
        QString::from(&*path.to_string_lossy())
    }

    fn library_dir(&self) -> QString {
        let path = Library::library_dir();
        QString::from(&*path.to_string_lossy())
    }

    fn is_running(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            return false;
        };
        omikuji_core::process::is_game_running(&game.metadata.id)
    }

    fn logs_dir(&self) -> QString {
        let path = omikuji_core::logs_dir();
        QString::from(&*path.to_string_lossy())
    }

    fn get_game_config(&self, index: i32) -> QMap<QMapPair_QString_QVariant> {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            return QMap::<QMapPair_QString_QVariant>::default();
        };
        config_map(game)
    }

    fn update_game_field(mut self: Pin<&mut Self>, index: i32, key: &QString, value: &QString) -> bool {
        let idx = index as usize;
        let k = key.to_string();
        let v = value.to_string();
        let Some(game) = self.as_mut().rust_mut().get_mut().library.game.get_mut(idx) else {
            return false;
        };
        if !apply_field_to_game(game, &k, &v) {
            return false;
        }
        let model_idx = self.as_ref().model_index(index, 0, &QModelIndex::default());
        let roles = cxx_qt_lib::QList::<i32>::default();
        self.as_mut().data_changed(&model_idx, &model_idx, &roles);
        true
    }

    fn save_game(self: Pin<&mut Self>, game_id: &QString) -> bool {
        let id = game_id.to_string();

        let game = match self.library.game.iter().find(|g| g.metadata.id == id) {
            Some(g) => g.clone(),
            None => {
                tracing::warn!("save_game: game with id '{}' not found", id);
                return false;
            }
        };

        tracing::debug!("saving game '{}' id '{}'", game.metadata.name, game.metadata.id);

        if let Err(e) = Library::save_game_static(&game) {
            tracing::error!("failed to save game config: {}", e);
            return false;
        }
        true
    }

    fn refetch_media(mut self: Pin<&mut Self>, game_id: &QString) {
        let id = game_id.to_string();
        let Some(game) = self.library.game.iter().find(|g| g.metadata.id == id) else {
            tracing::warn!("refetch_media: game id '{}' not found", id);
            return;
        };
        let name = game.metadata.name.clone();
        let gacha_manifest = if game.source.kind == "gacha" {
            omikuji_core::gachas::strategies::find_for_app_id(&game.source.app_id)
                .map(|(m, _, _)| m)
        } else {
            None
        };
        let steam_appid = (game.source.kind == "steam").then(|| game.source.app_id.clone());

        let qt_thread = self.as_mut().qt_thread();
        let on_asset = media_changed_notifier(qt_thread, id.clone());
        std::thread::spawn(move || {
            match (gacha_manifest, steam_appid) {
                (Some(m), _) => omikuji_core::gachas::art::fetch_into_library_cache(&m, &id, on_asset),
                (_, Some(appid)) => { let _ = media::fetch_steam_media_blocking_with(&appid, on_asset); }
                _ => { let _ = media::fetch_media_blocking_with(&id, &name, on_asset); }
            }
        });
    }

    fn apply_defaults_to_existing_games(
        mut self: Pin<&mut Self>,
        sections_csv: &QString,
        replace_maps: bool,
    ) -> i32 {
        let sections: Vec<String> = sections_csv
            .to_string()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if sections.is_empty() {
            return 0;
        }

        let defaults = omikuji_core::defaults::Defaults::load();
        let mut written = 0i32;

        self.as_mut().begin_reset_model();
        let library = &mut self.as_mut().rust_mut().get_mut().library;
        for game in library.game.iter_mut() {
            defaults.apply_sections_to(game, &sections, replace_maps);
            match Library::save_game_static(game) {
                Ok(_) => written += 1,
                Err(e) => tracing::error!(
                    "apply_defaults save failed for {}: {}",
                    game.metadata.id, e
                ),
            }
        }
        self.as_mut().end_reset_model();
        written
    }

    fn list_runners(&self) -> QString {
        let runners = omikuji_core::runners::list_installed_runners();
        match serde_json::to_string(&runners) {
            Ok(json) => QString::from(&json),
            Err(_) => QString::from("[]"),
        }
    }

    fn list_gpus(&self) -> QString {
        let gpus = omikuji_core::runners::list_gpus();
        match serde_json::to_string(&gpus) {
            Ok(json) => QString::from(&json),
            Err(_) => QString::from("[[\"Default\",\"\"]]"),
        }
    }

    fn system_info(&self) -> QString {
        let qt = option_env!("OMIKUJI_QT_VERSION").unwrap_or("unknown");
        QString::from(&omikuji_core::system_info::report(env!("CARGO_PKG_VERSION"), qt))
    }

    fn app_version(&self) -> QString {
        QString::from(env!("CARGO_PKG_VERSION"))
    }

    fn cpu_core_count(&self) -> i32 {
        std::thread::available_parallelism()
            .map(|n| n.get() as i32)
            .unwrap_or(0)
    }

    fn index_of_id(&self, game_id: &QString) -> i32 {
        let needle = game_id.to_string();
        self.library
            .game
            .iter()
            .position(|g| g.metadata.id == needle)
            .map(|i| i as i32)
            .unwrap_or(-1)
    }

    fn duplicate_game(mut self: Pin<&mut Self>, index: i32) -> bool {
        let idx = index as usize;
        if idx >= self.library.game.len() {
            tracing::warn!("duplicate_game: invalid index {}", index);
            return false;
        }

        let game = self.library.game[idx].clone();

        match omikuji_core::desktop::duplicate_game(&game) {
            Ok(new_game) => {
                let new_name = new_game.metadata.name.clone();
                let new_id = new_game.metadata.id.clone();
                self.as_mut().insert_game_sorted(new_game);

                tracing::info!("duplicated game '{}' -> '{}' (id: {})",
                    game.metadata.name, new_name, new_id);
                true
            }
            Err(e) => {
                tracing::error!("duplicate_game failed: {}", e);
                false
            }
        }
    }

    fn is_flatpak(&self) -> bool {
        std::env::var("FLATPAK_ID").is_ok()
    }

    fn open_file_dialog(self: Pin<&mut Self>, request_id: &QString, select_folder: bool, title: &QString, default_path: &QString, filter: &QString) {
        let rid = request_id.to_string();
        let title_str = title.to_string();
        let default_str = default_path.to_string();
        let filter_str = filter.to_string();

        std::thread::spawn(move || {
            let result = omikuji_core::desktop::show_file_dialog(select_folder, &title_str, &default_str, &filter_str);
            omikuji_core::install_sizes::push_file_dialog(
                omikuji_core::install_sizes::FileDialogResult {
                    request_id: rid,
                    path: result,
                },
            );
        });
    }

    // glibc malloc_trim: paired with the store-panel Loader unload after hide,
    // makes idle rss drop visibly. gate on target_env=gnu, not available on musl/bsd/macos.
    fn trim_heap(&self) {
        #[cfg(all(target_os = "linux", target_env = "gnu"))]
        unsafe {
            libc::malloc_trim(0);
        }
    }

    fn disk_free_space(&self, path: &QString) -> QString {
        let bytes = omikuji_core::desktop::disk_free_space(&path.to_string());
        QString::from(&bytes.to_string())
    }

    fn home_dir(&self) -> QString {
        let path = dirs::home_dir().unwrap_or_default();
        QString::from(&*path.to_string_lossy())
    }

}

