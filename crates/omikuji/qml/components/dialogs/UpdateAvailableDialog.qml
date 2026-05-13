import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Qt5Compat.GraphicalEffects

import "../widgets"

Item {
    id: root

    property string gameId: ""
    property string appId: ""
    property string displayName: ""
    property string fromVersion: ""
    property string toVersion: ""
    property real downloadBytes: 0
    property bool canDiff: true
    property bool deltaSupported: true

    visible: false
    z: 2000

    signal updateRequested(string gameId, string appId, string fromVersion)
    signal runAnywayRequested(string gameId)
    signal dismissed(string gameId)

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
        visible = true
        forceActiveFocus()
    }

    function hide() {
        visible = false
    }

    function formatBytes(b) {
        if (b >= 1024 * 1024 * 1024) return (b / (1024 * 1024 * 1024)).toFixed(2) + " GB"
        if (b >= 1024 * 1024) return (b / (1024 * 1024)).toFixed(1) + " MB"
        if (b >= 1024) return (b / 1024).toFixed(1) + " KB"
        return b + " B"
    }

    Rectangle {
        anchors.fill: parent
        color: Qt.rgba(0, 0, 0, 0.55)
        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            acceptedButtons: Qt.AllButtons
            onClicked: (mouse) => { if (mouse.button === Qt.LeftButton) { root.dismissed(root.gameId); root.hide() } }
            onWheel: (wheel) => wheel.accepted = true
            cursorShape: Qt.ArrowCursor
        }
    }

    Rectangle {
        id: card
        anchors.centerIn: parent
        width: Math.min(parent.width - 80, 460)
        height: Math.min(parent.height - 60, content.implicitHeight + 48)
        radius: 22
        color: theme.surface
        border.width: 1
        border.color: Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.08)

        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.AllButtons
            onClicked: {}
            onWheel: (wheel) => wheel.accepted = true
        }

        layer.enabled: true
        layer.effect: DropShadow {
            radius: 24
            samples: 32
            color: Qt.rgba(0, 0, 0, 0.4)
            horizontalOffset: 0
            verticalOffset: 6
        }

        Flickable {
            id: cardScroll
            anchors.fill: parent
            anchors.margins: 24
            contentWidth: width
            contentHeight: content.implicitHeight
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            interactive: contentHeight > height
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }

        ColumnLayout {
            id: content
            width: cardScroll.width
            spacing: 16

            RowLayout {
                Layout.fillWidth: true
                spacing: 10

                Rectangle {
                    width: 36
                    height: 36
                    radius: 18
                    color: Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.15)
                    Text {
                        anchors.centerIn: parent
                        text: "↻"
                        color: theme.accent
                        font.pixelSize: 20
                        font.weight: Font.Bold
                    }
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 2

                    Text {
                        Layout.fillWidth: true
                        text: "Update available"
                        color: theme.text
                        font.pixelSize: 17
                        font.weight: Font.DemiBold
                        wrapMode: Text.Wrap
                    }
                    Text {
                        Layout.fillWidth: true
                        text: root.displayName
                        color: theme.textMuted
                        font.pixelSize: 12
                        wrapMode: Text.Wrap
                        elide: Text.ElideRight
                    }
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.topMargin: 2
                radius: 12
                color: Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.04)
                implicitHeight: versionCol.implicitHeight + 24

                ColumnLayout {
                    id: versionCol
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.margins: 12
                    spacing: 8

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 8

                        Text {
                            text: root.fromVersion || "?"
                            color: theme.textMuted
                            font.pixelSize: 14
                            font.family: "monospace"
                        }
                        Text {
                            text: "→"
                            color: theme.textMuted
                            font.pixelSize: 14
                        }
                        Text {
                            text: root.toVersion || "?"
                            color: theme.accent
                            font.pixelSize: 14
                            font.family: "monospace"
                            font.weight: Font.DemiBold
                        }
                        Item { Layout.fillWidth: true }
                    }

                    Text {
                        Layout.fillWidth: true
                        text: {
                            if (root.canDiff) {
                                return root.downloadBytes > 0
                                    ? ("Delta update · " + root.formatBytes(root.downloadBytes))
                                    : "Delta update"
                            }
                            let name = root.displayName || "the game"
                            if (!root.deltaSupported) {
                                return "Seems there's an update for " + name + ", however, it doesn't handle delta patches. Wanna reinstall the game to update?"
                            }
                            return "Your install is too old for a delta patch. Reinstall " + name + " to update?"
                        }
                        color: theme.textMuted
                        font.pixelSize: 12
                        wrapMode: Text.Wrap
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true
                Layout.topMargin: 4
                spacing: 10

                Item { Layout.fillWidth: true }

                Item {
                    implicitWidth: Math.max(80, laterLabel.implicitWidth + 28)
                    implicitHeight: 36

                    Rectangle {
                        anchors.fill: parent
                        radius: 18
                        color: laterHover.containsPress
                            ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.12)
                            : laterHover.containsMouse
                                ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.06)
                                : "transparent"
                        Behavior on color { ColorAnimation { duration: 100 } }
                    }
                    Text {
                        id: laterLabel
                        anchors.centerIn: parent
                        text: "Cancel"
                        color: theme.text
                        font.pixelSize: 13
                        font.weight: Font.Medium
                    }
                    MouseArea {
                        id: laterHover
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: { root.dismissed(root.gameId); root.hide() }
                    }
                }

                Item {
                    implicitWidth: Math.max(110, runAnywayLabel.implicitWidth + 28)
                    implicitHeight: 36

                    Rectangle {
                        anchors.fill: parent
                        radius: 18
                        color: runAnywayHover.containsPress
                            ? Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.18)
                            : runAnywayHover.containsMouse
                                ? Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.10)
                                : "transparent"
                        border.width: 1
                        border.color: Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.45)
                        Behavior on color { ColorAnimation { duration: 100 } }
                    }
                    Text {
                        id: runAnywayLabel
                        anchors.centerIn: parent
                        text: "Run anyway"
                        color: theme.accent
                        font.pixelSize: 13
                        font.weight: Font.Medium
                    }
                    MouseArea {
                        id: runAnywayHover
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: { root.runAnywayRequested(root.gameId); root.hide() }
                    }
                }

                // disabled on full reinstall, we dont auto-chain(chan) into a reinstall yet
                Item {
                    implicitWidth: Math.max(96, updateLabel.implicitWidth + 28)
                    implicitHeight: 36

                    Rectangle {
                        anchors.fill: parent
                        radius: 18
                        color: theme.accent
                        opacity: updateHover.containsPress ? 0.8
                            : updateHover.containsMouse ? 0.95 : 0.9
                        scale: updateHover.containsPress ? 0.97 : 1.0
                        Behavior on opacity { NumberAnimation { duration: 100 } }
                        Behavior on scale { NumberAnimation { duration: 100 } }
                    }
                    Text {
                        id: updateLabel
                        anchors.centerIn: parent
                        text: root.canDiff ? "Update" : "Reinstall"
                        color: theme.accentOn
                        font.pixelSize: 13
                        font.weight: Font.DemiBold
                    }
                    MouseArea {
                        id: updateHover
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            root.updateRequested(root.gameId, root.appId, root.fromVersion)
                            root.hide()
                        }
                    }
                }
            }
        }
        }
    }
}
