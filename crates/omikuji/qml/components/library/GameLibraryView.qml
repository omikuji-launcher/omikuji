pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../lib/CategoryFilter.js" as CF


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
    property bool cardPlayButton: false
    property bool doubleClickLaunches: false
    property string barStyle: "floating"
    property int barHeight: 56
    property bool barOpaque: false

    readonly property alias wineToolsAnchor: floatingBar.wineToolsAnchor
    readonly property alias actionBar: floatingBar

    signal gameRightClicked(int index, real winX, real winY)
    signal selectionChanged()
    signal settingsRequested(int index)
    signal downloadActivityClicked()
    signal wineToolsRequested()

    function handleActionKey(event) {
        if (!root.active || !root.actions || !root.actions.hasSelection || event.modifiers !== Qt.NoModifier) return false
        switch (event.key) {
        case Qt.Key_E:
            root.settingsRequested(root.actions.selectedIndex)
            return true
        case Qt.Key_Q:
            if (!floatingBar.wineToolsAnchor.visible) return false
            root.wineToolsRequested()
            return true
        case Qt.Key_F:
        case Qt.Key_Menu:
            return root._openCardMenu(root.actions.selectedIndex)
        }
        return false
    }

    readonly property bool reorderActive: gameGrid.reorderActive

    function liftCard(index) {
        return gameGrid.liftCard(index)
    }

    function _openCardMenu(index) {
        const card = gameGrid.cardAt(index)
        if (!card) return false
        const p = card.mapToItem(null, card.width / 2, card.height / 2)
        root.gameRightClicked(index, p.x, p.y)
        return true
    }

    // memoized top 10 by lastPlayed desc, recomputes when the model changes or kind flips to recent
    property var _recentIds: ({})

    function _recomputeRecent() {
        root._recentIds = CF.computeRecent(root.gameModel)
    }

    function drawPool(excluded) {
        let pool = []
        if (!root.gameModel) return pool
        for (let i = 0; i < root.gameModel.count; i++) {
            let game = root.gameModel.get_game(i)
            if (!game) continue
            if (excluded && excluded[game.gameId]) continue
            if (root.passes(i, game.name || "", game.hidden === true, game.favourite === true)) {
                pool.push(i)
            }
        }
        return pool
    }

    function hiddenSplit(hidden) {
        return CF.hiddenSplit(root.filterKind, hidden, root.showHidden)
    }

    function passes(index, name, hidden, favourite) {
        if (!CF.nameMatches(name, root.searchText)) return false
        if (root.hiddenSplit(hidden)) return false

        let verdict = CF.cardVerdict(root.filterKind, hidden, favourite)
        if (verdict !== null) return verdict
        if (!root.gameModel) return true

        let game = root.gameModel.get_game(index)
        if (!game) return false
        return CF.gameVerdict(root.filterKind, root.filterValue, game, root._recentIds)
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
    enabled: root.active
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
        cardPlayButton: root.cardPlayButton
        actions: root.actions
        dimHidden: root.dimHidden
        searchText: root.searchText
        filterKind: root.filterKind
        onGameClicked: (index) => {
            if (root.actions) root.actions.selectedIndex = index
            root.selectionChanged()
        }
        onGameDoubleClicked: (index) => {
            if (root.doubleClickLaunches && root.actions) root.actions.play(index)
        }
        onGameRightClicked: (index, winX, winY) => root.gameRightClicked(index, winX, winY)
        onPlayRequested: (index) => { if (root.actions) root.actions.toggle(index) }
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
        barStyle: root.barStyle
        barHeight: root.barHeight
        barOpaque: root.barOpaque
        surfaceRadius: root.radius
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
