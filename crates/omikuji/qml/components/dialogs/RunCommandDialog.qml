pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../controls"
import "../primitives"

DialogCard {
    id: root
    sizeKey: "run_command"

    property string contextTitle: ""
    property string contextText: ""
    property var expander: null
    property bool running: false
    property string commandValue: ""
    property string outputText: ""

    signal submitted(string command)

    maxWidth: 560
    title: qsTr("Run wine command")

    function show(ctxTitle, ctx) {
        contextTitle = ctxTitle
        contextText = ctx
        commandValue = ""
        if (!running) outputText = ""
        open()
    }

    function appendLine(line) {
        outputText += (outputText.length ? "\n" : "") + line
    }

    function commandDone(ok, error) {
        if (!ok) appendLine(error && error.length ? error : qsTr("command failed"))
    }

    function submit() {
        let cmd = commandValue.trim()
        if (running || cmd === "") return
        appendLine((outputText.length ? "\n" : "") + "$ " + (expander ? expander(cmd) : cmd))
        commandValue = ""
        submitted(cmd)
    }

    onCloseRequested: close()

    body: Column {
        width: parent.width
        spacing: Theme.space.lg

        Squircle {
            width: parent.width
            height: contextColumn.implicitHeight + Theme.space.md
            radius: Theme.radius.sm
            fillColor: Theme.alpha(Theme.text, 0.06)

            Column {
                id: contextColumn
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                anchors.leftMargin: Theme.space.md
                anchors.rightMargin: Theme.space.md
                spacing: 2

                Text {
                    width: parent.width
                    visible: root.contextTitle !== ""
                    text: root.contextTitle
                    color: Theme.text
                    font.pixelSize: Theme.type.label.size
                    font.weight: Font.Medium
                    elide: Text.ElideRight
                }
                Text {
                    width: parent.width
                    text: root.contextText
                    color: Theme.accent
                    font.pixelSize: Theme.type.caption.size
                    font.family: "monospace"
                    wrapMode: Text.WrapAnywhere
                }
            }
        }

        Row {
            width: parent.width
            spacing: Theme.space.sm

            M3TextField {
                width: parent.width - runButton.width - parent.spacing
                placeholder: "winetricks corefonts"
                text: root.commandValue
                enabled: !root.running
                onTextEdited: (t) => root.commandValue = t
                onAccepted: root.submit()
            }

            M3Button {
                id: runButton
                anchors.verticalCenter: parent.verticalCenter
                text: root.running ? qsTr("Running…") : qsTr("Run")
                variant: "filled"
                enabled: !root.running && root.commandValue.trim() !== ""
                onClicked: root.submit()
            }
        }

        OutputLog {
            width: parent.width
            height: 220
            visible: root.running || root.outputText.length > 0
            text: root.outputText
        }
    }

    actions: M3Button {
        text: qsTr("Close")
        variant: "tonal"
        onClicked: root.close()
    }
}
