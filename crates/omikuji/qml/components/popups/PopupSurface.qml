import QtQuick
import omikuji 1.0
import QtQuick.Effects

Rectangle {
    id: surface

    color: Theme.active.window.hslLightness > 0.5
        ? Qt.darker(Theme.popup, 1.06)
        : Qt.lighter(Theme.popup, 1.3)
    radius: Theme.radius.md

    RectangularShadow {
        z: -1
        anchors.fill: parent
        anchors.topMargin: 3
        anchors.bottomMargin: -3
        blur: 16
        radius: surface.radius
        color: Qt.rgba(0, 0, 0, 0.35)
    }
}
