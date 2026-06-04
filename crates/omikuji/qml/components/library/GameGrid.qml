import QtQuick
import "../widgets"
import "../widgets/RunnerGrouping.js" as RG
import "."

Item {
    id: root

    property alias model: cardGrid.model
    property int selectedIndex: -1
    property real cardZoom: 1.0
    property int cardSpacing: 16
    property string cardFlow: "center"
    property bool cardElevation: false
    property int cardBaseWidth: 180
    property int cardBaseHeight: 240
    property string searchText: ""
    property string filterKind: "all"
    property string filterValue: ""
    property var gameModel: null

    signal gameClicked(int index)
    signal gameDoubleClicked(int index)
    signal gameRightClicked(int index, real winX, real winY)
    signal backgroundClicked()

    // memoized top 10 by lastPlayed desc, recomputes when the model changes or kind flips to recent
    property var _recentIds: ({})
    property int _recentStamp: 0

    function _recomputeRecent() {
        let dated = []
        for (let i = 0; i < gameModel.count; i++) {
            let g = gameModel.get_game(i)
            if (!g) continue
            let ts = Date.parse(g.lastPlayed || "") || 0
            if (ts > 0) dated.push({ id: g.gameId, ts: ts })
        }
        dated.sort((a, b) => b.ts - a.ts)
        let next = {}
        for (let i = 0; i < Math.min(10, dated.length); i++) next[dated[i].id] = true
        _recentIds = next
        _recentStamp = Date.now()
    }

    onFilterKindChanged: if (filterKind === "recent") _recomputeRecent()
    Connections {
        target: gameModel
        function onDataChanged() { if (root.filterKind === "recent") root._recomputeRecent() }
        function onRowsInserted() { if (root.filterKind === "recent") root._recomputeRecent() }
        function onRowsRemoved() { if (root.filterKind === "recent") root._recomputeRecent() }
    }

    function gamePassesFilter(index) {
        if (!gameModel) return true
        let game = gameModel.get_game(index)
        if (!game) return false

        switch (filterKind) {
            case "all":       return true
            case "favourite": return game.favourite === true
            case "recent":    return _recentIds[game.gameId] === true
            case "runner":    return RG.runnerBucket(game.runnerType) === filterValue
            case "tag": {
                let cats = []
                try { cats = JSON.parse(game.categories || "[]") } catch (e) { cats = [] }
                return cats.indexOf(filterValue) !== -1
            }
            default: return true
        }
    }

    CardGrid {
        id: cardGrid
        anchors.fill: parent
        cardZoom: root.cardZoom
        cardSpacing: root.cardSpacing
        cardFlow: root.cardFlow
        cardBaseWidth: root.cardBaseWidth
        cardBaseHeight: root.cardBaseHeight
        onBackgroundClicked: root.backgroundClicked()

        delegate: GameCard {
            required property int index
            required property string name
            required property string coverart
            required property string banner
            required property string color
            required property string runnerType
            required property string runner

            width: root.cardBaseWidth * root.cardZoom
            height: root.cardBaseHeight * root.cardZoom

            elevation: root.cardElevation
            selected: index === root.selectedIndex
            cardVisible: (root.searchText === "" ||
                         name.toLowerCase().includes(root.searchText.toLowerCase())) &&
                         root.gamePassesFilter(index)
            onClicked: root.gameClicked(index)
            onDoubleClicked: root.gameDoubleClicked(index)
            onRightClicked: (winX, winY) => root.gameRightClicked(index, winX, winY)
        }
    }
}
