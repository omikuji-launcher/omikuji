pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Layouts
import "../lib/Format.js" as Format

DialogCard {
    id: root

    property string gameId: ""
    property string displayName: ""
    property string headTitle: qsTr("Couldn't launch")
    property string message: ""
    property string action: ""

    readonly property var actionLabels: ({
        open_game_settings: qsTr("Open Settings"),
        open_global_settings: qsTr("Open Settings"),
        open_epic_store: qsTr("Sign in")
    })

    signal actionRequested(string action, string gameId)
    signal dismissed()

    maxWidth: 460

    function show(payload) {
        if (payload) {
            gameId = payload.gameId || ""
            displayName = payload.displayName || ""
            headTitle = payload.title || "Couldn't launch"
            message = payload.message || ""
            action = payload.action || ""
        }
        open()
    }
    function hide() { close() }

    onCloseRequested: { root.dismissed(); root.close() }

    body: ColumnLayout {
        width: parent.width
        spacing: Theme.space.lg

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.space.sm

            MoodySpirit {
                id: moody

                Layout.preferredWidth: moody.implicitWidth
                Layout.preferredHeight: moody.implicitHeight
                Layout.alignment: Qt.AlignVCenter
                animated: root.shown
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2
                Text {
                    Layout.fillWidth: true
                    text: root.headTitle
                    color: Theme.text
                    font.pixelSize: Theme.type.title.size
                    font.weight: Font.DemiBold
                    wrapMode: Text.Wrap
                }
                Text {
                    Layout.fillWidth: true
                    text: root.displayName
                    color: Theme.textMuted
                    font.pixelSize: Theme.type.caption.size
                    wrapMode: Text.Wrap
                    elide: Text.ElideRight
                }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            radius: Theme.radius.md
            color: Theme.alpha(Theme.text, 0.04)
            implicitHeight: messageText.implicitHeight + Theme.space.lg

            Text {
                id: messageText
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.top: parent.top
                anchors.margins: Theme.space.md
                text: Format.backticksToRichText(root.message, Theme.accent, Theme.mono)
                textFormat: Text.RichText
                color: Theme.text
                font.pixelSize: Theme.type.label.size
                wrapMode: Text.Wrap
            }
        }
    }

    actions: Row {
        spacing: Theme.space.sm

        M3Button {
            text: qsTr("Cancel")
            variant: "text"
            onClicked: { root.dismissed(); root.close() }
        }
        M3Button {
            text: root.actionLabels[root.action] || qsTr("Open Settings")
            variant: "filled"
            visible: root.action.length > 0
            onClicked: { root.actionRequested(root.action, root.gameId); root.close() }
        }
    }
}
