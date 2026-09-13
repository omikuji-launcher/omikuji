pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Item {
    id: root

    property real value: 0
    property color fillColor
    property color trackColor

    readonly property real thickness: Math.round(height * 0.4)
    readonly property real gap: Math.max(2, Math.round(thickness * 0.35))
    readonly property int count: Math.max(1, Math.floor((width + gap) / (thickness * 2.2 + gap)))
    readonly property real filled: value * count

    implicitWidth: 160
    implicitHeight: 24

    Row {
        anchors.verticalCenter: parent.verticalCenter
        spacing: root.gap

        Repeater {
            model: root.count

            Rectangle {
                required property int index

                width: (root.width - root.gap * (root.count - 1)) / root.count
                height: root.thickness
                radius: 2
                color: Theme.mix(root.trackColor, root.fillColor, Math.max(0, Math.min(1, root.filled - index)))
            }
        }
    }
}
