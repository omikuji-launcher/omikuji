pragma ComponentBehavior: Bound

import QtQuick
import "../lib/PlayState.js" as PlayState


QtObject {
    id: root

    property var gameModel: null
    property var downloadModel: null
    property var updatingRunners: ({})

    property int selectedIndex: -1
    property var selectedGame: null
    property string selectedGameId: ""
    property bool isRunning: false
    property bool isLaunching: false
    property var downloadActivity: null

    readonly property bool hasSelection: root.selectionValid()
    readonly property bool hasActivity:
        root.downloadActivity !== null && root.downloadActivity !== undefined
    readonly property bool runnerUpdating:
        root.selectedGame !== null && root.updatingRunners[root.selectedGame.runner] === true

    readonly property int playState:
        PlayState.stateFor(root.isLaunching, root.isRunning, root.hasActivity)

    signal downloadInFlight()
    signal componentRequired(int index, bool skipUpdateCheck, string missing)
    signal prefixPrepRequired(int index, bool skipUpdateCheck)
    signal forceLaunched(int index)

    function selectionValid() {
        return root.gameModel !== null
            && root.selectedIndex >= 0
            && root.selectedIndex < root.gameModel.count
    }

    function refreshRunState() {
        let valid = root.selectionValid()
        root.isRunning = valid && root.gameModel.is_running(root.selectedIndex)
        root.isLaunching = valid && root.gameModel.is_launching(root.selectedIndex)
    }

    function resyncSelection() {
        if (root.selectedGameId === "") return
        for (let i = 0; i < root.gameModel.count; i++) {
            let game = root.gameModel.get_game(i)
            if (game && game["gameId"] === root.selectedGameId) {
                root.selectedIndex = i
                return
            }
        }
        root.selectedIndex = -1
    }

    function updateSelection() {
        if (!root.selectionValid()) {
            root.selectedGame = null
            return
        }
        let game = root.gameModel.get_game(root.selectedIndex)
        root.selectedGame = {
            name: game["name"],
            playtime: game["playtime"] || 0,
            lastPlayed: game["lastPlayed"] || "",
            runner: game["runner"] || "",
            runnerType: game["runnerType"] || "",
            gameId: game["gameId"] || "",
            sourceAppId: game["sourceAppId"] || "",
            prefixPath: game["prefixPath"] || ""
        }
        root.refreshDownloadActivity()
    }

    function refreshDownloadActivity() {
        if (!root.selectedGame || !root.selectedGame.gameId) {
            root.downloadActivity = null
            return
        }
        let raw = root.downloadModel.active_for_game_id(root.selectedGame.gameId)
        if (!raw || raw.length === 0) {
            root.downloadActivity = null
            return
        }
        try {
            root.downloadActivity = JSON.parse(raw)
        } catch (e) {
            console.warn("active_for_game_id returned bad json:", raw)
            root.downloadActivity = null
        }
    }

    // redirects to downloads if an install is in flight, launching mid-patch would read files teh patcher is rewriting
    function play(index, forceSkipUpdateCheck = false) {
        if (root.gameModel === null || index < 0 || index >= root.gameModel.count) return false
        let game = root.gameModel.get_game(index)
        let gid = game ? (game["gameId"] || "") : ""
        if (gid.length > 0) {
            let raw = root.downloadModel.active_for_game_id(gid)
            if (raw && raw.length > 0) {
                root.downloadInFlight()
                return false
            }
        }
        let missing = root.gameModel.missing_component(index)
        if (missing.length > 0) {
            root.componentRequired(index, forceSkipUpdateCheck, missing)
            return true
        }
        if (root.gameModel.needs_prefix_prep(index)) {
            root.prefixPrepRequired(index, forceSkipUpdateCheck)
            return true
        }
        return root.launch(index, forceSkipUpdateCheck)
    }

    function launch(index, forceSkipUpdateCheck) {
        if (forceSkipUpdateCheck) {
            if (!root.gameModel.launch_game_force(index)) return false
            root.isLaunching = true
            root.forceLaunched(index)
            return true
        }
        if (!root.gameModel.launch_game(index)) return false
        root.isLaunching = true
        return true
    }

    function stop() {
        if (root.selectedGameId.length === 0) return false
        return root.gameModel.stop_game(root.selectedGameId)
    }

    onSelectedIndexChanged: {
        let game = root.selectionValid() ? root.gameModel.get_game(root.selectedIndex) : null
        root.selectedGameId = (game && game["gameId"]) ? game["gameId"] : ""
        root.refreshRunState()
        root.updateSelection()
    }

    property Timer launchPoll: Timer {
        interval: 60
        repeat: true
        running: root.isLaunching
        onTriggered: root.refreshRunState()
    }
}
