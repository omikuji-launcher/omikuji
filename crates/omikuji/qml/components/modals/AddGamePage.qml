import QtQuick

Item {
    id: root

    property var gameModel: null
    property var envSetsDialog: null
    property var dllSetsDialog: null

    // forwarded to TabRunnerOptions so the runner dropdown refreshes after an install without restart
    property int runnersVersion: 0

    property var config: ({})

    // set by save() on success so saveAndPlay can locate the new game
    property string newGameId: ""

    readonly property bool canSave: (config["meta.name"] || "").trim().length > 0

    readonly property string modalTitle: qsTr("New Game")
    readonly property string modalSubtitle: ""
    readonly property string primaryLabel: qsTr("Create & Play")
    readonly property string secondaryLabel: qsTr("Create")
    readonly property bool primaryEnabled: canSave
    readonly property bool secondaryEnabled: canSave

    signal gameCreated(string gameId)
    signal gameCreatedAndPlay(string gameId)

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
        return base
    }
    property int currentTabIndex: 0
    readonly property string currentKind:
        tabs[currentTabIndex] ? tabs[currentTabIndex].kind : "info"

    onTabsChanged: if (currentTabIndex >= tabs.length) currentTabIndex = 0

    Component.onCompleted: startDraft()

    function startDraft() {
        if (!gameModel) return
        config = gameModel.begin_new_game()
    }

    function updateField(key, value) {
        if (!gameModel) return
        let strVal = String(value)
        if (gameModel.update_draft_field(key, strVal)) {
            let next = gameModel.get_draft_config()
            if (key === "launch.args") next["launch.args"] = strVal
            config = next
        }
    }

    function openEnvSets() {
        if (envSetsDialog) envSetsDialog.openForGame(root.config["launch.env"] || "{}", root.config["launch.env_sets"] || "[]", root.updateField)
    }

    function openDllSets() {
        if (dllSetsDialog) dllSetsDialog.openForGame(root.config["wine.dll_overrides"] || "{}", root.config["wine.dll_override_sets"] || "[]", root.updateField)
    }

    // returns new game id or empty on validation failure, draft is presevred
    function save() {
        if (!gameModel) return ""
        let id = gameModel.commit_new_game()
        if (id && id.length > 0) {
            newGameId = id
            root.gameCreated(id)
            return id
        }
        return ""
    }

    function saveAndPlay() {
        let id = save()
        if (id && id.length > 0) {
            root.gameCreatedAndPlay(id)
        }
    }

    function primaryAction() { saveAndPlay() }
    function secondaryAction() { save() }
    function closeAction() { if (gameModel) gameModel.discard_draft() }

    implicitHeight: contentCol.implicitHeight

    Column {
        id: contentCol
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: 0

        // all three stay active so field state survives tab switches, a fresh Loader would lose user input
        Loader {
            width: parent.width
            active: true
            visible: root.currentKind === "info"
            source: "../settings/TabGameInfo.qml"
            onLoaded: {
                item.config = Qt.binding(() => root.config)
                item.updateField = root.updateField
                item.gameModel = Qt.binding(() => root.gameModel)
            }
        }

        Loader {
            width: parent.width
            active: true
            visible: root.currentKind === "runner"
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
            active: true
            visible: root.currentKind === "system"
            source: "../settings/TabSystem.qml"
            onLoaded: {
                item.config = Qt.binding(() => root.config)
                item.updateField = root.updateField
                item.gameModel = Qt.binding(() => root.gameModel)
                item.openEnvSets = root.openEnvSets
            }
        }
    }
}
