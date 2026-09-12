import QtQuick
import omikuji 1.0

import "../primitives"

Squircle {
    id: root

    property alias text: label.text
    property color tone: Theme.accent

    implicitWidth: label.implicitWidth + Theme.space.sm * 2
    implicitHeight: Math.max(18, label.implicitHeight + Theme.space.xs)
    width: implicitWidth
    height: implicitHeight
    radius: Theme.radius.xs
    fillColor: Theme.alpha(root.tone, 0.16)

    Text {
        id: label
        anchors.centerIn: parent
        color: root.tone
        font.pixelSize: Theme.type.micro.size
        font.weight: Font.DemiBold
        font.capitalization: Font.AllUppercase
        font.letterSpacing: 0.5
    }
}
