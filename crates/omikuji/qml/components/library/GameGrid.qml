import QtQuick
import Qt5Compat.GraphicalEffects
import omikuji 1.0
import "."
import "../cards"
import "../primitives"

Item {
    id: root

    property alias model: cardGrid.model
    property int selectedIndex: -1
    property real cardZoom: 1.0
    property int cardSpacing: 16
    property string cardFlow: "center"
    property string cardStyle: "normal"
    property bool cardElevation: false
    property int cardBaseWidth: 180
    property int cardBaseHeight: 240
    property string searchText: ""
    property string filterKind: "all"
    property string filterValue: ""
    property string cardSort: "default"
    property bool showHidden: false
    property bool dimHidden: false
    property var gameModel: null
    property var view: null

    readonly property bool reorderActive: cardSort === "custom" && searchText === "" && filterKind === "all"
    readonly property var reorderKeys: ["omikuji/card"]

    signal gameClicked(int index)
    signal gameDoubleClicked(int index)
    signal gameRightClicked(int index, real winX, real winY)
    signal backgroundClicked()

    // memoized top 10 by lastPlayed desc, recomputes when the model changes or kind flips to recent
    EmptyState {
        anchors.fill: parent
        visible: root.gameModel && root.gameModel.count === 0
        art: "qrc:/qt/qml/omikuji/qml/icons/dino.png"
        artSize: Math.round(Math.min(root.width * 0.5, 480))
        textSize: Theme.type.display.size
        hintSize: Theme.type.subtitle.size
        text: qsTr("nothing here yet")
        hint: qsTr("pick a store on the left to install games, or add one yourself with the + button up top")
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
            id: cardDelegate

            required property int index
            required property string name
            required property string coverart
            required property string banner
            required property string color
            required property string runnerType
            required property string runner
            required property bool hidden
            required property bool favourite

            width: root.cardBaseWidth * root.cardZoom
            height: styledHeight

            elevation: root.cardElevation && dragProxy.dragCard !== cardDelegate
            cardStyle: root.cardStyle
            selected: index === root.selectedIndex
            dimmed: root.dimHidden && hidden
            cardVisible: (root.searchText === "" ||
                         name.toLowerCase().includes(root.searchText.toLowerCase())) &&
                         (root.showHidden || !hidden) &&
                         (root.filterKind !== "favourite" || favourite) &&
                         (!root.view || root.view.gamePassesFilter(index))
            reorderable: root.reorderActive
            onClicked: root.gameClicked(index)
            onDoubleClicked: root.gameDoubleClicked(index)
            onRightClicked: (winX, winY) => root.gameRightClicked(index, winX, winY)
            onReorderStarted: (grabX, grabY) => root._startReorder(cardDelegate, grabX, grabY)
            onReorderMoved: (winX, winY) => root._trackReorder(winX, winY)
            onReorderEnded: root._endReorder()

            DropArea {
                anchors.fill: parent
                enabled: root.reorderActive && !cardDelegate.reordering
                keys: root.reorderKeys
                onEntered: (drag) => {
                    if (moveCooldown.running) return
                    let from = drag.source.dragIndex
                    if (from >= 0 && from !== cardDelegate.index) {
                        root.gameModel.moveGame(from, cardDelegate.index)
                        moveCooldown.restart()
                    }
                }
            }
        }
    }

    Timer {
        id: moveCooldown
        interval: 220
    }

    function _startReorder(card, grabX, grabY) {
        snapAnim.stop()
        dragProxy.grabX = grabX
        dragProxy.grabY = grabY
        let p = card.mapToItem(dragOverlay, grabX, grabY)
        dragProxy.x = p.x - grabX
        dragProxy.y = p.y - grabY
        dragProxy.startIndex = card.index
        dragProxy.dragCard = card
        dragProxy.scale = 1.05
    }

    function _trackReorder(winX, winY) {
        let p = dragOverlay.mapFromItem(null, winX, winY)
        dragProxy.x = p.x - dragProxy.grabX
        dragProxy.y = p.y - dragProxy.grabY
    }

    function _endReorder() {
        if (!dragProxy.dragCard) return
        let cx = dragProxy.x + dragProxy.grabX
        let cy = dragProxy.y + dragProxy.grabY
        let valid = cx >= 0 && cx <= dragOverlay.width && cy >= 0 && cy <= dragOverlay.height
        if (!valid && dragProxy.dragIndex !== dragProxy.startIndex)
            gameModel.moveGame(dragProxy.dragIndex, dragProxy.startIndex)
        dragProxy.commitValid = valid
        let slot = dragProxy.dragCard.mapToItem(dragOverlay, 0, 0)
        snapX.to = slot.x
        snapY.to = slot.y
        snapAnim.start()
    }

    SequentialAnimation {
        id: snapAnim
        ParallelAnimation {
            NumberAnimation { id: snapX; target: dragProxy; property: "x"; duration: 160; easing.type: Easing.OutCubic }
            NumberAnimation { id: snapY; target: dragProxy; property: "y"; duration: 160; easing.type: Easing.OutCubic }
            NumberAnimation { target: dragProxy; property: "scale"; to: 1.0; duration: 160; easing.type: Easing.OutCubic }
        }
        ScriptAction {
            script: {
                let commit = dragProxy.commitValid
                dragProxy.dragCard = null
                if (commit) root.gameModel.commitOrder()
            }
        }
    }

    Item {
        id: dragOverlay
        anchors.fill: parent
        z: 10
        visible: dragProxy.dragCard !== null

        Item {
            id: dragProxy
            property Item dragCard: null
            readonly property int dragIndex: dragCard ? dragCard.index : -1
            property int startIndex: -1
            property bool commitValid: false
            property real grabX: 0
            property real grabY: 0
            width: dragCard ? dragCard.width : 0
            height: dragCard ? dragCard.height : 0
            Behavior on scale { NumberAnimation { duration: 120; easing.type: Easing.OutCubic } }
            Drag.active: dragCard !== null && !snapAnim.running
            Drag.keys: root.reorderKeys
            Drag.hotSpot.x: grabX
            Drag.hotSpot.y: grabY

            DropShadow {
                anchors.fill: cardClone
                source: cardClone
                radius: 24
                samples: 25
                verticalOffset: 10
                color: Qt.rgba(0, 0, 0, 0.4)
            }

            ShaderEffectSource {
                id: cardClone
                anchors.fill: parent
                sourceItem: dragProxy.dragCard
                hideSource: true
            }
        }
    }
}
