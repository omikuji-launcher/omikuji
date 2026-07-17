import QtQuick
import QtQuick.Controls
import "../controls"


DialogCard {
    id: root

    property var bridge: null
    property bool done: false
    property string errorText: ""
    property string outputText: ""

    readonly property bool busy: bridge ? bridge.running : false
    readonly property bool expanded: busy || done || outputText !== "" || errorText !== ""

    maxWidth: 640
    fillHeight: expanded
    preferredHeight: 460
    scrollable: false
    escEnabled: false
    title: qsTr("Data folder update")

    function start() {
        done = false
        errorText = ""
        outputText = ""
        open()
    }

    Connections {
        target: bridge
        enabled: bridge !== null
        function onOutput(line) {
            root.outputText += (root.outputText.length ? "\n" : "") + line
        }
        function onFinished(ok, error) {
            if (ok) {
                root.done = true
            } else {
                root.errorText = (error && error.length > 0) ? error : qsTr("migration failed")
            }
        }
    }

    body: Item {
        width: parent.width
        height: parent.height
        implicitHeight: migrationHeader.implicitHeight

        Text {
            id: migrationHeader
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            text: root.errorText !== ""
                ? root.errorText
                : root.done
                    ? qsTr("Everything moved. Omikuji needs a restart to pick up the new layout.")
                    : root.busy
                        ? qsTr("Moving things around, hold on...")
                        : qsTr("This version reorganizes the data folder: runners and graphics layers now live under components/, GOG data moves into runtime/, and sources move to components.toml. Folders you relocated in settings stay where they are.")
            color: root.errorText !== "" ? theme.error : theme.textMuted
            font.pixelSize: theme.type.caption.size
            wrapMode: Text.WordWrap
        }

        OutputLog {
            visible: root.expanded
            anchors.top: migrationHeader.bottom
            anchors.topMargin: theme.space.sm
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            text: root.outputText
        }
    }

    actions: Row {
        spacing: theme.space.sm

        M3Button {
            visible: !root.busy && !root.done
            text: qsTr("Quit")
            variant: "tonal"
            onClicked: Qt.quit()
        }

        M3Button {
            visible: !root.busy && !root.done && root.errorText === ""
            text: qsTr("Move and continue")
            variant: "filled"
            onClicked: if (root.bridge) root.bridge.run()
        }

        M3Button {
            visible: root.done
            text: qsTr("Restart Omikuji")
            variant: "filled"
            onClicked: if (root.bridge) root.bridge.restartApp()
        }
    }
}
