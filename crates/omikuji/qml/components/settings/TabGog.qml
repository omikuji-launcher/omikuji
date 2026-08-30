pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

import "."
import "../controls"
import "../store"
import "../primitives"

Item {
    id: root

    property var config: ({})
    property var gameModel: null
    property string gameId: ""

    property var installedDlcs: []
    property real viewportHeight: 0

    readonly property bool isEmpty: installedDlcs.length === 0

    implicitHeight: root.isEmpty ? Math.max(root.viewportHeight, 240) : content.height

    function refreshDlcs() {
        if (!gameModel || gameId === "") { installedDlcs = []; return }
        try { installedDlcs = JSON.parse(gameModel.installed_dlcs(gameId) || "[]") }
        catch (e) { installedDlcs = [] }
    }

    onGameIdChanged: refreshDlcs()
    Component.onCompleted: refreshDlcs()

    EmptyState {
        anchors.fill: parent
        visible: root.isEmpty
        art: "qrc:/qt/qml/omikuji/qml/icons/dino.png"
        artSize: Math.round(Math.min(root.width * 0.75, 520))
        text: qsTr("well... seems there's nobody in here")
    }

    Column {
        id: content
        width: parent.width
        spacing: 20
        visible: !root.isEmpty

        SettingsSection {
            label: qsTr("DLC")
            icon: "layers"
            width: parent.width

            Column {
                width: parent.width
                spacing: Theme.space.md

                DlcPicker {
                    width: parent.width
                    readOnly: true
                    dlcs: root.installedDlcs
                }

                NoteChip {
                    width: parent.width
                    text: qsTr("gogdl cannot remove a single DLC. Dropping one means reinstalling the game without it. Fantastic, right?")
                }
            }
        }
    }
}
