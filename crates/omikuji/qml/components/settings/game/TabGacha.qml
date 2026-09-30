pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

import "../../lib/RunnerGrouping.js" as RG

Item {
    id: root

    property var config: ({})
    property var gameModel: null
    property var downloadModel: null
    property string gameId: ""

    property var packs: []
    property string packKind: ""
    property var checkedIds: []
    property var controls: []
    property var selection: ({})
    property string errorText: ""

    readonly property var installed: packs.filter(p => p.installed).map(root.asItem)
    readonly property var available: packs.filter(p => !p.installed).map(root.asItem)
    readonly property var downloading: packs.filter(p => p.downloading).map(p => p.id)
    readonly property var choiceControls: controls.filter(c => c.kind === "choice")
    readonly property var toggleControls: controls.filter(c => c.kind === "toggle")

    implicitHeight: content.height

    function asItem(pack) {
        return { id: pack.id, title: pack.label, short: pack.short }
    }

    function refresh() {
        if (!gameModel || gameId === "") { packs = []; return }
        let listing = {}
        try { listing = JSON.parse(gameModel.gacha_packs(gameId) || "{}") || {} }
        catch (e) { listing = {} }
        packs = listing.packs || []
        packKind = listing.kind || ""
        checkedIds = checkedIds.filter(id => available.some(p => p.id === id) && !downloading.includes(id))
        refreshControls()
    }

    function refreshControls() {
        let next = []
        if (gameModel) {
            try { next = JSON.parse(gameModel.gacha_launch_controls() || "[]") }
            catch (e) { next = [] }
        }
        let picked = {}
        for (const c of next) picked[c.id] = c.selected
        selection = picked
        if (controlsShape(next) !== controlsShape(controls)) controls = next
    }

    function controlsShape(list) {
        return JSON.stringify(list.map(c => [c.id, c.label, c.kind, c.choices]))
    }

    function select(controlId, choiceId) {
        errorText = gameModel.select_gacha_launch_choice(controlId, choiceId)
    }

    function choiceOptions(control) {
        const options = [{ label: qsTr("None"), value: "" }]
            .concat(control.choices.filter(c => c.available).map(c => ({ label: c.label, value: c.id })))
        return RG.withUnresolved(options, root.selection[control.id], { tint: Theme.error, missingLabel: qsTr("not installed") })
    }

    function installChecked() {
        for (const id of checkedIds) gameModel.add_gacha_pack(gameId, id)
        checkedIds = []
        refresh()
    }

    signal confirmRequested(string title, string message, string confirmText, var onConfirm)

    function requestRemove(id) {
        let removal = {}
        try { removal = JSON.parse(gameModel.gacha_pack_removal(id) || "{}") || {} }
        catch (e) { removal = {} }
        if (!removal.active) { remove(id); return }
        const pack = packs.find(p => p.id === id)
        root.confirmRequested(
            qsTr("Remove %1?").arg(pack ? pack.label : id),
            removal.fallback
                ? qsTr("Active pack switches to %1").arg(removal.fallback)
                : qsTr("No other pack is installed to switch to"),
            qsTr("Remove"),
            () => root.remove(id))
    }

    function remove(id) {
        errorText = gameModel.remove_gacha_pack(gameId, id)
        refresh()
    }

    onGameIdChanged: refresh()
    onConfigChanged: refreshControls()
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
            label: qsTr("Launch")
            icon: "tune"
            width: parent.width
            visible: root.controls.length > 0

            Column {
                width: parent.width
                spacing: Theme.space.md

                Repeater {
                    model: root.choiceControls

                    M3Dropdown {
                        required property var modelData
                        width: parent.width
                        label: modelData.label
                        options: root.choiceOptions(modelData)
                        currentIndex: Math.max(0, RG.indexOfValue(options, root.selection[modelData.id] || ""))
                        onSelected: (v) => root.select(modelData.id, v)
                    }
                }

                Repeater {
                    model: root.toggleControls

                    SwitchField {
                        required property var modelData
                        width: parent.width
                        label: modelData.label
                        description: modelData.choices[0].description
                        checked: !!root.selection[modelData.id]
                        onToggled: (val) => root.select(modelData.id, val ? modelData.id : "")
                    }
                }
            }
        }

        SettingsSection {
            visible: root.packs.length > 0
            label: PackLabels.section(root.packKind)
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
                    onRemoveRequested: (id) => root.requestRemove(id)
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
            }
        }

        NoteChip {
            width: parent.width
            visible: root.errorText !== ""
            text: root.errorText
        }
    }
}
