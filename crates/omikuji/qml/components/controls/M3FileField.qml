import QtQuick
import QtCore
import "../primitives"

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
        color: inputArea.activeFocus ? theme.accent : theme.textMuted
        font.pixelSize: theme.type.body.size
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
                color: root.readOnly ? theme.textMuted : theme.text
                font.pixelSize: theme.type.body.size
                clip: true
                readOnly: root.readOnly
                selectByMouse: !root.readOnly
                selectionColor: theme.accent
                selectedTextColor: theme.accentText

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
                color: theme.alpha(theme.text, 0.4)
                font.pixelSize: theme.type.body.size
                elide: Text.ElideRight
                visible: root.trailingHint !== "" && inputArea.text !== ""
            }

            Text {
                anchors.fill: parent
                anchors.leftMargin: 12
                verticalAlignment: Text.AlignVCenter
                text: root.placeholder
                color: theme.textSubtle
                font.pixelSize: theme.type.body.size
                visible: inputArea.text === "" && !inputArea.activeFocus
            }
        }

        FieldSurface {
            id: folderBtn
            width: 44
            height: 44
            opacity: root.readOnly ? 0.4 : 1.0

            Rectangle {
                anchors.fill: parent
                radius: parent.radius
                color: folderMouse.containsPress ? theme.statePressed
                      : folderMouse.containsMouse ? theme.stateHover
                      : "transparent"

                Behavior on color {
                    ColorAnimation { duration: theme.dur.fast }
                }
            }

            SvgIcon {
                anchors.centerIn: parent
                name: "folder"
                size: 20
                color: folderMouse.containsMouse ? theme.iconHover : theme.icon

                Behavior on color {
                    ColorAnimation { duration: 100 }
                }
            }

            MouseArea {
                id: folderMouse
                anchors.fill: parent
                hoverEnabled: !root.readOnly
                enabled: !root.readOnly
                cursorShape: root.readOnly ? Qt.ArrowCursor : Qt.PointingHandCursor
                onClicked: openFileDialog()
            }
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

    property string _dialogRequestId: ""

    Connections {
        target: root.gameModel
        enabled: root._dialogRequestId !== ""
        function onFile_dialog_result(requestId, path) {
            if (requestId !== root._dialogRequestId) return
            root._dialogRequestId = ""
            if (path && path !== "") {
                root.textEdited(path)
                root.accepted(path)
            }
        }
    }

    function openFileDialog() {
        if (!gameModel) {
            return
        }

        let title = root.selectFolder ? qsTr("Select Folder") : qsTr("Select File")
        let defaultPath = root.text || "/home"

        let id = Date.now().toString(36) + Math.random().toString(36).substring(2, 8)
        root._dialogRequestId = id
        gameModel.open_file_dialog(id, root.selectFolder, title, defaultPath, root.filter)
    }
}
