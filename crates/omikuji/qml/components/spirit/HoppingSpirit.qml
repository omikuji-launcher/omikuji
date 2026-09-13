pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

// is the header the only thing besides the output log? then it can go on its far right.
// is the dialog or place filled with fields, buttons, etc? then the output log needs tighter spacing, hence no spirit.

Item {
    id: root

    property color color: Theme.textMuted
    property real fontSize: Theme.type.caption.size
    property bool running: true

    readonly property real slotSpacing: root.fontSize * 0.55
    readonly property real spiritBox: root.fontSize * 1.2
    readonly property real hopHeight: root.fontSize * 0.34
    readonly property real pinchHeight: root.fontSize * 0.62
    readonly property real sink: root.fontSize * 0.12

    function slotX(slot) {
        return slot * root.slotSpacing
    }

    implicitWidth: root.spiritBox + root.slotSpacing * 2
    implicitHeight: root.spiritBox + root.pinchHeight + root.sink
    baselineOffset: root.implicitHeight

    PosedSpirit {
        id: ghost

        property real lift: 0
        property real dangle: 0

        width: root.spiritBox
        height: root.spiritBox
        x: root.slotX(0)
        y: parent.height - height - ghost.lift + root.sink

        accentColor: root.color
        coreColor: root.color
        blush: 0
        animated: false

        transform: Scale {
            origin.x: ghost.width / 2
            xScale: 1 - 0.16 * ghost.dangle
            yScale: 1 + 0.22 * ghost.dangle
        }
    }

    SequentialAnimation {
        running: root.running && root.visible
        loops: Animation.Infinite

        PropertyAction { target: ghost; property: "x"; value: root.slotX(0) }
        PropertyAction { target: ghost; property: "lift"; value: 0 }
        PropertyAction { target: ghost; property: "dangle"; value: 0 }
        PauseAnimation { duration: 240 }

        ParallelAnimation {
            NumberAnimation {
                target: ghost; property: "x"; to: root.slotX(1)
                duration: 250; easing.type: Easing.InOutQuad
            }
            SequentialAnimation {
                NumberAnimation {
                    target: ghost; property: "lift"; to: root.hopHeight
                    duration: 125; easing.type: Easing.OutQuad
                }
                NumberAnimation {
                    target: ghost; property: "lift"; to: 0
                    duration: 125; easing.type: Easing.InQuad
                }
            }
        }

        PauseAnimation { duration: 130 }

        ParallelAnimation {
            NumberAnimation {
                target: ghost; property: "x"; to: root.slotX(2)
                duration: 250; easing.type: Easing.InOutQuad
            }
            SequentialAnimation {
                NumberAnimation {
                    target: ghost; property: "lift"; to: root.hopHeight
                    duration: 125; easing.type: Easing.OutQuad
                }
                NumberAnimation {
                    target: ghost; property: "lift"; to: 0
                    duration: 125; easing.type: Easing.InQuad
                }
            }
        }

        PauseAnimation { duration: 240 }

        ParallelAnimation {
            NumberAnimation {
                target: ghost; property: "lift"; to: root.pinchHeight
                duration: 140; easing.type: Easing.OutBack
            }
            NumberAnimation { target: ghost; property: "dangle"; to: 1; duration: 140 }
        }

        NumberAnimation {
            target: ghost; property: "x"; to: root.slotX(0)
            duration: 200; easing.type: Easing.InOutQuad
        }

        ParallelAnimation {
            NumberAnimation {
                target: ghost; property: "lift"; to: 0
                duration: 150; easing.type: Easing.InQuad
            }
            NumberAnimation { target: ghost; property: "dangle"; to: 0; duration: 190 }
        }
    }
}
