import QtQuick
import omikuji 1.0

FieldSurface {
    id: root

    property alias label: sw.label
    property alias description: sw.description
    property alias checked: sw.checked

    signal toggled(bool val)

    implicitHeight: sw.implicitHeight + Theme.space.md * 2

    LabeledSwitch {
        id: sw
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: Theme.space.md
        anchors.rightMargin: Theme.space.md
        anchors.verticalCenter: parent.verticalCenter
        onToggled: (val) => root.toggled(val)
    }
}
