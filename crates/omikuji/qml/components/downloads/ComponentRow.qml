pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Layouts
import "../controls"
import "../primitives"


Rectangle {
    id: row

    property string name: ""
    property var entry: ({})

    signal retryRequested()

    radius: Theme.radius.md
    color: Theme.alpha(Theme.text, 0.05)
    implicitHeight: 56 + (errorNote.visible ? errorNote.height + Theme.space.sm : 0)

    readonly property string status: entry.status || "missing"
    readonly property real percent: entry.percent || 0
    readonly property string version: entry.version || ""
    readonly property string path: entry.path || ""
    readonly property string error: entry.error || ""
    readonly property bool isSystem: status === "system"
    readonly property bool isDone: status === "completed"
    readonly property bool isFailed: status === "failed"
    readonly property bool isActive: status === "installing" || status === "downloading"
        || status === "extracting" || status === "resolving"

    RowLayout {
        id: mainRow
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: 14
        anchors.rightMargin: 14
        height: 56
        spacing: 12

        SvgIcon {
            Layout.preferredWidth: 20
            Layout.preferredHeight: 20
            size: 20
            name: row.isDone || row.isSystem ? "check_circle"
                : row.isFailed ? "close"
                : "download"
            color: row.isDone ? Theme.accent
                : row.isFailed ? (Theme.error || "#e06060")
                : row.isActive ? Theme.accent
                : Theme.textMuted
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 3

            Text {
                text: row.name
                color: Theme.text
                font.pixelSize: Theme.type.body.size
                font.weight: Font.Medium
            }

            Text {
                Layout.fillWidth: true
                text: row.isFailed ? qsTr("Failed")
                    : row.isSystem ? (row.path || qsTr("Provided by your system"))
                    : row.isDone ? (row.version ? ("v" + row.version) : qsTr("Installed"))
                    : row.isActive ? (row.capitalize(row.status)
                          + (row.status === "downloading" ? " · " + Math.round(row.percent) + "%" : "…"))
                    : qsTr("Pending")
                color: row.isFailed ? Theme.error : Theme.textMuted
                font.pixelSize: Theme.type.caption.size
                elide: Text.ElideRight
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 2
                radius: 1
                color: Theme.alpha(Theme.text, 0.12)
                visible: row.isActive
                Rectangle {
                    anchors.left: parent.left
                    anchors.top: parent.top
                    anchors.bottom: parent.bottom
                    width: parent.width * Math.max(0, Math.min(1, row.percent / 100))
                    radius: parent.radius
                    color: Theme.accent
                    Behavior on width { NumberAnimation { duration: 150; easing.type: Easing.OutCubic } }
                }
            }
        }

        IconButton {
            visible: row.isFailed
            size: 28
            icon: "sync"
            onClicked: row.retryRequested()
        }
    }

    NoteChip {
        id: errorNote
        anchors.top: mainRow.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: 14
        anchors.rightMargin: 14
        visible: row.isFailed && row.error !== ""
        text: row.error
        icon: "error"
        tone: Theme.error
    }

    function capitalize(s) {
        if (!s || s.length === 0) return ""
        return s.charAt(0).toUpperCase() + s.slice(1)
    }
}
