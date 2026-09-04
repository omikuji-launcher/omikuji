pragma ComponentBehavior: Bound

import QtQuick
import QtQml
import QtQuick.Controls

import omikuji 1.0
import "components/categories"
import "components/controls"
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
    color: Theme.navBg

    flags: Qt.Window

    Connections {
        target: appSettings
        function onThemeChanged() {
            Theme.overrides = JSON.parse(appSettings.overridesJson())
        }
        function onFontSizesChanged() {
            Theme.fontSizes = JSON.parse(appSettings.fontSizesJson())
        }
        function onRadiusOverridesChanged() {
            Theme.radiusOverrides = JSON.parse(appSettings.radiusOverridesJson())
        }
        function onCardSortChanged() {
            gameModel.applySortMode(appSettings.cardSort)
        }
    }

    AppSettingsBridge {
        id: appSettings
        Component.onCompleted: {
            initWatcher()
            Theme.mutedIcons = Qt.binding(() => appSettings.mutedIcons)
            Theme.filledIcons = Qt.binding(() => appSettings.filledIcons)
            Theme.followSystemColors = Qt.binding(() => appSettings.followSystemColors)
            Theme.followSystemFont = Qt.binding(() => appSettings.followSystemFont)
            Theme.fontFamily = Qt.binding(() => appSettings.fontFamily)
            Theme.fillFields = Qt.binding(() => appSettings.fillFields)
            Theme.uiScale = Qt.binding(() => root.uiScale)
            Theme.overrides = JSON.parse(overridesJson())
            Theme.fontSizes = JSON.parse(fontSizesJson())
            Theme.radiusOverrides = JSON.parse(radiusOverridesJson())
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
            if (appSettings.showTrayIcon) {
                setEnabled(true)
                root.pushTrayRecent()
            }
        }
    }

    function pushTrayRecent() {
        if (!appSettings.showTrayIcon) return
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
                gameActions.play(i)
                return
            }
        }
    }

    onClosing: (close) => {
        if (appSettings.showTrayIcon) {
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

    readonly property string latestCategory: "runners_latest"
    property var latestToasts: ({})

    readonly property var updatingRunners: {
        let out = {}
        for (let source in root.latestToasts) out[source + "-Latest"] = true
        return out
    }

    function beginLatestToast(source, message) {
        if (root.latestToasts[source] !== undefined) return
        let next = Object.assign({}, root.latestToasts)
        next[source] = toastManager.showProgress("info", qsTr("Updating %1").arg(source), message)
        root.latestToasts = next
    }

    function endLatestToast(source) {
        let id = root.latestToasts[source]
        if (id === undefined) return
        toastManager.dismiss(id)
        let next = Object.assign({}, root.latestToasts)
        delete next[source]
        root.latestToasts = next
    }

    function installKey(category, source, tag) {
        return category + "/" + source + "/" + tag
    }

    function setInstallPhase(key, phase) {
        let next = Object.assign({}, root.archiveActiveInstalls)
        if (phase) next[key] = phase
        else delete next[key]
        root.archiveActiveInstalls = next
    }

    function isRunnerCategory(category) {
        return category === "runners" || category === root.latestCategory
    }

    property var latestToastMuted: ({})

    function latestToastMutedFor(source) {
        return root.latestToastMuted[source] === true
    }

    function unmuteLatestToast(source) {
        delete root.latestToastMuted[source]
    }

    Connections {
        target: archiveManager
        function onInstallStarted(category, source, tag) {
            root.setInstallPhase(root.installKey(category, source, tag), "starting")
            if (category !== root.latestCategory || root.latestToastMutedFor(source)) return
            root.beginLatestToast(source, qsTr("Starting download"))
        }
        function onInstallProgress(category, source, tag, phase, percent) {
            root.setInstallPhase(root.installKey(category, source, tag), phase)
            if (category !== root.latestCategory) return
            let id = root.latestToasts[source]
            if (id === undefined) return
            toastManager.update(id, phase === "extracting"
                ? qsTr("Extracting %1").arg(tag)
                : qsTr("Downloading %1").arg(tag), percent / 100.0)
        }
        function onInstallCompleted(category, source, tag, dir) {
            root.setInstallPhase(root.installKey(category, source, tag), "")
            if (root.isRunnerCategory(category)) root.runnersVersion++
            if (category !== root.latestCategory) return
            root.endLatestToast(source)
            if (!root.latestToastMutedFor(source))
                toastManager.show("success", qsTr("%1 updated").arg(source), tag)
            root.unmuteLatestToast(source)
        }
        function onInstallFailed(category, source, tag, err) {
            root.setInstallPhase(root.installKey(category, source, tag), "")
            if (category !== root.latestCategory) return
            root.endLatestToast(source)
            if (!root.latestToastMutedFor(source))
                toastManager.show("error", qsTr("Couldn't update %1").arg(source), err)
            root.unmuteLatestToast(source)
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
            let raw = gameModel.changelog_pending()
            let payload = null
            if (raw && raw.length > 0) {
                try { payload = JSON.parse(raw) } catch (e) { payload = null }
            }
            if (payload && payload.body && payload.body.length > 0) {
                changelogDialog.show(payload)
            } else {
                gameModel.mark_changelog_seen()
            }

            if (!appSettings.welcomeSeen) {
                welcomeDialog.show()
            }
        }
    }

    Connections {
        target: componentsBridge
        function onComponentFailed(name, error) {
            toastManager.show("error", qsTr("%1 failed").arg(name), qsTr("Retry it from the Downloads tab."))
        }
    }

    // qualified refs so delegate Components don't self-reference their own null proprty
    readonly property var gameModelRef: gameModel
    readonly property var downloadModelRef: downloadModel
    readonly property var epicModelRef: epicModel
    readonly property var gogModelRef: gogModel
    readonly property var nileModelRef: nileModel
    readonly property var appSettingsRef: appSettings
    readonly property var envSetsDialogRef: envSetsDialog
    readonly property var dllSetsDialogRef: dllSetsDialog
    property bool welcomeResumePending: false
    readonly property var componentsBridgeRef: componentsBridge
    readonly property var archiveManagerRef: archiveManager
    readonly property var ofudaBridgeRef: ofudaBridge
    readonly property var scriptsBridgeRef: scriptsBridge

    GameModel {
        id: gameModel

        onGame_stopped: (gameId) => {
            if (gameActions.selectedGame && gameActions.selectedGame.gameId === gameId) {
                gameActions.refreshRunState()
            }
            // !root.visible guard: don't clobber a manual re-show mid-session
            if (appSettings.minimizeOnLaunch && !root.visible) {
                root.visible = true
                root.raise()
                root.requestActivate()
            }
        }

        onLaunchProceeding: () => {
            if (appSettings.minimizeOnLaunch) root.minimizeForLaunch()
        }

        onUpdates_queued: (epicCount, gogCount, nileCount, gachaCount) => {
            let total = epicCount + gogCount + nileCount + gachaCount
            if (total <= 0) return
            let bits = []
            if (epicCount > 0) bits.push(epicCount + " Epic")
            if (gogCount > 0) bits.push(gogCount + " GOG")
            if (nileCount > 0) bits.push(nileCount + " Amazon")
            if (gachaCount > 0) bits.push(gachaCount + qsTr(" gacha"))
            toastManager.show("info", qsTr("Updates available"), qsTr("%1 queued in Downloads").arg(bits.join(" + ")))
        }

        onLatestRefreshQueued: (sources) => {
            let names = []
            try {
                names = JSON.parse(sources)
            } catch (e) {
                return
            }
            for (let i = 0; i < names.length; i++) {
                root.beginLatestToast(names[i], qsTr("Queued"))
            }
        }

        onLatestRefreshDone: (source) => root.endLatestToast(source)

        Component.onCompleted: {
            gameModel.scan_all_for_updates()
            gameModel.refreshLatestRunners()
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

    NileModel { id: nileModel }

    DownloadModel {
        id: downloadModel
        onDownload_failed: (id, error) => console.warn("[downloads] failed:", id, error)
        onState_changed: gameActions.refreshDownloadActivity()
    }

    LibraryWatcher {
        id: libWatcher
        onChanged: {
            gameModel.refresh(gameActions.selectedIndex)
            gameActions.resyncSelection()
            gameActions.updateSelection()
        }
        Component.onCompleted: watch(gameModel.library_dir())
    }

    Connections {
        target: gameModel
        function onRowsMoved() { gameActions.resyncSelection() }
        function onRowsInserted() { gameActions.resyncSelection() }
        function onRowsRemoved() { gameActions.resyncSelection() }
    }


    Timer {
        interval: 250
        repeat: true
        running: true
        onTriggered: downloadModel.drain_events()
    }

    // set before switching to settings view; the Loader-mounted page binds to this
    property int settingsGameIndex: -1

    GameActions {
        id: gameActions
        gameModel: root.gameModelRef
        downloadModel: root.downloadModelRef
        updatingRunners: root.updatingRunners
        onDownloadInFlight: root.currentView = "downloads"
        onComponentRequired: (index, skipUpdateCheck, missing) =>
            componentRequiredDialog.start(index, skipUpdateCheck, missing)
        onPrefixPrepRequired: (index, skipUpdateCheck) =>
            prefixPrepDialog.start(index, skipUpdateCheck)
        onForceLaunched: {
            if (appSettings.minimizeOnLaunch) root.minimizeForLaunch()
        }
    }

    property string currentView: "library"
    property string activeModal: ""

    readonly property string currentViewLabel: currentView === "steam" ? "Steam"
        : currentView === "epic" ? "Epic Games"
        : currentView === "gog" ? "GOG"
        : currentView === "nile" ? "Amazon"
        : currentView === "hoyo" ? "Gachas"
        : currentView === "downloads" ? "Downloads"
        : navTabs.tabs[navTabs.currentIndex]?.label || ""

    // clear search on view switch, but not on library filter tab flips (those dont change currentView, i think)
    onCurrentViewChanged: {
        topBar.searchText = ""
        topBar.defocusSearch()
    }

    function minimizeForLaunch() {
        steamStorePanel.keepAlive = false
        epicStorePanel.keepAlive = false
        gogStorePanel.keepAlive = false
        nileStorePanel.keepAlive = false
        hoyoStorePanel.keepAlive = false
        root.visible = false
        root.releaseResources()
        gameModel.trim_heap()
    }

    Timer {
        id: libPollTimer
        interval: 500
        repeat: true
        running: true
        onTriggered: {
            gameModel.check_exited_games()
            gameActions.refreshRunState()
            gameModel.drain_notifications()
            gameModel.drain_launch_requests()
            gameModel.drain_update_notifications()
            gameModel.drain_errors()
            gameModel.drain_install_sizes()
            gameModel.drain_game_details()
            gameModel.drain_game_log_events()
        }
    }

    Connections {
        target: gameModel
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

property real cardZoom: appSettings.cardZoom
    property string cardStyle: appSettings.cardStyle
    readonly property int cardBaseWidth: 180
    readonly property int cardBaseHeight: 240

    property real uiScale: appSettings.uiScale > 0 ? appSettings.uiScale : 1.0

    Shortcut {
        sequences: ["Ctrl+Plus", "Ctrl+Shift+=", "Ctrl+=", "Ctrl+Up"]
        onActivated: appSettings.applyUiScale(root.uiScale + 0.1)
    }
    Shortcut {
        sequences: ["Ctrl+-", "Ctrl+Shift+-", "Ctrl+Down"]
        onActivated: appSettings.applyUiScale(root.uiScale - 0.1)
    }
    Shortcut {
        sequence: "Ctrl+0"
        onActivated: appSettings.applyUiScale(1.0)
    }

    Item {
        id: scaledRoot
        width: root.width / root.uiScale
        height: root.height / root.uiScale
        transform: Scale { xScale: root.uiScale; yScale: root.uiScale; origin.x: 0; origin.y: 0 }

    MouseArea {
        anchors.fill: parent
        onClicked: forceActiveFocus()
    }

    NavTabs {
        id: navTabs
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        // above dropdown popups (z 50) so in-panel dropdowns dont bleed over the nav
        z: 100

        width: appSettings.navCollapsed ? 0 : appSettings.navWidth
        onWidthRequested: (v) => {
            if (v === 0) {
                // drag to zero = collapse, dont overwrite the remembered expanded width
                appSettings.applyNavCollapsed(true)
            } else {
                if (appSettings.navCollapsed) appSettings.applyNavCollapsed(false)
                appSettings.applyNavWidth(v)
            }
        }

        downloadCount: downloadModel.activeCount
        headerLabel: root.currentViewLabel

        appSettings: appSettings

        showSteam: appSettings.showSteam
        showEpic: appSettings.showEpic
        showGog: appSettings.showGog
        showNile: appSettings.showNile
        showGachas: appSettings.showGachas

        onCategoryMenuRequested: (sourceIndex, x, y) => categoryMenu.show(sourceIndex, x, y)

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
            } else if (storeName === "Nile") {
                navTabs.currentStore = "Nile"
                root.currentView = "nile"
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
        visible: appSettings.navCollapsed
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
            appSettings.applyNavCollapsed(false)
            const target = Math.max(navTabs.minWidth, Math.min(navTabs.maxWidth, mouse.x))
            appSettings.applyNavWidth(target)
        }
        onReleased: (mouse) => {
            // bare click restores at remebmered width
            if (!didDrag) appSettings.applyNavCollapsed(false)
        }

        Rectangle {
            anchors.left: parent.left
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            width: 2
            color: Theme.accent
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
        showTitle: appSettings.navCollapsed || navTabs.iconOnly
        leftInset: navTabs.width

        showAddButton: root.currentView === "library"
        showSearch: root.currentView === "library"
            || root.currentView === "steam"
            || root.currentView === "epic"
            || root.currentView === "gog"
            || root.currentView === "nile"
            || root.currentView === "hoyo"
        showDisplayOptions: root.currentView === "library"
            || root.currentView === "steam"
            || root.currentView === "epic"
            || root.currentView === "gog"
            || root.currentView === "nile"
            || root.currentView === "hoyo"
        zoomValue: appSettings.cardZoom
        spacingValue: appSettings.cardSpacing
        sortValue: appSettings.cardSort
        showSort: root.currentView === "library"
        showHiddenValue: appSettings.showHidden
        showHiddenOption: root.currentView === "library"
        cardStyleValue: appSettings.cardStyle

        onAddClicked: root.activeModal = "addGame"
        onInstallScriptClicked: scriptBrowserDialog.show()
        onConsoleModeClicked: gameModel.launch_console_mode()
        onZoomMoved: (v) => appSettings.applyCardZoom(v)
        onSpacingMoved: (v) => appSettings.applyCardSpacing(v)
        onSortSelected: (v) => appSettings.applyCardSort(v)
        onShowHiddenToggled: (v) => appSettings.applyShowHidden(v)
        onCardStyleSelected: (v) => appSettings.applyCardStyle(v)
    }

    Item {
        anchors.top: topBar.bottom
        anchors.left: navTabs.right
        anchors.right: parent.right
        anchors.bottom: parent.bottom

        GameLibraryView {
            id: libraryView
            gameModel: root.gameModelRef
            actions: gameActions
            active: root.currentView === "library"
            searchText: topBar.searchText
            filterKind: navTabs.tabs[navTabs.currentIndex]?.kind || "all"
            filterValue: navTabs.tabs[navTabs.currentIndex]?.value || ""
            showHidden: appSettings.showHidden
            dimHidden: appSettings.dimHidden
            cardZoom: root.cardZoom
            cardSpacing: appSettings.cardSpacing
            cardElevation: appSettings.cardElevation
            cardBaseWidth: root.cardBaseWidth
            cardBaseHeight: root.cardBaseHeight
            cardFlow: appSettings.cardFlow
            cardStyle: root.cardStyle
            cardSort: appSettings.cardSort
            doubleClickLaunches: appSettings.doubleClickLaunches
            onSelectionChanged: topBar.defocusSearch()
            onGameRightClicked: (index, winX, winY) => gameContextMenu.show(index, winX, winY)
            onSettingsRequested: (index) => {
                root.settingsGameIndex = index
                root.activeModal = "gameSettings"
            }
            onDownloadActivityClicked: root.currentView = "downloads"
            onWineToolsRequested: {
                if (!gameActions.selectedGame || !gameActions.selectedGame.gameId) return
                if (Date.now() - wineToolsMenu.lastClosedAt < 150) return
                wineToolsMenu.openAbove(libraryView.wineToolsAnchor)
            }
        }

        StorePanel {
            id: steamStorePanel
            viewName: "steam"
            currentView: root.currentView
            unloadIdle: appSettings.unloadStorePages
            onIdleUnloaded: gameModel.trim_heap()
            sourceComponent: SteamLibrary {
                gameModel: root.gameModelRef
                cardZoom: root.cardZoom
                cardStyle: root.cardStyle
                cardSpacing: appSettings.cardSpacing
                cardElevation: appSettings.cardElevation
                cardFlow: appSettings.cardFlow
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
            unloadIdle: appSettings.unloadStorePages
            onIdleUnloaded: gameModel.trim_heap()
            sourceComponent: EpicLibrary {
                storeModel: root.epicModelRef
                cardZoom: root.cardZoom
                cardStyle: root.cardStyle
                cardSpacing: appSettings.cardSpacing
                cardElevation: appSettings.cardElevation
                cardFlow: appSettings.cardFlow
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
            unloadIdle: appSettings.unloadStorePages
            onIdleUnloaded: gameModel.trim_heap()
            sourceComponent: GogLibrary {
                storeModel: root.gogModelRef
                cardZoom: root.cardZoom
                cardStyle: root.cardStyle
                cardSpacing: appSettings.cardSpacing
                cardElevation: appSettings.cardElevation
                cardFlow: appSettings.cardFlow
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
            id: nileStorePanel
            viewName: "nile"
            currentView: root.currentView
            unloadIdle: appSettings.unloadStorePages
            onIdleUnloaded: gameModel.trim_heap()
            sourceComponent: NileLibrary {
                storeModel: root.nileModelRef
                cardZoom: root.cardZoom
                cardStyle: root.cardStyle
                cardSpacing: appSettings.cardSpacing
                cardElevation: appSettings.cardElevation
                cardFlow: appSettings.cardFlow
                searchText: topBar.searchText
                activeDownloads: nileController.activeDownloads
                onBackClicked: {
                    navTabs.currentStore = ""
                    navTabs.currentIndex = 0
                    root.currentView = "library"
                }
                onInstallRequested: (index) => nileController.showInstall(index)
                onImportRequested: (index) => nileController.showInstall(index)
            }
        }

        StorePanel {
            id: hoyoStorePanel
            viewName: "hoyo"
            currentView: root.currentView
            unloadIdle: appSettings.unloadStorePages
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
                cardStyle: root.cardStyle
                cardSpacing: appSettings.cardSpacing
                cardElevation: appSettings.cardElevation
                cardFlow: appSettings.cardFlow
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
            color: Theme.surface
            radius: Theme.radius.md
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
                onPauseRequested: (id, displayName, atRisk) => {
                    pauseDownloadConfirm.message =
                        qsTr("Nile cannot resume a partial file. Pausing \"%1\" throws away the %2 it is currently writing.").arg(displayName).arg(atRisk)
                    pauseDownloadConfirm.show(id)
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

    NileController {
        id: nileController
        gameModel: root.gameModelRef
        nileModel: nileModel
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
        archiveManager: root.archiveManagerRef
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
        onPlayRequested: (idx) => gameActions.play(idx)
        onLogsRequested: (gid, gname) => root.openGameLogs(gid, gname)
        onConfigureRequested: (idx) => {
            gameActions.selectedIndex = idx
            root.settingsGameIndex = idx
            root.activeModal = "gameSettings"
        }
        onCategoriesRequested: (idx) => categoriesController.showForGame(idx)
        onRemoveRequested: (idx) => {
            if (gameActions.selectedIndex === idx) gameActions.selectedIndex = -1
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
        id: pauseDownloadConfirm
        anchors.fill: parent
        title: qsTr("Pause download?")
        confirmText: qsTr("Pause anyway")
        cancelText: qsTr("Keep downloading")
        destructive: true
        onConfirmed: (id) => { if (downloadModel) downloadModel.pause(id) }
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

    ImagePreviewDialog {
        id: imagePreviewDialog
        anchors.fill: parent
        title: qsTr("Art preview")
    }

    DefaultsApplyDialog {
        id: defaultsApplyDialog
        anchors.fill: parent
        defaults: defaultsBridge
        gameModel: root.gameModelRef
    }

    SetsDialog {
        id: envSetsDialog
        libRead: () => appSettings.envSetsJson()
        libWrite: (j) => appSettings.applyEnvSetsJson(j)
        copyKey: "launch.env"
        syncKey: "launch.env_sets"
        keyPlaceholder: "VAR_NAME"
        valuePlaceholder: "value"
        titleText: qsTr("Environment sets")
        manageTitle: qsTr("Manage env sets")
    }

    SetsDialog {
        id: dllSetsDialog
        libRead: () => appSettings.dllSetsJson()
        libWrite: (j) => appSettings.applyDllSetsJson(j)
        copyKey: "wine.dll_overrides"
        syncKey: "wine.dll_override_sets"
        keyPlaceholder: "dll_name"
        valuePlaceholder: "n,b"
        titleText: qsTr("DLL override sets")
        manageTitle: qsTr("Manage DLL sets")
    }

    FontSizesDialog {
        id: fontSizesDialog
        appSettings: root.appSettingsRef
    }

    RadiiDialog {
        id: radiiDialog
        appSettings: root.appSettingsRef
    }

    TemplateVarsDialog {
        id: templateVarsDialog
        anchors.fill: parent
        appSettings: root.appSettingsRef
        gameModel: root.gameModelRef
    }

    LogRulesDialog {
        id: logRulesDialog
        anchors.fill: parent
        appSettings: appSettings
    }

    CategoriesController {
        id: categoriesController
        appSettings: appSettings
        gameModel: gameModel
    }

    CategoryContextMenu {
        id: categoryMenu
        appSettings: appSettings
        onAddRequested: categoriesController.showAdd()
        onEditRequested: (idx, entry) => categoriesController.showEdit(idx, entry)
        onDeleteRequested: (idx, entry) => categoriesController.showDelete(idx, entry)
        onHideRequested: (idx) => categoriesController.setEnabled(idx, false)
    }

    ArchiveManageDialog {
        id: archiveManageDialog
        anchors.fill: parent
        archiveManager: archiveManager
        activeInstalls: root.archiveActiveInstalls
        latestCategory: root.latestCategory
        onLatestInstallRequested: (sourceName) => {
            root.latestToastMuted[sourceName] = true
            archiveManager.installLatest(sourceName)
        }
        onVersionDeleted: (category, sourceName, tag) => {
            if (category === "runners") root.runnersVersion++
        }
        onEditSourceRequested: (category, sourceName) => {
            let list = []
            try {
                list = JSON.parse(category === "runners"
                    ? archiveManager.listRunners()
                    : archiveManager.listDllPacks()) || []
            } catch (e) {
                return
            }
            let source = list.find(s => s.name === sourceName)
            if (!source) return
            archiveManageDialog.escEnabled = false
            archiveSourceDialog.showEdit(category, source)
        }

        onRemoveSourceRequested: (category, sourceName) => {
            removeSourceConfirm.message = qsTr("Removes \"%1\" from your sources. Installed versions stay on disk and keep working; adding a source with the same name picks them up again.").arg(sourceName)
            archiveManageDialog.escEnabled = false
            removeSourceConfirm.show({ category: category, source: sourceName })
        }
        onMoveToSteamRequested: (sourceName, tag) => {
            const dir = archiveManager.installedRunnerPath(sourceName, tag)
            steamMoveDialog.show(dir, tag, archiveManager.steamActionFor(dir))
        }
        onClosed: {
            if (!root.welcomeResumePending) return
            root.welcomeResumePending = false
            welcomeDialog.show()
        }
    }

    ArchiveSourceDialog {
        id: archiveSourceDialog
        anchors.fill: parent
        archiveManager: archiveManager
        onShownChanged: if (!shown) archiveManageDialog.escEnabled = true
    }

    FoundRunnersDialog {
        id: foundRunnersDialog
        anchors.fill: parent
        archiveManager: archiveManager
        onDeleteRunnerRequested: (name, dir) => {
            const err = archiveManager.deleteRunnerAt(dir)
            if (err !== "") toastManager.show("error", qsTr("Couldn't delete runner"), err)
            else toastManager.show("success", qsTr("%1 deleted").arg(name), "")
            foundRunnersDialog.refresh()
            root.runnersVersion++
        }
        onSteamActionRequested: (name, dir, action) => steamMoveDialog.show(dir, name, action)
    }

    SteamMoveDialog {
        id: steamMoveDialog
        anchors.fill: parent
        archiveManager: archiveManager
        onLinksApplied: (name, error) => {
            if (error !== "") {
                toastManager.show("error", qsTr("Couldn't update Steam links"), error)
                return
            }
            foundRunnersDialog.refresh()
            root.runnersVersion++
            toastManager.show("success", qsTr("Steam links updated for %1").arg(name), qsTr("Restart Steam to see it."))
        }
    }

    Connections {
        target: archiveManager
        function onMoveToSteamDone(name, error) {
            if (error && error.length > 0) return
            root.runnersVersion++
            archiveManageDialog.refreshInstalled()
            foundRunnersDialog.refresh()
            toastManager.show("success", qsTr("%1 moved to Steam").arg(name), qsTr("Restart Steam to see it."))
        }
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
            appSettings: root.appSettingsRef
            onWindowClosed: root.closeGameLogs(gameId)
        }
    }

    UpdateAvailableDialog {
        id: updateDialog
        anchors.fill: parent
        onUpdateRequested: (gid, aid, fromV) => {
            let newId = gameModel.enqueue_game_update(gid, fromV)
            if (newId && newId.length > 0) {
                toastManager.show("info", qsTr("Update queued"), gameActions.selectedGame ? gameActions.selectedGame.name : "")
            } else {
                toastManager.show("error", qsTr("Update failed"), qsTr("Could not enqueue update"))
            }
        }
        onRunAnywayRequested: (gid) => {
            let idx = gameModel.index_of_id(gid)
            if (idx >= 0) gameActions.play(idx, true)
        }
    }

    ChangelogDialog {
        id: changelogDialog
        anchors.fill: parent
        onDismissed: gameModel.mark_changelog_seen()
    }

    WelcomeDialog {
        id: welcomeDialog
        anchors.fill: parent
        appSettings: root.appSettingsRef
        componentsBridge: root.componentsBridgeRef
        archiveManager: root.archiveManagerRef
        onUmuInstallRequested: toastManager.show(
            "info",
            qsTr("Installing umu-run"),
            qsTr("See Downloads for progress.")
        )
        onManageRequested: (category, source, kind) => {
            root.welcomeResumePending = true
            welcomeDialog.close()
            archiveManageDialog.show(category, source, kind, true)
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

    ComponentRequiredDialog {
        id: componentRequiredDialog
        anchors.fill: parent
        componentsBridge: root.componentsBridgeRef
        onLaunchReady: (idx, skip) => gameActions.play(idx, skip)
    }

    PrefixPrepDialog {
        id: prefixPrepDialog
        anchors.fill: parent
        gameModel: root.gameModelRef
        onLaunchReady: (idx, skip) => gameActions.launch(idx, skip)
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
                ? qsTr("This deletes the prefix and everything in it. %n game(s) use it, and it won't be recoverable.", "", n)
                : qsTr("This deletes the prefix and everything in it. It won't be recoverable.")
            prefixDetailDialog.escEnabled = false
            deletePrefixConfirm.show(p)
        }
        onRunCommandRequested: (p) => {
            prefixDetailDialog.escEnabled = false
            ofudaRunCommandDialog.prefix = p
            ofudaRunCommandDialog.show("", p.path || "")
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

    RunCommandDialog {
        id: ofudaRunCommandDialog
        anchors.fill: parent
        property var prefix: ({})
        running: ofudaBridge.commandRunning
        onSubmitted: (cmd) => ofudaBridge.runCommand(prefix.path || "", prefix.runner || "", cmd)
        onShownChanged: if (!shown) prefixDetailDialog.escEnabled = true

        Connections {
            target: ofudaBridge
            function onCommandOutput(line) { ofudaRunCommandDialog.appendLine(line) }
            function onCommandFinished(ok, error) { ofudaRunCommandDialog.commandDone(ok, error) }
        }
    }

    RunCommandDialog {
        id: gameRunCommandDialog
        anchors.fill: parent
        property string gameId: ""
        expander: (t) => gameModel.expandGameVars(gameId, t)
        running: gameModel.wineCommandRunning
        onSubmitted: (cmd) => gameModel.run_wine_command(gameId, cmd)

        Connections {
            target: gameModel
            function onWineCommandOutput(line) { gameRunCommandDialog.appendLine(line) }
            function onWineCommandFinished(ok, error) { gameRunCommandDialog.commandDone(ok, error) }
        }
    }

    ContextMenu {
        id: wineToolsMenu

        items: [
            { text: qsTr("Configure (winecfg)"),    action: "winecfg" },
            { text: "Winetricks",                    action: "winetricks" },
            { text: qsTr("Registry (regedit)"),      action: "regedit" },
            { text: qsTr("Command Prompt (cmd)"),    action: "cmd" },
            { text: qsTr("File Explorer (explorer)"), action: "explorer" },
            { text: qsTr("Run EXE in prefix…"),      action: "run_exe" },
            { text: qsTr("Run wine command…"),       action: "run_command" },
            { text: qsTr("Kill wineserver"),         action: "killwineserver", danger: true }
        ]

        onItemClicked: (action) => {
            if (!gameActions.selectedGame || !gameActions.selectedGame.gameId) return
            let gid = gameActions.selectedGame.gameId
            if (action === "run_exe") {
                runExePicker.open()
            } else if (action === "run_command") {
                gameRunCommandDialog.gameId = gid
                gameRunCommandDialog.show(gameActions.selectedGame.name || "", gameActions.selectedGame.prefixPath || "")
            } else {
                gameModel.run_wine_tool(gid, action)
            }
        }

        FilePicker {
            id: runExePicker
            title: qsTr("Select EXE to run in prefix")
            startFolder: "/home"
            onPicked: (path) => {
                if (gameActions.selectedGame && gameActions.selectedGame.gameId) {
                    gameModel.run_wine_exe(gameActions.selectedGame.gameId, path)
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
                viewportHeight: gameSettingsModal.viewportHeight
                runnersVersion: root.runnersVersion
                gameIndex: root.settingsGameIndex
                envSetsDialog: root.envSetsDialogRef
                dllSetsDialog: root.dllSetsDialogRef
                onSaveRequested: (idx) => root.activeModal = ""
                onSaveAndPlayRequested: (idx) => {
                    root.activeModal = ""
                    gameActions.play(idx)
                }
                onRefetchMediaRequested: (gid) => refetchMediaConfirm.show(gid)
                onPreviewImageRequested: (src, caption) => imagePreviewDialog.show(src, caption)
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
                onPreviewImageRequested: (src, caption) => imagePreviewDialog.show(src, caption)
                onGameCreated: (gameId) => {
                    root.activeModal = ""
                    for (let i = 0; i < gameModel.count; i++) {
                        let g = gameModel.get_game(i)
                        if (g && g["gameId"] === gameId) {
                            gameActions.selectedIndex = i
                            break
                        }
                    }
                }
                onGameCreatedAndPlay: (gameId) => {
                    root.activeModal = ""
                    for (let i = 0; i < gameModel.count; i++) {
                        let g = gameModel.get_game(i)
                        if (g && g["gameId"] === gameId) {
                            gameActions.selectedIndex = i
                            gameActions.play(i)
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
                appSettings: root.appSettingsRef
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
                onManageFoundRunnersRequested: foundRunnersDialog.show()
                onManageSetsRequested: (kind) => {
                    if (kind === "vars") templateVarsDialog.open()
                    else (kind === "dll" ? dllSetsDialog : envSetsDialog).openManage()
                }
                onManageFontSizesRequested: fontSizesDialog.open()
                onManageRadiiRequested: radiiDialog.open()
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
        z: 99999
    }

    }
}

// TODO might just really need to spend a week just on un-spaghettifying the whole qml. just sayin
// TODO kill dropshadow so we can drop qt6-5compat
