import QtQuick

Item {
    id: root

    property var gameModel: null
    property int gameIndex: -1
    property var envSetsDialog: null
    property var dllSetsDialog: null

    // TabRunnerOptions re-queries list_runners without restart when this bumps
    property int runnersVersion: 0

    // stored separately becuase index can shift during refresh
    property string gameId: ""

    property var gameData: null
    property var config: ({})

    readonly property string modalTitle: gameData ? (gameData["name"] || "") : ""
    readonly property string modalSubtitle: gameId
    readonly property string primaryLabel: qsTr("Save & Play")
    readonly property string secondaryLabel: qsTr("Save")
    readonly property bool primaryEnabled: true
    readonly property bool secondaryEnabled: true

    signal saveRequested(int gameIndex)
    signal saveAndPlayRequested(int gameIndex)
    signal refetchMediaRequested(string gameId)

    property var tabs: {
        let base = [
            { label: qsTr("Game Info"), kind: "info",   icon: "sports_esports" },
            { label: qsTr("Runner"),    kind: "runner", icon: "wine_bar" }
        ]
        let isFlatpakLauncher = gameModel ? gameModel.is_flatpak() : false
        let isSteamGame = root.config["runner.type"] === "steam"
        if (!(isFlatpakLauncher && isSteamGame)) {
            base.push({ label: qsTr("System"), kind: "system", icon: "terminal" })
        }
        if (root.config["source.kind"] === "epic") {
            base.push({ label: "Epic", kind: "epic", icon: "shield_moon" })
        }
        if (root.config["source.kind"] === "gog") {
            base.push({ label: "GOG", kind: "gog", icon: "gog" })
        }
        return base
    }
    property int currentTabIndex: 0
    readonly property string currentKind:
        tabs[currentTabIndex] ? tabs[currentTabIndex].kind : "info"

    onTabsChanged: if (currentTabIndex >= tabs.length) currentTabIndex = 0

    onGameIndexChanged: loadGame()
    Component.onCompleted: loadGame()
    Component.onDestruction: {
        if (gameModel) gameModel.discard_draft()
    }

    function loadGame() {
        if (!gameModel || gameIndex < 0) {
            gameData = null
            config = {}
            gameId = ""
            return
        }
        let data = gameModel.get_game(gameIndex)
        gameData = data
        gameId = data ? data["gameId"] : ""
        config = gameModel.begin_edit_game(gameIndex)
    }

    function save() {
        if (gameModel && gameId !== "") {
            gameModel.commit_edit_game(gameId)
        }
        root.saveRequested(gameIndex)
    }

    function saveAndPlay() {
        save()
        root.saveAndPlayRequested(gameIndex)
    }

    function primaryAction() { saveAndPlay() }
    function secondaryAction() { save() }
    function closeAction() { if (gameModel) gameModel.discard_draft() }

    function updateField(key, value) {
        if (gameModel && gameId !== "") {
            let strVal = String(value)
            if (gameModel.update_draft_field(key, strVal)) {
                let next = gameModel.get_draft_config()
                if (key === "launch.args" || key === "launch.alongside_args") next[key] = strVal
                config = next
            }
        }
    }

    function openEnvSets() {
        if (envSetsDialog) envSetsDialog.openForGame(root.config["launch.env"] || "{}", root.config["launch.env_sets"] || "[]", root.updateField)
    }

    function openDllSets() {
        if (dllSetsDialog) dllSetsDialog.openForGame(root.config["wine.dll_overrides"] || "{}", root.config["wine.dll_override_sets"] || "[]", root.updateField)
    }

    // lookup by id because index may shift after a refresh
    function findIndex() {
        if (!gameModel || gameId === "") return -1
        for (let i = 0; i < gameModel.count; i++) {
            let g = gameModel.get_game(i)
            if (g && g["gameId"] === gameId) return i
        }
        return -1
    }

    function refreshConfig() {
        let idx = findIndex()
        if (idx >= 0) config = gameModel.begin_edit_game(idx)
    }

    property real viewportHeight: 0

    implicitHeight: contentCol.implicitHeight

    Column {
        id: contentCol
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: 0

        Loader {
            width: parent.width
            active: root.currentKind === "info"
            visible: active
            source: "../settings/TabGameInfo.qml"
            onLoaded: {
                item.config = Qt.binding(() => root.config)
                item.updateField = root.updateField
                item.gameModel = Qt.binding(() => root.gameModel)
                item.refetchMediaRequested.connect(() => root.refetchMediaRequested(root.gameId))
            }
        }

        Loader {
            width: parent.width
            active: root.currentKind === "runner"
            visible: active
            source: "../settings/TabRunnerOptions.qml"
            onLoaded: {
                item.config = Qt.binding(() => root.config)
                item.updateField = root.updateField
                item.gameModel = Qt.binding(() => root.gameModel)
                item.runnersVersion = Qt.binding(() => root.runnersVersion)
                item.openDllSets = root.openDllSets
            }
        }

        Loader {
            width: parent.width
            active: root.currentKind === "system"
            visible: active
            source: "../settings/TabSystem.qml"
            onLoaded: {
                item.config = Qt.binding(() => root.config)
                item.updateField = root.updateField
                item.gameModel = Qt.binding(() => root.gameModel)
                item.openEnvSets = root.openEnvSets
            }
        }

        Loader {
            width: parent.width
            active: root.currentKind === "epic"
            visible: active
            source: "../settings/TabEpic.qml"
            onLoaded: {
                item.config = Qt.binding(() => root.config)
                item.updateField = root.updateField
                item.refreshConfig = root.refreshConfig
                item.gameModel = Qt.binding(() => root.gameModel)
                item.gameId = Qt.binding(() => root.gameId)
            }
        }

        Loader {
            width: parent.width
            active: root.currentKind === "gog"
            visible: active
            source: "../settings/TabGog.qml"
            onLoaded: {
                item.config = Qt.binding(() => root.config)
                item.gameModel = Qt.binding(() => root.gameModel)
                item.gameId = Qt.binding(() => root.gameId)
                item.viewportHeight = Qt.binding(() => root.viewportHeight)
            }
        }
    }
}
