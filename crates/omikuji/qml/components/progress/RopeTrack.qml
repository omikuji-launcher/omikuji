pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Item {
    id: root

    property real value: 0
    property bool paused: false
    property bool animate: true
    property color spiritColor: Theme.accent
    property color lineColor: Theme.mix(root.spiritColor, Theme.surface, 0.55)
    property color fillColor: Theme.mix(root.lineColor, Theme.surface, 0.7)
    property color trackColor: Theme.alpha(Theme.text, 0.18)
    property real trackWidth: 4
    property real tug: 0
    property real pant: 0
    property real wipe: 0

    readonly property real ropeHeight: Math.round(height * 0.46)
    readonly property real ropeY: height * 0.6 - ropeHeight / 2
    readonly property real startLength: ropeHeight * 1.2
    readonly property real spiritOffset: height * 0.35
    readonly property real spiritReach: height * 0.78
    readonly property real tipX: startLength
                                 + value * Math.max(0, width - startLength - spiritReach)
                                 + tug * height * 0.12

    implicitWidth: 160
    implicitHeight: 24

    SequentialAnimation on tug {
        running: root.animate && !root.paused && root.visible
        loops: Animation.Infinite

        NumberAnimation { to: 1; duration: 220; easing.type: Easing.OutQuad }
        PauseAnimation { duration: 140 }
        NumberAnimation { to: 0; duration: 420; easing.type: Easing.InOutQuad }
        PauseAnimation { duration: 260 }

        onRunningChanged: if (!running) root.tug = 0
    }

    SequentialAnimation on pant {
        running: root.animate && root.paused && root.visible
        loops: Animation.Infinite

        NumberAnimation { to: 1; duration: 180; easing.type: Easing.OutQuad }
        NumberAnimation { to: 0; duration: 240; easing.type: Easing.InQuad }

        onRunningChanged: if (!running) root.pant = 0
    }

    SequentialAnimation on wipe {
        running: root.animate && root.paused && root.visible
        loops: Animation.Infinite

        PauseAnimation { duration: 2400 }
        NumberAnimation { from: 0; to: 1; duration: 520; easing.type: Easing.InOutQuad }

        onRunningChanged: if (!running) root.wipe = 0
    }

    Rectangle {
        x: root.tipX
        y: root.ropeY + (root.ropeHeight - height) / 2
        width: Math.max(0, root.width - root.tipX)
        height: root.trackWidth
        radius: height / 2
        color: root.trackColor
    }

    Item {
        anchors.fill: parent
        opacity: root.paused ? 0.6 : 1

        ShaderEffect {
            y: root.ropeY
            width: root.tipX
            height: root.ropeHeight
            blending: true

            property real lineWidth: Math.max(1.5, root.height * 0.07)
            property size resolution: Qt.size(width, height)
            property color lineColor: root.lineColor
            property color fillColor: root.fillColor

            fragmentShader: "shaders/rope.frag.qsb"
        }

        PosedSpirit {
            id: spirit

            height: root.height / 0.7
            width: height * 136 / 112
            x: root.tipX + root.spiritOffset - width / 2
            y: (root.height - height) / 2
            accentColor: root.spiritColor
            strain: root.paused ? 0 : 1
            squeeze: root.paused ? 0.3 : 0
            wobble: root.tug
            animated: root.animate && !root.paused

            readonly property real feet: spirit.height / 2 + root.height / 2

            transform: [
                Rotation {
                    origin.x: spirit.width / 2
                    origin.y: spirit.feet
                    angle: root.tug * 7
                },
                Scale {
                    origin.x: spirit.width / 2
                    origin.y: spirit.feet
                    xScale: 1 + 0.03 * root.pant
                    yScale: 1 - 0.045 * root.pant
                }
            ]
        }

        Repeater {
            model: [-1, 1]

            Hand {
                required property real modelData

                size: root.ropeHeight
                color: root.spiritColor
                x: root.tipX - width * 0.9
                y: root.ropeY + (root.ropeHeight - height) / 2 + modelData * root.ropeHeight * 0.22
                rotation: modelData * 8
                opacity: root.paused ? 0 : 1

                Behavior on opacity { NumberAnimation { duration: 150 } }
            }
        }

        Hand {
            readonly property real centerX: root.tipX + root.spiritOffset

            size: root.ropeHeight
            color: root.spiritColor
            x: centerX - root.height * 0.32 + root.wipe * root.height * 0.54 - width / 2
            y: root.height * 0.17 - height / 2
            rotation: -12
            opacity: Math.min(1, Math.sin(root.wipe * Math.PI) * 3)
        }
    }

    component Hand: Rectangle {
        property real size

        width: size * 0.95
        height: size * 0.62
        radius: height / 2
    }
}
