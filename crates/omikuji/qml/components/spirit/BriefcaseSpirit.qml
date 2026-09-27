pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Item {
    id: root

    property color leatherColor: "#8a5a32"
    property color edgeColor: "#4f3119"
    property color claspColor: "#d4a843"
    property color spiritColor: Theme.accent
    property bool animated: true

    readonly property real designWidth: 130
    readonly property real designHeight: 170

    implicitWidth: root.designWidth
    implicitHeight: root.designHeight

    property real _blink: 0

    SequentialAnimation on _blink {
        running: root.animated && root.visible
        loops: Animation.Infinite
        PauseAnimation { duration: 2800 }
        NumberAnimation { to: 1; duration: 90 }
        NumberAnimation { to: 0; duration: 110 }
    }

    Item {
        width: root.designWidth
        height: root.designHeight
        anchors.centerIn: parent
        scale: Math.min(root.width / width, root.height / height)

        PosedSpirit {
            anchors.fill: parent
            accentColor: root.spiritColor
            animated: root.animated
            squeeze: root._blink
        }

        Item {
            id: briefcase

            readonly property real handWidth: 24
            readonly property real handOverhang: 9
            readonly property real line: 2.5

            x: 23
            y: 94
            width: 84
            height: 54

            Rectangle {
                x: (briefcase.width - width) / 2
                y: -11
                width: 28
                height: 20
                radius: 6
                color: "transparent"
                border.color: root.edgeColor
                border.width: briefcase.line
            }

            Rectangle {
                anchors.fill: parent
                radius: 7
                color: root.leatherColor
                border.color: root.edgeColor
                border.width: briefcase.line
            }

            Rectangle {
                x: briefcase.line
                y: briefcase.height * 0.36
                width: briefcase.width - briefcase.line * 2
                height: briefcase.line
                color: root.edgeColor
            }

            Rectangle {
                x: (briefcase.width - width) / 2
                y: briefcase.height * 0.36 - 4
                width: 12
                height: 10
                radius: 2
                color: root.claspColor
                border.color: root.edgeColor
                border.width: briefcase.line
            }

            Repeater {
                model: [-1, 1]

                Rectangle {
                    required property real modelData

                    x: modelData < 0
                        ? -briefcase.handOverhang
                        : briefcase.width - briefcase.handWidth + briefcase.handOverhang
                    y: briefcase.height * 0.3
                    width: briefcase.handWidth
                    height: 15
                    radius: height / 2
                    rotation: modelData * 9
                    color: root.spiritColor
                }
            }
        }
    }
}
