pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

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
                description: root.gameModel && root.gameModel.epic_overlay_is_installed()
                    ? qsTr("in-game overlay for friends, invites and achievements")
                    : qsTr("installs on first enable - one-time download")
                width: parent.width

                M3Switch {
                    checked: root.config["source.eos_overlay"] === true
                    onToggled: (val) => {
                        if (!root.gameModel || root.gameId === "") return
                        root.gameModel.epic_toggle_overlay(root.gameId, val)
                        root.refreshConfig()
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
                    checked: root.config["source.cloud_saves"] === true
                    onToggled: (val) => {
                        if (!root.gameModel || root.gameId === "") return
                        root.gameModel.epic_set_cloud_saves(root.gameId, val)
                        root.refreshConfig()
                    }
                }
            }

            Column {
                width: parent.width
                spacing: 8
                visible: root.config["source.cloud_saves"] === true

                Text {
                    text: (root.config["source.save_path"] || "") === ""
                        ? qsTr("No save path detected yet - a toast will appear when discovery finishes, or enter one manually below.")
                        : qsTr("Save path: %1").arg(root.config["source.save_path"])
                    color: Theme.textMuted
                    font.pixelSize: Theme.type.caption.size
                    wrapMode: Text.WordWrap
                    width: parent.width
                }

                M3TextField {
                    label: qsTr("Save Path Override")
                    placeholder: qsTr("leave empty to use detected path")
                    text: root.config["source.save_path"] || ""
                    width: parent.width
                    onTextEdited: (t) => root.updateField("source.save_path", t)
                }
            }
        }
    }
}
