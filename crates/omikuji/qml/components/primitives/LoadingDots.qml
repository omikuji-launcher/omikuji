import QtQuick

Item {
    id: root

    property string text: ""
    property bool running: false
    property color textColor: theme.textMuted

    implicitWidth: label.implicitWidth + dotsRow.implicitWidth
    implicitHeight: Math.max(label.implicitHeight, dotsRow.implicitHeight)

    Row {
        anchors.centerIn: parent
        spacing: 2

        Text {
            id: label
            text: root.text
            color: root.textColor
            font.pixelSize: theme.type.title.size
            font.weight: Font.Medium
            anchors.verticalCenter: parent.verticalCenter
        }

        Row {
            id: dotsRow
            spacing: 3
            anchors.verticalCenter: parent.verticalCenter
            anchors.verticalCenterOffset: 1

            Repeater {
                model: 3

                Text {
                    required property int index
                    text: "."
                    color: root.textColor
                    font.pixelSize: 20
                    font.weight: Font.Bold
                    lineHeight: 0.6

                    // 150ms stagger per dot so they read as a wave not unison blinking (they still suck)
                    y: 0
                    SequentialAnimation on y {
                        loops: Animation.Infinite
                        running: root.running
                        PauseAnimation { duration: index * 150 }
                        NumberAnimation { from: 0; to: -4; duration: 220; easing.type: Easing.OutQuad }
                        NumberAnimation { from: -4; to: 0; duration: 220; easing.type: Easing.InQuad }
                        PauseAnimation { duration: 600 - index * 150 }
                    }
                }
            }
        }
    }
}
