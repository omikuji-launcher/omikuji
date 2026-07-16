import QtQuick

Item {
    id: root

    property string text: ""
    property string placeholder: ""
    property string label: ""
    property bool readOnly: false

    signal textEdited(string text)
    signal accepted()

    implicitWidth: 200
    implicitHeight: label ? labelText.height + 4 + field.height : field.height

    onTextChanged: if (input.text !== text) input.text = text

    Text {
        id: labelText
        text: root.label
        color: input.activeFocus ? theme.accent : theme.textMuted
        font.pixelSize: 14
        font.weight: Font.Medium
        visible: root.label !== ""

        Behavior on color {
            ColorAnimation { duration: 100 }
        }
    }

    FieldSurface {
        id: field
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: 44
        focused: input.activeFocus

        TextInput {
            id: input
            anchors.fill: parent
            anchors.leftMargin: 12
            anchors.rightMargin: 12
            verticalAlignment: TextInput.AlignVCenter
            color: theme.text
            font.pixelSize: 14
            clip: true
            readOnly: root.readOnly
            selectionColor: theme.accent
            selectedTextColor: theme.accentText
            selectByMouse: true

            onTextEdited: root.textEdited(input.text)
            onAccepted: root.accepted()

            Text {
                anchors.fill: parent
                verticalAlignment: Text.AlignVCenter
                text: root.placeholder
                color: theme.textSubtle
                font.pixelSize: 14
                visible: !input.text && !input.activeFocus
            }
        }
    }
}
