pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

// a new gameModel call in EpicInstallDialog needs a copy in demoBridge or the demo dialog breaks at runtime
Item {
    id: root

    anchors.fill: parent

    required property Item pageArea
    property var gameModel: null
    property var defaults: null
    property int runnersVersion: 0
    property real cardZoom: 1.0
    property string cardStyle: "normal"
    property int cardSpacing: 16
    property string cardFlow: "center"

    ListModel {
        id: demoStore

        readonly property bool isLoggedIn: true
        readonly property bool isRefreshing: false
        readonly property bool toolReady: true
        readonly property bool toolInstalling: false
        readonly property string displayName: "TheHumbleMagicalWizardWhyIsThisNameSoLongPleaseMakeItStop9000"

        function refresh_tools() {}
        function refresh() {}
        function logout() {}
        function get_game_at(i) { return get(i) }
        function enqueue_install() { return "" }

        ListElement { title: "Wizard Goose"; appName: "demo-goose"; isInstalled: false; hasLibraryEntry: false; coverart: ""; banner: ""; installPath: "" }
        ListElement { title: "Honkers: Stair Chairs"; appName: "demo-honkers"; isInstalled: false; hasLibraryEntry: false; coverart: ""; banner: ""; installPath: "" }
        ListElement { title: "Moyu Café"; appName: "demo-moyu"; isInstalled: false; hasLibraryEntry: false; coverart: ""; banner: ""; installPath: "" }
    }

    QtObject {
        id: demoBridge

        signal install_size_result(string requestId, string payload)
        signal game_details_result(string requestId, string payload)

        function home_dir() { return root.gameModel ? root.gameModel.home_dir() : "" }
        function list_runners() { return root.gameModel ? root.gameModel.list_runners() : "[]" }
        function disk_free_space(path) { return root.gameModel ? root.gameModel.disk_free_space(path) : "-1" }
        function expandVars(text) { return root.gameModel ? root.gameModel.expandVars(text) : text }

        function epic_check_existing_install(app, path) { return "{}" }
        function epic_dir_has_game(exe, path) { return false }
        function installed_dlcs(app) { return "[]" }
        function fetch_epic_game_details(requestId, app) {}

        function fetch_epic_install_size(requestId, app) {
            sizeReply.requestId = requestId
            sizeReply.restart()
        }
    }

    Timer {
        id: sizeReply
        property string requestId: ""
        interval: 700
        onTriggered: demoBridge.install_size_result(requestId, JSON.stringify({
            download: 2400000000,
            install: 3100000000,
            launchExe: "",
            dlcs: JSON.stringify([
                { id: "demo-dlc-hats", title: "Goose Hats Pack" },
                { id: "demo-dlc-honk", title: "Extra Honks" }
            ])
        }))
    }

    Rectangle {
        id: page
        x: root.pageArea.x
        y: root.pageArea.y
        width: root.pageArea.width
        height: root.pageArea.height
        color: Theme.bg

        SpotlightTarget { key: "demo.store"; target: page }

        EpicLibrary {
            anchors.fill: parent
            storeModel: demoStore
            cardZoom: root.cardZoom
            cardStyle: root.cardStyle
            cardSpacing: root.cardSpacing
            cardFlow: root.cardFlow
            firstCardSpotlightKey: "demo.store.card"
            onInstallRequested: (index) => {
                dialog.gameIndex = index
                dialog.show()
            }
        }
    }

    EpicInstallDialog {
        id: dialog
        gameModel: demoBridge
        epicModel: demoStore
        defaults: root.defaults
        runnersVersion: root.runnersVersion
        onCancelled: hide()
    }
}
