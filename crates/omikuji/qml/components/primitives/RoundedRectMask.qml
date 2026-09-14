import QtQuick

ShaderEffect {
    property var source: null
    property real radius: 0
    property rect maskRect: Qt.rect(0, 0, width, height)
    readonly property size itemSize: Qt.size(width, height)

    fragmentShader: Qt.resolvedUrl("shaders/rounded_rect_mask.frag.qsb")
}
