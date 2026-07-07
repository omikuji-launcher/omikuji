// apply_* prefix becuase cxx-qt already generates setXxx for qproperties;
// using the same name would collide. each apply_* sets the property then atonmically persists

use cxx_qt::{CxxQtType, Threading};
use omikuji_core::fs_watcher::FileWatcher;
use omikuji_core::ui_settings::{
    BehaviorSettings, CategoryEntry, ConsoleModeSettings, DisplaySettings, KvSet, LibrarySettings,
    NavSettings, TabsSettings, ThemeSettings, UiSettings, ui_settings_path,
};
use std::collections::BTreeMap;
use std::pin::Pin;
use std::time::{Duration, Instant};

unsafe extern "C" {
    fn omikuji_set_app_font(family: *const std::os::raw::c_char);
    fn omikuji_available_languages_json() -> *const std::os::raw::c_char;
}

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(f64, card_zoom, cxx_name = "cardZoom")]
        #[qproperty(i32, card_spacing, cxx_name = "cardSpacing")]
        #[qproperty(bool, card_elevation, cxx_name = "cardElevation")]
        #[qproperty(bool, unload_store_pages, cxx_name = "unloadStorePages")]
        #[qproperty(bool, show_gachas, cxx_name = "showGachas")]
        #[qproperty(bool, show_epic, cxx_name = "showEpic")]
        #[qproperty(bool, show_gog, cxx_name = "showGog")]
        #[qproperty(bool, show_steam, cxx_name = "showSteam")]
        #[qproperty(i32, nav_width, cxx_name = "navWidth")]
        #[qproperty(bool, nav_collapsed, cxx_name = "navCollapsed")]
        #[qproperty(bool, minimize_on_launch, cxx_name = "minimizeOnLaunch")]
        #[qproperty(bool, save_game_logs, cxx_name = "saveGameLogs")]
        #[qproperty(bool, double_click_launches, cxx_name = "doubleClickLaunches")]
        #[qproperty(bool, auto_check_epic_updates_on_launch, cxx_name = "autoCheckEpicUpdatesOnLaunch")]
        #[qproperty(bool, auto_check_gog_updates_on_launch, cxx_name = "autoCheckGogUpdatesOnLaunch")]
        #[qproperty(bool, auto_check_updates_on_boot, cxx_name = "autoCheckUpdatesOnBoot")]
        #[qproperty(bool, show_tray_icon, cxx_name = "showTrayIcon")]
        #[qproperty(bool, discord_rpc, cxx_name = "discordRpc")]
        #[qproperty(f64, ui_scale, cxx_name = "uiScale")]
        #[qproperty(bool, muted_icons, cxx_name = "mutedIcons")]
        #[qproperty(QString, card_flow, cxx_name = "cardFlow")]
        #[qproperty(QString, card_sort, cxx_name = "cardSort")]
        #[qproperty(QString, console_background, cxx_name = "consoleBackground")]
        #[qproperty(bool, follow_system_colors, cxx_name = "followSystemColors")]
        #[qproperty(bool, follow_system_font, cxx_name = "followSystemFont")]
        #[qproperty(QString, font_family, cxx_name = "fontFamily")]
        #[qproperty(bool, fill_fields, cxx_name = "fillFields")]
        #[qproperty(f64, radius_scale, cxx_name = "radiusScale")]
        #[qproperty(QString, language)]
        type UiSettingsBridge = super::UiSettingsRust;
    }

    unsafe extern "RustQt" {
        #[qsignal]
        #[cxx_name = "categoriesChanged"]
        fn categories_changed(self: Pin<&mut UiSettingsBridge>);

        #[qsignal]
        #[cxx_name = "envSetsChanged"]
        fn env_sets_changed(self: Pin<&mut UiSettingsBridge>);

        #[qsignal]
        #[cxx_name = "dllSetsChanged"]
        fn dll_sets_changed(self: Pin<&mut UiSettingsBridge>);

        #[qsignal]
        #[cxx_name = "themeChanged"]
        fn theme_changed(self: Pin<&mut UiSettingsBridge>);
    }

    // needed so the watcher bg thread can queue back to the ui thread
    impl cxx_qt::Threading for UiSettingsBridge {}

    unsafe extern "RustQt" {
        #[qinvokable]
        #[cxx_name = "applyCardZoom"]
        fn apply_card_zoom(self: Pin<&mut UiSettingsBridge>, value: f64);

        #[qinvokable]
        #[cxx_name = "applyCardSpacing"]
        fn apply_card_spacing(self: Pin<&mut UiSettingsBridge>, value: i32);

        #[qinvokable]
        #[cxx_name = "applyCardElevation"]
        fn apply_card_elevation(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyUnloadStorePages"]
        fn apply_unload_store_pages(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyShowGachas"]
        fn apply_show_gachas(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyShowEpic"]
        fn apply_show_epic(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyShowGog"]
        fn apply_show_gog(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyShowSteam"]
        fn apply_show_steam(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyNavWidth"]
        fn apply_nav_width(self: Pin<&mut UiSettingsBridge>, value: i32);

        #[qinvokable]
        #[cxx_name = "applyNavCollapsed"]
        fn apply_nav_collapsed(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyMinimizeOnLaunch"]
        fn apply_minimize_on_launch(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applySaveGameLogs"]
        fn apply_save_game_logs(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyDoubleClickLaunches"]
        fn apply_double_click_launches(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyAutoCheckEpicUpdatesOnLaunch"]
        fn apply_auto_check_epic_updates_on_launch(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyAutoCheckGogUpdatesOnLaunch"]
        fn apply_auto_check_gog_updates_on_launch(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyAutoCheckUpdatesOnBoot"]
        fn apply_auto_check_updates_on_boot(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyShowTrayIcon"]
        fn apply_show_tray_icon(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyDiscordRpc"]
        fn apply_discord_rpc(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyUiScale"]
        fn apply_ui_scale(self: Pin<&mut UiSettingsBridge>, value: f64);

        #[qinvokable]
        #[cxx_name = "applyMutedIcons"]
        fn apply_muted_icons(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyCardFlow"]
        fn apply_card_flow(self: Pin<&mut UiSettingsBridge>, value: &QString);

        #[qinvokable]
        #[cxx_name = "applyCardSort"]
        fn apply_card_sort(self: Pin<&mut UiSettingsBridge>, value: &QString);

        #[qinvokable]
        #[cxx_name = "applyConsoleBackground"]
        fn apply_console_background(self: Pin<&mut UiSettingsBridge>, value: &QString);

        #[qinvokable]
        #[cxx_name = "reloadFromDisk"]
        fn reload_from_disk(self: Pin<&mut UiSettingsBridge>);

        #[qinvokable]
        #[cxx_name = "categoriesJson"]
        fn categories_json(self: &UiSettingsBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "applyCategoriesJson"]
        fn apply_categories_json(self: Pin<&mut UiSettingsBridge>, json: &QString);

        #[qinvokable]
        #[cxx_name = "envSetsJson"]
        fn env_sets_json(self: &UiSettingsBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "applyEnvSetsJson"]
        fn apply_env_sets_json(self: Pin<&mut UiSettingsBridge>, json: &QString);

        #[qinvokable]
        #[cxx_name = "dllSetsJson"]
        fn dll_sets_json(self: &UiSettingsBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "applyDllSetsJson"]
        fn apply_dll_sets_json(self: Pin<&mut UiSettingsBridge>, json: &QString);

        #[qinvokable]
        #[cxx_name = "initWatcher"]
        fn init_watcher(self: Pin<&mut UiSettingsBridge>);

        #[qinvokable]
        #[cxx_name = "availableIconsJson"]
        fn available_icons_json(self: &UiSettingsBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "applyFollowSystemColors"]
        fn apply_follow_system_colors(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyFollowSystemFont"]
        fn apply_follow_system_font(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyFontFamily"]
        fn apply_font_family(self: Pin<&mut UiSettingsBridge>, value: &QString);

        #[qinvokable]
        #[cxx_name = "colorOverride"]
        fn color_override(self: &UiSettingsBridge, token: &QString) -> QString;

        #[qinvokable]
        #[cxx_name = "setColorOverride"]
        fn set_color_override(self: Pin<&mut UiSettingsBridge>, token: &QString, hex: &QString);

        #[qinvokable]
        #[cxx_name = "overridesJson"]
        fn overrides_json(self: &UiSettingsBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "availableFontsJson"]
        fn available_fonts_json(self: &UiSettingsBridge) -> QString;

        #[qinvokable]
        #[cxx_name = "applyLanguage"]
        fn apply_language(self: Pin<&mut UiSettingsBridge>, value: &QString);

        #[qinvokable]
        #[cxx_name = "availableLanguagesJson"]
        fn available_languages_json(self: &UiSettingsBridge) -> QString;
    }
}

include!(concat!(env!("OUT_DIR"), "/icon_names.rs"));

pub struct UiSettingsRust {
    pub card_zoom: f64,
    pub card_spacing: i32,
    pub card_elevation: bool,
    pub unload_store_pages: bool,
    pub show_gachas: bool,
    pub show_epic: bool,
    pub show_gog: bool,
    pub show_steam: bool,
    pub nav_width: i32,
    pub nav_collapsed: bool,
    pub minimize_on_launch: bool,
    pub save_game_logs: bool,
    pub auto_check_epic_updates_on_launch: bool,
    pub auto_check_gog_updates_on_launch: bool,
    pub auto_check_updates_on_boot: bool,
    pub show_tray_icon: bool,
    pub discord_rpc: bool,
    pub double_click_launches: bool,
    pub ui_scale: f64,
    pub muted_icons: bool,
    pub card_flow: cxx_qt_lib::QString,
    pub card_sort: cxx_qt_lib::QString,
    pub console_background: cxx_qt_lib::QString,
    pub follow_system_colors: bool,
    pub follow_system_font: bool,
    pub font_family: cxx_qt_lib::QString,
    pub fill_fields: bool,
    pub radius_scale: f64,
    pub language: cxx_qt_lib::QString,
    pub color_overrides: BTreeMap<String, String>,
    pub categories: Vec<CategoryEntry>,
    pub env_sets: Vec<KvSet>,
    pub dll_sets: Vec<KvSet>,
    pub watcher: Option<FileWatcher>,
    pub suppress_reload_until: Option<Instant>,
}

impl Default for UiSettingsRust {
    fn default() -> Self {
        let s = UiSettings::load();
        omikuji_core::discord::set_enabled(s.behavior.discord_rpc);
        Self::from_settings(&s)
    }
}

impl UiSettingsRust {
    fn from_settings(s: &UiSettings) -> Self {
        Self {
            card_zoom: s.library.card_zoom,
            card_spacing: s.library.card_spacing,
            card_elevation: s.library.card_elevation,
            unload_store_pages: s.library.unload_store_pages,
            show_gachas: s.tabs.show_gachas,
            show_epic: s.tabs.show_epic,
            show_gog: s.tabs.show_gog,
            show_steam: s.tabs.show_steam,
            nav_width: s.nav.width,
            nav_collapsed: s.nav.collapsed,
            minimize_on_launch: s.behavior.minimize_on_launch,
            save_game_logs: s.behavior.save_game_logs,
            auto_check_epic_updates_on_launch: s.behavior.auto_check_epic_updates_on_launch,
            auto_check_gog_updates_on_launch: s.behavior.auto_check_gog_updates_on_launch,
            auto_check_updates_on_boot: s.behavior.auto_check_updates_on_boot,
            show_tray_icon: s.behavior.show_tray_icon,
            discord_rpc: s.behavior.discord_rpc,
            double_click_launches: s.behavior.double_click_launches,
            ui_scale: s.display.scale,
            muted_icons: s.display.muted_icons,
            card_flow: cxx_qt_lib::QString::from(&s.display.card_flow),
            card_sort: cxx_qt_lib::QString::from(&s.display.card_sort),
            console_background: cxx_qt_lib::QString::from(&s.console_mode.background),
            follow_system_colors: s.theme.follow_system_colors,
            follow_system_font: s.theme.follow_system_font,
            font_family: cxx_qt_lib::QString::from(&s.theme.font_family),
            fill_fields: s.theme.fill_fields,
            radius_scale: s.theme.radius_scale,
            language: cxx_qt_lib::QString::from(&s.language),
            color_overrides: s.theme.colors.clone(),
            categories: s.categories.clone(),
            env_sets: s.env_sets.clone(),
            dll_sets: s.dll_sets.clone(),
            watcher: None,
            suppress_reload_until: None,
        }
    }
}

macro_rules! kv_json_accessor {
    ($get:ident, $set:ident, $field:ident, $ty:ty, $changed:ident, $label:literal) => {
        fn $get(&self) -> cxx_qt_lib::QString {
            let json = serde_json::to_string(&self.$field).unwrap_or_else(|_| "[]".to_string());
            cxx_qt_lib::QString::from(&json)
        }

        fn $set(mut self: Pin<&mut Self>, json: &cxx_qt_lib::QString) {
            match serde_json::from_str::<$ty>(&json.to_string()) {
                Ok(entries) => {
                    self.as_mut().rust_mut().get_mut().$field = entries;
                    self.as_mut().persist();
                    self.as_mut().$changed();
                }
                Err(e) => tracing::error!("bad {} json: {}", $label, e),
            }
        }
    };
}

macro_rules! apply_setting {
    ($apply:ident, $set:ident, $ty:ty) => {
        fn $apply(mut self: Pin<&mut Self>, value: $ty) {
            self.as_mut().$set(value);
            self.persist();
        }
    };
    (qstr $apply:ident, $set:ident) => {
        fn $apply(mut self: Pin<&mut Self>, value: &cxx_qt_lib::QString) {
            self.as_mut().$set(value.clone());
            self.persist();
        }
    };
}

impl qobject::UiSettingsBridge {
    fn snapshot(&self) -> UiSettings {
        UiSettings {
            language: self.language.to_string(),
            library: LibrarySettings {
                card_zoom: self.card_zoom,
                card_spacing: self.card_spacing,
                card_elevation: self.card_elevation,
                unload_store_pages: self.unload_store_pages,
            },
            tabs: TabsSettings {
                show_gachas: self.show_gachas,
                show_epic: self.show_epic,
                show_gog: self.show_gog,
                show_steam: self.show_steam,
            },
            nav: NavSettings {
                width: self.nav_width,
                collapsed: self.nav_collapsed,
            },
            behavior: BehaviorSettings {
                minimize_on_launch: self.minimize_on_launch,
                save_game_logs: self.save_game_logs,
                auto_check_epic_updates_on_launch: self.auto_check_epic_updates_on_launch,
                auto_check_gog_updates_on_launch: self.auto_check_gog_updates_on_launch,
                auto_check_updates_on_boot: self.auto_check_updates_on_boot,
                show_tray_icon: self.show_tray_icon,
                discord_rpc: self.discord_rpc,
                double_click_launches: self.double_click_launches,
            },
            display: DisplaySettings {
                scale: self.ui_scale,
                muted_icons: self.muted_icons,
                card_flow: self.card_flow.to_string(),
                card_sort: self.card_sort.to_string(),
            },
            theme: ThemeSettings {
                follow_system_colors: self.follow_system_colors,
                follow_system_font: self.follow_system_font,
                font_family: self.font_family.to_string(),
                colors: self.color_overrides.clone(),
                fill_fields: self.fill_fields,
                radius_scale: self.radius_scale,
            },
            console_mode: ConsoleModeSettings {
                background: self.console_background.to_string(),
                active: UiSettings::load().console_mode.active,
            },
            categories: self.categories.clone(),
            env_sets: self.env_sets.clone(),
            dll_sets: self.dll_sets.clone(),
        }
    }

    fn persist(mut self: Pin<&mut Self>) {
        // 600ms covers the 150ms debounce plus qt_thread hop slack
        self.as_mut().rust_mut().get_mut().suppress_reload_until =
            Some(Instant::now() + Duration::from_millis(600));
        if let Err(e) = self.snapshot().save() {
            tracing::error!("save failed: {}", e);
        }
    }

    apply_setting!(apply_card_zoom, set_card_zoom, f64);
    apply_setting!(apply_card_spacing, set_card_spacing, i32);
    apply_setting!(apply_card_elevation, set_card_elevation, bool);
    apply_setting!(apply_unload_store_pages, set_unload_store_pages, bool);
    apply_setting!(apply_show_gachas, set_show_gachas, bool);
    apply_setting!(apply_show_epic, set_show_epic, bool);
    apply_setting!(apply_show_gog, set_show_gog, bool);
    apply_setting!(apply_show_steam, set_show_steam, bool);
    apply_setting!(apply_nav_width, set_nav_width, i32);
    apply_setting!(apply_nav_collapsed, set_nav_collapsed, bool);
    apply_setting!(apply_minimize_on_launch, set_minimize_on_launch, bool);
    apply_setting!(apply_save_game_logs, set_save_game_logs, bool);
    apply_setting!(apply_double_click_launches, set_double_click_launches, bool);
    apply_setting!(apply_auto_check_epic_updates_on_launch, set_auto_check_epic_updates_on_launch, bool);
    apply_setting!(apply_auto_check_gog_updates_on_launch, set_auto_check_gog_updates_on_launch, bool);
    apply_setting!(apply_auto_check_updates_on_boot, set_auto_check_updates_on_boot, bool);
    apply_setting!(apply_show_tray_icon, set_show_tray_icon, bool);

    fn apply_discord_rpc(mut self: Pin<&mut Self>, value: bool) {
        self.as_mut().set_discord_rpc(value);
        omikuji_core::discord::set_enabled(value);
        self.persist();
    }

    fn apply_ui_scale(mut self: Pin<&mut Self>, value: f64) {
        let clamped = value.clamp(0.7, 2.0);
        self.as_mut().set_ui_scale(clamped);
        self.persist();
    }

    apply_setting!(apply_muted_icons, set_muted_icons, bool);

    fn apply_card_flow(mut self: Pin<&mut Self>, value: &cxx_qt_lib::QString) {
        let v = value.to_string();
        let allowed = matches!(v.as_str(), "left" | "center" | "right");
        let final_v = if allowed { value.clone() } else { cxx_qt_lib::QString::from("center") };
        self.as_mut().set_card_flow(final_v);
        self.persist();
    }

    fn apply_card_sort(mut self: Pin<&mut Self>, value: &cxx_qt_lib::QString) {
        let v = value.to_string();
        let allowed = matches!(v.as_str(), "default" | "a-z" | "z-a" | "custom");
        let final_v = if allowed { value.clone() } else { cxx_qt_lib::QString::from("default") };
        self.as_mut().set_card_sort(final_v);
        self.persist();
    }

    apply_setting!(qstr apply_console_background, set_console_background);
    apply_setting!(qstr apply_language, set_language);

    fn reload_from_disk(mut self: Pin<&mut Self>) {
        let s = UiSettings::load();
        self.as_mut().set_card_zoom(s.library.card_zoom);
        self.as_mut().set_card_spacing(s.library.card_spacing);
        self.as_mut().set_card_elevation(s.library.card_elevation);
        self.as_mut().set_unload_store_pages(s.library.unload_store_pages);
        self.as_mut().set_show_gachas(s.tabs.show_gachas);
        self.as_mut().set_show_epic(s.tabs.show_epic);
        self.as_mut().set_show_gog(s.tabs.show_gog);
        self.as_mut().set_show_steam(s.tabs.show_steam);
        self.as_mut().set_nav_width(s.nav.width);
        self.as_mut().set_nav_collapsed(s.nav.collapsed);
        self.as_mut().set_minimize_on_launch(s.behavior.minimize_on_launch);
        self.as_mut().set_save_game_logs(s.behavior.save_game_logs);
        self.as_mut().set_auto_check_epic_updates_on_launch(s.behavior.auto_check_epic_updates_on_launch);
        self.as_mut().set_auto_check_gog_updates_on_launch(s.behavior.auto_check_gog_updates_on_launch);
        self.as_mut().set_auto_check_updates_on_boot(s.behavior.auto_check_updates_on_boot);
        self.as_mut().set_show_tray_icon(s.behavior.show_tray_icon);
        self.as_mut().set_discord_rpc(s.behavior.discord_rpc);
        omikuji_core::discord::set_enabled(s.behavior.discord_rpc);
        self.as_mut().set_double_click_launches(s.behavior.double_click_launches);
        self.as_mut().set_ui_scale(s.display.scale);
        self.as_mut().set_muted_icons(s.display.muted_icons);
        self.as_mut().set_card_flow(cxx_qt_lib::QString::from(&s.display.card_flow));
        self.as_mut().set_card_sort(cxx_qt_lib::QString::from(&s.display.card_sort));
        self.as_mut().set_console_background(cxx_qt_lib::QString::from(&s.console_mode.background));
        self.as_mut().set_follow_system_colors(s.theme.follow_system_colors);
        self.as_mut().set_follow_system_font(s.theme.follow_system_font);
        self.as_mut().set_font_family(cxx_qt_lib::QString::from(&s.theme.font_family));
        self.as_mut().set_fill_fields(s.theme.fill_fields);
        self.as_mut().set_radius_scale(s.theme.radius_scale);
        self.as_mut().set_language(cxx_qt_lib::QString::from(&s.language));
        self.as_mut().rust_mut().get_mut().color_overrides = s.theme.colors;
        self.as_mut().rust_mut().get_mut().categories = s.categories;
        self.as_mut().categories_changed();
        self.as_mut().rust_mut().get_mut().env_sets = s.env_sets;
        self.as_mut().env_sets_changed();
        self.as_mut().rust_mut().get_mut().dll_sets = s.dll_sets;
        self.as_mut().dll_sets_changed();
        self.as_mut().theme_changed();
    }

    kv_json_accessor!(categories_json, apply_categories_json, categories, Vec<CategoryEntry>, categories_changed, "categories");
    kv_json_accessor!(env_sets_json, apply_env_sets_json, env_sets, Vec<KvSet>, env_sets_changed, "env sets");
    kv_json_accessor!(dll_sets_json, apply_dll_sets_json, dll_sets, Vec<KvSet>, dll_sets_changed, "dll sets");

    fn available_icons_json(&self) -> cxx_qt_lib::QString {
        let json = serde_json::to_string(ICON_NAMES).unwrap_or_else(|_| "[]".to_string());
        cxx_qt_lib::QString::from(&json)
    }

    fn apply_follow_system_colors(mut self: Pin<&mut Self>, value: bool) {
        self.as_mut().set_follow_system_colors(value);
        self.as_mut().persist();
        self.as_mut().theme_changed();
    }

    fn apply_follow_system_font(mut self: Pin<&mut Self>, value: bool) {
        self.as_mut().set_follow_system_font(value);
        self.as_mut().persist();
        self.as_mut().apply_effective_font();
        self.as_mut().theme_changed();
    }

    fn apply_font_family(mut self: Pin<&mut Self>, value: &cxx_qt_lib::QString) {
        self.as_mut().set_font_family(value.clone());
        self.as_mut().persist();
        self.as_mut().apply_effective_font();
        self.as_mut().theme_changed();
    }

    fn apply_effective_font(self: Pin<&mut Self>) {
        let family = if self.follow_system_font {
            String::new()
        } else {
            self.font_family.to_string()
        };
        if let Ok(c) = std::ffi::CString::new(family) { unsafe { omikuji_set_app_font(c.as_ptr()) } }
    }

    fn color_override(&self, token: &cxx_qt_lib::QString) -> cxx_qt_lib::QString {
        let key = token.to_string();
        let val = self.color_overrides.get(&key).cloned().unwrap_or_default();
        cxx_qt_lib::QString::from(&val)
    }

    fn set_color_override(mut self: Pin<&mut Self>, token: &cxx_qt_lib::QString, hex: &cxx_qt_lib::QString) {
        let key = token.to_string();
        let val = hex.to_string();
        let map = &mut self.as_mut().rust_mut().get_mut().color_overrides;
        if val.is_empty() {
            map.remove(&key);
        } else {
            map.insert(key, val);
        }
        self.as_mut().persist();
        self.as_mut().theme_changed();
    }

    fn overrides_json(&self) -> cxx_qt_lib::QString {
        let json = serde_json::to_string(&self.color_overrides).unwrap_or_else(|_| "{}".to_string());
        cxx_qt_lib::QString::from(&json)
    }

    fn available_fonts_json(&self) -> cxx_qt_lib::QString {
        let json = serde_json::to_string(&list_system_fonts()).unwrap_or_else(|_| "[]".to_string());
        cxx_qt_lib::QString::from(&json)
    }

    fn available_languages_json(&self) -> cxx_qt_lib::QString {
        let ptr = unsafe { omikuji_available_languages_json() };
        if ptr.is_null() {
            return cxx_qt_lib::QString::from("[]");
        }
        let json = unsafe { std::ffi::CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned();
        cxx_qt_lib::QString::from(&json)
    }

    fn init_watcher(mut self: Pin<&mut Self>) {
        if self.as_ref().rust().watcher.is_some() {
            return;
        }
        let path = ui_settings_path();
        let qt_thread = self.as_mut().qt_thread();
        let watcher = FileWatcher::watch(path, move || {
            let _ = qt_thread.queue(move |mut obj: Pin<&mut qobject::UiSettingsBridge>| {
                let within_window = obj
                    .as_ref()
                    .rust()
                    .suppress_reload_until
                    .map(|until| Instant::now() < until)
                    .unwrap_or(false);
                if within_window {
                    return;
                }
                obj.as_mut().reload_from_disk();
            });
        });
        match watcher {
            Ok(w) => {
                self.as_mut().rust_mut().get_mut().watcher = Some(w);
                tracing::debug!("watching {} via notify", ui_settings_path().display());
            }
            Err(e) => tracing::error!("failed to start watcher: {e}"),
        }
    }
}

fn list_system_fonts() -> Vec<String> {
    let output = std::process::Command::new("fc-list")
        .args([":scalable=true:fontformat=TrueType", "family"])
        .output();
    let Ok(out) = output else { return Vec::new(); };
    if !out.status.success() { return Vec::new(); }
    let mut set = std::collections::BTreeSet::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        if let Some(name) = line.split(',').next() {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                set.insert(trimmed.to_string());
            }
        }
    }
    set.into_iter().collect()
}
