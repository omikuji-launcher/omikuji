pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Effects

Item {
    id: icon

    property string name: ""
    property color color: "#ffffff"
    property int size: 20
    property bool _fillMissing: false

    readonly property int _res: Math.max(1, Math.round(size * Theme.uiScale))

    onNameChanged: _fillMissing = false

    width: size
    height: size

    Image {
        id: img
        anchors.fill: parent
        source: {
            if (!icon.name) return ""
            let fill = Theme.filledIcons && !icon._fillMissing && !icon.name.endsWith("_fill")
            return "qrc:/qt/qml/omikuji/qml/icons/" + icon.name + (fill ? "_fill" : "") + ".svg"
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
        onStatusChanged: if (status === Image.Error && Theme.filledIcons && !icon._fillMissing) Qt.callLater(function() { icon._fillMissing = true })
    }
}
