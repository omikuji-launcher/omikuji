#![allow(clippy::too_many_arguments)]

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
        );

        // cxx_name required: cxx-qt doesn't auto-camelCase signal names for qml handlers
        #[qsignal]
        #[cxx_name = "gachaManifestsReady"]
        fn gacha_manifests_ready(self: Pin<&mut GameModel>, fetched: i32);

        #[qsignal]
        #[cxx_name = "gameLogAppended"]
        fn game_log_appended(self: Pin<&mut GameModel>, game_id: &QString);

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
        fn discard_new_game(self: Pin<&mut GameModel>);

        #[qinvokable]
        fn remove_game(self: Pin<&mut GameModel>, index: i32);

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
        #[cxx_name = "cpuCoreCount"]
        fn cpu_core_count(self: &GameModel) -> i32;

        #[qinvokable]
        fn stop_game(self: &GameModel, game_id: &QString);

        #[qinvokable]
        fn run_wine_tool(self: &GameModel, game_id: &QString, tool: &QString);

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
        fn enqueue_game_update(
            self: Pin<&mut GameModel>,
            game_id: &QString,
            from_version: &QString,
        ) -> QString;

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
        fn duplicate_game(self: Pin<&mut GameModel>, index: i32) -> bool;

        #[qinvokable]
        fn steam_is_installed(self: &GameModel) -> bool;

        #[qinvokable]
        fn steam_get_installed_games(self: &GameModel) -> QString;

        #[qinvokable]
        fn steam_import_game(self: Pin<&mut GameModel>, appid: &QString, name: &QString) -> bool;

        #[qinvokable]
        fn steam_local_library_image(self: &GameModel, appid: &QString) -> QString;

        // blocking http inside the tokio runtime panics; we escape to an os thread first
        #[qinvokable]
        fn steam_sync_playtime(self: Pin<&mut GameModel>);

        // result arrives async via file_dialog_result signal, not as a return value
        #[qinvokable]
        fn open_file_dialog(self: Pin<&mut GameModel>, request_id: &QString, select_folder: bool, title: &QString, default_path: &QString);

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
        fn home_dir(self: &GameModel) -> QString;

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
        fn gacha_resolve_poster(self: &GameModel, manifest_id: &QString) -> QString;

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

use std::path::PathBuf;
use std::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QByteArray, QHash, QHashPair_i32_QByteArray, QModelIndex, QMap, QMapPair_QString_QVariant, QString, QVariant};

use omikuji_core::library::{Game, Library};
use omikuji_core::media::{self, MediaType};

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

pub struct GameModelRust {
    library: Library,
    count: i32,
    // in-memory staging slot for the add-game page. cleared on commit/discard.
    draft: Option<Game>,
}

impl Default for GameModelRust {
    fn default() -> Self {
        let library = Library::load().unwrap_or_default();
        let count = library.game.len() as i32;
        Self { library, count, draft: None }
    }
}

fn runner_display(game: &Game) -> String {
    match game.runner.runner_type.as_str() {
        "steam" if !game.source.app_id.is_empty() => format!("steam:{}", game.source.app_id),
        "flatpak" if !game.source.app_id.is_empty() => format!("flatpak:{}", game.source.app_id),
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

fn populate_config_map(game: &Game, m: &mut QMap<QMapPair_QString_QVariant>) {
    macro_rules! put_str {
        ($k:expr, $v:expr) => {
            m.insert(QString::from($k), QVariant::from(&QString::from(&*$v)));
        };
    }
    macro_rules! put_bool {
        ($k:expr, $v:expr) => {
            m.insert(QString::from($k), QVariant::from(&$v));
        };
    }
    macro_rules! put_int {
        ($k:expr, $v:expr) => {
            m.insert(QString::from($k), QVariant::from(&($v as i32)));
        };
    }

    put_str!("meta.id", game.metadata.id);
    put_str!("meta.name", game.metadata.name);
    put_str!("meta.sort_name", game.metadata.sort_name);
    put_str!("meta.slug", game.metadata.slug);
    put_str!("meta.exe", game.metadata.exe.to_string_lossy());
    put_str!("meta.color", game.metadata.color);
    put_str!("meta.banner", game.metadata.banner);
    put_str!("meta.coverart", game.metadata.coverart);
    put_str!("meta.icon", game.metadata.icon);
    put_bool!("meta.favourite", game.metadata.favourite);
    if let Ok(json) = serde_json::to_string(&game.metadata.categories) {
        put_str!("meta.categories", json);
    }

    put_str!("source.kind", game.source.kind);
    put_str!("source.app_id", game.source.app_id);
    put_bool!("source.eos_overlay", game.source.eos_overlay);
    put_bool!("source.cloud_saves", game.source.cloud_saves);
    put_str!("source.save_path", game.source.save_path);
    put_str!("source.patch", game.source.patch);

    put_str!("runner.type", game.runner.runner_type);

    put_str!("wine.version", game.wine.version);
    put_str!("wine.prefix", game.wine.prefix);
    put_str!("wine.prefix_arch", game.wine.prefix_arch);
    put_bool!("wine.esync", game.wine.esync);
    put_bool!("wine.fsync", game.wine.fsync);
    put_bool!("wine.ntsync", game.wine.ntsync);
    put_bool!("wine.dxvk", game.wine.dxvk);
    put_str!("wine.dxvk_version", game.wine.dxvk_version);
    put_bool!("wine.vkd3d", game.wine.vkd3d);
    put_str!("wine.vkd3d_version", game.wine.vkd3d_version);
    put_bool!("wine.d3d_extras", game.wine.d3d_extras);
    put_str!("wine.d3d_extras_version", game.wine.d3d_extras_version);
    put_bool!("wine.dxvk_nvapi", game.wine.dxvk_nvapi);
    put_str!("wine.dxvk_nvapi_version", game.wine.dxvk_nvapi_version);
    put_bool!("wine.fsr", game.wine.fsr);
    put_bool!("wine.battleye", game.wine.battleye);
    put_bool!("wine.easyanticheat", game.wine.easyanticheat);
    put_bool!("wine.dpi_scaling", game.wine.dpi_scaling);
    put_int!("wine.dpi", game.wine.dpi);
    put_str!("wine.audio_driver", game.wine.audio_driver);
    put_str!("wine.graphics_driver", game.wine.graphics_driver);

    if let Ok(json) = serde_json::to_string(&game.wine.dll_overrides) {
        put_str!("wine.dll_overrides", json);
    }

    let args_text = args_to_text(&game.launch.args);
    put_str!("launch.args", args_text);
    put_str!("launch.working_dir", game.launch.working_dir);
    put_str!("launch.command_prefix", game.launch.command_prefix);
    put_str!("launch.pre_launch_script", game.launch.pre_launch_script);
    put_str!("launch.post_exit_script", game.launch.post_exit_script);
    if let Ok(json) = serde_json::to_string(&game.launch.env) {
        put_str!("launch.env", json);
    }

    put_bool!("graphics.mangohud", game.graphics.mangohud);
    put_str!("graphics.gpu", game.graphics.gpu);

    put_bool!("graphics.gamescope.enabled", game.graphics.gamescope.enabled);
    put_int!("graphics.gamescope.width", game.graphics.gamescope.width);
    put_int!("graphics.gamescope.height", game.graphics.gamescope.height);
    put_int!("graphics.gamescope.game_width", game.graphics.gamescope.game_width);
    put_int!("graphics.gamescope.game_height", game.graphics.gamescope.game_height);
    put_int!("graphics.gamescope.fps", game.graphics.gamescope.fps);
    put_bool!("graphics.gamescope.fullscreen", game.graphics.gamescope.fullscreen);
    put_bool!("graphics.gamescope.borderless", game.graphics.gamescope.borderless);
    put_bool!("graphics.gamescope.integer_scaling", game.graphics.gamescope.integer_scaling);
    put_bool!("graphics.gamescope.hdr", game.graphics.gamescope.hdr);
    put_str!("graphics.gamescope.filter", game.graphics.gamescope.filter);
    put_int!("graphics.gamescope.fsr_sharpness", game.graphics.gamescope.fsr_sharpness);

    put_bool!("system.gamemode", game.system.gamemode);
    put_bool!("system.prevent_sleep", game.system.prevent_sleep);
    put_bool!("system.pulse_latency", game.system.pulse_latency);
    put_int!("system.cpu_limit", game.system.cpu_limit);
}

fn apply_field_to_game(game: &mut Game, key: &str, value: &str) -> bool {
    let parse_bool = |s: &str| -> bool { s == "true" };
    let parse_u32 = |s: &str| -> u32 { s.parse().unwrap_or(0) };

    match key {
        "meta.name" => game.metadata.name = value.to_string(),
        "meta.sort_name" => game.metadata.sort_name = value.to_string(),
        "meta.slug" => game.metadata.slug = value.to_string(),
        "meta.exe" => game.metadata.exe = PathBuf::from(value),
        "meta.color" => game.metadata.color = value.to_string(),
        "meta.banner" => game.metadata.banner = value.to_string(),
        "meta.coverart" => game.metadata.coverart = value.to_string(),
        "meta.icon" => game.metadata.icon = value.to_string(),
        "meta.favourite" => game.metadata.favourite = parse_bool(value),
        "meta.categories" => {
            if let Ok(cats) = serde_json::from_str(value) {
                game.metadata.categories = cats;
            }
        }

        "source.save_path" => game.source.save_path = value.to_string(),
        "source.app_id" => game.source.app_id = value.to_string(),

        "runner.type" => game.runner.runner_type = value.to_string(),

        "wine.version" => game.wine.version = value.to_string(),
        "wine.prefix" => game.wine.prefix = value.to_string(),
        "wine.prefix_arch" => game.wine.prefix_arch = value.to_string(),
        "wine.esync" => game.wine.esync = parse_bool(value),
        "wine.fsync" => game.wine.fsync = parse_bool(value),
        "wine.ntsync" => game.wine.ntsync = parse_bool(value),
        "wine.dxvk" => game.wine.dxvk = parse_bool(value),
        "wine.dxvk_version" => game.wine.dxvk_version = value.to_string(),
        "wine.vkd3d" => game.wine.vkd3d = parse_bool(value),
        "wine.vkd3d_version" => game.wine.vkd3d_version = value.to_string(),
        "wine.d3d_extras" => game.wine.d3d_extras = parse_bool(value),
        "wine.d3d_extras_version" => game.wine.d3d_extras_version = value.to_string(),
        "wine.dxvk_nvapi" => game.wine.dxvk_nvapi = parse_bool(value),
        "wine.dxvk_nvapi_version" => game.wine.dxvk_nvapi_version = value.to_string(),
        "wine.fsr" => game.wine.fsr = parse_bool(value),
        "wine.battleye" => game.wine.battleye = parse_bool(value),
        "wine.easyanticheat" => game.wine.easyanticheat = parse_bool(value),
        "wine.dpi_scaling" => game.wine.dpi_scaling = parse_bool(value),
        "wine.dpi" => game.wine.dpi = parse_u32(value),
        "wine.audio_driver" => game.wine.audio_driver = value.to_string(),
        "wine.graphics_driver" => game.wine.graphics_driver = value.to_string(),
        "wine.dll_overrides" => {
            if let Ok(map) = serde_json::from_str(value) {
                game.wine.dll_overrides = map;
            }
        }

        "launch.args" => game.launch.args = args_from_text(value),
        "launch.working_dir" => game.launch.working_dir = value.to_string(),
        "launch.command_prefix" => game.launch.command_prefix = value.to_string(),
        "launch.pre_launch_script" => game.launch.pre_launch_script = value.to_string(),
        "launch.post_exit_script" => game.launch.post_exit_script = value.to_string(),
        "launch.env" => {
            if let Ok(env) = serde_json::from_str(value) {
                game.launch.env = env;
            }
        }

        "graphics.mangohud" => game.graphics.mangohud = parse_bool(value),
        "graphics.gpu" => game.graphics.gpu = value.to_string(),

        "graphics.gamescope.enabled" => game.graphics.gamescope.enabled = parse_bool(value),
        "graphics.gamescope.width" => game.graphics.gamescope.width = parse_u32(value),
        "graphics.gamescope.height" => game.graphics.gamescope.height = parse_u32(value),
        "graphics.gamescope.game_width" => game.graphics.gamescope.game_width = parse_u32(value),
        "graphics.gamescope.game_height" => game.graphics.gamescope.game_height = parse_u32(value),
        "graphics.gamescope.fps" => game.graphics.gamescope.fps = parse_u32(value),
        "graphics.gamescope.fullscreen" => game.graphics.gamescope.fullscreen = parse_bool(value),
        "graphics.gamescope.borderless" => game.graphics.gamescope.borderless = parse_bool(value),
        "graphics.gamescope.integer_scaling" => game.graphics.gamescope.integer_scaling = parse_bool(value),
        "graphics.gamescope.hdr" => game.graphics.gamescope.hdr = parse_bool(value),
        "graphics.gamescope.filter" => game.graphics.gamescope.filter = value.to_string(),
        "graphics.gamescope.fsr_sharpness" => game.graphics.gamescope.fsr_sharpness = parse_u32(value),

        "system.gamemode" => game.system.gamemode = parse_bool(value),
        "system.prevent_sleep" => game.system.prevent_sleep = parse_bool(value),
        "system.pulse_latency" => game.system.pulse_latency = parse_bool(value),
        "system.cpu_limit" => game.system.cpu_limit = parse_u32(value),

        _ => {
            eprintln!("unknown config key: {}", key);
            return false;
        }
    }

    true
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
            eprintln!("[data] row={} name='{}' coverart='{}'",
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
        hash.insert_clone(&ROLE_CATEGORIES, &QByteArray::from("categories"));
        hash.insert_clone(&ROLE_RUNNER_TYPE, &QByteArray::from("runnerType"));
        hash
    }

    fn begin_new_game(mut self: Pin<&mut Self>) -> QMap<QMapPair_QString_QVariant> {
        let mut game = Game::new(String::new(), PathBuf::new());
        game.seed_from_defaults(&omikuji_core::defaults::Defaults::load());
        let mut m = QMap::<QMapPair_QString_QVariant>::default();
        populate_config_map(&game, &mut m);
        self.as_mut().rust_mut().get_mut().draft = Some(game);
        m
    }

    fn get_draft_config(&self) -> QMap<QMapPair_QString_QVariant> {
        let mut m = QMap::<QMapPair_QString_QVariant>::default();
        if let Some(game) = &self.rust().draft {
            populate_config_map(game, &mut m);
        }
        m
    }

    fn update_draft_field(mut self: Pin<&mut Self>, key: &QString, value: &QString) -> bool {
        let k = key.to_string();
        let v = value.to_string();
        let Some(game) = self.as_mut().rust_mut().get_mut().draft.as_mut() else {
            return false;
        };
        apply_field_to_game(game, &k, &v)
    }

    // on failure, draft is preserved so the user can fix fields and retry (a bit useless most of the times but may it be a connection error)
    fn commit_new_game(mut self: Pin<&mut Self>) -> QString {
        let Some(mut game) = self.as_mut().rust_mut().get_mut().draft.take() else {
            eprintln!("commit_new_game: no draft");
            return QString::default();
        };

        // exe is allowed empty for non-wine runners (steam, flatpak, etc)
        if game.metadata.name.trim().is_empty() {
            eprintln!("commit_new_game: name is required");
            self.as_mut().rust_mut().get_mut().draft = Some(game);
            return QString::default();
        }

        game.metadata.name = game.metadata.name.trim().to_string();

        let game_id = game.metadata.id.clone();
        let game_name = game.metadata.name.clone();
        let row = self.library.game.len() as i32;

        if let Err(e) = Library::save_game_static(&game) {
            eprintln!("commit_new_game: failed to save: {}", e);
            self.as_mut().rust_mut().get_mut().draft = Some(game);
            return QString::default();
        }

        self.as_mut().begin_insert_rows(&QModelIndex::default(), row, row);
        self.as_mut().rust_mut().get_mut().library.game.push(game);
        let count = self.library.game.len() as i32;
        self.as_mut().set_count(count);
        self.as_mut().end_insert_rows();

        let qt_thread = self.as_mut().qt_thread();
        let id_for_refresh = game_id.clone();
        std::thread::spawn(move || {
            let result = media::fetch_media_blocking_with(&game_id, &game_name, |_| {
                let id_inner = id_for_refresh.clone();
                let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::GameModel>| {
                    let Some(row) = obj.library.game.iter().position(|g| g.metadata.id == id_inner) else {
                        return;
                    };
                    let idx = obj.as_ref().model_index(row as i32, 0, &QModelIndex::default());
                    let roles = cxx_qt_lib::QList::<i32>::default();
                    obj.as_mut().data_changed(&idx, &idx, &roles);
                });
            });
            let fetched: Vec<&str> = [
                result.banner.as_ref().map(|_| "banner"),
                result.coverart.as_ref().map(|_| "coverart"),
                result.icon.as_ref().map(|_| "icon"),
            ]
            .into_iter()
            .flatten()
            .collect();
            if fetched.is_empty() {
                eprintln!("no media found for '{}'", game_name);
            } else {
                eprintln!("fetched {} for '{}'", fetched.join(", "), game_name);
            }
        });

        QString::from(&*self.library.game.last().unwrap().metadata.id)
    }

    fn discard_new_game(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().get_mut().draft = None;
    }

    fn remove_game(mut self: Pin<&mut Self>, index: i32) {
        let idx = index as usize;
        if idx >= self.library.game.len() {
            return;
        }

        let game_id = self.library.game[idx].metadata.id.clone();

        self.as_mut()
            .begin_remove_rows(&QModelIndex::default(), index, index);

        self.as_mut().rust_mut().get_mut().library.game.remove(idx);

        let lib = &mut self.as_mut().rust_mut().get_mut().library;
        if let Err(e) = lib.remove_game(&game_id) {
            eprintln!("failed to remove game file: {}", e);
        }

        media::remove_cached_media(&game_id);

        let count = self.library.game.len() as i32;
        self.as_mut().set_count(count);
        self.end_remove_rows();
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
            Ok(new_lib) => {
                let new_count = new_lib.game.len() as i32;
                self.as_mut().begin_reset_model();
                self.as_mut().rust_mut().get_mut().library = new_lib;
                self.as_mut().set_count(new_count);
                self.as_mut().end_reset_model();
                QString::from(&*selected_id)
            }
            Err(e) => {
                eprintln!("failed to reload library: {}", e);
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

    fn launch_game(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            eprintln!("launch_game: invalid index {}", index);
            return false;
        };

        if omikuji_core::process::is_game_running(&game.metadata.id) {
            eprintln!("game '{}' is already running", game.metadata.name);
            return false;
        }

        // pre-launch update check for gacha games only. network errors are intentionally swallowed so a hiccup doesnt block the user from playing.
        if game.source.kind == "gacha"
            && let Some(info) = blocking_check_gacha_update(&game.source.app_id) {
                omikuji_core::process::notify_update_required(
                    omikuji_core::process::UpdateNotification {
                        game_id: game.metadata.id.clone(),
                        app_id: game.source.app_id.clone(),
                        from_version: info.from_version,
                        to_version: info.to_version,
                        download_size: info.download_size,
                        can_diff: info.can_diff,
                    },
                );
                return false;
            }

        match omikuji_core::launch::build_launch(game) {
            Ok(config) => {
                eprintln!("launching '{}': {:?}", game.metadata.name, config.command);

                // spawn os thread + build fresh runtime: we're already inside the #[tokio::main] runtime, cant block_on from here directly
                let game_name = game.metadata.name.clone();
                let logs_dir = omikuji_core::logs_dir();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        match omikuji_core::process::launch_game(&config).await {
                            Ok(proc_id) => {
                                eprintln!("game '{}' launched, process id: {:?}", game_name, proc_id);
                                eprintln!("logs: {}", logs_dir.display());
                            }
                            Err(e) => {
                                eprintln!("failed to launch '{}': {}", game_name, e);
                            }
                        }
                    });
                });

                true
            }
            Err(e) => {
                eprintln!("failed to build launch config: {}", e);
                false
            }
        }
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
        let mut m = QMap::<QMapPair_QString_QVariant>::default();
        populate_config_map(game, &mut m);
        m
    }

    fn update_game_field(mut self: Pin<&mut Self>, index: i32, key: &QString, value: &QString) -> bool {
        let idx = index as usize;
        let k = key.to_string();
        let v = value.to_string();
        let Some(game) = self.as_mut().rust_mut().get_mut().library.game.get_mut(idx) else {
            return false;
        };
        apply_field_to_game(game, &k, &v)
    }

    fn save_game(self: Pin<&mut Self>, game_id: &QString) -> bool {
        let id = game_id.to_string();

        let game = match self.library.game.iter().find(|g| g.metadata.id == id) {
            Some(g) => g.clone(),
            None => {
                eprintln!("save_game: game with id '{}' not found", id);
                return false;
            }
        };

        eprintln!("[save_game] saving game '{}' id '{}'", game.metadata.name, game.metadata.id);

        if let Err(e) = Library::save_game_static(&game) {
            eprintln!("failed to save game config: {}", e);
            return false;
        }
        true
    }

    fn refetch_media(mut self: Pin<&mut Self>, game_id: &QString) {
        let id = game_id.to_string();
        let name = match self.library.game.iter().find(|g| g.metadata.id == id) {
            Some(g) => g.metadata.name.clone(),
            None => {
                eprintln!("refetch_media: game id '{}' not found", id);
                return;
            }
        };

        let qt_thread = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            let id_for_refresh = id.clone();
            media::fetch_media_blocking_with(&id, &name, |_| {
                let id_inner = id_for_refresh.clone();
                let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::GameModel>| {
                    let Some(row) = obj.library.game.iter().position(|g| g.metadata.id == id_inner) else {
                        return;
                    };
                    let idx = obj.as_ref().model_index(row as i32, 0, &QModelIndex::default());
                    let roles = cxx_qt_lib::QList::<i32>::default();
                    obj.as_mut().data_changed(&idx, &idx, &roles);
                });
            });
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
                Err(e) => eprintln!(
                    "[apply_defaults] save failed for {}: {}",
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

    fn cpu_core_count(&self) -> i32 {
        std::thread::available_parallelism()
            .map(|n| n.get() as i32)
            .unwrap_or(0)
    }

    fn stop_game(&self, game_id: &QString) {
        let id = game_id.to_string();
        eprintln!("[stop_game] requesting stop for game '{}'", id);
        omikuji_core::process::stop_game(&id);
    }

    // tool name must match one of the WineTool enum arms below; unknown names are dropped
    fn run_wine_tool(&self, game_id: &QString, tool: &QString) {
        let id = game_id.to_string();
        let tool_name = tool.to_string();
        let Some(game) = self
            .library
            .game
            .iter()
            .find(|g| g.metadata.id == id)
            .cloned()
        else {
            eprintln!("[run_wine_tool] game '{}' not found", id);
            return;
        };
        let t = match tool_name.as_str() {
            "winecfg" => omikuji_core::wine_tools::WineTool::Winecfg,
            "winetricks" => omikuji_core::wine_tools::WineTool::Winetricks,
            "regedit" => omikuji_core::wine_tools::WineTool::Regedit,
            "cmd" => omikuji_core::wine_tools::WineTool::Cmd,
            "winefile" => omikuji_core::wine_tools::WineTool::Winefile,
            "killwineserver" => omikuji_core::wine_tools::WineTool::KillWineserver,
            other => {
                eprintln!("[run_wine_tool] unknown tool '{}'", other);
                return;
            }
        };
        let display_name = game.metadata.name.clone();
        let tool_label = tool_name.clone();
        // prefix-init and umu-run startup can be slow, detach so the ui doesnt block
        std::thread::spawn(move || match omikuji_core::wine_tools::run(&game, t) {
            Ok(_child) => {
                omikuji_core::notifications::info(&display_name, format!("Opened {}", tool_label));
            }
            Err(e) => {
                omikuji_core::notifications::error(
                    &display_name,
                    format!("{} failed: {}", tool_label, e),
                );
            }
        });
    }

    fn run_wine_exe(&self, game_id: &QString, exe_path: &QString) {
        let id = game_id.to_string();
        let exe = exe_path.to_string();
        if exe.is_empty() {
            return;
        }
        let Some(game) = self
            .library
            .game
            .iter()
            .find(|g| g.metadata.id == id)
            .cloned()
        else {
            eprintln!("[run_wine_exe] game '{}' not found", id);
            return;
        };
        let display_name = game.metadata.name.clone();
        let path = std::path::PathBuf::from(&exe);
        let file_label = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| exe.clone());
        std::thread::spawn(move || {
            match omikuji_core::wine_tools::run(&game, omikuji_core::wine_tools::WineTool::RunExe(path)) {
                Ok(_child) => {
                    omikuji_core::notifications::info(
                        &display_name,
                        format!("Running {}", file_label),
                    );
                }
                Err(e) => {
                    omikuji_core::notifications::error(
                        &display_name,
                        format!("Run failed: {}", e),
                    );
                }
            }
        });
    }

    fn check_exited_games(mut self: Pin<&mut Self>) {
        for game_id in omikuji_core::process::take_exited_games() {
            self.as_mut().game_stopped(&QString::from(&game_id));
        }
    }

    fn drain_game_log_events(mut self: Pin<&mut Self>) {
        for id in omikuji_core::game_logs::drain_dirty() {
            self.as_mut().game_log_appended(&QString::from(&id));
        }
    }

    fn game_log(&self, game_id: &QString) -> QString {
        QString::from(&omikuji_core::game_logs::get_log(&game_id.to_string()))
    }

    fn clear_game_log(&self, game_id: &QString) {
        omikuji_core::game_logs::clear_log(&game_id.to_string());
    }

    fn save_game_log(&self, game_id: &QString) -> QString {
        let id = game_id.to_string();
        let body = omikuji_core::game_logs::get_log(&id);
        if body.is_empty() {
            return QString::from("");
        }
        let dir = omikuji_core::logs_dir();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("[game_log] couldn't create {}: {}", dir.display(), e);
            return QString::from("");
        }
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let file = dir.join(format!("{}_{}.log", id, ts));
        match std::fs::write(&file, body) {
            Ok(_) => QString::from(file.to_string_lossy().as_ref()),
            Err(e) => {
                eprintln!("[game_log] write {} failed: {}", file.display(), e);
                QString::from("")
            }
        }
    }

    fn drain_notifications(mut self: Pin<&mut Self>) {
        for n in omikuji_core::notifications::take_pending() {
            self.as_mut().notification(
                &QString::from(n.level.as_str()),
                &QString::from(&n.title),
                &QString::from(&n.message),
            );
        }
    }

    fn drain_update_notifications(mut self: Pin<&mut Self>) {
        for n in omikuji_core::process::take_update_notifications() {
            let display_name = self
                .library
                .game
                .iter()
                .find(|g| g.metadata.id == n.game_id)
                .map(|g| g.metadata.name.clone())
                .unwrap_or_default();
            self.as_mut().update_required(
                &QString::from(&n.game_id),
                &QString::from(&n.app_id),
                &QString::from(&display_name),
                &QString::from(&n.from_version),
                &QString::from(&n.to_version),
                &QString::from(&n.download_size.to_string()),
                n.can_diff,
            );
        }
    }

    fn enqueue_game_update(
        mut self: Pin<&mut Self>,
        game_id: &QString,
        from_version: &QString,
    ) -> QString {
        let gid = game_id.to_string();
        let from = from_version.to_string();

        let Some(game) = self.library.game.iter().find(|g| g.metadata.id == gid) else {
            eprintln!("[update] enqueue_game_update: game '{}' not found", gid);
            return QString::from("");
        };

        if game.source.kind != "gacha" {
            eprintln!("[update] enqueue_game_update: game '{}' is not a gacha", gid);
            return QString::from("");
        }

        let app_id = game.source.app_id.clone();
        let display_name = game.metadata.name.clone();

        let (source_key, banner_url) = match omikuji_core::gachas::strategies::find_for_app_id(&app_id) {
            Some((manifest, _edition_id, _voices)) => {
                let src = match omikuji_core::gachas::strategies::source_key(&manifest) {
                    Ok(s) => s.to_string(),
                    Err(e) => {
                        eprintln!("[update] unknown strategy for '{}': {}", manifest.id, e);
                        return QString::from("");
                    }
                };
                let poster = omikuji_core::gachas::strategies::resolve_poster(&manifest);
                (src, if poster.is_empty() { None } else { Some(poster) })
            }
            None => {
                eprintln!("[update] no gacha manifest for app_id '{}'", app_id);
                return QString::from("");
            }
        };

        // exe's parent dir is the game-data root; sophon patcher and resource probes need that, not th exe itself
        let install_path = std::path::PathBuf::from(&game.metadata.exe)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from(&game.metadata.exe));

        let prefix = if game.wine.prefix.is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(&game.wine.prefix))
        };
        let runner_version = game.wine.version.clone();

        let req = omikuji_core::downloads::DownloadRequest {
            source: source_key,
            app_id,
            display_name: format!("{} · update", display_name),
            banner_url,
            install_path,
            prefix_path: prefix,
            runner_version,
            temp_dir: None,
            kind: omikuji_core::downloads::DownloadKind::Update { from_version: from },
            // updates patch an existing install, never wipe on cancel
            destructive_cleanup: false,
        };

        let id = omikuji_core::downloads::manager().enqueue(req);
        let _ = self.as_mut();
        QString::from(&id)
    }

    fn browse_files(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            eprintln!("browse_files: invalid index {}", index);
            return false;
        };

        let Some(dir) = omikuji_core::desktop::get_game_browse_dir(game) else {
            eprintln!("browse_files: no directory for game '{}'", game.metadata.name);
            return false;
        };

        match omikuji_core::desktop::browse_files(&dir) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("browse_files failed: {}", e);
                false
            }
        }
    }


    fn create_desktop_shortcut(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            eprintln!("create_desktop_shortcut: invalid index {}", index);
            return false;
        };

        match omikuji_core::desktop::create_desktop_shortcut(game) {
            Ok(path) => {
                eprintln!("created desktop shortcut: {}", path.display());
                true
            }
            Err(e) => {
                eprintln!("create_desktop_shortcut failed: {}", e);
                false
            }
        }
    }

    fn create_menu_shortcut(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            eprintln!("create_menu_shortcut: invalid index {}", index);
            return false;
        };

        match omikuji_core::desktop::create_menu_shortcut(game) {
            Ok(path) => {
                eprintln!("created menu shortcut: {}", path.display());
                true
            }
            Err(e) => {
                eprintln!("create_menu_shortcut failed: {}", e);
                false
            }
        }
    }

    fn remove_desktop_shortcut(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            return false;
        };

        match omikuji_core::desktop::remove_desktop_shortcut(game) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("remove_desktop_shortcut failed: {}", e);
                false
            }
        }
    }

    fn remove_menu_shortcut(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            return false;
        };

        match omikuji_core::desktop::remove_menu_shortcut(game) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("remove_menu_shortcut failed: {}", e);
                false
            }
        }
    }

    fn has_desktop_shortcut(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            return false;
        };
        omikuji_core::desktop::desktop_shortcut_exists(game)
    }

    fn has_menu_shortcut(&self, index: i32) -> bool {
        let idx = index as usize;
        let Some(game) = self.library.game.get(idx) else {
            return false;
        };
        omikuji_core::desktop::menu_shortcut_exists(game)
    }

    fn duplicate_game(mut self: Pin<&mut Self>, index: i32) -> bool {
        let idx = index as usize;
        if idx >= self.library.game.len() {
            eprintln!("duplicate_game: invalid index {}", index);
            return false;
        }

        let game = self.library.game[idx].clone();

        match omikuji_core::desktop::duplicate_game(&game) {
            Ok(new_game) => {
                let new_name = new_game.metadata.name.clone();
                let new_id = new_game.metadata.id.clone();
                let row = self.library.game.len() as i32;

                self.as_mut()
                    .begin_insert_rows(&QModelIndex::default(), row, row);

                self.as_mut()
                    .rust_mut()
                    .get_mut()
                    .library
                    .game
                    .push(new_game);

                let count = self.library.game.len() as i32;
                self.as_mut().set_count(count);
                self.as_mut().end_insert_rows();

                eprintln!("duplicated game '{}' -> '{}' (id: {})",
                    game.metadata.name, new_name, new_id);
                true
            }
            Err(e) => {
                eprintln!("duplicate_game failed: {}", e);
                false
            }
        }
    }

    fn steam_is_installed(&self) -> bool {
        omikuji_core::steam::is_steam_installed()
    }

    fn steam_get_installed_games(&self) -> QString {
        let games = omikuji_core::steam::get_installed_games();
        let json_games: Vec<serde_json::Value> = games.iter().map(|g| {
            serde_json::json!({
                "appid": g.appid,
                "name": g.name,
                "is_installed": g.is_installed()
            })
        }).collect();

        match serde_json::to_string(&json_games) {
            Ok(json) => QString::from(&json),
            Err(_) => QString::from("[]"),
        }
    }

    fn steam_local_library_image(&self, appid: &QString) -> QString {
        let appid_str = appid.to_string();
        match omikuji_core::steam::local::find_local_library_image(&appid_str) {
            Some(path) => QString::from(&*path.to_string_lossy()),
            None => QString::default(),
        }
    }

    fn steam_import_game(mut self: Pin<&mut Self>, appid: &QString, name: &QString) -> bool {
        let appid_str = appid.to_string();
        let name_str = name.to_string();

        eprintln!("[steam_import] importing {} - {}", appid_str, name_str);

        let already_imported = self.library.game.iter().any(|g| {
            g.metadata.id == appid_str
        });

        if already_imported {
            eprintln!("[steam_import] already imported: {}", appid_str);
            return true;
        }

        use omikuji_core::library::{Metadata, RunnerConfig, SourceConfig, WineConfig, LaunchConfig, GraphicsConfig, SystemConfig, default_color};

        let mut game = Game {
            metadata: Metadata {
                id: appid_str.clone(),
                name: name_str.clone(),
                sort_name: String::new(),
                slug: String::new(),
                exe: PathBuf::new(),
                color: default_color(),
                playtime: 0.0,
                last_played: String::new(),
                banner: String::new(),
                coverart: String::new(),
                icon: String::new(),
                favourite: false,
                categories: Vec::new(),
            },
            source: SourceConfig {
                kind: "steam".to_string(),
                app_id: appid_str.clone(),
                ..SourceConfig::default()
            },
            runner: RunnerConfig {
                runner_type: "steam".to_string(),
            },
            wine: WineConfig {
                version: format!("steam:{}", appid_str),
                ..WineConfig::default()
            },
            launch: LaunchConfig::default(),
            graphics: GraphicsConfig::default(),
            system: SystemConfig::default(),
        };
        game.seed_from_defaults(&omikuji_core::defaults::Defaults::load());

        let row = self.library.game.len() as i32;

        if let Err(e) = Library::save_game_static(&game) {
            eprintln!("[steam_import] failed to save game: {}", e);
            return false;
        }

        let appid_for_media = appid_str.clone();
        std::thread::spawn(move || {
            let result = media::fetch_steam_media_blocking(&appid_for_media);
            let fetched: Vec<&str> = [
                result.banner.as_ref().map(|_| "banner"),
                result.coverart.as_ref().map(|_| "coverart"),
            ]
            .into_iter()
            .flatten()
            .collect();

            if fetched.is_empty() {
                eprintln!("no steam media found for appid {}", appid_for_media);
            } else {
                eprintln!("fetched steam {} for appid {}", fetched.join(", "), appid_for_media);
            }
        });

        self.as_mut()
            .begin_insert_rows(&QModelIndex::default(), row, row);

        self.as_mut()
            .rust_mut()
            .get_mut()
            .library
            .game
            .push(game);

        let count = self.library.game.len() as i32;
        self.as_mut().set_count(count);
        self.as_mut().end_insert_rows();

        eprintln!("[steam_import] imported '{}' (steam appid: {})", name_str, appid_str);
        true
    }

    fn steam_sync_playtime(mut self: Pin<&mut Self>) {
        let api_key = omikuji_core::settings::get().steam.api_key.clone();
        if api_key.is_empty() {
            return;
        }

        eprintln!("[steam_sync] syncing playtime from steam api...");
        let qt_thread = self.as_mut().qt_thread();

        // blocking reqwest inside #[tokio::main] panics; escape to an os thread, then marshal the mutation back via qt_thread.queue
        std::thread::spawn(move || {
            let fetch_result = omikuji_core::steam::fetch_playtime_data(&api_key);

            let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::GameModel>| {
                let steam_data = match fetch_result {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("[steam_sync] failed: {}", e);
                        return;
                    }
                };

                let library = &mut obj.as_mut().rust_mut().get_mut().library;
                let (updated, total) = omikuji_core::steam::apply_playtime_data(library, &steam_data);
                eprintln!("[steam_sync] updated {}/{} steam games", updated, total);

                let mut saved = 0;
                for game in &library.game {
                    if game.runner.runner_type == "steam" {
                        if let Err(e) = Library::save_game_static(game) {
                            eprintln!("[steam_sync] failed to save {}: {}", game.metadata.id, e);
                        } else {
                            saved += 1;
                        }
                    }
                }
                eprintln!("[steam_sync] saved {} games to disk", saved);

                obj.as_mut().begin_reset_model();
                obj.as_mut().end_reset_model();
            });
        });
    }

    fn open_file_dialog(self: Pin<&mut Self>, request_id: &QString, select_folder: bool, title: &QString, default_path: &QString) {
        let rid = request_id.to_string();
        let title_str = title.to_string();
        let default_str = default_path.to_string();

        std::thread::spawn(move || {
            let result = omikuji_core::desktop::show_file_dialog(select_folder, &title_str, &default_str);
            omikuji_core::install_sizes::push_file_dialog(
                omikuji_core::install_sizes::FileDialogResult {
                    request_id: rid,
                    path: result,
                },
            );
        });
    }

    fn drain_file_dialog_results(mut self: Pin<&mut Self>) {
        for r in omikuji_core::install_sizes::take_file_dialog_pending() {
            self.as_mut().file_dialog_result(
                &QString::from(&r.request_id),
                &QString::from(&r.path),
            );
        }
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

    fn epic_check_existing_install(
        &self,
        app_name: &QString,
        install_path: &QString,
    ) -> QString {
        let app_s = app_name.to_string();
        let install_s = install_path.to_string();
        if app_s.is_empty() || install_s.trim().is_empty() {
            return QString::from(r#"{"bytes":0,"hasResume":false}"#);
        }
        let install = std::path::PathBuf::from(install_s.trim());
        let (bytes, has_resume) = omikuji_core::epic::inspect_existing_install(&app_s, &install);
        QString::from(&format!(
            r#"{{"bytes":{},"hasResume":{}}}"#,
            bytes, has_resume
        ))
    }

    fn fetch_epic_install_size(
        self: Pin<&mut Self>,
        request_id: &QString,
        app_name: &QString,
    ) {
        let rid = request_id.to_string();
        let app_name_str = app_name.to_string();

        // os thread + fresh runtime: cant call block_on inside the existing tokio context
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[epic_size] failed to create tokio runtime: {}", e);
                    omikuji_core::install_sizes::push(omikuji_core::install_sizes::InstallSizeResult {
                        request_id: rid,
                        download_bytes: 0,
                        install_bytes: 0,
                        error: format!("tokio runtime: {}", e),
                    });
                    return;
                }
            };
            let result = rt.block_on(async {
                omikuji_core::epic::fetch_install_size(&app_name_str).await
            });

            let pushed = match result {
                Ok(size) => omikuji_core::install_sizes::InstallSizeResult {
                    request_id: rid,
                    download_bytes: size.download_bytes,
                    install_bytes: size.install_bytes,
                    error: String::new(),
                },
                Err(e) => {
                    eprintln!("[epic_size] error: {}", e);
                    omikuji_core::install_sizes::InstallSizeResult {
                        request_id: rid,
                        download_bytes: 0,
                        install_bytes: 0,
                        error: format!("{}", e),
                    }
                }
            };
            omikuji_core::install_sizes::push(pushed);
        });
    }

    fn drain_install_sizes(mut self: Pin<&mut Self>) {
        for r in omikuji_core::install_sizes::take_pending() {
            let payload = serde_json::json!({
                "download": r.download_bytes.to_string(),
                "install": r.install_bytes.to_string(),
                "error": r.error,
            })
            .to_string();
            self.as_mut().install_size_result(
                &QString::from(&r.request_id),
                &QString::from(&payload),
            );
        }
    }

    fn home_dir(&self) -> QString {
        let path = dirs::home_dir().unwrap_or_default();
        QString::from(&*path.to_string_lossy())
    }

    fn epic_import_after_install(
        mut self: Pin<&mut Self>,
        app_name: &QString,
        display_name: &QString,
        prefix_path: &QString,
        runner_version: &QString,
    ) -> QString {
        use omikuji_core::library::{
            default_color, GraphicsConfig, LaunchConfig, Metadata, RunnerConfig, SourceConfig, SystemConfig, WineConfig,
        };

        let app_name_s = app_name.to_string();

        if self.library.game.iter().any(|g| g.metadata.id == app_name_s) {
            eprintln!("[epic_import] already in library: {}", app_name_s);
            return QString::from(&app_name_s);
        }

        let Some(info) = omikuji_core::epic::find_installed_info(&app_name_s) else {
            eprintln!("[epic_import] no install info for {} — leaving library alone", app_name_s);
            return QString::default();
        };

        let display_str = display_name.to_string();
        let title = info
            .title
            .clone()
            .unwrap_or_else(|| if display_str.is_empty() { app_name_s.clone() } else { display_str });
        let prefix_str = prefix_path.to_string();
        let runner_str = runner_version.to_string();

        let mut game = Game {
            metadata: Metadata {
                id: app_name_s.clone(),
                name: title.clone(),
                sort_name: String::new(),
                slug: String::new(),
                exe: info.executable.clone(),
                color: default_color(),
                playtime: 0.0,
                last_played: String::new(),
                banner: String::new(),
                coverart: String::new(),
                icon: String::new(),
                favourite: false,
                categories: vec!["Epic Games".to_string()],
            },
            source: SourceConfig {
                kind: "epic".to_string(),
                app_id: app_name_s.clone(),
                ..SourceConfig::default()
            },
            runner: RunnerConfig {
                runner_type: "wine".to_string(),
            },
            wine: WineConfig {
                version: runner_str,
                prefix: prefix_str,
                ..WineConfig::default()
            },
            launch: LaunchConfig::default(),
            graphics: GraphicsConfig::default(),
            system: SystemConfig::default(),
        };
        game.seed_from_defaults(&omikuji_core::defaults::Defaults::load());

        let row = self.library.game.len() as i32;

        if let Err(e) = Library::save_game_static(&game) {
            eprintln!("[epic_import] failed to save: {}", e);
            return QString::default();
        }

        let id_for_media = game.metadata.id.clone();
        let name_for_media = game.metadata.name.clone();
        let qt_thread = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            let id_for_refresh = id_for_media.clone();
            media::fetch_media_blocking_with(&id_for_media, &name_for_media, |_| {
                let id_inner = id_for_refresh.clone();
                let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::GameModel>| {
                    let Some(row) = obj.library.game.iter().position(|g| g.metadata.id == id_inner) else {
                        return;
                    };
                    let idx = obj.as_ref().model_index(row as i32, 0, &QModelIndex::default());
                    let roles = cxx_qt_lib::QList::<i32>::default();
                    obj.as_mut().data_changed(&idx, &idx, &roles);
                });
            });
        });

        self.as_mut().begin_insert_rows(&QModelIndex::default(), row, row);
        self.as_mut().rust_mut().get_mut().library.game.push(game);
        let count = self.library.game.len() as i32;
        self.as_mut().set_count(count);
        self.as_mut().end_insert_rows();

        eprintln!("[epic_import] imported '{}' as id '{}'", title, app_name_s);
        QString::from(&app_name_s)
    }

    fn gog_check_existing_install(
        &self,
        app_name: &QString,
        install_path: &QString,
    ) -> QString {
        let app_s = app_name.to_string();
        let install_s = install_path.to_string();
        if app_s.is_empty() || install_s.trim().is_empty() {
            return QString::from(r#"{"bytes":0,"hasResume":false}"#);
        }
        let install = std::path::PathBuf::from(install_s.trim());
        let (bytes, has_resume) = omikuji_core::gog::inspect_existing_install(&app_s, &install);
        QString::from(&format!(
            r#"{{"bytes":{},"hasResume":{}}}"#,
            bytes, has_resume
        ))
    }

    fn fetch_gog_install_size(
        self: Pin<&mut Self>,
        request_id: &QString,
        app_name: &QString,
    ) {
        let rid = request_id.to_string();
        let app_name_str = app_name.to_string();

        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[gog_size] failed to create tokio runtime: {}", e);
                    omikuji_core::install_sizes::push(omikuji_core::install_sizes::InstallSizeResult {
                        request_id: rid,
                        download_bytes: 0,
                        install_bytes: 0,
                        error: format!("tokio runtime: {}", e),
                    });
                    return;
                }
            };
            let result = rt.block_on(async {
                omikuji_core::gog::fetch_install_size(&app_name_str).await
            });

            let pushed = match result {
                Ok(size) => omikuji_core::install_sizes::InstallSizeResult {
                    request_id: rid,
                    download_bytes: size.download_bytes,
                    install_bytes: size.install_bytes,
                    error: String::new(),
                },
                Err(e) => {
                    eprintln!("[gog_size] error: {}", e);
                    omikuji_core::install_sizes::InstallSizeResult {
                        request_id: rid,
                        download_bytes: 0,
                        install_bytes: 0,
                        error: format!("{}", e),
                    }
                }
            };
            omikuji_core::install_sizes::push(pushed);
        });
    }

    fn gog_import_after_install(
        mut self: Pin<&mut Self>,
        app_name: &QString,
        display_name: &QString,
        prefix_path: &QString,
        runner_version: &QString,
    ) -> QString {
        use omikuji_core::library::{
            default_color, GraphicsConfig, LaunchConfig, Metadata, RunnerConfig, SourceConfig, SystemConfig, WineConfig,
        };

        let app_name_s = app_name.to_string();

        if self.library.game.iter().any(|g| g.metadata.id == app_name_s) {
            eprintln!("[gog_import] already in library: {}", app_name_s);
            return QString::from(&app_name_s);
        }

        let Some(info) = omikuji_core::gog::find_installed_info(&app_name_s) else {
            eprintln!("[gog_import] no install info for {} — leaving library alone", app_name_s);
            return QString::default();
        };

        let display_str = display_name.to_string();
        let title = info
            .title
            .clone()
            .unwrap_or_else(|| if display_str.is_empty() { app_name_s.clone() } else { display_str });
        let prefix_str = prefix_path.to_string();
        let runner_str = runner_version.to_string();

        let mut game = Game {
            metadata: Metadata {
                id: app_name_s.clone(),
                name: title.clone(),
                sort_name: String::new(),
                slug: String::new(),
                exe: info.executable.clone(),
                color: default_color(),
                playtime: 0.0,
                last_played: String::new(),
                banner: String::new(),
                coverart: String::new(),
                icon: String::new(),
                favourite: false,
                categories: vec!["GOG".to_string()],
            },
            source: SourceConfig {
                kind: "gog".to_string(),
                app_id: app_name_s.clone(),
                ..SourceConfig::default()
            },
            runner: RunnerConfig {
                runner_type: "wine".to_string(),
            },
            wine: WineConfig {
                version: runner_str,
                prefix: prefix_str,
                ..WineConfig::default()
            },
            launch: LaunchConfig::default(),
            graphics: GraphicsConfig::default(),
            system: SystemConfig::default(),
        };
        game.seed_from_defaults(&omikuji_core::defaults::Defaults::load());

        let row = self.library.game.len() as i32;

        if let Err(e) = Library::save_game_static(&game) {
            eprintln!("[gog_import] failed to save: {}", e);
            return QString::default();
        }

        let id_for_media = game.metadata.id.clone();
        let name_for_media = game.metadata.name.clone();
        let qt_thread = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            let id_for_refresh = id_for_media.clone();
            media::fetch_media_blocking_with(&id_for_media, &name_for_media, |_| {
                let id_inner = id_for_refresh.clone();
                let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::GameModel>| {
                    let Some(row) = obj.library.game.iter().position(|g| g.metadata.id == id_inner) else {
                        return;
                    };
                    let idx = obj.as_ref().model_index(row as i32, 0, &QModelIndex::default());
                    let roles = cxx_qt_lib::QList::<i32>::default();
                    obj.as_mut().data_changed(&idx, &idx, &roles);
                });
            });
        });

        self.as_mut().begin_insert_rows(&QModelIndex::default(), row, row);
        self.as_mut().rust_mut().get_mut().library.game.push(game);
        let count = self.library.game.len() as i32;
        self.as_mut().set_count(count);
        self.as_mut().end_insert_rows();

        eprintln!("[gog_import] imported '{}' as id '{}'", title, app_name_s);
        QString::from(&app_name_s)
    }

    fn gog_uninstall(self: Pin<&mut Self>, game_id: &QString) -> bool {
        let id = game_id.to_string();
        let Some(game) = self
            .library
            .game
            .iter()
            .find(|g| g.metadata.id == id)
            .cloned()
        else {
            eprintln!("[gog_uninstall] game '{}' not found", id);
            return false;
        };
        if game.source.kind != "gog" || game.source.app_id.is_empty() {
            eprintln!("[gog_uninstall] game '{}' is not a gog entry", id);
            return false;
        }

        let app_id = game.source.app_id.clone();
        let name = game.metadata.name.clone();
        let game_id_owned = game.metadata.id.clone();
        let install_path = omikuji_core::gog::find_installed_info(&app_id)
            .map(|i| i.install_path.clone());

        std::thread::spawn(move || {
            omikuji_core::notifications::info(&name, "Removing GOG game...");
            if let Some(path) = install_path
                && path.exists()
                    && let Err(e) = std::fs::remove_dir_all(&path) {
                        omikuji_core::notifications::error(
                            &name,
                            format!("failed to remove install dir: {}", e),
                        );
                        return;
                    }
            if let Err(e) = omikuji_core::gog::remove_install(&app_id) {
                eprintln!("[gog_uninstall] registry remove failed: {}", e);
            }
            if let Ok(mut lib) = omikuji_core::library::Library::load() {
                let _ = lib.remove_game(&game_id_owned);
            }
            omikuji_core::notifications::success(&name, "Uninstalled");
        });

        true
    }

    fn list_gachas(&self) -> QString {
        let manifests = omikuji_core::gachas::manifest::load_all();
        match serde_json::to_string(&manifests) {
            Ok(s) => QString::from(&s),
            Err(e) => {
                eprintln!("[list_gachas] serialize failed: {}", e);
                QString::from("[]")
            }
        }
    }

    fn ensure_gacha_manifests(self: Pin<&mut Self>) {
        let sender = self.as_ref().qt_thread();
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("[gachas] couldn't build runtime: {}", e);
                    return;
                }
            };
            let fetched = match rt.block_on(omikuji_core::gachas::remote::ensure_all_fetched()) {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("[gachas] fetch failed: {}", e);
                    omikuji_core::notifications::warning(
                        "Gachas",
                        "Couldn't fetch manifests. Existing cached games still work.",
                    );
                    0
                }
            };
            let _ = sender.queue(move |mut m: Pin<&mut qobject::GameModel>| {
                m.as_mut().gacha_manifests_ready(fetched as i32);
            });
        });
    }

    fn get_gacha_manifest(&self, manifest_id: &QString) -> QString {
        let id = manifest_id.to_string();
        match omikuji_core::gachas::manifest::find(&id) {
            Some(m) => match serde_json::to_string(&m) {
                Ok(s) => QString::from(&s),
                Err(e) => {
                    eprintln!("[get_gacha_manifest] serialize failed: {}", e);
                    QString::default()
                }
            },
            None => QString::default(),
        }
    }

    fn gacha_manifest_for_app_id(&self, app_id: &QString) -> QString {
        let aid = app_id.to_string();
        let Some((manifest, edition_id, _voices)) =
            omikuji_core::gachas::strategies::find_for_app_id(&aid)
        else {
            return QString::default();
        };
        QString::from(&format!(
            r#"{{"manifest_id":"{}","edition_id":"{}"}}"#,
            manifest.id, edition_id
        ))
    }

    fn gacha_posters(&self) -> QString {
        let manifests = omikuji_core::gachas::manifest::load_all();
        let mut map = serde_json::Map::new();
        for m in &manifests {
            let url = omikuji_core::gachas::strategies::resolve_poster(m);
            map.insert(m.id.clone(), serde_json::Value::String(url));
        }
        QString::from(&serde_json::Value::Object(map).to_string())
    }

    fn gacha_resolve_poster(&self, manifest_id: &QString) -> QString {
        let id = manifest_id.to_string();
        let Some(m) = omikuji_core::gachas::manifest::find(&id) else {
            return QString::default();
        };
        QString::from(&omikuji_core::gachas::strategies::resolve_poster(&m))
    }

    fn fetch_gacha_install_size(
        self: Pin<&mut Self>,
        request_id: &QString,
        manifest_id: &QString,
        edition_id: &QString,
        voices_csv: &QString,
    ) {
        let rid = request_id.to_string();
        let mid = manifest_id.to_string();
        let eid = edition_id.to_string();
        let voices_str = voices_csv.to_string();

        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[gacha_size] tokio: {}", e);
                    omikuji_core::install_sizes::push(
                        omikuji_core::install_sizes::InstallSizeResult {
                            request_id: rid,
                            download_bytes: 0,
                            install_bytes: 0,
                            error: format!("runtime: {}", e),
                        },
                    );
                    return;
                }
            };

            let pushed = rt.block_on(async move {
                let Some(manifest) = omikuji_core::gachas::manifest::find(&mid) else {
                    return omikuji_core::install_sizes::InstallSizeResult {
                        request_id: rid.clone(),
                        download_bytes: 0,
                        install_bytes: 0,
                        error: format!("unknown manifest: {}", mid),
                    };
                };
                let voices: Vec<String> = voices_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                match omikuji_core::gachas::strategies::fetch_install_size(&manifest, &eid, &voices).await {
                    Ok(sz) => omikuji_core::install_sizes::InstallSizeResult {
                        request_id: rid,
                        download_bytes: sz.download_bytes,
                        install_bytes: sz.install_bytes,
                        error: String::new(),
                    },
                    Err(e) => {
                        eprintln!("[gacha_size] {}: {}", mid, e);
                        omikuji_core::install_sizes::InstallSizeResult {
                            request_id: rid,
                            download_bytes: 0,
                            install_bytes: 0,
                            error: e.to_string(),
                        }
                    }
                }
            });

            omikuji_core::install_sizes::push(pushed);
        });
    }

    fn gacha_check_existing_install(
        &self,
        manifest_id: &QString,
        edition_id: &QString,
        install_path: &QString,
        temp_path: &QString,
    ) -> QString {
        let mid = manifest_id.to_string();
        let eid = edition_id.to_string();
        let path_s = install_path.to_string();
        let temp_s = temp_path.to_string();
        if path_s.trim().is_empty() {
            return QString::from(r#"{"bytes":0,"segments":0,"has_install":false}"#);
        }
        let Some(manifest) = omikuji_core::gachas::manifest::find(&mid) else {
            return QString::from(r#"{"bytes":0,"segments":0,"has_install":false}"#);
        };
        let install = std::path::PathBuf::from(path_s.trim());
        let temp = if temp_s.trim().is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(temp_s.trim()))
        };
        let info = omikuji_core::gachas::strategies::inspect_existing(
            &manifest,
            &eid,
            &install,
            temp.as_deref(),
        );
        QString::from(&format!(
            r#"{{"bytes":{},"segments":{},"has_install":{}}}"#,
            info.scratch_bytes, info.segments, info.has_install
        ))
    }

    fn gacha_import_after_install(
        mut self: Pin<&mut Self>,
        manifest_id: &QString,
        edition_id: &QString,
        display_name: &QString,
        install_path: &QString,
        runner_version: &QString,
        prefix_path: &QString,
    ) -> QString {
        use omikuji_core::library::{
            default_color, GraphicsConfig, LaunchConfig, Metadata, RunnerConfig, SourceConfig,
            SystemConfig, WineConfig,
        };

        let mid = manifest_id.to_string();
        let eid = edition_id.to_string();
        let install_s = install_path.to_string();
        let display_s = display_name.to_string();
        let prefix_s = prefix_path.to_string();
        let runner_s = runner_version.to_string();

        let Some(manifest) = omikuji_core::gachas::manifest::find(&mid) else {
            eprintln!("[gacha_import] unknown manifest: {}", mid);
            return QString::default();
        };
        let Some(edition) = manifest.editions.iter().find(|e| e.id == eid) else {
            eprintln!("[gacha_import] unknown edition '{}' for '{}'", eid, mid);
            return QString::default();
        };
        let app_id = omikuji_core::gachas::strategies::build_app_id(&manifest, &eid, &[]);

        if self.library.game.iter().any(|g| {
            g.source.kind == "gacha" && g.source.app_id == app_id
        }) {
            eprintln!("[gacha_import] already in library: {}", app_id);
            return QString::default();
        }

        let exe = std::path::Path::new(&install_s).join(&edition.exe_name);
        let category = if manifest.category.is_empty() {
            "Gacha".to_string()
        } else {
            manifest.category.clone()
        };
        let game_id = omikuji_core::library::generate_id();

        let mut game = Game {
            metadata: Metadata {
                id: game_id.clone(),
                name: display_s.clone(),
                sort_name: String::new(),
                slug: String::new(),
                exe,
                color: default_color(),
                playtime: 0.0,
                last_played: String::new(),
                banner: String::new(),
                coverart: String::new(),
                icon: String::new(),
                favourite: false,
                categories: vec![category],
            },
            source: SourceConfig {
                kind: "gacha".to_string(),
                app_id: app_id.clone(),
                patch: manifest.launch_patch.clone(),
                ..SourceConfig::default()
            },
            runner: RunnerConfig {
                runner_type: "wine".to_string(),
            },
            wine: WineConfig {
                version: runner_s,
                prefix: prefix_s,
                ..WineConfig::default()
            },
            launch: LaunchConfig::default(),
            graphics: GraphicsConfig::default(),
            system: SystemConfig::default(),
        };
        game.seed_from_defaults(&omikuji_core::defaults::Defaults::load());

        let row = self.library.game.len() as i32;

        if let Err(e) = Library::save_game_static(&game) {
            eprintln!("[gacha_import] failed to save: {}", e);
            return QString::default();
        }

        let id_for_media = game.metadata.id.clone();
        let name_for_media = game.metadata.name.clone();
        let qt_thread = self.as_mut().qt_thread();
        std::thread::spawn(move || {
            let id_for_refresh = id_for_media.clone();
            media::fetch_media_blocking_with(&id_for_media, &name_for_media, |_| {
                let id_inner = id_for_refresh.clone();
                let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::GameModel>| {
                    let Some(row) = obj.library.game.iter().position(|g| g.metadata.id == id_inner) else {
                        return;
                    };
                    let idx = obj.as_ref().model_index(row as i32, 0, &QModelIndex::default());
                    let roles = cxx_qt_lib::QList::<i32>::default();
                    obj.as_mut().data_changed(&idx, &idx, &roles);
                });
            });
        });

        self.as_mut().begin_insert_rows(&QModelIndex::default(), row, row);
        self.as_mut().rust_mut().get_mut().library.game.push(game);
        let count = self.library.game.len() as i32;
        self.as_mut().set_count(count);
        self.as_mut().end_insert_rows();

        eprintln!(
            "[gacha_import] imported '{}' ({}) as id '{}'",
            display_s, app_id, game_id
        );
        QString::from(&game_id)
    }

    fn epic_uninstall(self: Pin<&mut Self>, game_id: &QString) -> bool {
        let id = game_id.to_string();
        let Some(game) = self
            .library
            .game
            .iter()
            .find(|g| g.metadata.id == id)
            .cloned()
        else {
            eprintln!("[epic_uninstall] game '{}' not found", id);
            return false;
        };
        if game.source.kind != "epic" || game.source.app_id.is_empty() {
            eprintln!("[epic_uninstall] game '{}' is not an epic entry", id);
            return false;
        }

        let app_id = game.source.app_id.clone();
        let name = game.metadata.name.clone();
        let game_id_owned = game.metadata.id.clone();

        std::thread::spawn(move || {
            omikuji_core::notifications::info(&name, "Uninstalling via legendary...");

            let Some(legendary_bin) = omikuji_core::downloads::legendary::find_legendary() else {
                omikuji_core::notifications::error(&name, "legendary not found");
                return;
            };

            let result = std::process::Command::new(&legendary_bin)
                .arg("-y")
                .arg("uninstall")
                .arg(&app_id)
                .output();

            match result {
                Ok(out) if out.status.success() => {
                    // library watcher picks up the toml deletion ~500ms later
                    if let Ok(mut lib) = omikuji_core::library::Library::load() {
                        let _ = lib.remove_game(&game_id_owned);
                    }
                    omikuji_core::notifications::success(&name, "Uninstalled");
                }
                Ok(out) => {
                    let err = String::from_utf8_lossy(&out.stderr);
                    omikuji_core::notifications::error(
                        &name,
                        format!("legendary uninstall failed: {}", err.trim()),
                    );
                }
                Err(e) => {
                    omikuji_core::notifications::error(
                        &name,
                        format!("couldn't run legendary: {}", e),
                    );
                }
            }
        });

        true
    }

    fn epic_toggle_overlay(
        mut self: Pin<&mut Self>,
        game_id: &QString,
        enable: bool,
    ) -> bool {
        let id = game_id.to_string();
        let Some(idx) = self.library.game.iter().position(|g| g.metadata.id == id) else {
            eprintln!("[epic_overlay] game '{}' not found", id);
            return false;
        };
        if !self.library.game[idx].is_epic() {
            eprintln!("[epic_overlay] game '{}' is not epic", id);
            return false;
        }

        let (game_name, prefix) = {
            let game = &mut self.as_mut().rust_mut().get_mut().library.game[idx];
            game.source.eos_overlay = enable;
            let _ = Library::save_game_static(game);
            (
                game.metadata.name.clone(),
                omikuji_core::launch::resolve_prefix(game),
            )
        };

        let id_for_thread = id;
        std::thread::spawn(move || {
            use omikuji_core::epic::eos_overlay;
            use omikuji_core::notifications as notif;

            let verb = if enable { "Enabling" } else { "Disabling" };
            notif::info("EOS Overlay", format!("{} for {}…", verb, game_name));

            let result = if enable {
                eos_overlay::enable(&prefix)
            } else {
                eos_overlay::disable(&prefix)
            };

            match result {
                Ok(_) => {
                    let verb = if enable { "Enabled" } else { "Disabled" };
                    notif::success("EOS Overlay", format!("{} for {}", verb, game_name));
                }
                Err(e) => {
                    notif::error("EOS Overlay", format!("{} failed: {}", verb, e));
                    // roll back the persisted flag so the ui toggle re-syncs to the real state
                    if let Ok(Some(mut game)) = omikuji_core::library::Library::load_game_by_id(&id_for_thread) {
                        game.source.eos_overlay = !enable;
                        let _ = omikuji_core::library::Library::save_game_static(&game);
                    }
                }
            }
        });

        true
    }

    fn epic_overlay_is_installed(&self) -> bool {
        omikuji_core::epic::eos_overlay::is_installed()
    }

    fn epic_set_cloud_saves(
        mut self: Pin<&mut Self>,
        game_id: &QString,
        enable: bool,
    ) -> bool {
        let id = game_id.to_string();
        let Some(idx) = self.library.game.iter().position(|g| g.metadata.id == id) else {
            eprintln!("[epic_cloud] game '{}' not found", id);
            return false;
        };
        if !self.library.game[idx].is_epic() {
            eprintln!("[epic_cloud] game '{}' is not epic", id);
            return false;
        }

        // persist the flag first; only probe legendary if save_path is still empty
        let (game_name, should_probe, game_clone) = {
            let game = &mut self.as_mut().rust_mut().get_mut().library.game[idx];
            let needs_probe = enable && game.source.save_path.is_empty();
            game.source.cloud_saves = enable;
            let _ = Library::save_game_static(game);
            (game.metadata.name.clone(), needs_probe, game.clone())
        };

        if !should_probe {
            return true;
        }

        let id_for_thread = id;
        std::thread::spawn(move || {
            use omikuji_core::notifications as notif;

            notif::info(
                "Cloud Saves",
                format!("Discovering save path for {}…", game_name),
            );

            match omikuji_core::epic::discover_save_path(&game_clone) {
                Ok(path) if !path.is_empty() => {
                    if let Ok(Some(mut game)) = omikuji_core::library::Library::load_game_by_id(&id_for_thread) {
                        game.source.save_path = path.clone();
                        let _ = omikuji_core::library::Library::save_game_static(&game);
                    }
                    notif::success("Cloud Saves", format!("Save path resolved: {}", path));
                }
                Ok(_) => {
                    notif::warning(
                        "Cloud Saves",
                        "No cloud save path found — this game may not support Epic cloud saves. You can enter one manually below.",
                    );
                }
                Err(e) => {
                    notif::error("Cloud Saves", format!("Discovery failed: {}", e));
                }
            }
        });

        true
    }
}

// flattened update info passed from gacha backends to the launch hook
struct GachaUpdateInfo {
    from_version: String,
    to_version: String,
    download_size: u64,
    can_diff: bool,
}

// launch_game is called from the Qt event loop, which already runs inside the
// #[tokio::main] runtime. building a second runtime on that thread panics
// ("cannot start a runtime from within a runtime"). a plain os thread gives us a clean context to block_on from
fn blocking_check_gacha_update(app_id: &str) -> Option<GachaUpdateInfo> {
    let aid = app_id.to_string();
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("[launch] update check: runtime build failed: {}", e);
                return None;
            }
        };
        rt.block_on(async {
            let (manifest, edition_id, _voices) =
                omikuji_core::gachas::strategies::find_for_app_id(&aid)?;
            let info =
                omikuji_core::gachas::strategies::check_for_update(&manifest, &edition_id).await?;
            Some(GachaUpdateInfo {
                from_version: info.from_version,
                to_version: info.to_version,
                download_size: info.download_size,
                can_diff: info.can_diff,
            })
        })
    })
    .join()
    .ok()
    .flatten()
}
