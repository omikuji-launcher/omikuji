pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Item {
    id: root

    property color lineColor: Theme.text
    property color fillColor: Theme.surface
    property color spiritColor: Theme.accent
    property real shake: 0
    property real tilt: 0
    property real paperOut: 0

    readonly property real designWidth: 200
    readonly property real designHeight: 170
    readonly property real spiritFeet: 146

    signal revealed()

    function start() {
        ritual.restart()
    }

    implicitWidth: root.designWidth
    implicitHeight: root.designHeight

    SequentialAnimation {
        id: ritual

        PropertyAction { target: root; property: "paperOut"; value: 0 }
        PropertyAction { target: root; property: "tilt"; value: 0 }
        PropertyAction { target: root; property: "shake"; value: 0 }

        ParallelAnimation {
            NumberAnimation {
                target: root; property: "tilt"; to: -16
                duration: 180; easing.type: Easing.OutQuad
            }
            NumberAnimation { target: root; property: "shake"; to: 1; duration: 140 }
        }

        NumberAnimation { target: root; property: "tilt"; to: 18; duration: 90 }
        NumberAnimation { target: root; property: "tilt"; to: -13; duration: 90 }
        NumberAnimation { target: root; property: "tilt"; to: 11; duration: 85 }
        NumberAnimation { target: root; property: "tilt"; to: -8; duration: 85 }
        NumberAnimation { target: root; property: "tilt"; to: 5; duration: 80 }

        ParallelAnimation {
            NumberAnimation {
                target: root; property: "tilt"; to: 0
                duration: 170; easing.type: Easing.OutBack
            }
            NumberAnimation { target: root; property: "shake"; to: 0; duration: 200 }
        }

        PauseAnimation { duration: 90 }

        ParallelAnimation {
            NumberAnimation {
                target: root; property: "tilt"; to: 24
                duration: 240; easing.type: Easing.InOutQuad
            }
            SequentialAnimation {
                PauseAnimation { duration: 90 }
                NumberAnimation {
                    target: root; property: "paperOut"; to: 1
                    duration: 340; easing.type: Easing.OutBack
                }
            }
        }

        PauseAnimation { duration: 140 }
        ScriptAction { script: root.revealed() }

        NumberAnimation {
            target: root; property: "tilt"; to: 0
            duration: 220; easing.type: Easing.OutCubic
        }
    }

    Item {
        width: root.designWidth
        height: root.designHeight
        anchors.centerIn: parent
        scale: Math.min(root.width / width, root.height / height)

        PosedSpirit {
            id: spirit

            anchors.fill: parent
            accentColor: root.spiritColor
            squeeze: root.shake
            wobble: root.shake
            lookUp: root.paperOut

            transform: [
                Rotation {
                    origin.x: spirit.width / 2
                    origin.y: root.spiritFeet
                    angle: root.tilt * 0.3
                },
                Scale {
                    origin.x: spirit.width / 2
                    origin.y: root.spiritFeet
                    xScale: 1 + 0.03 * root.shake
                    yScale: 1 - 0.04 * root.shake
                }
            ]
        }

        Item {
            id: box

            readonly property real handWidth: 26
            readonly property real handOverhang: 9

            x: 72
            y: 96
            width: 56
            height: 66
            rotation: root.tilt

            Rectangle {
                x: (box.width - width) / 2
                y: 10 - root.paperOut * 52
                width: 18
                height: 46
                radius: 3
                color: root.fillColor
                border.color: root.lineColor
                border.width: 2.5
            }

            MikujiBox {
                anchors.fill: parent
                lineColor: root.lineColor
                fillColor: root.fillColor
            }

            Repeater {
                model: [-1, 1]

                Rectangle {
                    required property real modelData

                    x: modelData < 0
                        ? -box.handOverhang
                        : box.width - box.handWidth + box.handOverhang
                    y: box.height * 0.22
                    width: box.handWidth
                    height: 15
                    radius: height / 2
                    rotation: modelData * 9
                    color: root.spiritColor
                }
            }
        }
    }
}
