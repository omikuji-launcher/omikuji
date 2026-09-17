import QtQuick
import QtQuick.Controls
import omikuji 1.0
import "../lib/Nav.js" as Nav

Item {
    id: trap

    required property Item host
    required property Item scope
    property var sections: []
    property bool active: false

    readonly property bool onTop: active && OverlayStack.top === host

    property Item _prevFocus: null
    property Item _lastInside: null
    property var _lastRect: null

    function _currentFocus() {
        return Window.window ? Window.window.activeFocusItem : null
    }

    function handleKey(event) {
        const f = _currentFocus()
        event.accepted = onTop && Nav.hostKey(event, scope, sections, f, f === scope ? _lastRect : null)
    }

    function reset() {
        _lastInside = null
        _lastRect = null
    }

    function _grab() {
        if (active && scope && scope.visible) scope.forceActiveFocus()
    }

    function _track() {
        const f = _currentFocus()
        if (!onTop || !f || f === scope || Nav.contains(Overlay.overlay, f)) return
        const owner = Nav.contains(scope, f) ? Nav.navigableOwner(f, scope) : null
        if (owner) {
            _lastInside = owner
            _lastRect = Nav.sourceRect(owner)
            Nav.revealAncestors(owner, scope)
        } else {
            Qt.callLater(_returnFocus)
        }
    }

    function _returnFocus() {
        if (onTop && Nav.focusLost(scope, _currentFocus(), _lastInside)) Nav.guardReturn(scope, _lastInside)
    }

    onActiveChanged: {
        if (active) {
            _prevFocus = _currentFocus()
            _grab()
            return
        }
        if (_prevFocus && _prevFocus.visible) _prevFocus.forceActiveFocus()
        _prevFocus = null
        reset()
    }

    Connections {
        target: trap.scope
        function onVisibleChanged() { trap._grab() }
    }

    Connections {
        target: trap.Window.window
        enabled: trap.active
        function onActiveFocusItemChanged() { trap._track() }
    }

    Connections {
        target: trap._lastInside
        enabled: trap.onTop
        function onVisibleChanged() { Qt.callLater(trap._returnFocus) }
        function onEnabledChanged() { Qt.callLater(trap._returnFocus) }
        function onOpacityChanged() { Qt.callLater(trap._returnFocus) }
    }
}
