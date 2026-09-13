import QtQuick
import omikuji 1.0

Item {
    id: root

    property color gateColor: "#d0462e"
    property color gateCoreColor: Qt.lighter(root.gateColor, 1.35)
    property color trimColor: "#2b1c1e"
    property color spiritColor: "#ffffff"
    property color spiritCoreColor: Theme.coreTone(root.spiritColor)
    property bool animated: true
    property int supersample: 4

    implicitWidth: 88
    implicitHeight: 76

    ShaderEffect {
        id: fx

        anchors.fill: parent
        blending: true

        layer.enabled: true
        layer.smooth: true
        // 4x layer aliases on hard edges without mips
        layer.mipmap: true
        layer.textureSize: Qt.size(Math.round(fx.width * root.supersample),
                                   Math.round(fx.height * root.supersample))

        property real time: 0
        property size resolution: Qt.size(fx.width, fx.height)
        property color gateColor: root.gateColor
        property color gateCoreColor: root.gateCoreColor
        property color trimColor: root.trimColor
        property color spiritColor: root.spiritColor
        property color spiritCoreColor: root.spiritCoreColor

        fragmentShader: "qrc:/qt/qml/omikuji/qml/components/primitives/shaders/torii.frag.qsb"

        NumberAnimation on time {
            from: 0
            to: 1000000
            duration: 1000000000
            loops: Animation.Infinite
            running: root.animated && root.visible
        }
    }
}
