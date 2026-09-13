import QtQuick
import omikuji 1.0

Item {
    id: root

    property string text: ""
    property bool running: false
    property color color: Theme.accent
    property color coreColor: Theme.coreTone(root.color)
    property int supersample: 3
    property int spiritWidth: 136
    property int textSize: Theme.type.body.size

    implicitWidth: Math.max(root.spiritWidth, label.implicitWidth)
    implicitHeight: stack.implicitHeight

    Column {
        id: stack
        anchors.centerIn: parent
        spacing: Theme.space.sm

        ShaderEffect {
            id: fx

            anchors.horizontalCenter: parent.horizontalCenter
            width: root.spiritWidth
            height: Math.round(root.spiritWidth / 2)
            blending: true

            layer.enabled: true
            layer.smooth: true
            layer.textureSize: Qt.size(Math.round(fx.width * root.supersample),
                                       Math.round(fx.height * root.supersample))

            property real time: 0
            property size resolution: Qt.size(fx.width, fx.height)
            property color accentColor: root.color
            property color coreColor: root.coreColor

            fragmentShader: "qrc:/qt/qml/omikuji/qml/components/primitives/shaders/spirit.frag.qsb"

            NumberAnimation on time {
                from: 0
                to: 1000000
                duration: 1000000000
                loops: Animation.Infinite
                running: root.running && root.visible
            }
        }

        Text {
            id: label
            anchors.horizontalCenter: parent.horizontalCenter
            visible: root.text !== ""
            text: root.text
            color: Theme.textFaint
            font.pixelSize: root.textSize
        }
    }
}
