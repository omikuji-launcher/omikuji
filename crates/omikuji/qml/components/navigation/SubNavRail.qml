pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../primitives"

Item {
    id: root

    property var items: []
    property int currentIndex: 0

    signal itemClicked(int index)

    readonly property int itemHeight: 46
    readonly property int gap: Theme.space.xs

    readonly property var topIndexes: root._indexesPinned(false)
    readonly property var bottomIndexes: root._indexesPinned(true)

    implicitWidth: 200
    implicitHeight: items.length * itemHeight + Math.max(0, items.length - 1) * gap

    function _indexesPinned(pinned) {
        let out = []
        for (let i = 0; i < items.length; i++)
            if (!!items[i].pinned === pinned) out.push(i)
        return out
    }

    function _rowY(index, top, bottom, railHeight) {
        let step = itemHeight + gap
        let t = top.indexOf(index)
        if (t >= 0) return t * step
        let b = bottom.indexOf(index)
        if (b < 0) return 0
        return railHeight - bottom.length * step + gap + b * step
    }

    component RailRow: Item {
        id: railRow
        property string icon: ""
        property string label: ""
        property bool selected: false
        signal activated()

        width: root.width
        height: root.itemHeight

        Rectangle {
            anchors.fill: parent
            anchors.margins: Theme.space.xs
            radius: Theme.radius.md
            color: hov.containsMouse && !railRow.selected ? Theme.stateHover : Theme.alpha(Theme.text, 0)
            Behavior on color { ColorAnimation { duration: Theme.dur.fast } }
        }

        Row {
            anchors.left: parent.left
            anchors.leftMargin: Theme.space.md
            anchors.right: parent.right
            anchors.rightMargin: Theme.space.sm
            anchors.verticalCenter: parent.verticalCenter
            spacing: Theme.space.sm + 2

            SvgIcon {
                anchors.verticalCenter: parent.verticalCenter
                name: railRow.icon
                size: 20
                color: railRow.selected ? Theme.accent : Theme.icon
                visible: railRow.icon !== ""
                Behavior on color { ColorAnimation { duration: Theme.dur.fast } }
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - (railRow.icon !== "" ? 30 : 0)
                text: railRow.label
                color: railRow.selected ? Theme.text : Theme.textMuted
                font.pixelSize: Theme.type.label.size
                font.weight: railRow.selected ? Font.DemiBold : Font.Medium
                elide: Text.ElideRight
                Behavior on color { ColorAnimation { duration: Theme.dur.fast } }
            }
        }

        MouseArea {
            id: hov
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: railRow.activated()
        }
    }

    Squircle {
        readonly property int inset: Theme.space.xs
        x: inset
        width: parent.width - inset * 2
        height: root.itemHeight - inset * 2
        radius: Theme.radius.md
        fillColor: Theme.alpha(Theme.accent, 0.16)
        y: root._rowY(root.currentIndex, root.topIndexes, root.bottomIndexes, root.height) + inset
        visible: root.currentIndex >= 0 && root.currentIndex < root.items.length

        Behavior on y {
            NumberAnimation { duration: Theme.dur.med; easing.type: Theme.ease.emphasized; easing.overshoot: Theme.ease.overshoot }
        }
    }

    Column {
        anchors.top: parent.top
        width: parent.width
        spacing: root.gap

        Repeater {
            model: root.topIndexes

            RailRow {
                required property var modelData
                icon: root.items[modelData].icon || ""
                label: root.items[modelData].label || ""
                selected: modelData === root.currentIndex
                onActivated: root.itemClicked(modelData)
            }
        }
    }

    Column {
        anchors.bottom: parent.bottom
        width: parent.width
        spacing: root.gap

        Repeater {
            model: root.bottomIndexes

            RailRow {
                required property var modelData
                icon: root.items[modelData].icon || ""
                label: root.items[modelData].label || ""
                selected: modelData === root.currentIndex
                onActivated: root.itemClicked(modelData)
            }
        }
    }
}
