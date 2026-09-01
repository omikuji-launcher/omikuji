import QtQuick
import omikuji 1.0

import "."
import "../controls"

Item {
    id: root

    property var appSettings: null

    readonly property int rowLabelWidth: 200

    implicitHeight: content.height

    component SpinRow: SettingsRow {
        id: spinRow

        property real from: 0
        property real to: 128
        property real stepSize: 1
        property real value: 0
        property int decimals: 0
        property string zeroPlaceholder: ""
        property string suffix: ""

        signal moved(real value)

        labelWidth: root.rowLabelWidth
        contentRightMargin: 74

        M3SpinBox {
            from: spinRow.from
            to: spinRow.to
            stepSize: spinRow.stepSize
            value: spinRow.value
            decimals: spinRow.decimals
            zeroPlaceholder: spinRow.zeroPlaceholder
            suffix: spinRow.suffix
            onMoved: (val) => spinRow.moved(val)
        }
    }

    Column {
        id: content
        width: parent.width
        spacing: Theme.space.xxl

        SettingsSection {
            label: qsTr("Window")
            width: parent.width

            SettingsRow {
                label: qsTr("Hide while playing")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.minimizeOnLaunch : false
                    onToggled: (val) => appSettings.applyMinimizeOnLaunch(val)
                }
            }

            SettingsRow {
                label: qsTr("Show tray icon")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.showTrayIcon : false
                    onToggled: (val) => appSettings.applyShowTrayIcon(val)
                }
            }
        }

        SettingsSection {
            label: qsTr("Runners")
            width: parent.width

            SettingsRow {
                label: qsTr("Ignore Steam runners")
                description: qsTr("Runners found in Steam's folders")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.ignoreSteamRunners : false
                    onToggled: (val) => appSettings.applyIgnoreSteamRunners(val)
                }
            }
        }

        SettingsSection {
            label: qsTr("Updates")
            width: parent.width

            SettingsRow {
                label: qsTr("Check EG games updates on run")
                description: qsTr("Might slowdown start times for Epic games")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.autoCheckEpicUpdatesOnLaunch : false
                    onToggled: (val) => appSettings.applyAutoCheckEpicUpdatesOnLaunch(val)
                }
            }

            SettingsRow {
                label: qsTr("Check GOG games updates on run")
                description: qsTr("Might slowdown start times for GOG games")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.autoCheckGogUpdatesOnLaunch : false
                    onToggled: (val) => appSettings.applyAutoCheckGogUpdatesOnLaunch(val)
                }
            }

            SettingsRow {
                label: qsTr("Check for updates on app launch")
                description: qsTr("Queues updates in the downloads page on startup")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.autoCheckUpdatesOnBoot : false
                    onToggled: (val) => appSettings.applyAutoCheckUpdatesOnBoot(val)
                }
            }
        }

        SettingsSection {
            label: qsTr("Discord")
            width: parent.width

            SettingsRow {
                label: qsTr("Show app name in Discord RPC")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.discordShowLauncher : true
                    onToggled: (val) => appSettings.applyDiscordShowLauncher(val)
                }
            }
        }

        SettingsSection {
            label: qsTr("Downloads")
            width: parent.width

            SpinRow {
                label: qsTr("Bandwidth limit")
                description: qsTr("Applies to every download, whatever the source.")
                width: parent.width
                from: 0
                to: 1000
                stepSize: 0.5
                decimals: 2
                suffix: "MB/s"
                zeroPlaceholder: qsTr("Unlimited")
                value: appSettings ? appSettings.bandwidthMbPerSec : 0
                onMoved: (val) => appSettings.applyBandwidthMbPerSec(val)
            }

            SettingsRow {
                label: qsTr("Notify when a download finishes")
                description: qsTr("Sends a desktop notification.")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.notifyOnDownloadComplete : true
                    onToggled: (val) => appSettings.applyNotifyOnDownloadComplete(val)
                }
            }
        }

        SettingsSection {
            label: qsTr("Epic downloads")
            width: parent.width

            SpinRow {
                label: qsTr("Download workers")
                description: qsTr("Auto lets legendary pick, which is min(2 x CPUs, 16).")
                width: parent.width
                from: 0
                to: 128
                zeroPlaceholder: qsTr("Auto")
                value: appSettings ? appSettings.epicWorkers : 0
                onMoved: (val) => appSettings.applyEpicWorkers(Math.round(val))
            }

            SpinRow {
                label: qsTr("Shared memory")
                description: qsTr("Buffer the workers download into. Auto is 1024.")
                width: parent.width
                from: 0
                to: 16384
                stepSize: 256
                suffix: "MiB"
                zeroPlaceholder: qsTr("Auto")
                value: appSettings ? appSettings.epicSharedMemoryMb : 0
                onMoved: (val) => appSettings.applyEpicSharedMemoryMb(Math.round(val))
            }
        }

        SettingsSection {
            label: qsTr("GOG downloads")
            width: parent.width

            SpinRow {
                label: qsTr("Download workers")
                description: qsTr("Auto lets gogdl pick.")
                width: parent.width
                from: 0
                to: 128
                zeroPlaceholder: qsTr("Auto")
                value: appSettings ? appSettings.gogWorkers : 0
                onMoved: (val) => appSettings.applyGogWorkers(Math.round(val))
            }
        }

        SettingsSection {
            label: qsTr("Gacha downloads")
            width: parent.width

            SpinRow {
                label: qsTr("Max connections")
                description: qsTr("How many files download at the same time.")
                width: parent.width
                from: 1
                to: 128
                value: appSettings ? appSettings.gachaConnections : 32
                onMoved: (val) => appSettings.applyGachaConnections(Math.round(val))
            }

            SpinRow {
                label: qsTr("HoYo patch threads")
                description: qsTr("Patches applied at once after an update. Raise it if patching feels slow.")
                width: parent.width
                from: 1
                to: 32
                value: appSettings ? appSettings.gachaPatchThreads : 4
                onMoved: (val) => appSettings.applyGachaPatchThreads(Math.round(val))
            }
        }

        SettingsSection {
            label: qsTr("Resources")
            width: parent.width

            SettingsRow {
                label: qsTr("Unload store tabs")
                description: qsTr("After 15s idle")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.unloadStorePages : true
                    onToggled: (val) => appSettings.applyUnloadStorePages(val)
                }
            }

            SettingsRow {
                label: qsTr("Save game logs to disk")
                description: qsTr("Off: logs live in memory only until the game exits. On: also written to cache/logs/.")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.saveGameLogs : false
                    onToggled: (val) => appSettings.applySaveGameLogs(val)
                }
            }
        }
    }
}
