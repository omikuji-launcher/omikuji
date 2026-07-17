import QtQuick
import QtQuick.Layouts
import "../controls"
import "../popups"
import "../primitives"
import "../lib/Format.js" as Format

Item {
    id: hero

    required property string id
    required property string source
    required property string displayName
    required property string banner
    required property string status
    required property string kind
    required property real progress
    required property real speed
    required property real bytesDownloaded
    required property real bytesTotal

    property var downloadModel: null
    property bool pageVisible: true

    signal cancelRequested(string id, string displayName)

    readonly property real designWidth: 1000
    readonly property real shrink: width > 0 ? Math.min(1, width / designWidth) : 1
    implicitHeight: Math.round(196 * shrink)

    readonly property bool isUninterruptible: status === "Extracting" || status === "Patching"
    readonly property bool isPaused: status === "Paused"

    property var netSamples: []
    property var diskSamples: []

    readonly property real netBps: netSamples.length > 0 ? netSamples[netSamples.length - 1] : speed
    readonly property real diskBps: diskSamples.length > 0 ? diskSamples[diskSamples.length - 1] : 0

    Timer {
        interval: 1000
        repeat: true
        triggeredOnStart: true
        running: hero.visible && hero.pageVisible && hero.downloadModel !== null
        onTriggered: {
            let h = JSON.parse(hero.downloadModel.speedHistoryJson())
            hero.netSamples = h.net
            hero.diskSamples = h.disk
        }
    }

    component StatCell: ColumnLayout {
        property string label
        property string value
        property real minWidth: 0
        Layout.minimumWidth: minWidth
        spacing: 2
        CapsLabel { text: parent.label }
        Text {
            text: parent.value
            color: theme.text
            font.pixelSize: theme.type.subtitle.size
            font.weight: Font.DemiBold
        }
    }

    Squircle {
        anchors.fill: parent
        radius: theme.radius.xl
        smoothing: 0.75
        fillColor: theme.cardBg
    }

    Item {
        width: Math.max(hero.width, hero.designWidth)
        height: 196
        scale: hero.shrink
        transformOrigin: Item.TopLeft

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: theme.space.lg
            anchors.bottomMargin: theme.space.md
            spacing: theme.space.md

            RowLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: theme.space.lg

                BannerThumb {
                    Layout.preferredWidth: 218
                    Layout.fillHeight: true
                    source: hero.banner
                    cornerRadius: theme.radius.md
                    fallbackFrom: hero.displayName
                    fallbackTextSize: 34
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    spacing: 4

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: theme.space.sm

                        Text {
                            text: hero.displayName
                            color: theme.text
                            font.pixelSize: theme.type.headline.size
                            font.weight: Font.DemiBold
                            elide: Text.ElideRight
                            Layout.maximumWidth: parent.width * 0.7
                        }

                        KindChip { kind: hero.kind }

                        Item { Layout.fillWidth: true }

                        IconButton {
                            id: pauseBtn
                            icon: hero.isPaused ? "play_arrow" : "pause"
                            size: 36
                            blocked: hero.isUninterruptible
                            onClicked: {
                                if (!hero.downloadModel) return
                                if (hero.isPaused) hero.downloadModel.resume(hero.id)
                                else hero.downloadModel.pause(hero.id)
                            }

                            Tooltip {
                                text: qsTr("don't you dare.")
                                tipVisible: pauseBtn.hovered && hero.isUninterruptible
                            }
                        }

                        IconButton {
                            icon: "close"
                            size: 36
                            danger: true
                            onClicked: hero.cancelRequested(hero.id, hero.displayName)
                        }
                    }

                    Text {
                        text: hero.status
                        color: hero.isUninterruptible ? theme.warning : theme.textMuted
                        font.pixelSize: theme.type.label.size
                    }

                    Item { Layout.fillHeight: true }

                    RowLayout {
                        spacing: theme.space.xxl

                        StatCell {
                            label: qsTr("Progress")
                            minWidth: 64
                            value: Math.round(hero.progress) + "%"
                        }
                        StatCell {
                            label: qsTr("ETA")
                            minWidth: 96
                            value: !hero.isUninterruptible && !hero.isPaused && hero.speed > 0 && hero.bytesTotal > hero.bytesDownloaded
                                ? Format.formatEta((hero.bytesTotal - hero.bytesDownloaded) / hero.speed)
                                : "—"
                        }
                        StatCell {
                            label: qsTr("Downloaded")
                            minWidth: 190
                            value: hero.bytesTotal > 0
                                ? Format.formatBytes(hero.bytesDownloaded) + " / " + Format.formatBytes(hero.bytesTotal)
                                : (hero.bytesDownloaded > 0 ? Format.formatBytes(hero.bytesDownloaded) : "—")
                        }
                    }
                }

                Item {
                    Layout.preferredWidth: 248
                    Layout.fillHeight: true

                    Squircle {
                        anchors.fill: parent
                        radius: theme.radius.md
                        smoothing: 0.75
                        fillColor: theme.alpha(theme.text, 0.04)
                    }

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: theme.space.md
                        spacing: 4

                        RowLayout {
                            Layout.fillWidth: true

                            ColumnLayout {
                                spacing: 0
                                Text {
                                    text: hero.isPaused ? "—" : Format.formatSpeed(hero.netBps)
                                    color: theme.text
                                    font.pixelSize: theme.type.subtitle.size
                                    font.weight: Font.DemiBold
                                }
                                CapsLabel { text: qsTr("Net"); size: 9 }
                            }

                            Item { Layout.fillWidth: true }

                            ColumnLayout {
                                spacing: 0
                                Text {
                                    Layout.alignment: Qt.AlignRight
                                    text: hero.isPaused ? "—" : Format.formatSpeed(hero.diskBps)
                                    color: theme.textMuted
                                    font.pixelSize: theme.type.subtitle.size
                                    font.weight: Font.DemiBold
                                }
                                CapsLabel {
                                    Layout.alignment: Qt.AlignRight
                                    text: qsTr("Disk")
                                    size: 9
                                }
                            }
                        }

                        Sparkline {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            samples: hero.netSamples
                            samples2: hero.diskSamples
                        }
                    }
                }
            }

            WavyProgressBar {
                Layout.fillWidth: true
                Layout.topMargin: theme.space.xs
                value: Math.max(0, Math.min(1, hero.progress / 100.0))
                wavy: !hero.isPaused
                animate: hero.pageVisible && !hero.isPaused
                fillColor: hero.isPaused ? theme.alpha(theme.text, 0.3) : theme.accent
                handleColor: fillColor
                trackColor: theme.alpha(theme.text, 0.18)
            }
        }
    }
}
