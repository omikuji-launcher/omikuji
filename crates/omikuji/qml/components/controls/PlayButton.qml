import QtQuick
import "../primitives"

Item {
    id: btn

    width: 110
    height: 36

    signal clicked()

    Rectangle {
        id: bg
        anchors.fill: parent
        radius: 18
        color: theme.accent
        opacity: hoverArea.containsPress ? 0.8 : (hoverArea.containsMouse ? 0.95 : 0.9)
        scale: hoverArea.containsPress ? 0.97 : 1.0

        Behavior on opacity {
            NumberAnimation { duration: 100 }
        }
        Behavior on scale {
            NumberAnimation { duration: 100; easing.type: Easing.OutCubic }
        }
    }

    Row {
        anchors.centerIn: parent
        spacing: 6

        SvgIcon {
            anchors.verticalCenter: parent.verticalCenter
            name: "play_arrow"
            size: 18
            color: theme.accentOn
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Play")
            color: theme.accentOn
            font.pixelSize: theme.type.label.size
            font.weight: Font.DemiBold
        }
    }

    MouseArea {
        id: hoverArea
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: btn.clicked()
    }
}
