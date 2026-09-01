pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../primitives"

FieldSurface {
    id: root

    property string icon: ""
    property bool blocked: false

    signal clicked()

    implicitWidth: 44
    implicitHeight: 44
    opacity: blocked ? 0.4 : 1.0

    Behavior on opacity { NumberAnimation { duration: Theme.dur.fast } }

    Rectangle {
        anchors.fill: parent
        radius: parent.radius
        color: root.blocked ? "transparent"
              : mouse.containsPress ? Theme.statePressed
              : mouse.containsMouse ? Theme.stateHover
              : "transparent"

        Behavior on color { ColorAnimation { duration: Theme.dur.fast } }
    }

    SvgIcon {
        anchors.centerIn: parent
        name: root.icon
        size: 20
        color: !root.blocked && mouse.containsMouse ? Theme.iconHover : Theme.icon

        Behavior on color { ColorAnimation { duration: 100 } }
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: root.blocked ? Qt.ForbiddenCursor : Qt.PointingHandCursor
        onClicked: if (!root.blocked) root.clicked()
    }
}
