import QtQuick
import omikuji 1.0

Item {
    id: root

    property color accentColor: Theme.accent
    property color coreColor: Theme.coreTone(root.accentColor)
    property real lookUp: 0
    property real squeeze: 0
    property real wobble: 0
    property real blush: 1
    property real strain: 0
    property bool animated: true

    implicitWidth: 136
    implicitHeight: 112

    ShaderEffect {
        id: fx

        anchors.fill: parent
        blending: true

        property real time: 0
        property real lookUp: root.lookUp
        property real squeeze: root.squeeze
        property real wobble: root.wobble
        property real blush: root.blush
        property real strain: root.strain
        property size resolution: Qt.size(fx.width, fx.height)
        property color accentColor: root.accentColor
        property color coreColor: root.coreColor

        fragmentShader: "shaders/spirit_posed.frag.qsb"

        NumberAnimation on time {
            from: 0
            to: 1000000
            duration: 1000000000
            loops: Animation.Infinite
            running: root.animated && root.visible
        }
    }
}
