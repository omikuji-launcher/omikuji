import QtQuick

// Flow+Repeater not GridView becuase GridView only repositions on model changes, we need the slide when a card flips visible false for filtering
Item {
    id: root

    property alias model: repeater.model
    property alias count: repeater.count
    property Component delegate: null

    property Component headerComponent: null
    property int headerHeight: 36
    property int headerTopMargin: 12
    property int headerSideMargin: 20
    property int headerSpacing: 4

    property real cardZoom: 1.0
    property int cardSpacing: 16
    property int cardBaseWidth: 180
    property int cardBaseHeight: 240
    property string cardFlow: "center"

    // library uses this to clear selection, stores ignore it
    signal backgroundClicked()

    Loader {
        id: header
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.topMargin: root.headerTopMargin
        anchors.leftMargin: root.headerSideMargin
        anchors.rightMargin: root.headerSideMargin
        height: active ? root.headerHeight : 0
        sourceComponent: root.headerComponent
        active: sourceComponent !== null
    }

    Flickable {
        id: gridFlick
        anchors.top: header.active ? header.bottom : parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.topMargin: header.active ? root.headerSpacing : 0
        contentHeight: grid.height + 100
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        flickDeceleration: 3000
        maximumFlickVelocity: 1500

        // covers the whole scrollable area so clicks on empty flow space falls through
        MouseArea {
            width: parent.width
            height: Math.max(grid.height + 100, gridFlick.height)
            z: 0
            onClicked: {
                forceActiveFocus()
                root.backgroundClicked()
            }
        }

        Flow {
            id: grid
            y: 20
            spacing: root.cardSpacing
            flow: Flow.LeftToRight
            z: 1

            // width shrinks to exactly N full cards so the last partial row sits left-aligned
            property int cardW: Math.round(root.cardBaseWidth * root.cardZoom)
            readonly property int sidePad: 12
            property int avail: Math.max(0, parent.width - sidePad * 2)
            property int maxCols: Math.max(1, Math.floor((avail + root.cardSpacing) / (cardW + root.cardSpacing)))
            property int colsToUse: Math.max(1, Math.min(maxCols, repeater.count))
            width: colsToUse * cardW + (colsToUse - 1) * root.cardSpacing
            x: {
                if (root.cardFlow === "left") return sidePad
                if (root.cardFlow === "right") return Math.max(sidePad, parent.width - width - sidePad)
                return Math.max(sidePad, (parent.width - width) / 2)
            }

            move: Transition {
                NumberAnimation { properties: "x,y"; duration: 200; easing.type: Easing.OutCubic }
            }

            Repeater {
                id: repeater
                delegate: root.delegate
            }
        }
    }

    // y measured from gridFlick.y so a headered panel doesnt start the scrollbar inside the header
    Rectangle {
        anchors.right: parent.right
        anchors.rightMargin: 4
        y: gridFlick.y + 8 + gridFlick.contentY / Math.max(1, gridFlick.contentHeight) * (gridFlick.height - 16)
        width: 3
        height: Math.max(30, gridFlick.height / Math.max(1, gridFlick.contentHeight) * (gridFlick.height - 16))
        radius: 2
        color: theme.textFaint
        visible: gridFlick.contentHeight > gridFlick.height
        opacity: gridFlick.moving ? 0.8 : 0.3

        Behavior on opacity {
            NumberAnimation { duration: 200 }
        }
    }
}
