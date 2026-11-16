pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Item {
    id: root

    property var config: ({})
    property var updateField: function(key, value) {}
    property var gameModel: null
    property string gameId: ""

    property var installedDlcs: []

    implicitHeight: content.height

    function refreshDlcs() {
        if (!gameModel || gameId === "") { installedDlcs = []; return }
        try { installedDlcs = JSON.parse(gameModel.installed_dlcs(gameId) || "[]") }
        catch (e) { installedDlcs = [] }
    }

    onGameIdChanged: refreshDlcs()
    Component.onCompleted: refreshDlcs()

    Column {
        id: content
        width: parent.width
        spacing: 20

        SettingsSection {
            label: "GOG Galaxy"
            icon: "verified"
            width: parent.width

            SwitchField {
                width: parent.width
                label: "Comet"
                description: qsTr("Syncs achievements and stats to your GOG profile")
                checked: root.config["source.comet"] === true
                onToggled: (val) => root.updateField("source.comet", val)
            }
        }

        SettingsSection {
            label: qsTr("DLC")
            icon: "layers"
            width: parent.width
            visible: root.installedDlcs.length > 0

            Column {
                width: parent.width
                spacing: Theme.space.md

                ArtCheckList {
                    width: parent.width
                    readOnly: true
                    items: root.installedDlcs
                }

                NoteChip {
                    width: parent.width
                    text: qsTr("gogdl cannot remove a single DLC. Dropping one means reinstalling the game without it. Fantastic, right?")
                }
            }
        }
    }
}
