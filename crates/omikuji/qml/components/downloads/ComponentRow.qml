import QtQuick
import QtQuick.Layouts
import "../controls"
import "../primitives"


Rectangle {
    id: row

    property string name: ""
    property var entry: ({})

    signal retryRequested()

    radius: theme.radius.md
    color: theme.alpha(theme.text, 0.05)
    implicitHeight: 56

    readonly property string status: entry.status || "missing"
    readonly property real percent: entry.percent || 0
    readonly property string version: entry.version || ""
    readonly property string error: entry.error || ""
    readonly property bool isDone: status === "completed"
    readonly property bool isFailed: status === "failed"
    readonly property bool isActive: status === "installing" || status === "downloading"
        || status === "extracting" || status === "resolving"

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 14
        anchors.rightMargin: 14
        spacing: 12

        SvgIcon {
            Layout.preferredWidth: 20
            Layout.preferredHeight: 20
            size: 20
            name: row.isDone ? "check_circle"
                : row.isFailed ? "close"
                : "download"
            color: row.isDone ? theme.accent
                : row.isFailed ? (theme.error || "#e06060")
                : row.isActive ? theme.accent
                : theme.textMuted
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 3

            Text {
                text: row.name
                color: theme.text
                font.pixelSize: theme.type.body.size
                font.weight: Font.Medium
            }

            Text {
                Layout.fillWidth: true
                text: row.isFailed ? qsTr("Failed: %1").arg(row.error)
                    : row.isDone ? (row.version ? ("v" + row.version) : qsTr("Installed"))
                    : row.isActive ? (capitalize(row.status)
                          + (row.status === "downloading" ? " · " + Math.round(row.percent) + "%" : "…"))
                    : qsTr("Pending")
                color: row.isFailed ? (theme.error || "#e06060") : theme.textMuted
                font.pixelSize: theme.type.caption.size
                elide: Text.ElideRight
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 2
                radius: 1
                color: theme.alpha(theme.text, 0.12)
                visible: row.isActive
                Rectangle {
                    anchors.left: parent.left
                    anchors.top: parent.top
                    anchors.bottom: parent.bottom
                    width: parent.width * Math.max(0, Math.min(1, row.percent / 100))
                    radius: parent.radius
                    color: theme.accent
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

    function capitalize(s) {
        if (!s || s.length === 0) return ""
        return s.charAt(0).toUpperCase() + s.slice(1)
    }
}
