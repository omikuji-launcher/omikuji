import QtQuick
import QtQuick.Controls
import QtQuick.Window

import omikuji 1.0
import "components"
import "components/consolemode"

ApplicationWindow {
    id: root

    visibility: Window.FullScreen
    visible: true
    title: "Omikuji"
    color: theme.surface

    flags: Qt.Window

    readonly property real uiScale: Math.max(0.5, Math.min(width / 1920, height / 1080, 2.0))

    readonly property string consoleBackground: uiSettings.consoleBackground || "wave"

    property bool oskOpen: false
    readonly property bool oskVisible: topBar.searchExpanded && oskOpen

    property bool gamepadActive: false

    onGamepadActiveChanged: {
        if (!gamepadActive) {
            topBar.clearFocus()
            settingsDialog.clearFocus()
        }
    }

    HoverHandler {
        id: mouseTracker
    }

    Timer {
        interval: 80
        running: root.visible
        repeat: true
        property real _lx: 0
        property real _ly: 0
        onTriggered: {
            if (!mouseTracker.point) return
            const x = mouseTracker.point.position.x
            const y = mouseTracker.point.position.y
            if (x !== _lx || y !== _ly) {
                _lx = x
                _ly = y
                if (root.gamepadActive) root.gamepadActive = false
            }
        }
    }

    Theme {
        id: theme
        mutedIcons: uiSettings.mutedIcons
        followSystemColors: uiSettings.followSystemColors
        followSystemFont: uiSettings.followSystemFont
        fontFamily: uiSettings.fontFamily
    }

    Connections {
        target: uiSettings
        function onThemeChanged() {
            theme.overrides = JSON.parse(uiSettings.overridesJson())
        }
    }

    GameModel {
        id: gameModel

        onGame_stopped: (stoppedId) => {
            cardRow.markGameStopped(stoppedId)
            if (!root.visible) {
                root.visible = true
                root.raise()
                root.requestActivate()
            }
        }
    }

    UiSettingsBridge {
        id: uiSettings
        Component.onCompleted: {
            initWatcher()
            theme.overrides = JSON.parse(overridesJson())
        }
    }

    GamepadBridge {
        id: gamepad
        Component.onCompleted: start()

        onButton_pressed: (name) => root.handleGamepadButton(name)
    }

    function handleGamepadButton(name) {
        if (!root.active) return
        root.gamepadActive = true

        if (settingsDialog.visible) {
            switch (name) {
                case "dpad_up":    settingsDialog.handleDpadUp(); break
                case "dpad_down":  settingsDialog.handleDpadDown(); break
                case "south":      settingsDialog.handleAPress(); break
                case "east":       settingsDialog.handleBPress(); break
            }
            return
        }

        if (root.oskVisible) {
            switch (name) {
                case "dpad_left":  osk.moveLeft(); break
                case "dpad_right": osk.moveRight(); break
                case "dpad_up":    osk.moveUp(); break
                case "dpad_down":  osk.moveDown(); break
                case "south":      osk.tapFocused(); break
                case "east":       topBar.toggleSearch(); break
                case "west":       topBar.searchClear(); break
                case "lb":         topBar.searchBackspace(); break
                case "rb":         topBar.submitSearch(); break
                case "start":      topBar.submitSearch(); break
            }
            return
        }

        if (topBar.focusIndex >= 0) {
            switch (name) {
                case "dpad_left":  topBar.focusPrev(); break
                case "dpad_right": topBar.focusNext(); break
                case "south":      activateTopBarFocused(); break
                case "east":       topBar.clearFocus(); break
                case "north":      topBar.clearFocus(); break
            }
            return
        }

        switch (name) {
            case "dpad_left":  cardRow.navLeft(); break
            case "dpad_right": cardRow.navRight(); break
            case "dpad_up":    cardRow.prevCategory(); break
            case "dpad_down":  cardRow.nextCategory(); break
            case "lb":         cardRow.prevCategory(); break
            case "rb":         cardRow.nextCategory(); break
            case "south":      cardRow.toggleFocused(); break
            case "east":       if (topBar.searchExpanded) topBar.toggleSearch(); else gameModel.launch_desktop_mode(); break
            case "north":      topBar.focusIndex = 0; break
        }
    }

    function activateTopBarFocused() {
        const idx = topBar.focusIndex
        topBar.clearFocus()
        if (idx === 0) {
            if (!topBar.searchExpanded) topBar.toggleSearch()
            root.oskOpen = true
        } else if (idx === 1) {
            settingsDialog.open()
            settingsDialog.focusFirst()
        }
    }

    Timer {
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

    ShaderEffect {
        id: background
        anchors.fill: parent
        visible: root.consoleBackground !== "hero"

        property real time: 0
        property size resolution: Qt.size(width, height)
        property color accentColor: theme.accent
        property color baseColor: theme.surface

        fragmentShader: visible
            ? "qrc:/qt/qml/omikuji/qml/components/consolemode/shaders/" + root.consoleBackground + ".frag.qsb"
            : ""

        NumberAnimation on time {
            from: 0
            to: 1000000
            duration: 1000000000
            loops: Animation.Infinite
            running: root.visible && background.visible
        }
    }

    Item {
        id: heroBackground
        anchors.fill: parent
        visible: root.consoleBackground === "hero"

        Rectangle {
            anchors.fill: parent
            color: theme.surface
        }

        Image {
            id: heroImage
            anchors.fill: parent
            source: cardRow.focusedBanner
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
            cache: true
            visible: false
        }

        ShaderEffect {
            anchors.fill: parent
            property variant src: heroImage
            fragmentShader: heroBackground.visible
                ? "qrc:/qt/qml/omikuji/qml/components/consolemode/shaders/hero.frag.qsb"
                : ""
            visible: heroImage.status === Image.Ready
        }
    }

    ConsoleTopBar {
        id: topBar
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.topMargin: 28 * root.uiScale
        anchors.rightMargin: 36 * root.uiScale
        uiScale: root.uiScale

        onSearchSubmitted: { root.oskOpen = false; cardRow.focusList() }
        onSearchClosed: { root.oskOpen = false; cardRow.focusList() }
        onAppIconClicked: settingsDialog.open()
    }

    ConsoleSettingsDialog {
        id: settingsDialog
        anchors.fill: parent
        currentBackground: root.consoleBackground
        onBackgroundSelected: (name) => uiSettings.applyConsoleBackground(name)
    }

    ConsoleCardRow {
        id: cardRow
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.topMargin: 110 * root.uiScale
        gameModelRef: gameModel
        uiSettingsRef: uiSettings
        uiScale: root.uiScale
        searchText: topBar.searchText

        onLaunchRequested: (gameId) => root.tryPlay(gameId)
        onStopRequested: (gameId) => root.tryStop(gameId)
        onEscapePressed: gameModel.launch_desktop_mode()
    }

    ConsoleHintBar {
        id: hintBar
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottomMargin: 28 * root.uiScale
        uiScale: root.uiScale
        controllerKind: gamepad.controllerKind
        visible: !root.oskVisible
    }

    ConsoleOsk {
        id: osk
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 32 * root.uiScale
        uiScale: root.uiScale
        opacity: root.oskVisible ? 1 : 0
        visible: opacity > 0.01
        scale: root.oskVisible ? 1 : 0.92
        Behavior on opacity { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
        Behavior on scale { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }

        onKeyPressed: (ch) => topBar.searchAppendChar(ch)
        onBackspaceRequested: topBar.searchBackspace()
        onSpaceRequested: topBar.searchAddSpace()
        onSubmitRequested: topBar.submitSearch()
    }

    function tryPlay(gameId) {
        if (!gameId || gameId.length === 0) return false
        let idx = gameModel.index_of_id(gameId)
        if (idx < 0) return false
        if (gameModel.launch_game(idx)) {
            cardRow.markFocusedRunning()
            if (uiSettings.minimizeOnLaunch) {
                root.visible = false
                root.releaseResources()
                gameModel.trim_heap()
            }
            return true
        }
        return false
    }

    function tryStop(gameId) {
        if (!gameId || gameId.length === 0) return
        gameModel.stop_game(gameId)
    }


}
