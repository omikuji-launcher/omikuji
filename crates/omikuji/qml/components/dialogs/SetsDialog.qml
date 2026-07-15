import QtQuick
import QtQuick.Layouts
import "../controls"
import "../primitives"

DialogCard {
    sizeKey: "sets"
    id: root

    property var libRead: function() { return "[]" }
    property var libWrite: function(json) {}
    property var applyFn: null
    property var baseEnv: ({})

    property string copyKey: "launch.env"
    property string syncKey: "launch.env_sets"
    property string keyPlaceholder: "KEY"
    property string valuePlaceholder: "value"
    property string titleText: "Sets"
    property string manageTitle: "Manage sets"

    property var sets: []
    property var checkedIds: []
    property var syncedIds: []
    property bool manageOnly: false
    property int editingIndex: -1
    property string editName: ""

    maxWidth: 600
    title: root.editingIndex >= 0 ? "Edit set" : (root.manageOnly ? root.manageTitle : root.titleText)

    function _genId() {
        return (Math.random().toString(36) + "000000").slice(2, 8)
    }

    function openForGame(mapJson, syncedJson, fn) {
        root.manageOnly = false
        root.applyFn = fn
        try { root.baseEnv = JSON.parse(mapJson || "{}") }
        catch (e) { root.baseEnv = ({}) }
        try { root.syncedIds = JSON.parse(syncedJson || "[]") }
        catch (e) { root.syncedIds = [] }
        root.checkedIds = []
        root.editingIndex = -1
        root._load()
        root.open()
    }

    function openManage() {
        root.manageOnly = true
        root.applyFn = null
        root.checkedIds = []
        root.syncedIds = []
        root.editingIndex = -1
        root._load()
        root.open()
    }

    function _load() {
        let arr = []
        try { arr = JSON.parse(root.libRead()) }
        catch (e) { arr = [] }
        let changed = false
        for (let i = 0; i < arr.length; i++) {
            if (!arr[i].id) { arr[i].id = root._genId(); changed = true }
        }
        root.sets = arr
        if (changed) root._persist()
    }

    function _persist() {
        root.libWrite(JSON.stringify(root.sets))
    }

    function _persistSynced() {
        if (root.applyFn) root.applyFn(root.syncKey, JSON.stringify(root.syncedIds))
    }

    function _toggleCheck(id) {
        let c = root.checkedIds.slice()
        let i = c.indexOf(id)
        if (i === -1) c.push(id)
        else c.splice(i, 1)
        root.checkedIds = c
    }

    function _toggleSync(id) {
        let sn = root.syncedIds.slice()
        let i = sn.indexOf(id)
        if (i === -1) sn.push(id)
        else sn.splice(i, 1)
        root.syncedIds = sn
        root._persistSynced()
    }

    function _enterEdit(i) {
        if (i < 0 || i >= root.sets.length) return
        root.editName = root.sets[i].name || ""
        editVars.clear()
        let vars = root.sets[i].vars || []
        for (let j = 0; j < vars.length; j++)
            editVars.append({ key: vars[j].key || "", value: vars[j].value || "" })
        root.editingIndex = i
    }

    function _commitEdit() {
        if (root.editingIndex < 0) return
        let vars = []
        for (let i = 0; i < editVars.count; i++) {
            let r = editVars.get(i)
            vars.push({ key: r.key, value: r.value })
        }
        let s = root.sets.slice()
        s[root.editingIndex] = { id: s[root.editingIndex].id, name: root.editName, vars: vars }
        root.sets = s
        root._persist()
    }

    function _doneEdit() {
        root._commitEdit()
        root.editingIndex = -1
    }

    function _newSet() {
        let s = root.sets.slice()
        s.push({ id: root._genId(), name: "New set", vars: [] })
        root.sets = s
        root._persist()
        root._enterEdit(s.length - 1)
    }

    function _deleteSet() {
        if (root.editingIndex < 0) return
        let id = root.sets[root.editingIndex].id
        let s = root.sets.slice()
        s.splice(root.editingIndex, 1)
        root.sets = s
        let c = root.checkedIds.slice()
        let ci = c.indexOf(id)
        if (ci !== -1) { c.splice(ci, 1); root.checkedIds = c }
        let sn = root.syncedIds.slice()
        let si = sn.indexOf(id)
        if (si !== -1) { sn.splice(si, 1); root.syncedIds = sn; root._persistSynced() }
        root.editingIndex = -1
        root._persist()
    }

    function _add() {
        let merged = ({})
        for (let k in root.baseEnv) merged[k] = root.baseEnv[k]
        for (let i = 0; i < root.sets.length; i++) {
            if (root.checkedIds.indexOf(root.sets[i].id) === -1) continue
            let vars = root.sets[i].vars || []
            for (let j = 0; j < vars.length; j++) {
                let key = String(vars[j].key || "").trim()
                if (key === "") continue
                merged[key] = String(vars[j].value || "")
            }
        }
        if (root.applyFn) root.applyFn(root.copyKey, JSON.stringify(merged))
        root.close()
    }

    onCloseRequested: {
        if (root.editingIndex >= 0) root._doneEdit()
        else root.close()
    }

    ListModel { id: editVars }

    body: ColumnLayout {
        width: parent.width
        spacing: theme.space.sm

        ColumnLayout {
            Layout.fillWidth: true
            spacing: theme.space.sm
            visible: root.editingIndex < 0

            Text {
                Layout.fillWidth: true
                text: root.manageOnly
                    ? qsTr("Create and edit reusable sets. Apply them per-game from a game's settings.")
                    : qsTr("Check + Add to copy a set's entries into this game. Sync applies a set live at launch, atop this game's values.")
                color: theme.textMuted
                font.pixelSize: theme.type.caption.size
                wrapMode: Text.Wrap
            }

            Text {
                Layout.fillWidth: true
                text: qsTr("No sets yet. Create one to reuse values across games.")
                color: theme.textSubtle
                font.pixelSize: theme.type.caption.size
                wrapMode: Text.Wrap
                visible: root.sets.length === 0
            }

            Repeater {
                model: root.sets

                delegate: Item {
                    id: setRow
                    required property var modelData
                    required property int index

                    Layout.fillWidth: true
                    Layout.preferredHeight: 46

                    readonly property bool selected: root.checkedIds.indexOf(modelData.id) !== -1
                    readonly property bool synced: root.syncedIds.indexOf(modelData.id) !== -1

                    Rectangle {
                        anchors.fill: parent
                        radius: theme.radius.sm
                        color: rowHover.containsMouse ? theme.alpha(theme.text, 0.06) : "transparent"
                        Behavior on color { ColorAnimation { duration: 100 } }
                    }

                    MouseArea {
                        id: rowHover
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.manageOnly ? root._enterEdit(setRow.index) : root._toggleCheck(setRow.modelData.id)
                    }

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 10
                        anchors.rightMargin: 6
                        spacing: theme.space.md

                        SvgIcon {
                            visible: !root.manageOnly
                            name: setRow.selected ? "check_box" : "check_box_outline_blank"
                            size: 20
                            color: setRow.selected ? theme.accent : theme.alpha(theme.text, 0.55)
                        }

                        Text {
                            Layout.fillWidth: true
                            text: setRow.modelData.name
                            color: theme.text
                            font.pixelSize: 14
                            elide: Text.ElideRight
                        }

                        Text {
                            text: qsTr("%1 vars").arg(setRow.modelData.vars ? setRow.modelData.vars.length : 0)
                            color: theme.textSubtle
                            font.pixelSize: theme.type.caption.size
                        }

                        M3Button {
                            visible: !root.manageOnly
                            text: setRow.synced ? qsTr("Unsync") : qsTr("Sync")
                            variant: "tonal"
                            danger: setRow.synced
                            onClicked: root._toggleSync(setRow.modelData.id)
                        }

                        IconButton {
                            icon: "edit"
                            onClicked: root._enterEdit(setRow.index)
                        }
                    }
                }
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: theme.space.md
            visible: root.editingIndex >= 0

            M3TextField {
                Layout.fillWidth: true
                label: qsTr("Set name")
                text: root.editName
                onTextEdited: (t) => root.editName = t
            }

            Text {
                text: qsTr("Variables")
                color: theme.textMuted
                font.pixelSize: theme.type.body.size
                font.weight: Font.Medium
            }

            Text {
                Layout.fillWidth: true
                text: qsTr("No variables yet.")
                color: theme.textSubtle
                font.pixelSize: theme.type.caption.size
                visible: editVars.count === 0
            }

            Repeater {
                model: editVars

                delegate: RowLayout {
                    required property int index
                    required property string key
                    required property string value

                    Layout.fillWidth: true
                    spacing: theme.space.sm

                    M3TextField {
                        Layout.fillWidth: true
                        Layout.preferredWidth: 1
                        placeholder: root.keyPlaceholder
                        text: key
                        onTextEdited: (t) => editVars.setProperty(index, "key", t)
                    }

                    M3TextField {
                        Layout.fillWidth: true
                        Layout.preferredWidth: 1
                        placeholder: root.valuePlaceholder
                        text: value
                        onTextEdited: (t) => editVars.setProperty(index, "value", t)
                    }

                    IconButton {
                        icon: "close"
                        danger: true
                        onClicked: editVars.remove(index)
                    }
                }
            }

            M3Button {
                text: qsTr("Add variable")
                variant: "tonal"
                icon: "add"
                onClicked: editVars.append({ key: "", value: "" })
            }
        }
    }

    footerLeft: Row {
        spacing: theme.space.sm

        M3Button {
            visible: root.editingIndex < 0
            text: qsTr("New set")
            variant: "tonal"
            icon: "add"
            onClicked: root._newSet()
        }

        M3Button {
            visible: root.editingIndex >= 0
            text: qsTr("Delete set")
            variant: "tonal"
            danger: true
            icon: "remove"
            onClicked: root._deleteSet()
        }
    }

    actions: Row {
        spacing: theme.space.sm

        M3Button {
            visible: root.editingIndex >= 0
            text: qsTr("Done")
            variant: "filled"
            onClicked: root._doneEdit()
        }

        M3Button {
            visible: root.editingIndex < 0
            text: qsTr("Close")
            variant: "text"
            onClicked: root.close()
        }

        M3Button {
            visible: root.editingIndex < 0 && !root.manageOnly
            text: qsTr("Add")
            variant: "filled"
            enabled: root.checkedIds.length > 0
            onClicked: root._add()
        }
    }
}
