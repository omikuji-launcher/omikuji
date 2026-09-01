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
    property bool tonal: false
    property bool squircle: false
    // blocked: looks disabled and swallows clicks but still hovers, Item.enabled would kill hover events
    property bool blocked: false

    readonly property bool hovered: hoverArea.containsMouse

    readonly property color fill: {
        if (btn.tonal)
            return Theme.alpha(Theme.accent, hoverArea.containsPress ? 0.28
                : hoverArea.containsMouse ? 0.20 : 0.13)
        if (hoverArea.containsPress)
            return btn.danger ? Theme.alpha(Theme.error, 0.28) : Theme.statePressed
        if (hoverArea.containsMouse)
            return btn.danger ? Theme.alpha(Theme.error, 0.18) : Theme.stateHover
        return Theme.alpha(Theme.text, 0)
    }

    signal clicked()

    width: size
    height: size

    opacity: blocked ? 0.35 : 1.0
    Behavior on opacity { NumberAnimation { duration: 140 } }

    Item {
        anchors.fill: parent
        scale: hoverArea.containsPress ? 0.9 : 1.0

        Behavior on scale {
            NumberAnimation { duration: 80; easing.type: Easing.OutCubic }
        }

        Rectangle {
            anchors.fill: parent
            visible: !btn.squircle
            radius: btn.rounded ? btn.size / 2 : 8
            color: btn.fill

            Behavior on color {
                ColorAnimation { duration: 100 }
            }
        }

        Squircle {
            anchors.fill: parent
            visible: btn.squircle
            radius: btn.rounded ? btn.size / 2 : Theme.radius.md
            fillColor: btn.fill

            Behavior on fillColor {
                ColorAnimation { duration: 100 }
            }
        }
    }

    SvgIcon {
        anchors.centerIn: parent
        name: btn.icon
        size: Math.round(btn.size * 0.55)
        color: btn.tonal ? Theme.accent
            : hoverArea.containsMouse ? (btn.danger ? Theme.error : Theme.iconHover)
            : Theme.icon

        Behavior on color {
            ColorAnimation { duration: 100 }
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
