import QtQuick
import omikuji 1.0

Item {
    id: root

    property color color: "#ffffff"
    property color coreColor: Theme.coreTone(root.color)
    property bool animated: true
    property int supersample: 4
    property real phase: 0

    implicitWidth: 20
    implicitHeight: 26

    ShaderEffect {
        id: fx

        anchors.fill: parent
        blending: true

        layer.enabled: true
        layer.smooth: true
        layer.textureSize: Qt.size(Math.round(fx.width * root.supersample),
                                   Math.round(fx.height * root.supersample))

        property real time: 0
        property real phase: root.phase
        property size resolution: Qt.size(fx.width, fx.height)
        property color accentColor: root.color
        property color coreColor: root.coreColor

        fragmentShader: "shaders/hitodama.frag.qsb"

        NumberAnimation on time {
            from: 0
            to: 1000000
            duration: 1000000000
            loops: Animation.Infinite
            running: root.animated && root.visible
        }
    }
}
