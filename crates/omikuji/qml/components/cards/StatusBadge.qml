import QtQuick

Item {
    id: badge

    property string status: "ready"

    property var statusMap: ({
        "ready":      { text: qsTr("Ready"),       color: "#4ade80" },
        "update":     { text: qsTr("Update"),      color: "#fbbf24" },
        "installing": { text: qsTr("Installing"),  color: "#60a5fa" },
        "patching":   { text: qsTr("Patching"),    color: "#c084fc" }
    })

    property var info: statusMap[status] ?? statusMap["ready"]

    width: label.width + 20
    height: 28

    Rectangle {
        anchors.fill: parent
        radius: 14
        color: Qt.rgba(0, 0, 0, 0.4)
        border.width: 1
        border.color: Qt.rgba(badge.info.color.r, badge.info.color.g, badge.info.color.b, 0.4)
    }

    Row {
        anchors.centerIn: parent
        spacing: 6

        Rectangle {
            width: 6
            height: 6
            radius: 3
            color: badge.info.color
            anchors.verticalCenter: parent.verticalCenter
        }

        Text {
            id: label
            text: badge.info.text
            color: badge.info.color
            font.pixelSize: theme.type.micro.size
            font.weight: Font.Medium
        }
    }
}
