import QtQuick
import omikuji 1.0

Squircle {
    id: root

    default property alias content: inner.data
    property alias spacing: inner.spacing

    width: parent ? parent.width : 400
    height: inner.implicitHeight + Theme.space.md
    radius: Theme.radius.sm
    fillColor: Theme.alpha(Theme.text, 0.06)

    Column {
        id: inner
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        anchors.leftMargin: Theme.space.md
        anchors.rightMargin: Theme.space.md
    }
}
