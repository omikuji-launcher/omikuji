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

    implicitWidth: 200
    implicitHeight: items.length * itemHeight + Math.max(0, items.length - 1) * gap

    Squircle {
        readonly property int inset: Theme.space.xs
        x: inset
        width: parent.width - inset * 2
        height: root.itemHeight - inset * 2
        radius: Theme.radius.md
        fillColor: Theme.alpha(Theme.accent, 0.16)
        y: root.currentIndex * (root.itemHeight + root.gap) + inset
        visible: root.currentIndex >= 0 && root.currentIndex < root.items.length

        Behavior on y {
            NumberAnimation { duration: Theme.dur.med; easing.type: Theme.ease.emphasized; easing.overshoot: Theme.ease.overshoot }
        }
    }

    Column {
        width: parent.width
        spacing: root.gap

        Repeater {
            model: root.items

            Item {
                id: rowItem
                required property int index
                required property var modelData
                width: parent.width
                height: root.itemHeight

                readonly property bool selected: index === root.currentIndex

                Rectangle {
                    anchors.fill: parent
                    anchors.margins: Theme.space.xs
                    radius: Theme.radius.md
                    color: hov.containsMouse && !rowItem.selected ? Theme.stateHover : Theme.alpha(Theme.text, 0)
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
                        name: rowItem.modelData.icon || ""
                        size: 20
                        color: rowItem.selected ? Theme.accent : Theme.icon
                        visible: (rowItem.modelData.icon || "") !== ""
                        Behavior on color { ColorAnimation { duration: Theme.dur.fast } }
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width - (rowItem.modelData.icon ? 30 : 0)
                        text: rowItem.modelData.label || ""
                        color: rowItem.selected ? Theme.text : Theme.textMuted
                        font.pixelSize: Theme.type.label.size
                        font.weight: rowItem.selected ? Font.DemiBold : Font.Medium
                        elide: Text.ElideRight
                        Behavior on color { ColorAnimation { duration: Theme.dur.fast } }
                    }
                }

                MouseArea {
                    id: hov
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.itemClicked(rowItem.index)
                }
            }
        }
    }
}
