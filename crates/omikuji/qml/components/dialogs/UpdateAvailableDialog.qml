pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Layouts
import "../controls"
import "../lib/Format.js" as Format

DialogCard {
    sizeKey: "update_available"
    id: root

    property string gameId: ""
    property string appId: ""
    property string displayName: ""
    property string fromVersion: ""
    property string toVersion: ""
    property real downloadBytes: 0
    property bool canDiff: true
    property bool deltaSupported: true

    signal updateRequested(string gameId, string appId, string fromVersion)
    signal runAnywayRequested(string gameId)
    signal dismissed(string gameId)

    maxWidth: 460

    function show(payload) {
        if (payload) {
            gameId = payload.gameId || ""
            appId = payload.appId || ""
            displayName = payload.displayName || ""
            fromVersion = payload.fromVersion || ""
            toVersion = payload.toVersion || ""
            downloadBytes = payload.downloadBytes || 0
            canDiff = payload.canDiff === undefined ? true : payload.canDiff
            deltaSupported = payload.deltaSupported === undefined ? true : payload.deltaSupported
        }
        open()
    }
    function hide() { close() }

    onCloseRequested: { root.dismissed(root.gameId); root.close() }

    body: ColumnLayout {
        width: parent.width
        spacing: Theme.space.lg

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.space.sm

            Rectangle {
                width: 36; height: 36; radius: 18
                color: Theme.alpha(Theme.accent, 0.15)
                Text {
                    anchors.centerIn: parent
                    text: "↻"
                    color: Theme.accent
                    font.pixelSize: 20
                    font.weight: Font.Bold
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2
                Text {
                    Layout.fillWidth: true
                    text: qsTr("Update available")
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
            implicitHeight: versionCol.implicitHeight + Theme.space.lg

            ColumnLayout {
                id: versionCol
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.top: parent.top
                anchors.margins: Theme.space.md
                spacing: Theme.space.sm

                RowLayout {
                    Layout.fillWidth: true
                    spacing: Theme.space.sm
                    Text { text: root.fromVersion || "?"; color: Theme.textMuted; font.pixelSize: Theme.type.body.size; font.family: "monospace" }
                    Text { text: "→"; color: Theme.textMuted; font.pixelSize: Theme.type.body.size }
                    Text { text: root.toVersion || "?"; color: Theme.accent; font.pixelSize: Theme.type.body.size; font.family: "monospace"; font.weight: Font.DemiBold }
                    Item { Layout.fillWidth: true }
                }

                Text {
                    Layout.fillWidth: true
                    text: {
                        if (root.canDiff) {
                            return root.downloadBytes > 0
                                ? qsTr("Delta update · %1").arg(Format.formatBytes(root.downloadBytes))
                                : qsTr("Delta update")
                        }
                        let name = root.displayName || qsTr("the game")
                        if (!root.deltaSupported) {
                            return qsTr("Seems there's an update for %1, however, it doesn't handle delta patches. Wanna reinstall the game to update?").arg(name)
                        }
                        return qsTr("Your install is too old for a delta patch. Reinstall %1 to update?").arg(name)
                    }
                    color: Theme.textMuted
                    font.pixelSize: Theme.type.caption.size
                    wrapMode: Text.Wrap
                }
            }
        }
    }

    actions: Row {
        spacing: Theme.space.sm

        M3Button {
            text: qsTr("Cancel")
            variant: "text"
            onClicked: { root.dismissed(root.gameId); root.close() }
        }
        M3Button {
            text: qsTr("Run anyway")
            variant: "tonal"
            onClicked: { root.runAnywayRequested(root.gameId); root.close() }
        }
        M3Button {
            text: root.canDiff ? qsTr("Update") : qsTr("Reinstall")
            variant: "filled"
            onClicked: {
                root.updateRequested(root.gameId, root.appId, root.fromVersion)
                root.close()
            }
        }
    }
}
