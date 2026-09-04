pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../primitives"


Row {
    id: root

    property var game: null
    property real nameMaxWidth: 200

    function playtimeLabel() {
        let h = root.game ? root.game.playtime : 0
        return h >= 1 ? Math.floor(h) + "h " + Math.round((h % 1) * 60) + "m"
                      : Math.round(h * 60) + "m"
    }

    component Dot: Rectangle {
        width: 4; height: 4; radius: 2
        color: Theme.dot
        anchors.verticalCenter: parent.verticalCenter
    }

    component Stat: Row {
        id: stat

        property string icon: ""
        property string label: ""
        property color tint: Theme.textMuted

        spacing: 6
        anchors.verticalCenter: parent.verticalCenter

        SvgIcon {
            name: stat.icon
            size: 14
            color: stat.tint
            anchors.verticalCenter: parent.verticalCenter
        }

        Text {
            text: stat.label
            color: stat.tint
            font.pixelSize: Theme.type.caption.size
            anchors.verticalCenter: parent.verticalCenter
        }
    }

    spacing: 16

    Text {
        text: root.game ? root.game.name : ""
        color: Theme.text
        font.pixelSize: Theme.type.body.size
        font.weight: Font.DemiBold
        elide: Text.ElideRight
        width: Math.min(implicitWidth, root.nameMaxWidth)
        anchors.verticalCenter: parent.verticalCenter
    }

    Dot {}

    Stat {
        icon: "schedule"
        label: root.playtimeLabel()
        tint: Theme.textMuted
    }

    Dot {}

    Stat {
        icon: "calendar_month"
        label: root.game ? root.game.lastPlayed : ""
        tint: Theme.textSubtle
    }

    Dot {}

    Text {
        text: root.game ? root.game.runner : ""
        color: Theme.textFaint
        font.pixelSize: Theme.type.caption.size
        anchors.verticalCenter: parent.verticalCenter
    }
}
