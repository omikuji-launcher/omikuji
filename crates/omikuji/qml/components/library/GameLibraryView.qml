pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../lib/RunnerGrouping.js" as RG


Rectangle {
    id: root

    property bool isDropdownHost: true

    property var gameModel: null
    property var actions: null
    property bool active: false

    property string searchText: ""
    property string filterKind: "all"
    property string filterValue: ""
    property bool showHidden: false
    property bool dimHidden: false

    property real cardZoom: 1.0
    property int cardSpacing: 16
    property string cardFlow: "center"
    property string cardStyle: "normal"
    property bool cardElevation: false
    property int cardBaseWidth: 180
    property int cardBaseHeight: 240
    property string cardSort: "default"
    property bool doubleClickLaunches: false

    readonly property alias wineToolsAnchor: floatingBar.wineToolsAnchor

    signal gameRightClicked(int index, real winX, real winY)
    signal selectionChanged()
    signal settingsRequested(int index)
    signal downloadActivityClicked()
    signal wineToolsRequested()

    property var _recentIds: ({})

    function _recomputeRecent() {
        let dated = []
        for (let i = 0; i < root.gameModel.count; i++) {
            let g = root.gameModel.get_game(i)
            if (!g) continue
            let ts = Date.parse(g.lastPlayed || "") || 0
            if (ts > 0) dated.push({ id: g.gameId, ts: ts })
        }
        dated.sort((a, b) => b.ts - a.ts)
        let next = {}
        for (let i = 0; i < Math.min(10, dated.length); i++) next[dated[i].id] = true
        root._recentIds = next
    }

    function gamePassesFilter(index) {
        if (!root.gameModel) return true
        let game = root.gameModel.get_game(index)
        if (!game) return false

        switch (root.filterKind) {
            case "all":       return true
            case "favourite": return game.favourite === true
            case "recent":    return root._recentIds[game.gameId] === true
            case "runner":    return RG.runnerBucket(game.runnerType) === root.filterValue
            case "tag": {
                let cats = []
                try { cats = JSON.parse(game.categories || "[]") } catch (e) { cats = [] }
                return cats.indexOf(root.filterValue) !== -1
            }
            default: return true
        }
    }

    onFilterKindChanged: if (root.filterKind === "recent") root._recomputeRecent()

    Connections {
        target: root.gameModel
        function onDataChanged() { if (root.filterKind === "recent") root._recomputeRecent() }
        function onRowsInserted() { if (root.filterKind === "recent") root._recomputeRecent() }
        function onRowsRemoved() { if (root.filterKind === "recent") root._recomputeRecent() }
    }

    anchors.fill: parent
    color: Theme.surface
    radius: Theme.radius.md
    visible: opacity > 0
    opacity: root.active ? 1 : 0

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
        model: root.gameModel
        gameModel: root.gameModel
        view: root
        selectedIndex: root.actions ? root.actions.selectedIndex : -1
        cardZoom: root.cardZoom
        cardSpacing: root.cardSpacing
        cardElevation: root.cardElevation
        cardBaseWidth: root.cardBaseWidth
        cardBaseHeight: root.cardBaseHeight
        cardFlow: root.cardFlow
        cardStyle: root.cardStyle
        cardSort: root.cardSort
        showHidden: root.showHidden
        dimHidden: root.dimHidden
        searchText: root.searchText
        filterKind: root.filterKind
        filterValue: root.filterValue
        onGameClicked: (index) => {
            if (root.actions) root.actions.selectedIndex = index
            root.selectionChanged()
        }
        onGameDoubleClicked: (index) => {
            if (root.doubleClickLaunches && root.actions) root.actions.play(index)
        }
        onGameRightClicked: (index, winX, winY) => root.gameRightClicked(index, winX, winY)
        onBackgroundClicked: {
            if (root.actions) root.actions.selectedIndex = -1
            root.selectionChanged()
        }
    }

    FloatingBar {
        id: floatingBar
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        actions: root.actions
        selectedGame: root.actions ? root.actions.selectedGame : null
        hasSelection: root.actions ? root.actions.hasSelection : false
        isRunning: root.actions ? root.actions.isRunning : false
        isLaunching: root.actions ? root.actions.isLaunching : false
        runnerUpdating: root.actions ? root.actions.runnerUpdating : false
        downloadActivity: root.actions ? root.actions.downloadActivity : null
        onSettingsClicked: root.settingsRequested(root.actions ? root.actions.selectedIndex : -1)
        onDownloadActivityClicked: root.downloadActivityClicked()
        onWineToolsClicked: root.wineToolsRequested()
    }
}
