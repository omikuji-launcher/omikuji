import QtQuick
import QtQuick.Layouts

import "."
import "../controls"

// these toggles have side-effects beyond toml writes, so refreshConfig instead of updateField
Item {
    id: root

    property var config: ({})
    property var updateField: function(key, value) {}
    property var refreshConfig: function() {}
    property var gameModel: null
    property string gameId: ""

    implicitHeight: content.height

    Column {
        id: content
        width: parent.width
        spacing: 20

        SettingsSection {
            label: "Epic Online Services"
            icon: "verified"
            width: parent.width

            SettingsRow {
                label: "EOS Overlay"
                description: gameModel && gameModel.epic_overlay_is_installed()
                    ? qsTr("in-game overlay for friends, invites and achievements")
                    : qsTr("installs on first enable - one-time download")
                width: parent.width

                M3Switch {
                    checked: config["source.eos_overlay"] === true
                    onToggled: (val) => {
                        if (!gameModel || gameId === "") return
                        gameModel.epic_toggle_overlay(gameId, val)
                        refreshConfig()
                    }
                }
            }
        }

        SettingsSection {
            label: qsTr("Cloud Saves")
            icon: "sync"
            width: parent.width

            SettingsRow {
                label: qsTr("Auto-sync")
                description: qsTr("download before launch, upload after exit")
                width: parent.width

                M3Switch {
                    checked: config["source.cloud_saves"] === true
                    onToggled: (val) => {
                        if (!gameModel || gameId === "") return
                        gameModel.epic_set_cloud_saves(gameId, val)
                        refreshConfig()
                    }
                }
            }

            Column {
                width: parent.width
                spacing: 8
                visible: config["source.cloud_saves"] === true

                Text {
                    text: (config["source.save_path"] || "") === ""
                        ? qsTr("No save path detected yet - a toast will appear when discovery finishes, or enter one manually below.")
                        : qsTr("Save path: %1").arg(config["source.save_path"])
                    color: theme.textMuted
                    font.pixelSize: theme.type.caption.size
                    wrapMode: Text.WordWrap
                    width: parent.width
                }

                M3TextField {
                    label: qsTr("Save Path Override")
                    placeholder: qsTr("leave empty to use detected path")
                    text: config["source.save_path"] || ""
                    width: parent.width
                    onTextEdited: (t) => updateField("source.save_path", t)
                }
            }
        }
    }
}
