import QtQuick
import omikuji 1.0

Rectangle {
    property bool focused: false

    radius: Theme.radius.sm
    color: Theme.fillFields ? (focused ? Theme.fieldBgFocus : Theme.fieldBg) : "transparent"
    border.width: Theme.fillFields ? 0 : 1
    border.color: Theme.outline

    Behavior on color { ColorAnimation { duration: Theme.dur.fast } }

    Rectangle {
        anchors.fill: parent
        radius: parent.radius
        color: "transparent"
        border.width: 2
        border.color: Theme.accent
        opacity: parent.focused ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: Theme.dur.fast } }
    }
}
