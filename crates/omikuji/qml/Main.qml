import QtQuick
import QtQml
import QtQuick.Controls
import QtQuick.Layouts

import omikuji 1.0
import "components"
import "components/categories"
import "components/dialogs"
import "components/downloads"
import "components/library"
import "components/navigation"
import "components/store"
import "components/modals"
import "components/popups"

/*
yes this is cursed. yes it works. we ballin
*/

ApplicationWindow {
    id: root

    width: 1060
    height: 640
    minimumWidth: 780
    minimumHeight: 500
    visible: true
    title: "Omikuji"
    color: theme.navBg

    flags: Qt.Window

    Theme {
        id: theme
        mutedIcons: uiSettings.mutedIcons
        filledIcons: uiSettings.filledIcons
        followSystemColors: uiSettings.followSystemColors
        followSystemFont: uiSettings.followSystemFont
        fontFamily: uiSettings.fontFamily
        fillFields: uiSettings.fillFields
        radiusScale: uiSettings.radiusScale
        uiScale: root.uiScale
    }

    Connections {
        target: uiSettings
        function onThemeChanged() {
            theme.overrides = JSON.parse(uiSettings.overridesJson())
        }
        function onCardSortChanged() {
            gameModel.applySortMode(uiSettings.cardSort)
        }
    }

    UiSettingsBridge {
        id: uiSettings
        Component.onCompleted: {
            initWatcher()
            theme.overrides = JSON.parse(overridesJson())
        }
        onShowTrayIconChanged: {
            trayBridge.setEnabled(showTrayIcon)
            if (showTrayIcon) root.pushTrayRecent()
        }
    }

    TrayBridge {
        id: trayBridge

        onShow_window_requested: root.showFromTray()
        onToggle_window_requested: root.toggleFromTray()
        onLaunch_game_requested: (gameId) => root.launchFromTray(gameId)
        onQuit_requested: trayBridge.quitApp()

        Component.onCompleted: {
            initThread()
            setIcon(":/qt/qml/omikuji/qml/icons/app.png")
            if (uiSettings.showTrayIcon) {
                setEnabled(true)
                root.pushTrayRecent()
            }
        }
    }

    function pushTrayRecent() {
        if (!uiSettings.showTrayIcon) return
        let dated = []
        for (let i = 0; i < gameModel.count; i++) {
            let g = gameModel.get_game(i)
            if (!g) continue
            let ts = Date.parse(g.lastPlayed || "") || 0
            if (ts > 0) dated.push({ id: g.gameId, name: g.name, ts: ts })
        }
        dated.sort((a, b) => b.ts - a.ts)
        let out = []
        for (let i = 0; i < Math.min(10, dated.length); i++) {
            out.push({ id: dated[i].id, name: dated[i].name })
        }
        trayBridge.setRecentGames(JSON.stringify(out))
    }

    function showFromTray() {
        root.visible = true
        root.raise()
        root.requestActivate()
    }

    function toggleFromTray() {
        if (!root.visible) {
            showFromTray()
        } else if (!root.active) {
            root.raise()
            root.requestActivate()
        } else {
            root.visible = false
        }
    }

    function launchFromTray(gameId) {
        if (!gameId) return
        for (let i = 0; i < gameModel.count; i++) {
            let g = gameModel.get_game(i)
            if (g && g.gameId === gameId) {
                root.tryPlay(i)
                return
            }
        }
    }

    onClosing: (close) => {
        if (uiSettings.showTrayIcon) {
            close.accepted = false
            root.visible = false
        }
    }

    DefaultsBridge {
        id: defaultsBridge
        Component.onCompleted: initWatcher()
    }

    ComponentsBridge { id: componentsBridge }

    ArchiveManagerBridge { id: archiveManager }

    OfudaBridge { id: ofudaBridge }

    ScriptsBridge { id: scriptsBridge }

    MigrationBridge { id: migrationBridge }

    property var archiveActiveInstalls: ({})

    // bumped on runner changes so consumers re-query witout restart
    property int runnersVersion: 0

    property var openLogs: []

    function openGameLogs(gameId, gameName) {
        if (!gameId) return
        for (let i = 0; i < openLogs.length; i++) {
            if (openLogs[i].gameId === gameId) return
        }
        let next = openLogs.slice()
        next.push({ gameId: gameId, gameName: gameName || gameId })
        openLogs = next
    }

    function closeGameLogs(gameId) {
        openLogs = openLogs.filter(w => w.gameId !== gameId)
    }

    Connections {
        target: archiveManager
        function onInstallStarted(category, source, tag) {
            let k = category + "/" + source + "/" + tag
            let next = Object.assign({}, root.archiveActiveInstalls)
            next[k] = "starting"
            root.archiveActiveInstalls = next
        }
        function onInstallProgress(category, source, tag, phase, percent) {
            let k = category + "/" + source + "/" + tag
            let next = Object.assign({}, root.archiveActiveInstalls)
            next[k] = phase
            root.archiveActiveInstalls = next
        }
        function onInstallCompleted(category, source, tag, dir) {
            let k = category + "/" + source + "/" + tag
            let next = Object.assign({}, root.archiveActiveInstalls)
            delete next[k]
            root.archiveActiveInstalls = next
            if (category === "runners") root.runnersVersion++
        }
        function onInstallFailed(category, source, tag, err) {
            let k = category + "/" + source + "/" + tag
            let next = Object.assign({}, root.archiveActiveInstalls)
            delete next[k]
            root.archiveActiveInstalls = next
        }
    }

    Timer {
        interval: 500
        repeat: true
        running: true
        onTriggered: componentsBridge.drainEvents()
    }

    Timer {
        interval: 500
        repeat: true
        running: true
        onTriggered: archiveManager.drainEvents()
    }

    // 150ms so the window and toast manager are mounted before the first toast fires
    Timer {
        id: setupKickTimer
        interval: 150
        repeat: false
        running: true
        onTriggered: {
            if (componentsBridge.pendingCount > 0 && !componentsBridge.inProgress) {
                toastManager.show(
                    "info",
                    qsTr("Setting up omikuji"),
                    qsTr("Fetching runtime components — see Downloads for progress.")
                )
                componentsBridge.installEager()
            }
        }
    }

    Connections {
        target: componentsBridge
        function onComponentFailed(name, error) {
            toastManager.show("error", qsTr("%1 failed").arg(name), qsTr("Retry it from the Downloads tab."))
        }
        function onAllDoneChanged() {
            if (componentsBridge.allDone && componentsBridge.totalCount > 0) {
                toastManager.show("success", qsTr("omikuji is ready"), qsTr("Runtime components installed."))
            }
        }
    }

    // qualified refs so delegate Components don't self-reference their own null proprty
    readonly property var gameModelRef: gameModel
    readonly property var themeRef: theme
    readonly property var epicModelRef: epicModel
    readonly property var gogModelRef: gogModel
    readonly property var uiSettingsRef: uiSettings
    readonly property var envSetsDialogRef: envSetsDialog
    readonly property var dllSetsDialogRef: dllSetsDialog
    readonly property var componentsBridgeRef: componentsBridge
    readonly property var archiveManagerRef: archiveManager
    readonly property var ofudaBridgeRef: ofudaBridge
    readonly property var scriptsBridgeRef: scriptsBridge

    GameModel {
        id: gameModel

        onGame_stopped: (gameId) => {
            if (root.selectedGame && root.selectedGame.gameId === gameId) {
                root.isSelectedGameRunning = false
            }
            // !root.visible guard: don't clobber a manual re-show mid-session
            if (uiSettings.minimizeOnLaunch && !root.visible) {
                root.visible = true
                root.raise()
                root.requestActivate()
            }
        }

        onUpdates_queued: (epicCount, gogCount) => {
            let total = epicCount + gogCount
            if (total <= 0) return
            let bits = []
            if (epicCount > 0) bits.push(epicCount + " Epic")
            if (gogCount > 0) bits.push(gogCount + " GOG")
            toastManager.show("info", qsTr("Updates available"), qsTr("%1 queued in Downloads").arg(bits.join(" + ")))
        }

        Component.onCompleted: {
            gameModel.scan_all_for_updates()
            root.pushTrayRecent()
        }
    }

    Connections {
        target: gameModel
        function onDataChanged() { root.pushTrayRecent() }
        function onRowsInserted() { root.pushTrayRecent() }
        function onRowsRemoved() { root.pushTrayRecent() }
    }

    EpicModel { id: epicModel }

    GogModel { id: gogModel }

    DownloadModel {
        id: downloadModel
        onDownload_failed: (id, error) => console.warn("[downloads] failed:", id, error)
        onState_changed: root.refreshSelectedDownloadActivity()
    }

    LibraryWatcher {
        id: libWatcher
        onChanged: {
            gameModel.refresh(selectedGameIndex)
            resyncSelectedIndex()
            updateSelection()
        }
        Component.onCompleted: watch(gameModel.library_dir())
    }

    function resyncSelectedIndex() {
        if (selectedGameId === "") return
        for (let i = 0; i < gameModel.count; i++) {
            let game = gameModel.get_game(i)
            if (game && game["gameId"] === selectedGameId) {
                selectedGameIndex = i
                return
            }
        }
        selectedGameIndex = -1
    }

    Connections {
        target: gameModel
        function onRowsMoved() { root.resyncSelectedIndex() }
        function onRowsInserted() { root.resyncSelectedIndex() }
        function onRowsRemoved() { root.resyncSelectedIndex() }
    }


    Timer {
        interval: 250
        repeat: true
        running: true
        onTriggered: downloadModel.drain_events()
    }

    property int selectedGameIndex: -1
    // set before switching to settings view; the Loader-mounted page binds to this
    property int settingsGameIndex: -1
    property bool hasSelection: selectedGameIndex >= 0 && selectedGameIndex < gameModel.count
    property var selectedGame: null
    property string selectedGameId: ""
    property bool isSelectedGameRunning: false

    property string currentView: "library"
    property string activeModal: ""

    readonly property string currentViewLabel: currentView === "steam" ? "Steam"
        : currentView === "epic" ? "Epic Games"
        : currentView === "gog" ? "GOG"
        : currentView === "hoyo" ? "Gachas"
        : currentView === "downloads" ? "Downloads"
        : navTabs.tabs[navTabs.currentIndex]?.label || ""

    // clear search on view switch, but not on library filter tab flips (those dont change currentView, i think)
    onCurrentViewChanged: {
        topBar.searchText = ""
        topBar.defocusSearch()
    }

    onSelectedGameIndexChanged: {
        if (selectedGameIndex >= 0 && selectedGameIndex < gameModel.count) {
            let game = gameModel.get_game(selectedGameIndex)
            selectedGameId = game ? game["gameId"] : ""
            isSelectedGameRunning = gameModel.is_running(selectedGameIndex)
        } else {
            selectedGameId = ""
            isSelectedGameRunning = false
        }
        updateSelection()
    }

    function updateSelection() {
        let idx = selectedGameIndex
        if (idx < 0 || idx >= gameModel.count) {
            selectedGame = null
            return
        }
        let game = gameModel.get_game(idx)
        selectedGame = {
            name: game["name"],
            playtime: game["playtime"] || 0,
            lastPlayed: game["lastPlayed"] || "",
            runner: game["runner"] || "",
            runnerType: game["runnerType"] || "",
            gameId: game["gameId"] || "",
            sourceAppId: game["sourceAppId"] || ""
        }
        refreshSelectedDownloadActivity()
    }

    property var selectedDownloadActivity: null
    function refreshSelectedDownloadActivity() {
        if (!selectedGame || !selectedGame.sourceAppId) {
            selectedDownloadActivity = null
            return
        }
        let raw = downloadModel.active_for_app_id(selectedGame.sourceAppId)
        if (!raw || raw.length === 0) {
            selectedDownloadActivity = null
            return
        }
        try {
            selectedDownloadActivity = JSON.parse(raw)
        } catch (e) {
            console.warn("active_for_app_id returned bad json:", raw)
            selectedDownloadActivity = null
        }
    }

    // redirects to downloads if an install is in flight, launching mid-patch would read files teh patcher is rewriting
    function tryPlay(idx, forceSkipUpdateCheck = false) {
        if (idx < 0 || idx >= gameModel.count) return false
        let game = gameModel.get_game(idx)
        let appId = game ? (game["sourceAppId"] || "") : ""
        if (appId.length > 0) {
            let raw = downloadModel.active_for_app_id(appId)
            if (raw && raw.length > 0) {
                currentView = "downloads"
                return false
            }
        }
        if (gameModel.needs_prefix_prep(idx)) {
            prefixPrepDialog.start(idx, forceSkipUpdateCheck)
            return true
        }
        return doLaunch(idx, forceSkipUpdateCheck)
    }

    function doLaunch(idx, forceSkipUpdateCheck) {
        let launched = forceSkipUpdateCheck
            ? gameModel.launch_game_force(idx)
            : gameModel.launch_game(idx)
        if (launched) {
            isSelectedGameRunning = true
            if (uiSettings.minimizeOnLaunch) {
                steamStorePanel.keepAlive = false
                epicStorePanel.keepAlive = false
                gogStorePanel.keepAlive = false
                hoyoStorePanel.keepAlive = false
                root.visible = false
                root.releaseResources()
                gameModel.trim_heap()
            }
            return true
        }
        return false
    }

    Timer {
        id: libPollTimer
        interval: 500
        repeat: true
        running: true
        onTriggered: {
            gameModel.check_exited_games()
            gameModel.drain_notifications()
            gameModel.drain_update_notifications()
            gameModel.drain_errors()
            gameModel.drain_install_sizes()
            gameModel.drain_file_dialog_results()
            gameModel.drain_game_log_events()
        }
    }

    Connections {
        target: gameModel
        function onGame_stopped(gameId) {
            if (root.selectedGame && gameId === root.selectedGame.gameId) {
                root.isSelectedGameRunning = false
            }
        }
        function onNotification(level, title, message) {
            toastManager.show(level, title, message)
        }
        function onUpdate_required(gameId, appId, displayName, fromVersion, toVersion, downloadSize, canDiff, deltaSupported) {
            updateDialog.show({
                gameId: gameId,
                appId: appId,
                displayName: displayName,
                fromVersion: fromVersion,
                toVersion: toVersion,
                downloadBytes: parseInt(downloadSize, 10) || 0,
                canDiff: canDiff,
                deltaSupported: deltaSupported
            })
        }
        function onError_required(gameId, displayName, title, message, action) {
            errorDialog.show({
                gameId: gameId,
                displayName: displayName,
                title: title,
                message: message,
                action: action
            })
        }
    }

property real cardZoom: uiSettings.cardZoom
    readonly property int cardBaseWidth: 180
    readonly property int cardBaseHeight: 240

    property real uiScale: uiSettings.uiScale > 0 ? uiSettings.uiScale : 1.0

    // Ctrl+Plus (named key) because "Ctrl++" doesnt work. i mean not that this works for me but i guess whatever the fuck this thing wants.
    Shortcut {
        sequences: ["Ctrl+Plus", "Ctrl+Shift+=", "Ctrl+=", "Ctrl+Up", StandardKey.ZoomIn]
        onActivated: uiSettings.applyUiScale(root.uiScale + 0.1)
    }
    Shortcut {
        sequences: ["Ctrl+-", "Ctrl+Down", StandardKey.ZoomOut]
        onActivated: uiSettings.applyUiScale(root.uiScale - 0.1)
    }
    Shortcut {
        sequence: "Ctrl+0"
        onActivated: uiSettings.applyUiScale(1.0)
    }

    Item {
        id: scaledRoot
        width: root.width / root.uiScale
        height: root.height / root.uiScale
        transform: Scale { xScale: root.uiScale; yScale: root.uiScale; origin.x: 0; origin.y: 0 }

    NavTabs {
        id: navTabs
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        // above dropdown popups (z 50) so in-panel dropdowns dont bleed over the nav
        z: 100

        width: uiSettings.navCollapsed ? 0 : uiSettings.navWidth
        onWidthRequested: (v) => {
            if (v === 0) {
                // drag to zero = collapse, dont overwrite the remembered expanded width
                uiSettings.applyNavCollapsed(true)
            } else {
                if (uiSettings.navCollapsed) uiSettings.applyNavCollapsed(false)
                uiSettings.applyNavWidth(v)
            }
        }

        downloadCount: downloadModel.activeCount
        headerLabel: root.currentViewLabel

        uiSettings: uiSettings

        showSteam: uiSettings.showSteam
        showEpic: uiSettings.showEpic
        showGog: uiSettings.showGog
        showGachas: uiSettings.showGachas

        onStoreSelected: (storeName) => {
            navTabs.currentBottom = ""
            if (storeName === "Steam") {
                navTabs.currentStore = "Steam"
                root.currentView = "steam"
            } else if (storeName === "Epic") {
                navTabs.currentStore = "Epic"
                root.currentView = "epic"
            } else if (storeName === "GOG") {
                navTabs.currentStore = "GOG"
                root.currentView = "gog"
            } else if (storeName === "HoYo") {
                navTabs.currentStore = "HoYo"
                root.currentView = "hoyo"
            }
        }

        onTabSelected: (index) => {
            navTabs.currentStore = ""
            navTabs.currentBottom = ""
            root.currentView = "library"
        }

        onDownloadsClicked: {
            navTabs.currentBottom = "downloads"
            root.currentView = "downloads"
        }

        onSettingsClicked: root.activeModal = "globalSettings"
    }

    MouseArea {
        id: navExpander
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: 6
        // above chrome (z 100) but below dialogs/toasts
        z: 150
        visible: uiSettings.navCollapsed
        enabled: visible
        cursorShape: Qt.SizeHorCursor
        hoverEnabled: true

        property real pressStartX: 0
        property bool didDrag: false

        onPressed: (mouse) => {
            pressStartX = mouse.x
            didDrag = false
        }
        onPositionChanged: (mouse) => {
            if (!pressed) return
            if (!didDrag && Math.abs(mouse.x - pressStartX) > 4) didDrag = true
            if (!didDrag) return
            if (mouse.x < 20) return
            uiSettings.applyNavCollapsed(false)
            const target = Math.max(navTabs.minWidth, Math.min(navTabs.maxWidth, mouse.x))
            uiSettings.applyNavWidth(target)
        }
        onReleased: (mouse) => {
            // bare click restores at remebmered width
            if (!didDrag) uiSettings.applyNavCollapsed(false)
        }

        Rectangle {
            anchors.left: parent.left
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            width: 2
            color: theme.accent
            opacity: navExpander.pressed ? 0.7 : (navExpander.containsMouse ? 0.35 : 0)
            Behavior on opacity { NumberAnimation { duration: 120 } }
        }
    }

    TopBar {
        id: topBar
        anchors.top: parent.top
        anchors.left: navTabs.right
        anchors.right: parent.right
        z: 100

        currentTabLabel: root.currentViewLabel
        showTitle: uiSettings.navCollapsed
        leftInset: navTabs.width

        showAddButton: root.currentView === "library"
        showSearch: root.currentView === "library"
            || root.currentView === "steam"
            || root.currentView === "epic"
            || root.currentView === "gog"
            || root.currentView === "hoyo"
        showDisplayOptions: root.currentView === "library"
            || root.currentView === "steam"
            || root.currentView === "epic"
            || root.currentView === "gog"
            || root.currentView === "hoyo"
        zoomValue: uiSettings.cardZoom
        spacingValue: uiSettings.cardSpacing
        sortValue: uiSettings.cardSort
        showSort: root.currentView === "library"
        showHiddenValue: uiSettings.showHidden
        showHiddenOption: root.currentView === "library"

        onAddClicked: root.activeModal = "addGame"
        onInstallScriptClicked: scriptBrowserDialog.show()
        onConsoleModeClicked: gameModel.launch_console_mode()
        onZoomMoved: (v) => uiSettings.applyCardZoom(v)
        onSpacingMoved: (v) => uiSettings.applyCardSpacing(v)
        onSortSelected: (v) => uiSettings.applyCardSort(v)
        onShowHiddenToggled: (v) => uiSettings.applyShowHidden(v)
    }

    Item {
        anchors.top: topBar.bottom
        anchors.left: navTabs.right
        anchors.right: parent.right
        anchors.bottom: parent.bottom

        Rectangle {
            id: contentPanel
            property bool isDropdownHost: true
            anchors.fill: parent
            color: theme.surface
            radius: theme.radius.md
            visible: opacity > 0
            opacity: root.currentView === "library" ? 1 : 0

            Behavior on opacity {
                NumberAnimation { duration: 200; easing.type: Easing.OutCubic }
            }

            Rectangle {
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                width: parent.radius
                height: parent.radius
                color: parent.color
                visible: parent.visible
            }

            GameGrid {
                id: gameGrid
                anchors.fill: parent
                model: gameModel
                gameModel: gameModel
                selectedIndex: root.selectedGameIndex
                cardZoom: root.cardZoom
                cardSpacing: uiSettings.cardSpacing
                cardElevation: uiSettings.cardElevation
                cardBaseWidth: root.cardBaseWidth
                cardBaseHeight: root.cardBaseHeight
                cardFlow: uiSettings.cardFlow
                cardSort: uiSettings.cardSort
                showHidden: uiSettings.showHidden
                dimHidden: uiSettings.dimHidden
                searchText: topBar.searchText
                filterKind: navTabs.tabs[navTabs.currentIndex]?.kind || "all"
                filterValue: navTabs.tabs[navTabs.currentIndex]?.value || ""
                onGameClicked: (index) => {
                    root.selectedGameIndex = index
                    topBar.defocusSearch()
                }
                onGameDoubleClicked: (index) => {
                    if (uiSettings.doubleClickLaunches) root.tryPlay(index)
                }
                onGameRightClicked: (index, winX, winY) => gameContextMenu.show(index, winX, winY)
                onBackgroundClicked: {
                    root.selectedGameIndex = -1
                    topBar.defocusSearch()
                }
            }

            FloatingBar {
                id: floatingBar
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                selectedGame: root.selectedGame
                hasSelection: root.hasSelection
                isRunning: root.isSelectedGameRunning
                downloadActivity: root.selectedDownloadActivity
                onSettingsClicked: {
                    root.settingsGameIndex = root.selectedGameIndex
                    root.activeModal = "gameSettings"
                }
                onDownloadActivityClicked: {
                    root.currentView = "downloads"
                }
                onPlayClicked: root.tryPlay(root.selectedGameIndex)
                onStopClicked: {
                    if (root.selectedGame && root.selectedGame.gameId) {
                        gameModel.stop_game(root.selectedGame.gameId)
                    }
                }
                onWineToolsClicked: {
                    if (!root.selectedGame || !root.selectedGame.gameId) return
                    if (Date.now() - wineToolsMenu.lastClosedAt < 150) return
                    wineToolsMenu.openAbove(floatingBar.wineToolsAnchor)
                }
            }
        }

        StorePanel {
            id: steamStorePanel
            viewName: "steam"
            currentView: root.currentView
            unloadIdle: uiSettings.unloadStorePages
            onIdleUnloaded: gameModel.trim_heap()
            sourceComponent: SteamLibrary {
                gameModel: root.gameModelRef
                cardZoom: root.cardZoom
                cardSpacing: uiSettings.cardSpacing
                cardElevation: uiSettings.cardElevation
                cardFlow: uiSettings.cardFlow
                searchText: topBar.searchText
                onBackClicked: {
                    navTabs.currentStore = ""
                    navTabs.currentIndex = 0
                    root.currentView = "library"
                }
            }
        }

        StorePanel {
            id: epicStorePanel
            viewName: "epic"
            currentView: root.currentView
            unloadIdle: uiSettings.unloadStorePages
            onIdleUnloaded: gameModel.trim_heap()
            sourceComponent: EpicLibrary {
                epicModel: root.epicModelRef
                cardZoom: root.cardZoom
                cardSpacing: uiSettings.cardSpacing
                cardElevation: uiSettings.cardElevation
                cardFlow: uiSettings.cardFlow
                searchText: topBar.searchText
                activeDownloads: epicController.activeDownloads
                onBackClicked: {
                    navTabs.currentStore = ""
                    navTabs.currentIndex = 0
                    root.currentView = "library"
                }
                onInstallRequested: (index) => epicController.showInstall(index)
                onImportRequested: (index) => epicController.showInstall(index)
            }
        }

        StorePanel {
            id: gogStorePanel
            viewName: "gog"
            currentView: root.currentView
            unloadIdle: uiSettings.unloadStorePages
            onIdleUnloaded: gameModel.trim_heap()
            sourceComponent: GogLibrary {
                gogModel: root.gogModelRef
                cardZoom: root.cardZoom
                cardSpacing: uiSettings.cardSpacing
                cardElevation: uiSettings.cardElevation
                cardFlow: uiSettings.cardFlow
                searchText: topBar.searchText
                activeDownloads: gogController.activeDownloads
                onBackClicked: {
                    navTabs.currentStore = ""
                    navTabs.currentIndex = 0
                    root.currentView = "library"
                }
                onInstallRequested: (index) => gogController.showInstall(index)
                onImportRequested: (index) => gogController.showInstall(index)
            }
        }

        StorePanel {
            id: hoyoStorePanel
            viewName: "hoyo"
            currentView: root.currentView
            unloadIdle: uiSettings.unloadStorePages
            onIdleUnloaded: gameModel.trim_heap()
            property bool manifestsFetched: false
            onActivated: {
                if (!manifestsFetched) {
                    manifestsFetched = true
                    gameModel.ensureGachaManifests()
                }
            }
            sourceComponent: GachaLibrary {
                gameModel: root.gameModelRef
                cardZoom: root.cardZoom
                cardSpacing: uiSettings.cardSpacing
                cardElevation: uiSettings.cardElevation
                cardFlow: uiSettings.cardFlow
                searchText: topBar.searchText
                onBackClicked: {
                    navTabs.currentStore = ""
                    navTabs.currentIndex = 0
                    root.currentView = "library"
                }
                onInstallRequested: (manifestId) => gachaController.showInstall(manifestId)
            }
        }

        Rectangle {
            id: downloadsPanel
            property bool isDropdownHost: true
            anchors.fill: parent
            color: theme.surface
            radius: theme.radius.md
            visible: opacity > 0
            opacity: root.currentView === "downloads" ? 1 : 0

            Behavior on opacity {
                NumberAnimation { duration: 200; easing.type: Easing.OutCubic }
            }

            Rectangle {
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                width: parent.radius
                height: parent.radius
                color: parent.color
                visible: parent.visible
            }

            DownloadsPage {
                anchors.fill: parent
                downloadModel: downloadModel
                componentsBridge: componentsBridge
                pageVisible: root.currentView === "downloads"
                onCancelRequested: (id, displayName) => {
                    cancelDownloadConfirm.message =
                        qsTr("This will stop \"%1\" and delete the partially downloaded files.").arg(displayName)
                    cancelDownloadConfirm.show(id)
                }
            }
        }

    }

    EpicController {
        id: epicController
        gameModel: root.gameModelRef
        epicModel: epicModel
        downloadModel: downloadModel
        defaults: defaultsBridge
        runnersVersion: root.runnersVersion
        onInstallEnqueued: {
            navTabs.currentStore = ""
            navTabs.currentBottom = "downloads"
            root.currentView = "downloads"
        }
    }

    GogController {
        id: gogController
        gameModel: root.gameModelRef
        gogModel: gogModel
        downloadModel: downloadModel
        defaults: defaultsBridge
        runnersVersion: root.runnersVersion
        onInstallEnqueued: {
            navTabs.currentStore = ""
            navTabs.currentBottom = "downloads"
            root.currentView = "downloads"
        }
    }

    GachaController {
        id: gachaController
        gameModel: root.gameModelRef
        downloadModel: downloadModel
        defaults: defaultsBridge
        runnersVersion: root.runnersVersion
        onInstallEnqueued: {
            navTabs.currentStore = ""
            navTabs.currentBottom = "downloads"
            root.currentView = "downloads"
        }
    }

    GameContextMenu {
        id: gameContextMenu
        gameModel: root.gameModelRef
        onPlayRequested: (idx) => root.tryPlay(idx)
        onLogsRequested: (gid, gname) => root.openGameLogs(gid, gname)
        onConfigureRequested: (idx) => {
            root.selectedGameIndex = idx
            root.settingsGameIndex = idx
            root.activeModal = "gameSettings"
        }
        onCategoriesRequested: (idx) => categoriesController.showForGame(idx)
        onRemoveRequested: (idx) => {
            if (root.selectedGameIndex === idx) root.selectedGameIndex = -1
        }
    }

    // hoisted to root so the dim backdrop covers the whole window
    ConfirmDialog {
        id: cancelDownloadConfirm
        anchors.fill: parent
        title: qsTr("Cancel download?")
        confirmText: qsTr("Cancel & delete")
        cancelText: qsTr("Keep")
        destructive: true
        onConfirmed: (id) => { if (downloadModel) downloadModel.cancel(id) }
    }

    ConfirmDialog {
        id: refetchMediaConfirm
        anchors.fill: parent
        title: qsTr("Refetch art from SGDB")
        message: qsTr("Replaces the cached banner, cover art, and icon with a fresh pull from SteamGridDB. Manual overrides you've set won't be touched.")
        confirmText: qsTr("Refetch")
        cancelText: qsTr("Cancel")
        onConfirmed: (id) => { if (id && gameModel) gameModel.refetch_media(id) }
    }

    DefaultsApplyDialog {
        id: defaultsApplyDialog
        anchors.fill: parent
        defaults: defaultsBridge
        gameModel: root.gameModelRef
    }

    SetsDialog {
        id: envSetsDialog
        libRead: () => uiSettings.envSetsJson()
        libWrite: (j) => uiSettings.applyEnvSetsJson(j)
        copyKey: "launch.env"
        syncKey: "launch.env_sets"
        keyPlaceholder: "VAR_NAME"
        valuePlaceholder: "value"
        titleText: qsTr("Environment sets")
        manageTitle: qsTr("Manage env sets")
    }

    SetsDialog {
        id: dllSetsDialog
        libRead: () => uiSettings.dllSetsJson()
        libWrite: (j) => uiSettings.applyDllSetsJson(j)
        copyKey: "wine.dll_overrides"
        syncKey: "wine.dll_override_sets"
        keyPlaceholder: "dll_name"
        valuePlaceholder: "n,b"
        titleText: qsTr("DLL override sets")
        manageTitle: qsTr("Manage DLL sets")
    }

    LogRulesDialog {
        id: logRulesDialog
        anchors.fill: parent
        uiSettings: uiSettings
    }

    CategoriesController {
        id: categoriesController
        uiSettings: uiSettings
        gameModel: gameModel
    }

    ArchiveManageDialog {
        id: archiveManageDialog
        anchors.fill: parent
        archiveManager: archiveManager
        activeInstalls: root.archiveActiveInstalls
        onVersionDeleted: (category, sourceName, tag) => {
            if (category === "runners") root.runnersVersion++
        }
        onRemoveSourceRequested: (category, sourceName) => {
            removeSourceConfirm.message = qsTr("Removes \"%1\" from your sources. Installed versions stay on disk and keep working; adding a source with the same name picks them up again.").arg(sourceName)
            archiveManageDialog.escEnabled = false
            removeSourceConfirm.show({ category: category, source: sourceName })
        }
    }

    ArchiveSourceDialog {
        id: archiveSourceDialog
        anchors.fill: parent
        archiveManager: archiveManager
    }

    ConfirmDialog {
        id: removeSourceConfirm
        anchors.fill: parent
        destructive: true
        confirmText: qsTr("Remove")
        onConfirmed: (p) => {
            archiveManageDialog.escEnabled = true
            const err = archiveManager.removeSource(p.category, p.source)
            if (err === "") archiveManageDialog.hide()
            else toastManager.show("error", qsTr("Couldn't remove source"), err)
        }
        onCancelled: archiveManageDialog.escEnabled = true
    }

    Instantiator {
        model: root.openLogs
        active: true
        asynchronous: false
        delegate: GameLogsWindow {
            required property var modelData
            gameId: modelData.gameId
            gameName: modelData.gameName
            gameModel: root.gameModelRef
            theme: root.themeRef
            uiSettings: root.uiSettingsRef
            onWindowClosed: root.closeGameLogs(gameId)
        }
    }

    UpdateAvailableDialog {
        id: updateDialog
        anchors.fill: parent
        onUpdateRequested: (gid, aid, fromV) => {
            let newId = gameModel.enqueue_game_update(gid, fromV)
            if (newId && newId.length > 0) {
                toastManager.show("info", qsTr("Update queued"), root.selectedGame ? root.selectedGame.name : "")
            } else {
                toastManager.show("error", qsTr("Update failed"), qsTr("Could not enqueue update"))
            }
        }
        onRunAnywayRequested: (gid) => {
            let idx = gameModel.index_of_id(gid)
            if (idx >= 0) root.tryPlay(idx, true)
        }
    }

    ErrorDialog {
        id: errorDialog
        anchors.fill: parent
        onActionRequested: (act, gid) => {
            if (act === "open_game_settings") {
                let idx = gameModel.index_of_id(gid)
                if (idx >= 0) {
                    root.settingsGameIndex = idx
                    root.activeModal = "gameSettings"
                }
            } else if (act === "open_global_settings") {
                root.activeModal = "globalSettings"
            }
        }
    }

    PrefixCreateDialog {
        id: prefixCreateDialog
        anchors.fill: parent
        gameModel: root.gameModelRef
        ofudaBridge: root.ofudaBridgeRef
    }

    ScriptBrowserDialog {
        id: scriptBrowserDialog
        anchors.fill: parent
        scriptsBridge: root.scriptsBridgeRef
        gameModel: root.gameModelRef
        onScriptChosen: (path) => scriptRunDialog.show(path)
    }

    ScriptRunDialog {
        id: scriptRunDialog
        anchors.fill: parent
        scriptsBridge: root.scriptsBridgeRef
        gameModel: root.gameModelRef
        ofudaBridge: root.ofudaBridgeRef
        onInstalled: (gameId, gameName) => toastManager.show("success", qsTr("Game added"), gameName)
    }

    PrefixPrepDialog {
        id: prefixPrepDialog
        anchors.fill: parent
        gameModel: root.gameModelRef
        onLaunchReady: (idx, skip) => root.doLaunch(idx, skip)
    }

    MigrationDialog {
        id: migrationDialog
        anchors.fill: parent
        bridge: migrationBridge
        Component.onCompleted: if (migrationBridge.pending()) start()
    }

    PrefixDetailDialog {
        id: prefixDetailDialog
        anchors.fill: parent
        ofudaBridge: root.ofudaBridgeRef
        onDeleteRequested: (p) => {
            const n = (p.games || []).length
            deletePrefixConfirm.message = n > 0
                ? (n === 1
                    ? qsTr("This deletes the prefix and everything in it. 1 game uses it, and it won't be recoverable.")
                    : qsTr("This deletes the prefix and everything in it. %1 games use it, and it won't be recoverable.").arg(n))
                : qsTr("This deletes the prefix and everything in it. It won't be recoverable.")
            prefixDetailDialog.escEnabled = false
            deletePrefixConfirm.show(p)
        }
    }

    ConfirmDialog {
        id: deletePrefixConfirm
        anchors.fill: parent
        title: qsTr("Delete prefix?")
        confirmText: qsTr("Delete")
        cancelText: qsTr("Cancel")
        destructive: true
        onConfirmed: (p) => {
            if (ofudaBridge && p) ofudaBridge.deletePrefix(p.path)
            prefixDetailDialog.escEnabled = true
            prefixDetailDialog.close()
        }
        onCancelled: prefixDetailDialog.escEnabled = true
    }

    ContextMenu {
        id: wineToolsMenu
        property string pendingRunExeRequestId: ""

        items: [
            { text: qsTr("Configure (winecfg)"),    action: "winecfg" },
            { text: "Winetricks",                    action: "winetricks" },
            { text: qsTr("Registry (regedit)"),      action: "regedit" },
            { text: qsTr("Command Prompt (cmd)"),    action: "cmd" },
            { text: qsTr("File Explorer (explorer)"), action: "explorer" },
            { text: qsTr("Run EXE in prefix…"),      action: "run_exe" },
            { text: qsTr("Kill wineserver"),         action: "killwineserver", danger: true }
        ] // TODO: dry up the dispatcher mighjt aswell kill it

        onItemClicked: (action) => {
            if (!root.selectedGame || !root.selectedGame.gameId) return
            let gid = root.selectedGame.gameId
            if (action === "run_exe") {
                let rid = "wine_run_exe_" + Date.now().toString(36)
                pendingRunExeRequestId = rid
                gameModel.open_file_dialog(rid, false, qsTr("Select EXE to run in prefix"), "/home", "")
            } else {
                gameModel.run_wine_tool(gid, action)
            }
        }

        Connections {
            target: gameModel
            enabled: wineToolsMenu.pendingRunExeRequestId !== ""
            function onFile_dialog_result(requestId, path) {
                if (requestId !== wineToolsMenu.pendingRunExeRequestId) return
                wineToolsMenu.pendingRunExeRequestId = ""
                if (path && path !== "" && root.selectedGame && root.selectedGame.gameId) {
                    gameModel.run_wine_exe(root.selectedGame.gameId, path)
                }
            }
        }
    }

    SettingsModal {
        id: gameSettingsModal
        shown: root.activeModal === "gameSettings"
        sizeKey: "game_settings"
        onCloseRequested: { if (pageItem) pageItem.closeAction(); root.activeModal = "" }
        pageComponent: Component {
            GameSettingsPage {
                gameModel: root.gameModelRef
                runnersVersion: root.runnersVersion
                gameIndex: root.settingsGameIndex
                envSetsDialog: root.envSetsDialogRef
                dllSetsDialog: root.dllSetsDialogRef
                onSaveRequested: (idx) => root.activeModal = ""
                onSaveAndPlayRequested: (idx) => {
                    root.activeModal = ""
                    root.tryPlay(idx)
                }
                onRefetchMediaRequested: (gid) => refetchMediaConfirm.show(gid)
            }
        }
    }

    SettingsModal {
        id: addGameModal
        shown: root.activeModal === "addGame"
        sizeKey: "add_game"
        onCloseRequested: { if (pageItem) pageItem.closeAction(); root.activeModal = "" }
        pageComponent: Component {
            AddGamePage {
                gameModel: root.gameModelRef
                runnersVersion: root.runnersVersion
                envSetsDialog: root.envSetsDialogRef
                dllSetsDialog: root.dllSetsDialogRef
                onGameCreated: (gameId) => {
                    root.activeModal = ""
                    for (let i = 0; i < gameModel.count; i++) {
                        let g = gameModel.get_game(i)
                        if (g && g["gameId"] === gameId) {
                            root.selectedGameIndex = i
                            break
                        }
                    }
                }
                onGameCreatedAndPlay: (gameId) => {
                    root.activeModal = ""
                    for (let i = 0; i < gameModel.count; i++) {
                        let g = gameModel.get_game(i)
                        if (g && g["gameId"] === gameId) {
                            root.selectedGameIndex = i
                            root.tryPlay(i)
                            break
                        }
                    }
                }
            }
        }
    }

    SettingsModal {
        id: globalSettingsModal
        shown: root.activeModal === "globalSettings"
        sizeKey: "global_settings"
        onCloseRequested: root.activeModal = ""
        pageComponent: Component {
            GlobalSettingsPage {
                uiSettings: root.uiSettingsRef
                componentsBridge: root.componentsBridgeRef
                archiveManager: root.archiveManagerRef
                ofudaBridge: root.ofudaBridgeRef
                defaults: defaultsBridge
                gameModel: root.gameModelRef
                activeInstalls: root.archiveActiveInstalls
                onManageRequested: (category, source, kind) => {
                    archiveManageDialog.show(category, source, kind)
                }
                onAddSourceRequested: (category) => archiveSourceDialog.show(category)
                onManageSetsRequested: (kind) => (kind === "dll" ? dllSetsDialog : envSetsDialog).openManage()
                onManageLogRulesRequested: logRulesDialog.show()
                onCategoryAddRequested: categoriesController.showAdd()
                onCategoryEditRequested: (idx, entry) => categoriesController.showEdit(idx, entry)
                onCategoryDeleteRequested: (idx, entry) => categoriesController.showDelete(idx, entry)
                onDefaultsApplyToExistingRequested: defaultsApplyDialog.show()
                onPrefixOpenRequested: (p) => prefixDetailDialog.show(p)
                onPrefixCreateRequested: prefixCreateDialog.show()
            }
        }
    }

    // z:1000 so it overlays all panels, dropdowns, and dialogs
    ToastManager {
        id: toastManager
        anchors.fill: parent
        z: 1000
    }

    }
}

// TODO might just really need to spend a week just on un-spaghettifying the whole qml. just sayin
// TODO refractor game cards (need proper indexes for sorting)
// TODO restyle toast
// TODO kill dropshadow so we can drop qt6-5compat