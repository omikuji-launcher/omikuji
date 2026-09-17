import QtQuick
import QtQuick.Controls

QtObject {
    id: ret

    required property Popup popup

    property Item _prev: null

    readonly property Connections _watch: Connections {
        target: ret.popup
        function onAboutToShow() {
            ret._prev = ret.popup.parent ? ret.popup.parent.Window.activeFocusItem : null
        }
        function onClosed() {
            const prev = ret._prev
            ret._prev = null
            if (prev && prev.visible && prev.enabled) prev.forceActiveFocus()
        }
    }
}
