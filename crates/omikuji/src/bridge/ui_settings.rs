// apply_* prefix becuase cxx-qt already generates setXxx for qproperties;
// using the same name would collide. each apply_* sets the property then atonmically persists

use cxx_qt::{CxxQtType, Threading};
use omikuji_core::fs_watcher::FileWatcher;
use omikuji_core::ui_settings::{
    BehaviorSettings, CategoryEntry, ConsoleModeSettings, DisplaySettings, LibrarySettings,
    NavSettings, TabsSettings, UiSettings, ui_settings_path,
};
use std::pin::Pin;
use std::time::{Duration, Instant};

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
        #[qproperty(bool, auto_check_epic_updates_on_launch, cxx_name = "autoCheckEpicUpdatesOnLaunch")]
        #[qproperty(bool, auto_check_gog_updates_on_launch, cxx_name = "autoCheckGogUpdatesOnLaunch")]
        #[qproperty(bool, auto_check_updates_on_boot, cxx_name = "autoCheckUpdatesOnBoot")]
        #[qproperty(f64, ui_scale, cxx_name = "uiScale")]
        #[qproperty(bool, muted_icons, cxx_name = "mutedIcons")]
        #[qproperty(QString, card_flow, cxx_name = "cardFlow")]
        #[qproperty(QString, console_background, cxx_name = "consoleBackground")]
        type UiSettingsBridge = super::UiSettingsRust;
    }

    unsafe extern "RustQt" {
        #[qsignal]
        #[cxx_name = "categoriesChanged"]
        fn categories_changed(self: Pin<&mut UiSettingsBridge>);
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
        #[cxx_name = "applyAutoCheckEpicUpdatesOnLaunch"]
        fn apply_auto_check_epic_updates_on_launch(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyAutoCheckGogUpdatesOnLaunch"]
        fn apply_auto_check_gog_updates_on_launch(self: Pin<&mut UiSettingsBridge>, value: bool);

        #[qinvokable]
        #[cxx_name = "applyAutoCheckUpdatesOnBoot"]
        fn apply_auto_check_updates_on_boot(self: Pin<&mut UiSettingsBridge>, value: bool);

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
        #[cxx_name = "initWatcher"]
        fn init_watcher(self: Pin<&mut UiSettingsBridge>);

        #[qinvokable]
        #[cxx_name = "availableIconsJson"]
        fn available_icons_json(self: &UiSettingsBridge) -> QString;
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
    pub ui_scale: f64,
    pub muted_icons: bool,
    pub card_flow: cxx_qt_lib::QString,
    pub console_background: cxx_qt_lib::QString,
    pub categories: Vec<CategoryEntry>,
    pub watcher: Option<FileWatcher>,
    // skips the watcher echo from our own persist() writes
    pub suppress_reload_until: Option<Instant>,
}

impl Default for UiSettingsRust {
    fn default() -> Self {
        let s = UiSettings::load();
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
            ui_scale: s.display.scale,
            muted_icons: s.display.muted_icons,
            card_flow: cxx_qt_lib::QString::from(&s.display.card_flow),
            console_background: cxx_qt_lib::QString::from(&s.console_mode.background),
            categories: s.categories.clone(),
            watcher: None,
            suppress_reload_until: None,
        }
    }
}

impl qobject::UiSettingsBridge {
    fn snapshot(&self) -> UiSettings {
        UiSettings {
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
            },
            display: DisplaySettings {
                scale: self.ui_scale,
                muted_icons: self.muted_icons,
                card_flow: self.card_flow.to_string(),
            },
            console_mode: ConsoleModeSettings {
                background: self.console_background.to_string(),
                active: UiSettings::load().console_mode.active,
            },
            categories: self.categories.clone(),
        }
    }

    fn persist(mut self: Pin<&mut Self>) {
        // 600ms covers the 150ms debounce plus qt_thread hop slack
        self.as_mut().rust_mut().get_mut().suppress_reload_until =
            Some(Instant::now() + Duration::from_millis(600));
        if let Err(e) = self.snapshot().save() {
            eprintln!("[ui_settings] save failed: {}", e);
        }
    }

    fn apply_card_zoom(mut self: Pin<&mut Self>, value: f64) {
        self.as_mut().set_card_zoom(value);
        self.persist();
    }

    fn apply_card_spacing(mut self: Pin<&mut Self>, value: i32) {
        self.as_mut().set_card_spacing(value);
        self.persist();
    }

    fn apply_card_elevation(mut self: Pin<&mut Self>, value: bool) {
        self.as_mut().set_card_elevation(value);
        self.persist();
    }

    fn apply_unload_store_pages(mut self: Pin<&mut Self>, value: bool) {
        self.as_mut().set_unload_store_pages(value);
        self.persist();
    }

    fn apply_show_gachas(mut self: Pin<&mut Self>, value: bool) {
        self.as_mut().set_show_gachas(value);
        self.persist();
    }

    fn apply_show_epic(mut self: Pin<&mut Self>, value: bool) {
        self.as_mut().set_show_epic(value);
        self.persist();
    }

    fn apply_show_gog(mut self: Pin<&mut Self>, value: bool) {
        self.as_mut().set_show_gog(value);
        self.persist();
    }

    fn apply_show_steam(mut self: Pin<&mut Self>, value: bool) {
        self.as_mut().set_show_steam(value);
        self.persist();
    }

    fn apply_nav_width(mut self: Pin<&mut Self>, value: i32) {
        self.as_mut().set_nav_width(value);
        self.persist();
    }

    fn apply_nav_collapsed(mut self: Pin<&mut Self>, value: bool) {
        self.as_mut().set_nav_collapsed(value);
        self.persist();
    }

    fn apply_minimize_on_launch(mut self: Pin<&mut Self>, value: bool) {
        self.as_mut().set_minimize_on_launch(value);
        self.persist();
    }

    fn apply_save_game_logs(mut self: Pin<&mut Self>, value: bool) {
        self.as_mut().set_save_game_logs(value);
        self.persist();
    }

    fn apply_auto_check_epic_updates_on_launch(mut self: Pin<&mut Self>, value: bool) {
        self.as_mut().set_auto_check_epic_updates_on_launch(value);
        self.persist();
    }

    fn apply_auto_check_gog_updates_on_launch(mut self: Pin<&mut Self>, value: bool) {
        self.as_mut().set_auto_check_gog_updates_on_launch(value);
        self.persist();
    }

    fn apply_auto_check_updates_on_boot(mut self: Pin<&mut Self>, value: bool) {
        self.as_mut().set_auto_check_updates_on_boot(value);
        self.persist();
    }

    fn apply_ui_scale(mut self: Pin<&mut Self>, value: f64) {
        let clamped = value.clamp(0.7, 2.0);
        self.as_mut().set_ui_scale(clamped);
        self.persist();
    }

    fn apply_muted_icons(mut self: Pin<&mut Self>, value: bool) {
        self.as_mut().set_muted_icons(value);
        self.persist();
    }

    fn apply_card_flow(mut self: Pin<&mut Self>, value: &cxx_qt_lib::QString) {
        let v = value.to_string();
        let allowed = matches!(v.as_str(), "left" | "center" | "right");
        let final_v = if allowed { value.clone() } else { cxx_qt_lib::QString::from("center") };
        self.as_mut().set_card_flow(final_v);
        self.persist();
    }

    fn apply_console_background(mut self: Pin<&mut Self>, value: &cxx_qt_lib::QString) {
        self.as_mut().set_console_background(value.clone());
        self.persist();
    }

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
        self.as_mut().set_ui_scale(s.display.scale);
        self.as_mut().set_muted_icons(s.display.muted_icons);
        self.as_mut().set_card_flow(cxx_qt_lib::QString::from(&s.display.card_flow));
        self.as_mut().set_console_background(cxx_qt_lib::QString::from(&s.console_mode.background));
        self.as_mut().rust_mut().get_mut().categories = s.categories;
        self.as_mut().categories_changed();
    }

    fn categories_json(&self) -> cxx_qt_lib::QString {
        let json = serde_json::to_string(&self.categories).unwrap_or_else(|_| "[]".to_string());
        cxx_qt_lib::QString::from(&json)
    }

    fn apply_categories_json(mut self: Pin<&mut Self>, json: &cxx_qt_lib::QString) {
        let s = json.to_string();
        match serde_json::from_str::<Vec<CategoryEntry>>(&s) {
            Ok(entries) => {
                self.as_mut().rust_mut().get_mut().categories = entries;
                self.as_mut().persist();
                self.as_mut().categories_changed();
            }
            Err(e) => eprintln!("[ui_settings] bad categories json: {e}"),
        }
    }

    fn available_icons_json(&self) -> cxx_qt_lib::QString {
        let json = serde_json::to_string(ICON_NAMES).unwrap_or_else(|_| "[]".to_string());
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
                eprintln!("[ui_settings] watching {} via notify", ui_settings_path().display());
            }
            Err(e) => eprintln!("[ui_settings] failed to start watcher: {e}"),
        }
    }
}
