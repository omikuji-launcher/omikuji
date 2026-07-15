import QtQuick
import QtQuick.Effects

Item {
    id: icon

    property string name: ""
    property color color: "#ffffff"
    property int size: 20
    property bool _fillMissing: false

    readonly property int _res: Math.max(1, Math.round(size * theme.uiScale))

    onNameChanged: _fillMissing = false

    width: size
    height: size

    Image {
        id: img
        anchors.fill: parent
        source: {
            if (!name) return ""
            let fill = theme.filledIcons && !icon._fillMissing && !name.endsWith("_fill")
            return "qrc:/qt/qml/omikuji/qml/icons/" + name + (fill ? "_fill" : "") + ".svg"
        }
        sourceSize: Qt.size(icon._res, icon._res)
        layer.enabled: true
        layer.smooth: true
        layer.textureSize: Qt.size(icon._res, icon._res)
        layer.effect: MultiEffect {
            contrast: -1
            brightness: 0.5
            colorization: 1
            colorizationColor: Qt.rgba(icon.color.r, icon.color.g, icon.color.b, 1)
            opacity: icon.color.a
        }
        onStatusChanged: if (status === Image.Error && theme.filledIcons && !icon._fillMissing) Qt.callLater(function() { icon._fillMissing = true })
    }
}
