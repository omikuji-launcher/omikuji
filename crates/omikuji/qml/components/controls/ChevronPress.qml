import QtQuick
import omikuji 1.0

PressArea {
    id: root

    readonly property real trailingWidth: icon.width + Theme.space.lg * 2

    hoverEnabled: true
    ringRadius: Theme.radius.md

    Rectangle {
        anchors.fill: parent
        anchors.margins: Theme.space.xs
        radius: Theme.radius.sm
        color: root.containsMouse ? Theme.stateHover : "transparent"

        Behavior on color { ColorAnimation { duration: Theme.dur.fast } }
    }

    IconTile {
        id: icon
        anchors.right: parent.right
        anchors.rightMargin: Theme.space.lg
        anchors.verticalCenter: parent.verticalCenter
        width: 28
        height: 28
        iconSize: 20
        icon: "chevron_left"
        rotation: 180
        opacity: root.containsMouse ? 1 : 0.8

        Behavior on opacity { NumberAnimation { duration: Theme.dur.fast; easing.type: Theme.ease.standard } }
    }
}
