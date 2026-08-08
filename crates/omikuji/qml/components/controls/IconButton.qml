pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../primitives"


Item {
    id: btn

    property string icon: ""
    property int size: 28
    property bool rounded: false
    property bool danger: false
    // blocked: looks disabled and swallows clicks but still hovers, Item.enabled would kill hover events
    property bool blocked: false

    readonly property bool hovered: hoverArea.containsMouse

    signal clicked()

    width: size
    height: size

    opacity: blocked ? 0.35 : 1.0
    Behavior on opacity { NumberAnimation { duration: 140 } }

    Rectangle {
        anchors.fill: parent
        radius: btn.rounded ? btn.size / 2 : 8
        color: hoverArea.containsPress ? (btn.danger ? Theme.alpha(Theme.error, 0.28) : Theme.statePressed)
              : hoverArea.containsMouse ? (btn.danger ? Theme.alpha(Theme.error, 0.18) : Theme.stateHover)
              : Theme.alpha(Theme.text, 0)
        scale: hoverArea.containsPress ? 0.9 : 1.0

        Behavior on color {
            ColorAnimation { duration: 100 }
        }
        Behavior on scale {
            NumberAnimation { duration: 80; easing.type: Easing.OutCubic }
        }

        SvgIcon {
            anchors.centerIn: parent
            name: btn.icon
            size: Math.round(btn.size * 0.55)
            color: hoverArea.containsMouse ? (btn.danger ? Theme.error : Theme.iconHover) : Theme.icon

            Behavior on color {
                ColorAnimation { duration: 100 }
            }
        }
    }

    MouseArea {
        id: hoverArea
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: btn.blocked ? Qt.ForbiddenCursor : Qt.PointingHandCursor
        onClicked: {
            if (btn.blocked) return
            btn.clicked()
        }
    }
}
