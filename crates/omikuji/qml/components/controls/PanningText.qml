import QtQuick
import omikuji 1.0

Item {
    id: root

    property alias text: label.text
    property alias color: label.color
    property alias font: label.font
    property color bg: "transparent"
    property bool active: false
    property bool fades: true
    property bool animateFades: true

    readonly property real overflow: width > 0 ? Math.max(0, label.implicitWidth - width) : 0
    property real pan: 0
    property bool manualPan: false

    function panBy(delta) {
        manualPan = true
        pan = Math.max(0, Math.min(overflow, pan + delta))
    }
    function reset() {
        manualPan = false
        pan = 0
    }

    implicitHeight: label.implicitHeight
    clip: true

    Text {
        id: label
        x: -Math.round(root.pan)
    }

    SequentialAnimation {
        running: root.active && root.overflow > 0 && !root.manualPan
        loops: Animation.Infinite
        PauseAnimation { duration: 350 }
        NumberAnimation {
            target: root
            property: "pan"
            to: root.overflow
            duration: Math.max(300, root.overflow * 16)
        }
        PauseAnimation { duration: 650 }
        NumberAnimation {
            target: root
            property: "pan"
            to: 0
            duration: Math.max(300, root.overflow * 16)
        }
        PauseAnimation { duration: 450 }
    }

    Rectangle {
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: 16
        visible: root.fades
        opacity: root.pan > 1 ? 1 : 0
        gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0.0; color: root.bg }
            GradientStop { position: 1.0; color: Theme.alpha(root.bg, 0) }
        }
        Behavior on opacity { enabled: root.animateFades; NumberAnimation { duration: Theme.dur.sm } }
    }

    Rectangle {
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: 16
        visible: root.fades
        opacity: root.overflow > 0 && root.pan < root.overflow - 1 ? 1 : 0
        gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0.0; color: Theme.alpha(root.bg, 0) }
            GradientStop { position: 1.0; color: root.bg }
        }
        Behavior on opacity { enabled: root.animateFades; NumberAnimation { duration: Theme.dur.sm } }
    }
}
