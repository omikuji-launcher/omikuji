pragma Singleton
import QtQuick

QtObject {
    id: stack

    property var entries: []
    // stays top over dialogs opened after it
    property var grabbed: null

    readonly property var top: grabbed ? grabbed : (entries.length > 0 ? entries[entries.length - 1] : null)

    function push(item) {
        if (entries.indexOf(item) !== -1) return
        entries = entries.concat([item])
    }

    function pop(item) {
        const i = entries.indexOf(item)
        if (i === -1) return
        const next = entries.slice()
        next.splice(i, 1)
        entries = next
    }
}
