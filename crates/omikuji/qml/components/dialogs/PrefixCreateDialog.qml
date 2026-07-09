import QtQuick
import QtQuick.Controls

import "../lib/RunnerGrouping.js" as RG
import "../controls"

DialogCard {
    id: root

    property var gameModel: null
    property var ofudaBridge: null

    readonly property bool busy: ofudaBridge ? ofudaBridge.creating : false
    property string errorText: ""

    property string nameValue: ""
    property string runnerValue: ""
    property string setValue: "game"
    property string outputText: ""

    maxWidth: 460
    title: qsTr("New prefix")

    function show() {
        nameValue = ""
        errorText = ""
        outputText = ""
        open()
    }

    onCloseRequested: if (!busy) close()

    Connections {
        target: ofudaBridge
        enabled: ofudaBridge !== null
        function onCreateOutput(line) {
            root.outputText += (root.outputText.length ? "\n" : "") + line
        }
        function onCreateFinished(ok, error) {
            if (ok) root.close()
            else root.errorText = (error && error.length > 0) ? error : qsTr("winetricks failed")
        }
    }

    body: Column {
        width: parent.width
        spacing: theme.space.md

        Column {
            width: parent.width
            spacing: theme.space.md
            enabled: !root.busy

            M3TextField {
                label: qsTr("Name")
                placeholder: qsTr("my-prefix")
                width: parent.width
                text: root.nameValue
                onTextEdited: (t) => root.nameValue = t
            }

            M3Dropdown {
                label: qsTr("Runner")
                width: parent.width
                options: RG.groupRunners(JSON.parse(root.gameModel ? root.gameModel.list_runners() : "[]"))
                currentIndex: {
                    let f = RG.firstNonHeader(options)
                    return f >= 0 ? f : 0
                }
                onSelected: (v) => root.runnerValue = v
                Component.onCompleted: root.runnerValue = currentValue
            }

            M3Dropdown {
                label: qsTr("Set")
                width: parent.width
                options: [
                    { label: qsTr("Game"), value: "game" },
                    { label: qsTr("Application"), value: "app" }
                ]
                onSelected: (v) => root.setValue = v
                Component.onCompleted: root.setValue = currentValue
            }
        }

        Column {
            width: parent.width
            spacing: theme.space.xs
            visible: root.busy || root.outputText.length > 0

            Text {
                visible: root.busy
                text: qsTr("Setting up your Ofuda…")
                color: theme.accent
                font.pixelSize: 12
            }

            OutputLog {
                width: parent.width
                height: 200
                text: root.outputText
            }
        }

        Text {
            visible: root.errorText !== ""
            width: parent.width
            text: root.errorText
            color: theme.error
            font.pixelSize: 12
            wrapMode: Text.WordWrap
        }
    }

    actions: Row {
        spacing: theme.space.sm

        M3Button {
            text: qsTr("Cancel")
            variant: "tonal"
            enabled: !root.busy
            onClicked: root.close()
        }
        M3Button {
            text: root.busy ? qsTr("Working…") : qsTr("Create")
            variant: "filled"
            enabled: !root.busy && root.nameValue.trim() !== "" && !!root.runnerValue
            onClicked: root.ofudaBridge.createPrefix(root.nameValue.trim(), root.runnerValue, root.setValue)
        }
    }
}
