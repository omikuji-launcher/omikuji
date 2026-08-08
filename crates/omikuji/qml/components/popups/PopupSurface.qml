import QtQuick
import omikuji 1.0
import Qt5Compat.GraphicalEffects

Rectangle {
    id: surface

    color: Theme.active.window.hslLightness > 0.5
        ? Qt.darker(Theme.popup, 1.06)
        : Qt.lighter(Theme.popup, 1.3)
    radius: Theme.radius.md

    RectangularGlow {
        z: -1
        anchors.fill: parent
        anchors.topMargin: 3
        anchors.bottomMargin: -3
        glowRadius: 16
        spread: 0.05
        color: Qt.rgba(0, 0, 0, 0.35)
        cornerRadius: surface.radius + 16
    }
}
