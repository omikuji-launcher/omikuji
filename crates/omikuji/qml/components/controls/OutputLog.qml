import QtQuick
import QtQuick.Controls

Rectangle {
    id: root

    property alias text: area.text
    property alias textArea: area
    property alias textColor: area.color
    property alias fontSize: area.font.pixelSize
    property bool follow: true

    color: theme.bgAlt
    radius: theme.radius.sm
    clip: true

    ThemedLogHighlighter {
        id: highlighter
        settings: uiSettings
    }

    Component.onCompleted: highlighter.attach(area.textDocument)

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.NoButton
        onWheel: (wheel) => wheel.accepted = true
    }

    ScrollView {
        anchors.fill: parent
        anchors.margins: 8

        TextArea {
            id: area
            readOnly: true
            wrapMode: TextArea.Wrap
            selectByMouse: true
            color: theme.textMuted
            font.family: "monospace"
            font.pixelSize: theme.type.caption.size
            background: Rectangle { color: "transparent" }
            onTextChanged: if (root.follow) cursorPosition = length
        }
    }
}
