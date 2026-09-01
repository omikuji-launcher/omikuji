import QtQuick
import omikuji 1.0

Item {
    id: root

    property string label: ""
    property string placeholder: ""
    property string text: ""
    property bool selectFolder: false
    property string filter: ""
    property bool readOnly: false
    property string trailingHint: ""
    property var gameModel: null
    property bool expandHint: true
    property var expandWith: expandHint && gameModel ? (t) => gameModel.expandVars(t) : null
    property alias trailingActions: trailing.data

    readonly property real boxCenterY: fieldRow.y + fieldRow.height / 2

    signal textEdited(string text)
    signal accepted(string path)

    onTextChanged: if (inputArea.text !== text) inputArea.text = text

    implicitWidth: 200
    implicitHeight: (label ? labelText.height + 4 : 0) + fieldRow.height
                    + (expansion.visible ? expansion.implicitHeight + 3 : 0)

    Text {
        id: labelText
        text: root.label
        color: inputArea.activeFocus ? Theme.accent : Theme.textMuted
        font.pixelSize: Theme.type.body.size
        font.weight: Font.Medium
        visible: root.label !== ""

        Behavior on color {
            ColorAnimation { duration: 100 }
        }
    }

    Row {
        id: fieldRow
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.topMargin: root.label ? labelText.height + 4 : 0
        height: 44
        spacing: 8

        FieldSurface {
            id: inputBg
            width: parent.width - folderBtn.width - parent.spacing
                   - (trailing.visible ? trailing.width + parent.spacing : 0)
            height: parent.height
            focused: inputArea.activeFocus

            TextInput {
                id: inputArea
                anchors.left: parent.left
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                anchors.leftMargin: 12
                anchors.right: parent.right
                anchors.rightMargin: 12
                verticalAlignment: TextInput.AlignVCenter
                color: root.readOnly ? Theme.textMuted : Theme.text
                font.pixelSize: Theme.type.body.size
                clip: true
                readOnly: root.readOnly
                selectByMouse: !root.readOnly
                selectionColor: Theme.accent
                selectedTextColor: Theme.accentText

                onTextEdited: root.textEdited(text)
                onAccepted: root.accepted(text)
            }

            // x-positioned not anchored, anchoring feeds back into inputArea.width and causes a binding loop apparently
            Text {
                id: hintText
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                verticalAlignment: Text.AlignVCenter
                x: inputArea.x + inputArea.contentWidth + 2
                width: Math.max(0, Math.min(
                    implicitWidth,
                    parent.width - 12 - x
                ))
                text: root.trailingHint
                color: Theme.alpha(Theme.text, 0.4)
                font.pixelSize: Theme.type.body.size
                elide: Text.ElideRight
                visible: root.trailingHint !== "" && inputArea.text !== ""
            }

            Text {
                anchors.fill: parent
                anchors.leftMargin: 12
                verticalAlignment: Text.AlignVCenter
                text: root.placeholder
                color: Theme.textSubtle
                font.pixelSize: Theme.type.body.size
                visible: inputArea.text === "" && !inputArea.activeFocus
            }
        }

        FieldButton {
            id: folderBtn
            icon: "folder"
            blocked: root.readOnly
            onClicked: picker.open()
        }

        Row {
            id: trailing
            height: parent.height
            spacing: parent.spacing
            visible: children.length > 0
        }
    }

    ExpansionHint {
        id: expansion
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: fieldRow.bottom
        anchors.topMargin: 3
        anchors.leftMargin: 2
        source: inputArea.text
        resolver: root.expandWith
    }

    FilePicker {
        id: picker
        selectFolder: root.selectFolder
        title: root.selectFolder ? qsTr("Select Folder") : qsTr("Select File")
        filter: root.filter
        startFolder: root.selectFolder
            ? (root.text || "/home")
            : (root.text.lastIndexOf("/") > 0 ? root.text.substring(0, root.text.lastIndexOf("/")) : "/home")
        onPicked: (path) => {
            root.textEdited(path)
            root.accepted(path)
        }
    }
}
