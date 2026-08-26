pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Controls

Rectangle {
    id: root

    property alias text: area.text
    property alias textArea: area
    property alias textColor: area.color
    property alias fontSize: area.font.pixelSize
    property bool follow: true

    color: Theme.bgAlt
    radius: Theme.radius.sm
    clip: true

    ThemedLogHighlighter {
        id: highlighter
        settings: appSettings
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
        Component.onCompleted: contentItem.boundsBehavior = Flickable.StopAtBounds

        TextArea {
            id: area
            readOnly: true
            wrapMode: TextArea.Wrap
            selectByMouse: true
            color: Theme.textMuted
            font.family: "monospace"
            font.pixelSize: Theme.type.caption.size
            background: Rectangle { color: "transparent" }
            onTextChanged: if (root.follow) cursorPosition = length
        }
    }
}
