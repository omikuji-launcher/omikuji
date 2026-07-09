import QtQuick
import QtQuick.Layouts
import Qt5Compat.GraphicalEffects
import "../controls"
import "../popups"
import "../primitives"


Item {
    id: row

    required property string id
    required property string source
    required property string appId
    required property string displayName
    required property string banner
    required property string status
    required property real progress
    required property real speed
    required property real bytesDownloaded
    required property real bytesTotal
    required property string error

    property var downloadModel: null

    // forwarded from DownloadsPage, false when the view is hidden
    property bool pageVisible: true

    signal cancelRequested(string id, string displayName)

    implicitHeight: 112

    readonly property bool isActive: status === "Queued" || status === "Starting" || status === "Downloading" || status === "Extracting" || status === "Patching"
    readonly property bool isPaused: status === "Paused"
    readonly property bool isFinished: status === "Completed" || status === "Failed" || status === "Cancelled"
    // 7z and hpatchz cant be stopped mid-operation without corrupting the output
    readonly property bool isUninterruptible: status === "Extracting" || status === "Patching"

    Rectangle {
        id: card
        anchors.fill: parent
        anchors.margins: 4
        radius: 14
        color: theme.cardBg

        layer.enabled: true
        layer.effect: DropShadow {
            transparentBorder: true
            horizontalOffset: 0
            verticalOffset: 2
            radius: theme.radius.md
            samples: 25
            color: Qt.rgba(0, 0, 0, 0.28)
        }

        RowLayout {
            anchors.fill: parent
            anchors.margins: 14
            spacing: 16

            Rectangle {
                Layout.preferredWidth: 184
                Layout.preferredHeight: 84
                radius: 10
                color: theme.alpha(theme.text, 0.05)
                clip: true

                Image {
                    id: bannerImg
                    anchors.fill: parent
                    source: row.banner || ""
                    fillMode: Image.PreserveAspectCrop
                    asynchronous: true
                    visible: status === Image.Ready
                    layer.enabled: true
                    layer.effect: OpacityMask {
                        maskSource: Rectangle {
                            width: bannerImg.width
                            height: bannerImg.height
                            radius: 10
                        }
                    }
                }

                Text {
                    anchors.centerIn: parent
                    text: row.displayName ? row.displayName.charAt(0) : "?"
                    color: theme.textFaint
                    font.pixelSize: 32
                    font.weight: Font.Bold
                    visible: !bannerImg.visible
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignVCenter
                spacing: 6

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 8

                    SvgIcon {
                        name: row.source === "epic" ? "shield_moon"
                            : row.source === "gog" ? "gog"
                            : "download"
                        size: 18
                        color: theme.textMuted
                    }

                    Text {
                        Layout.fillWidth: true
                        text: row.displayName
                        color: theme.text
                        font.pixelSize: 18
                        font.weight: Font.DemiBold
                        elide: Text.ElideRight
                    }
                }

                Text {
                    Layout.fillWidth: true
                    text: row.statusLine()
                    color: status === "Failed" ? theme.danger || "#e06060" : theme.textMuted
                    font.pixelSize: 13
                    elide: Text.ElideRight
                }

                WavyProgressBar {
                    Layout.fillWidth: true
                    Layout.preferredHeight: implicitHeight
                    Layout.topMargin: 4
                    visible: !row.isFinished || row.status === "Completed"
                    value: Math.max(0, Math.min(1, row.progress / 100.0))
                    wavy: row.isActive
                    animate: row.isActive && row.pageVisible
                    fillColor: row.status === "Failed" ? "#e06060"
                        : row.isPaused ? theme.alpha(theme.text, 0.3)
                        : theme.accent
                    handleColor: fillColor
                    trackColor: theme.alpha(theme.text, 0.18)
                }
            }

            RowLayout {
                Layout.alignment: Qt.AlignVCenter
                spacing: 4

                IconButton {
                    id: pauseBtn
                    icon: row.isPaused ? "play" : "pause"
                    size: 36
                    visible: row.isActive || row.isPaused
                    blocked: row.isUninterruptible
                    onClicked: {
                        if (!row.downloadModel) return
                        if (row.isPaused) row.downloadModel.resume(row.id)
                        else row.downloadModel.pause(row.id)
                    }

                    Tooltip {
                        text: qsTr("don't you dare.")
                        tipVisible: pauseBtn.hovered && row.isUninterruptible
                    }
                }

                IconButton {
                    icon: "sync"
                    size: 36
                    visible: row.status === "Failed"
                    onClicked: {
                        if (!row.downloadModel) return
                        row.downloadModel.retry(row.id)
                    }
                }

                IconButton {
                    icon: "close"
                    size: 36
                    danger: !row.isFinished
                    onClicked: {
                        if (!row.downloadModel) return
                        // finished entries dismiss witout a prompt, files are kept either way
                        if (row.isFinished) {
                            row.downloadModel.dismiss(row.id)
                        } else {
                            row.cancelRequested(row.id, row.displayName)
                        }
                    }
                }
            }
        }
    }

    function statusLine() {
        let parts = []

        if (status === "Failed" && error) {
            return qsTr("Failed: %1").arg(error)
        }

        if (status !== "Queued" && bytesTotal > 0) {
            parts.push(formatBytes(bytesDownloaded) + " / " + formatBytes(bytesTotal))
        } else if (bytesDownloaded > 0) {
            parts.push(formatBytes(bytesDownloaded))
        }

        if (status === "Extracting") {
            return qsTr("Extracting · %1%").arg(progress.toFixed(0))
        }

        if (status === "Patching") {
            return qsTr("Patching · %1%").arg(progress.toFixed(0))
        }

        if (status === "Downloading" && speed > 0) {
            parts.push(formatBytes(speed) + "/s")
            if (bytesTotal > bytesDownloaded) {
                let etaSecs = (bytesTotal - bytesDownloaded) / speed
                parts.push(qsTr("%1 left").arg(formatEta(etaSecs)))
            }
        }

        let head = status === "Downloading" && parts.length > 0 ? "" : status + (parts.length > 0 ? " · " : "")
        return head + parts.join(" · ")
    }

    function formatBytes(b) {
        if (b >= 1024 * 1024 * 1024) return (b / (1024 * 1024 * 1024)).toFixed(2) + " GB"
        if (b >= 1024 * 1024) return (b / (1024 * 1024)).toFixed(1) + " MB"
        if (b >= 1024) return (b / 1024).toFixed(1) + " KB"
        return Math.round(b) + " B"
    }

    function formatEta(secs) {
        if (!isFinite(secs) || secs <= 0) return "?"
        if (secs >= 3600) return Math.floor(secs / 3600) + "h " + Math.floor((secs % 3600) / 60) + "m"
        if (secs >= 60) return Math.floor(secs / 60) + "m " + Math.floor(secs % 60) + "s"
        return Math.floor(secs) + "s"
    }
}
