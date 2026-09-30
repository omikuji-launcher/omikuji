pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Item {
    id: root

    property var config: ({})
    property var gameModel: null
    property var downloadModel: null
    property string gameId: ""

    property var packs: []
    property var checkedIds: []
    property string errorText: ""

    readonly property var installed: packs.filter(p => p.installed).map(root.asItem)
    readonly property var available: packs.filter(p => !p.installed).map(root.asItem)
    readonly property var downloading: packs.filter(p => p.downloading).map(p => p.id)

    implicitHeight: content.height

    function asItem(pack) {
        return { id: pack.id, title: pack.label }
    }

    function refresh() {
        if (!gameModel || gameId === "") { packs = []; return }
        try { packs = JSON.parse(gameModel.gacha_packs(gameId) || "[]") }
        catch (e) { packs = [] }
        checkedIds = checkedIds.filter(id => available.some(p => p.id === id) && !downloading.includes(id))
    }

    function installChecked() {
        for (const id of checkedIds) gameModel.add_gacha_pack(gameId, id)
        checkedIds = []
        refresh()
    }

    function remove(id) {
        errorText = gameModel.remove_gacha_pack(gameId, id)
        refresh()
    }

    onGameIdChanged: refresh()
    Component.onCompleted: refresh()

    Connections {
        target: root.downloadModel
        function onDownload_completed(id, source, appId) {
            if (appId === root.config["source.app_id"]) root.refresh()
        }
        function onDownload_failed() { root.refresh() }
        function onRowsRemoved() { root.refresh() }
    }

    Column {
        id: content
        width: parent.width
        spacing: 20

        SettingsSection {
            label: qsTr("Packs")
            icon: "layers"
            width: parent.width

            Column {
                width: parent.width
                spacing: Theme.space.md

                ArtCheckList {
                    width: parent.width
                    visible: root.installed.length > 0
                    title: qsTr("Installed")
                    readOnly: true
                    removable: true
                    items: root.installed
                    onRemoveRequested: (id) => root.remove(id)
                }

                ArtCheckList {
                    width: parent.width
                    visible: root.available.length > 0
                    title: qsTr("Available")
                    items: root.available
                    checkedIds: root.checkedIds
                    lockedIds: root.downloading
                    lockedHint: qsTr("Already downloading. Check the Downloads tab")
                    onSelectionRequested: (ids) => root.checkedIds = ids
                }

                M3Button {
                    visible: root.available.length > 0
                    enabled: root.checkedIds.length > 0
                    text: qsTr("Install selected")
                    variant: "tonal"
                    onClicked: root.installChecked()
                }

                NoteChip {
                    width: parent.width
                    visible: root.errorText !== ""
                    text: root.errorText
                }
            }
        }
    }
}
