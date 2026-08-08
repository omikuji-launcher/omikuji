pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Layouts
import "../controls"

DialogCard {
    id: root

    property string gameId: ""
    property string displayName: ""
    property string headTitle: qsTr("Couldn't launch")
    property string message: ""
    property string action: ""

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

    function renderMessage(raw) {
        if (!raw) return ""
        let accent = Theme.accent
        let hex = Qt.colorEqual(accent, "transparent")
            ? "#888"
            : "#" + Math.round(accent.r * 255).toString(16).padStart(2, "0")
                  + Math.round(accent.g * 255).toString(16).padStart(2, "0")
                  + Math.round(accent.b * 255).toString(16).padStart(2, "0")
        let escaped = String(raw)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
        return escaped.replace(/`([^`]+)`/g, function(_m, p1) {
            return '<span style="color:' + hex + '; font-family:monospace">' + p1 + '</span>'
        })
    }

    onCloseRequested: { root.dismissed(); root.close() }

    body: ColumnLayout {
        width: parent.width
        spacing: Theme.space.lg

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.space.sm

            Rectangle {
                width: 36; height: 36; radius: 18
                color: Theme.alpha(Theme.error, 0.18)
                Text {
                    anchors.fill: parent
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                    text: "!"
                    color: Theme.error
                    font.pixelSize: 20
                    font.weight: Font.Bold
                }
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
                text: root.renderMessage(root.message)
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
            text: qsTr("Open Settings")
            variant: "filled"
            visible: root.action.length > 0
            onClicked: { root.actionRequested(root.action, root.gameId); root.close() }
        }
    }
}
