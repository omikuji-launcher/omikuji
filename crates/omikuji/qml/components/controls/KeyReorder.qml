import QtQuick

QtObject {
    id: root

    property int index: -1
    property int count: 0
    property int rowStep: 1
    property bool horizontal: false

    property bool lifted: false
    property int liftedFrom: -1

    signal moveRequested(int from, int to)
    signal committed()

    function lift() {
        liftedFrom = index
        lifted = true
    }

    function step(delta) {
        const to = index + delta
        if (to >= 0 && to < count) moveRequested(index, to)
    }

    function drop(commit) {
        if (!lifted) return
        lifted = false
        if (index === liftedFrom) return
        if (commit) committed()
        else moveRequested(index, liftedFrom)
    }

    function handleShortcutOverride(event) {
        event.accepted = lifted && event.key === Qt.Key_Escape
    }

    function handleKey(event) {
        event.accepted = lifted
        if (!lifted) return
        switch (event.key) {
        case Qt.Key_Up: step(-rowStep); break
        case Qt.Key_Down: step(rowStep); break
        case Qt.Key_Left: if (horizontal) step(-1); break
        case Qt.Key_Right: if (horizontal) step(1); break
        case Qt.Key_Return:
        case Qt.Key_Enter:
        case Qt.Key_Space: drop(true); break
        case Qt.Key_Escape: drop(false); break
        case Qt.Key_Tab:
        case Qt.Key_Backtab:
            drop(false)
            event.accepted = false
            break
        }
    }
}
