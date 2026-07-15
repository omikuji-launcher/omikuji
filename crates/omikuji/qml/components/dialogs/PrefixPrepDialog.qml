import QtQuick
import QtQuick.Controls
import "../controls"


DialogCard {
    sizeKey: "prefix_prep"
    id: root

    property var gameModel: null
    property int gameIndex: -1
    property bool skipUpdateCheck: false
    property bool cancelled: false
    property string errorText: ""
    property string outputText: ""

    signal launchReady(int idx, bool skip)

    readonly property bool busy: gameModel ? gameModel.preparing : false

    maxWidth: 640
    fillHeight: true
    preferredHeight: 440
    scrollable: false
    title: qsTr("Preparing prefix")

    function start(idx, skip) {
        gameIndex = idx
        skipUpdateCheck = skip
        cancelled = false
        errorText = ""
        outputText = ""
        open()
        if (gameModel) gameModel.prepare_prefix(idx)
    }

    onCloseRequested: { root.cancelled = true; close() }

    Connections {
        target: gameModel
        enabled: gameModel !== null
        function onPrepareOutput(line) {
            root.outputText += (root.outputText.length ? "\n" : "") + line
        }
        function onPrepareFinished(ok, error) {
            if (root.cancelled) return
            if (ok) {
                root.close()
                root.launchReady(root.gameIndex, root.skipUpdateCheck)
            } else {
                root.errorText = (error && error.length > 0) ? error : qsTr("prefix setup failed")
            }
        }
    }

    body: Item {
        width: parent.width
        height: parent.height

        Text {
            id: prepHeader
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            text: root.errorText !== ""
                ? root.errorText
                : qsTr("First launch for this game, setting up the wine prefix. It'll start once this finishes.")
            color: root.errorText !== "" ? theme.error : theme.textMuted
            font.pixelSize: 12
            wrapMode: Text.WordWrap
        }

        OutputLog {
            anchors.top: prepHeader.bottom
            anchors.topMargin: theme.space.sm
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            text: root.outputText
        }
    }

    actions: M3Button {
        text: root.busy ? qsTr("Cancel") : qsTr("Close")
        variant: "tonal"
        onClicked: { root.cancelled = true; root.close() }
    }
}
