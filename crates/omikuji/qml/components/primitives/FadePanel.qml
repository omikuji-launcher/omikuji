import QtQuick
import omikuji 1.0

Rectangle {
    id: root

    property bool shown: false

    // M3Dropdown walks up looking for this flag to reparent its popup
    property bool isDropdownHost: true

    anchors.fill: parent
    color: Theme.surface
    radius: Theme.radius.md
    visible: opacity > 0
    enabled: shown
    opacity: shown ? 1 : 0
    layer.enabled: fade.running

    Behavior on opacity {
        NumberAnimation { id: fade; duration: 200; easing.type: Easing.OutCubic }
    }

    // corner mask so the rounded bottom-right doesnt let content bleed
    Rectangle {
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        width: parent.radius
        height: parent.radius
        color: parent.color
        visible: parent.visible
    }
}
