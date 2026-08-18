import QtQuick
import "../dialogs"

Item {
    id: ctrl

    anchors.fill: parent
    // hoists the nested dialogs above nav/topbar so their dim covers the whole window
    z: 2000

    property var uiSettings: null
    property var gameModel: null

    property int _pendingGameIdx: -1

    function showAdd() { editDialog.showAdd() }
    function showEdit(index, entry) { editDialog.showEdit(index, entry) }

    function showDelete(index, entry) {
        deleteConfirm.pendingIndex = index
        deleteConfirm.message = qsTr("Delete \"%1\" from your categories?").arg(entry.name || "")
        deleteConfirm.show()
    }

    function showForGame(gameIndex) { gameDialog.show(gameIndex) }

    function setEnabled(index, value) {
        let entries = ctrl._readEntries()
        if (index < 0 || index >= entries.length) return
        entries[index].enabled = value
        ctrl._applyEntries(entries)
    }

    function _readEntries() {
        if (!ctrl.uiSettings) return []
        try { return JSON.parse(ctrl.uiSettings.categoriesJson()) }
        catch (e) { return [] }
    }

    function _applyEntries(entries) {
        if (ctrl.uiSettings) ctrl.uiSettings.applyCategoriesJson(JSON.stringify(entries))
    }

    CategoryEditDialog {
        id: editDialog
        anchors.fill: parent
        onSaved: (entry, idx) => {
            let entries = ctrl._readEntries()
            if (idx === -1) entries.push(entry)
            else if (idx >= 0 && idx < entries.length) entries[idx] = entry
            ctrl._applyEntries(entries)
        }
        onClosed: {
            if (ctrl._pendingGameIdx >= 0) {
                let idx = ctrl._pendingGameIdx
                ctrl._pendingGameIdx = -1
                gameDialog.show(idx)
            }
        }
    }

    GameCategoriesDialog {
        id: gameDialog
        anchors.fill: parent
        gameModel: ctrl.gameModel
        uiSettings: ctrl.uiSettings
        onRequestNewCategory: {
            ctrl._pendingGameIdx = gameDialog.gameIndex
            gameDialog.hide()
            editDialog.showAdd()
        }
    }

    ConfirmDialog {
        id: deleteConfirm
        anchors.fill: parent
        property int pendingIndex: -1
        title: qsTr("Delete category")
        confirmText: qsTr("Delete")
        cancelText: qsTr("Keep")
        destructive: true
        onConfirmed: {
            let idx = deleteConfirm.pendingIndex
            if (idx < 0) return
            let entries = ctrl._readEntries()
            if (idx < entries.length) entries.splice(idx, 1)
            ctrl._applyEntries(entries)
            deleteConfirm.pendingIndex = -1
        }
    }
}
