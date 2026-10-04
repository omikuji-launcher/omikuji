pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Rectangle {
    id: root

    property alias icon: tile.icon
    property alias source: tile.source
    property alias monogram: tile.monogram
    property alias cache: tile.cache
    property alias iconSize: tile.iconSize
    property real tileSize: 36
    property string title: ""
    property string subtitle: ""
    property bool danger: false
    readonly property color tint: danger ? Theme.error : Theme.secondary
    property alias titleAccessories: accessoryRow.data
    default property alias trailing: trailingRow.data

    readonly property bool active: area.containsMouse || InputMode.keyFocus(area)

    signal activated()

    implicitHeight: 56
    radius: Theme.radius.md
    color: area.containsMouse ? Theme.alpha(Theme.text, 0.08) : "transparent"

    IconTile {
        id: tile
        width: root.tileSize
        height: root.tileSize
        tint: root.tint
        anchors.left: parent.left
        anchors.leftMargin: Theme.space.sm
        anchors.verticalCenter: parent.verticalCenter
    }

    Column {
        id: textCol
        anchors.left: tile.right
        anchors.leftMargin: Theme.space.md
        anchors.right: trailingRow.left
        anchors.rightMargin: Theme.space.md
        anchors.verticalCenter: parent.verticalCenter
        spacing: 2

        Row {
            spacing: Theme.space.xs

            Text {
                width: Math.min(implicitWidth, textCol.width - accessoryRow.width - Theme.space.xs)
                text: root.title
                color: root.danger ? Theme.error : Theme.text
                font.pixelSize: Theme.type.body.size
                font.weight: Font.DemiBold
                elide: Text.ElideRight
            }
            Row {
                id: accessoryRow
                spacing: Theme.space.xs
                anchors.verticalCenter: parent.verticalCenter
            }
        }
        Text {
            width: parent.width
            text: root.subtitle
            visible: text !== ""
            color: Theme.textMuted
            font.pixelSize: Theme.type.caption.size
            elide: Text.ElideRight
        }
    }

    PressArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        ringRadius: root.radius
        onActivated: root.activated()
    }

    Row {
        id: trailingRow
        anchors.right: parent.right
        anchors.rightMargin: Theme.space.sm
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.space.sm
    }
}
