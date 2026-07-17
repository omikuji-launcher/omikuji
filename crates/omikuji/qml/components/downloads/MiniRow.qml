import QtQuick
import QtQuick.Layouts
import "../controls"
import "../primitives"
import "../lib/Format.js" as Format

Item {
    id: row

    required property string id
    required property string source
    required property string displayName
    required property string banner
    required property string status
    required property string kind
    required property real bytesDownloaded
    required property real bytesTotal
    required property string error

    property var downloadModel: null

    signal cancelRequested(string id, string displayName)

    implicitHeight: 58

    readonly property bool isPaused: status === "Paused"
    readonly property bool isFailed: status === "Failed"
    readonly property bool isDone: status === "Completed" || status === "Cancelled"

    Squircle {
        anchors.fill: parent
        radius: theme.radius.lg
        smoothing: 0.75
        fillColor: row.isDone ? theme.alpha(theme.text, 0.03) : theme.cardBg
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: theme.space.sm
        anchors.rightMargin: theme.space.sm
        spacing: theme.space.md

        BannerThumb {
            Layout.preferredWidth: 76
            Layout.preferredHeight: 42
            source: row.banner
            cornerRadius: theme.radius.sm
            fallbackFrom: row.displayName
            fallbackTextSize: 18
        }

        Text {
            text: row.displayName
            color: theme.text
            font.pixelSize: theme.type.body.size
            font.weight: Font.Medium
            elide: Text.ElideRight
            Layout.maximumWidth: parent.width * 0.45
        }

        KindChip {
            visible: !row.isDone
            kind: row.kind
        }

        Text {
            Layout.fillWidth: true
            text: row.metaLine()
            color: row.isFailed ? theme.error : theme.textMuted
            font.pixelSize: theme.type.caption.size
            elide: Text.ElideRight
        }

        IconButton {
            icon: "play_arrow"
            size: 32
            visible: row.isPaused
            onClicked: if (row.downloadModel) row.downloadModel.resume(row.id)
        }

        IconButton {
            icon: "sync"
            size: 32
            visible: row.isFailed
            onClicked: if (row.downloadModel) row.downloadModel.retry(row.id)
        }

        IconButton {
            icon: "close"
            size: 32
            danger: !row.isDone && !row.isFailed
            onClicked: {
                if (!row.downloadModel) return
                if (row.isDone || row.isFailed) row.downloadModel.dismiss(row.id)
                else row.cancelRequested(row.id, row.displayName)
            }
        }
    }

    function metaLine() {
        if (isFailed) return error || qsTr("Failed")
        if (isPaused) {
            let bytes = bytesTotal > 0
                ? Format.formatBytes(bytesDownloaded) + " / " + Format.formatBytes(bytesTotal)
                : Format.formatBytes(bytesDownloaded)
            return qsTr("Paused") + "  ·  " + bytes
        }
        if (isDone) {
            let size = bytesTotal > 0 ? bytesTotal : bytesDownloaded
            return (size > 0 ? Format.formatBytes(size) + "  ·  " : "") + qsTr("Completed")
        }
        return bytesTotal > 0
            ? qsTr("Queued") + "  ·  " + Format.formatBytes(bytesTotal)
            : qsTr("Queued")
    }
}
