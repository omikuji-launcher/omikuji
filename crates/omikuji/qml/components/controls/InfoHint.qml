pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

import "../primitives"
import "../popups"

Item {
    id: root

    property string text: ""
    property int iconSize: 14

    implicitWidth: iconSize
    implicitHeight: iconSize

    SvgIcon {
        anchors.centerIn: parent
        name: "info"
        size: root.iconSize
        color: hover.hovered ? Theme.icon : Theme.textMuted

        Behavior on color {
            ColorAnimation { duration: Theme.dur.fast }
        }
    }

    HoverHandler {
        id: hover
        cursorShape: Qt.WhatsThisCursor
    }

    Tooltip {
        text: root.text
        tipVisible: hover.hovered
        showDelay: 0
    }
}
