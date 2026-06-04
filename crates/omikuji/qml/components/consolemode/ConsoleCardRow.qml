import QtQuick
import QtQuick.Layouts
import ".."
import "../widgets/RunnerGrouping.js" as RG

Item {
    id: row

    property var gameModelRef
    property var uiSettingsRef
    property real uiScale: 1.0
    property bool isFocusedRunning: false
    property string searchText: ""

    readonly property url focusedBanner: listView.currentItem ? listView.currentItem.bannerSource : ""

    onSearchTextChanged: _rebuildFilter()

    signal launchRequested(string gameId)
    signal stopRequested(string gameId)
    signal escapePressed()

    function refreshRunningState() {
        if (!listView.currentItem || !gameModelRef) {
            isFocusedRunning = false
            return
        }
        let gameId = listView.currentItem.gameId
        if (!gameId || gameId === "") {
            isFocusedRunning = false
            return
        }
        let idx = gameModelRef.index_of_id(gameId)
        isFocusedRunning = idx >= 0 && gameModelRef.is_running(idx)
    }

    function markFocusedRunning() { isFocusedRunning = true }

    function markGameStopped(stoppedId) {
        if (listView.currentItem && listView.currentItem.gameId === stoppedId) {
            isFocusedRunning = false
        }
    }

    readonly property int sideMargin: 64 * uiScale
    readonly property int focusedSlot: 660 * uiScale
    readonly property int unfocusedSlot: 220 * uiScale
    readonly property int cardSpacing: 14 * uiScale
    readonly property int cardHeight: 370 * uiScale
    readonly property int headerHeight: 50 * uiScale
    readonly property int navCooldown: 90

    implicitHeight: cardHeight + 220 * uiScale

    property var _categories: [{ kind: "all", value: "", name: "Library" }]
    property int _categoryIndex: 0
    property var _filteredGames: []
    property var _recentIds: ({})

    readonly property var _currentCategory: _categories[_categoryIndex] || { kind: "all", value: "", name: "Library" }

    function _loadCategories() {
        let arr = []
        if (uiSettingsRef) {
            try { arr = JSON.parse(uiSettingsRef.categoriesJson()) } catch (e) { arr = [] }
        }
        let enabled = arr.filter(c => c.enabled !== false)
        if (enabled.length === 0) {
            enabled = [{ kind: "all", value: "", name: "Library" }]
        }
        _categories = enabled
        if (_categoryIndex >= _categories.length) _categoryIndex = 0
        _rebuildFilter()
    }

    function _recomputeRecent() {
        _recentIds = {}
        if (!gameModelRef) return
        let dated = []
        for (let i = 0; i < gameModelRef.count; i++) {
            let g = gameModelRef.get_game(i)
            if (!g) continue
            let ts = Date.parse(g.lastPlayed || "") || 0
            if (ts > 0) dated.push({ id: g.gameId, ts: ts })
        }
        dated.sort((a, b) => b.ts - a.ts)
        let next = {}
        for (let i = 0; i < Math.min(10, dated.length); i++) next[dated[i].id] = true
        _recentIds = next
    }

    function _matches(g) {
        if (!g) return false

        let q = (searchText || "").trim().toLowerCase()
        if (q.length > 0) {
            let name = (g.name || "").toLowerCase()
            if (name.indexOf(q) === -1) return false
        }

        let cat = _currentCategory
        let kind = cat.kind || "all"
        let value = cat.value || ""
        switch (kind) {
            case "all":       return true
            case "favourite": return g.favourite === true
            case "recent":    return _recentIds[g.gameId] === true
            case "runner":    return RG.runnerBucket(g.runnerType) === value
            case "tag": {
                let cats = []
                try { cats = JSON.parse(g.categories || "[]") } catch (e) { cats = [] }
                return cats.indexOf(value) !== -1
            }
            default: return true
        }
    }

    function _rebuildFilter() {
        if (!gameModelRef) { _filteredGames = []; return }
        if (_currentCategory.kind === "recent") _recomputeRecent()
        let out = []
        for (let i = 0; i < gameModelRef.count; i++) {
            let g = gameModelRef.get_game(i)
            if (g && _matches(g)) out.push(g)
        }
        _filteredGames = out
        listView.currentIndex = out.length > 0 ? 0 : -1
    }

    function _selectCategory(i) {
        if (i < 0 || i >= _categories.length || i === _categoryIndex) return
        _categoryIndex = i
        _rebuildFilter()
    }
    function nextCategory() { _selectCategory((_categoryIndex + 1) % _categories.length) }
    function prevCategory() { _selectCategory((_categoryIndex - 1 + _categories.length) % _categories.length) }

    function toggleFocused() {
        if (!listView.currentItem) return
        let gameId = listView.currentItem.gameId
        if (!gameId || gameId === "") return
        if (row.isFocusedRunning) {
            row.stopRequested(gameId)
        } else {
            row.launchRequested(gameId)
        }
    }

    function focusList() { listView.forceActiveFocus() }

    property bool _navLocked: false
    Timer {
        id: navUnlock
        interval: row.navCooldown
        onTriggered: row._navLocked = false
    }

    function navLeft() {
        if (_navLocked || listView.currentIndex <= 0) return
        _navLocked = true
        navUnlock.restart()
        listView.decrementCurrentIndex()
    }
    function navRight() {
        if (_navLocked || listView.currentIndex >= listView.count - 1) return
        _navLocked = true
        navUnlock.restart()
        listView.incrementCurrentIndex()
    }

    Component.onCompleted: _loadCategories()

    Connections {
        target: row.gameModelRef
        function onDataChanged() { row._rebuildFilter() }
        function onRowsInserted() { row._rebuildFilter() }
        function onRowsRemoved() { row._rebuildFilter() }
        function onModelReset() { row._rebuildFilter() }
    }

    Connections {
        target: row.uiSettingsRef
        ignoreUnknownSignals: true
        function onChanged() { row._loadCategories() }
    }

    Item {
        id: header
        x: row.sideMargin
        y: 0
        height: row.headerHeight
        width: parent.width - row.sideMargin * 2

        Row {
            id: catRow
            anchors.bottom: parent.bottom
            spacing: 26 * row.uiScale

            Repeater {
                model: row._categories
                delegate: Item {
                    id: catItem
                    required property int index
                    required property var modelData

                    readonly property bool isSelected: row._categoryIndex === index

                    implicitWidth: label.implicitWidth
                    implicitHeight: row.headerHeight

                    Text {
                        id: label
                        anchors.bottom: parent.bottom
                        anchors.bottomMargin: 2
                        text: catItem.modelData.name || "Library"
                        color: catItem.isSelected ? theme.text : theme.textMuted
                        font.pixelSize: catItem.isSelected ? 38 * row.uiScale : 22 * row.uiScale
                        font.weight: catItem.isSelected ? Font.Bold : Font.Medium

                        Behavior on color { ColorAnimation { duration: 180 } }
                        Behavior on font.pixelSize { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: row._selectCategory(catItem.index)
                    }
                }
            }
        }
    }

    ListView {
        id: listView
        anchors.top: header.bottom
        anchors.topMargin: 28 * row.uiScale
        anchors.left: parent.left
        anchors.right: parent.right
        height: row.cardHeight

        orientation: ListView.Horizontal
        model: row._filteredGames
        spacing: row.cardSpacing
        cacheBuffer: 2000
        clip: false

        focus: true

        preferredHighlightBegin: row.sideMargin
        preferredHighlightEnd: row.sideMargin + row.focusedSlot
        highlightRangeMode: ListView.StrictlyEnforceRange
        highlightMoveDuration: 180
        highlightMoveVelocity: -1

        Keys.onLeftPressed: row.navLeft()
        Keys.onRightPressed: row.navRight()
        Keys.onUpPressed: row.prevCategory()
        Keys.onDownPressed: row.nextCategory()
        Keys.onReturnPressed: row.toggleFocused()
        Keys.onEnterPressed: row.toggleFocused()
        Keys.onEscapePressed: row.escapePressed()

        onCurrentIndexChanged: row.refreshRunningState()

        delegate: ConsoleCard {
            required property int index
            required property var modelData

            readonly property string gameId: modelData.gameId || ""
            readonly property string playtimeHours: modelData.playtime !== undefined ? modelData.playtime.toFixed(1) : ""
            readonly property string lastPlayedDate: modelData.lastPlayed !== undefined ? modelData.lastPlayed : ""

            title: modelData.name || ""
            bannerSource: modelData.banner || ""
            coverartSource: modelData.coverart || ""
            tint: modelData.color && modelData.color.length > 0 ? modelData.color : theme.accent
            focused: ListView.isCurrentItem
            uiScale: row.uiScale

            onFocusRequested: {
                listView.currentIndex = index
                listView.forceActiveFocus()
            }
            onLaunchRequested: {
                listView.currentIndex = index
                listView.forceActiveFocus()
                row.toggleFocused()
            }
        }
    }

    Column {
        id: meta
        anchors.top: listView.bottom
        anchors.topMargin: 36 * row.uiScale
        anchors.left: parent.left
        anchors.leftMargin: row.sideMargin
        spacing: 14 * row.uiScale

        opacity: listView.currentItem ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }

        Text {
            text: listView.currentItem ? listView.currentItem.title : ""
            color: theme.accent
            font.pixelSize: 34 * row.uiScale
            font.weight: Font.Bold
        }

        Row {
            spacing: 18 * row.uiScale

            Text {
                text: {
                    const h = listView.currentItem ? listView.currentItem.playtimeHours : ""
                    return h !== "" ? h + " hrs" : ""
                }
                color: theme.text
                font.pixelSize: 16 * row.uiScale
                font.weight: Font.Medium
                visible: text.length > 0
            }

            Text {
                text: {
                    const d = listView.currentItem ? listView.currentItem.lastPlayedDate : ""
                    return d !== "" ? "Last played " + d : "Never played"
                }
                color: theme.textMuted
                font.pixelSize: 16 * row.uiScale
                font.weight: Font.Medium
            }
        }

        ConsolePlayButton {
            readonly property string _gameId: listView.currentItem ? listView.currentItem.gameId : ""
            isRunning: row.isFocusedRunning
            uiScale: row.uiScale
            onPlayClicked: row.launchRequested(_gameId)
            onStopClicked: row.stopRequested(_gameId)
        }
    }
}
