pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Rectangle {
    id: root

    property alias text: label.text
    property string icon: "info"
    property color tone: Theme.accent

    radius: Theme.radius.md
    color: Theme.alpha(root.tone, 0.10)
    implicitHeight: noteRow.implicitHeight + Theme.space.md * 2
    height: implicitHeight

    Row {
        id: noteRow
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.space.md
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.space.sm

        SvgIcon {
            name: root.icon
            size: 18
            color: root.tone
            anchors.verticalCenter: parent.verticalCenter
        }
        Text {
            id: label
            width: parent.width - 18 - Theme.space.sm
            color: Theme.text
            font.pixelSize: Theme.type.caption.size
            wrapMode: Text.WordWrap
        }
    }
}
