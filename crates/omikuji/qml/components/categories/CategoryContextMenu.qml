import QtQuick
import "../popups"

Item {
    id: ctrl
    anchors.fill: parent
    z: 2000

    property var appSettings: null

    signal addRequested()
    signal editRequested(int index, var entry)
    signal deleteRequested(int index, var entry)
    signal hideRequested(int index)

    function show(index, x, y) {
        ctrl._pendingIndex = index
        ctrl._pendingX = x
        ctrl._pendingY = y
        menu.close()
        delayTimer.start()
    }

    property int _pendingIndex: -1
    property real _pendingX: 0
    property real _pendingY: 0

    function _entry(index) {
        if (!ctrl.appSettings || index < 0) return null
        let entries = []
        try { entries = JSON.parse(ctrl.appSettings.categoriesJson()) } catch (e) { entries = [] }
        return index < entries.length ? entries[index] : null
    }

    Timer {
        id: delayTimer
        interval: 100
        onTriggered: menu.setPosition(ctrl._pendingIndex, ctrl._pendingX, ctrl._pendingY)
    }

    ContextMenu {
        id: menu
        property int currentIndex: -1

        function setPosition(index, mouseX, mouseY) {
            if (!ctrl._entry(index)) return
            items = [
                { text: qsTr("New category"), action: "add" },
                { text: qsTr("Edit"), action: "edit" },
                { text: qsTr("Hide"), action: "hide" },
                { text: qsTr("Delete"), action: "delete", danger: true }
            ]
            currentIndex = index
            openAtCursor(mouseX, mouseY)
        }

        onItemClicked: (action) => {
            let idx = menu.currentIndex
            if (action === "add") {
                ctrl.addRequested()
                return
            }
            let entry = ctrl._entry(idx)
            if (!entry) return
            switch (action) {
                case "edit":
                    ctrl.editRequested(idx, entry)
                    break
                case "hide":
                    ctrl.hideRequested(idx)
                    break
                case "delete":
                    ctrl.deleteRequested(idx, entry)
                    break
            }
        }
    }
}
