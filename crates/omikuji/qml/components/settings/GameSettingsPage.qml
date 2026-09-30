import QtQuick
import omikuji 1.0
import "../lib/RunnerGrouping.js" as RG

Item {
    id: root

    property var gameModel: null
    property var downloadModel: null
    property int gameIndex: -1
    property var envSetsDialog: null
    property var dllSetsDialog: null

    // TabRunnerOptions re-queries list_runners without restart when this bumps
    property int runnersVersion: 0

    // stored separately becuase index can shift during refresh
    property string gameId: ""

    property var gameData: null
    property var config: ({})
    property bool draftLoaded: false
    property bool dirty: false

    readonly property string modalTitle: gameData ? (gameData["name"] || "") : ""
    readonly property string modalSubtitle: gameId
    readonly property string primaryLabel: qsTr("Save & Play")
    readonly property string secondaryLabel: qsTr("Save")
    readonly property bool primaryEnabled: dirty
    readonly property bool secondaryEnabled: dirty

    signal saveRequested(int gameIndex)
    signal saveAndPlayRequested(int gameIndex)
    signal refetchMediaRequested(string gameId)
    signal previewImageRequested(string source, string caption)
    signal pickMediaRequested(string gameId, string kind)
    signal confirmRequested(string title, string message, string confirmText, var onConfirm)

    property var tabs: {
        let base = [
            { label: qsTr("Game Info"), kind: "info",   icon: "sports_esports" },
            { label: qsTr("Runner"),    kind: "runner", icon: RG.runnerIcon(root.config["runner.type"]) }
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
        if (root.hasGachaTab) {
            base.push({ label: qsTr("Gacha"), kind: "gacha", icon: "local_activity" })
        }
        return base
    }
    readonly property bool hasGachaTab: root.config["source.kind"] === "gacha"
        && gameModel !== null && gameId !== ""
        && (JSON.parse(gameModel.gacha_packs(gameId) || "[]").length > 0
            || JSON.parse(gameModel.gacha_launch_controls() || "[]").length > 0)
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
            draftLoaded = false
            gameData = null
            config = {}
            gameId = ""
            return
        }
        let data = gameModel.get_game(gameIndex)
        gameData = data
        gameId = data ? data["gameId"] : ""
        config = gameModel.begin_edit_game(gameIndex)
        draftLoaded = true
        refreshDirty()
    }

    function refreshDirty() {
        dirty = gameModel ? gameModel.draft_dirty() : false
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
                refreshDirty()
            }
        }
    }

    function openEnvSets() {
        if (envSetsDialog) envSetsDialog.openForGame(root.config["launch.env"] || "{}", root.config["launch.env_sets"] || "[]", root.updateField)
    }

    function openDllSets() {
        if (dllSetsDialog) dllSetsDialog.openForGame(root.config["wine.dll_overrides"] || "{}", root.config["wine.dll_override_sets"] || "[]", root.updateField)
    }

    Connections {
        target: root.gameModel
        function onDraftRebased(gameId) {
            if (gameId !== root.gameId) return
            root.config = root.gameModel.get_draft_config()
            root.refreshDirty()
        }
        function onMediaChanged(gameId) {
            if (gameId === root.gameId) root.refreshDirty()
        }
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
            active: root.draftLoaded && root.currentKind === "info"
            visible: active
            sourceComponent: TabGameInfo {
                config: root.config
                updateField: root.updateField
                gameModel: root.gameModel
                gameId: root.gameId
                onRefetchMediaRequested: root.refetchMediaRequested(root.gameId)
                onPreviewImageRequested: (source, caption) => root.previewImageRequested(source, caption)
                onPickMediaRequested: (gameId, kind) => root.pickMediaRequested(gameId, kind)
            }
        }

        Loader {
            width: parent.width
            active: root.draftLoaded && root.currentKind === "runner"
            visible: active
            sourceComponent: TabRunnerOptions {
                config: root.config
                updateField: root.updateField
                gameModel: root.gameModel
                runnersVersion: root.runnersVersion
                openDllSets: root.openDllSets
            }
        }

        Loader {
            width: parent.width
            active: root.draftLoaded && root.currentKind === "system"
            visible: active
            sourceComponent: TabSystem {
                config: root.config
                updateField: root.updateField
                gameModel: root.gameModel
                openEnvSets: root.openEnvSets
            }
        }

        Loader {
            width: parent.width
            active: root.draftLoaded && root.currentKind === "epic"
            visible: active
            sourceComponent: TabEpic {
                config: root.config
                updateField: root.updateField
                gameModel: root.gameModel
                gameId: root.gameId
            }
        }

        Loader {
            width: parent.width
            active: root.draftLoaded && root.currentKind === "gog"
            visible: active
            sourceComponent: TabGog {
                config: root.config
                gameModel: root.gameModel
                gameId: root.gameId
                viewportHeight: root.viewportHeight
            }
        }

        Loader {
            width: parent.width
            active: root.draftLoaded && root.currentKind === "gacha"
            visible: active
            sourceComponent: TabGacha {
                config: root.config
                gameModel: root.gameModel
                downloadModel: root.downloadModel
                gameId: root.gameId
                onConfirmRequested: (title, message, confirmText, onConfirm) => root.confirmRequested(title, message, confirmText, onConfirm)
            }
        }
    }
}
