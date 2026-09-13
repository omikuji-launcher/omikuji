import QtQuick
import omikuji 1.0

Item {
    id: root

    property real value: 0
    property bool animate: true
    property color fillColor
    property color trackColor
    property real offset: 0

    readonly property real thickness: Math.round(height * 0.42)
    readonly property real period: thickness * 1.4

    implicitWidth: 160
    implicitHeight: 24

    NumberAnimation on offset {
        from: 0
        to: root.period
        duration: 700
        loops: Animation.Infinite
        running: root.animate && root.visible
    }

    Rectangle {
        anchors.verticalCenter: parent.verticalCenter
        width: parent.width
        height: root.thickness
        radius: height / 2
        color: root.trackColor
    }

    ShaderEffect {
        anchors.verticalCenter: parent.verticalCenter
        width: Math.max(height, root.width * root.value)
        height: root.thickness
        visible: root.value > 0
        opacity: root.fillColor.a
        blending: true

        property real offset: root.offset
        property real period: root.period
        property size resolution: Qt.size(width, height)
        property color fillColor: Qt.rgba(root.fillColor.r, root.fillColor.g, root.fillColor.b, 1)
        property color stripeColor: Theme.mix(fillColor, Theme.bg, 0.25)

        fragmentShader: "qrc:/qt/qml/omikuji/qml/components/primitives/shaders/stripes.frag.qsb"
    }
}
