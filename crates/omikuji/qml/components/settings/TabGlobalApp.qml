import QtQuick
import omikuji 1.0

import "."
import "../controls"

Item {
    id: root

    property var appSettings: null

    readonly property int rowLabelWidth: 200

    implicitHeight: content.height

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
