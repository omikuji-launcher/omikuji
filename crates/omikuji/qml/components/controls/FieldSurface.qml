import QtQuick
import omikuji 1.0

Rectangle {
    id: surface

    property bool focused: false
    property bool squareRight: false

    radius: Theme.radius.sm
    topRightRadius: squareRight ? 0 : radius
    bottomRightRadius: squareRight ? 0 : radius
    color: Theme.fillFields ? (focused ? Theme.fieldBgFocus : Theme.fieldBg) : "transparent"
    border.width: Theme.fillFields ? 0 : 1
    border.color: Theme.outline

    Behavior on color { ColorAnimation { duration: Theme.dur.fast } }

    Rectangle {
        anchors.fill: parent
        radius: parent.radius
        topRightRadius: surface.topRightRadius
        bottomRightRadius: surface.bottomRightRadius
        color: "transparent"
        border.width: 2
        border.color: Theme.accent
        opacity: parent.focused ? 1 : 0
        Behavior on opacity { NumberAnimation { duration: Theme.dur.fast } }
    }
}
