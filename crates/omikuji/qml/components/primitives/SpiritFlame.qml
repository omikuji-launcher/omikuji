import QtQuick

Item {
    id: root

    property color color: "#ffffff"
    property color coreColor: Qt.hsla(root.color.hslHue,
                                      Math.min(1, root.color.hslSaturation * 2.4),
                                      Math.min(1, root.color.hslLightness * 1.10),
                                      1)
    property bool animated: true
    property int supersample: 4

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
        property size resolution: Qt.size(fx.width, fx.height)
        property color accentColor: root.color
        property color coreColor: root.coreColor

        fragmentShader: "qrc:/qt/qml/omikuji/qml/components/primitives/shaders/hitodama.frag.qsb"

        NumberAnimation on time {
            from: 0
            to: 1000000
            duration: 1000000000
            loops: Animation.Infinite
            running: root.animated && root.visible
        }
    }
}
