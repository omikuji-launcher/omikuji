import QtQuick
import omikuji 1.0

Item {
    id: root

    property string text: ""
    property string placeholder: ""
    property string label: ""
    property bool readOnly: false
    property var gameModel: null
    property bool expandHint: true
    property var expandWith: expandHint && gameModel ? (t) => gameModel.expandVars(t) : null

    readonly property real boxCenterY: field.y + field.height / 2

    signal textEdited(string text)
    signal accepted()

    implicitWidth: 200
    implicitHeight: (label ? labelText.height + 4 : 0) + field.height
                    + (hint.visible ? hint.implicitHeight + 3 : 0)

    onTextChanged: if (input.text !== text) input.text = text

    Text {
        id: labelText
        text: root.label
        color: input.activeFocus ? Theme.accent : Theme.textMuted
        font.pixelSize: Theme.type.body.size
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
        anchors.top: parent.top
        anchors.topMargin: root.label ? labelText.height + 4 : 0
        height: 44
        focused: input.activeFocus

        TextInput {
            id: input
            anchors.fill: parent
            anchors.leftMargin: 12
            anchors.rightMargin: 12
            verticalAlignment: TextInput.AlignVCenter
            color: Theme.text
            font.pixelSize: Theme.type.body.size
            clip: true
            readOnly: root.readOnly
            selectionColor: Theme.accent
            selectedTextColor: Theme.accentText
            selectByMouse: true

            onTextEdited: root.textEdited(input.text)
            onAccepted: root.accepted()

            Text {
                anchors.fill: parent
                verticalAlignment: Text.AlignVCenter
                text: root.placeholder
                color: Theme.textSubtle
                font.pixelSize: Theme.type.body.size
                visible: !input.text && !input.activeFocus
            }
        }
    }

    ExpansionHint {
        id: hint
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: field.bottom
        anchors.topMargin: 3
        anchors.leftMargin: 2
        source: input.text
        resolver: root.expandWith
    }
}
