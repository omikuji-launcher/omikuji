import QtQuick
import "../controls"
import "../primitives"

DialogCard {
    id: root
    sizeKey: "run_command"

    property string contextTitle: ""
    property string contextText: ""
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
        appendLine((outputText.length ? "\n" : "") + "$ " + cmd)
        commandValue = ""
        submitted(cmd)
    }

    onCloseRequested: close()

    body: Column {
        width: parent.width
        spacing: theme.space.md

        Squircle {
            width: parent.width
            height: contextColumn.implicitHeight + theme.space.md
            radius: theme.radius.sm
            fillColor: theme.alpha(theme.text, 0.06)

            Column {
                id: contextColumn
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                anchors.leftMargin: theme.space.md
                anchors.rightMargin: theme.space.md
                spacing: 2

                Text {
                    width: parent.width
                    visible: root.contextTitle !== ""
                    text: root.contextTitle
                    color: theme.text
                    font.pixelSize: 13
                    font.weight: Font.Medium
                    elide: Text.ElideRight
                }
                Text {
                    width: parent.width
                    text: root.contextText
                    color: theme.accent
                    font.pixelSize: 12
                    font.family: "monospace"
                    wrapMode: Text.WrapAnywhere
                }
            }
        }

        Row {
            width: parent.width
            spacing: theme.space.sm

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
